# claudebar — OpenWiki quickstart

> Status: first-pass documentation. Grounded in source inspection; regenerate
> after significant changes (`openwiki-update` or skill re-run).

claudebar is a **Powerline-style status line for Claude Code** — a single native
Rust binary that reads Claude Code's session JSON from stdin and emits a
themed, styled ANSI status line on stdout. It ships with a TUI configurator
(`claudebar config`), built-in themes and styles, a bash fallback for
zero-toolchain environments, and an installer with supply-chain verification.

## What it does

- **Hook integration.** Claude Code's `statusLine.command` invokes
  `claudebar render`, which parses `--hook --stdin` JSON and prints one
  (or more) lines. The contract never changes: stdin JSON → stdout ANSI.
- **Segments.** 11 renderable segments — directory, git, context usage,
  rate limits, model/effort, dev context, cost, lines, duration, clock, burn.
- **Themes & styles.** Fixed-slot 256-color themes (16) and glyph styles (7);
  resolved by name, fallback to `tokyo-night` / `powerline`.
- **TUI configurator** (`tui` feature): interactive theme/style/segment setup
  with a **live preview** that reuses the exact render path (`render_with`).
- **Bash fallback** (`statusline-command.sh`): jq+git implementation covering
  a subset of segments, with distinct (non-identical) rendering surface.

## Repository layout

| Path | Purpose |
|---|---|
| `src/main.rs`, `src/cli.rs` | CLI dispatch, stdin read, subcommands |
| `src/model/` | `InputData` (hook JSON), `Config` (TOML), `Theme`, `Style`, `Thresholds`, `Coerce<T>` |
| `src/render/` | `render_line`/`render_with`, `SegmentWriter`, bar, width, float (ANSI emission) |
| `src/segment/` | `Segment` trait + 11 segment implementations |
| `src/themes/`, `src/styles/` | Name → value registries (one file per theme/style) |
| `src/tui/` | TUI configurator (feature-gated) |
| `src/setup.rs` | `setup`/`sync`/`doctor`/`edit` — Claude Code `settings.json` wiring |
| `statusline-command.sh` | Bash fallback |
| `install.sh`, `scripts/` | Installer, docs/screenshot generation, benchmark |
| `tests/` | Rust integration + insta golden snapshots + bats suites |
| `.github/workflows/` | CI, security, release, benchmark, openwiki self-maintenance |

## Read next

- [Architecture overview](architecture/overview.md) — module graph, render path invariant
- [Input contract](concepts/input-contract.md) — the hook JSON and `Coerce<T>` degradation
- [Config reference](reference/config.md) — every TOML key, defaults, load/save semantics
- [CLI reference](reference/cli-commands.md) — subcommands, flags, exit codes
- [Render pipeline](systems/render-pipeline.md) — fixed vs auto layout, separator rules, writer
- [Segments](systems/segments/README.md) — per-segment contracts
- [TUI configurator](systems/tui-configurator.md) — interactive setup, keymap
- [Bash fallback](systems/bash-fallback.md) — divergence-aware contract vs the Rust binary
- [Cross-session state](systems/cross-session-state.md) — `limit_sync` + `burn` caches
- [Themes & styles](reference/themes-styles.md) — color slots, glyph sets
- [Installation](systems/installation.md) — channels, checksums, attestation
- [Release pipeline](reference/release-pipeline.md) — CalVer model, Homebrew tap
- [Testing](testing/overview.md) — pyramid, golden snapshots, bats

## Task routing

| Intent | Start at | Key symbols | Validation |
|---|---|---|---|
| Change a segment's output | `systems/segments/` page + `concepts/segment-seam.md` | `Segment::render`, `SegmentWriter` | `cargo test <segment>` |
| Add a new segment | `concepts/segment-seam.md` (extension recipe) | `SegmentKind`, `as_segment` | new unit tests + golden |
| Modify the hook render | `architecture/data-flow` + `systems/render-pipeline.md` | `render_line`, `render_with`, `InputData::parse` | `cargo test render` |
| Tweak a theme/style | `reference/themes-styles.md` | `themes::get`, `styles::get` | `cargo test themes styles` |
| Change TUI behavior | `systems/tui-configurator.md` | `App`, `ui::draw`, `preview` | `cargo test -F tui app` |
| Edit install/release | `systems/installation.md`, `reference/release-pipeline.md` | `install.sh`, workflows | `bats tests/install.bats` |
| Update the wiki | this file + skill | — | `openwiki code --update --print` |

## Conventions to respect

- **Single render path** — never add a second ANSI composition path; TUI
  preview and hook must share `render_with`.
- **No new runtime dependencies** for the core binary; render path must build
  with `--no-default-features`.
- Segments must be **deterministic & testable** — inject `now`/`home` via
  `RenderCtx`, never read env/time directly.
- Errors surface to stderr and degrade to defaults — `InputData::parse` is
  infallible, `render_with` never panics.