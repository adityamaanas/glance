<p><img src="assets/logo/glance-lockup.svg" alt="glance" width="240"></p>

# glance

A live orientation panel for one Claude Code session, shown in a herdr split pane
beside the session. It answers "what are we working on, what is happening now,
what is the plan and how far along is it, what is open, what was decided" so
that coming back to a session after a break takes a glance instead of a re-read.

Nothing about how you use Claude Code changes. glance only reads.

## How it works

```
herdr tab
┌──────────────────────────────┬──────────────┐
│ Claude Code (pane A)         │ glance       │
│                              │ (pane B)     │
└──────────────────────────────┴──────────────┘
        │ writes transcript            ▲
        ▼                              │ tails
~/.claude/projects/<slug>/<session>.jsonl
```

1. `glance-panel attach`, run inside a Claude Code pane, asks herdr to split the
   pane to the right and starts `glance-panel --pane <A>` in the new pane.
2. The panel asks herdr which Claude session lives in pane A (herdr's Claude
   integration reports it) and tails that transcript.
3. Free fields come straight from the transcript: title, branch or worktree,
   linked PR, cost, last message from Claude.
4. herdr's `pane.agent_status_changed` event says when a turn ends. The panel
   then runs `claude -p` with Sonnet on your Claude subscription over the turns
   since the last pass and updates topline, now, plan, open questions, decisions
   and blockers. The result is cached in `~/.glance/<session>.json`, so a
   reopened panel is instant and the model only runs when the transcript grew.

The helper `claude -p` call runs with the `HERDR_*` variables removed (so herdr
does not register it as an agent), with session persistence off (no transcript
left behind), with no tools, and with no settings sources.

## Requirements

- macOS or Linux. Windows is on the roadmap; the herdr transport and the hook
  need a named-pipe client and a shell-free entry point first.
- [herdr](https://herdr.dev) 0.8 or later with its Claude integration installed
  (`herdr integration install claude`). herdr is how glance learns which session
  lives in the neighbouring pane and when a turn ends.
- Claude Code from the 2.1 line, logged in. Summaries run through `claude -p`
  on your own login; no API key.

## Install

```sh
cargo install glance-panel                 # from crates.io, once published
cargo install --git https://github.com/adityamaanas/glance   # or straight from the repo
cargo install --path .                     # from a checkout
```

The binary is `glance-panel` (the name `glance` is taken by other tools).
Prebuilt binaries and a Homebrew tap are on the roadmap.

## Privacy

glance reads the transcript Claude Code already writes to disk. The only place
any of it goes is Anthropic, through your own `claude -p` call, the same as
typing into Claude Code. The helper call runs with session persistence off, no
tools, and no settings sources. Nothing is sent anywhere else and nothing is
collected.

## Automatic attach (SessionStart hook)

The first time the panel opens it asks, in a banner, whether to open glance
automatically for every Claude Code session. `y` registers `glance-panel hook` as a
SessionStart hook (startup, resume, clear, fork) in `~/.claude/settings.json`,
merging into whatever is there and keeping a backup at
`settings.json.bak-glance`. `n` records the answer in `~/.glance/config.json`
and never asks again. The same can be done by hand:

```sh
glance-panel hook --install
glance-panel hook --uninstall
```

`glance-panel hook` reads the hook JSON on stdin and always exits 0. It does nothing
outside herdr, for subagents, and for `claude -p` runs, and logs one line per
decision to `~/.glance/hook.log`.

`glance-panel attach` itself is idempotent: if a sibling pane already runs glance it
stops; if a sibling pane is an idle shell (what herdr leaves behind after a
server restart) it starts the panel there instead of splitting; otherwise it
splits. It waits for the new shell's prompt before typing the command and
confirms the panel came up. A brand-new session has no transcript until its
first prompt, so the panel starts in a waiting state and fills in from there.
After `/clear`, the panel follows the pane to the new session on the next
status change.

## Use

```sh
glance-panel attach            # from a shell in a Claude Code pane: split right, start the panel
# From a running Claude Code session, type this at the Claude prompt (the ! prefix runs it in that pane):
#   ! glance-panel attach
glance-panel attach --ratio 0.35   # wider panel
glance-panel --pane w7:p5      # follow a specific herdr pane
glance-panel --session <id>    # follow a session directly, no herdr needed
glance-panel summarize --session <id>  # print the summary JSON, seed the cache, exit
```

The summary model is Claude Sonnet by default. Set `GLANCE_MODEL` to change it,
for example `GLANCE_MODEL=claude-haiku-4-5-20251001`.

Keys in the panel: `q` quit, `r` re-run the summary, `j`/`k` scroll, `v` rail view.
With branches: `[` and `]` move focus, `0` shows everything, `p` toggles between
following the conversation and staying pinned.

## Branches

A session often carries several threads at once: one session reviews every open
PR (the trunk) and works each PR on its own (a branch each). When the model finds
threads like that, the panel shows a `BRANCHES` strip under NOW with each thread
and its state (active, parked, done), highlights the one the newest turns belong
to, and filters the plan, questions and decisions to it, with trunk items dimmed.
Focus follows the conversation until you move it yourself; `p` lets go again.

Branches are threads of work, not git branches and not forks in the message
tree. Most sessions have none, and then the strip does not appear.

`v` switches to the rail: the trunk and one lane per branch, time flowing down,
one row per plan step, question, decision or blocker, placed on the lane of its
branch at the turn it appeared. Lanes that do not fit the pane fold into a
`+n more` marker and their items are tagged with the branch name instead.

## Files it touches

- `~/.glance/<session>.json`: cached panel state per session.
- `~/.glance/config.json` and `~/.glance/hook.log`: the hook offer answer and hook decisions.
- `~/.claude/settings.json`: only when you accept the hook offer or run `glance-panel hook --install`.
- Transcripts are read, never written.

## Logo

`assets/logo/glance.svg` (mark) and `glance-lockup.svg` (mark and wordmark). The
other drafts are kept under `assets/logo/drafts/`.

## Contributing

Issues and pull requests are welcome. Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`
and `cargo test` before opening one; CI runs the same three on macOS and Linux.
The [roadmap](ROADMAP.md) lists what is planned and the notes for picking each item up.

## License

Apache-2.0. See [LICENSE](LICENSE).
