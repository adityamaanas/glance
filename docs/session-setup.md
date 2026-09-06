# Sessions, setup, and terminal placement

Glance works in a normal terminal as well as herdr. Open a split in your terminal and choose a conversation:

```sh
glance-panel --cwd .                # most recently updated session in this project
glance-panel --session <id>         # follow a specific session
glance-panel                       # picker when herdr is unavailable
glance-panel pick                   # choose and print an ID
glance-panel pick --cwd . --latest
glance-panel pick --query "rollout" --list
```

The picker lists up to 30 recent matches; `--query` narrows the list and `--list` returns all matching metadata as JSON. Scripts should use `--latest` or `--list`. Project matching uses the recorded working directory, not just the directory's encoded name. Discovery samples the beginning and end of each transcript and ignores nested subagent files.

## Install hooks once

```sh
glance-panel setup
glance-panel setup --yes
glance-panel setup --remove
```

Invoking setup opts into installation. It registers SessionStart, Stop, and StopFailure hooks, merges existing settings, preserves other hooks, and keeps a settings backup. Repeating setup is idempotent. The older `hook --install` / `hook --uninstall` commands remain available.

SessionStart attaches automatically inside herdr. Outside herdr, open the split yourself. Turn-end hooks write small local markers so Glance can summarize promptly; transcript settling remains the fallback when hooks are absent. Markers contain the transcript path, byte length, and timestamp, without conversation text. Neither hook calls a model.

Setup uses Claude Code's current executable-and-arguments hook format, which avoids shell quoting on Windows. Update Claude Code if your installed version does not support hook `args`.

## tmux and Zellij

```sh
glance-panel attach --backend tmux --session <id> --ratio 0.3
glance-panel attach --backend zellij --cwd .
glance-panel --no-model attach --session <id>
```

`attach` detects herdr, then tmux, then Zellij. tmux and herdr honor `--ratio`; Zellij uses its own tiled layout sizing. Each tmux/Zellij invocation creates a new pane. Custom `GLANCE_HOME`, `CLAUDE_CONFIG_DIR`, and `GLANCE_MODEL` values are passed into the new pane. For terminals without a supported multiplexer, use the terminal's split command and run Glance there.

## herdr sidebar

Run the panel with `--sidebar`, or set `"sidebar_metadata": true` in `~/.glance/config.json`. Add the tokens to your herdr configuration:

```toml
[ui.sidebar.agents.rows_by_agent]
claude = [
  ["state_icon", "agent", "state_text"],
  ["$step", "$progress"],
  ["workspace", "tab"],
]
```

Glance reports a short current step and completed/total plan count every 30 seconds while the panel runs. Tokens expire after 90 seconds without a refresh. This changes presentation only; herdr's agent state and lifecycle remain authoritative.

## Claude Code plugin alternative

Install the Glance binary first and make sure `glance-panel` is on PATH. After this change is available on the repository's default branch:

```sh
claude plugin marketplace add adityamaanas/glance
claude plugin install glance-panel@glance
```

To try the checked-out plugin locally:

```sh
claude --plugin-dir ./claude-plugin
```

Use either the plugin or `glance-panel setup` to manage hooks. Remove the setup-managed hooks before switching to the plugin to avoid duplicate callbacks. The plugin follows Claude Code's standard hook discovery layout.

## Windows

herdr IPC uses named pipes; provide the endpoint through `HERDR_SOCKET_PATH`. Auto-attach recognizes PowerShell and Command Prompt, quotes commands for the detected shell, and checks ancestors through a bounded hidden PowerShell process to skip headless runs. If that check cannot determine the parent mode, automatic attachment is skipped; explicit `attach` remains available.

The Windows build, local pipe transport, hook setup, and shell command construction are tested. A live Windows herdr session and a live Zellij split still need user-environment verification.

References: [Claude hooks](https://code.claude.com/docs/en/hooks), [Claude plugins](https://code.claude.com/docs/en/plugins-reference), [tmux](https://man.openbsd.org/tmux), [Zellij actions](https://zellij.dev/documentation/cli-actions), [herdr metadata](https://herdr.dev/docs/socket-api/).
