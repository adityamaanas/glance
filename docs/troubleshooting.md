# Troubleshooting

[← Home](../README.md) · [User guide](usage.md)

## `HERDR_PANE_ID not set`

Run `glance-panel attach` inside herdr. At a Claude prompt, use `! glance-panel attach`. Elsewhere, open your own split and pass `--session <session-id>`.

## No session or transcript found

Confirm Claude Code is running and run `herdr integration install claude`. A fresh session gets its transcript after the first prompt. A direct ID must match a `.jsonl` filename under `~/.claude/projects/`.

## Attach does not create a split

Glance exits if a sibling already runs Glance, and may reuse an idle sibling shell. If the tab has other busy panes, `glance-panel attach --force` permits another split.

## Summary is missing or stale

Check the footer's error and last-update age. Metadata can update while a summary is pending. Confirm `claude` is on `PATH`, its login works, and `GLANCE_MODEL` names an available model. Press `r` to request another pass.

For a diagnostic invocation that uses the model and updates the cache:

```sh
glance-panel summarize --session <session-id>
```

Include structured-result parsing errors in bug reports, but sanitize any transcript or response content. CLI response parsing and session-switch reliability are tracked fixes.

## Automatic opening is not working

Inspect `~/.glance/hook.log`. Reinstall the hook after moving the executable because the hook stores its absolute path. Manual installation works even if you previously declined the offer.

## Windows build fails at `std::os::unix`

The current herdr transport is Unix-only. Windows support is planned; this is not a missing Cargo flag.

## Report a problem

Use the [bug report form](https://github.com/adityamaanas/glance/issues/new?template=bug_report.yml). Include OS, Glance commit/version, Claude Code and herdr versions, invocation, and sanitized errors. See [security reporting](../SECURITY.md) for vulnerabilities.
