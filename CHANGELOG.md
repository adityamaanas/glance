# Changelog

## 0.2.0

- Branches: the summary models separate threads of work (for example one PR
  review among several) with a focus on the thread the newest turns belong to.
  The panel shows a `BRANCHES` strip, filters plan, questions and decisions to
  the focused thread, and lets you move or pin focus (`[` `]` `0` `p`).
- Rail view (`v`): trunk and one lane per branch, one row per item, drawn with
  box-drawing characters.
- Every item carries the transcript turn it arose at; turns are rendered to the
  model with absolute `[tN]` markers.
- `GLANCE_MODEL` overrides the summary model.
- `summarize` seeds the panel cache.
- Cache format versioned; earlier caches are discarded and rebuilt once.

## 0.1.0

- First working panel: header, topline, now, plan, open questions, decisions,
  blockers, last message from Claude.
- `attach` splits the herdr pane, reuses an idle sibling pane after a herdr
  restart, waits for the shell prompt and confirms the panel came up.
- Follows the pane to a new session after `/clear`; waits for a fresh session's
  first prompt.
- SessionStart hook (`hook`), offered once on first run inside the panel;
  skips subagents, non-herdr panes and print-mode runs.
- Summaries run through `claude -p` with a JSON schema on the user's own Claude
  login; heuristic fallback so the panel is never empty.
