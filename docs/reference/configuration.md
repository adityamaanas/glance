# Configuration file

[Docs home](../README.md) · [Command line](cli.md) · [Files and environment](files-and-environment.md)

Glance reads `config.json` from its state directory: `~/.glance/config.json`, or `$GLANCE_HOME/config.json` when `GLANCE_HOME` is set. The file is optional, and every setting is optional. A command-line flag always overrides the same setting in the file.

```json
{
  "refresh_seconds": 60,
  "summary_models": { "claude": "claude-haiku-4-5" },
  "cache_retention_days": 30,
  "sidebar_metadata": true
}
```

## Settings

| Setting | Type | Default | Flag | Effect |
| --- | --- | --- | --- | --- |
| `no_model` | boolean | `false` | `--no-model` | Never call a summary agent, including for `summarize`. |
| `refresh_seconds` | number | `30` | `--refresh-seconds` | Minimum seconds between automatic summaries. `r` in the panel ignores it. |
| `summary_harness` | agent name | the session's agent | `--summary-harness` | Which agent's CLI writes summaries: `claude`, `codex`, `gemini`, `pi`, `opencode` or `cursor`. |
| `summary_fallback` | agent name | none | `--summary-fallback` | Agent to use only when the summary agent's executable cannot be found. |
| `summary_models` | object | `{}` | `--model` (for the requested agent) | Model per summary agent, for example `{"claude": "claude-haiku-4-5", "codex": "gpt-5.4-mini"}`. |
| `model` | string | `claude-sonnet-5` | none | Claude summary model, used when neither `--model` nor `summary_models.claude` is set. Applies to Claude only. |
| `prompt` | string | none | none | Extra instructions appended to the summary prompt, for example `"Keep the current step under ten words."` |
| `cache_retention_days` | number | none | none | When set, opening a panel removes summary caches not updated for this many days. |
| `sidebar_metadata` | boolean | `false` | `--sidebar` | Report the current step and plan progress to herdr's sidebar. |
| `hook_offer` | string | none | none | Set by Glance to `accepted` or `declined` after the first-run banner, so it is not shown again. |

For how the model is chosen when several of these apply, see [Choose who writes summaries](../how-to/summary-providers.md#change-the-model).

## Behavior

- Glance keeps settings it does not recognize when it updates the file.
- If the file is not valid JSON, Glance stops with an error that names the file, so a typo is never silently ignored. Hooks still exit successfully so they never block your agent.
- The `GLANCE_MODEL` environment variable sits between `summary_models.claude` and `model` for Claude summaries. See [Files and environment](files-and-environment.md#environment-variables).
