# Agent transcript adapters

Choose a transcript format with `--harness`. The selected adapter supplies the same plan, evidence, graph, and personal-todo views.

```sh
glance-panel --harness codex --session <id>
glance-panel --harness pi --cwd .
glance-panel --harness gemini pick --list
glance-panel --harness opencode --session <id>
glance-panel --harness cursor --session <id> --transcript conversation.jsonl
```

herdr's native session metadata selects the adapter automatically when it includes the agent name. An explicit `--harness` takes precedence. New sessions need a transcript before discovery can find them.

| Agent | Sources | Behavior |
| --- | --- | --- |
| Claude Code | `~/.claude/projects/*/*.jsonl` | Existing incremental reader |
| Codex | `~/.codex/sessions/` rollouts | Visible message events and tool results; duplicate transport messages, instructions, and encrypted reasoning are excluded |
| Gemini CLI | `~/.gemini/tmp/*/chats/` | Legacy JSON and current JSONL, including message replacements, metadata checkpoints, and rewinds |
| pi | `~/.pi/agent/sessions/` | Latest persisted branch via entry IDs and parents; legacy linear sessions also work |
| OpenCode | `~/.local/share/opencode/opencode.db`, or JSON export | Read-only SQLite message/part queries, including committed WAL data and tool outcomes |
| Cursor | IDE `agent-transcripts` text/JSONL, CLI `stream-json`, or hook event JSONL | Visible prompts, responses, and tool results; the final CLI result is not counted twice |

The readers exclude thinking/reasoning fields. They read agent data without modifying it. Snapshot adapters compare the visible turn sequence, so a rewind or changed branch invalidates incompatible summaries and rejects stale in-flight results. Cache and todo names are separated by agent; Claude's existing filenames stay compatible.

## Custom locations and exports

Use `--transcript <path>` for a custom file or an OpenCode JSON export. If the file contains a session ID, it must match `--session`. You can inspect what Glance will summarize without calling a model:

```sh
glance-panel --harness codex transcript --session <id>
opencode export <id> > opencode-session.json
glance-panel --harness opencode --transcript opencode-session.json transcript --session <id>
```

For Cursor CLI, save its documented `--output-format stream-json` output and follow that file with `--harness cursor --transcript <path> --session <id>`. The CLI's private `store.db` format is not read. For live IDE hooks and CLI stream capture, see [Cursor setup](cursor.md).

Environment overrides: `CODEX_HOME`, `CLAUDE_CONFIG_DIR`, `GLANCE_GEMINI_HOME`, `PI_CODING_AGENT_SESSION_DIR`, `GLANCE_CURSOR_HOME`, `OPENCODE_DB`, and `XDG_DATA_HOME` for OpenCode's default data location. Glance-owned data remains under `GLANCE_HOME` or `~/.glance`.

## Limits and verification

The adapter flag selects the transcript reader and its default summary CLI. See [summary providers](summary-providers.md) for model overrides and explicit fallback. Use `--no-model` for local fields, personal todos, and compatible cached summaries without model calls.

Discovery samples the first and last 64 KiB of files; legacy Gemini JSON may require a full read. JSONL transcripts are read incrementally: only appended bytes are parsed, and a rewrite of earlier content triggers a fresh read. Whole-file exports (legacy Gemini JSON, OpenCode JSON, Cursor text) are re-read on change and bounded at 128 MiB per file. Lookup by session ID checks files whose names contain the ID first. Project filtering requires recorded working-directory metadata, which some Gemini and Cursor exports omit. Use an explicit session or transcript in that case.

pi reflects the latest persisted branch, so moving its in-memory selection without saving another entry may not be visible. OpenCode's message/part schema is supported; unsupported database schemas should use a compatible JSON export. The adapters have synthetic fixtures and Windows/Linux tests, including rollback, deduplication, namespace isolation, and read-only WAL access. Live compatibility still depends on the installed agent version.

Format references: [Codex non-interactive mode](https://learn.chatgpt.com/docs/non-interactive-mode), [Gemini recording types](https://github.com/google-gemini/gemini-cli/blob/main/packages/core/src/services/chatRecordingTypes.ts), [pi session format](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/session-format.md), [OpenCode session schema](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/session/session.sql.ts), [Cursor CLI output](https://cursor.com/docs/cli/reference/output-format), [Cursor hooks](https://cursor.com/docs/hooks).
