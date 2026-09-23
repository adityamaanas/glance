# Keep personal todos

[Docs home](../README.md) · [Reading the panel](../reading-the-panel.md#todo-list) · [Keys](../reference/keys.md)

Personal todos are your own reminders, shown under **MY TODOS** beside the agent's plan. You write them; the summary can only mark them done or in progress when the transcript shows it, and it can never add, reword or delete one.

**You need:** a panel open on a session, or the session's ID for the command line.

## In the panel

| Key | Does |
| --- | --- |
| `a` | Add a todo. Type it, then `Enter` to save or `Esc` to cancel. |
| `t` | Open or close the todo list |
| `↑` `↓` or click | Select a todo in the list |
| `x` | Toggle the selected todo between pending and done |
| `d` | Delete the selected todo |
| `j` `k` | Scroll the details box |

A todo is one line of up to 500 characters; a session can hold 100.

## From the command line

Every command prints the session's todos as JSON and never calls a model.

```sh
glance-panel todo "Ask about the rollout plan" --session <session-id>   # add
glance-panel todo --session <session-id>                               # list
glance-panel todo --session <session-id> --set todo-1 --status done    # change status
glance-panel todo --session <session-id> --set todo-1 --status in-progress
glance-panel todo --session <session-id> --delete todo-1               # delete
```

Inside a herdr agent pane you can leave out `--session`; Glance uses that pane's session and agent. This also works from inside Claude Code with `!`:

```text
! glance-panel todo "Check the error budget before merging"
```

For other agents outside herdr, add `--harness`, for example `glance-panel --harness codex todo --session <session-id>`.

## How status updates from the summary work

Each time the summary is written, it sees your todos and may report that one is now in progress or done, with the transcript turns that show it. Glance only accepts that update when:

- the todo still exists and you have not edited it since the summary started;
- the evidence comes from turns after your last manual change.

Your own changes always win. If you mark a todo pending again, the summary can only change it once newer evidence appears. The details box in the todo list shows who set the current status (`set by user` or by the summary) and, for summary updates, the note and supporting turns.

With `--no-model`, todos work exactly the same; only your own changes happen.

## Carry todos to a new session

After `/clear` or when starting fresh, todos stay with their original session. To copy their wording into the new session as fresh pending items:

```sh
glance-panel todo --session <new-session-id> --carry-from <old-session-id>
```

The original list is not changed. Running it twice adds a second copy.

## What to expect

- Todos are stored in `~/.glance/<session-id>.todos.json` (for other agents, `<agent>--<session-id>.todos.json`). Cache cleanup never removes them.
- Edits from the panel and the command line can happen at the same time; a short lock keeps them from overwriting each other.
- If the file is damaged, Glance reports it and leaves it untouched rather than overwriting it.
