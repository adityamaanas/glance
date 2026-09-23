# Roadmap

[Documentation](docs/README.md) · [Implementation checklist](docs/maintainers/implementation-checklist.md) · [Compatibility](docs/explanation/compatibility.md)

The following capabilities are merged on `main` and unreleased until a version is
published.
The compatibility matrix distinguishes fixture/process checks from live use.

## Implemented

| Area | Capabilities |
| --- | --- |
| Orientation | Goal, current work, plans, open questions, decisions, blockers and workstream focus |
| Navigation | Panel, rail, keyboard/mouse selection, evidence drawer and relationship graph |
| Export | Offline HTML graph with search, workstream filtering and source excerpts |
| Personal todos | Panel/CLI editing, per-agent stores, evidence-backed status updates and explicit carry |
| Discovery | Session picker, latest session by cwd, direct ID and custom transcript paths |
| Placement | herdr, tmux, Zellij and manual splits |
| Activity | herdr status, Claude/Cursor stop markers and settled-growth fallback |
| Setup | Idempotent Claude/Cursor hook setup/removal and optional Claude plugin |
| Platforms | Windows named pipes, Unix sockets, Windows/macOS/Linux CI |
| Agents | Claude Code, Codex, Gemini CLI, pi, OpenCode and Cursor CLI/IDE adapters |
| Summaries | Matching-agent CLI provider, explicit overrides/fallback, model settings, prompt and refresh controls |
| Reliability | Bounded process I/O, stale-generation rejection, rewrite detection, atomic state and locked todos |
| Long sessions | Forward chunks retain early turns; individual message excerpts remain clipped |
| Distribution | Five target archives, checksums, native installer smoke tests and a manual crate workflow |

## Distribution

Glance installs with one command from GitHub releases (shell and PowerShell
installers, five platforms). Planned channels, in order:

- **crates.io** (`cargo install glance-panel`, and `cargo binstall glance-panel`
  for prebuilt binaries): the manual publish workflow exists; it needs a
  crates.io token in a protected `crates-io` environment and a first reviewed
  publish.
- **Homebrew** (`brew install adityamaanas/tap/glance-panel`): the formula is
  already generated with each release; it needs a `homebrew-tap` repository,
  a publishing token, and the tap enabled in `dist-workspace.toml`.
- **Other package managers** (for example winget, Scoop, AUR or Nix) once there
  is demand.

## Live validation work

- Record authenticated live CLI versions for each provider and test actual
  Cursor IDE/CLI sessions after setup. Mock contracts do not establish every
  installed CLI version's behavior.
- Exercise herdr interactively on Windows/macOS/Linux and Zellij in a real
  terminal. Windows named-pipe tests and Zellij command tests already exist.
- Broaden terminal appearance checks across themes/fonts as live environments
  become available.

Cursor IDE uses visible hook events and exports. Historical private-database
ingestion is not an advertised capability. See [Follow Cursor](docs/how-to/cursor.md) for
capture scope and limitations.
