// Compiled by the provider integration test. No network or model access.
use std::io::{Read, Write};
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let kind = if args.iter().any(|a| a == "--json-schema") { "claude" }
        else if args.first().map(String::as_str) == Some("exec") { "codex" }
        else if args.first().map(String::as_str) == Some("run") { "opencode" }
        else if args.iter().any(|a| a == "--no-tools") { "pi" }
        else if args.iter().any(|a| a == "ask") { "cursor" } else { "gemini" };
    assert_eq!(std::env::var("GLANCE_SUMMARY_HELPER").unwrap(), "1");
    assert!(std::env::var_os("HERDR_PANE_ID").is_none());
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    assert!(input.contains("Check retries") || args.iter().any(|a| a.contains("Check retries")));
    assert!(args.iter().any(|a| a == "test-model"));
    let log = std::env::var("GLANCE_TEST_LOG").unwrap();
    writeln!(std::fs::OpenOptions::new().create(true).append(true).open(log).unwrap(), "{kind}").unwrap();
    if std::env::var_os("GLANCE_TEST_FAIL").is_some() { eprintln!("fixture quota error"); std::process::exit(1); }
    let summary = r#"{"topline":"Retries verified","now":"Done","plan":[]}"#;
    let quoted = r#"{\"topline\":\"Retries verified\",\"now\":\"Done\",\"plan\":[]}"#;
    match kind {
        "claude" => println!("{{\"structured_output\":{summary}}}"),
        "codex" => {
            let n = args.iter().position(|a| a == "--output-last-message").unwrap();
            std::fs::write(&args[n+1],summary).unwrap();
            println!("transport log, not summary");
        }
        "gemini" => println!("{{\"response\":\"{quoted}\"}}"),
        "cursor" => println!("{{\"result\":\"{quoted}\"}}"),
        "opencode" => println!("{{\"type\":\"text\",\"part\":{{\"text\":\"{quoted}\"}}}}"),
        _ => println!("{summary}"),
    }
}
