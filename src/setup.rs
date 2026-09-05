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
    pub model: Option<String>,
    pub refresh_seconds: Option<u64>,
    pub prompt: Option<String>,
    #[serde(default)]
    pub no_model: bool,
    pub cache_retention_days: Option<u64>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
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

pub fn read_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    serde_json::from_str(&std::fs::read_to_string(&path)?).context("parse ~/.glance/config.json")
}

pub fn save_config(cfg: &Config) -> Result<()> {
    atomic_write(
        &config_path()?,
        serde_json::to_string_pretty(cfg)?.as_bytes(),
    )?;
    Ok(())
}

fn settings_path() -> Result<PathBuf> {
    Ok(crate::transcript::claude_dir()?.join("settings.json"))
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
    atomic_write(&path, text.as_bytes())?;
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
                    .map(is_glance_command)
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn is_glance_command(command: &str) -> bool {
    let Ok(words) = shell_words::split(command) else {
        return false;
    };
    let Some(exe) = words.first() else {
        return false;
    };
    let name = exe.rsplit(['/', '\\']).next().unwrap_or(exe);
    matches!(name, "glance" | "glance-panel" | "glance-panel.exe")
        && words.get(1).map(String::as_str) == Some("hook")
        || name == "glance-attach.sh"
}

fn remove_glance_hooks(list: &mut Vec<Value>) {
    list.retain_mut(|entry| {
        if !is_glance_entry(entry) {
            return true;
        }
        let Some(hooks) = entry.get_mut("hooks").and_then(Value::as_array_mut) else {
            return true;
        };
        hooks.retain(|hook| {
            !hook
                .get("command")
                .and_then(Value::as_str)
                .map(is_glance_command)
                .unwrap_or(false)
        });
        !hooks.is_empty()
    });
}

pub fn atomic_write(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let parent = path.parent().ok_or_else(|| anyhow!("path has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(bytes)?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)
        .map_err(|e| anyhow!("replace {}: {}", path.display(), e.error))?;
    Ok(())
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
    remove_glance_hooks(list);
    let command = format!("{} hook", shell_words::quote(&exe));
    list.push(json!({
        "matcher": MATCHER,
        "hooks": [{ "type": "command", "command": command, "timeout": 20 }]
    }));
    write_settings(&settings)?;
    // Retire the shell-script version if it is still around.
    if let Ok(dir) = crate::transcript::claude_dir() {
        let _ = std::fs::remove_file(dir.join("hooks/glance-attach.sh"));
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
        remove_glance_hooks(list);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removal_preserves_grouped_hooks_and_substring_matches() {
        let unrelated = json!({"type":"command", "command":"echo glance"});
        let mut entries = vec![
            json!({"matcher":"startup", "hooks":[{"command":"'/a path/glance-panel' hook"}, unrelated.clone()]}),
            json!({"hooks":[{"command":"/bin/glance-panel-helper hook"}]}),
            json!({"hooks":[{"command":"/bin/glance-panel hook"}]}),
        ];
        remove_glance_hooks(&mut entries);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["hooks"], json!([unrelated]));
        assert_eq!(entries[0]["matcher"], "startup");
        let before = entries.clone();
        remove_glance_hooks(&mut entries);
        assert_eq!(entries, before);
    }

    #[test]
    fn quoted_executable_round_trips_and_atomic_write_replaces() {
        let exe = "/a path/it's/glance-panel";
        let cmd = format!("{} hook", shell_words::quote(exe));
        assert!(is_glance_command(&cmd));
        assert_eq!(shell_words::split(&cmd).unwrap()[0], exe);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        atomic_write(&path, b"first").unwrap();
        atomic_write(&path, b"second").unwrap();
        assert_eq!(std::fs::read(path).unwrap(), b"second");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
