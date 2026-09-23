//! Keeps docs/reference/cli.md identical to the binary's own `--help` output.
//!
//! Regenerate after changing a command or flag:
//!     GLANCE_UPDATE_DOCS=1 cargo test --test cli_reference
use std::process::Command;

const SUBCOMMANDS: &[&str] = &[
    "attach",
    "pick",
    "todo",
    "graph",
    "transcript",
    "summarize",
    "setup",
    "cache-clean",
    "cursor-stream",
    "hook",
];

fn help(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_glance-panel"))
        .args(args)
        .arg("--help")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap().replace("\r\n", "\n");
    text.lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

fn render() -> String {
    let mut doc = String::from(
        "# Command-line reference\n\n\
         [Docs home](../README.md) · [Keys](keys.md) · [Configuration](configuration.md) · \
         [Files and environment](files-and-environment.md)\n\n\
         <!-- Generated from `glance-panel --help` by tests/cli_reference.rs. Do not edit by hand:\n     \
         GLANCE_UPDATE_DOCS=1 cargo test --test cli_reference -->\n\n\
         Global options (such as `--harness`, `--transcript`, `--no-model` and the summary \
         options) can be given before or after a command.\n\n\
         ## glance-panel\n\n```text\n",
    );
    doc.push_str(&help(&[]));
    doc.push_str("\n```\n");
    for sub in SUBCOMMANDS {
        doc.push_str(&format!("\n## glance-panel {sub}\n\n```text\n"));
        doc.push_str(&help(&[sub]));
        doc.push_str("\n```\n");
    }
    doc
}

#[test]
fn cli_reference_matches_help_output() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/docs/reference/cli.md");
    let expected = render();
    if std::env::var_os("GLANCE_UPDATE_DOCS").is_some() {
        std::fs::write(path, &expected).unwrap();
        return;
    }
    let actual = std::fs::read_to_string(path)
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert!(
        actual == expected,
        "docs/reference/cli.md is out of date; run GLANCE_UPDATE_DOCS=1 cargo test --test cli_reference"
    );
}
