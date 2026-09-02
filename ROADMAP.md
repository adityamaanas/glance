# Roadmap

Ideas queued for glance, in rough priority. None are started unless marked.

## Branches (workstreams) and visual mode

The biggest planned change. A session often carries several threads at once:
one session reviews every open PR (the trunk) and, inside it, works each PR on
its own (a branch each). The panel today flattens that into one plan. It should
show the trunk and the branches, know which branch the conversation is on right
now, and let the user orient on one branch or on the whole.

"Branch" here means a thread of work, not a git branch and not a fork in the
message tree. Branches are what a person would name if asked "what are the
separate things going on in this session?"

### Phase 1: branches in the data, focus in the panel (done in 0.2.0)

- Summary schema gains `branches: [{ id, name, status, summary }]` with status
  `active`, `parked`, or `done`, plus `focus: <branch id | trunk>` for the branch
  the newest turns belong to. Every plan item, open question, decision and
  blocker gets a `branch` field (or `trunk`). Ids are stable across passes; the
  model receives the previous structure and must carry ids forward, merge
  duplicates, and only create a branch when the work is genuinely independent.
- Panel: a `BRANCHES` strip under NOW listing each branch with its status glyph,
  the focused one highlighted. Below it, PLAN / OPEN QUESTIONS / DECISIONS show
  the focused branch's items; trunk items stay visible in a dimmed style. Keys:
  `[` and `]` move focus, `0` returns to the whole session, `p` pins focus so
  it stops following the conversation.
- Cap at roughly six live branches; older done branches collapse into a count.
  A 64-column pane cannot show more, and the model's structure JSON has to stay
  small since it is re-sent every pass.

### Phase 2: rail view (done in 0.2.0, without node selection)

- `v` toggles a drawing of the same data: a vertical trunk with one lane per
  branch, time flowing down, in the style of `git log --graph`. Nodes are the
  plan steps, questions, decisions and blockers, each with its glyph, placed on
  the lane of their branch at the turn they appeared. Branch-off and merge points
  come from the first and last turn a branch touched.
- Draw it with plain box-drawing lines into a text buffer, not a chart library.
  Three visible lanes at 64 columns; wider panes show more; the rest fold into a
  `+n parked` marker. Scroll with `j`/`k`; selecting a node shows its excerpt in
  the drawer from the navigable panel.
- Node selection and the excerpt drawer wait on the navigable panel. Items carry
  a single `turn` index today; the navigable panel will want the full list.

### Phase 3: full session graph

- A `g` view that shows where open questions and decisions branch out of the
  sequence of work: the rail, plus edges for "this question came from that step"
  and "this decision closed that question". The model emits `from: <item id>` on
  questions and decisions; the view draws the edge.
- When the terminal is too small, `glance graph --html` writes the same graph as
  a single self-contained HTML file and opens it. The terminal stays the primary
  surface; the export exists for reading a long session after the fact.

### Boundaries

- This is structure of the work: branches, steps, questions, decisions. It is not
  a map of the reasoning (angles explored and dropped, contradictions,
  revisits). That is session-map's territory; keep the two apart.
- Everything still comes from the one summary pass per turn. No second model
  call for the graph.

## Navigable panel

Arrow keys or mouse select an item (a plan step, a decision, an open question)
and a drawer shows the transcript turns behind it. Needs the summary to carry turn
indices per item (`source_turns: [n, ...]`), which the summarizer can emit since it
sees the rendered turns; then the view maps indices back to `Transcript.turns`.

## Your own todos for the session

A section the user writes: reminders, steps the conversation has not reached
yet, things to raise with Claude later. The model keeps their status current
the same way it does for open questions, but the wording stays the user's.

- Panel: a `MY TODOS` section between PLAN and OPEN QUESTIONS. `a` opens a
  one-line input at the bottom of the pane; Enter adds the item, Esc cancels.
  `x` on a selected item toggles done by hand, `d` deletes. Selection uses the
  same navigation the navigable panel introduces, so build that first or together.
- Model side: each summary pass receives the user's todos (id, text, status) and
  returns status updates only: pending, in_progress, done, plus an optional
  one-line note ("done in turn 42: PR opened"). The model may not reword, add or
  delete a user item; it marks done only with evidence in the transcript, the same
  rule the plan follows. A user's manual toggle wins over the model's next pass
  until new evidence appears.
- Storage: `~/.glance/<session>.todos.json`, next to the summary cache. Items carry
  id, text, status, created timestamp, and who set the status last (user or model).
- Also writable from outside the panel: `glance todo "text"` appends to the todos
  of the session in the current herdr tab, so `! glance todo ...` works at the
  Claude prompt without leaving the conversation.
- Optional: follow the session across `/clear` the way the panel does, but ask
  before carrying todos over, since a cleared session is usually a new topic.

## herdr sidebar strip

Push a one-line current step and a progress count into herdr's own agent
sidebar with `herdr pane report-metadata --token step=... --token progress=3/7`,
and configure `[ui.sidebar.agents.rows_by_agent] claude` to show `$step` and
`$progress`. Gives every session a glance line without opening its tab.

## Outside herdr

Today the panel needs herdr for two things: which Claude session is in the
neighbouring pane, and the working/idle signal. Both have fallbacks that make
glance usable in a plain terminal, tmux, Zellij, WezTerm or Ghostty splits, or a
separate window:

- Session discovery: `glance --session <id>`, `glance --cwd <path>` (most recent
  session for that project), or a picker over `~/.claude/projects` like
  session-map's. Inside a Claude Code session, `! glance pick` can print the id.
- Turn-end signal: a Claude Code Stop hook that touches `~/.glance/<session>.stop`
  (glance offers to install it alongside the SessionStart hook), with transcript
  growth settling as the fallback that already exists.
- Placement: the user opens the split themselves; for tmux and Zellij a
  `glance attach` variant can issue the multiplexer's split command.

## Windows

Three things block it today, none of them deep:

- `herdr.rs` talks to herdr over a Unix socket (`std::os::unix::net`). herdr uses a
  named pipe on Windows. Put the transport behind a small trait and add a named-pipe
  client (the `interprocess` crate covers both).
- The SessionStart hook is a `/bin/sh` script that shells out to `ps`. Move its logic
  into the binary as `glance hook` (stdin JSON in, exit 0 out) so the settings entry
  is the same on every platform and no shell is involved.
- Paths: use `dirs` everywhere (already the case) and stop assuming `~/.cargo/bin`.

ratatui, crossterm, `claude -p` and the transcript layout under `~/.claude/projects`
already work on Windows.

## Other harnesses (Codex, OpenCode, Gemini CLI, pi)

herdr already detects these agents and reports their state, so the status signal is
free. What glance needs per harness is an adapter: where the transcript lives, how to
parse turns and free fields, and which headless command to summarize with so the
work stays on that user's own subscription.

- Introduce a `Harness` trait: `find_transcript(session) -> path`, `parse(line) -> Turn | Free`,
  `summarize_cmd() -> Command`. Claude Code becomes the first implementation.
- Codex CLI: session rollouts are JSONL under `~/.codex/sessions/` (verify layout and
  whether herdr's Codex integration reports the session id the same way).
- OpenCode: sessions live in a local SQLite database under its data dir (verify path and
  schema; needs `rusqlite`).
- Summarizer per harness: `codex exec`, `opencode run`, or fall back to `claude -p` when
  the user has it.

## Distribution

- GitHub Releases with prebuilt binaries for macOS (arm64, x86_64), Linux (x86_64,
  arm64) and Windows, built by `cargo-dist` on tag push. Removes the Rust requirement.
- Homebrew tap (`brew install <tap>/glance`) generated by the same cargo-dist run, with a
  caveats message pointing at `glance setup`.
- `curl | sh` installer (cargo-dist emits one) that ends by offering `glance setup`.
- crates.io publish for `cargo install glance`.

## Setup without the manual hook step

`cargo install` and `brew install` cannot safely edit `~/.claude/settings.json`, so the
hook needs one explicit, idempotent command:

- `glance setup [--yes]`: embeds the hook logic, writes the SessionStart entry into
  `~/.claude/settings.json` (merge, never overwrite), prints what it changed, and
  `glance setup --remove` undoes it. The curl installer runs it at the end; the brew
  caveats and README name it as the one post-install step.
- First run: if `glance` or `glance attach` finds no hook installed, offer to run setup
  once and remember the answer in `~/.glance/config.toml`.
- Alternative for the Claude side: ship the hook as a Claude Code plugin
  (`hooks/hooks.json` calling `glance hook`), installable with `claude plugin install`,
  so Claude Code manages the hook and settings.json is never touched.

## Smaller items

- Model selection. `GLANCE_MODEL` overrides the summary model today (0.2.0).
  Next: a `model` key in `~/.glance/config.json`, a `--model` flag on the panel,
  and, with other harnesses, a per-harness default so each user's summaries run
  on the model and subscription they already pay for. Measured on 2026-09-02:
  Haiku 4.5 produced a thinner plan (2 items against Sonnet's 7) and was not
  faster end to end, so Sonnet stays the default.
- Configurable prompt (`~/.glance/config.json`).
- Cache eviction for old sessions in `~/.glance/`.
- Handle very long sessions: the first pass caps input at 60k characters; a
  two-stage pass (chunk summaries, then merge) would keep early context.
