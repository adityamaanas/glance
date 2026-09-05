# Implementation checklist

[← Home](../README.md) · [Roadmap](../ROADMAP.md)

Work is delivered through focused PRs and granular commits. Checked items are implemented and validated at the stated level; planned support is not advertised as shipped. Cursor includes both its CLI and IDE conversations.

## 1. Repository presentation

- [x] Redesign README hierarchy, navigation, quick start, feature table, and badges.
- [x] Add accessible SVG hero, panel illustration, and rail illustration using fictional content.
- [x] Add usage, architecture, privacy, and troubleshooting guides.
- [x] Add contribution and security guidance, issue forms, and a PR template.
- [x] Reconcile stale roadmap descriptions and command names.
- [x] Validate 46 local links, SVG rendering, desktop/mobile README layout, and formatting.

## 2. Correctness and reliability

- [ ] Parse Claude's structured output field with response fixtures.
- [ ] Reject stale summary results after session changes.
- [ ] Preserve unrelated hooks during installation/removal, including grouped commands.
- [ ] Drain subprocess pipes concurrently; enforce timeouts and reap children.
- [ ] Pass transcript content through stdin; avoid logging ancestor prompt arguments.
- [ ] Quote shell commands correctly and validate attach ratios.
- [ ] Handle transcript truncation/replacement and partially written UTF-8.
- [ ] Validate cache/session paths, use safe atomic writes, and detect stale cache cursors.
- [ ] Resynchronize status/session after reconnects; bound retry behavior.
- [ ] Preserve compact tool outcomes for summary evidence.

## 3. Compatibility and summary controls

- [ ] Separate discovery, parsing, activity, and summary execution behind agent adapters.
- [ ] Add sanitized fixtures and deterministic boundary tests without paid model calls.
- [ ] Add CLI/env/config model precedence and per-agent defaults.
- [ ] Add refresh intervals, model-free operation, custom prompt configuration, and usage visibility.
- [ ] Chunk long-session input without marking omitted context as processed.
- [ ] Add cache cleanup/retention controls.

## 4. Navigation, evidence, and graph

- [ ] Select items by keyboard and mouse; show supporting transcript excerpts in a drawer.
- [ ] Add stable item IDs, validated source-turn references, and relationship edges.
- [ ] Collapse older completed branches while preserving access to their context.
- [ ] Add terminal graph view and narrow-terminal handling.
- [ ] Export a self-contained HTML graph with escaped content and optional opening.

## 5. Personal todos

- [ ] Add, select, complete, and delete todos in the panel and CLI.
- [ ] Store todos separately per session with stable IDs and update provenance.
- [ ] Let summaries update statuses only, backed by evidence; preserve user wording.
- [ ] Protect manual overrides until newer evidence exists.
- [ ] Make carrying todos across session changes an explicit user choice.

## 6. Discovery, placement, and setup

- [ ] Discover the latest session by working directory and provide a session picker.
- [ ] Add Stop-hook activity detection with settled-growth fallback.
- [ ] Add tmux and Zellij attachment; document manual terminal splits.
- [ ] Publish current step/progress to herdr sidebar metadata.
- [ ] Add unified setup/removal with idempotent hook registration.
- [ ] Package an optional Claude plugin for hook lifecycle management.
- [ ] Add Windows named-pipe transport, command launching, and platform-safe paths.

## 7. Agent integrations

- [ ] Claude Code: preserve current behavior through the adapter migration.
- [ ] Codex: discovery, rollout parsing, activity, and summary execution.
- [ ] OpenCode: discovery, database ingestion, and summary execution.
- [ ] Gemini CLI: discovery, transcript parsing, activity, and summary execution.
- [ ] pi: discovery, session parsing, activity, and summary execution.
- [ ] Cursor CLI: discovery, transcript parsing, activity, and summary execution.
- [ ] Cursor IDE: research available local interfaces; implement conversation discovery and ingestion.
- [ ] Record tested versions and distinguish fixture coverage from live verification for every adapter.

## 8. Distribution and final validation

- [ ] Automate release archives for macOS arm64/x86_64, Linux arm64/x86_64, and Windows.
- [ ] Add checksums, installer support, and installation smoke tests.
- [ ] Prepare Homebrew distribution and crates.io publishing configuration.
- [ ] Document release credentials/setup that maintainers must provide.
- [ ] Maintain an OS/terminal/agent compatibility matrix.
- [ ] Run formatting, lint, unit/integration tests, and platform builds on each relevant PR.
- [ ] Check narrow/wide terminal rendering, Unicode, and keyboard navigation.
- [ ] Refresh docs and release notes to describe verified final behavior.
