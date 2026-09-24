# Changelog

## Unreleased

- `glance-panel setup` no longer overwrites its backup of Claude Code's `settings.json` on every run. The backup now keeps the file as it was before Glance first changed it, as Cursor setup already did.
- Release notes no longer suggest `brew install glance-panel`, which does not work until a Homebrew tap exists. Use the one-line installers.

## 0.3.0 (2026-09-23)

First release with prebuilt binaries and one-line installers. Existing summary caches rebuild once after upgrading.

### Added

- **More agents.** Follow Codex, Gemini CLI, pi, OpenCode and Cursor sessions as well as Claude Code (`--harness`). Under herdr the agent is detected automatically.
- **Cursor capture.** `setup --harness cursor` records new IDE chats through Cursor's hooks; `cursor-stream` records Cursor CLI runs while passing the output through unchanged.
- **Summaries from any agent.** Summaries are written by the session's own agent CLI by default, with `--summary-harness`, per-agent models (`summary_models`) and an opt-in fallback for a missing CLI (`--summary-fallback`).
- **Evidence and graph.** Every summary item links to the transcript turns behind it (`Enter`/`e`), and a relationship graph (`g`) shows how steps, questions and decisions follow from each other. `graph --html` exports an offline, searchable page.
- **Personal todos.** Add your own reminders in the panel (`a`, `t`, `x`, `d`) or with `glance-panel todo`. Only you write them; summaries can only update their status, with evidence. `--carry-from` copies them to a new session.
- **Any terminal.** `attach` opens the panel in herdr, tmux or Zellij; `--cwd .` follows the latest session in a project; `pick` lists and chooses sessions.
- **Setup command.** `glance-panel setup` installs Claude Code's session-start and turn-end hooks (`--remove` undoes it). An optional Claude Code plugin installs the same hooks.
- **herdr sidebar.** `--sidebar` or `sidebar_metadata` shows the current step and plan progress in herdr's sidebar.
- **Windows support**, including herdr's named pipe (`HERDR_SOCKET_PATH`) and PowerShell/Command Prompt hooks.
- **Controls.** `--no-model`/`no_model`, `--refresh-seconds`, a custom `prompt`, `cache-clean` and `cache_retention_days`. The footer shows summary calls and reported cost (`+` marks a lower bound).
- **One-line installers** and prebuilt binaries for macOS (Intel and Apple Silicon), Linux (x86-64 and ARM64) and Windows. See the README's install section.

### Changed

- Up/Down now open the item list and select items; `j`/`k` scroll the panel.
- The mouse is only captured while a list is open, so text selection and scrollback work in the main panel.
- `--model` applies only to the requested summary agent. `GLANCE_MODEL` and config `model` apply to Claude only, and a fallback never inherits `--model`.
- Long sessions are summarized in forward chunks, so early turns are never skipped.
- `hook --install`/`--uninstall` are superseded by `setup`/`setup --remove` (still accepted).
- Minimum Rust version is 1.88.
- Documentation rewritten: a getting-started tutorial, a guide to the panel with real screenshots, task-focused guides, and reference pages checked against the code.

### Fixed

- Summaries read Claude's structured output correctly and no longer hang on large output.
- A summary for a previous session can no longer overwrite the current one after `/clear` or a resume.
- Installing the hook keeps other hooks in the same group, and settings, caches and todos are written atomically.
- Rewritten or truncated transcripts and split UTF-8 characters are handled; stale caches are detected.
- The herdr connection refreshes its status after reconnecting and no longer polls while idle.

## 0.2.0

- Branches: the summary models separate threads of work (for example one PR
  review among several) with a focus on the thread the newest turns belong to.
  The panel shows a `BRANCHES` strip, filters plan, questions and decisions to
  the focused thread, and lets you move or pin focus (`[` `]` `0` `p`).
- Rail view (`v`): trunk and one lane per branch, one row per item, drawn with
  box-drawing characters.
- Every item carries the transcript turn it arose at; turns are rendered to the
  model with absolute `[tN]` markers.
- `GLANCE_MODEL` overrides the summary model.
- `summarize` seeds the panel cache.
- Cache format versioned; earlier caches are discarded and rebuilt once.

## 0.1.0

- First working panel: header, topline, now, plan, open questions, decisions,
  blockers, last message from Claude.
- `attach` splits the herdr pane, reuses an idle sibling pane after a herdr
  restart, waits for the shell prompt and confirms the panel came up.
- Follows the pane to a new session after `/clear`; waits for a fresh session's
  first prompt.
- SessionStart hook (`hook`), offered once on first run inside the panel;
  skips subagents, non-herdr panes and print-mode runs.
- Summaries run through `claude -p` with a JSON schema on the user's own Claude
  login; heuristic fallback so the panel is never empty.
