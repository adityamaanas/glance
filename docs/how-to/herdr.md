# Use Glance with herdr

[Docs home](../README.md) · [Other terminals](other-terminals.md) · [Troubleshooting](../troubleshooting.md)

[herdr](https://herdr.dev) knows which agent runs in each pane and whether it is busy. With it, Glance can open itself beside every new session, follow the pane through `/clear` and resumes, and show progress in herdr's sidebar.

**You need:** Glance installed ([Getting started](../getting-started.md)), herdr 0.8 or newer, and herdr's integration for your agent.

## Open the panel once

1. Enable herdr's Claude Code integration, if you have not already:

   ```sh
   herdr integration install claude
   ```

2. In the herdr pane where your agent runs, split and start the panel:

   ```sh
   glance-panel attach
   ```

   From inside a Claude Code conversation, prefix it with `!` to run it in that pane: `! glance-panel attach`.

Glance opens in a pane to the right, using 30% of the width. To change the width, pass `--ratio 0.4` (any value between 0 and 1).

`attach` reuses an idle shell pane in the same tab rather than splitting again, and does nothing if a Glance panel is already open. If the tab has other busy panes it stops with a message; add `--force` to split anyway.

## Open the panel automatically for every session

```sh
glance-panel setup
```

This registers three Claude Code hooks in your Claude settings:

- **SessionStart** opens a panel next to each new or resumed session. It only does so inside herdr, skips subagents and headless `claude -p` runs, and always exits successfully, so a failure never blocks the session.
- **Stop** and **StopFailure** tell Glance the moment a turn ends, so summaries start promptly. They store only the transcript path, size and time, never conversation text.

Setup keeps your other hooks, backs up the settings file, and is safe to run again. Answering `y` to the panel's first-run banner does the same thing. To undo it:

```sh
glance-panel setup --remove
```

Re-run `glance-panel setup` if you move the `glance-panel` binary. Setup uses Claude Code's direct executable-and-arguments hook format; update Claude Code if your version rejects it.

### Alternative: the Claude Code plugin

Instead of `setup`, you can install the same hooks as a Claude Code plugin. `glance-panel` must be on your `PATH`:

```sh
claude plugin marketplace add adityamaanas/glance
claude plugin install glance-panel@glance
```

Use either the plugin or `setup`, not both, or each turn is signalled twice.

## Follow a specific pane

The panel normally follows the agent pane next to it. To follow another pane, pass its herdr pane ID:

```sh
glance-panel --pane w7:p5
```

herdr reports which agent runs in the pane, and Glance picks the matching transcript reader automatically. After `/clear` or a resume, the panel switches to the new session on the next status change.

## Show progress in herdr's sidebar

Glance can report the current step and plan progress (for example `3/5`) to herdr's sidebar.

1. Turn it on, either per panel with `--sidebar` or for every panel in `~/.glance/config.json`:

   ```json
   { "sidebar_metadata": true }
   ```

2. Add the `$step` and `$progress` tokens to your herdr configuration:

   ```toml
   [ui.sidebar.agents.rows_by_agent]
   claude = [
     ["state_icon", "agent", "state_text"],
     ["$step", "$progress"],
     ["workspace", "tab"],
   ]
   ```

Glance refreshes the tokens every 30 seconds while the panel runs; they disappear 90 seconds after the panel closes. herdr's own agent state is unaffected.

## What to expect

- The header shows the agent's state (`● working`, `● idle`, …) instead of `○ no herdr`.
- Summaries wait until the agent is idle, so they never describe a half-finished turn.
- If something goes wrong, `~/.glance/hook.log` records what each hook decided. See [Troubleshooting](../troubleshooting.md#attach-does-not-open-a-panel).
