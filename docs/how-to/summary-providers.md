# Choose who writes summaries

[Docs home](../README.md) · [Configuration](../reference/configuration.md) · [Privacy](../explanation/privacy.md)

Glance's summaries are written by an agent CLI you already have installed. By default that is the same agent as the session: Claude Code sessions are summarized by `claude`, Codex sessions by `codex`, and so on. This page shows how to change the agent, the model, and what happens when a CLI is missing.

**You need:** the summary agent's CLI installed and logged in. It uses its own login and billing; Glance never copies credentials between agents.

## Change the model

For one panel:

```sh
glance-panel --model claude-haiku-4-5
```

For every panel, per agent, in `~/.glance/config.json`:

```json
{
  "summary_models": {
    "claude": "claude-haiku-4-5",
    "codex": "gpt-5.4-mini"
  }
}
```

The model is chosen in this order:

1. `--model`, for the summary agent you asked for.
2. `summary_models` for that agent.
3. For Claude only: the `GLANCE_MODEL` environment variable, then `model` in `config.json`, then Glance's default (`claude-sonnet-5`).
4. For other agents: the CLI's own default model.

Use model names your account can access.

## Use a different agent to summarize

To keep reading a Cursor session but have Claude write the summary:

```sh
glance-panel --harness cursor --summary-harness claude --cwd .
```

Or for every session, in `config.json`:

```json
{ "summary_harness": "claude" }
```

Leave `summary_harness` out to follow each session's own agent.

## Fall back when a CLI is missing

```sh
glance-panel --harness gemini --summary-fallback claude --cwd .
```

The fallback is used **only** when the chosen CLI cannot be found. It is never used after a login, quota, timeout or bad-response error, so a failure never silently moves your conversation to another service. A fallback does not receive `--model`; it uses its own `summary_models` entry or default. The footer always names the agent and model that actually wrote the summary.

## Point Glance at a specific executable

If a CLI is not on your `PATH` or has an unusual name, set its path:

| Agent | Variable | Default name |
| --- | --- | --- |
| Claude Code | `GLANCE_CLAUDE_BIN` | `claude` |
| Codex | `GLANCE_CODEX_BIN` | `codex` |
| Gemini CLI | `GLANCE_GEMINI_BIN` | `gemini` |
| pi | `GLANCE_PI_BIN` | `pi` |
| OpenCode | `GLANCE_OPENCODE_BIN` | `opencode` |
| Cursor | `GLANCE_CURSOR_BIN` | `agent` (set it to `cursor-agent` if that is your install's name) |

These are paths to an executable, not shell commands.

## How each agent is run

Every summary runs in a temporary directory outside your project, with a 150-second limit, and asks the CLI to disable tools:

| Agent | Restrictions requested | Keeps a history of the summary call? |
| --- | --- | --- |
| Claude Code | No tools, no settings files, no session saved | No |
| Codex | Read-only sandbox; shell, web search and sub-agents disabled; ephemeral | No |
| pi | No tools, extensions, skills or context files; no session | No |
| Gemini CLI | A policy that denies every tool | May, per your Gemini settings |
| OpenCode | A dedicated agent with all permissions denied | May, per your OpenCode settings |
| Cursor | Ask mode, with shell, file, web and MCP access denied | May, per your Cursor settings |

These rely on each CLI honoring its documented options; they are not an operating-system sandbox. Most agents receive the conversation on standard input. **Cursor receives it as a command-line argument**, which other users on the same machine can see in the process list, and very long prompts can exceed the operating system's limit (especially on Windows). Choose another summary agent if either matters.

## What to expect

- The footer shows the agent and model that wrote the summary, the call count, and the cost when the CLI reports one.
- If a summary fails, the previous summary stays on screen with a red `⚠` message. Press `r` to try again. Glance does not retry automatically.
- To stop all model calls, see [Run without model calls](run-without-models.md).
