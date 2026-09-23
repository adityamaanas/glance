# Documentation standard

[Docs home](README.md) · [Contributing](../CONTRIBUTING.md)

This is the bar for Glance's documentation. It follows the [Diátaxis](https://diataxis.fr/) framework and the habits of well-presented command-line projects. Read it before adding or changing a page.

## 1. Four kinds of page, kept separate

Every page is one of these. If a page tries to be two, split it.

| Kind | Reader's question | Where | Rules |
| --- | --- | --- | --- |
| **Tutorial** | "Walk me through it once" | [getting-started.md](getting-started.md) | One path, every step runnable, ends in visible success. No options or alternatives. |
| **How-to guide** | "How do I do X?" | [how-to/](how-to/) | Titled by the task. Starts with what you need. Numbered steps, then a short "what to expect". Links to reference for details. |
| **Reference** | "What exactly does this do?" | [reference/](reference/) | Complete, exact and dry. One fact lives in one place. Generated from code where possible. |
| **Explanation** | "Why does it work this way?" | [explanation/](explanation/) | Background, design choices and limits. No step-by-step instructions. |

Two pages sit outside the four kinds: [reading the panel](reading-the-panel.md), which is the tool's visual reference, and [troubleshooting](troubleshooting.md), which is organized by symptom.

## 2. README contract

The project README is the front door, not the manual. In this order:

1. What Glance is, in one sentence.
2. A real screenshot within the first screen.
3. Install instructions for every supported channel.
4. A quick start that works for the most common setup, plus a "pick your setup" table.
5. A short feature tour: one line and one visual or example each.
6. Links into these docs.

Keep it under about 150 lines. Detail belongs in `docs/`.

## 3. Writing rules

- **Lead with the reader's goal.** Each page opens with one sentence saying what it helps you do, and how-to pages add a "You need" line.
- **Examples default to the most common setup:** Claude Code, with herdr where placement matters. Other agents and terminals get their own guide or a table, not a scatter of flags in every example.
- **Every command is copy-pasteable.** Use `<session-id>`-style placeholders only when unavoidable, and say where the value comes from.
- **Show what success looks like:** the expected output, screen or file.
- **State limits plainly.** Say what is untested or unsupported. Never imply a feature exists before it ships.
- **Plain words, short sentences, one idea per paragraph.** Define a term the first time it appears, or link to the [glossary](glossary.md).
- **Examples use fictional content only.** Never paste a real transcript, path, name or token.
- **Navigation line** at the top of each page linking to the docs home and the most related pages.

## 4. Visuals

- Screenshots are **real renders of the current build**, produced by [`scripts/capture-screens.py`](../scripts/capture-screens.py) from the fictional session in [`assets/demo/`](assets/demo/). Do not hand-draw screens of the product.
- Re-run the script in the same PR as any change to the panel's layout, and commit the updated SVGs.
- Every image has alt text describing what it shows. Illustrations that are not real output are labeled as such.

## 5. Keeping docs true

A change is not done until its documentation is:

- **Same PR.** Commands, keys, settings, files and behavior change together with their docs. The PR template has a checkbox for this.
- **Generated reference.** [`reference/cli.md`](reference/cli.md) is generated from `--help` and checked by `tests/cli_reference.rs`. Regenerate it with `GLANCE_UPDATE_DOCS=1 cargo test --test cli_reference`. A unit test fails if a `config.json` setting is missing from [`reference/configuration.md`](reference/configuration.md).
- **Checked links.** CI runs [`scripts/check-docs.py`](../scripts/check-docs.py), which fails on broken relative links, missing anchors and missing images.
- **One changelog.** User-visible changes go in [`CHANGELOG.md`](../CHANGELOG.md) under "Unreleased", written for users (what changed for them), not reviewers. CI fails a PR that changes `src/` without touching the changelog; add the `no-changelog` label for internal-only changes such as refactors and tests.
- **Versions change only at release time**, in a dedicated release PR (see [releasing](maintainers/releasing.md#publish-a-release)), never in feature PRs.

## 6. Later, at first release

Publish these docs as a searchable site generated from this same `docs/` folder (mdBook suits a Rust project), and add an install table covering prebuilt archives, installers and package managers.
