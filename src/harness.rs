//! Read-only adapters for agent-owned conversation formats.
use crate::transcript::{clip, Free, Turn};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Claude,
    Codex,
    Gemini,
    Pi,
    Opencode,
    Cursor,
}
impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::Pi => "pi",
            Self::Opencode => "opencode",
            Self::Cursor => "cursor",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "claude" | "claude-code" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            "gemini" | "gemini-cli" => Some(Self::Gemini),
            "pi" => Some(Self::Pi),
            "opencode" => Some(Self::Opencode),
            "cursor" | "cursor-agent" => Some(Self::Cursor),
            _ => None,
        }
    }
    pub fn storage_key(self, id: &str) -> String {
        if self == Self::Claude {
            id.into()
        } else {
            format!("{}--{id}", self.name())
        }
    }
}

#[derive(Default)]
pub struct Snapshot {
    pub skip: bool,
    pub id: Option<String>,
    pub free: Free,
    pub turns: Vec<Turn>,
}

fn text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter(|p| p["thought"].as_bool() != Some(true))
            .filter_map(|p| match p["type"].as_str() {
                Some("thinking" | "reasoning" | "image" | "image_url") => None,
                _ => p["text"].as_str(),
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}
fn push(snapshot: &mut Snapshot, role: &str, content: String) {
    if content.trim().is_empty() {
        return;
    }
    match role {
        "user" => {
            if snapshot.free.title.is_none() {
                snapshot.free.title = Some(clip(&content, 90));
            }
            snapshot.free.last_prompt = Some(content.clone());
            snapshot.turns.push(Turn::User(content));
        }
        "assistant" | "gemini" => {
            snapshot.free.last_assistant = Some(content.clone());
            snapshot.turns.push(Turn::Assistant(content));
        }
        "tool" | "toolResult" => snapshot.turns.push(Turn::Tool(clip(&content, 4000))),
        _ => {}
    }
}
fn metadata(out: &mut Snapshot, v: &Value) {
    if let Some(cwd) = v["cwd"].as_str() {
        out.free.cwd = Some(cwd.into());
    }
    if let Some(model) = v["model"].as_str() {
        out.free.model = Some(model.into());
    }
    if let Some(timestamp) = v["timestamp"].as_str() {
        out.free.last_timestamp = Some(timestamp.into());
    }
}

pub fn read(kind: Kind, path: &Path, id: &str) -> Result<Snapshot> {
    if kind == Kind::Opencode
        && !matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("json" | "jsonl")
        )
    {
        let mut snapshot = crate::opencode::read(path, id)?;
        snapshot.free.agent = Some(kind.name().into());
        return Ok(snapshot);
    }
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Snapshot::default()),
        Err(e) => return Err(e.into()),
    };
    let mut bytes = Vec::new();
    file.take(128 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 128 * 1024 * 1024 {
        bail!("transcript exceeds the 128 MiB adapter limit; export a smaller session");
    }
    let snapshot = parse(kind, &bytes)?;
    check_id(&snapshot, id)?;
    Ok(snapshot)
}

pub fn check_id(snapshot: &Snapshot, id: &str) -> Result<()> {
    if !id.is_empty() && snapshot.id.as_ref().is_some_and(|found| found != id) {
        bail!("transcript session ID does not match {id}");
    }
    Ok(())
}

/// Line-delimited transcripts can be read incrementally; exports and databases are read whole.
pub fn incremental(kind: Kind, path: &Path) -> bool {
    !matches!(kind, Kind::Claude | Kind::Opencode)
        && path.extension().and_then(|e| e.to_str()) == Some("jsonl")
}

/// Interpret already-parsed records of a line-delimited transcript.
pub fn fold(kind: Kind, records: &[Value]) -> Result<Snapshot> {
    let out = match kind {
        Kind::Codex => codex(records),
        Kind::Gemini => gemini(records),
        Kind::Pi => pi(records),
        Kind::Cursor => cursor(records, &[]),
        Kind::Opencode | Kind::Claude => bail!("{} is not read record by record", kind.name()),
    };
    Ok(finish(kind, out))
}

pub fn parse(kind: Kind, bytes: &[u8]) -> Result<Snapshot> {
    let whole = serde_json::from_slice::<Value>(bytes).ok();
    let records: Vec<Value> = if let Some(v) = &whole {
        vec![v.clone()]
    } else {
        bytes
            .split_inclusive(|b| *b == b'\n')
            .filter(|line| line.ends_with(b"\n"))
            .filter_map(|line| serde_json::from_slice(line).ok())
            .collect()
    };
    let out = match kind {
        Kind::Codex => codex(&records),
        Kind::Gemini => gemini(&records),
        Kind::Pi => pi(&records),
        Kind::Cursor => cursor(&records, bytes),
        Kind::Opencode => opencode_export(whole.as_ref().context("invalid OpenCode JSON export")?),
        Kind::Claude => bail!("Claude uses its incremental transcript reader"),
    };
    Ok(finish(kind, out))
}

fn finish(kind: Kind, mut out: Snapshot) -> Snapshot {
    out.free.agent = Some(kind.name().into());
    out.free.last_prompt = out.turns.iter().rev().find_map(|t| {
        if let Turn::User(s) = t {
            Some(s.clone())
        } else {
            None
        }
    });
    out.free.last_assistant = out.turns.iter().rev().find_map(|t| {
        if let Turn::Assistant(s) = t {
            Some(s.clone())
        } else {
            None
        }
    });
    if out.free.custom_title.is_none() && out.free.title.is_none() {
        out.free.title = Some(format!("{} session", kind.name()));
    }
    out
}

fn codex(records: &[Value]) -> Snapshot {
    let mut out = Snapshot::default();
    let events = records
        .iter()
        .any(|v| v["type"] == "event_msg" && v["payload"]["type"] == "user_message");
    for v in records {
        let p = &v["payload"];
        match v["type"].as_str() {
            Some("session_meta") => {
                out.id = p["id"]
                    .as_str()
                    .or(p["session_id"].as_str())
                    .map(str::to_string);
                metadata(&mut out, p);
            }
            Some("turn_context") => metadata(&mut out, p),
            Some("event_msg") if events => match p["type"].as_str() {
                Some("thread_rolled_back") => {
                    for _ in 0..p["num_turns"]
                        .as_u64()
                        .unwrap_or(0)
                        .min(out.turns.len() as u64)
                    {
                        if let Some(index) =
                            out.turns.iter().rposition(|t| matches!(t, Turn::User(_)))
                        {
                            out.turns.truncate(index);
                        }
                    }
                }
                Some("user_message") => push(&mut out, "user", text(&p["message"])),
                Some("agent_message") => push(&mut out, "assistant", text(&p["message"])),
                _ => {}
            },
            Some("response_item") => match p["type"].as_str() {
                Some("message") if !events => push(
                    &mut out,
                    p["role"].as_str().unwrap_or(""),
                    text(&p["content"]),
                ),
                Some("function_call_output" | "custom_tool_call_output") => {
                    push(&mut out, "tool", text(&p["output"]))
                }
                _ => {}
            },
            _ => {}
        }
    }
    out
}

fn gemini(records: &[Value]) -> Snapshot {
    let mut out = Snapshot::default();
    let mut messages: Vec<Value> = Vec::new();
    for v in records {
        if let Some(id) = v["sessionId"].as_str() {
            out.id = Some(id.into());
        }
        if v["kind"] == "subagent" {
            return Snapshot {
                skip: true,
                ..Snapshot::default()
            };
        }
        let meta = if v["$set"].is_object() { &v["$set"] } else { v };
        if let Some(title) = meta["summary"].as_str() {
            out.free.custom_title = Some(title.into());
        }
        if let Some(cwd) = meta["directories"][0].as_str() {
            out.free.cwd = Some(cwd.into());
        }
        if let Some(list) = meta["messages"].as_array() {
            messages = list.clone();
        }
        if let Some(id) = v["$rewindTo"].as_str() {
            // An unknown target (for example outside a partial read) leaves history intact.
            if let Some(index) = messages.iter().position(|m| m["id"] == id) {
                messages.truncate(index);
            }
        } else if v["id"].is_string() && v["type"].is_string() {
            if let Some(index) = messages.iter().position(|m| m["id"] == v["id"]) {
                messages[index] = v.clone();
            } else {
                messages.push(v.clone());
            }
        }
    }
    for m in messages {
        metadata(&mut out, &m);
        push(
            &mut out,
            m["type"].as_str().unwrap_or(""),
            text(
                m.get("displayContent")
                    .filter(|v| !v.is_null())
                    .unwrap_or(&m["content"]),
            ),
        );
        if let Some(tools) = m["toolCalls"].as_array() {
            for t in tools {
                let output = text(&t["result"]);
                push(
                    &mut out,
                    "tool",
                    format!(
                        "{} [{}]: {}",
                        t["name"].as_str().unwrap_or("tool"),
                        t["status"].as_str().unwrap_or("unknown"),
                        if output.is_empty() {
                            t["result"].to_string()
                        } else {
                            output
                        }
                    ),
                );
            }
        }
    }
    out
}

fn pi(records: &[Value]) -> Snapshot {
    let mut out = Snapshot::default();
    let mut map = HashMap::new();
    for (n, v) in records.iter().enumerate() {
        if v["type"] == "session" {
            out.id = v["id"].as_str().map(str::to_string);
            metadata(&mut out, v);
        } else if let Some(id) = v["id"].as_str() {
            map.insert(id, n);
        }
        if v["type"] == "session_info" {
            out.free.custom_title = v["name"].as_str().map(str::to_string);
        }
    }
    let mut path = Vec::new();
    let mut visited = HashSet::new();
    let mut next = records
        .iter()
        .rev()
        .find(|v| v["type"] != "session" && v["id"].is_string())
        .and_then(|v| v["id"].as_str());
    while let Some(id) = next {
        if !visited.insert(id) {
            break;
        }
        let Some(&index) = map.get(id) else { break };
        path.push(index);
        next = records[index]["parentId"].as_str();
    }
    path.reverse();
    if path.is_empty() {
        path.extend(0..records.len());
    }
    for index in path {
        let v = &records[index];
        if v["type"] == "model_change" {
            out.free.model = v["modelId"].as_str().map(str::to_string);
        }
        if v["type"] != "message" {
            continue;
        }
        let m = &v["message"];
        metadata(&mut out, m);
        let role = m["role"].as_str().unwrap_or("");
        if role == "bashExecution" {
            push(
                &mut out,
                "tool",
                format!(
                    "{} [exit {}]: {}",
                    m["command"].as_str().unwrap_or("shell"),
                    m["exitCode"],
                    m["output"].as_str().unwrap_or("")
                ),
            );
        } else if role == "toolResult" {
            push(
                &mut out,
                "tool",
                format!(
                    "{} [{}]: {}",
                    m["toolName"].as_str().unwrap_or("tool"),
                    if m["isError"] == true {
                        "error"
                    } else {
                        "result"
                    },
                    text(&m["content"])
                ),
            );
        } else {
            push(&mut out, role, text(&m["content"]));
        }
    }
    out
}

fn cursor(records: &[Value], bytes: &[u8]) -> Snapshot {
    let mut out = Snapshot::default();
    for v in records {
        if let Some(id) = v["conversation_id"].as_str().or(v["session_id"].as_str()) {
            out.id = Some(id.into());
        }
        metadata(&mut out, v);
        if let Some(cwd) = v["workspace_roots"][0].as_str() {
            out.free.cwd = Some(cwd.into());
        }
        match v["type"].as_str().or(v["role"].as_str()) {
            Some("user" | "assistant") => push(
                &mut out,
                v["type"].as_str().or(v["role"].as_str()).unwrap(),
                text(v.pointer("/message/content").unwrap_or(&v["content"])),
            ),
            Some("tool_call") if v["subtype"] == "completed" => {
                push(&mut out, "tool", v["tool_call"].to_string())
            }
            _ => match v["hook_event_name"].as_str() {
                Some("beforeSubmitPrompt") => push(&mut out, "user", text(&v["prompt"])),
                Some("afterAgentResponse") => push(&mut out, "assistant", text(&v["text"])),
                Some("postToolUse" | "postToolUseFailure") => push(
                    &mut out,
                    "tool",
                    format!(
                        "{}: {}",
                        v["tool_name"].as_str().unwrap_or("tool"),
                        v.get("tool_output")
                            .or(v.get("error"))
                            .map(text)
                            .unwrap_or_default()
                    ),
                ),
                _ => {}
            },
        }
    }
    if records.is_empty() {
        let input = String::from_utf8_lossy(bytes);
        let mut role = "";
        let mut content = String::new();
        for line in input.lines() {
            match line.trim() {
                "user:" | "assistant:" => {
                    push(&mut out, role, std::mem::take(&mut content));
                    role = line.trim().trim_end_matches(':');
                }
                "<user_query>" | "</user_query>" => {}
                _ => {
                    content.push_str(line);
                    content.push('\n');
                }
            }
        }
        push(&mut out, role, content);
    }
    out
}

pub fn opencode_export(v: &Value) -> Snapshot {
    let mut out = Snapshot {
        id: v["info"]["id"].as_str().map(str::to_string),
        ..Snapshot::default()
    };
    out.free.cwd = v["info"]["directory"].as_str().map(str::to_string);
    out.free.custom_title = v["info"]["title"].as_str().map(str::to_string);
    if let Some(messages) = v["messages"].as_array() {
        for m in messages {
            let role = m["info"]["role"].as_str().unwrap_or("");
            if let Some(parts) = m["parts"].as_array() {
                for p in parts {
                    if p["type"] == "text" {
                        push(&mut out, role, text(&p["text"]));
                    }
                    if p["type"] == "tool" {
                        push(
                            &mut out,
                            "tool",
                            format!(
                                "{} [{}]: {}",
                                p["tool"].as_str().unwrap_or("tool"),
                                p["state"]["status"].as_str().unwrap_or("unknown"),
                                text(p["state"].get("output").unwrap_or(&p["state"]["error"]))
                            ),
                        );
                    }
                }
            }
        }
    }
    out
}

pub fn root(kind: Kind) -> Result<PathBuf> {
    let base = |key: &str, relative: &str| -> Result<PathBuf> {
        match std::env::var_os(key) {
            Some(path) => Ok(PathBuf::from(path)),
            None => Ok(dirs::home_dir()
                .context("no home directory")?
                .join(relative)),
        }
    };
    Ok(match kind {
        Kind::Claude => crate::transcript::claude_dir()?.join("projects"),
        Kind::Codex => base("CODEX_HOME", ".codex")?.join("sessions"),
        Kind::Gemini => base("GLANCE_GEMINI_HOME", ".gemini")?.join("tmp"),
        Kind::Pi => base("PI_CODING_AGENT_SESSION_DIR", ".pi/agent/sessions")?,
        Kind::Cursor => base("GLANCE_CURSOR_HOME", ".cursor")?.join("projects"),
        Kind::Opencode => match std::env::var_os("OPENCODE_DB") {
            Some(path) => PathBuf::from(path),
            None => base("XDG_DATA_HOME", ".local/share")?.join("opencode/opencode.db"),
        },
    })
}

pub fn files(kind: Kind) -> Result<Vec<PathBuf>> {
    fn walk(path: &Path, depth: usize, paths: &mut Vec<PathBuf>) -> Result<()> {
        if depth > 5 {
            return Ok(());
        }
        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        for entry in entries.flatten() {
            let ty = entry.file_type()?;
            let path = entry.path();
            if ty.is_dir() {
                walk(&path, depth + 1, paths)?;
            } else if ty.is_file()
                && matches!(
                    path.extension().and_then(|s| s.to_str()),
                    Some("json" | "jsonl" | "txt")
                )
            {
                paths.push(path);
            }
        }
        Ok(())
    }
    let mut paths = Vec::new();
    walk(&root(kind)?, 0, &mut paths)?;
    if kind == Kind::Cursor {
        let captured = crate::summary::state_dir()?.join("cursor");
        walk(&captured, 0, &mut paths)?;
        paths.retain(|p| {
            p.to_string_lossy().contains("agent-transcripts")
                || p.parent() == Some(captured.as_path())
        });
    }
    Ok(paths)
}

pub fn inspect(kind: Kind, path: &Path) -> Result<Snapshot> {
    use std::io::{Seek, SeekFrom};
    let mut input = std::fs::File::open(path)?;
    let len = input.metadata()?.len();
    let mut data = Vec::new();
    Read::by_ref(&mut input)
        .take(65536)
        .read_to_end(&mut data)?;
    if len > 65536 {
        data.push(b'\n');
        input.seek(SeekFrom::Start(len.saturating_sub(65536)))?;
        Read::by_ref(&mut input)
            .take(65536)
            .read_to_end(&mut data)?;
    }
    let mut result = parse(kind, &data)?;
    if result.skip {
        return Ok(result);
    }
    if result.id.is_none() {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let suffix = stem.get(stem.len().saturating_sub(36)..).unwrap_or(stem);
        if suffix.len() == 36 && suffix.bytes().filter(|b| *b == b'-').count() == 4 {
            result.id = Some(suffix.into());
        } else if kind == Kind::Cursor {
            result.id = Some(stem.into());
        } else if kind == Kind::Gemini {
            result = read(kind, path, "")?;
        }
    }
    Ok(result)
}

pub fn locate(kind: Kind, id: &str, explicit: Option<&Path>) -> Result<PathBuf> {
    crate::transcript::validate_session_id(id)?;
    if let Some(path) = explicit {
        return Ok(std::path::absolute(path)?);
    }
    if kind == Kind::Claude {
        return crate::transcript::find_transcript(id);
    }
    if kind == Kind::Opencode {
        let path = root(kind)?;
        if !path.is_file() {
            bail!("no OpenCode database found; pass --transcript for an export");
        }
        return Ok(path);
    }
    // Agents usually put the session ID in the file name; check those files first.
    let mut paths = files(kind)?;
    paths.sort_by_key(|p| {
        !p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.contains(id))
    });
    for path in paths {
        if inspect(kind, &path).ok().and_then(|s| s.id).as_deref() == Some(id) {
            return Ok(path);
        }
    }
    bail!("no {} transcript found for {id}; pass --transcript <path> for an export or custom location", kind.name())
}
