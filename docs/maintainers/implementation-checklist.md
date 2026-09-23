# Implementation checklist

[Docs home](../README.md) · [Roadmap](../../ROADMAP.md) · [Compatibility](../explanation/compatibility.md)

Checked items are implemented with the stated automated or local validation.
These development PRs do not constitute a published release or authenticated
live verification of every external agent.

## Repository presentation

- [x] README hierarchy, quick start, guide links and feature overview.
- [x] Accessible SVG hero/panel/rail illustrations with fictional content.
- [x] Usage, architecture, privacy, troubleshooting and contribution guides.
- [x] Security reporting, issue forms and PR template.
- [x] Final roadmap, release notes and compatibility matrix.

## Correctness and reliability

- [x] Structured response envelopes and visible-text fallbacks with fixtures.
- [x] Reject stale summaries after session switches and transcript rewinds.
- [x] Preserve unrelated grouped hooks during setup/removal.
- [x] Drain subprocess pipes, enforce deadlines and reap the direct child.
- [x] Use stdin for supported providers and avoid logging ancestor prompt arguments.
- [x] Document Cursor argv input and provider-specific retention/tool limits.
- [x] Quote launch commands, preserve split arguments and validate ratios.
- [x] Handle truncation, earlier rewrites, partial lines and partial UTF-8.
- [x] Validate session paths/cache fingerprints and use atomic state writes.
- [x] Resynchronize herdr state on reconnect and bound transport waits.
- [x] Preserve compact tool outcomes as evidence.

## Controls and agents

- [x] Separate discovery, normalization, activity and provider execution.
- [x] Sanitized fixtures and mock process contracts without paid model calls.
- [x] CLI/env/config model precedence and per-provider model configuration.
- [x] Refresh interval, custom prompt, no-model mode and reported usage/cost.
- [x] Forward long-session chunks without silently advancing past early turns.
- [x] Cache retention and dry-run cleanup.
- [x] Claude Code, Codex, Gemini CLI, pi, OpenCode and Cursor transcript adapters.
- [x] Cursor CLI stream forwarding/capture and Cursor IDE visible event hooks.
- [x] Matching-agent summary defaults and explicit missing-executable fallback.
- [x] Record format coverage and identify absent live version verification.

## Navigation and personal todos

- [x] Keyboard/mouse selection and supporting transcript drawer.
- [x] Stable item IDs, validated source turns and relationship edges.
- [x] Completed workstream collapsing and focus.
- [x] Terminal graph plus offline HTML export with escaped content.
- [x] Panel/CLI add, toggle, delete and list for personal todos.
- [x] Per-session/per-agent stores, provenance, locks and atomic updates.
- [x] Model status-only updates and manual override protection.
- [x] Explicit carry across sessions; no automatic todo transfer.
- [x] Infer the current herdr pane's agent for todo commands.

## Discovery, placement and setup

- [x] Latest session by cwd and bounded session picker.
- [x] Stop markers and settled-growth fallback.
- [x] herdr, tmux and Zellij placement plus manual split instructions.
- [x] Optional herdr sidebar step/progress metadata with expiry.
- [x] Idempotent setup/removal and optional Claude plugin packaging.
- [x] Windows named pipes, platform paths and command quoting.
- [x] Local tmux smoke interaction for panel/todo navigation.

## Distribution and validation

- [x] Archive builds for macOS ARM64/x86-64, Linux ARM64/x86-64 and Windows x86-64.
- [x] Checksums and real shell/PowerShell installer smoke tests on all five CI targets.
- [x] Homebrew formula generation and manual crates.io workflow with default dry run.
- [x] Document maintainer credentials, environment protection and release steps.
- [x] Crate packaging/compile dry run without publication.
- [x] Formatting, lint, unit/integration checks and minimum Rust version CI.
- [x] Automated narrow/wide rendering geometry, Unicode and input coverage.
- [x] Documentation/illustration checks and combined PR validation.

## Explicit remaining release and live checks

- [ ] Publish a versioned release and crate after maintainer review.
- [ ] Configure and publish a Homebrew tap if desired.
- [ ] Record authenticated live versions for all six summary providers.
- [ ] Exercise actual Cursor IDE/CLI sessions against their installed versions.
- [ ] Complete interactive herdr coverage on all three OS families and live Zellij coverage.
- [ ] Broaden terminal/font/theme appearance checks beyond current fixtures.

These unchecked items need release authority, credentials or specific live
environments. No paid model call or external publication is part of automated CI.
