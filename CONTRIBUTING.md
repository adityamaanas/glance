# Contributing to Glance

Small, focused contributions are welcome: bug fixes, clearer docs, terminal usability, and realistic compatibility fixtures.

## Start here

- Check [issues](https://github.com/adityamaanas/glance/issues) and the [roadmap](ROADMAP.md) before starting a substantial feature.
- Explain the user problem and intended behavior.
- Use focused branches and PRs. Keep commits granular: one coherent, checked change per commit.
- Keep discussion constructive, respectful, and about the work.

## Development

Use current stable Rust on macOS or Linux. Claude Code and herdr are needed for live integration checks; unit tests should use local fixtures and avoid paid model calls.

```sh
git clone https://github.com/adityamaanas/glance.git
cd glance
cargo build
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

CI runs these checks on Linux and macOS. See [architecture](docs/architecture.md) for module responsibilities.

## What a good PR contains

- The concrete problem and resulting behavior, with a reproduction where relevant.
- Tests at the appropriate boundary: sanitized format fixtures and deterministic process/event tests.
- What was tested and what still needs a live environment.
- Updated documentation when commands, configuration, support, or behavior change.

Do not commit tokens, personal transcripts, caches, or agent databases. Replace fixture content with fictional data while preserving the shape needed to reproduce the issue. Do not make real model calls in CI.

For visual changes, check narrow and wide layouts, Unicode width, and light and dark backgrounds. Include useful alt text for documentation assets and label illustrations.

## Documentation map

The README introduces the project and gets users started. Details belong in `docs/`. Keep shipped functionality separate from planned work in `ROADMAP.md`, and record user-visible changes in `CHANGELOG.md`.

For vulnerabilities, follow [SECURITY.md](SECURITY.md).
