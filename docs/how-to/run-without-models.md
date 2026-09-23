# Run without model calls

[Docs home](../README.md) · [Privacy](../explanation/privacy.md) · [Files and environment](../reference/files-and-environment.md)

Glance is useful even when it never calls a model: the panel still shows the session's title, branch, latest messages and any summary saved earlier, and personal todos work fully. This page also covers controlling cost and removing stored data.

## Turn model calls off

For one panel or command:

```sh
glance-panel --cwd . --no-model
```

For everything, in `~/.glance/config.json`:

```json
{ "no_model": true }
```

With either, Glance never starts a summary agent, including for `glance-panel summarize`, which stops with an error instead. The footer shows `heuristic`, or the model that wrote an earlier cached summary.

## Spend less when summaries are on

- **Summarize less often.** Summaries run after the agent finishes a turn, at most once every 30 seconds by default. Raise that with `--refresh-seconds 120` or `"refresh_seconds": 120`. Press `r` for an immediate refresh when you want one.
- **Use a smaller model.** See [Choose who writes summaries](summary-providers.md#change-the-model).
- **Watch the footer.** It shows the number of summary calls and the cost the agent's CLI reported for this session. A `+` after the amount means some calls reported no cost.

Each summary reads only the turns added since the previous one, together with the previous summary, so cost grows with the new conversation, not the whole session.

## Remove stored data

Summary caches are disposable; Glance rebuilds them when needed. Preview, then remove caches not updated in 30 days:

```sh
glance-panel cache-clean --older-than-days 30 --dry-run
glance-panel cache-clean --older-than-days 30
```

To clean up automatically whenever a panel starts, set `"cache_retention_days": 30` in `config.json`.

Cache cleaning never touches your configuration, personal todos or Cursor captures. To remove those, close any open panels and delete the files you no longer want from `~/.glance/`; the [file list](../reference/files-and-environment.md#what-glance-stores) says what each one is.

## Uninstall

Remove the hooks before deleting the binary, so your agents do not keep calling a missing program:

```sh
glance-panel setup --remove                  # Claude Code hooks
glance-panel setup --harness cursor --remove # Cursor hooks, if you installed them
cargo uninstall glance-panel
```

Then delete `~/.glance/` if you want to remove all stored data.
