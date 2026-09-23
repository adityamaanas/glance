# Follow Codex, Gemini, pi or OpenCode

[Docs home](../README.md) · [Agent formats](../reference/agents.md) · [Summary providers](summary-providers.md)

Glance reads the session files that Codex, Gemini CLI, pi and OpenCode already write, and shows them in the same panel as Claude Code. For Cursor, see [Follow Cursor](cursor.md).

**You need:** Glance installed, the agent installed, and a session with at least one prompt sent.

## Choose the agent with `--harness`

Tell Glance which agent's files to read. `--harness` works with every command and in any position.

```sh
glance-panel --harness codex --cwd .
glance-panel --harness gemini --cwd .
glance-panel --harness pi --cwd .
glance-panel --harness opencode --cwd .
```

Under herdr you can usually leave `--harness` out: herdr reports which agent runs in the pane and Glance picks the matching reader. An explicit `--harness` always wins.

The same flag applies to the other commands:

```sh
glance-panel --harness codex pick --list
glance-panel --harness codex attach --backend tmux
glance-panel --harness codex todo "Re-run the migration" --session <session-id>
```

## Who writes the summaries

By default the summary is written by the **same agent's CLI** (for example `codex exec` for a Codex session), using that CLI's own login and billing. It runs in a temporary directory with tools restricted. To use a different agent or a specific model, see [Choose who writes summaries](summary-providers.md). To avoid model calls entirely, add `--no-model`.

## Read an exported or relocated transcript

Point `--transcript` at the file and give the session ID:

```sh
glance-panel --harness codex --transcript ./rollout.jsonl --session <session-id>
```

For OpenCode, you can export a session instead of reading its database:

```sh
opencode export <session-id> > opencode-session.json
glance-panel --harness opencode --transcript opencode-session.json --session <session-id>
```

If the file records its own session ID, it must match `--session`.

Agents that keep their data somewhere other than the default can also be found through environment variables such as `CODEX_HOME` or `OPENCODE_DB`; see [Files and environment](../reference/files-and-environment.md).

## Check what Glance reads

To see exactly which turns Glance extracted, without any model call:

```sh
glance-panel --harness codex transcript --session <session-id>
```

This prints the normalized turns as JSON. Reasoning and "thinking" content is never included.

## What to expect

- Each agent has its own summary cache and todo list, even for identical session IDs.
- If the agent rewinds or rolls back the conversation, Glance notices and rebuilds the summary rather than keeping items from discarded history.
- A brand-new session only appears once its first prompt has been written to disk. If `--cwd .` finds nothing, send a prompt and try again.
- Formats, default locations and per-agent limits are listed in [Agent transcript formats](../reference/agents.md).
