//! Incremental reader for a Claude Code transcript (`~/.claude/projects/<slug>/<session>.jsonl`).

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Fields Claude Code writes as their own transcript entries; free to read, no model needed.
#[derive(Debug, Default, Clone)]
pub struct Free {
    pub title: Option<String>,
    pub custom_title: Option<String>,
    pub last_prompt: Option<String>,
    pub pr_number: Option<u64>,
    pub worktree: Option<String>,
    pub branch: Option<String>,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub cost_usd: Option<f64>,
    pub last_assistant: Option<String>,
    pub last_timestamp: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Turn {
    User(String),
    Assistant(String),
    Tool(String),
}

pub struct Transcript {
    pub path: PathBuf,
    offset: u64,
    partial: Vec<u8>,
    tail: Vec<u8>,
    pub revision: u64,
    pub free: Free,
    pub turns: Vec<Turn>,
}

/// Where Claude Code will write a session's transcript for a given working directory
/// (the project slug replaces every non-alphanumeric character with '-').
pub fn expected_path(cwd: &str, session_id: &str) -> Result<PathBuf> {
    let slug: String = cwd
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow!("no home dir"))?
        .join(".claude/projects")
        .join(slug)
        .join(format!("{session_id}.jsonl")))
}

/// Locate a session's transcript under any project directory.
pub fn find_transcript(session_id: &str) -> Result<PathBuf> {
    let projects = dirs::home_dir()
        .ok_or_else(|| anyhow!("no home dir"))?
        .join(".claude/projects");
    let name = format!("{session_id}.jsonl");
    for entry in std::fs::read_dir(&projects)
        .context("read ~/.claude/projects")?
        .flatten()
    {
        let candidate = entry.path().join(&name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(anyhow!("no transcript found for session {session_id}"))
}

impl Transcript {
    pub fn open(path: &Path) -> Transcript {
        Transcript {
            path: path.to_path_buf(),
            offset: 0,
            partial: Vec::new(),
            tail: Vec::new(),
            revision: 0,
            free: Free::default(),
            turns: Vec::new(),
        }
    }

    /// Read appended lines; returns how many turns were added. A missing file reads as empty.
    pub fn read_new(&mut self) -> Result<usize> {
        let mut file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e.into()),
        };
        let mut replaced = file.metadata()?.len() < self.offset;
        if !replaced && !self.tail.is_empty() {
            file.seek(SeekFrom::Start(self.offset - self.tail.len() as u64))?;
            let mut tail = vec![0; self.tail.len()];
            file.read_exact(&mut tail)?;
            replaced = tail != self.tail;
        }
        if replaced {
            self.offset = 0;
            self.partial.clear();
            self.tail.clear();
            self.turns.clear();
            self.free = Free::default();
            self.revision = self.revision.wrapping_add(1);
        }
        file.seek(SeekFrom::Start(self.offset))?;
        let mut reader = BufReader::new(file);
        let before = self.turns.len();
        let mut buf = Vec::new();
        loop {
            buf.clear();
            let n = reader.read_until(b'\n', &mut buf)?;
            if n == 0 {
                break;
            }
            self.offset += n as u64;
            self.partial.extend_from_slice(&buf);
            if !buf.ends_with(b"\n") {
                break;
            }
            let line = std::mem::take(&mut self.partial);
            if let Ok(v) = serde_json::from_slice::<Value>(&line) {
                self.ingest(&v);
            }
        }
        let mut file = reader.into_inner();
        let tail_len = self.offset.min(128) as usize;
        file.seek(SeekFrom::Start(self.offset - tail_len as u64))?;
        self.tail.resize(tail_len, 0);
        file.read_exact(&mut self.tail)?;
        Ok(self.turns.len() - before)
    }

    fn ingest(&mut self, v: &Value) {
        let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_string);
        match v.get("type").and_then(Value::as_str).unwrap_or("") {
            "ai-title" => self.free.title = s("aiTitle"),
            "custom-title" => self.free.custom_title = s("customTitle"),
            "last-prompt" => self.free.last_prompt = s("lastPrompt"),
            "pr-link" => self.free.pr_number = v.get("prNumber").and_then(Value::as_u64),
            "worktree-state" => {
                self.free.worktree = v
                    .pointer("/worktreeSession/worktreeName")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            }
            "cost-state" => self.free.cost_usd = v.get("totalCostUSD").and_then(Value::as_f64),
            "user" => self.ingest_user(v),
            "assistant" => self.ingest_assistant(v),
            _ => {}
        }
    }

    fn ingest_common(&mut self, v: &Value) {
        if let Some(b) = v.get("gitBranch").and_then(Value::as_str) {
            self.free.branch = Some(b.to_string());
        }
        if let Some(c) = v.get("cwd").and_then(Value::as_str) {
            self.free.cwd = Some(c.to_string());
        }
        if let Some(t) = v.get("timestamp").and_then(Value::as_str) {
            self.free.last_timestamp = Some(t.to_string());
        }
    }

    fn ingest_user(&mut self, v: &Value) {
        if v.get("isSidechain")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || v.get("isMeta").and_then(Value::as_bool).unwrap_or(false)
        {
            return;
        }
        self.ingest_common(v);
        let Some(content) = v.pointer("/message/content") else {
            return;
        };
        let mut text = String::new();
        match content {
            Value::String(t) => text.push_str(t),
            Value::Array(blocks) => {
                for b in blocks {
                    if b.get("type").and_then(Value::as_str) == Some("text") {
                        if let Some(t) = b.get("text").and_then(Value::as_str) {
                            text.push_str(t);
                            text.push('\n');
                        }
                    }
                    if b.get("type").and_then(Value::as_str) == Some("tool_result") {
                        let outcome = match b.get("content") {
                            Some(Value::String(s)) => s.clone(),
                            Some(Value::Array(parts)) => parts
                                .iter()
                                .filter_map(|p| p.get("text").and_then(Value::as_str))
                                .collect::<Vec<_>>()
                                .join("\n"),
                            _ => String::new(),
                        };
                        let status = if b.get("is_error").and_then(Value::as_bool) == Some(true) {
                            "error"
                        } else {
                            "result"
                        };
                        self.turns
                            .push(Turn::Tool(format!("{status}: {}", clip(&outcome, 1500))));
                    }
                }
            }
            _ => {}
        }
        let clean = strip_wrappers(&text);
        if !clean.is_empty() {
            self.turns.push(Turn::User(clean));
        }
    }

    fn ingest_assistant(&mut self, v: &Value) {
        if v.get("isSidechain")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return;
        }
        self.ingest_common(v);
        if let Some(m) = v.pointer("/message/model").and_then(Value::as_str) {
            self.free.model = Some(m.to_string());
        }
        let Some(blocks) = v.pointer("/message/content").and_then(Value::as_array) else {
            return;
        };
        for b in blocks {
            match b.get("type").and_then(Value::as_str) {
                Some("text") => {
                    let t = b.get("text").and_then(Value::as_str).unwrap_or("").trim();
                    if !t.is_empty() {
                        self.free.last_assistant = Some(t.to_string());
                        self.turns.push(Turn::Assistant(t.to_string()));
                    }
                }
                Some("tool_use") => {
                    let name = b.get("name").and_then(Value::as_str).unwrap_or("tool");
                    let brief = tool_brief(name, b.get("input"));
                    self.turns.push(Turn::Tool(format!("{name}: {brief}")));
                }
                _ => {}
            }
        }
    }

    /// Render turns `from..` as compact text for the model.
    pub fn render(&self, from: usize, max_chars: usize) -> String {
        let mut out = String::new();
        for (i, t) in self.turns.iter().enumerate().skip(from) {
            match t {
                Turn::User(s) => out.push_str(&format!("[t{i}] USER: {}\n\n", clip(s, 2000))),
                Turn::Assistant(s) => {
                    out.push_str(&format!("[t{i}] CLAUDE: {}\n\n", clip(s, 2000)))
                }
                Turn::Tool(s) => out.push_str(&format!("[t{i}] TOOL: {}\n", clip(s, 1500))),
            }
        }
        if out.len() > max_chars {
            let cut = out.len() - max_chars;
            let mut start = cut;
            while !out.is_char_boundary(start) {
                start += 1;
            }
            out = format!("(earlier turns omitted)\n\n{}", &out[start..]);
        }
        out
    }

    pub fn first_user(&self) -> Option<&str> {
        self.turns.iter().find_map(|t| match t {
            Turn::User(s) => Some(s.as_str()),
            _ => None,
        })
    }

    pub fn fingerprint(&self, count: usize) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        for turn in self.turns.iter().take(count) {
            match turn {
                Turn::User(s) => (0, s).hash(&mut hash),
                Turn::Assistant(s) => (1, s).hash(&mut hash),
                Turn::Tool(s) => (2, s).hash(&mut hash),
            }
        }
        hash.finish()
    }
}

fn tool_brief(name: &str, input: Option<&Value>) -> String {
    let Some(input) = input else {
        return String::new();
    };
    let pick = |keys: &[&str]| {
        keys.iter()
            .find_map(|k| input.get(*k).and_then(Value::as_str))
            .map(|s| s.lines().next().unwrap_or("").to_string())
    };
    match name {
        "Bash" => pick(&["description", "command"]),
        "Read" | "Edit" | "Write" | "NotebookEdit" => pick(&["file_path"]),
        "Agent" | "Workflow" => pick(&["description"]),
        "Grep" | "Glob" | "WebSearch" => pick(&["pattern", "query"]),
        "WebFetch" => pick(&["url"]),
        "Skill" => pick(&["skill"]),
        _ => pick(&["description", "prompt", "query", "command", "file_path"]),
    }
    .unwrap_or_default()
}

/// Drop `<system-reminder>` blocks and local-command noise from a user message.
fn strip_wrappers(text: &str) -> String {
    let mut s = text.to_string();
    for (open, close) in [
        ("<system-reminder>", "</system-reminder>"),
        ("<local-command-caveat>", "</local-command-caveat>"),
        ("<local-command-stdout>", "</local-command-stdout>"),
        ("<command-name>", "</command-name>"),
        ("<command-message>", "</command-message>"),
        ("<command-args>", "</command-args>"),
    ] {
        while let Some(a) = s.find(open) {
            match s[a..].find(close) {
                Some(rel) => s.replace_range(a..a + rel + close.len(), ""),
                None => {
                    s.truncate(a);
                    break;
                }
            }
        }
    }
    s.trim().to_string()
}

pub fn clip(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_utf8_and_rewritten_transcript_recover_without_old_turns() {
        use std::io::Write;
        let mut tr = tr_with(&[r#"{"type":"user","message":{"content":"old"}}"#]);
        let record = "{\"type\":\"user\",\"message\":{\"content\":\"café\"}}\n".as_bytes();
        let split = record.iter().position(|b| *b == 0xc3).unwrap() + 1;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&tr.path)
            .unwrap();
        file.write_all(&record[..split]).unwrap();
        assert_eq!(tr.read_new().unwrap(), 0);
        file.write_all(&record[split..]).unwrap();
        assert_eq!(tr.read_new().unwrap(), 1);
        assert!(matches!(&tr.turns[1], Turn::User(s) if s == "café"));
        drop(file);
        let old_hash = tr.fingerprint(tr.turns.len());
        std::fs::write(
            &tr.path,
            b"{\"type\":\"user\",\"message\":{\"content\":\"new\"}}\n",
        )
        .unwrap();
        tr.read_new().unwrap();
        assert_eq!(tr.turns.len(), 1);
        assert_eq!(tr.revision, 1);
        assert_ne!(old_hash, tr.fingerprint(tr.turns.len()));
        std::fs::write(
            &tr.path,
            b"{\"type\":\"user\",\"message\":{\"content\":\"alt\"}}\n",
        )
        .unwrap();
        tr.read_new().unwrap();
        assert!(matches!(&tr.turns[0], Turn::User(s) if s == "alt"));
        assert_eq!(tr.revision, 2);
    }

    #[test]
    fn tool_failures_are_evidence_with_absolute_turn_indices() {
        let tr = tr_with(&[
            r#"{"type":"user","message":{"content":[{"type":"tool_result","is_error":true,"content":"test failed: duplicate charge"}]}}"#,
        ]);
        assert!(tr
            .render(0, 10000)
            .contains("[t0] TOOL: error: test failed: duplicate charge"));
    }

    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    fn tr_with(lines: &[&str]) -> Transcript {
        let dir = std::env::temp_dir().join(format!("glance-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let path = dir.join(format!("{n}.jsonl"));
        std::fs::write(&path, lines.join("\n") + "\n").unwrap();
        let mut tr = Transcript::open(&path);
        tr.read_new().unwrap();
        tr
    }

    #[test]
    fn strips_system_reminders_and_local_command_noise() {
        let s = strip_wrappers("hello <system-reminder>secret</system-reminder> world");
        assert_eq!(s, "hello  world");
        assert_eq!(
            strip_wrappers("<local-command-caveat>x</local-command-caveat>"),
            ""
        );
        assert_eq!(strip_wrappers("<system-reminder>unterminated"), "");
    }

    #[test]
    fn reads_user_and_assistant_turns_and_free_fields() {
        let tr = tr_with(&[
            r#"{"type":"ai-title","aiTitle":"Fix the login bug"}"#,
            r#"{"type":"user","gitBranch":"dev","cwd":"/repo","message":{"role":"user","content":"fix login"}}"#,
            r#"{"type":"user","isMeta":true,"message":{"role":"user","content":"ignored"}}"#,
            r#"{"type":"assistant","message":{"model":"claude-sonnet-5","content":[{"type":"text","text":"On it."},{"type":"tool_use","name":"Bash","input":{"description":"run tests","command":"cargo test"}}]}}"#,
            r#"{"type":"user","message":{"role":"user","content":[{"type":"tool_result","content":"ok"}]}}"#,
            r#"{"type":"pr-link","prNumber":42}"#,
        ]);
        assert_eq!(tr.turns.len(), 4);
        assert!(matches!(&tr.turns[0], Turn::User(t) if t == "fix login"));
        assert!(matches!(&tr.turns[1], Turn::Assistant(t) if t == "On it."));
        assert!(matches!(&tr.turns[2], Turn::Tool(t) if t == "Bash: run tests"));
        assert_eq!(tr.free.title.as_deref(), Some("Fix the login bug"));
        assert_eq!(tr.free.branch.as_deref(), Some("dev"));
        assert_eq!(tr.free.pr_number, Some(42));
        assert_eq!(tr.free.model.as_deref(), Some("claude-sonnet-5"));
        assert_eq!(tr.free.last_assistant.as_deref(), Some("On it."));
    }

    #[test]
    fn render_uses_absolute_turn_indices() {
        let tr = tr_with(&[
            r#"{"type":"user","message":{"role":"user","content":"one"}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"two"}]}}"#,
            r#"{"type":"user","message":{"role":"user","content":"three"}}"#,
        ]);
        let out = tr.render(1, 10_000);
        assert!(out.starts_with("[t1] CLAUDE: two"));
        assert!(out.contains("[t2] USER: three"));
        assert!(!out.contains("[t0]"));
    }

    #[test]
    fn render_caps_length_and_marks_omission() {
        let long = "x".repeat(500);
        let line = format!(r#"{{"type":"user","message":{{"role":"user","content":"{long}"}}}}"#);
        let tr = tr_with(&[&line, &line, &line]);
        let out = tr.render(0, 700);
        assert!(out.starts_with("(earlier turns omitted)"));
        assert!(out.len() < 800);
    }

    #[test]
    fn missing_file_reads_as_empty_and_partial_lines_wait() {
        let mut tr = Transcript::open(std::path::Path::new("/nonexistent/glance/x.jsonl"));
        assert_eq!(tr.read_new().unwrap(), 0);
        let mut tr = tr_with(&[r#"{"type":"user","message":{"role":"user","content":"a"}}"#]);
        std::fs::OpenOptions::new()
            .append(true)
            .open(&tr.path)
            .unwrap();
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&tr.path)
            .unwrap();
        write!(f, r#"{{"type":"user","message":{{"role":"user","con"#).unwrap();
        assert_eq!(tr.read_new().unwrap(), 0);
        writeln!(f, r#"tent":"b"}}}}"#).unwrap();
        assert_eq!(tr.read_new().unwrap(), 1);
        assert_eq!(tr.turns.len(), 2);
    }

    #[test]
    fn expected_path_slugs_every_non_alphanumeric() {
        let p = expected_path("/Users/me/Documents/my_project", "abc").unwrap();
        let s = p.to_string_lossy();
        assert!(
            s.ends_with("/.claude/projects/-Users-me-Documents-my-project/abc.jsonl"),
            "{s}"
        );
    }

    #[test]
    fn clip_adds_ellipsis() {
        assert_eq!(clip("abcdef", 4), "abc…");
        assert_eq!(clip("  ab  ", 10), "ab");
    }
}
