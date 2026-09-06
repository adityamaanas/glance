use std::process::Command;

#[test]
fn each_provider_runs_with_mock_cli_and_fallback_requires_opt_in() {
    let dir = tempfile::tempdir().unwrap();
    let helper = dir.path().join(if cfg!(windows) {
        "mock-provider.exe"
    } else {
        "mock-provider"
    });
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = Command::new(rustc)
        .arg("--edition=2021")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/mock-provider.rs"
        ))
        .arg("-o")
        .arg(&helper)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let transcript = dir.path().join("input.jsonl");
    std::fs::write(&transcript, include_str!("fixtures/codex.jsonl")).unwrap();
    let state = dir.path().join("state");
    std::fs::create_dir(&state).unwrap();
    std::fs::write(state.join("config.json"),r#"{"summary_models":{"claude":"test-model","codex":"test-model","gemini":"test-model","pi":"test-model","opencode":"test-model","cursor":"test-model"}}"#).unwrap();
    let log = dir.path().join("calls.txt");
    let command = || {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_glance-panel"));
        cmd.env("GLANCE_HOME", &state)
            .env_remove("GLANCE_MODEL")
            .env("HERDR_PANE_ID", "fixture-pane")
            .env("GLANCE_TEST_LOG", &log)
            .args(["--harness", "codex", "--transcript"])
            .arg(&transcript)
            .args(["summarize", "--session", "same-id"]);
        for kind in ["CLAUDE", "CODEX", "GEMINI", "PI", "OPENCODE", "CURSOR"] {
            cmd.env(format!("GLANCE_{kind}_BIN"), &helper);
        }
        cmd
    };
    for kind in ["claude", "codex", "gemini", "pi", "opencode", "cursor"] {
        let out = command()
            .args(["--summary-harness", kind])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{kind}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(value["topline"], "Retries verified");
        assert_eq!(value["backend"], format!("{kind}/test-model"));
    }
    let before = std::fs::read_to_string(&log).unwrap();
    assert_eq!(
        before.lines().collect::<Vec<_>>(),
        ["claude", "codex", "gemini", "pi", "opencode", "cursor"]
    );
    let absent = dir.path().join("missing");
    let out = command().env("GLANCE_CODEX_BIN", &absent).output().unwrap();
    assert!(!out.status.success());
    assert_eq!(before, std::fs::read_to_string(&log).unwrap());
    let out = command()
        .env("GLANCE_CODEX_BIN", &absent)
        .args(["--summary-fallback", "claude"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["backend"], "claude/test-model");
    let out = command()
        .env("GLANCE_TEST_FAIL", "1")
        .args(["--summary-fallback", "claude"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let lines = std::fs::read_to_string(&log).unwrap();
    assert_eq!(lines.lines().count(), 8);
    assert_eq!(lines.lines().last(), Some("codex"));
    let out = command().arg("--no-model").output().unwrap();
    assert!(!out.status.success());
    std::fs::write(state.join("config.json"), r#"{"no_model":true}"#).unwrap();
    assert!(!command().output().unwrap().status.success());
    assert_eq!(lines, std::fs::read_to_string(&log).unwrap());
}
