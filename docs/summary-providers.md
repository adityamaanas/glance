# Summary providers

By default, Glance uses the CLI for the selected transcript agent: Claude, Codex, Gemini, pi, OpenCode or Cursor. Each CLI uses its own configured authentication and billing. Glance does not copy credentials between agents.

```sh
glance-panel --harness codex --session <id>
glance-panel --harness cursor --summary-harness claude --session <id>
glance-panel --harness pi --model <available-model> --session <id>
glance-panel --harness gemini --summary-fallback claude --session <id>
```

`--summary-harness` changes the summarizer while retaining the transcript reader. `--summary-fallback` applies only when the selected executable cannot be found; it does not retry authentication, quota, timeout or malformed-response errors with another service. Fallback is disabled unless explicitly configured. The cache and panel record the backend that actually answered.

## Configuration

In `$GLANCE_HOME/config.json` (default `~/.glance/config.json`):

```json
{
  "summary_harness": "codex",
  "summary_models": {
    "claude": "sonnet",
    "codex": "gpt-5.4-mini"
  },
  "refresh_seconds": 30
}
```

The model precedence is `--model`, `GLANCE_MODEL`, `model` in config, then `summary_models[backend]`. Claude retains Glance's existing default; other backends use their CLI default when no override is set. Choose model IDs available to your account. Omit `summary_harness` to follow each transcript's agent automatically. Set `summary_fallback` only if you want the missing-executable fallback described above.

Override executable discovery with `GLANCE_CLAUDE_BIN`, `GLANCE_CODEX_BIN`, `GLANCE_GEMINI_BIN`, `GLANCE_PI_BIN`, `GLANCE_OPENCODE_BIN`, or `GLANCE_CURSOR_BIN`. These name executable paths, not shell command strings. The default Cursor executable is `agent`; point `GLANCE_CURSOR_BIN` at `cursor-agent` if that is the installed name.

New tmux/Zellij panels preserve explicit model, provider, transcript, refresh and sidebar options, along with Glance/agent location overrides. herdr pane commands preserve the explicit options using the target shell's quoting rules. `--no-model` prevents summary calls, including explicit `summarize` commands.

## Execution and compatibility

Helpers run in temporary directories outside the source project. Claude uses an empty tool set, Codex requests read-only execution with shell, collaboration and web search disabled, pi disables tools and extension discovery, Gemini receives a tool-denial policy, OpenCode receives a dedicated agent with denied permissions, and Cursor uses ask mode with denied shell/file/MCP/web-fetch permissions.

These controls use the agent's documented CLI interfaces; they depend on that CLI version and its administrator settings. They are not an independent operating-system sandbox. All helpers have a 150-second deadline and bounded output capture. Errors leave the existing panel available for manual refresh; Glance does not automatically repeat failed requests.

Claude and Codex use schema-aware output. Other providers receive the same schema in the prompt, and Glance parses their final response or visible output events. CLI failures and invalid JSON are reported. Cost is shown only when the CLI reports it; an unavailable estimate is not zero.

Claude, Codex and pi request ephemeral sessions. Gemini, OpenCode and Cursor may retain helper history according to their own settings. Cursor takes its prompt as a process argument; very large prompts exceed platform limits, especially Windows. Glance reports that limit rather than dropping content; select another summary backend in that case.

Windows/Linux tests launch a compiled mock CLI for every provider, validate input delivery and output parsing, and check explicit fallback and no-model behavior. They do not call paid models. Live authentication, model availability and installed-version behavior still need a smoke run with the desired CLI.

References: [Claude headless](https://code.claude.com/docs/en/headless), [Codex non-interactive mode](https://learn.chatgpt.com/docs/non-interactive-mode), [Gemini headless](https://geminicli.com/docs/cli/headless/), [Gemini policies](https://geminicli.com/docs/reference/policy-engine/), [pi CLI](https://github.com/earendil-works/pi/tree/main/packages/coding-agent), [OpenCode CLI](https://opencode.ai/docs/cli/), [Cursor CLI](https://cursor.com/docs/cli/reference/parameters).
