# Evidence and session graphs

Press `e` or Enter to explore summary items. Up/Down and mouse clicks select an item; `j`/`k` scroll its transcript evidence. Esc returns to the panel. Press `g` to view explicit relationships between steps, questions, decisions, and blockers. The existing `v` rail still shows workstream lanes.

Each model-produced item carries a stable ID, up to eight source-turn references, and an optional parent item ID. Invalid turn references, unknown parents, and cycles are removed. Missing evidence is displayed explicitly; an association is still a model interpretation, not proof that a claim is correct.

Older completed workstreams collapse into a count in the branch strip. Cycle with `[` and `]` to focus them; focused branches remain visible.

## Export

```sh
glance-panel graph --session <id>
glance-panel graph --session <id> --html
glance-panel graph --session <id> --html session-graph.html --open
```

The export uses the existing summary cache and does not call the model. Refresh the summary first if no compatible cache exists. The HTML file contains the summary and supporting transcript excerpts, works offline, and supports search, workstream filtering, and evidence inspection. Treat it as private conversation data when sharing it.

The graph represents current summary items and their explicit relationships, not the agent's hidden reasoning or a permanent history of every earlier item. Cache format 4 adds evidence references and rebuilds earlier caches once.
