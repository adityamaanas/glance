# User guide

[← Home](../README.md) · [Troubleshooting](troubleshooting.md) · [Privacy](privacy.md)

## Install

Use current stable Rust and Cargo on macOS or Linux:

```sh
cargo install --git https://github.com/adityamaanas/glance
# Or, from a checkout:
cargo install --path .
```

Glance currently integrates with Claude Code 2.1 and herdr 0.8+. Install Claude Code, log in, and run `herdr integration install claude` before automatic attachment. Prebuilt releases, Homebrew, and crates.io distribution are planned.

## Attach a panel

```sh
glance-panel attach
glance-panel attach --ratio 0.35
glance-panel attach --force
glance-panel --pane w7:p5
```

Run `attach` inside the herdr pane hosting Claude Code. At the Claude prompt, prefix it with `!`. The ratio is the fraction of width requested for the panel; use a value between 0 and 1.

Attach checks sibling panes for an existing Glance process, reuses an idle sibling shell if available, or creates a split. If other panes exist and none can be reused, `--force` allows another split. Consider the idle-shell reuse behavior when arranging panes.

Fresh sessions may not have a transcript until the first prompt. Glance waits for it. In pane-following mode, status changes let Glance discover a different session after `/clear` or resume.

## Follow a session directly

```sh
glance-panel --session <session-id>
glance-panel --session <session-id> --no-model
glance-panel summarize --session <session-id>
```

Session IDs correspond to `.jsonl` filenames under `~/.claude/projects/<project>/`. Direct mode works in a split you open yourself and uses settled transcript growth to schedule summaries. `summarize` prints summary JSON and seeds the cache; it invokes the model.

## Navigate

| Key | Action |
| --- | --- |
| `j` / Down, `k` / Up | Scroll |
| `r` | Request another summary pass |
| `v` | Toggle panel and rail |
| `[` / Left, `]` / Right | Cycle focus and pin it |
| `0` | Pin focus to all workstreams |
| `p` | Toggle pinned focus and automatic following |
| `q`, Esc, Ctrl+C | Quit |

Workstreams appear when the summary identifies separate threads. Focusing one shows its items plus dimmed trunk items. The rail is a view of the current summary, not a complete event history.

## Configure summaries

`GLANCE_MODEL` overrides the default in [`src/summary.rs`](../src/summary.rs). Choose a model available to your Claude account:

```sh
GLANCE_MODEL=sonnet glance-panel --session <session-id>
```

`--no-model` avoids summary invocations for the panel while still reading metadata and cached context. Refresh settings, prompt overrides, and a model configuration file are planned.

## Automatic opening

```sh
glance-panel hook --install
glance-panel hook --uninstall
```

The panel offers installation once. `y` accepts and `n` records a decline. Installation adds a SessionStart command to `~/.claude/settings.json` and backs up the file first. The hook skips non-herdr contexts, subagents, and detected print-mode ancestors, and logs decisions.

## Files

| Path | Purpose |
| --- | --- |
| `~/.claude/projects/<project>/<session>.jsonl` | Source transcript; read only |
| `~/.glance/<session>.json` | Summary, processed turn count, model, timestamp, cache version |
| `~/.glance/config.json` | Answer to the automatic-hook offer |
| `~/.glance/hook.log` | Hook decisions and errors |
| `~/.claude/settings.json` | Hook registration, only on installation/removal |
| `~/.claude/settings.json.bak-glance` | Backup taken before writing hook settings |
