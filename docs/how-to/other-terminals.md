# Use tmux, Zellij or any other terminal

[Docs home](../README.md) · [herdr](herdr.md) · [Files and environment](../reference/files-and-environment.md)

Glance does not need herdr. It can open its own split in tmux or Zellij, or run in any pane or window you open yourself.

**You need:** Glance installed ([Getting started](../getting-started.md)) and a session with at least one prompt sent, so its transcript exists.

## tmux

From a shell inside tmux, in your project directory:

```sh
glance-panel attach --backend tmux
```

This splits the current window to the right (30% wide; change it with `--ratio 0.4`) and follows the most recent session in the current directory. Name a session instead with `--session <session-id>`, or another project with `--cwd <path>`.

## Zellij

```sh
glance-panel attach --backend zellij
```

This opens a pane named `glance` to the right. Zellij sizes tiled panes itself, so `--ratio` has no effect.

`attach` without `--backend` picks herdr, then tmux, then Zellij, whichever you are inside. Each run opens a new pane.

## Any other terminal

Open a split, tab or window yourself, go to the project directory and run:

```sh
glance-panel --cwd .
```

Other ways to choose the session:

| Command | Follows |
| --- | --- |
| `glance-panel --cwd .` | The most recent session in this project |
| `glance-panel --session <session-id>` | One specific session |
| `glance-panel` | A session you pick from a list (outside herdr) |

## Find a session ID

```sh
glance-panel pick                          # choose from the 30 most recent
glance-panel pick --cwd . --latest         # print the newest ID for this project
glance-panel pick --query "webhook" --list # every match, as JSON, for scripts
```

`pick` prints the chosen ID so you can pass it to `--session`. It matches the working directory recorded in each transcript, not just the folder name.

## What carries over into a new pane

These options given to `attach` are passed on to the new panel: `--harness`, `--transcript`, `--no-model`, `--model`, `--summary-harness`, `--summary-fallback`, `--refresh-seconds` and `--sidebar`. In tmux and Zellij, Glance's location and executable variables (`GLANCE_HOME`, `CLAUDE_CONFIG_DIR`, `GLANCE_MODEL`, the agent directory variables and the `GLANCE_*_BIN` overrides listed in [Files and environment](../reference/files-and-environment.md)) are passed on too.

## What to expect

- The header shows `○ no herdr`. Without herdr, Glance cannot see whether the agent is busy, so it waits until the transcript has been quiet for two seconds before summarizing. Running `glance-panel setup` adds turn-end hooks that make this faster for Claude Code (see [herdr](herdr.md#open-the-panel-automatically-for-every-session); the turn-end hooks work in any terminal).
- The panel stays on the session you chose. After `/clear`, start the panel again or use `--cwd .`, which picks the newest session when it starts.
