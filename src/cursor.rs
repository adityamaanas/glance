//! Opt-in Cursor capture. Hook input is projected onto visible fields before storage.
use anyhow::{bail, Context, Result};
use fs2::FileExt;
use serde_json::{json, Value};
use std::io::{BufRead, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const EVENTS: &[&str] = &[
    "sessionStart",
    "beforeSubmitPrompt",
    "afterAgentResponse",
    "postToolUse",
    "postToolUseFailure",
    "stop",
];
const MAX_INPUT: u64 = 8 * 1024 * 1024;
const MARKER: &str = "glance-generated-cursor-hook-v1";
const SHELL_HOOK: &str = "./hooks/glance-capture.sh";
const CMD_HOOK: &str = "hooks\\glance-capture.cmd";

fn home() -> Result<PathBuf> {
    match std::env::var_os("GLANCE_CURSOR_HOME") {
        Some(dir) => Ok(dir.into()),
        None => Ok(dirs::home_dir()
            .context("no home directory")?
            .join(".cursor")),
    }
}

fn script(exe: &Path, state: &Path) -> Result<String> {
    let exe = exe.to_str().context("executable path is not UTF-8")?;
    let state = state.to_str().context("state path is not UTF-8")?;
    if exe.chars().chain(state.chars()).any(char::is_control) {
        bail!("hook paths cannot contain control characters");
    }
    #[cfg(windows)]
    {
        if exe.contains('"') || state.contains('"') {
            bail!("invalid hook path");
        }
        Ok(format!("@rem {MARKER}\r\n@echo off\r\nsetlocal DisableDelayedExpansion\r\n\"{}\" cursor-hook --state-dir \"{}\"\r\nexit /b 0\r\n", exe.replace('%', "%%"), state.replace('%', "%%")))
    }
    #[cfg(not(windows))]
    Ok(format!(
        "#!/bin/sh\n# {MARKER}\nexec {} cursor-hook --state-dir {}\n",
        shell_words::quote(exe),
        shell_words::quote(state)
    ))
}

pub fn setup(remove: bool) -> Result<String> {
    let dir = std::path::absolute(home()?)?;
    let path = dir.join("hooks.json");
    let mut value: Value = match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .context("parse Cursor hooks.json; file left unchanged")?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => json!({"version":1,"hooks":{}}),
        Err(e) => return Err(e.into()),
    };
    let root = value
        .as_object_mut()
        .context("Cursor hooks.json must be an object")?;
    if root.get("version").is_some_and(|v| v != 1) {
        bail!("unsupported Cursor hooks version; file left unchanged");
    }
    root.entry("version").or_insert(json!(1));
    let hooks = root
        .entry("hooks")
        .or_insert(json!({}))
        .as_object_mut()
        .context("Cursor hooks must be an object")?;
    for event in EVENTS {
        let entries = hooks
            .entry(*event)
            .or_insert(json!([]))
            .as_array_mut()
            .context("Cursor hook entries must be arrays")?;
        entries.retain(|v| !matches!(v["command"].as_str(), Some(SHELL_HOOK | CMD_HOOK)));
        if !remove {
            entries.push(json!({"command":if cfg!(windows) {CMD_HOOK} else {SHELL_HOOK},"timeout":5,"failClosed":false}));
        }
    }
    if !remove {
        let wrapper = dir.join(if cfg!(windows) {
            "hooks/glance-capture.cmd"
        } else {
            "hooks/glance-capture.sh"
        });
        if wrapper.exists()
            && !std::fs::read_to_string(&wrapper)?
                .lines()
                .take(2)
                .any(|line| line.contains(MARKER))
        {
            bail!(
                "{} already exists and is not a Glance wrapper; file left unchanged",
                wrapper.display()
            );
        }
        let state = std::path::absolute(crate::summary::state_dir()?)?;
        crate::setup::atomic_write(
            &wrapper,
            script(&std::env::current_exe()?, &state)?.as_bytes(),
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700))?;
        }
    }
    // Keep the first backup: it is the only copy from before Glance changed the file.
    let backup = path.with_extension("json.bak-glance");
    if path.exists() && !backup.exists() {
        std::fs::copy(&path, backup)?;
    }
    crate::setup::atomic_write(&path, serde_json::to_string_pretty(&value)?.as_bytes())?;
    Ok(format!(
        "Cursor capture hooks {} in {}",
        if remove { "removed" } else { "installed" },
        path.display()
    ))
}

fn visible(input: &Value) -> Option<Value> {
    let event = input["hook_event_name"].as_str()?;
    if !EVENTS.contains(&event) || input["agent_id"].as_str().is_some() {
        return None;
    }
    let id = input["conversation_id"].as_str()?;
    if crate::transcript::validate_session_id(id).is_err() {
        return None;
    }
    let mut out = json!({"conversation_id":id,"hook_event_name":event});
    for key in ["generation_id", "model", "cwd"] {
        if let Some(s) = input[key].as_str() {
            out[key] = json!(crate::transcript::clip(s, 4096));
        }
    }
    if let Some(cwd) = input["workspace_roots"][0].as_str() {
        out["cwd"] = json!(crate::transcript::clip(cwd, 4096));
    }
    for (key, limit) in match event {
        "beforeSubmitPrompt" => vec![("prompt", 60_000)],
        "afterAgentResponse" => vec![("text", 60_000)],
        "postToolUse" => vec![
            ("tool_name", 200),
            ("tool_use_id", 200),
            ("tool_output", 8000),
        ],
        "postToolUseFailure" => vec![
            ("tool_name", 200),
            ("tool_use_id", 200),
            ("error_message", 8000),
        ],
        _ => vec![],
    } {
        if let Some(s) = input[key].as_str() {
            out[key] = json!(crate::transcript::clip(s, limit));
        }
    }
    Some(out)
}

fn append(state: &Path, input: &Value) -> Result<()> {
    let id = input["conversation_id"]
        .as_str()
        .context("missing conversation ID")?;
    crate::transcript::validate_session_id(id)?;
    let dir = state.join("cursor");
    std::fs::create_dir_all(&dir)?;
    let mut options = std::fs::OpenOptions::new();
    options.create(true).read(true).write(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(dir.join(format!("{id}.jsonl")))?;
    let started = Instant::now();
    loop {
        match file.try_lock_exclusive() {
            Ok(()) => break,
            Err(e)
                if e.raw_os_error() == fs2::lock_contended_error().raw_os_error()
                    && started.elapsed() < Duration::from_secs(1) =>
            {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(e) => return Err(e.into()),
        }
    }
    // Ignore a duplicate recent hook delivery, but retain equal text in another generation.
    let len = file.metadata()?.len();
    if len > 128 * 1024 * 1024 {
        bail!(
            "Cursor capture exceeds 128 MiB; archive this conversation capture before continuing"
        );
    }
    file.seek(SeekFrom::Start(len.saturating_sub(131072)))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let tail = String::from_utf8_lossy(&bytes);
    let duplicate = input["generation_id"].as_str().is_some()
        && tail
            .lines()
            .any(|line| serde_json::from_str::<Value>(line).is_ok_and(|v| v == *input));
    if !duplicate {
        file.seek(SeekFrom::End(0))?;
        if len > 0 && !tail.ends_with('\n') {
            file.write_all(b"\n")?;
        }
        serde_json::to_writer(&mut file, input)?;
        file.write_all(b"\n")?;
        file.flush()?;
    }
    if input["hook_event_name"] == "stop" {
        let signal = crate::signals::Stop {
            path: dir
                .join(format!("{id}.jsonl"))
                .to_string_lossy()
                .into_owned(),
            bytes: file.metadata()?.len(),
            at: crate::summary::now_secs(),
        };
        crate::setup::atomic_write(
            &state.join(format!("cursor--{id}.stop")),
            &serde_json::to_vec(&signal)?,
        )?;
    }
    Ok(())
}

/// Always fail open, including malformed Glance configuration or incomplete hook payloads.
pub fn hook(state: Option<&Path>) {
    let result = (|| -> Result<()> {
        let mut bytes = Vec::new();
        std::io::stdin()
            .take(MAX_INPUT + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_INPUT || std::env::var_os("GLANCE_SUMMARY_HELPER").is_some() {
            return Ok(());
        }
        let input: Value = serde_json::from_slice(&bytes)?;
        if let Some(out) = visible(&input) {
            let state = match state {
                Some(path) => path.to_path_buf(),
                None => crate::summary::state_dir()?,
            };
            append(&state, &out)?;
        }
        Ok(())
    })();
    if let Err(e) = result {
        eprintln!("glance: Cursor capture skipped: {e}");
    }
    println!("{{}}");
}

/// Import the documented complete-message CLI stream without reading Cursor's private database.
///
/// Forwarding comes first: every input byte reaches stdout before capture is attempted, and
/// a capture problem is reported on stderr without interrupting the downstream pipeline.
pub fn stream() -> Result<()> {
    let stdin = std::io::stdin();
    let mut reader = stdin.lock();
    let mut out = std::io::stdout().lock();
    let mut capture = StreamCapture {
        state: crate::summary::state_dir().ok(),
        ..StreamCapture::default()
    };
    let mut oversized = false;
    loop {
        let mut bytes = Vec::new();
        Read::by_ref(&mut reader)
            .take(MAX_INPUT + 1)
            .read_until(b'\n', &mut bytes)?;
        if bytes.is_empty() {
            break;
        }
        out.write_all(&bytes)?;
        out.flush()?;
        let complete = bytes.ends_with(b"\n");
        if oversized || bytes.len() as u64 > MAX_INPUT {
            // Forward the rest of an oversized record untouched; capture resumes after it.
            if !oversized {
                capture.warn("Cursor stream record exceeds 8 MiB; not captured".into());
            }
            oversized = !complete;
            continue;
        }
        if let Err(e) = capture.record(&bytes) {
            capture.warn(format!("{e:#}"));
        }
    }
    if capture.problems > 1 {
        eprintln!(
            "glance: {} more Cursor capture problems",
            capture.problems - 1
        );
    }
    if capture.id.is_none() && capture.problems == 0 {
        eprintln!("glance: nothing captured; use --output-format stream-json");
    }
    Ok(())
}

#[derive(Default)]
struct StreamCapture {
    state: Option<PathBuf>,
    id: Option<String>,
    cwd: Value,
    model: Value,
    problems: usize,
}

impl StreamCapture {
    /// Report the first problem; later ones are counted to keep stderr readable.
    fn warn(&mut self, message: String) {
        if self.problems == 0 {
            eprintln!("glance: Cursor capture skipped: {message}");
        }
        self.problems += 1;
    }

    fn record(&mut self, bytes: &[u8]) -> Result<()> {
        let state = self
            .state
            .clone()
            .context("Glance state directory is unavailable")?;
        let v: Value = serde_json::from_slice(bytes).context("invalid Cursor stream JSON")?;
        if let Some(found) = v["session_id"].as_str() {
            crate::transcript::validate_session_id(found)?;
            match &self.id {
                Some(id) if id != found => bail!("Cursor stream changed session ID"),
                Some(_) => {}
                None => {
                    eprintln!("glance: capturing Cursor session {found}");
                    self.id = Some(found.into());
                }
            }
        }
        let session = self.id.clone().context(
            "Cursor stream needs an initial session_id; use --output-format stream-json",
        )?;
        if v["cwd"].is_string() {
            self.cwd = v["cwd"].clone();
        }
        if v["model"].is_string() {
            self.model = v["model"].clone();
        }
        let (cwd, model) = (&self.cwd, &self.model);
        if v["type"] == "system" {
            append(
                &state,
                &json!({"conversation_id":session,"hook_event_name":"sessionStart","cwd":cwd,"model":model}),
            )?;
        }
        let parsed = crate::harness::parse(crate::harness::Kind::Cursor, bytes)?;
        for turn in parsed.turns {
            let (event, field, content) = match turn {
                crate::transcript::Turn::User(t) => ("beforeSubmitPrompt", "prompt", t),
                crate::transcript::Turn::Assistant(t) => ("afterAgentResponse", "text", t),
                crate::transcript::Turn::Tool(t) => ("postToolUse", "tool_output", t),
            };
            let mut input =
                json!({"conversation_id":session,"hook_event_name":event,"cwd":cwd,"model":model});
            input[field] = json!(content);
            append(
                &state,
                &visible(&input).context("invalid normalized Cursor event")?,
            )?;
        }
        if v["type"] == "result" {
            append(
                &state,
                &json!({"conversation_id":session,"hook_event_name":"stop"}),
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concurrent_capture_is_complete_private_and_deduplicates_retries() {
        let dir = tempfile::tempdir().unwrap();
        std::thread::scope(|s| {
            for n in 0..12 {
                let path = dir.path();
                s.spawn(move || {
                    let input = json!({"conversation_id":"test","generation_id":n.to_string(),"hook_event_name":"beforeSubmitPrompt","prompt":"visible","user_email":"secret","thought":"secret","attachments":["secret"]});
                    let v = visible(&input).unwrap();
                    append(path, &v).unwrap(); append(path, &v).unwrap();
                });
            }
        });
        let text = std::fs::read_to_string(dir.path().join("cursor/test.jsonl")).unwrap();
        assert_eq!(text.lines().count(), 12);
        assert!(!text.contains("secret"));
        assert!(visible(&json!({"conversation_id":"../bad","hook_event_name":"stop"})).is_none());
        assert!(visible(
            &json!({"conversation_id":"test","hook_event_name":"afterAgentThought","text":"secret"})
        )
        .is_none());
    }
}
