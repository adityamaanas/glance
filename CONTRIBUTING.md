# Contributing to Glance

Small, focused contributions are welcome: bug fixes, clearer docs, terminal usability, and realistic compatibility fixtures.

## Start here

- Check [issues](https://github.com/adityamaanas/glance/issues) and the [roadmap](ROADMAP.md) before starting a substantial feature.
- Explain the user problem and intended behavior.
- Use focused branches and PRs. Keep commits granular: one coherent, checked change per commit.
- Keep discussion constructive, respectful, and about the work.

## Development

Use current stable Rust on Windows, macOS or Linux (minimum Rust 1.88). Agent CLIs and the relevant terminal are needed for live integration checks; automated tests use local fixtures and avoid paid model calls.

```sh
git clone https://github.com/adityamaanas/glance.git
cd glance
cargo build
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

CI runs these checks on Windows, Linux and macOS, plus a Rust 1.88 check. Release CI builds and smoke-tests installers on five targets. See [how Glance works](docs/explanation/how-it-works.md) for module responsibilities and [compatibility](docs/explanation/compatibility.md) for live verification gaps.

## What a good PR contains

- The concrete problem and resulting behavior, with a reproduction where relevant.
- Tests at the appropriate boundary: sanitized format fixtures and deterministic process/event tests.
- What was tested and what still needs a live environment.
- Updated documentation when commands, configuration, support, or behavior change.

Do not commit tokens, personal transcripts, caches, or agent databases. Replace fixture content with fictional data while preserving the shape needed to reproduce the issue. Do not make real model calls in CI.

For visual changes, check narrow and wide layouts, Unicode width, and light and dark backgrounds. Include useful alt text for documentation assets and label illustrations.

## Documentation

Follow the [documentation standard](docs/STANDARD.md). In short:

- Put each change in the right kind of page: the [getting-started tutorial](docs/getting-started.md), a task-focused [how-to guide](docs/how-to/), a [reference](docs/reference/) page, or an [explanation](docs/explanation/).
- Update docs in the same PR as the behavior change, and add a user-facing line to `CHANGELOG.md` under "Unreleased". CI requires this whenever `src/` changes; for internal-only changes (refactors, tests), add the `no-changelog` label instead.
- Don't bump the version in feature PRs; it changes only in a release PR.
- After changing a command or flag, regenerate the command-line reference with `GLANCE_UPDATE_DOCS=1 cargo test --test cli_reference`.
- After changing the panel's layout, regenerate the screenshots with `cargo build --release && python3 scripts/capture-screens.py` (needs tmux).
- Check links with `python3 scripts/check-docs.py`.

Keep shipped functionality separate from planned work in `ROADMAP.md`.

For vulnerabilities, follow [SECURITY.md](SECURITY.md).
