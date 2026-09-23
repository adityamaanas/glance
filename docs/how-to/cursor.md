# Follow Cursor

[Docs home](../README.md) · [Agent formats](../reference/agents.md) · [Privacy](../explanation/privacy.md)

Glance can follow Cursor conversations from the IDE or the Cursor CLI. The IDE route captures new messages through Cursor's hooks; the CLI route records the CLI's JSON output as it passes through.

**You need:** Glance installed, and Cursor (IDE or CLI).

## Cursor IDE

1. Install the capture hooks once:

   ```sh
   glance-panel setup --harness cursor
   ```

2. Send a prompt in a Cursor chat.

3. In a terminal in the same project, open the panel:

   ```sh
   glance-panel --harness cursor --cwd .
   ```

Capture starts when you install the hooks; earlier chats are not imported. The panel stays on the conversation it opened with, so run it again after starting a new chat, or choose one with `glance-panel --harness cursor pick`.

Setup adds a small wrapper script under `~/.cursor/hooks/` and entries in `~/.cursor/hooks.json`. It keeps your other hooks, and backs up `hooks.json` the first time it changes it (as `hooks.json.bak-glance`). Re-run setup if you move the `glance-panel` binary. The hooks never block or change what Cursor does, even if capture fails.

To stop capturing:

```sh
glance-panel setup --harness cursor --remove
```

Removal keeps what was already captured under `~/.glance/cursor/`; delete those files yourself if you no longer want them.

## Cursor CLI

Pipe the CLI's `stream-json` output through `glance-panel cursor-stream`:

```sh
agent -p --output-format stream-json "Review the current changes" | glance-panel cursor-stream
```

Then, in another pane:

```sh
glance-panel --harness cursor --cwd .
```

`cursor-stream` passes every byte through to its output unchanged, so you can keep piping into other tools. It records the visible turns as they arrive and prints the conversation ID on stderr. If a line cannot be recorded, it still passes through and the problem is reported once on stderr. Do not use `--stream-partial-output`; Glance expects complete messages.

To read a saved stream later without recording it again:

```sh
glance-panel --harness cursor --transcript saved-stream.jsonl --session <conversation-id>
```

## What is captured

Prompts, responses, tool results and turn endings. Email addresses, attachments, thoughts and any fields Glance does not recognize are dropped before anything is written. Each event is capped (60,000 characters for messages, 8,000 for tool output), and each conversation's capture file stops growing at about 128 MiB.

Visible messages can still contain sensitive material. Summaries of Cursor sessions use the Cursor CLI by default, which receives its prompt as a command-line argument that other local users can see in the process list; see [Choose who writes summaries](summary-providers.md) to use another agent.

## What to expect

- Captures live in `~/.glance/cursor/<conversation-id>.jsonl`.
- Glance's own capture is preferred over Cursor's exported transcript for the same conversation; pass `--transcript` to choose an exported one.
- Live IDE and CLI behavior depends on your Cursor version; see [Compatibility](../explanation/compatibility.md).
