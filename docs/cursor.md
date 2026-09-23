# Cursor IDE and CLI

Glance follows a selected Cursor conversation in a terminal panel. The IDE integration captures new visible events through [Cursor's hooks](https://cursor.com/docs/hooks); the CLI integration consumes its documented [stream output](https://cursor.com/docs/cli/reference/output-format).

## IDE setup

```sh
glance-panel setup --harness cursor
# Send a prompt in Cursor, then open its project in a terminal:
glance-panel --harness cursor --cwd .
```

Setup installs a small platform-specific wrapper under `~/.cursor/hooks/` and adds Glance entries to `hooks.json`, preserving other hooks and settings. The wrapper remembers the executable and Glance data directory, so a GUI launch does not need to inherit your terminal's `GLANCE_HOME`. Re-run setup after moving the executable. `GLANCE_CURSOR_HOME` selects an alternate Cursor configuration directory.

Capture covers session metadata, prompts, responses, tool outcomes and turn-end signals. It starts at installation; it does not reconstruct earlier chat history. Glance-owned captures take precedence over discovered IDE transcripts for the same conversation. Use `--transcript <path>` to select an older native transcript explicitly.

```sh
glance-panel --harness cursor pick --list
glance-panel --harness cursor --session <conversation-id>
glance-panel --harness cursor attach --backend tmux --cwd .
glance-panel setup --harness cursor --remove
```

Removal unregisters Glance's hooks. It retains your captured conversations and generated wrapper. Remove unwanted captures from `~/.glance/cursor/` yourself; the summary-cache cleaner preserves them. Hook installation is optional and does not happen during package installation.

## CLI capture

Pipe complete `stream-json` output into the recorder:

```sh
agent -p --output-format stream-json "Review the current changes" | glance-panel cursor-stream
# In another terminal, after the stream starts:
glance-panel --harness cursor --cwd /path/to/project
```

`cursor-stream` prints the conversation ID to stderr, forwards the original stream unchanged to stdout, and saves normalized visible turns. Forwarding never depends on capture: an unreadable, oversized or out-of-session record is still forwarded, and the capture problem is reported once on stderr. It does not launch an agent itself. Avoid `--stream-partial-output`; the importer expects complete messages. The final result does not duplicate the assistant response. Replaying a saved CLI stream appends it again, so use `--transcript` to read a saved stream without importing it.

## Data and limits

Captures live at `$GLANCE_HOME/cursor/<id>.jsonl` (default `~/.glance/cursor/`). Prompts and responses are limited to 60,000 characters per event, tool outputs to 8,000, input records to 8 MiB, and captures to roughly 128 MiB. Concurrent writers coordinate through a file lock. Recent identical hook retries within the same generation are ignored.

Email, attachments, thoughts and unknown hook fields are excluded. Visible messages and tool output can still contain sensitive material. `--no-model` keeps the panel local; otherwise summaries use the configured summary backend. Hooks always return an empty success response, including when capture fails, and never modify Cursor's tool or prompt decisions.

The panel stays on the selected conversation; pick again when you open a new IDE chat. Local user hooks do not install anything in cloud agents. The integration is tested with synthetic Cursor events, actual Windows/POSIX wrapper execution, concurrent capture, stream forwarding and removal. A live Cursor IDE/CLI end-to-end run still needs validation on an installed version.
