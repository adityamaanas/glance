# Agent transcript formats

[Docs home](../README.md) · [Other agents](../how-to/other-agents.md) · [Cursor](../how-to/cursor.md)

Which files Glance reads for each agent (`--harness`), and what it takes from them. Glance only reads these files; it never changes them. Reasoning and "thinking" content is always excluded.

| Agent (`--harness`) | Reads | Includes | Notes |
| --- | --- | --- | --- |
| Claude Code (`claude`, default) | `~/.claude/projects/<project>/<session-id>.jsonl` | Your messages, Claude's replies, tool calls and results; title, git branch or worktree, linked PR | Read incrementally as it grows |
| Codex (`codex`) | `~/.codex/sessions/**` rollout files | Visible messages and tool results | Duplicate transport messages, instructions and encrypted reasoning are skipped; rollbacks remove the discarded turns |
| Gemini CLI (`gemini`) | `~/.gemini/tmp/*/chats/` | Visible messages and tool calls with results | Current JSONL and legacy JSON; message edits and rewinds are applied |
| pi (`pi`) | `~/.pi/agent/sessions/` | Messages, tool results and shell commands | Follows the latest saved branch of the conversation |
| OpenCode (`opencode`) | `~/.local/share/opencode/opencode.db`, or a JSON export via `--transcript` | Text and tool results | Opened read-only, including recent unflushed writes; unsupported database versions can use `opencode export` |
| Cursor (`cursor`) | Glance's own captures in `~/.glance/cursor/`, then Cursor's exported `agent-transcripts` | Prompts, responses and tool results | Captures come from `setup --harness cursor` or `cursor-stream` ([Follow Cursor](../how-to/cursor.md)) |

Default locations can be moved with the variables in [Files and environment](files-and-environment.md#where-agents-keep-their-sessions), or bypassed with `--transcript <file>`.

## How sessions are found

- `--session <id>` finds the file containing that session, checking files whose names contain the ID first.
- `--cwd <dir>` and `pick --cwd` match the working directory recorded inside each session. Some Gemini and Cursor exports do not record one; use `--session` for those.
- Discovery reads only the first and last 64 KiB of each file, so listing many sessions stays fast.

## Size limits

JSONL transcripts (Claude Code, Codex, Gemini, pi, Cursor) are read incrementally: only new lines are parsed as the file grows. Whole-file exports (legacy Gemini JSON, OpenCode JSON, Cursor text) are re-read when they change and are limited to 128 MiB.

For format specifications, see the agents' own documentation: [Codex](https://learn.chatgpt.com/docs/non-interactive-mode), [Gemini recording types](https://github.com/google-gemini/gemini-cli/blob/main/packages/core/src/services/chatRecordingTypes.ts), [pi session format](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/session-format.md), [OpenCode schema](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/session/session.sql.ts), [Cursor CLI output](https://cursor.com/docs/cli/reference/output-format), [Cursor hooks](https://cursor.com/docs/hooks).
