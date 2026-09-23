# Troubleshooting and FAQ

[Docs home](README.md) · [Reading the panel](reading-the-panel.md) · [Files and environment](reference/files-and-environment.md)

Find your symptom below. Two commands help with almost every problem, and neither calls a model:

```sh
glance-panel pick --list                          # which sessions can Glance find?
glance-panel transcript --session <session-id>    # what does Glance read from one?
```

Add `--harness <agent>` to both for agents other than Claude Code.

## The panel can't find my session

**`no matching sessions`, or the panel says `waiting for the first prompt`.** An agent only writes its transcript after the first prompt. Send a prompt, then start the panel again.

**It finds a session, but the wrong one.** `--cwd .` picks the most recent session whose recorded working directory is this directory. Check with `glance-panel pick --cwd . --list`, then choose one with `--session <session-id>`.

**It's not Claude Code.** Tell Glance which agent with `--harness codex` (or `gemini`, `pi`, `opencode`, `cursor`). Under herdr this is usually detected automatically. See [Follow Codex, Gemini, pi or OpenCode](how-to/other-agents.md).

**The agent keeps its files somewhere unusual.** Set the location variable from [Files and environment](reference/files-and-environment.md#where-agents-keep-their-sessions), or point at the file with `--transcript <path>`.

**A Cursor IDE chat is missing.** Capture only covers chats after `glance-panel setup --harness cursor`. Earlier chats need an exported transcript and `--transcript`. See [Follow Cursor](how-to/cursor.md).

## Attach does not open a panel

**`tab already has other panes; use --force to split anyway`** (herdr). Glance avoids splitting a busy tab. Add `--force`, or run `attach` from a tab with an idle shell pane, which it reuses.

**Nothing happens** (herdr). A Glance panel is probably already open in that tab; `attach` does not open a second one.

**`no supported multiplexer detected`.** You are not inside herdr, tmux or Zellij. Open a split yourself and run `glance-panel --cwd .`. See [Use tmux, Zellij or any other terminal](how-to/other-terminals.md).

**The panel doesn't open automatically for new sessions.** Automatic opening needs herdr and the hooks from `glance-panel setup`. Check `~/.glance/hook.log`: each line records what the hook decided, such as `skip: not in herdr`, `skip: subagent`, `skip: print mode` or `attach failed: …`. Re-run `glance-panel setup` if you moved the binary.

**`ratio must be greater than 0 and less than 1`.** Use a fraction, for example `--ratio 0.35`.

## The summary never updates or shows an error

Look at the footer first ([what it means](reading-the-panel.md#the-footer)).

**The footer says `heuristic` and nothing changes.**

- Model calls may be off: check for `--no-model` or `"no_model": true` in `config.json`.
- The agent may still be busy. Summaries wait for the turn to end, then at least `refresh_seconds` (30 by default) between runs. Press `r` to summarize now.

**A red `⚠` message.** The summary agent failed. The previous summary stays visible. Common causes:

| Message mentions | Fix |
| --- | --- |
| `summary CLI not found` | Install the agent's CLI, put it on your `PATH`, or set `GLANCE_<AGENT>_BIN`. See [Choose who writes summaries](how-to/summary-providers.md#point-glance-at-a-specific-executable). |
| login, auth, quota or rate limit | Run the agent's CLI once yourself to fix its login or quota. Glance uses that CLI's account. |
| `timed out` | The CLI took more than 150 seconds. Try a smaller model, or press `r` later. |
| `exceeds the Windows command-line limit` / `single-argument limit` | Long Cursor sessions; switch to another `--summary-harness`. |
| a model name | Choose a model your account can use ([model settings](how-to/summary-providers.md#change-the-model)). |

To see the full error outside the panel, run one summary from the command line. This calls the model:

```sh
glance-panel summarize --session <session-id>
```

**The summary was replaced by the heuristic view.** The transcript was rewritten (for example by a rewind), so the old summary no longer matched. Glance rebuilds it on the next summary.

## Todos or configuration errors

**`invalid todo file; original preserved`** or **`parse ~/.glance/config.json`.** The file is not valid JSON. Glance leaves it untouched; fix or delete it, then retry.

**`todo file is busy`.** Another panel or command held the todo lock for more than two seconds. Retry.

**`pass todo --session <id> outside herdr`.** Outside herdr, todo commands need `--session` (and `--harness` for non-Claude agents).

## FAQ

**Does Glance change my agent's behavior or files?**
No. It reads transcripts and never writes to them or to your session. The only files it changes outside `~/.glance/` are hook settings, and only when you run `setup`.

**What does it cost?**
Each summary is one call to your agent's CLI on your account, covering only new turns. The footer shows the call count and reported cost. Use `--no-model` for no calls at all, or [spend less](how-to/run-without-models.md#spend-less-when-summaries-are-on).

**Can I trust the summary?**
Treat it as a helpful interpretation. Open the [evidence](how-to/evidence-and-graphs.md) behind any item to check it against the conversation.

**Why are there "BRANCHES" when I only use one git branch?**
They are [workstreams](glossary.md#workstream), separate threads of work within the conversation, not git branches.

**Does it work without herdr?**
Yes. herdr adds automatic opening and live agent status; everything else works in any terminal.

**Where is my data?**
In `~/.glance/`. See [Files and environment](reference/files-and-environment.md) and [Privacy](explanation/privacy.md).

## Report a problem

Use the [bug report form](https://github.com/adityamaanas/glance/issues/new?template=bug_report.yml). Include your OS, Glance version (`glance-panel --version`), agent and terminal versions, the command you ran and the error. Remove private conversation content from anything you paste. Report security issues privately, as described in [SECURITY.md](../SECURITY.md).
