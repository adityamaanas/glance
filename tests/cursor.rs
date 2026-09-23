use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn cursor_setup_wrapper_capture_and_removal_preserve_user_settings() {
    let dir = tempfile::tempdir().unwrap();
    let cursor = dir.path().join("cursor space");
    let state = dir.path().join("state space ' & %mark% !");
    std::fs::create_dir_all(&cursor).unwrap();
    let settings = cursor.join("hooks.json");
    let unrelated = json!({"command":"echo keep","timeout":15});
    std::fs::write(
        &settings,
        json!({"version":1,"extra":"keep","hooks":{"stop":[unrelated.clone()]}}).to_string(),
    )
    .unwrap();
    let run = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
            .env("GLANCE_HOME", &state)
            .env("GLANCE_CURSOR_HOME", &cursor)
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
    let original = std::fs::read(&settings).unwrap();
    run(&["setup", "--harness", "cursor"]);
    let first = std::fs::read(&settings).unwrap();
    run(&["setup", "--harness", "cursor"]);
    assert_eq!(first, std::fs::read(&settings).unwrap());
    // A repeated setup keeps the backup taken before Glance first changed the file.
    assert_eq!(
        std::fs::read(settings.with_extension("json.bak-glance")).unwrap(),
        original
    );
    // Actually execute the installed platform wrapper without inheriting GLANCE_HOME.
    let invoke = |event: Value| {
        #[cfg(windows)]
        let mut cmd = {
            let mut c = Command::new("cmd.exe");
            c.args(["/D", "/C", "hooks\\glance-capture.cmd"]);
            c
        };
        #[cfg(not(windows))]
        let mut cmd = Command::new(cursor.join("hooks/glance-capture.sh"));
        let mut child = cmd
            .current_dir(&cursor)
            .env_remove("GLANCE_HOME")
            .env_remove("GLANCE_SUMMARY_HELPER")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(event.to_string().as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success());
        assert_eq!(
            String::from_utf8_lossy(&out.stdout).trim(),
            "{}",
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    for (event, content) in [
        ("beforeSubmitPrompt", json!({"prompt":"Check retries"})),
        (
            "postToolUseFailure",
            json!({"tool_name":"Shell","error_message":"Test failed"}),
        ),
        ("afterAgentResponse", json!({"text":"Fixed retries"})),
        ("stop", json!({})),
    ] {
        let mut value = json!({"conversation_id":"test","generation_id":"one","hook_event_name":event,"workspace_roots":[dir.path()],"user_email":"not-stored","thought":"not-stored"});
        value
            .as_object_mut()
            .unwrap()
            .extend(content.as_object().unwrap().clone());
        invoke(value);
    }
    let captured = std::fs::read_to_string(state.join("cursor/test.jsonl")).unwrap();
    assert!(!captured.contains("not-stored"));
    assert!(state.join("cursor--test.stop").exists());
    let out = run(&["--harness", "cursor", "transcript", "--session", "test"]);
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["turns"].as_array().unwrap().len(), 3);
    assert!(v.to_string().contains("Test failed"));
    // Corrupt Glance config cannot make the observational hook block Cursor.
    std::fs::write(state.join("config.json"), "broken").unwrap();
    invoke(
        json!({"hook_event_name":"afterAgentThought","conversation_id":"test","text":"not-stored"}),
    );
    std::fs::write(state.join("config.json"), "{}").unwrap();
    run(&["setup", "--harness", "cursor", "--remove"]);
    let v: Value = serde_json::from_slice(&std::fs::read(&settings).unwrap()).unwrap();
    assert_eq!(v["hooks"]["stop"], json!([unrelated]));
    assert_eq!(v["extra"], "keep");
    let broken = "not JSON";
    std::fs::write(&settings, broken).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
        .env("GLANCE_HOME", &state)
        .env("GLANCE_CURSOR_HOME", &cursor)
        .args(["setup", "--harness", "cursor"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(std::fs::read_to_string(&settings).unwrap(), broken);
}

#[test]
fn cursor_cli_stream_is_forwarded_and_captured_without_duplicate_result() {
    let dir = tempfile::tempdir().unwrap();
    let fixture = include_bytes!("fixtures/cursor.jsonl");
    let mut child = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
        .env("GLANCE_HOME", dir.path())
        .arg("cursor-stream")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(fixture).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, fixture);
    let out = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
        .env("GLANCE_HOME", dir.path())
        .args(["--harness", "cursor", "transcript", "--session", "same-id"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["turns"].as_array().unwrap().len(), 3);
    assert!(dir.path().join("cursor--same-id.stop").exists());
}

#[test]
fn cursor_stream_forwards_everything_when_capture_fails() {
    let dir = tempfile::tempdir().unwrap();
    let fixture = include_bytes!("fixtures/cursor.jsonl");
    // An invalid record and an oversized record must reach stdout unchanged, and
    // capture must continue with the records after them.
    let mut input = b"not json\n".to_vec();
    input.extend_from_slice(fixture);
    input.extend(std::iter::repeat_n(b'x', 9 * 1024 * 1024));
    input.extend_from_slice(b"\ntrailing text without newline");
    let mut child = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
        .env("GLANCE_HOME", dir.path())
        .arg("cursor-stream")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = {
        let input = input.clone();
        std::thread::spawn(move || stdin.write_all(&input).unwrap())
    };
    let out = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(out.status.success());
    assert_eq!(out.stdout, input);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("capture skipped"), "{stderr}");
    let out = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
        .env("GLANCE_HOME", dir.path())
        .args(["--harness", "cursor", "transcript", "--session", "same-id"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["turns"].as_array().unwrap().len(), 3);
}
