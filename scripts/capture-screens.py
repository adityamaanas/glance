#!/usr/bin/env python3
"""Render real Glance screens for the documentation.

Runs the actual `glance-panel` binary in an isolated tmux server against the
fictional demo session in docs/assets/demo/, captures each view, and writes
SVG images to docs/assets/screens/. Nothing is read from your own agent
sessions and no model is called: a stub `claude` returns the demo summary.

    cargo build --release
    python3 scripts/capture-screens.py            # uses target/release/glance-panel
    python3 scripts/capture-screens.py --bin PATH --out DIR

Requires Unix and tmux 3.2+. Re-run it after any change to the panel's layout
and commit the updated SVGs with that change.
"""
import argparse
import html
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEMO = ROOT / "docs" / "assets" / "demo"
SESSION = "7f3c2a9e-demo-4b1d-9c0e-glance000001"
PROJECT = "-home-dev-acme-shop"
SOCKET = "glance-docs"

# (file name, columns, rows, keys pressed after the panel opens)
SCREENS = [
    ("panel", 100, 38, []),
    ("evidence", 100, 34, ["Enter", "Down", "Down"]),
    ("graph", 100, 26, ["g"]),
    ("rail", 100, 24, ["v"]),
    ("todos", 100, 18, ["t"]),
    ("first-run", 100, 30, ["fresh"]),
]

# A dark palette for the 16 named colours the panel uses.
PALETTE = {
    0: "#1b2b33", 1: "#f07178", 2: "#9ccf8a", 3: "#f2c46d",
    4: "#7aa7e0", 5: "#d7a0e8", 6: "#76d8cc", 7: "#c3cfd2",
    8: "#6b8791", 9: "#ff8f8f", 10: "#b7e3a4", 11: "#ffd98a",
    12: "#9cc2f2", 13: "#e7bdf3", 14: "#9ff0e6", 15: "#f2f6f5",
}
FG, BG = "#dfe8e6", "#0f2129"
CELL_W, CELL_H, PAD = 8.4, 17.0, 14


def run(cmd, **kw):
    return subprocess.run(cmd, check=True, text=True, capture_output=True, **kw)


def tmux(*args):
    return run(["tmux", "-L", SOCKET, "-f", "/dev/null", *args]).stdout


def prepare(workdir: Path, binary: Path, with_cache: bool) -> dict:
    claude = workdir / "claude"
    state = workdir / "state"
    transcript = claude / "projects" / PROJECT / f"{SESSION}.jsonl"
    transcript.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy(DEMO / "session.jsonl", transcript)
    stub = workdir / "claude-stub"
    stub.write_text('#!/bin/sh\ncat >/dev/null\ncat "$GLANCE_DEMO_SUMMARY"\n')
    stub.chmod(0o755)
    env = {
        **os.environ,
        "CLAUDE_CONFIG_DIR": str(claude),
        "GLANCE_HOME": str(state),
        "GLANCE_CLAUDE_BIN": str(stub),
        "GLANCE_DEMO_SUMMARY": str(DEMO / "summary.json"),
        "TZ": "UTC",
    }
    for key in ("HERDR_PANE_ID", "HERDR_SOCKET_PATH", "GLANCE_MODEL", "TMUX"):
        env.pop(key, None)
    # Answer the first-run hook offer so it does not cover the screenshots.
    state.mkdir(parents=True, exist_ok=True)
    (state / "config.json").write_text('{"hook_offer": "declined"}\n')
    if with_cache:
        run([str(binary), "summarize", "--session", SESSION], env=env)
        for text in (
            "Confirm the key retention window with product",
            "Check the refund test after webhooks land",
        ):
            run([str(binary), "todo", text, "--session", SESSION], env=env)
    return env


SGR = re.compile(r"\x1b\[([0-9;]*)m")


def parse(line: str):
    """Split one captured line into (text, style) runs."""
    style = {"fg": None, "bg": None, "bold": False, "dim": False, "rev": False}
    runs, pos = [], 0
    for match in SGR.finditer(line):
        if match.start() > pos:
            runs.append((line[pos:match.start()], dict(style)))
        pos = match.end()
        codes = [int(c) if c else 0 for c in match.group(1).split(";")]
        i = 0
        while i < len(codes):
            c = codes[i]
            if c == 0:
                style = {"fg": None, "bg": None, "bold": False, "dim": False, "rev": False}
            elif c == 1:
                style["bold"] = True
            elif c == 2:
                style["dim"] = True
            elif c == 7:
                style["rev"] = True
            elif c in (22,):
                style["bold"] = style["dim"] = False
            elif c == 27:
                style["rev"] = False
            elif 30 <= c <= 37:
                style["fg"] = PALETTE[c - 30]
            elif 90 <= c <= 97:
                style["fg"] = PALETTE[c - 90 + 8]
            elif 40 <= c <= 47:
                style["bg"] = PALETTE[c - 40]
            elif 100 <= c <= 107:
                style["bg"] = PALETTE[c - 100 + 8]
            elif c == 39:
                style["fg"] = None
            elif c == 49:
                style["bg"] = None
            elif c in (38, 48) and i + 2 < len(codes) and codes[i + 1] == 5:
                n = codes[i + 2]
                colour = PALETTE.get(n, FG if c == 38 else BG)
                style["fg" if c == 38 else "bg"] = colour
                i += 2
            elif c in (38, 48) and i + 4 < len(codes) and codes[i + 1] == 2:
                r, g, b = codes[i + 2:i + 5]
                style["fg" if c == 38 else "bg"] = f"#{r:02x}{g:02x}{b:02x}"
                i += 4
            i += 1
    if pos < len(line):
        runs.append((line[pos:], dict(style)))
    return runs


def svg(lines, cols, rows, title):
    width = cols * CELL_W + PAD * 2
    height = rows * CELL_H + PAD * 2 + 22
    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width:.0f}" height="{height:.0f}" '
        f'viewBox="0 0 {width:.0f} {height:.0f}" role="img" aria-label="{html.escape(title)}">',
        f"<title>{html.escape(title)}</title>",
        f'<rect width="100%" height="100%" rx="10" fill="{BG}"/>',
        '<g fill="#6b8791">'
        + "".join(f'<circle cx="{PAD + 6 + i * 16}" cy="{PAD + 2}" r="5"/>' for i in range(3))
        + "</g>",
        '<g font-family="ui-monospace,SFMono-Regular,Menlo,Consolas,monospace" '
        f'font-size="13.5" xml:space="preserve">',
    ]
    top = PAD + 22
    for row, line in enumerate(lines[:rows]):
        col = 0
        y = top + row * CELL_H
        for text, style in parse(line):
            fg, bg = style["fg"] or FG, style["bg"]
            if style["rev"]:
                fg, bg = bg or BG, fg
            x = PAD + col * CELL_W
            if bg:
                out.append(
                    f'<rect x="{x:.1f}" y="{y:.1f}" width="{len(text) * CELL_W:.1f}" '
                    f'height="{CELL_H}" fill="{bg}"/>'
                )
            attrs = f'fill="{fg}"'
            if style["bold"]:
                attrs += ' font-weight="bold"'
            if style["dim"]:
                attrs += ' opacity="0.6"'
            # One element per word keeps every glyph on its terminal cell, whatever the font.
            for word in re.finditer(r"\S+", text):
                wx = x + word.start() * CELL_W
                out.append(
                    f'<text x="{wx:.1f}" y="{y + CELL_H - 4:.1f}" {attrs} '
                    f'textLength="{len(word.group()) * CELL_W:.1f}" lengthAdjust="spacingAndGlyphs">'
                    f"{html.escape(word.group())}</text>"
                )
            col += len(text)
    out.append("</g></svg>")
    return "\n".join(out) + "\n"


TITLES = {
    "panel": "Glance panel following a fictional session: goal, current step, workstreams, plan, personal todos, open question, decisions and the last agent message",
    "evidence": "Evidence view: a selected plan step with the transcript turns that support it",
    "graph": "Session graph: plan steps, questions and decisions linked to the steps they follow from",
    "rail": "Rail view: summary items arranged by transcript turn in one lane per workstream",
    "todos": "Personal todo list with the selected reminder highlighted",
    "first-run": "The panel before any summary: the title and latest messages come straight from the transcript",
}


def capture(binary: Path, out: Path, name, cols, rows, keys):
    with tempfile.TemporaryDirectory(prefix="glance-screens-") as tmp:
        fresh = keys == ["fresh"]
        env = prepare(Path(tmp), binary, with_cache=not fresh)
        exports = " ".join(
            f"{k}={shlex.quote(env[k])}"
            for k in ("CLAUDE_CONFIG_DIR", "GLANCE_HOME", "GLANCE_CLAUDE_BIN", "TZ")
        )
        command = f"env {exports} {shlex.quote(str(binary))} --session {SESSION} --no-model"
        tmux("kill-server") if server_running() else None
        tmux("new-session", "-d", "-x", str(cols), "-y", str(rows), "-s", "docs", command)
        time.sleep(1.5)
        for key in [] if fresh else keys:
            tmux("send-keys", "-t", "docs", key)
            time.sleep(0.4)
        time.sleep(0.8)
        text = tmux("capture-pane", "-p", "-e", "-t", "docs")
        tmux("kill-server")
    lines = text.rstrip("\n").split("\n")
    lines = [re.sub(r"updated \d+[smh] ago", "updated 1m ago", l) for l in lines]
    (out / f"{name}.svg").write_text(svg(lines, cols, rows, TITLES[name]))
    print(f"wrote {out / name}.svg")


def server_running():
    return (
        subprocess.run(
            ["tmux", "-L", SOCKET, "has-session"], capture_output=True
        ).returncode
        == 0
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--bin", type=Path, default=ROOT / "target" / "release" / "glance-panel")
    parser.add_argument("--out", type=Path, default=ROOT / "docs" / "assets" / "screens")
    args = parser.parse_args()
    if not args.bin.is_file():
        sys.exit(f"{args.bin} not found; run cargo build --release or pass --bin")
    if shutil.which("tmux") is None:
        sys.exit("tmux is required")
    args.out.mkdir(parents=True, exist_ok=True)
    for name, cols, rows, keys in SCREENS:
        capture(args.bin.resolve(), args.out, name, cols, rows, keys)


if __name__ == "__main__":
    main()
