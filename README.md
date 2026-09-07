<div align="center">

<img src="assets/readme/hero.svg" alt="glance — Pick up where you left off. A live orientation panel for your coding-agent session." width="960">

[![CI](https://github.com/adityamaanas/glance/actions/workflows/ci.yml/badge.svg)](https://github.com/adityamaanas/glance/actions/workflows/ci.yml)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-76d8cc?labelColor=172c35)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-efb78b?labelColor=172c35)](Cargo.toml)

**Your session, back in focus.**

A live panel beside your coding agent: the goal, the current work, the plan, and the loose ends.
Come back after a break without rereading the conversation.

[Quick start](#quick-start) · [User guide](docs/usage.md) · [How it works](docs/architecture.md) · [Roadmap](ROADMAP.md) · [Contribute](CONTRIBUTING.md)

</div>

---

## See the work at a glance

<img src="assets/readme/panel.svg" alt="Illustrative split-pane layout: a Claude Code conversation beside a Glance panel showing the goal, current work, plan, an open question, and a decision." width="960">

*Illustrative example using fictional session content. The terminal layout adapts to your pane.*

| Keep your bearings | Follow the details |
| :--- | :--- |
| **One stable goal.** Remember what the session is working toward. | **A living plan.** See completed steps, current work, and blockers. |
| **The open loops.** Keep unanswered questions and decisions in view. | **Separate workstreams.** Focus on one thread or see the whole session. |
| **A quick return.** Cached summaries appear when you reopen the panel. | **A second perspective.** Switch to the rail to see items arranged by workstream. |
| **Check the evidence.** Open the transcript turns behind an item. | **Keep your own reminders.** Add todos whose wording stays yours. |
| **See relationships.** Explore a graph or export it as offline HTML. | **Choose your agent.** Read Claude, Codex, Gemini, pi, OpenCode and Cursor conversations. |

Glance reads local transcripts and exports; Cursor IDE can supply new events through optional hooks. Summaries run through a separate invocation of your selected agent CLI using its configured login. The default provider matches the transcript's agent. See the [compatibility matrix](docs/compatibility.md) for fixture coverage and live verification limits.

## Quick start

**Platforms:** Windows, macOS and Linux. Building from source requires Rust 1.88 or newer. Model summaries require an installed and authenticated agent CLI; `--no-model` works without one. [herdr](https://herdr.dev), tmux and Zellij provide automatic split placement, and manual splits work with a session ID.

```sh
# Install from source
cargo install --git https://github.com/adityamaanas/glance

# Enable herdr's Claude integration
herdr integration install claude

# Run inside the herdr pane hosting Claude Code
glance-panel attach
```

In a running Claude Code conversation, use `! glance-panel attach` to run the command in that pane.

The first Claude panel offers hook setup. Accept with `y`, or decline with `n`. Change this later with `glance-panel setup` or `glance-panel setup --remove`. Cursor IDE setup is explicit: `glance-panel setup --harness cursor`.

**Using another terminal?** Open your own split and follow a known session ID:

```sh
glance-panel --session <session-id>
glance-panel --harness codex --cwd /path/to/project
glance-panel --harness cursor --session <conversation-id> --no-model
```

The installed binary is **`glance-panel`**. Release archives and installers are configured for five targets; download them when a release containing these changes is published. Homebrew tap publication and crates.io credentials require maintainer setup. See [distribution](docs/distribution.md).

## Small controls, useful context

| Key | Action |
| :--- | :--- |
| `j` / `k` | Scroll down / up |
| Up / Down | Select an item |
| Enter / `e` | Open supporting transcript evidence |
| `r` | Request another summary |
| `v` | Toggle panel / rail view |
| `g` | Toggle relationship graph |
| `a` / `t` | Add a todo / select the todo list |
| `x` / `d` | Toggle / delete the selected todo |
| `[` / `]` | Move between workstreams |
| `0` | Show all workstreams |
| `p` | Toggle pinned focus / follow the conversation |
| `q` | Quit |

<details>
<summary><strong>Explore the rail view</strong></summary>

<br>
<img src="assets/readme/rail.svg" alt="Illustrative rail view with a trunk and two workstream lanes, showing plan steps, questions, and decisions in transcript order." width="800">

The rail arranges summary items by transcript turn and workstream. A workstream is a thread of work, such as reviewing a PR; it is separate from a Git branch. Narrow panes fold extra lanes into a count. This illustration uses fictional content.

</details>

## Designed to stay out of the way

- Transcript metadata supplies the title, branch, linked PR, and other available fields.
- A background model pass updates the summary after activity settles, while herdr supplies working/idle status.
- Versioned caches live in `~/.glance/`; a heuristic provides initial context when no cache exists.
- `--no-model` displays metadata and cached context without starting a summary invocation.

Summaries are interpretations and can be incomplete or wrong. Use the evidence drawer to check the conversation. Read [privacy and data handling](docs/privacy.md) for what is read, saved, and passed to your summary provider.

## Find your way around

| Guide | What you will find |
| :--- | :--- |
| [Usage](docs/usage.md) | Attach, sessions, focus, configuration, and files |
| [Troubleshooting](docs/troubleshooting.md) | Empty panels, hooks, model failures, and recovery |
| [Architecture](docs/architecture.md) | Transcript → summary → terminal, and module boundaries |
| [Roadmap](ROADMAP.md) | Shipped capabilities and planned milestones |
| [Implementation checklist](docs/implementation-checklist.md) | Detailed work plan and validation gates |
| [Changelog](CHANGELOG.md) | Changes by version |
| [Release notes](docs/release-notes.md) | Unreleased feature and reliability changes |
| [Compatibility](docs/compatibility.md) | OS, terminal and agent validation coverage |
| [Agent adapters](docs/agent-transcripts.md) | Discovery, formats and custom transcript paths |
| [Cursor](docs/cursor.md) | IDE hooks, CLI capture and export workflows |
| [Summary providers](docs/summary-providers.md) | Provider selection, models and helper restrictions |
| [Distribution](docs/distribution.md) | Archives, installers and maintainer release steps |

## Contributing

Bug reports, focused improvements, and documentation fixes are welcome. Start with the [contribution guide](CONTRIBUTING.md). Please use [private reporting](SECURITY.md) for security concerns.

Licensed under [Apache 2.0](LICENSE).
