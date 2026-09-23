# Privacy and data handling

What Glance reads, what it stores, and what leaves your machine.

[Docs home](../README.md) · [Files and environment](../reference/files-and-environment.md) · [Security reporting](../../SECURITY.md)

## What Glance reads and captures

Glance reads local transcripts, exports and OpenCode's database without changing
them. herdr integration reads pane/session/status metadata. Adapters select
visible user, assistant and tool content and exclude recognized internal
reasoning fields; this is not a secret-redaction system.

Optional Cursor IDE hooks capture new visible events into Glance-owned files.
`cursor-stream` captures CLI events while forwarding stdout; forwarding never
depends on capture succeeding. These routes do
not automatically import historical IDE chats or read Cursor's private database.
See [Follow Cursor](../how-to/cursor.md).

## What reaches a summary provider

Model-enabled summaries send the title, previous summary, user todos and compact
transcript excerpts to a separate agent CLI. Content may include code, paths,
prompts, tool output and secrets present in the conversation.

The default provider matches the transcript's agent. `--summary-harness`
changes it explicitly. Optional fallback applies only when the primary
executable is absent; authentication, quota and runtime errors do not switch
providers. Authentication, network routing and server retention follow the CLI
and account.

Most providers receive conversation text through stdin. Cursor's print
interface uses a process argument, so local process inspection can expose that
prompt; oversized arguments fail with a clear error. Glance avoids logging
ancestor prompt arguments and has no separate telemetry/upload service.

Helpers use temporary directories outside the project and request provider
specific tool restrictions. Claude, Codex and pi request ephemeral/no-session
behavior; Gemini, OpenCode and Cursor may retain helper history under their own
policies. These settings are not a universal OS security boundary. Read
[Choose who writes summaries](../how-to/summary-providers.md) before selecting a provider.

## Local storage and sharing

Caches, todos, preferences, logs and captured Cursor content live under
`~/.glance/`, or `GLANCE_HOME`; the [file list](../reference/files-and-environment.md#what-glance-stores)
describes each one. Treat derived summaries and graph exports as
sensitive conversation content. HTML exports embed excerpts and work offline.

Explicit setup writes Claude/Cursor hook settings with backups and preserves
unrelated registrations. Removal removes Glance-owned hooks. Errors can appear
in local logs; sanitize logs before sharing them.

## Model-free operation and cleanup

```sh
glance-panel --harness codex --session <id> --no-model
glance-panel cache-clean --older-than-days 30 --dry-run
```

The global `--no-model` flag and config `no_model: true` prevent summary calls,
including explicit `summarize`. They still allow local transcript/cache reads.
Installed Cursor hooks continue capturing visible events until removed.

Cache cleanup removes old summary caches and preserves configuration and todos.
To remove captures/reminders, close their panels and remove the appropriate
Glance-owned files deliberately. Run `setup --remove` and/or
`setup --harness cursor --remove` before deleting the executable if hooks were
installed.
