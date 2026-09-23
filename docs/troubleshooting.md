# Troubleshooting

[← Home](../README.md) · [User guide](usage.md)

## No pane, session or transcript

Use `attach` inside herdr, tmux or Zellij; specify `--backend` when detection
is ambiguous. A manual split with `--session <id>` works elsewhere. Outside
herdr, select non-Claude formats with `--harness`.

Try `glance-panel --harness codex pick --list`, or add `--cwd` to narrow the
project. A fresh session may need its first prompt. Use `--transcript` for a
custom export. See [adapter locations](agent-transcripts.md). Cursor IDE
capture contains events after setup; use an export for older content.

## Attach does not create a split

herdr attachment exits if a sibling runs Glance and may reuse an idle sibling
shell. With other busy panes, `attach --force` permits another split. The
ratio must be between 0 and 1. On Windows, keep the binary at a stable path and
re-run setup after moving it.

## Missing or stale summary

Check the footer's error, provider and update age. Metadata can update while a
summary is pending. Confirm the selected executable is on PATH and its
login/model works. `r` forces a refresh past the interval. Config or CLI
`no_model` prevents calls.

This diagnostic command invokes the selected provider and seeds a cache:

```sh
glance-panel --harness codex summarize --session <id>
```

For parsing diagnostics without a model call:

```sh
glance-panel --harness codex transcript --session <id>
```

Rewrites and agent changes invalidate old cache evidence. Older cache formats
rebuild once. Missing executables can use an explicit fallback; auth/quota/runtime
failures require fixing that provider. Native contracts are tested with mocks,
so a changed installed CLI may need an update. See
[summary providers](summary-providers.md).

## Hooks or Cursor capture are not working

Inspect `~/.glance/hook.log` and the [Cursor guide](cursor.md). Re-run `setup`
for Claude or `setup --harness cursor` after moving the executable. Unrelated
hooks are preserved; malformed settings are reported rather than overwritten.
Hook entry points fail open so they do not block the agent.

Stop hooks improve activity timing; settled transcript growth remains a
fallback. herdr sidebar reports are optional with `--sidebar`.

## Todo or config error

Outside herdr pass `todo --session <id>`, with `--harness` for another agent.
Each agent has its own store. Invalid config/todo JSON is reported and preserved;
fix it before retrying. Concurrent writers wait briefly for a lock and can
report contention. Carrying reminders requires `--carry-from`.

## Report a problem

Use the [bug report form](https://github.com/adityamaanas/glance/issues/new?template=bug_report.yml).
Include OS, Glance commit/version, agent CLI and terminal versions, invocation
and sanitized errors. Review private transcripts, captures, database files and
model output before sharing. See [security reporting](../SECURITY.md) for
vulnerabilities.
