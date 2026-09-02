//! First-run offer and installation of the Claude Code SessionStart hook.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

const MATCHER: &str = "startup|resume|clear|fork";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// "accepted" or "declined" once the user has answered the hook offer.
    #[serde(default)]
    pub hook_offer: Option<String>,
}

fn config_path() -> Result<PathBuf> {
    Ok(crate::summary::state_dir()?.join("config.json"))
}

pub fn load_config() -> Config {
    config_path()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_config(cfg: &Config) -> Result<()> {
    std::fs::write(config_path()?, serde_json::to_string_pretty(cfg)?)?;
    Ok(())
}

fn settings_path() -> Result<PathBuf> {
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow!("no home dir"))?
        .join(".claude/settings.json"))
}

fn read_settings() -> Result<Value> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(json!({}));
    }
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

fn write_settings(v: &Value) -> Result<()> {
    let path = settings_path()?;
    if path.exists() {
        std::fs::copy(&path, path.with_extension("json.bak-glance"))?;
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut text = serde_json::to_string_pretty(v)?;
    text.push('\n');
    std::fs::write(&path, text)?;
    Ok(())
}

fn is_glance_entry(entry: &Value) -> bool {
    entry
        .get("hooks")
        .and_then(Value::as_array)
        .map(|hs| {
            hs.iter().any(|h| {
                h.get("command")
                    .and_then(Value::as_str)
                    .map(|c| c.contains("glance"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Whether a SessionStart entry calling glance is registered.
pub fn hook_installed() -> bool {
    read_settings()
        .ok()
        .and_then(|s| {
            s.pointer("/hooks/SessionStart")
                .and_then(Value::as_array)
                .cloned()
        })
        .map(|arr| arr.iter().any(is_glance_entry))
        .unwrap_or(false)
}

/// Register `glance hook` as a SessionStart hook (replacing any earlier glance entry).
pub fn install_hook() -> Result<String> {
    let exe = std::env::current_exe()?.to_string_lossy().to_string();
    let mut settings = read_settings()?;
    let root = settings
        .as_object_mut()
        .ok_or_else(|| anyhow!("settings.json is not an object"))?;
    let hooks = root.entry("hooks").or_insert_with(|| json!({}));
    let hooks = hooks
        .as_object_mut()
        .ok_or_else(|| anyhow!("settings.hooks is not an object"))?;
    let list = hooks.entry("SessionStart").or_insert_with(|| json!([]));
    let list = list
        .as_array_mut()
        .ok_or_else(|| anyhow!("settings.hooks.SessionStart is not an array"))?;
    list.retain(|e| !is_glance_entry(e));
    let command = format!("'{exe}' hook");
    list.push(json!({
        "matcher": MATCHER,
        "hooks": [{ "type": "command", "command": command, "timeout": 20 }]
    }));
    write_settings(&settings)?;
    // Retire the shell-script version if it is still around.
    if let Some(home) = dirs::home_dir() {
        let _ = std::fs::remove_file(home.join(".claude/hooks/glance-attach.sh"));
    }
    Ok(format!(
        "SessionStart hook registered in {} ({command})",
        settings_path()?.display()
    ))
}

pub fn uninstall_hook() -> Result<String> {
    let mut settings = read_settings()?;
    if let Some(list) = settings
        .pointer_mut("/hooks/SessionStart")
        .and_then(Value::as_array_mut)
    {
        list.retain(|e| !is_glance_entry(e));
    }
    write_settings(&settings)?;
    Ok("glance SessionStart hook removed".to_string())
}

/// True when the panel should show the one-time offer banner.
pub fn should_offer() -> bool {
    !hook_installed() && load_config().hook_offer.is_none()
}

pub fn record_offer(answer: &str) -> Result<()> {
    let mut cfg = load_config();
    cfg.hook_offer = Some(answer.to_string());
    save_config(&cfg)
}
