use std::process::Command;

#[cfg(unix)]
#[test]
fn long_session_summary_and_html_export_keep_early_evidence() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let state = dir.path().join("state");
    let config = dir.path().join("claude");
    let bin = dir.path().join("bin");
    std::fs::create_dir_all(config.join("projects/test")).unwrap();
    std::fs::create_dir_all(&bin).unwrap();
    let transcript = (0..40).map(|n| serde_json::json!({"type":"user","message":{"content":format!("Evidence {n}: {}", "x".repeat(2100))}}).to_string()+"\n").collect::<String>();
    std::fs::write(config.join("projects/test/session.jsonl"), transcript).unwrap();
    let stub = bin.join("claude");
    std::fs::write(
        &stub,
        b"#!/bin/sh\ncat >> \"$GLANCE_CAPTURE\"\ncat \"$GLANCE_FIXTURE\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    let fixture = dir.path().join("response.json");
    std::fs::write(&fixture, serde_json::json!({"structured_output":{"topline":"Review evidence", "plan":[{"id":"p1","text":"Inspect first turn","status":"done","source_turns":[0]}]},"total_cost_usd":0.01}).to_string()).unwrap();
    let capture = dir.path().join("input.txt");
    let mut command = Command::new(env!("CARGO_BIN_EXE_glance-panel"));
    command
        .env("GLANCE_HOME", &state)
        .env("CLAUDE_CONFIG_DIR", &config)
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env("GLANCE_FIXTURE", &fixture)
        .env("GLANCE_CAPTURE", &capture)
        .args(["summarize", "--session", "session"]);
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let cache: serde_json::Value =
        serde_json::from_slice(&std::fs::read(state.join("session.json")).unwrap()).unwrap();
    assert_eq!(cache["turns_done"], 40);
    assert_eq!(cache["summary"]["usage"]["calls"], 2);
    let input = std::fs::read_to_string(capture).unwrap();
    assert!(input.contains("[t0] USER:"));
    assert!(input.contains("[t39] USER:"));
    let output = dir.path().join("graph.html");
    let result = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
        .env("GLANCE_HOME", &state)
        .env("CLAUDE_CONFIG_DIR", &config)
        .args(["graph", "--session", "session", "--html"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(std::fs::read_to_string(output)
        .unwrap()
        .contains("Evidence 0:"));
}

#[test]
fn model_free_config_prevents_explicit_summary_invocation() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.json"), r#"{"no_model":true}"#).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
        .env("GLANCE_HOME", dir.path())
        .args(["summarize", "--session", "fictional-session"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("model calls disabled"));
}

#[test]
fn malformed_config_is_reported_but_hook_remains_nonblocking() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.json"), b"not json").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
        .env("GLANCE_HOME", dir.path())
        .args(["cache-clean", "--dry-run"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("parse ~/.glance/config.json"));
    let out = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
        .env("GLANCE_HOME", dir.path())
        .env_remove("HERDR_PANE_ID")
        .arg("hook")
        .output()
        .unwrap();
    assert!(out.status.success());
}
