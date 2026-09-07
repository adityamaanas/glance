# Compatibility and validation

[← Home](../README.md) · [Adapters](agent-transcripts.md) · [Summary providers](summary-providers.md)

Coverage recorded on 2026-09-07. Fixture support, process contract checks and
authenticated live operation are different validation levels. No live provider
version is certified by the mock tests below.

## Platforms and terminals

| Environment | Validated | Remaining live coverage |
| --- | --- | --- |
| Windows x86-64 | Local Rust tests/lint, named-pipe framing/deadlines, hook wrappers, release archive and PowerShell installer | Interactive herdr and actual agent sessions |
| Linux x86-64 | Local WSL tests/lint, Unix socket tests, tmux interaction, archive and shell installer; Ubuntu release CI | More terminal/agent combinations |
| Linux ARM64 | Ubuntu 22.04 ARM release build and native installer/binary smoke | Interactive session use |
| macOS Apple Silicon | macOS 14 release build and native installer/binary smoke; macOS Rust CI | Interactive agent/terminal use |
| macOS Intel | macOS 15 Intel release build and native installer/binary smoke | Interactive agent/terminal use |
| herdr | Socket/named-pipe protocol tests, reconnect refresh, session changes, argument forwarding and metadata behavior | Full interactive matrix across OS versions |
| tmux | Real isolated Linux split/panel/todo smoke and quoted command tests | Other OS/terminal combinations |
| Zellij | Split command/argument tests | Live split/session interaction |
| Manual terminal split | Direct-session mode and narrow/wide rendering tests | Individual terminal/font/theme combinations |

Rust 1.88 is the tested minimum. Release builds include SQLite, so prebuilt
binaries do not require a separate SQLite installation. Linux binaries target
glibc; Alpine/musl and native Windows ARM64 builds are outside this matrix.
See [distribution](distribution.md) for release details.

## Agent adapters and summary providers

| Agent | Transcript coverage | Summary helper coverage | Authenticated live version |
| --- | --- | --- | --- |
| Claude Code | JSONL user/assistant/tool outcomes, metadata, partial UTF-8, truncation and earlier rewrites | Structured output/text responses, stdin, timeout/pipe tests and mock launch | Not recorded in this validation |
| Codex | Rollout event messages, tool outputs, duplicate transport suppression and rollback | `codex exec` contract, schema/output files and mock launch | Not recorded |
| Gemini CLI | Legacy JSON and JSONL replacement/rewind behavior, visible-text filtering | Headless JSON contract, deny policy and mock launch | Not recorded |
| pi | JSONL parent-chain selection and visible messages/tools | Print contract, disabled tools/context extensions and mock launch | Not recorded |
| OpenCode | Read-only SQLite with committed WAL data and exported messages | JSON events, deny permissions and mock launch | Not recorded |
| Cursor CLI | Stream JSON, visible event normalization and exact stdout forwarding | Ask/JSON invocation and mock launch | Not recorded |
| Cursor IDE | Exported text/JSONL and newly captured visible hook events; wrapper execution and settings preservation on Windows/POSIX | Uses the Cursor CLI summary provider | Not recorded; actual IDE event delivery still needs a live check |

Mock provider tests run real subprocesses with synthetic executables and
fictional transcripts. They check argument, input and response behavior without
authentication or model usage. They cannot prove a new external CLI version
still supports every flag or honors every restriction. See the provider guide
for isolation limits and executable overrides.

Cursor IDE hooks capture future visible events, not all old chats. CLI capture
does not launch the agent itself. Private Cursor databases and hidden reasoning
are not ingestion sources.

## Review evidence

- [Release matrix and installer smoke](https://github.com/adityamaanas/glance/actions/runs/34120474986)
  passed on all five targets.
- [Release packaging PR](https://github.com/adityamaanas/glance/pull/10) records
  the Windows/Linux local installer checks and crate dry run.
- [Consistency PR](https://github.com/adityamaanas/glance/pull/11) records 35 unit
  and 8 integration tests on Windows, plus 39 unit and 10 integration tests on
  Linux, with lint passing on both.

Before describing a new version as live-compatible, record the actual CLI,
terminal and OS versions, a sanitized reproduction, and the resulting behavior.
