# Privacy and data handling

[← Home](../README.md) · [User guide](usage.md) · [Security reporting](../SECURITY.md)

## What Glance reads

Glance reads local Claude Code transcripts under `~/.claude/projects/`. In pane-following mode it also reads herdr session, process, and status metadata. It does not modify source transcripts.

## What goes to the summarizer

With model updates enabled, Glance passes the session title, previous summary, and compact transcript text to a separate `claude -p` process. This can include code excerpts, paths, prompts, and other sensitive conversation content. It is not a redaction system.

The helper requests no session persistence, no tools, and no settings sources, and removes known herdr integration variables. Authentication, model routing, and service-side handling depend on the installed Claude CLI and its environment. Glance does not implement its own telemetry or separate upload service.

The prompt is currently a process argument: users with sufficient local process-inspection access may see it. Process input and isolation improvements are tracked in the checklist.

## Local storage

Caches and preferences are saved under `~/.glance/`. Caches contain derived conversation content and should be treated as sensitive. Hook logs contain decisions and errors; print-mode detection may include an ancestor command line.

Installing or removing the hook writes `~/.claude/settings.json` after making a backup. Declining the offer saves the preference in Glance's configuration.

## Model-free use

```sh
glance-panel --session <session-id> --no-model
```

This disables summary invocations for that panel. It continues to read transcripts and cached summaries. The separate `summarize` command invokes the model.

To remove stored summaries, close the relevant panels and delete their cache files from `~/.glance/`. Run `glance-panel hook --uninstall` before removing the binary if you enabled automatic opening.
