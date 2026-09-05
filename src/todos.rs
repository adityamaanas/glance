//! User-owned reminders, serialized under an advisory lock shared by panel and CLI.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    Pending,
    InProgress,
    Done,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub text: String,
    pub status: Status,
    pub created_at: u64,
    pub status_by: String,
    pub revision: u64,
    pub manual_after_turn: usize,
    pub manual_fingerprint: u64,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub source_turns: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Store {
    version: u32,
    next_id: u64,
    pub items: Vec<Todo>,
}
impl Default for Store {
    fn default() -> Self {
        Self {
            version: 1,
            next_id: 1,
            items: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Update {
    pub id: String,
    pub status: Status,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub source_turns: Vec<usize>,
}

pub fn path(session: &str) -> Result<PathBuf> {
    crate::transcript::validate_session_id(session)?;
    Ok(crate::summary::state_dir()?.join(format!("{session}.todos.json")))
}

pub fn load(path: &Path) -> Result<Store> {
    match std::fs::read(path) {
        Ok(bytes) => {
            let store: Store =
                serde_json::from_slice(&bytes).context("invalid todo file; original preserved")?;
            if store.version != 1 {
                bail!("unsupported todo file version {}", store.version);
            }
            Ok(store)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Store::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn edit(path: &Path, change: impl FnOnce(&mut Store) -> Result<()>) -> Result<Store> {
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path.with_extension("lock"))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match fs2::FileExt::try_lock_exclusive(&lock) {
            Ok(()) => break,
            Err(e)
                if e.raw_os_error() == fs2::lock_contended_error().raw_os_error()
                    && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(20))
            }
            Err(e) => return Err(e).context("todo file is busy"),
        }
    }
    let mut store = load(path)?;
    change(&mut store)?;
    crate::setup::atomic_write(path, &serde_json::to_vec_pretty(&store)?)?;
    // Dropping the dedicated lock handle releases it even if parsing or writing failed.
    Ok(store)
}

impl Store {
    pub fn add(&mut self, text: &str, turns: usize, fingerprint: u64) -> Result<()> {
        let text = text.trim();
        if text.is_empty() || text.chars().count() > 500 || text.chars().any(char::is_control) {
            bail!("todo must be a single line of 1–500 characters");
        }
        if self.items.len() >= 100 {
            bail!("at most 100 personal todos per session");
        }
        let id = format!("todo-{}", self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .context("todo ID limit reached")?;
        self.items.push(Todo {
            id,
            text: text.into(),
            status: Status::Pending,
            created_at: crate::summary::now_secs(),
            status_by: "user".into(),
            revision: 0,
            manual_after_turn: turns,
            manual_fingerprint: fingerprint,
            note: String::new(),
            source_turns: vec![],
        });
        Ok(())
    }

    pub fn set(&mut self, id: &str, status: Status, turns: usize, fingerprint: u64) -> Result<()> {
        let item = self
            .items
            .iter_mut()
            .find(|t| t.id == id)
            .context("unknown todo ID")?;
        item.status = status;
        item.revision += 1;
        item.status_by = "user".into();
        item.manual_after_turn = turns;
        item.manual_fingerprint = fingerprint;
        item.note.clear();
        item.source_turns.clear();
        Ok(())
    }

    pub fn delete(&mut self, id: &str) -> Result<()> {
        let index = self
            .items
            .iter()
            .position(|t| t.id == id)
            .context("unknown todo ID")?;
        self.items.remove(index);
        Ok(())
    }

    pub fn apply(
        &mut self,
        snapshot: &[Todo],
        updates: &[Update],
        tr: &crate::transcript::Transcript,
        end: usize,
    ) {
        for update in updates {
            let Some(item) = self.items.iter_mut().find(|t| t.id == update.id) else {
                continue;
            };
            let Some(original) = snapshot.iter().find(|t| t.id == item.id) else {
                continue;
            };
            if original.revision != item.revision
                || original.text != item.text
                || item.manual_after_turn > tr.turns.len()
                || tr.fingerprint(item.manual_after_turn) != item.manual_fingerprint
            {
                continue;
            }
            let mut sources = update.source_turns.clone();
            sources.retain(|n| *n >= item.manual_after_turn && *n < end && *n < tr.turns.len());
            sources.sort_unstable();
            sources.dedup();
            sources.truncate(8);
            if sources.is_empty() {
                continue;
            }
            item.status = update.status;
            item.status_by = "model".into();
            item.note = crate::transcript::clip(&update.note, 160)
                .chars()
                .filter(|c| !c.is_control())
                .collect();
            item.source_turns = sources;
            item.revision += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simultaneous_writers_keep_every_item_and_corruption_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.todos.json");
        let workers: Vec<_> = (0..12)
            .map(|n| {
                let p = path.clone();
                std::thread::spawn(move || {
                    edit(&p, |s| s.add(&format!("Reminder {n}"), 0, 0)).unwrap()
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
        let store = load(&path).unwrap();
        assert_eq!(store.items.len(), 12);
        let ids: std::collections::HashSet<_> = store.items.iter().map(|t| &t.id).collect();
        assert_eq!(ids.len(), 12);
        std::fs::write(&path, "{broken").unwrap();
        assert!(edit(&path, |s| s.add("test", 0, 0)).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{broken");
    }

    #[test]
    fn manual_changes_deleted_items_and_old_evidence_win_over_model() {
        let mut tr = crate::transcript::Transcript::open(Path::new("unused"));
        tr.turns = vec![crate::transcript::Turn::User("start".into())];
        let mut store = Store::default();
        store
            .add("Keep my exact words", 0, tr.fingerprint(0))
            .unwrap();
        let snapshot = store.items.clone();
        let update = Update {
            id: "todo-1".into(),
            status: Status::Done,
            note: "finished".into(),
            source_turns: vec![0, 99],
        };
        store
            .set("todo-1", Status::Pending, 1, tr.fingerprint(1))
            .unwrap();
        store.apply(&snapshot, std::slice::from_ref(&update), &tr, 1);
        assert_eq!(store.items[0].status, Status::Pending);
        let snapshot = store.items.clone();
        store.apply(&snapshot, std::slice::from_ref(&update), &tr, 1);
        assert_eq!(store.items[0].status, Status::Pending);
        tr.turns
            .push(crate::transcript::Turn::Assistant("finished".into()));
        let update = Update {
            source_turns: vec![1],
            ..update
        };
        store.apply(&snapshot, std::slice::from_ref(&update), &tr, 2);
        assert_eq!(store.items[0].status, Status::Done);
        assert_eq!(store.items[0].text, "Keep my exact words");
        store.delete("todo-1").unwrap();
        store.apply(&snapshot, &[update], &tr, 2);
        assert!(store.items.is_empty());
    }
}
