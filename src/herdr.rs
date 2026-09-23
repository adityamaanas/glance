//! Minimal client for the herdr socket API (newline-delimited JSON over a Unix socket).

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentSession {
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentInfo {
    pub agent_status: String,
    pub agent_session: Option<AgentSession>,
    pub cwd: Option<String>,
}

pub struct Client {
    path: PathBuf,
}

impl Client {
    /// Socket from HERDR_SOCKET_PATH, else the default location if it exists.
    pub fn from_env() -> Option<Client> {
        if let Ok(p) = std::env::var("HERDR_SOCKET_PATH") {
            return Some(Client {
                path: PathBuf::from(p),
            });
        }
        let default = dirs::home_dir()?.join(".config/herdr/herdr.sock");
        default.exists().then_some(Client { path: default })
    }

    pub fn call(&self, method: &str, params: Value) -> Result<Value> {
        let mut stream = UnixStream::connect(&self.path)
            .with_context(|| format!("connect to herdr socket {}", self.path.display()))?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        let req = json!({ "id": format!("glance:{}", method), "method": method, "params": params });
        stream.write_all(format!("{}\n", req).as_bytes())?;
        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line)?;
        let resp: Value = serde_json::from_str(line.trim()).context("parse herdr response")?;
        if let Some(err) = resp.get("error") {
            bail!(
                "herdr {method}: {}",
                err.get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("error")
            );
        }
        resp.get("result")
            .cloned()
            .ok_or_else(|| anyhow!("herdr {method}: no result"))
    }

    pub fn agent_get(&self, target: &str) -> Result<AgentInfo> {
        let result = self.call("agent.get", json!({ "target": target }))?;
        let agent = result
            .get("agent")
            .cloned()
            .ok_or_else(|| anyhow!("agent.get: no agent"))?;
        Ok(serde_json::from_value(agent)?)
    }

    /// Ids of all panes in the tab that holds `pane_id`.
    pub fn tab_panes(&self, pane_id: &str) -> Result<Vec<String>> {
        let result = self.call("pane.layout", json!({ "pane_id": pane_id }))?;
        Ok(result
            .pointer("/layout/panes")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|p| p.get("pane_id").and_then(Value::as_str).map(str::to_string))
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Names of the foreground processes in a pane (empty when the shell is at its prompt).
    pub fn foreground_names(&self, pane_id: &str) -> Result<Vec<String>> {
        let result = self.call("pane.process_info", json!({ "pane_id": pane_id }))?;
        Ok(result
            .pointer("/process_info/foreground_processes")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|p| p.get("name").and_then(Value::as_str).map(str::to_string))
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Stream `pane.agent_status_changed` events for one pane into `tx` until the socket closes.
    pub fn watch_status(&self, pane_id: String, tx: Sender<String>) {
        let path = self.path.clone();
        thread::spawn(move || loop {
            if let Ok(mut stream) = UnixStream::connect(&path) {
                let req = json!({
                    "id": "glance:events",
                    "method": "events.subscribe",
                    "params": { "subscriptions": [
                        { "type": "pane.agent_status_changed", "pane_id": pane_id }
                    ]}
                });
                if stream.write_all(format!("{}\n", req).as_bytes()).is_ok() {
                    // Refresh state after disconnects, even if no new event follows.
                    if let Ok(info) = (Client { path: path.clone() }).agent_get(&pane_id) {
                        if tx.send(info.agent_status).is_err() {
                            return;
                        }
                    }
                    let reader = BufReader::new(stream);
                    for line in reader.lines().map_while(Result::ok) {
                        let Ok(v) = serde_json::from_str::<Value>(&line) else {
                            continue;
                        };
                        if v.get("event").and_then(Value::as_str)
                            == Some("pane.agent_status_changed")
                        {
                            if let Some(s) = v.pointer("/data/agent_status").and_then(Value::as_str)
                            {
                                if tx.send(s.to_string()).is_err() {
                                    return;
                                }
                            }
                        }
                    }
                }
            }
            thread::sleep(Duration::from_secs(2));
        });
    }
}

fn herdr_bin() -> String {
    std::env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".to_string())
}

/// Split `pane_id` to the right and return the new pane id (uses the herdr CLI).
pub fn split_right(pane_id: &str, ratio: f64, cwd: &str) -> Result<String> {
    let out = Command::new(herdr_bin())
        .args([
            "pane",
            "split",
            "--pane",
            pane_id,
            "--direction",
            "right",
            "--ratio",
        ])
        .arg(format!("{ratio}"))
        .args(["--cwd", cwd, "--no-focus"])
        .output()
        .context("run herdr pane split")?;
    if !out.status.success() {
        bail!(
            "herdr pane split failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let v: Value = serde_json::from_slice(&out.stdout).context("parse pane split output")?;
    v.pointer("/result/pane/pane_id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("pane split: no pane id in {}", v))
}

/// Type a command into a pane and press Enter (uses the herdr CLI).
pub fn run_in_pane(pane_id: &str, command: &str) -> Result<()> {
    let out = Command::new(herdr_bin())
        .args(["pane", "run", pane_id, command])
        .output()
        .context("run herdr pane run")?;
    if !out.status.success() {
        bail!(
            "herdr pane run failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn subscription_refreshes_current_state_without_an_event() {
        use std::os::unix::net::UnixListener;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("herdr.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let server = thread::spawn(move || {
            let (subscription, _) = listener.accept().unwrap();
            let mut line = String::new();
            BufReader::new(&subscription).read_line(&mut line).unwrap();
            assert!(line.contains("events.subscribe"));
            let (mut state, _) = listener.accept().unwrap();
            line.clear();
            BufReader::new(&state).read_line(&mut line).unwrap();
            assert!(line.contains("agent.get"));
            writeln!(
                state,
                "{}",
                json!({"result":{"agent":{"agent_status":"idle","agent_session":{"value":"new"}}}})
            )
            .unwrap();
        });
        let (tx, rx) = std::sync::mpsc::channel();
        Client { path }.watch_status("pane".into(), tx);
        assert_eq!(rx.recv_timeout(Duration::from_secs(3)).unwrap(), "idle");
        server.join().unwrap();
    }
}
