//! Optional turn-end signals written by the hook; no transcript text is stored.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Stop {
    pub path: String,
    pub bytes: u64,
    pub at: u64,
}

pub fn record(input: &serde_json::Value) -> Result<()> {
    let id = input["session_id"]
        .as_str()
        .context("hook has no session_id")?;
    crate::transcript::validate_session_id(id)?;
    let path = input["transcript_path"]
        .as_str()
        .context("hook has no transcript_path")?;
    let signal = Stop {
        path: path.into(),
        bytes: std::fs::metadata(path)?.len(),
        at: crate::summary::now_secs(),
    };
    crate::setup::atomic_write(
        &crate::summary::state_dir()?.join(format!("{id}.stop")),
        &serde_json::to_vec(&signal)?,
    )
}

pub fn matches(session: &str, path: &Path) -> bool {
    let Ok(dir) = crate::summary::state_dir() else {
        return false;
    };
    let Ok(data) = std::fs::read(dir.join(format!("{session}.stop"))) else {
        return false;
    };
    let Ok(signal) = serde_json::from_slice::<Stop>(&data) else {
        return false;
    };
    Path::new(&signal.path) == path
        && std::fs::metadata(path).is_ok_and(|m| {
            m.len() == signal.bytes
                && m.modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .is_some_and(|t| t.as_secs() <= signal.at)
        })
}
