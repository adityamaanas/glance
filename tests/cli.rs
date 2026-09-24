use std::process::Command;

#[cfg(unix)]
#[test]
fn todos_infer_the_pane_agent_and_respect_explicit_harness() {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;
    use std::time::{Duration, Instant};
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("herdr.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let server = std::thread::spawn(move || {
        for _ in 0..3 {
            let deadline = Instant::now() + Duration::from_secs(15);
            let mut connection = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "todo did not query herdr");
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("{error}"),
                }
            };
            connection
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = String::new();
            BufReader::new(&connection).read_line(&mut request).unwrap();
            let request: serde_json::Value = serde_json::from_str(&request).unwrap();
            assert_eq!(request["method"], "agent.get");
            writeln!(connection, "{}", serde_json::json!({"result":{"agent":{"agent_status":"idle","agent_session":{"agent":"codex","value":"same-id"}}}})).unwrap();
        }
    });
    for (flags, expected_key) in [
        (vec![], "codex--same-id"),
        (vec!["--harness", "claude"], "same-id"),
        (vec!["--harness", "gemini"], "gemini--same-id"),
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
            .env("GLANCE_HOME", dir.path().join("state"))
            .env("HERDR_SOCKET_PATH", &socket)
            .env("HERDR_PANE_ID", "w1:p1")
            .env("CLAUDE_CONFIG_DIR", dir.path().join("missing-claude"))
            .env("CODEX_HOME", dir.path().join("missing-codex"))
            .env("GLANCE_GEMINI_HOME", dir.path().join("missing-gemini"))
            .args(flags)
            .args(["todo", "Remember this agent"])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            dir.path()
                .join("state")
                .join(format!("{expected_key}.todos.json"))
                .exists(),
            "expected {expected_key}, got {:?}; stdout: {}",
            std::fs::read_dir(dir.path().join("state"))
                .unwrap()
                .map(|p| p.unwrap().file_name())
                .collect::<Vec<_>>(),
            String::from_utf8_lossy(&result.stdout)
        );
    }
    server.join().unwrap();
}

#[test]
fn agent_fixtures_exclude_discarded_history_and_do_not_share_todos() {
    let dir = tempfile::tempdir().unwrap();
    for (kind, fixture) in [
        ("codex", include_str!("fixtures/codex.jsonl")),
        ("gemini", include_str!("fixtures/gemini.jsonl")),
        ("pi", include_str!("fixtures/pi.jsonl")),
        ("cursor", include_str!("fixtures/cursor.jsonl")),
        ("opencode", include_str!("fixtures/opencode.json")),
    ] {
        let path = dir.path().join(format!("{kind}.json"));
        std::fs::write(&path, fixture).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
            .env("GLANCE_HOME", dir.path())
            .args(["--harness", kind, "--no-model", "--transcript"])
            .arg(&path)
            .args(["transcript", "--session", "same-id"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}: {}",
            kind,
            String::from_utf8_lossy(&out.stderr)
        );
        let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(
            value["turns"].as_array().unwrap().len(),
            3,
            "{kind}: {value}"
        );
        assert!(!value.to_string().contains("Discard"), "{kind}: {value}");
        assert!(!value.to_string().contains("private reasoning"));
        assert!(!value.to_string().contains("duplicate transport"));
        let out = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
            .env("GLANCE_HOME", dir.path())
            .args(["--harness", kind, "todo", kind, "--session", "same-id"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let list: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(list.as_array().unwrap().len(), 1);
        assert_eq!(list[0]["text"], kind);
    }
}

#[test]
fn setup_is_idempotent_preserves_other_hooks_and_stop_records_no_text() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("claude");
    std::fs::create_dir_all(&config).unwrap();
    let settings = config.join("settings.json");
    let unrelated = serde_json::json!({"type":"command","command":"echo keep"});
    std::fs::write(
        &settings,
        serde_json::json!({"theme":"dark","hooks":{"Stop":[{"hooks":[unrelated.clone()]}]}})
            .to_string(),
    )
    .unwrap();
    let original = std::fs::read(&settings).unwrap();
    let backup = settings.with_extension("json.bak-glance");
    let run = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
            .env("GLANCE_HOME", dir.path().join("state"))
            .env("CLAUDE_CONFIG_DIR", &config)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        out
    };
    run(&["setup", "--yes"]);
    let first = std::fs::read(&settings).unwrap();
    run(&["setup"]);
    assert_eq!(first, std::fs::read(&settings).unwrap());
    let value: serde_json::Value = serde_json::from_slice(&first).unwrap();
    assert_eq!(
        value["hooks"]["SessionStart"][0]["hooks"][0]["args"],
        serde_json::json!(["hook"])
    );
    assert_eq!(value["hooks"]["Stop"].as_array().unwrap().len(), 2);
    let transcript = dir.path().join("session.jsonl");
    std::fs::write(&transcript, "{}\n").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
        .env("GLANCE_HOME", dir.path().join("state"))
        .env("CLAUDE_CONFIG_DIR", &config)
        .arg("hook")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(serde_json::json!({"hook_event_name":"Stop","session_id":"session","transcript_path":transcript,"last_assistant_message":"private words"}).to_string().as_bytes()).unwrap();
    assert!(child.wait_with_output().unwrap().status.success());
    let marker = std::fs::read_to_string(dir.path().join("state/session.stop")).unwrap();
    assert!(!marker.contains("private words"));
    run(&["setup", "--remove"]);
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&settings).unwrap()).unwrap();
    assert_eq!(value["theme"], "dark");
    assert_eq!(
        value["hooks"]["Stop"],
        serde_json::json!([{"hooks":[unrelated]}])
    );
    // Every run rewrote settings.json; the backup still holds the user's original.
    assert_eq!(std::fs::read(&backup).unwrap(), original);
}

#[test]
fn todo_cli_preserves_wording_and_requires_explicit_carry() {
    let dir = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
            .env("GLANCE_HOME", dir.path())
            .env("CLAUDE_CONFIG_DIR", dir.path().join("claude"))
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap()
    };
    let first = run(&["todo", "Ask about café rollout", "--session", "first"]);
    assert_eq!(first[0]["text"], "Ask about café rollout");
    let changed = run(&[
        "todo",
        "--session",
        "first",
        "--set",
        "todo-1",
        "--status",
        "done",
    ]);
    assert_eq!(changed[0]["status"], "done");
    assert_eq!(run(&["todo", "--session", "second"]), serde_json::json!([]));
    let carried = run(&["todo", "--session", "second", "--carry-from", "first"]);
    assert_eq!(carried[0]["text"], first[0]["text"]);
    assert_eq!(carried[0]["status"], "pending");
    assert_eq!(
        run(&["todo", "--session", "first", "--delete", "todo-1"]),
        serde_json::json!([])
    );
    assert_eq!(
        run(&["todo", "--session", "second"])[0]["text"],
        first[0]["text"]
    );
}

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
    let added = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
        .env("GLANCE_HOME", &state)
        .env("CLAUDE_CONFIG_DIR", &config)
        .args(["todo", "Keep the first evidence", "--session", "session"])
        .output()
        .unwrap();
    assert!(added.status.success());
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
    std::fs::write(&fixture, serde_json::json!({"structured_output":{"topline":"Review evidence", "plan":[{"id":"p1","text":"Inspect first turn","status":"done","source_turns":[0]}], "todo_updates":[{"id":"todo-1","status":"done","note":"First turn retained","source_turns":[0]}]},"total_cost_usd":0.01}).to_string()).unwrap();
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
    let todos: serde_json::Value =
        serde_json::from_slice(&std::fs::read(state.join("session.todos.json")).unwrap()).unwrap();
    assert_eq!(todos["items"][0]["status"], "done");
    assert_eq!(todos["items"][0]["text"], "Keep the first evidence");
    assert_eq!(todos["items"][0]["source_turns"], serde_json::json!([0]));
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
