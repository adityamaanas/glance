#!/usr/bin/env python3
"""Check the Markdown documentation for broken local links, anchors and images.

    python3 scripts/check-docs.py

Checks every tracked-style Markdown file outside target/: relative links and
images must point at existing files, and #anchors must match a heading in the
target page (GitHub's slug rules). External http(s) links are not fetched.
Exits non-zero and lists every problem found.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKIP = {"target", ".git", "node_modules"}
LINK = re.compile(r"(?<!\!)\[[^\]]*\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")
IMAGE = re.compile(r"!\[[^\]]*\]\(([^)\s]+)\)")
HTML_SRC = re.compile(r"""<(?:img|a)\b[^>]*\b(?:src|href)=["']([^"']+)["']""")
FENCE = re.compile(r"^(```|~~~)")


def markdown_files():
    for path in sorted(ROOT.rglob("*.md")):
        if not SKIP.intersection(path.relative_to(ROOT).parts):
            yield path


def slug(heading: str) -> str:
    text = re.sub(r"<[^>]+>", "", heading).strip().lower()
    text = re.sub(r"[`*_~\[\]()]", "", text)
    text = re.sub(r"[^\w\- ]", "", text)
    return text.replace(" ", "-")


def anchors(path: Path) -> set:
    found, counts, fenced = set(), {}, False
    for line in path.read_text(encoding="utf-8").splitlines():
        if FENCE.match(line.strip()):
            fenced = not fenced
            continue
        if fenced:
            continue
        match = re.match(r"^#{1,6}\s+(.*?)\s*#*$", line)
        if match:
            base = slug(match.group(1))
            n = counts.get(base, 0)
            found.add(base if n == 0 else f"{base}-{n}")
            counts[base] = n + 1
    return found


def targets(path: Path):
    fenced = False
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if FENCE.match(line.strip()):
            fenced = not fenced
            continue
        if fenced:
            continue
        line = re.sub(r"`[^`]*`", "", line)
        for pattern in (LINK, IMAGE, HTML_SRC):
            for match in pattern.finditer(line):
                yield number, match.group(1)


def main() -> int:
    problems = []
    cache = {}
    files = list(markdown_files())
    for path in files:
        for number, target in targets(path):
            if re.match(r"^[a-z][a-z0-9+.-]*:", target):
                continue  # http:, https:, mailto: and similar
            file_part, _, anchor = target.partition("#")
            dest = (path.parent / file_part).resolve() if file_part else path
            where = f"{path.relative_to(ROOT)}:{number}"
            if not dest.exists():
                problems.append(f"{where}: missing {target}")
                continue
            if anchor and dest.suffix == ".md":
                if dest not in cache:
                    cache[dest] = anchors(dest)
                if anchor not in cache[dest]:
                    problems.append(f"{where}: no heading for #{anchor} in {dest.relative_to(ROOT)}")
    for problem in problems:
        print(problem)
    print(f"checked {len(files)} Markdown files: {len(problems)} problem(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
