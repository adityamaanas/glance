# Unreleased development notes

[← Home](../README.md) · [Published-version changelog](../CHANGELOG.md) · [Compatibility](compatibility.md)

These changes are delivered as focused review PRs. No new version number,
GitHub release, Homebrew tap or crates.io publication is implied by this list.

## Session orientation and evidence

- Select summary items and open the supporting transcript turns.
- Explore validated item relationships in the terminal or an offline HTML graph.
- Keep rail/workstream focus and access older completed branches.
- Add personal todos in the panel or CLI. Wording remains user-owned; model
  changes are limited to evidence-backed statuses.
- Carry reminders between sessions explicitly, with concurrent writes protected
  by a lock and atomic persistence.

## Agents and placement

- Normalize Claude Code, Codex, Gemini CLI, pi, OpenCode and Cursor transcripts.
- Read OpenCode SQLite/WAL data without modifying it.
- Capture future Cursor IDE events through optional hooks and Cursor CLI JSON
  streams through a stdout-preserving pipeline.
- Discover sessions by cwd, picker, ID or explicit transcript path.
- Place panels with herdr, tmux or Zellij, or in a manual terminal split.
- Use Windows named pipes as well as Unix sockets.
- Register/remove Claude and Cursor hooks explicitly and preserve unrelated
  settings. Package an optional Claude hook plugin.
- Report optional current step/progress metadata to herdr's sidebar.

## Summary controls and reliability

- Default to the transcript agent's summary CLI. Explicit provider/model
  overrides and missing-executable fallback are available.
- Configure refresh intervals, prompt additions, model-free operation and cache
  retention. Report successful call counts and provider-reported cost, marking
  totals that include unpriced calls as lower bounds.
- Process long sessions forward in chunks, retaining early turns while clipping
  individual excerpts.
- Parse native structured envelopes, bound subprocess I/O and timeouts, and
  reject stale generations after session changes or rewinds.
- Recover partial UTF-8 and earlier transcript rewrites even with unchanged
  final records. Validate cached evidence against source fingerprints.
- Infer the herdr pane's agent when choosing a personal todo store.

## Distribution and upgrade notes

- Build five release targets, checksums, native installers and a Homebrew formula.
- Exercise real installers and extracted binaries on all five CI targets.
- Provide a manual, default-dry-run crate publishing workflow and maintainer
  release instructions.
- Refresh onboarding, privacy, troubleshooting, architecture and compatibility
  documentation.

Old summary caches rebuild when their version or transcript fingerprint no
longer matches. Claude stores retain their original session key; other agents
use an agent-prefixed key. Personal todos are separate from disposable caches.

Most helpers receive input on stdin. Cursor uses a bounded process argument,
and some providers may retain helper sessions. See [privacy](privacy.md) and
[summary providers](summary-providers.md). Read the compatibility matrix before
assuming authenticated live support for a particular installed agent version.
