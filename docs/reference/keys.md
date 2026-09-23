# Keys and mouse

[Docs home](../README.md) · [Reading the panel](../reading-the-panel.md) · [Command line](cli.md)

Every key Glance responds to. The footer always shows the most useful keys for the current view.

## Main panel

| Key | Action |
| --- | --- |
| `j` / `k` | Scroll down / up |
| `↓` / `↑` or `Enter` / `e` | Open the item list with evidence ([Evidence view](#evidence-and-graph-views)) |
| `g` | Open the relationship graph |
| `v` | Switch between the panel and the rail |
| `]` or `→` | Focus the next workstream (and stop following the conversation) |
| `[` or `←` | Focus the previous workstream |
| `0` | Show all workstreams |
| `p` | Toggle between a fixed focus and following the conversation |
| `r` | Summarize now, ignoring the refresh interval (re-reads the latest turn too) |
| `a` | Add a personal todo |
| `t` | Open the todo list |
| `q`, `Esc` or `Ctrl`+`C` | Quit |

## Evidence and graph views

| Key | Action |
| --- | --- |
| `↓` / `↑` | Select the next / previous item |
| Click, mouse wheel | Select an item |
| `j` / `k` | Scroll the evidence box |
| `e` or `Enter` | Close the evidence view |
| `g` | Switch between the evidence list and the graph |
| `Esc` | Return to the panel |

## Todo list

| Key | Action |
| --- | --- |
| `↓` / `↑`, click, mouse wheel | Select a todo |
| `x` | Toggle between pending and done |
| `d` | Delete the selected todo |
| `a` | Add a todo |
| `j` / `k` | Scroll the details box |
| `t` or `Esc` | Return to the panel |

## Typing a new todo

After pressing `a`, the footer becomes an input line.

| Key | Action |
| --- | --- |
| Any character | Type (up to 500 characters) |
| `Backspace` | Delete the last character |
| `Enter` | Save the todo |
| `Esc` | Cancel |
| `Ctrl`+`C` | Quit Glance |

## Rail

| Key | Action |
| --- | --- |
| `j` / `k` | Scroll |
| `v` | Return to the panel |
| `q` | Quit |

## First-run banner

| Key | Action |
| --- | --- |
| `y` | Install the Claude Code hooks (same as `glance-panel setup`) |
| `n` | Don't install them, and don't ask again |

## Mouse

The mouse is only captured while the evidence, graph or todo list is open. In the main panel and rail, your terminal's own text selection and scrolling keep working.
