//! Panel state: produced by Sonnet through `claude -p` on the user's subscription, cached per session.

use crate::transcript::{clip, Transcript};
use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const DEFAULT_MODEL: &str = "claude-sonnet-5";

/// Model for the summary pass: `GLANCE_MODEL` if set, else the default.
pub fn model() -> String {
    std::env::var("GLANCE_MODEL")
        .ok()
        .filter(|m| !m.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_MODEL.to_string())
}
const TIMEOUT: Duration = Duration::from_secs(150);
const MAX_NEW_CHARS: usize = 60_000;

pub const CACHE_VERSION: u32 = 2;

fn trunk() -> String {
    "trunk".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Branch {
    pub id: String,
    pub name: String,
    /// active, parked, or done
    pub status: String,
    #[serde(default)]
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PlanItem {
    pub text: String,
    pub status: String,
    #[serde(default = "trunk")]
    pub branch: String,
    #[serde(default)]
    pub turn: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Item {
    pub text: String,
    #[serde(default = "trunk")]
    pub branch: String,
    #[serde(default)]
    pub turn: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Summary {
    #[serde(default)]
    pub topline: String,
    #[serde(default)]
    pub now: String,
    /// Branch id the newest turns belong to, or "trunk".
    #[serde(default = "trunk")]
    pub focus: String,
    #[serde(default)]
    pub branches: Vec<Branch>,
    #[serde(default)]
    pub plan: Vec<PlanItem>,
    #[serde(default)]
    pub open_questions: Vec<Item>,
    #[serde(default)]
    pub decisions: Vec<Item>,
    #[serde(default)]
    pub blockers: Vec<Item>,
}

impl Summary {
    /// Point every item at a branch that exists; the model sometimes invents or drops ids.
    pub fn normalize(&mut self) {
        let ids: Vec<String> = self.branches.iter().map(|b| b.id.clone()).collect();
        let fix = |b: &mut String| {
            if b != "trunk" && !ids.contains(b) {
                *b = trunk();
            }
        };
        for p in &mut self.plan {
            fix(&mut p.branch);
        }
        for it in self
            .open_questions
            .iter_mut()
            .chain(&mut self.decisions)
            .chain(&mut self.blockers)
        {
            fix(&mut it.branch);
        }
        fix(&mut self.focus);
        for b in &mut self.branches {
            if !matches!(b.status.as_str(), "active" | "parked" | "done") {
                b.status = "active".into();
            }
        }
    }

    pub fn is_multi(&self) -> bool {
        !self.branches.is_empty()
    }

    pub fn branch_name(&self, id: &str) -> String {
        if id == "trunk" {
            return "trunk".into();
        }
        self.branches
            .iter()
            .find(|b| b.id == id)
            .map(|b| b.name.clone())
            .unwrap_or_else(|| id.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Cache {
    #[serde(default)]
    pub version: u32,
    pub summary: Summary,
    pub turns_done: usize,
    pub updated_at: u64,
    pub source: String,
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn state_dir() -> Result<PathBuf> {
    let dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("no home dir"))?
        .join(".glance");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn cache_path(session_id: &str) -> Result<PathBuf> {
    Ok(state_dir()?.join(format!("{session_id}.json")))
}

pub fn load_cache(session_id: &str) -> Option<Cache> {
    let text = std::fs::read_to_string(cache_path(session_id).ok()?).ok()?;
    let cache: Cache = serde_json::from_str(&text).ok()?;
    (cache.version == CACHE_VERSION).then_some(cache)
}

pub fn save_cache(session_id: &str, cache: &Cache) -> Result<()> {
    let path = cache_path(session_id)?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(cache)?)?;
    std::fs::rename(tmp, path)?;
    Ok(())
}

/// Zero-cost fallback so the panel is never empty.
pub fn heuristic(tr: &Transcript) -> Summary {
    Summary {
        topline: tr
            .free
            .custom_title
            .clone()
            .or_else(|| tr.first_user().map(|s| clip(s, 200)))
            .unwrap_or_default(),
        now: tr
            .free
            .last_assistant
            .as_deref()
            .map(|s| clip(s, 200))
            .unwrap_or_default(),
        ..Summary::default()
    }
}

const SYSTEM: &str = "You maintain a compact orientation panel shown beside a developer's live Claude Code session. \
You receive the previous panel state (JSON, may be empty) and the transcript turns since it was written. \
Each user or Claude turn is prefixed with its absolute index like [t42]. Return the updated panel state.\n\
Fields:\n\
- topline: one sentence, what this session is about and working toward. Stable across turns; change only when the goal changes.\n\
- now: one line, what is happening right now or just finished. Present tense.\n\
- branches: the separate threads of work in this session, if any. A branch is a thread a person would name if asked \
'what are the separate things going on here?' (for example one PR review among several, one bug among two being fixed). \
Most sessions have NO branches: return an empty list unless several turns of independent work exist on more than one thread. \
Reuse the previous ids and names exactly; create a branch rarely; merge duplicates; never rename. \
id is a short slug, name is at most 24 characters, status is active, parked (not touched recently but not finished), or done. \
summary is one line. At most 6 branches that are not done.\n\
- focus: the id of the branch the newest turns belong to, or 'trunk' when they concern the session as a whole.\n\
- plan: the steps as stated or clearly implied. Carry prior items forward, update their status, merge duplicates, \
drop items that were abandoned. At most 8 items. status is pending, in_progress, done, or blocked. \
Mark done only with evidence in the transcript. At most 2 items in_progress.\n\
- open_questions: questions Claude asked the user that are still unanswered, and open unknowns named in the discussion. \
Remove once answered. At most 5.\n\
- decisions: choices made and their gist (what and why, tersely). Keep prior ones unless reversed. At most 6.\n\
- blockers: things waiting on the user or an external party. At most 3.\n\
Every plan item, question, decision and blocker carries branch (a branch id, or 'trunk') and turn (the [tN] index \
where it arose or last changed; keep prior values unless the item changed; 0 if unknown).\n\
Every text at most 90 characters. Plain words, no markdown. Never invent facts not in the transcript.";

/// Run Sonnet over the new turns and return the updated summary.
pub fn summarize(prev: &Summary, new_turns: &str, title: Option<&str>) -> Result<Summary> {
    let item = json!({"type": "object", "properties": {
        "text": {"type": "string"}, "branch": {"type": "string"}, "turn": {"type": "integer"}
    }, "required": ["text", "branch", "turn"]});
    let schema = json!({
        "type": "object",
        "properties": {
            "topline": {"type": "string"},
            "now": {"type": "string"},
            "focus": {"type": "string"},
            "branches": {"type": "array", "items": {"type": "object", "properties": {
                "id": {"type": "string"}, "name": {"type": "string"},
                "status": {"type": "string", "enum": ["active", "parked", "done"]},
                "summary": {"type": "string"}
            }, "required": ["id", "name", "status", "summary"]}},
            "plan": {"type": "array", "items": {"type": "object", "properties": {
                "text": {"type": "string"},
                "status": {"type": "string", "enum": ["pending", "in_progress", "done", "blocked"]},
                "branch": {"type": "string"}, "turn": {"type": "integer"}
            }, "required": ["text", "status", "branch", "turn"]}},
            "open_questions": {"type": "array", "items": item},
            "decisions": {"type": "array", "items": item},
            "blockers": {"type": "array", "items": item}
        },
        "required": ["topline", "now", "focus", "branches", "plan", "open_questions", "decisions", "blockers"]
    });
    let prompt = format!(
        "Session title: {}\n\nPREVIOUS PANEL STATE:\n{}\n\nNEW TRANSCRIPT TURNS:\n{}\n\nReturn the updated panel state.",
        title.unwrap_or("(untitled)"),
        serde_json::to_string_pretty(prev)?,
        new_turns
    );
    let mut cmd = Command::new("claude");
    let model = model();
    cmd.args(["-p", "--output-format", "json", "--model", &model])
        .arg("--json-schema")
        .arg(schema.to_string())
        .args(["--append-system-prompt", SYSTEM])
        .args([
            "--no-session-persistence",
            "--tools",
            "",
            "--setting-sources",
            "",
        ])
        .current_dir(state_dir()?)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Do not let herdr register the helper process as an agent in the panel's pane.
    for key in [
        "HERDR_ENV",
        "HERDR_PANE_ID",
        "HERDR_TAB_ID",
        "HERDR_WORKSPACE_ID",
        "HERDR_SOCKET_PATH",
        "HERDR_BIN_PATH",
    ] {
        cmd.env_remove(key);
    }
    let stdout = run_process(&mut cmd, prompt.into_bytes(), TIMEOUT)?;
    parse_response(&stdout)
}

/// Drain both pipes while the child runs, including while it consumes stdin.
fn run_process(cmd: &mut Command, input: Vec<u8>, timeout: Duration) -> Result<String> {
    let mut child = cmd.spawn().context("spawn summary process")?;
    let mut stdin = child.stdin.take().context("summary stdin not piped")?;
    let stdout = child.stdout.take().context("summary stdout not piped")?;
    let stderr = child.stderr.take().context("summary stderr not piped")?;
    let writer = std::thread::spawn(move || stdin.write_all(&input));
    let out = std::thread::spawn(move || read_output(stdout));
    let err = std::thread::spawn(move || read_output(stderr));
    let start = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait()? {
            break s;
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            bail!("summary process timed out after {}s", timeout.as_secs());
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let stdout = out
        .join()
        .map_err(|_| anyhow!("stdout reader panicked"))??;
    let stderr = err
        .join()
        .map_err(|_| anyhow!("stderr reader panicked"))??;
    if !status.success() {
        bail!(
            "summary process exited {}: {}",
            status,
            clip(stderr.trim(), 300)
        );
    }
    writer
        .join()
        .map_err(|_| anyhow!("stdin writer panicked"))??;
    Ok(stdout)
}

fn read_output(mut pipe: impl Read) -> std::io::Result<String> {
    // Drain excess bytes too: retaining a bounded prefix must not block the child.
    let mut kept = Vec::new();
    let mut buf = [0; 8192];
    loop {
        let n = pipe.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let retain = n.min((4 * 1024 * 1024usize).saturating_sub(kept.len()));
        kept.extend_from_slice(&buf[..retain]);
    }
    Ok(String::from_utf8_lossy(&kept).into_owned())
}

fn parse_response(stdout: &str) -> Result<Summary> {
    let envelope: Value =
        serde_json::from_str(stdout.trim()).context("parse claude -p envelope")?;
    if envelope
        .get("is_error")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        bail!(
            "claude -p reported an error: {}",
            clip(&envelope.to_string(), 300)
        );
    }
    let result = envelope
        .get("structured_output")
        .filter(|v| !v.is_null())
        .or_else(|| envelope.get("result"))
        .ok_or_else(|| anyhow!("no structured output in envelope"))?;
    let mut summary: Summary = match result {
        Value::String(s) => serde_json::from_str(s).context("parse structured result")?,
        other => serde_json::from_value(other.clone()).context("parse structured result")?,
    };
    summary.normalize();
    Ok(summary)
}

/// Text to send for the turns not yet summarized.
pub fn pending_text(tr: &Transcript, turns_done: usize) -> String {
    tr.render(turns_done, MAX_NEW_CHARS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structured_output_takes_precedence_over_text_result() {
        let s = parse_response(include_str!("../tests/fixtures/claude-structured.json")).unwrap();
        assert_eq!(s.topline, "Fix retry handling");
        assert_eq!(s.plan[0].status, "done");
        assert!(parse_response(r#"{"is_error":true,"result":"rate limited"}"#).is_err());
        assert!(parse_response(r#"{"result":"not JSON"}"#).is_err());
        assert!(parse_response(r#"{"result":"{\"topline\":\"legacy\"}"}"#).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn process_drains_large_pipes_and_consumes_stdin() {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "head -c 200000 /dev/zero >&2; cat"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let output = run_process(&mut cmd, vec![b'x'; 200_000], Duration::from_secs(5)).unwrap();
        assert_eq!(output.len(), 200_000);
    }

    #[cfg(unix)]
    #[test]
    fn process_timeout_is_bounded_and_errors_include_stderr() {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "exec sleep 10"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let start = Instant::now();
        assert!(run_process(&mut cmd, Vec::new(), Duration::from_millis(50))
            .unwrap_err()
            .to_string()
            .contains("timed out"));
        assert!(start.elapsed() < Duration::from_secs(2));
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "echo failure >&2; exit 2"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        assert!(run_process(&mut cmd, Vec::new(), Duration::from_secs(2))
            .unwrap_err()
            .to_string()
            .contains("failure"));
    }

    #[test]
    fn normalize_points_unknown_branches_at_trunk() {
        let mut s = Summary {
            focus: "ghost".into(),
            branches: vec![Branch {
                id: "a".into(),
                name: "A".into(),
                status: "weird".into(),
                summary: String::new(),
            }],
            plan: vec![
                PlanItem {
                    text: "x".into(),
                    status: "pending".into(),
                    branch: "a".into(),
                    turn: 1,
                },
                PlanItem {
                    text: "y".into(),
                    status: "pending".into(),
                    branch: "zzz".into(),
                    turn: 2,
                },
            ],
            open_questions: vec![Item {
                text: "q".into(),
                branch: "nope".into(),
                turn: 0,
            }],
            ..Summary::default()
        };
        s.normalize();
        assert_eq!(s.focus, "trunk");
        assert_eq!(s.plan[0].branch, "a");
        assert_eq!(s.plan[1].branch, "trunk");
        assert_eq!(s.open_questions[0].branch, "trunk");
        assert_eq!(s.branches[0].status, "active");
        assert_eq!(s.branch_name("a"), "A");
        assert_eq!(s.branch_name("trunk"), "trunk");
        assert_eq!(s.branch_name("missing"), "missing");
    }

    #[test]
    fn old_schema_items_deserialize_with_defaults() {
        let s: Summary = serde_json::from_str(
            r#"{"topline":"t","now":"n","plan":[{"text":"a","status":"done"}],
            "open_questions":[{"text":"q"}],"decisions":[],"blockers":[]}"#,
        )
        .unwrap();
        assert_eq!(s.plan[0].branch, "trunk");
        assert_eq!(s.plan[0].turn, 0);
        assert_eq!(s.focus, "trunk");
        assert!(!s.is_multi());
    }

    #[test]
    fn cache_version_is_serialized() {
        let c = Cache {
            version: CACHE_VERSION,
            ..Cache::default()
        };
        let text = serde_json::to_string(&c).unwrap();
        assert!(text.contains(&format!("\"version\":{CACHE_VERSION}")));
        let back: Cache = serde_json::from_str(&text).unwrap();
        assert_eq!(back.version, CACHE_VERSION);
    }

    #[test]
    fn model_override_from_env() {
        std::env::remove_var("GLANCE_MODEL");
        assert_eq!(model(), DEFAULT_MODEL);
        std::env::set_var("GLANCE_MODEL", "claude-haiku-4-5-20251001");
        assert_eq!(model(), "claude-haiku-4-5-20251001");
        std::env::remove_var("GLANCE_MODEL");
    }
}
