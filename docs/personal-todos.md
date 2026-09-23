# Personal todos

Keep reminders beside the model's plan. Your wording stays yours; Glance only accepts model status changes with supporting transcript turns.

| Key | Action |
| --- | --- |
| `a` | Add a reminder; Enter saves, Esc cancels |
| `t` | Open or close your todo list |
| Up / Down / click | Select a todo in the list |
| `x` | Toggle the selected todo between pending and done |
| `d` | Delete the selected todo |
| `j` / `k` | Scroll its evidence |

The main panel shows **MY TODOS** between the plan and open questions. A reminder can contain up to 500 characters, with at most 100 reminders per session. Edits from another panel or the CLI appear automatically.

## From the command line

```sh
glance-panel todo "Ask about the rollout" --session <id>
glance-panel todo --session <id>
glance-panel todo --session <id> --set todo-1 --status done
glance-panel todo --session <id> --set todo-1 --status in-progress
glance-panel todo --session <id> --delete todo-1
```

Inside a herdr agent pane, omit `--session` to use that pane's session, including from a Claude Code `!` command. Todo commands return JSON and never call a model.

## Status and ownership

Each summary pass receives the current todo list and can return `pending`, `in_progress`, or `done`, a brief note, and source-turn references. Glance rejects unknown IDs, deleted items, stale revisions, and updates without evidence after your last manual edit. The model cannot add, delete, or reword a reminder. Evidence links still reflect a model interpretation; you can always correct its status.

Manual edits take precedence over an in-flight summary. Later summaries can update status when new transcript evidence appears. Rewritten transcript history cannot silently override a manual edit tied to the old history. With `--no-model`, personal todos continue to work; only manual status changes occur.

Todos live in `~/.glance/<session>.todos.json` (or `GLANCE_HOME`). Writes use a separate advisory lock and an atomic replacement so panels and CLI commands do not overwrite each other's changes. Invalid files are reported and preserved. Cache cleanup leaves todos intact.

## After clearing a session

Todos stay with their original session. To explicitly copy their wording into another session as fresh pending items:

```sh
glance-panel todo --session <new-id> --carry-from <old-id>
```

The original list is preserved. Copying again adds another set of reminders.
