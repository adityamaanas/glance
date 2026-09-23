# Command-line reference

[Docs home](../README.md) · [Keys](keys.md) · [Configuration](configuration.md) · [Files and environment](files-and-environment.md)

<!-- Generated from `glance-panel --help` by tests/cli_reference.rs. Do not edit by hand:
     GLANCE_UPDATE_DOCS=1 cargo test --test cli_reference -->

Global options (such as `--harness`, `--transcript`, `--no-model` and the summary options) can be given before or after a command.

## glance-panel

```text
Glance: a live orientation panel beside your coding-agent session.

Run without a command to open the panel. Inside herdr it follows the neighbouring agent pane; elsewhere pass --session, --cwd or --transcript, or pick from a list.

Usage: glance-panel [OPTIONS] [COMMAND]

Commands:
  cursor-stream  Capture Cursor CLI stream-json from stdin, forwarding it to stdout
  transcript     Print normalized transcript turns without calling a model
  setup          Install or remove hooks: Claude Code session and turn-end hooks, or Cursor capture with --harness cursor
  pick           Choose a local session and print its ID
  todo           Manage personal reminders (defaults to this herdr pane's session)
  graph          Show cached item relationships, or export a standalone HTML graph
  cache-clean    Remove old summary caches; personal configuration and todos are preserved
  attach         Split the terminal (herdr, tmux or Zellij) and start the panel in the new pane
  summarize      Summarize a whole session once, save the cache and print the summary JSON
  hook           Claude Code hook entry point (reads hook JSON on stdin, always exits 0). Prefer `setup`
  help           Print this message or the help of the given subcommand(s)

Options:
      --pane <PANE>
          herdr pane id of the agent session to follow (default: the pane next to this one)

      --session <SESSION>
          Session ID to follow directly (no herdr needed)

      --cwd <CWD>
          Follow the most recently updated session for this project

      --harness <HARNESS>
          Agent whose transcript to read (default: claude; herdr sessions are detected automatically)

          [possible values: claude, codex, gemini, pi, opencode, cursor]

      --transcript <TRANSCRIPT>
          Read this transcript or export file instead of searching the agent's default location

      --no-model
          Never call a model; show transcript fields, cached summaries and todos only

      --sidebar
          Report the current step and progress to herdr's sidebar

      --model <MODEL>
          Model for the requested summary agent (overrides summary_models; never passed to a fallback)

      --summary-harness <SUMMARY_HARNESS>
          Agent CLI used for summaries (default: the transcript's agent)

          [possible values: claude, codex, gemini, pi, opencode, cursor]

      --summary-fallback <SUMMARY_FALLBACK>
          Alternate CLI, used only when the selected summary executable is absent

          [possible values: claude, codex, gemini, pi, opencode, cursor]

      --refresh-seconds <REFRESH_SECONDS>
          Minimum seconds between summary calls (default 30)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## glance-panel attach

```text
Split the terminal (herdr, tmux or Zellij) and start the panel in the new pane

Usage: glance-panel attach [OPTIONS]

Options:
      --backend <BACKEND>
          Multiplexer to use; auto detects herdr, then tmux, then Zellij [default: auto] [possible values: auto, herdr, tmux, zellij]
      --session <SESSION>
          Session ID for tmux/Zellij (default: the latest session in --cwd)
      --cwd <CWD>
          Project directory for tmux/Zellij (default: the current directory)
      --harness <HARNESS>
          Agent whose transcript to read (default: claude; herdr sessions are detected automatically) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --ratio <RATIO>
          Fraction of the width the new pane takes [default: 0.3]
      --force
          Split even if the tab already has more than one pane
      --transcript <TRANSCRIPT>
          Read this transcript or export file instead of searching the agent's default location
      --no-model
          Never call a model; show transcript fields, cached summaries and todos only
      --model <MODEL>
          Model for the requested summary agent (overrides summary_models; never passed to a fallback)
      --summary-harness <SUMMARY_HARNESS>
          Agent CLI used for summaries (default: the transcript's agent) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --summary-fallback <SUMMARY_FALLBACK>
          Alternate CLI, used only when the selected summary executable is absent [possible values: claude, codex, gemini, pi, opencode, cursor]
      --refresh-seconds <REFRESH_SECONDS>
          Minimum seconds between summary calls (default 30)
  -h, --help
          Print help
```

## glance-panel pick

```text
Choose a local session and print its ID

Usage: glance-panel pick [OPTIONS]

Options:
      --cwd <CWD>
          Only sessions whose recorded working directory is this project
      --query <QUERY>
          Only sessions whose ID, title or directory contains this text
      --latest
          Print the most recently updated match without asking
      --harness <HARNESS>
          Agent whose transcript to read (default: claude; herdr sessions are detected automatically) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --list
          Print every match as JSON
      --transcript <TRANSCRIPT>
          Read this transcript or export file instead of searching the agent's default location
      --no-model
          Never call a model; show transcript fields, cached summaries and todos only
      --model <MODEL>
          Model for the requested summary agent (overrides summary_models; never passed to a fallback)
      --summary-harness <SUMMARY_HARNESS>
          Agent CLI used for summaries (default: the transcript's agent) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --summary-fallback <SUMMARY_FALLBACK>
          Alternate CLI, used only when the selected summary executable is absent [possible values: claude, codex, gemini, pi, opencode, cursor]
      --refresh-seconds <REFRESH_SECONDS>
          Minimum seconds between summary calls (default 30)
  -h, --help
          Print help
```

## glance-panel todo

```text
Manage personal reminders (defaults to this herdr pane's session)

Usage: glance-panel todo [OPTIONS] [TEXT]

Arguments:
  [TEXT]  Text to append; omit to list todos

Options:
      --session <SESSION>
          Session ID (default: the session in this herdr pane)
      --set <SET>
          Todo ID whose status should change
      --status <STATUS>
          New status for --set [possible values: pending, in-progress, done]
      --delete <DELETE>
          Todo ID to delete
      --harness <HARNESS>
          Agent whose transcript to read (default: claude; herdr sessions are detected automatically) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --carry-from <CARRY_FROM>
          Explicitly copy reminders from another session as pending items
      --transcript <TRANSCRIPT>
          Read this transcript or export file instead of searching the agent's default location
      --no-model
          Never call a model; show transcript fields, cached summaries and todos only
      --model <MODEL>
          Model for the requested summary agent (overrides summary_models; never passed to a fallback)
      --summary-harness <SUMMARY_HARNESS>
          Agent CLI used for summaries (default: the transcript's agent) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --summary-fallback <SUMMARY_FALLBACK>
          Alternate CLI, used only when the selected summary executable is absent [possible values: claude, codex, gemini, pi, opencode, cursor]
      --refresh-seconds <REFRESH_SECONDS>
          Minimum seconds between summary calls (default 30)
  -h, --help
          Print help
```

## glance-panel graph

```text
Show cached item relationships, or export a standalone HTML graph

Usage: glance-panel graph [OPTIONS] --session <SESSION>

Options:
      --session <SESSION>
          Session ID whose cached summary to use
      --html [<HTML>]
          Write an offline HTML graph (default file: glance-graph.html)
      --open
          Open the HTML file in the default browser
      --harness <HARNESS>
          Agent whose transcript to read (default: claude; herdr sessions are detected automatically) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --transcript <TRANSCRIPT>
          Read this transcript or export file instead of searching the agent's default location
      --no-model
          Never call a model; show transcript fields, cached summaries and todos only
      --model <MODEL>
          Model for the requested summary agent (overrides summary_models; never passed to a fallback)
      --summary-harness <SUMMARY_HARNESS>
          Agent CLI used for summaries (default: the transcript's agent) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --summary-fallback <SUMMARY_FALLBACK>
          Alternate CLI, used only when the selected summary executable is absent [possible values: claude, codex, gemini, pi, opencode, cursor]
      --refresh-seconds <REFRESH_SECONDS>
          Minimum seconds between summary calls (default 30)
  -h, --help
          Print help
```

## glance-panel transcript

```text
Print normalized transcript turns without calling a model

Usage: glance-panel transcript [OPTIONS] --session <SESSION>

Options:
      --session <SESSION>
          Session ID to read
      --harness <HARNESS>
          Agent whose transcript to read (default: claude; herdr sessions are detected automatically) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --transcript <TRANSCRIPT>
          Read this transcript or export file instead of searching the agent's default location
      --no-model
          Never call a model; show transcript fields, cached summaries and todos only
      --model <MODEL>
          Model for the requested summary agent (overrides summary_models; never passed to a fallback)
      --summary-harness <SUMMARY_HARNESS>
          Agent CLI used for summaries (default: the transcript's agent) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --summary-fallback <SUMMARY_FALLBACK>
          Alternate CLI, used only when the selected summary executable is absent [possible values: claude, codex, gemini, pi, opencode, cursor]
      --refresh-seconds <REFRESH_SECONDS>
          Minimum seconds between summary calls (default 30)
  -h, --help
          Print help
```

## glance-panel summarize

```text
Summarize a whole session once, save the cache and print the summary JSON

Usage: glance-panel summarize [OPTIONS] --session <SESSION>

Options:
      --session <SESSION>
          Session ID to summarize
      --harness <HARNESS>
          Agent whose transcript to read (default: claude; herdr sessions are detected automatically) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --transcript <TRANSCRIPT>
          Read this transcript or export file instead of searching the agent's default location
      --no-model
          Never call a model; show transcript fields, cached summaries and todos only
      --model <MODEL>
          Model for the requested summary agent (overrides summary_models; never passed to a fallback)
      --summary-harness <SUMMARY_HARNESS>
          Agent CLI used for summaries (default: the transcript's agent) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --summary-fallback <SUMMARY_FALLBACK>
          Alternate CLI, used only when the selected summary executable is absent [possible values: claude, codex, gemini, pi, opencode, cursor]
      --refresh-seconds <REFRESH_SECONDS>
          Minimum seconds between summary calls (default 30)
  -h, --help
          Print help
```

## glance-panel setup

```text
Install or remove hooks: Claude Code session and turn-end hooks, or Cursor capture with --harness cursor

Usage: glance-panel setup [OPTIONS]

Options:
      --remove
          Remove Glance's hooks instead of installing them
      --yes
          Accepted for unattended installation; invoking setup already opts in
      --harness <HARNESS>
          Agent whose transcript to read (default: claude; herdr sessions are detected automatically) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --transcript <TRANSCRIPT>
          Read this transcript or export file instead of searching the agent's default location
      --no-model
          Never call a model; show transcript fields, cached summaries and todos only
      --model <MODEL>
          Model for the requested summary agent (overrides summary_models; never passed to a fallback)
      --summary-harness <SUMMARY_HARNESS>
          Agent CLI used for summaries (default: the transcript's agent) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --summary-fallback <SUMMARY_FALLBACK>
          Alternate CLI, used only when the selected summary executable is absent [possible values: claude, codex, gemini, pi, opencode, cursor]
      --refresh-seconds <REFRESH_SECONDS>
          Minimum seconds between summary calls (default 30)
  -h, --help
          Print help
```

## glance-panel cache-clean

```text
Remove old summary caches; personal configuration and todos are preserved

Usage: glance-panel cache-clean [OPTIONS]

Options:
      --older-than-days <OLDER_THAN_DAYS>
          Remove caches not updated for this many days [default: 30]
      --dry-run
          List what would be removed without deleting it
      --harness <HARNESS>
          Agent whose transcript to read (default: claude; herdr sessions are detected automatically) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --transcript <TRANSCRIPT>
          Read this transcript or export file instead of searching the agent's default location
      --no-model
          Never call a model; show transcript fields, cached summaries and todos only
      --model <MODEL>
          Model for the requested summary agent (overrides summary_models; never passed to a fallback)
      --summary-harness <SUMMARY_HARNESS>
          Agent CLI used for summaries (default: the transcript's agent) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --summary-fallback <SUMMARY_FALLBACK>
          Alternate CLI, used only when the selected summary executable is absent [possible values: claude, codex, gemini, pi, opencode, cursor]
      --refresh-seconds <REFRESH_SECONDS>
          Minimum seconds between summary calls (default 30)
  -h, --help
          Print help
```

## glance-panel cursor-stream

```text
Capture Cursor CLI stream-json from stdin, forwarding it to stdout

Usage: glance-panel cursor-stream [OPTIONS]

Options:
      --harness <HARNESS>
          Agent whose transcript to read (default: claude; herdr sessions are detected automatically) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --transcript <TRANSCRIPT>
          Read this transcript or export file instead of searching the agent's default location
      --no-model
          Never call a model; show transcript fields, cached summaries and todos only
      --model <MODEL>
          Model for the requested summary agent (overrides summary_models; never passed to a fallback)
      --summary-harness <SUMMARY_HARNESS>
          Agent CLI used for summaries (default: the transcript's agent) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --summary-fallback <SUMMARY_FALLBACK>
          Alternate CLI, used only when the selected summary executable is absent [possible values: claude, codex, gemini, pi, opencode, cursor]
      --refresh-seconds <REFRESH_SECONDS>
          Minimum seconds between summary calls (default 30)
  -h, --help
          Print help
```

## glance-panel hook

```text
Claude Code hook entry point (reads hook JSON on stdin, always exits 0). Prefer `setup`

Usage: glance-panel hook [OPTIONS]

Options:
      --install
          Legacy: register hooks (same as `setup`)
      --uninstall
          Legacy: remove hooks (same as `setup --remove`)
      --harness <HARNESS>
          Agent whose transcript to read (default: claude; herdr sessions are detected automatically) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --transcript <TRANSCRIPT>
          Read this transcript or export file instead of searching the agent's default location
      --no-model
          Never call a model; show transcript fields, cached summaries and todos only
      --model <MODEL>
          Model for the requested summary agent (overrides summary_models; never passed to a fallback)
      --summary-harness <SUMMARY_HARNESS>
          Agent CLI used for summaries (default: the transcript's agent) [possible values: claude, codex, gemini, pi, opencode, cursor]
      --summary-fallback <SUMMARY_FALLBACK>
          Alternate CLI, used only when the selected summary executable is absent [possible values: claude, codex, gemini, pi, opencode, cursor]
      --refresh-seconds <REFRESH_SECONDS>
          Minimum seconds between summary calls (default 30)
  -h, --help
          Print help
```
