//! Bounded, read-only session discovery and a terminal-friendly picker.
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::io::{BufRead, IsTerminal, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct Session {
    pub id: String,
    pub path: PathBuf,
    pub cwd: Option<String>,
    pub title: String,
    pub modified: u64,
}

fn same_path(a: &Path, b: &Path) -> bool {
    let a = std::fs::canonicalize(a).unwrap_or_else(|_| a.to_path_buf());
    let b = std::fs::canonicalize(b).unwrap_or_else(|_| b.to_path_buf());
    #[cfg(windows)]
    {
        a.to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        a == b
    }
}

pub fn sessions(cwd: Option<&Path>, query: Option<&str>) -> Result<Vec<Session>> {
    scan(
        &crate::transcript::claude_dir()?.join("projects"),
        cwd,
        query,
    )
}

fn scan(root: &Path, cwd: Option<&Path>, query: Option<&str>) -> Result<Vec<Session>> {
    let projects = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e).context("read session projects"),
    };
    let mut sessions = Vec::new();
    for project in projects.flatten() {
        if !project.file_type().is_ok_and(|t| t.is_dir()) {
            continue;
        }
        let Ok(files) = std::fs::read_dir(project.path()) else {
            continue;
        };
        for file in files.flatten() {
            if !file.file_type().is_ok_and(|t| t.is_file()) {
                continue;
            }
            let path = file.path();
            if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if crate::transcript::validate_session_id(id).is_err() {
                continue;
            }
            let Ok(mut input) = std::fs::File::open(&path) else {
                continue;
            };
            let metadata = input.metadata()?;
            let mut data = Vec::new();
            Read::by_ref(&mut input)
                .take(65536)
                .read_to_end(&mut data)?;
            if metadata.len() > 65536 {
                data.push(b'\n');
                input.seek(SeekFrom::Start(metadata.len().saturating_sub(65536)))?;
                Read::by_ref(&mut input)
                    .take(65536)
                    .read_to_end(&mut data)?;
            }
            let mut session = Session {
                id: id.into(),
                path: path.clone(),
                cwd: None,
                title: id.into(),
                modified: metadata
                    .modified()?
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            for line in data.split(|b| *b == b'\n') {
                let Ok(v) = serde_json::from_slice::<serde_json::Value>(line) else {
                    continue;
                };
                if let Some(cwd) = v["cwd"].as_str() {
                    session.cwd = Some(cwd.into());
                }
                if let Some(title) = v["customTitle"].as_str().or(v["aiTitle"].as_str()) {
                    session.title = title.into();
                } else if session.title == id {
                    if let Some(text) = v.pointer("/message/content").and_then(|v| v.as_str()) {
                        session.title = text.into();
                    }
                }
            }
            session.title = crate::transcript::clip(
                &session
                    .title
                    .chars()
                    .filter(|c| !c.is_control())
                    .collect::<String>(),
                90,
            );
            if cwd.is_some_and(|wanted| {
                !session
                    .cwd
                    .as_ref()
                    .is_some_and(|c| same_path(Path::new(c), wanted))
            }) {
                continue;
            }
            if query.is_some_and(|q| {
                !format!(
                    "{} {} {}",
                    session.id,
                    session.title,
                    session.cwd.as_deref().unwrap_or("")
                )
                .to_lowercase()
                .contains(&q.to_lowercase())
            }) {
                continue;
            }
            sessions.push(session);
        }
    }
    sessions.sort_by(|a, b| b.modified.cmp(&a.modified).then_with(|| a.id.cmp(&b.id)));
    Ok(sessions)
}

pub fn pick(cwd: Option<&Path>, query: Option<&str>, latest: bool) -> Result<String> {
    let sessions = sessions(cwd, query)?;
    if sessions.is_empty() {
        bail!("no matching sessions; start a conversation first");
    }
    if latest {
        return Ok(sessions[0].id.clone());
    }
    if !std::io::stdin().is_terminal() {
        bail!("interactive picker needs a terminal; use pick --latest or pick --list");
    }
    for (n, s) in sessions.iter().take(30).enumerate() {
        eprintln!(
            "{:>2}. {} · {} · {}",
            n + 1,
            s.title,
            s.cwd.as_deref().unwrap_or("unknown project"),
            s.id
        );
    }
    eprint!("Choose 1–{} (empty cancels): ", sessions.len().min(30));
    std::io::stderr().flush()?;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    let number: usize = line
        .trim()
        .parse()
        .context("selection cancelled or invalid")?;
    sessions
        .get(number.wrapping_sub(1))
        .filter(|_| number <= 30)
        .map(|s| s.id.clone())
        .context("selection out of range")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discovery_filters_real_cwd_ignores_subagents_and_cleans_terminal_text() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("slug");
        std::fs::create_dir_all(project.join("subagents")).unwrap();
        let cwd = dir.path().join("a");
        std::fs::create_dir_all(&cwd).unwrap();
        std::fs::write(
            project.join("one.jsonl"),
            serde_json::json!({"cwd":cwd,"customTitle":"Check\u{1b} status"}).to_string() + "\n",
        )
        .unwrap();
        std::fs::write(
            project.join("two.jsonl"),
            serde_json::json!({"cwd":"/different","aiTitle":"Different"}).to_string() + "\n",
        )
        .unwrap();
        std::fs::write(project.join("subagents/agent.jsonl"), "{}\n").unwrap();
        let found = scan(dir.path(), Some(&cwd), Some("check")).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "one");
        assert!(!found[0].title.contains('\u{1b}'));
        assert_eq!(scan(dir.path(), None, None).unwrap().len(), 2);
    }
}
