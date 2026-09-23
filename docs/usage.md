# User guide

[← Home](../README.md) · [Troubleshooting](troubleshooting.md) · [Privacy](privacy.md)

## Install and choose an agent

Build on Windows, macOS or Linux with Rust 1.88 or newer:

```sh
cargo install --git https://github.com/adityamaanas/glance
# From a checkout:
cargo install --path .
```

See [distribution](distribution.md) for archives, installers and release setup.
Model summaries need an installed and authenticated agent CLI. Select the
transcript format with `--harness claude|codex|gemini|pi|opencode|cursor`.
Claude is the default outside herdr; herdr can detect the pane's agent.
[Adapter documentation](agent-transcripts.md) covers formats and locations.
[Cursor](cursor.md) covers both IDE hooks and CLI capture.

## Find and follow a session

```sh
glance-panel --harness codex pick --list
glance-panel --harness codex pick --cwd /path/to/project --latest
glance-panel --harness codex --cwd /path/to/project
glance-panel --harness codex --session <id>
glance-panel --harness cursor --transcript /path/to/export.txt --session <id>
glance-panel --harness codex --session <id> --no-model
```

The picker prints an ID; `--cwd` follows the latest matching session. Use
`--transcript` for a nonstandard file/export location. Fresh sessions may need
their first prompt before a transcript exists. A known Claude session inside
herdr can wait at its expected path; other agents may require another attempt
after their transcript is created.

## Place a panel

```sh
glance-panel attach
glance-panel attach --ratio 0.35
glance-panel attach --force
glance-panel --pane w7:p5
glance-panel --harness codex attach --backend tmux --session <id>
glance-panel --harness codex attach --backend zellij --session <id>
```

Run herdr attachment in the pane hosting the agent. At a Claude prompt use
`! glance-panel attach`. Auto placement detects herdr, tmux or Zellij; pass
`--backend` to choose. Ratios must be strictly between 0 and 1.

herdr attachment reuses an idle sibling shell when available, avoids duplicate
panels, and needs `--force` to split again when other panes are busy. Pane
following switches sessions after a detected clear/resume. For other terminals,
open a split yourself and use `--session`.

## Navigate and inspect evidence

| Key | Action |
| --- | --- |
| Up / Down, mouse wheel or click | Select an item (the mouse is captured only while a list is open, so text selection works in the normal panel) |
| `j` / `k` | Scroll the view or evidence |
| Enter / `e` | Toggle the evidence drawer |
| `v` | Toggle panel and rail |
| `g` | Toggle the relationship graph |
| `[` / Left, `]` / Right | Cycle and pin workstream focus |
| `0` | Pin focus to all workstreams |
| `p` | Toggle pinned focus and automatic following |
| `r` | Request a refresh, bypassing the interval |
| `a` | Add a todo; Enter saves, Esc cancels |
| `t` | Switch between summary-item and todo selection |
| `x` / `d` | Toggle / delete the selected todo |
| `q`, Esc, Ctrl+C | Quit; Esc first dismisses an open input/detail mode |

Items reference transcript turns. Invalid references and relationship cycles
are removed. Older completed workstreams collapse in the branch strip; focusing
them makes them visible. The rail and graph show current summary state.

```sh
glance-panel --harness codex graph --session <id>
glance-panel --harness codex graph --session <id> --html graph.html --open
```

Exports use the existing cache without a model call, work offline and contain
conversation excerpts. See [evidence and graphs](evidence-and-graph.md).

## Personal todos

```sh
glance-panel --harness codex todo "Check the migration" --session <id>
glance-panel --harness codex todo --session <id>
glance-panel --harness codex todo --session <id> --set todo-1 --status done
glance-panel --harness codex todo --session <id> --delete todo-1
glance-panel --harness codex todo --session <new-id> --carry-from <old-id>
```

Inside herdr, omitting `--session` selects the current pane's session and agent;
an explicit `--harness` wins. Other contexts require a session ID. Carrying
todos is explicit and resets copies to pending.

The model can update statuses with evidence, but cannot create, rewrite or
delete your reminders. Manual changes win until newer evidence is available.
File locking and atomic writes coordinate panel and CLI changes.
See [personal todos](personal-todos.md).

## Configure summaries

Edit `~/.glance/config.json` (or `$GLANCE_HOME/config.json`):

```json
{
  "refresh_seconds": 60,
  "no_model": false,
  "cache_retention_days": 30,
  "sidebar_metadata": false,
  "prompt": "Keep the current step concise."
}
```

The default refresh interval is 30 seconds. Model precedence is `--model` (for
the requested summary agent only), then `summary_models[provider]`. `GLANCE_MODEL`
and config `model` apply to Claude only; other providers otherwise use their CLI
default, and a fallback never inherits `--model`. The summary provider defaults to the transcript's agent. Select another
with `--summary-harness`; fallback requires `--summary-fallback` and applies
only when the chosen executable is missing. See
[summary providers](summary-providers.md) for the CLI contracts.

`--no-model` or config `no_model: true` disables model execution, including
`summarize`. Without that setting, summarize invokes the provider and seeds
the cache:

```sh
glance-panel --harness codex summarize --session <id>
glance-panel cache-clean --older-than-days 30 --dry-run
glance-panel cache-clean --older-than-days 30
```

Long sessions use forward chunks so early turns are not silently skipped.
Individual message/tool excerpts are still clipped. The footer shows successful
call counts and cost when reported by the provider; a trailing `+` means some
calls reported no cost, so the figure is a lower bound. It is not a billing meter.

## Optional setup and files

```sh
glance-panel setup
glance-panel setup --remove
glance-panel setup --harness cursor
glance-panel setup --harness cursor --remove
```

Claude setup registers SessionStart, Stop and StopFailure hooks. Cursor setup
registers visible event capture hooks. Setup preserves unrelated hooks and
backs up settings. Re-run it after moving the binary. See
[session setup](session-setup.md) for sidebar metadata, placement and the optional
Claude plugin.

| Path under `~/.glance/` | Contents |
| --- | --- |
| `config.json` | Preferences and summary controls |
| `<session>.json` | Claude summary cache |
| `<agent>--<session>.json` | Other agents' summary caches |
| `<key>.todos.json` / `<key>.todos.lock` | Personal reminders and writer lock |
| `<key>.stop` | Turn-end activity marker |
| `hook.log` | Hook decisions and errors |
| `cursor/` | Captured visible Cursor events and hook wrappers |

`GLANCE_HOME` relocates Glance state. `CLAUDE_CONFIG_DIR` relocates Claude
settings/transcripts. Other agent roots are in the adapter guide. Source
transcripts/databases are read only; Cursor capture writes Glance's own files.
