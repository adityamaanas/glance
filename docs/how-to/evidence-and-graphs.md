# Check the evidence and export a graph

[Docs home](../README.md) · [Reading the panel](../reading-the-panel.md#evidence) · [Keys](../reference/keys.md)

Summaries are interpretations. Every item links to the transcript turns behind it, so you can check a claim in seconds, and the relationships between items can be exported as a page you can share.

**You need:** a panel with a summary (the footer names a model, not `heuristic`).

## Check why the panel says something

1. Press `Enter` (or `↓`) to open the item list.
2. Use `↑` and `↓`, or click, to select an item.
3. Read the **TRANSCRIPT EVIDENCE** box below. It shows the supporting turns, such as `[t8] TOOL` followed by that turn's content. Scroll it with `j` and `k`.
4. Press `Esc` to return.

`From item: retry` means this item follows from the item with ID `retry`. If an item has no supporting turns, the box says so; treat that item with more care.

Evidence shows where a claim came from, not that the claim is correct. Glance drops references to turns that do not exist and relationships that loop.

## See how items relate

Press `g`. Items are indented under the step they follow from, so a question sits under the step that raised it. Selecting an item shows its evidence, as above.

For a timeline instead, press `v` for the [rail](../reading-the-panel.md#rail).

## Print or export the graph

From the command line, using the saved summary (no model call):

```sh
glance-panel graph --session <session-id>
```

prints the tree:

```text
● [read-client] Read WebhookClient and its tests
  └─ [retry] Retry timeouts and 5xx with jittered backoff
    └─ [dedupe] Ignore duplicate deliveries in the receiver
```

To export a standalone web page:

```sh
glance-panel graph --session <session-id> --html session-graph.html --open
```

`--html` alone writes `glance-graph.html` in the current directory, and `--open` opens it in your browser. The page works offline, with search, a workstream filter, and each item's transcript excerpts. Add `--harness` for agents other than Claude Code.

## What to expect

- The export needs an up-to-date summary. If the transcript changed since the last summary, open the panel or run `glance-panel summarize --session <session-id>` first.
- The HTML file contains conversation excerpts. Treat it like the conversation itself before sharing it.
- The graph shows the current summary, not a history of every earlier version.
