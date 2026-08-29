---
type: "Reference"
title: "Architecture overview"
description: "Layered module graph and invariants of the claudebar binary: model → render → segment → themes/styles, the single-render-path invariant, render-composition sub-layers (writer/bar/width/float), auto-layout, and tui feature gating."
tags: [architecture, render, segments, themes, feature-gating]
verified:
  - by: openwiki/0.4.0
    at: 2026-08-29T00:17:43.706Z
sources:
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
  - id: openwiki-source-017711f48cc9b66315d5ce67
    resource: repo://src/model/mod.rs
  - id: openwiki-source-f8dfbb6a10fc50e65133b3f7
    resource: repo://src/render/bar.rs
  - id: openwiki-source-763738302a84ffdcefbc9913
    resource: repo://src/render/float.rs
  - id: openwiki-source-1d33473d874a4090bb6026e0
    resource: repo://src/render/mod.rs
  - id: openwiki-source-ae1cea23e3748175b7a74fdf
    resource: repo://src/render/width.rs
  - id: openwiki-source-d977bd28254dbfcf5d7fe3bb
    resource: repo://src/render/writer.rs
  - id: openwiki-source-0700af36d25875a20e6db044
    resource: repo://src/segment/git.rs
  - id: openwiki-source-f4acb0adeb73aa0d54303a27
    resource: repo://src/segment/mod.rs
  - id: openwiki-source-5d948000f74b098b1187bdc9
    resource: repo://src/segment/update_notice.rs
  - id: openwiki-source-c5d0aaf913f55a00eb2e5796
    resource: repo://src/styles/mod.rs
  - id: openwiki-source-079dcbc1bf8b946f21372942
    resource: repo://src/themes/mod.rs
  - id: openwiki-source-f1010f412e64365e722e378c
    resource: repo://src/tui/preview.rs
  - id: openwiki-source-0ecba5538b5fd9860f10332f
    resource: repo://src/update.rs
  - id: openwiki-source-d6e43e19ed4d1ddc97fba7dc
    resource: repo://tests/render_golden.rs
generated: {by: "openwiki/0.4.0", at: "2026-08-29T00:17:43.706Z"}
---

# Architecture overview

## Layout model

claudebar is a single binary with a layered module graph:

```text
main.rs / cli.rs          argument parse, stdin read, now() injection
        │
        ▼
model/                    InputData, Config, SegmentKind, Thresholds,
                          Theme, Style, Coerce<T>
        │
        ▼
render/                   render_line → render_with → RenderCtx
   │   ├─ writer.rs       SegmentWriter — the single place segments emit
   │   │                  colored, glyph-decorated text
   │   ├─ bar.rs          write_bar / write_bar_dots progress bars
   │   ├─ width.rs        visible_width (ANSI-stripping column count)
   │   └─ float.rs        best-effort plain-text side file (emit_float)
   ▼
segment/                  Segment trait; 12 implementations in SegmentKind::ALL
   │                      (dependency-injected context, no env reads)
   ▼
themes/ styles/           name → value registries (16 themes, 8 styles)
```

The TUI (`src/tui/`, feature-gated) depends on the same `model` and `render`
layers: the preview calls `render_with` with a fixture sample, so the live
preview can never diverge from the hook output.

## Render-composition sub-layers

`render/mod.rs` declares four composition helpers that segments build on:

- **`writer.rs`** — [`SegmentWriter`](repo://src/render/writer.rs) is the single
  place a segment emits colored, glyph-decorated text. A segment never embeds a
  raw ANSI code, never hardcodes a theme color, and never decides whether icons
  render; the active theme × style does the rest. Nested `colored_with` spans
  restore the innermost open color instead of leaving trailing text at the
  terminal default.
- **`bar.rs`** — `write_bar` appends a self-colored `width`-cell bar for `pct`
  percent, and `write_bar_dots` a quarter-resolution variant using a 5-level
  `levels` array; both clamp over-limit percentages and guarantee at least one
  filled cell when `pct > 0`.
- **`width.rs`** — `visible_width(s)` strips ANSI SGR escapes via a small state
  machine, then counts terminal columns: 1 per codepoint, 2 for wide CJK /
  kana / Hangul / emoji, 0 for combining marks. Pure Rust, no dependency
  tables; it drives the responsive auto-layout path.
- **`float.rs`** — `emit_float` writes a one-line ANSI-free summary of the
  selected segments to a file (e.g. for tmux / menu-bar piping). It is a
  best-effort side effect of the status render: any I/O error is silently
  swallowed so a float failure can never break the status line.

## Single-render-path invariant

`render_line(input, cfg, now)` is the **only** entrypoint for the hook output.
It resolves the configured theme/style/`$HOME`, reads any cached update
version, and delegates to `render_ctx`, which builds a `RenderCtx` and
dispatches to one of two compositors. The TUI preview
([`src/tui/preview.rs`](repo://src/tui/preview.rs)) calls `render_with` directly
with already-resolved theme/style and fixed `now` (`FIXED_NOW`) and home
(`PREVIEW_HOME`) — there is no second composition path. This invariant is
enforced by construction: segment implementations are pure functions over
`RenderCtx`.

`render_line` also triggers the float readout side effect when
`cfg.thresholds.float` is set; `render_with` (used by the TUI and tests) does
not — it only builds the status line.

## Segments

Each segment is a zero-sized struct implementing the `Segment` trait:

```rust
fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool
```

- Returns `true` if it emitted anything, `false` to skip (e.g. no git repo,
  zero cost, no dev context).
- Segments never know about their neighbors; separators are inserted by the
  composition layer only between two non-empty segments.
- All ambient state (`now`, `home`, `tz_offset_seconds`, cached `update`
  version) is injected via `RenderCtx` — segments never call `std::env::var` or
  system time, and the update-notice cache read happens once in the render
  layer, never inside a segment.
- `SegmentKind::ALL` now enumerates **12** segments (Directory, Git, Model,
  Context, RateLimits, DevContext, Cost, Lines, Duration, Burn, Clock,
  UpdateNotice); `DEFAULT` is the 8-segment layout that mirrors the bash
  statusline.

## Layout: fixed vs auto

`render_ctx` selects the compositor from `cfg.thresholds.layout`:

- **fixed** — `render_fixed` writes every enabled segment in order onto a single
  line, inserting separators between non-empty segments and skipping empty ones
  (the default).
- **auto** — `render_auto` is the responsive layout: it renders each segment
  into its own `String`, drops empty ones, then greedily packs segments onto up
  to `max_lines` lines using `visible_width` so each line stays within the
  terminal width minus `wrap_margin`. Terminal width comes from `$COLUMNS`, then
  `stty size`, else 0 (unknown terminal), which disables wrapping and falls
  back to a single line.

## Feature gating

The `tui` feature (default-on) adds `ratatui`/`crossterm`/`ansi-to-tui` and the
whole `src/tui/` module (`lib.rs:48-49`). These crates are optional
dependencies and are never linked on the render hot path. `cargo build
--no-default-features` produces a **render-only hook** with no TUI dependency —
the render/float/hook path must stay TUI-free. `claudebar config` without the
feature exits 1 with a message pointing at the TOML config. This is declared in
`Cargo.toml` (`features`/optional deps) and enforced by CI and the Taskfile
`build-minimal` target.

## Data flow (render subcommand)

<!-- openwiki: mermaid below restored after rephrasing the Coerce<T> label that previously failed to parse. -->
```mermaid
sequenceDiagram
    participant CC as Claude Code
    participant CB as claudebar
    CC->>CB: --hook --stdin JSON (session state)
    CB->>CB: InputData::parse (infallible, Coerce type-coercion)
    CB->>CB: Config::load (TOML, defaults on missing)
    CB->>CB: render_line: resolve theme/style, inject now/home
    CB->>CB: render_ctx: build RenderCtx, fixed or auto layout
    CB->>CB: iterate segments via SegmentWriter
    CB->>CC: ANSI status line on stdout
```

A side effect: `render_line` calls `float::emit_float` when `thresholds.float`
is enabled (best-effort plain-text file; see
[Cross-session state](../systems/cross-session-state.md)).

## Key architectural constraints

- **Threading:** none — single-threaded render and TUI event loop. The only
  concurrency is the background update check, which spawns a detached child and
  never blocks the render.
- **Global state:** none — theme/style registries return owned values; `App` in
  the TUI holds all mutable configurator state.
- **Infallible render:** `InputData::parse` never fails; wrong-typed fields
  degrade to `None`; `render_with` never panics.
- **External I/O:** `src/segment/git.rs` is the only non-stdin external I/O in
  the render path (spawns `git`); the update-notice check spawns a detached
  child (`update --check`) at most once a day.
- **Deterministic tests:** `now`/`home`/`tz_offset` injection makes every
  render reproducible; golden snapshots (`tests/render_golden.rs`) pin exact
  output across the theme × style matrix.

## What to watch out for

- Do not add a second render path; extend `render_with` / `render_ctx` and the
  `Segment` trait.
- Keep the render/float/hook path free of `tui`-feature dependencies.
- When adding a segment: enum variant + label + `as_segment` wiring + module +
  writer conventions + unit tests + golden coverage (see
  `concepts/segment-seam.md`).
