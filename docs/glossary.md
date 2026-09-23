# Glossary

[Docs home](README.md) · [Reading the panel](reading-the-panel.md)

Terms used in Glance and these docs.

### Agent

The coding assistant whose session Glance follows: Claude Code, Codex, Gemini CLI, pi, OpenCode or Cursor.

### Evidence

The transcript turns that support a summary item, shown as `[t8]`, `[t9]` and so on. Evidence shows where a claim came from, not that it is true. See [Check the evidence](how-to/evidence-and-graphs.md).

### Focus

The workstream the panel is showing. By default focus follows the conversation; `[`, `]` and `0` fix it, and `p` switches back to following.

### Harness

Glance's name for which agent's transcript format to read, chosen with `--harness`. The same word selects the agent that writes summaries (`--summary-harness`).

### Heuristic view

What the panel shows before any summary exists: the title, first request and latest message, read directly from the transcript. The footer says `heuristic`.

### herdr

A terminal workspace for coding agents that knows which agent runs in each pane and whether it is busy. Optional; see [Use Glance with herdr](how-to/herdr.md).

### Hooks

Small commands an agent runs at certain moments, such as when a session starts or a turn ends. `glance-panel setup` installs Glance's hooks for Claude Code or Cursor.

### Item

One entry in the summary: a plan step, open question, decision or blocker. Each has an ID, such as `retry`, used for evidence and relationships.

### Rail

A timeline view (`v`) listing every item in the order it arose, with one lane per workstream.

### Session

One agent conversation, identified by a session ID. `/clear` or a resume starts a new session.

### Summary

The goal, current step, workstreams, plan, questions, decisions and blockers, written by a summary agent from the transcript and saved in a cache.

### Summary provider

The agent CLI that writes summaries. By default it is the session's own agent. See [Choose who writes summaries](how-to/summary-providers.md).

### Todo (personal todo)

A reminder you write, shown under MY TODOS. Only you can add, reword or delete it; the summary can only update its status, with evidence.

### Transcript

The file an agent writes as the conversation happens. Glance only reads it.

### Trunk

Items that concern the session as a whole rather than one workstream. They stay visible whatever the focus.

### Turn

One entry in the conversation: your message, the agent's reply, a tool call or a tool result. Turns are numbered from `t0`.

### Workstream

A separate thread of work within one session, such as fixing a bug while a flaky test waits. The panel labels workstreams **BRANCHES**, but they are unrelated to git branches. Most sessions have none.
