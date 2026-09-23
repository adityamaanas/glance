# Getting started

[Docs home](README.md) · [Reading the panel](reading-the-panel.md) · [Troubleshooting](troubleshooting.md)

In about ten minutes you will install Glance, open it beside a Claude Code session, watch it write its first summary, and try each view. You only need two terminal panes side by side; no multiplexer is required.

**You need:**

- [Claude Code](https://code.claude.com/docs), installed and logged in. Glance uses your Claude login to write summaries.
- A terminal where you can put two panes or windows side by side.

Using a different agent? Finish this tutorial with Claude Code if you can, then follow [Follow Codex, Gemini, pi or OpenCode](how-to/other-agents.md) or [Follow Cursor](how-to/cursor.md). The panel works the same way.

## 1. Install Glance

On macOS or Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/adityamaanas/glance/releases/latest/download/glance-panel-installer.sh | sh
```

On Windows, in PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/adityamaanas/glance/releases/latest/download/glance-panel-installer.ps1 | iex"
```

The installer puts a prebuilt `glance-panel` into `~/.cargo/bin` and adds that directory to your `PATH` if needed; open a new terminal afterwards. With [Rust](https://www.rust-lang.org/tools/install) 1.88 or newer you can build from source instead: `cargo install --locked --git https://github.com/adityamaanas/glance`.

Check it worked:

```sh
glance-panel --version
```

You should see `glance-panel` followed by a version number. The command is `glance-panel`; the project is called Glance.

## 2. Start a Claude Code session

In your first pane, go to any project and start Claude Code:

```sh
cd ~/code/my-project
claude
```

Give it a real task, for example *"Read the README and list three improvements."* Claude Code writes the conversation to a transcript file as it works. Glance reads that file; it never types into your session.

## 3. Open the panel beside it

In a second pane, go to the **same project directory** and start Glance:

```sh
cd ~/code/my-project
glance-panel --cwd .
```

`--cwd .` means "follow the most recent session in this project".

The first time, a banner asks whether to open Glance automatically for every Claude Code session. Press `n` for now; you can turn this on later with `glance-panel setup` (see [Use Glance with herdr](how-to/herdr.md)).

Straight away the panel shows what it can read directly from the transcript: the session title, the git branch, your first request and Claude's latest message. The footer says `heuristic · updated never`, because no summary has been written yet.

![The panel before its first summary: the title, branch, first request and latest message come straight from the transcript](assets/screens/first-run.svg)

## 4. Watch the first summary arrive

When Claude finishes a turn and the transcript has been quiet for a couple of seconds, the footer shows `⟳ analyzing…`. Glance is running a separate, tool-less `claude -p` call that reads the new part of the conversation. After a few seconds the panel fills in:

![The panel with a summary: goal, current step, workstreams, plan with progress, personal todos, an open question, decisions and the last message from Claude](assets/screens/panel.svg)

*Screenshots show a fictional session about retrying payment webhooks.*

- **WHAT WE ARE WORKING ON** is the goal of the whole session.
- **NOW** is the current step.
- **PLAN 3/5** tracks progress; ✔ is done, ▶ is in progress, ○ is still to do.
- **OPEN QUESTIONS**, **DECISIONS** and **BLOCKED ON** hold the loose ends.
- The footer shows which model wrote the summary, when, how many calls it took and roughly what they cost.

The summary updates after each turn, at most once every 30 seconds. Every section is explained in [Reading the panel](reading-the-panel.md).

Summaries use your Claude account like any other `claude -p` call. To keep everything local, start the panel with `--no-model` instead; see [Run without model calls](how-to/run-without-models.md).

## 5. Try each view

With the panel focused, press these keys. `Esc` always takes you back to the panel.

1. **Evidence.** Press `Enter`, then `↓` to move through items. The lower box shows the transcript turns behind the selected item, so you can check the summary against what actually happened.
   ![Evidence view with a plan step selected and its supporting transcript turns below](assets/screens/evidence.svg)
2. **Graph.** Press `g` to see how steps, questions and decisions follow from each other.
3. **Rail.** Press `v` for a timeline with one lane per workstream. Press `v` again to return.
4. **Your own todo.** Press `a`, type *"Check the new README section"*, and press `Enter`. It appears under **MY TODOS**. The wording is yours; the summary can only mark it done when the transcript shows it happened. Press `t` to manage the list, `x` to toggle and `d` to delete.
5. **Quit** with `q`. Reopening the panel later is instant, because the summary is cached.

## What you learned

You installed Glance, followed a live session, read a model-written summary, checked its evidence and added a todo of your own.

## Next steps

- Open the panel automatically: [with herdr](how-to/herdr.md), or with [tmux, Zellij or another terminal](how-to/other-terminals.md).
- Follow [another agent](how-to/other-agents.md), or [Cursor](how-to/cursor.md).
- Look up any key in the [keys reference](reference/keys.md).
- Something not working? See [Troubleshooting](troubleshooting.md).
