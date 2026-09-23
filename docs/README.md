# Glance documentation

[← Project README](../README.md)

Glance is a terminal panel that sits beside your coding agent and keeps the session's goal, current step, plan and loose ends in view. These pages are organized by what you are trying to do.

## New to Glance

| Start here | You will |
| --- | --- |
| [Getting started](getting-started.md) | Install Glance, open a panel next to a real session and try every view (10 minutes) |
| [Reading the panel](reading-the-panel.md) | Learn what each part of the screen means, with real screenshots |
| [Glossary](glossary.md) | Look up terms such as workstream, evidence and summary provider |

## How-to guides

Short, task-focused recipes. Each one stands alone.

| Guide | Use it when you want to |
| --- | --- |
| [Use Glance with herdr](how-to/herdr.md) | Open the panel automatically for every session, and show progress in herdr's sidebar |
| [Use tmux, Zellij or any other terminal](how-to/other-terminals.md) | Run Glance without herdr |
| [Follow Codex, Gemini, pi or OpenCode](how-to/other-agents.md) | Use an agent other than Claude Code, or read an exported transcript |
| [Follow Cursor](how-to/cursor.md) | Capture Cursor IDE chats or Cursor CLI runs |
| [Check the evidence and export a graph](how-to/evidence-and-graphs.md) | See why the panel says something, or share an offline map of the session |
| [Keep personal todos](how-to/personal-todos.md) | Track your own reminders beside the agent's plan |
| [Choose who writes summaries](how-to/summary-providers.md) | Pick the summary agent and model, or set a fallback |
| [Run without model calls](how-to/run-without-models.md) | Keep everything local, control cost, or clean up stored data |
| [Use Glance on Windows](how-to/windows.md) | Set up herdr's named pipe and hooks on Windows |

## Reference

Complete and exact. The command-line page is generated from the program itself.

| Reference | Contents |
| --- | --- |
| [Command line](reference/cli.md) | Every command and flag |
| [Keys and mouse](reference/keys.md) | Every key binding, by view |
| [Configuration file](reference/configuration.md) | Every `config.json` setting |
| [Files and environment](reference/files-and-environment.md) | What Glance stores where, and every environment variable |
| [Agent transcript formats](reference/agents.md) | Where each agent's sessions are read from, and what is included |

## Background

| Topic | Explains |
| --- | --- |
| [How Glance works](explanation/how-it-works.md) | The path from transcript to panel, and the design choices behind it |
| [Privacy and data handling](explanation/privacy.md) | What is read, stored and sent to a summary provider |
| [Compatibility](explanation/compatibility.md) | What has been tested on which platforms, terminals and agents |

## When something goes wrong

[Troubleshooting and FAQ](troubleshooting.md) is organized by symptom, such as "the panel is empty" or "the summary never updates".

## Contributing to these docs

Read the [documentation standard](STANDARD.md). Maintainer material lives in [maintainers/](maintainers/releasing.md).
