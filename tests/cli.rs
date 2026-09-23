use std::process::Command;

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
