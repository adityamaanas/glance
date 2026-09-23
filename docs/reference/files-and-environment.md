# Files and environment

[Docs home](../README.md) · [Configuration](configuration.md) · [Privacy](../explanation/privacy.md)

## What Glance stores

Everything Glance writes lives in one state directory: `~/.glance/`, or the directory named by `GLANCE_HOME`.

| Path | Contents | Safe to delete? |
| --- | --- | --- |
| `config.json` | Your [settings](configuration.md) | Yes; defaults apply |
| `<session-id>.json` | Claude Code summary cache | Yes; rebuilt on the next summary (costs a model call) |
| `<agent>--<session-id>.json` | Summary cache for another agent, for example `codex--<id>.json` | Yes; rebuilt as above |
| `<key>.todos.json` | Personal todos for one session (`<key>` is the session ID, prefixed with the agent for non-Claude sessions) | Only if you no longer want those todos |
| `<key>.todos.lock` | Lock file that coordinates todo edits | Yes, when no panel is open |
| `<key>.stop` | Turn-end marker from the Stop hooks: transcript path, size and time, no conversation text | Yes |
| `hook.log` | One line per hook run, with what it decided or why it failed | Yes |
| `cursor/<conversation-id>.jsonl` | Captured Cursor conversations ([Follow Cursor](../how-to/cursor.md)) | Only if you no longer want that capture |

`glance-panel cache-clean` removes only summary caches. Every write is atomic, so an interrupted write never leaves a half-written file.

## Files Glance changes outside its directory

Only when you run `setup`:

| Command | Changes | Backup |
| --- | --- | --- |
| `glance-panel setup` | Adds Glance's hooks to Claude Code's `settings.json` (`~/.claude/`, or `CLAUDE_CONFIG_DIR`) | `settings.json.bak-glance` |
| `glance-panel setup --harness cursor` | Adds hooks to `~/.cursor/hooks.json` and writes `~/.cursor/hooks/glance-capture.sh` (`.cmd` on Windows) | `hooks.json.bak-glance`, kept from the first change |

`--remove` removes only Glance's entries; your other hooks are untouched. Glance never modifies agent transcripts or databases.

## Environment variables

### Glance

| Variable | Effect |
| --- | --- |
| `GLANCE_HOME` | State directory (default `~/.glance`) |
| `GLANCE_MODEL` | Claude summary model, after `--model` and `summary_models.claude` and before `model` in `config.json` |
| `GLANCE_CLAUDE_BIN`, `GLANCE_CODEX_BIN`, `GLANCE_GEMINI_BIN`, `GLANCE_PI_BIN`, `GLANCE_OPENCODE_BIN`, `GLANCE_CURSOR_BIN` | Path to that agent's executable for summaries |

### Where agents keep their sessions

| Variable | Agent | Default location |
| --- | --- | --- |
| `CLAUDE_CONFIG_DIR` | Claude Code transcripts and settings | `~/.claude` |
| `CODEX_HOME` | Codex | `~/.codex` |
| `GLANCE_GEMINI_HOME` | Gemini CLI | `~/.gemini` |
| `PI_CODING_AGENT_SESSION_DIR` | pi | `~/.pi/agent/sessions` |
| `OPENCODE_DB` | OpenCode database file | `$XDG_DATA_HOME/opencode/opencode.db` |
| `XDG_DATA_HOME` | Base for OpenCode's default location | `~/.local/share` |
| `GLANCE_CURSOR_HOME` | Cursor configuration and transcripts | `~/.cursor` |

### Set by your terminal

Glance reads these to decide where it is running; you do not normally set them.

| Variable | Used for |
| --- | --- |
| `HERDR_PANE_ID`, `HERDR_SOCKET_PATH`, `HERDR_BIN_PATH` | Finding herdr and the current pane. On Windows, set `HERDR_SOCKET_PATH` to herdr's named pipe ([Windows](../how-to/windows.md)). |
| `TMUX`, `ZELLIJ` | Choosing the `attach` backend |
