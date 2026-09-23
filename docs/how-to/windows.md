# Use Glance on Windows

[Docs home](../README.md) · [herdr](herdr.md) · [Compatibility](../explanation/compatibility.md)

Glance runs natively on Windows. Following a session in its own terminal window works everywhere; herdr integration needs one extra setting.

**You need:** your agent installed, and Glance. Install Glance in PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/adityamaanas/glance/releases/latest/download/glance-panel-installer.ps1 | iex"
```

This installs `glance-panel.exe` into `%USERPROFILE%\.cargo\bin` and adds it to your `PATH`; open a new terminal afterwards.

## Follow a session

Open a second Windows Terminal pane (`Alt`+`Shift`+`+`) in the project directory and run:

```powershell
glance-panel --cwd .
```

Everything in [Getting started](../getting-started.md) applies from step 3 on.

## With herdr

On Windows, herdr talks over a named pipe rather than a socket file, and Glance does not guess its name. Set it to the endpoint herdr reports:

```powershell
$env:HERDR_SOCKET_PATH = "<pipe endpoint from herdr>"
glance-panel attach
```

Automatic attachment recognizes PowerShell and Command Prompt and quotes the command for whichever one the target pane runs. Before opening a panel from the SessionStart hook, Glance checks (with a short, hidden PowerShell call) that the session is interactive rather than a headless `claude -p` run. If it cannot tell, it skips automatic attachment; `glance-panel attach` still works.

## Hooks

`glance-panel setup` (Claude Code) and `glance-panel setup --harness cursor` work on Windows. Keep `glance-panel.exe` at a stable path and re-run setup if you move it. Command Prompt cannot safely run paths containing `%`, `!` or `"`; install to a plain path or use PowerShell.

## Known limits

- The Windows build, named-pipe transport, hooks and shell quoting are tested automatically. A live Windows herdr session has not been verified yet; see [Compatibility](../explanation/compatibility.md).
- Cursor summaries pass their prompt as a command-line argument, which Windows limits in length. Use another [summary agent](summary-providers.md) for very long Cursor sessions.
