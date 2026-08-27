---
type: guide
title: "Quickstart: navigating the claudebar wiki"
description: "Entry point for the claudebar wiki — what this single native Rust binary does, the major documentation domains, and a task-routing map to the render pipeline, segments, config/CLI, the Claude Code hook, distribution, updating, and testing pages."
tags: [quickstart, index, claudebar, routing, statusline]
verified:
  - by: openwiki/0.4.0
    at: 2026-08-26T22:48:34.063Z
sources:
  - id: openwiki-source-8037e2358a2c4f9b2c722a11
    resource: repo://AGENTS.md
  - id: openwiki-source-651d1fb6c9e49916a916ab51
    resource: repo://Cargo.toml
  - id: openwiki-source-ed8bf05e307c6278442542c2
    resource: repo://src/lib.rs
  - id: openwiki-source-b55a21a31ede1b56cd31a6a6
    resource: repo://src/main.rs
  - id: openwiki-source-c5edfb46b7c4acb766451a37
    resource: repo://src/model/config.rs
  - id: openwiki-source-5d4fb36fe9d34b6bc366e220
    resource: repo://src/model/input.rs
  - id: openwiki-source-1d33473d874a4090bb6026e0
    resource: repo://src/render/mod.rs
  - id: openwiki-source-3a6ff89030cacfe8ee730edf
    resource: repo://src/sanitize.rs
  - id: openwiki-source-f4acb0adeb73aa0d54303a27
    resource: repo://src/segment/mod.rs
  - id: openwiki-source-0ecba5538b5fd9860f10332f
    resource: repo://src/update.rs
generated: {by: "openwiki/0.4.0", at: "2026-08-26T22:48:34.063Z"}
---

# Quickstart: navigating the claudebar wiki

claudebar is a **Powerline-style status line for Claude Code**: a single native
Rust binary that reads Claude Code's session JSON from stdin and writes a
themed, styled ANSI status line to stdout. It is a plain CLI tool and hook —
**not** an OpenWiki repository itself, even though this documentation lives
under an `openwiki/` folder. It ships with a TUI configurator, built-in themes
and styles, a Bash fallback for zero-toolchain environments, and installers
with supply-chain verification.

> **Start here.** This page maps the wiki's major domains and routes you to the
> page that matches your task. When a task touches the render path, the hook
> integration, or a new feature, read the linked page first — most pages carry
> the invariants you must respect to change the code safely.

## What it is

- **A single native Rust binary** (`src/main.rs` dispatches subcommands; the
  render path lives in `src/render/` and is re-exported through
  `src/lib.rs`). Configuration-less operation renders the 8-segment default
  using the Tokyo Night palette and Powerline style (`src/model/config.rs`).
- **A hook, not a server.** Claude Code's `statusLine` invokes
  `claudebar render`, which parses stdin JSON and emits the ANSI line. The
  contract is always stdin JSON → stdout ANSI; `InputData::parse` is
  infallible and `render_line` never panics.
- **Composable segments, themes and styles.** 11 renderable segments compose
  the line; 16 built-in themes and 7 styles are resolved by name.

## Reading order / domain map

The wiki is organized into four domains that mirror how you interact with the
tool and the code:

| Domain | Page | Covers |
|---|---|---|
| Architecture | [render pipeline](/openwiki/architecture/render-pipeline.md) | the single render hot path shared by the hook and TUI preview (`render_line` / `render_with`) |
| Architecture | [input parsing](/openwiki/architecture/input-parsing.md) | how stdin session JSON is parsed into `InputData` via the forgiving `Coerce<T>` deserializer |
| Concepts | [segments](/openwiki/concepts/segments.md) | the `Segment` trait seam, `RenderCtx`, `SegmentWriter`, each of the 11 segments |
| Concepts | [themes and styles](/openwiki/concepts/themes-and-styles.md) | the fixed-struct `Theme` color slots, `Style`/`GlyphSet`, and the built-in registries |
| Concepts | [rate limits](/openwiki/concepts/rate-limits.md) | the rate-limits segment, its windows/thresholds, and the cross-session sync store |
| Concepts | [security and sanitization](/openwiki/concepts/security-and-sanitization.md) | terminal-injection hardening applied to every host-provided string |
| Concepts | [TUI configurator](/openwiki/concepts/tui-configurator.md) | the feature-gated interactive configurator and its live preview |
| Integrations | [Claude Code hook](/openwiki/integrations/claude-code-hook.md) | `setup` patching `settings.json`'s `statusLine` key, and the legacy bash statusline |
| Operations | [CLI and config](/openwiki/operations/cli-and-config.md) | every subcommand, the shared render overrides, and the `config.toml` schema |
| Operations | [install and distribution](/openwiki/operations/installation-and-distribution.md) | install.sh, npm, Homebrew, release channels, SHA256/provenance model |
| Operations | [update command](/openwiki/operations/update-command.md) | `claudebar update`, channel-aware version comparison, exit codes |
| Operations | [releasing](/openwiki/operations/releasing.md) | CalVer version model, cargo-dist CI pipeline, npm publish |
| Testing | [testing overview](/openwiki/testing/overview.md) | Rust unit/golden tests, bats shell tests, CI end-to-end verification |

## Central invariants to remember

These run across the whole codebase and are explained in depth on the linked
pages:

- **Single render path.** `render_line` is the one entrypoint shared by the
  hook and the TUI preview; there is deliberately no second ANSI composition
  path, so the live preview can never diverge from what the hook emits
  (`src/render/mod.rs`, `src/lib.rs`).
- **Deterministic, testable segments.** Segments never read env/time directly;
  `now`, `home`, and `tz_offset_seconds` are injected through `RenderCtx`
  (`src/segment/mod.rs`). Amibient state is resolved once at the top of the
  render path by `render_line`.
- **Forgiving inputs.** Every JSON field is deserialized with `Coerce<T>` so a
  wrong-typed or absent field degrades to `None` instead of aborting the
  render (`src/model/input.rs`).
- **Sanitize all host strings.** Every host-provided string is stripped of
  terminal-control bytes (`strip_control`) before reaching the rendered line
  (`src/sanitize.rs`).
- **The render hot path stays offline.** Only `claudebar update` touches the
  network (via `curl` against the GitHub releases API), and it has its own
  documented exit codes (`src/update.rs`).

## Task routing map

| Intent | Start at | Key symbols | Validation |
|---|---|---|---|
| First read of how a line is built | [render pipeline](/openwiki/architecture/render-pipeline.md) then [segments](/openwiki/concepts/segments.md) | `render_line`, `render_with`, `RenderCtx`, `SegmentWriter` | `cargo test render` |
| Add or change a segment | [segments](/openwiki/concepts/segments.md) | `Segment`, `SegmentKind::as_segment`, `SegmentWriter` | new unit tests + golden snapshot |
| Understand hook JSON / why a field degrades | [input parsing](/openwiki/architecture/input-parsing.md) | `InputData::parse`, `Coerce<T>` | `cargo test input` |
| Change the hook wiring (`statusLine`) | [Claude Code hook](/openwiki/integrations/claude-code-hook.md) | `setup::apply`, `desired_status_line` | `cargo test setup`, `claudebar setup` |
| Tweak a theme or style | [themes and styles](/openwiki/concepts/themes-and-styles.md) | `themes::get`, `styles::get` | `cargo test themes styles` |
| Change rate-limit rendering or sync | [rate limits](/openwiki/concepts/rate-limits.md) | `rate_limits`, `limit_sync` | `cargo test rate_limits` |
| Harden or audit output | [security](/openwiki/concepts/security-and-sanitization.md) | `sanitize::strip_control` | `cargo test sanitize` |
| Change TUI configurator | [TUI configurator](/openwiki/concepts/tui-configurator.md) | `App`, `ui::draw`, preview via `render_with` | `cargo test -F tui` |
| Everything command/flag/config | [CLI and config](/openwiki/operations/cli-and-config.md) | `Cli`, `Command`, `Config`, `Overrides` | `cargo test cli` |
| Install / distribute / channels | [install and distribution](/openwiki/operations/installation-and-distribution.md) | `install.sh`, npm, Homebrew tap | `bats tests/install.bats` |
| Wire `claudebar setup` | [Claude Code hook](/openwiki/integrations/claude-code-hook.md) + [CLI](/openwiki/operations/cli-and-config.md) | `setup`, `doctor`, `smoke` | `claudebar doctor`, `claudebar smoke` |
| Cut a release | [releasing](/openwiki/operations/releasing.md) | `Cargo.toml` version, `release.yml`, npm publish | release CI workflow |
| Check for a newer release | [update command](/openwiki/operations/update-command.md) | `update`, `Version` | `claudebar update --check` |
| Validate a change end-to-end | [testing overview](/openwiki/testing/overview.md) | unit, insta golden, bats, CI | `task test`, CI jobs |

## Related conventions

- The render hot path must build with `--no-default-features` (the `tui`
  feature is the only extra; everything else is unconditionally included).
- The `Vec<SegmentKind>` in `Config` encodes both *which* segments are enabled
  and their *render order*; the default 8-segment layout deliberately mirrors
  the bash statusline's ordering.
- Errors surface to stderr and degrade to defaults rather than panicking or
  aborting the status line.
