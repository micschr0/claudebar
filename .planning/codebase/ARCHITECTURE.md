<!-- refreshed: 2026-06-20 -->
# Architecture

**Analysis Date:** 2026-06-20

## System Overview

```text
┌─────────────────────────────────────────────────────────────────────┐
│                        CLI Entry Point                              │
│                        `src/main.rs`                                │
│   Render  │  Config (TUI)  │  Init  │  List  │  Migrate            │
└─────┬─────┴───────┬────────┴────────┴────────┴─────────────────────┘
      │             │
      ▼             ▼
┌──────────────┐  ┌──────────────────────────────────────────────────┐
│ InputData    │  │  TUI Configurator (feature = "tui")              │
│ `src/model/  │  │  `src/tui/`                                      │
│  input.rs`   │  │  App state + event loop + ratatui draw           │
└──────┬───────┘  └──────────────────┬───────────────────────────────┘
       │                             │
       │  + Config                   │ also calls render_with()
       ▼                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Render Composition Layer                          │
│                    `src/render/mod.rs`                              │
│   render_line() / render_with() — iterates Config.segments,         │
│   dispatches each SegmentKind → Segment::render()                   │
└───────────────────────┬─────────────────────────────────────────────┘
                        │  per-segment
                        ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Segment Implementations                           │
│                    `src/segment/`                                   │
│   Directory · Git · Context · RateLimits · DevContext · Model       │
│   each writes to SegmentWriter; returns bool (emitted anything?)    │
└───────────────────────┬─────────────────────────────────────────────┘
                        │
                        ▼
┌───────────────────────────────────────────────────────────────────┐
│   SegmentWriter  `src/render/writer.rs`   +  make_bar()           │
│   `src/render/bar.rs`                                             │
│   Encapsulates ANSI color emission; segments never embed raw codes │
└───────────────────────────────────────────────────────────────────┘
                        │ reads from
                        ▼
┌──────────────────────────────────────────────────────────────────┐
│   Theme struct  `src/model/palette.rs`                           │
│   Style struct  `src/model/style.rs`                             │
│   Resolved by:  `src/themes/mod.rs`  /  `src/styles/mod.rs`     │
└──────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| `main.rs` | CLI dispatch, stdin read, now() injection | `src/main.rs` |
| `cli.rs` | clap structs: `Cli`, `Command` | `src/cli.rs` |
| `lib.rs` | Public re-exports for render path | `src/lib.rs` |
| `InputData` | JSON deserialization from Claude Code stdin, infallible parse | `src/model/input.rs` |
| `Config` | TOML-persisted user settings (segments list, theme, style, thresholds) | `src/model/config.rs` |
| `Theme` | 256-color ANSI slots for each semantic role | `src/model/palette.rs` |
| `Style` | Glyph characters, icon flag, bar fill/empty chars, separator | `src/model/style.rs` |
| `render_line` / `render_with` | Compose segments into one ANSI string | `src/render/mod.rs` |
| `SegmentWriter` | Buffer ANSI output; expose `colored`, `dim`, `icon`, `bar` API | `src/render/writer.rs` |
| `make_bar` | Build a progress bar string from pct, width, and colors | `src/render/bar.rs` |
| `Segment` trait | Contract each segment implements: `render(&RenderCtx, &mut SegmentWriter) -> bool` | `src/segment/mod.rs` |
| Segment impls | Domain logic per segment; pure functions, no env reads | `src/segment/*.rs` |
| `themes::get` | Resolve theme name → `Theme` struct | `src/themes/mod.rs` |
| `styles::get` | Resolve style name → `Style` struct | `src/styles/mod.rs` |
| `sanitize` | Strip ESC/BEL/CR/LF from host strings before emission | `src/sanitize.rs` |
| TUI `run()` | Terminal setup (crossterm), RAII guard, event loop | `src/tui/mod.rs` |
| `App` | Flat-list cursor state, dirty-tracking, save/reset | `src/tui/app.rs` |
| `ui::draw` | ratatui widget layout | `src/tui/ui.rs` |
| `preview` | Live render preview in TUI using `render_with()` | `src/tui/preview.rs` |
| `sample` | Fixture-based sample `InputData` for TUI preview | `src/tui/sample.rs` |

## Pattern Overview

**Overall:** Pipeline / Compositor

**Key Characteristics:**
- A single shared render path (`render_with`) is used by both the CLI hook and the TUI live preview — no divergence possible.
- Segments are stateless zero-sized types implementing `Segment`; `SegmentKind::as_segment()` returns `&'static dyn Segment`.
- All ambient state (current time, `$HOME`) is injected into `RenderCtx`, making rendering deterministic and testable.
- `InputData::parse` is infallible: any JSON shape (including invalid JSON) degrades to `InputData::default()` via `Coerce<T>`.
- Theme and Style are value types (structs, `Copy`), resolved by name in registry match arms.

## Layers

**CLI / IO Layer:**
- Purpose: Parse args, read stdin, inject `now`, dispatch to subcommands
- Location: `src/main.rs`, `src/cli.rs`
- Contains: `clap` parsing, stdin read, `SystemTime` call, `Config::load`
- Depends on: model, render, tui
- Used by: nothing (binary entry point)

**Model Layer:**
- Purpose: Data types — input, config, colors, styles
- Location: `src/model/`
- Contains: `InputData`, `Config`, `SegmentKind`, `Thresholds`, `Theme`, `Style`, `Coerce<T>`
- Depends on: serde, toml
- Used by: all other layers

**Render Layer:**
- Purpose: Compose segments into ANSI output
- Location: `src/render/`
- Contains: `render_line`, `render_with`, `SegmentWriter`, `make_bar`
- Depends on: model, segment
- Used by: main.rs (render subcommand), tui/preview.rs

**Segment Layer:**
- Purpose: Domain logic for each status segment
- Location: `src/segment/`
- Contains: `Segment` trait, `RenderCtx`, six segment structs
- Depends on: model, render/writer (via SegmentWriter), sanitize, std::process (git subprocess)
- Used by: render layer

**Theme / Style Registries:**
- Purpose: Name-to-value resolution for built-in themes and styles
- Location: `src/themes/`, `src/styles/`
- Contains: one file per theme/style, `mod.rs` with `get()` and `NAMES`
- Depends on: model (Theme, Style types)
- Used by: render layer, TUI

**TUI Layer (feature-gated):**
- Purpose: Interactive configurator, only compiled with `tui` feature
- Location: `src/tui/`
- Contains: terminal setup, event loop, widget layout, preview, sample fixtures
- Depends on: ratatui, crossterm, model, render
- Used by: main.rs (config subcommand)

## Data Flow

### Primary Request Path (render subcommand)

1. Claude Code writes session JSON to stdin; `main.rs` reads it to a `String` (`src/main.rs:53–56`)
2. `InputData::parse(&buf)` deserializes, degrading bad fields to `None` (`src/model/input.rs:114`)
3. `resolve_config(cli)` loads TOML config; CLI flags override theme/style (`src/main.rs:25–50`)
4. `SystemTime::now()` captured as `i64` epoch seconds — only ambient state captured at this point (`src/main.rs:59–62`)
5. `render_line(&input, &cfg, now)` resolves theme and style by name, calls `render_with` (`src/render/mod.rs:18–23`)
6. `render_with` builds `RenderCtx`, iterates `cfg.segments`, dispatches each to `kind.as_segment().render(&ctx, &mut w)` (`src/render/mod.rs:28–59`)
7. Each segment writes to its own `SegmentWriter`; returns `true` if non-empty
8. Non-empty segments are joined with `separator()` into the output `String`
9. `println!` emits the ANSI line to stdout (`src/main.rs:63`)

### TUI Preview Path

1. `tui/preview.rs` calls `render_with(sample_input, cfg, theme, style, now, home)` — same function as render subcommand
2. Output is displayed in a ratatui paragraph widget

**State Management:**
- No global mutable state. `App` in the TUI holds all mutable state for the configurator session.
- Config is loaded from disk at startup; saved explicitly on `s` keypress.
- Segment implementations are zero-sized; all data flows through `RenderCtx`.

## Key Abstractions

**`Segment` trait:**
- Purpose: Uniform interface for all status bar segments
- Examples: `src/segment/directory.rs`, `src/segment/git.rs`, `src/segment/rate_limits.rs`
- Pattern: `fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool`

**`SegmentWriter`:**
- Purpose: Encapsulate ANSI color emission so segments never embed raw escape codes
- File: `src/render/writer.rs`
- Pattern: Methods `colored`, `dim`, `icon`, `bar`, `raw` — theme and style resolved internally

**`Coerce<T>`:**
- Purpose: Forgiving JSON number deserializer; wrong-typed or absent fields degrade to `None`
- File: `src/model/input.rs`
- Pattern: Custom `Deserialize` impl that maps bool/seq/map/garbage → `Coerce(None)`

**`RenderCtx`:**
- Purpose: Dependency injection bundle passed to every segment
- File: `src/segment/mod.rs`
- Pattern: `&InputData`, `&Theme`, `&Style`, `&Thresholds`, `now: i64`, `home: Option<&str>`

**Theme / Style value types:**
- Purpose: Named color slot structs and glyph character structs; resolved by `get(name)`
- Files: `src/model/palette.rs`, `src/model/style.rs`, `src/themes/mod.rs`, `src/styles/mod.rs`
- Pattern: Fixed struct fields = compile error if a theme omits a color

## Entry Points

**`render_line` (hook):**
- Location: `src/render/mod.rs:18`
- Triggers: `claudebar` / `claudebar render` invoked by Claude Code status line hook
- Responsibilities: Resolve theme/style, inject `$HOME`, delegate to `render_with`

**`render_with` (shared):**
- Location: `src/render/mod.rs:28`
- Triggers: Called by `render_line` and by TUI preview
- Responsibilities: Build `RenderCtx`, iterate segments, compose ANSI output string

**`tui::run` (configurator):**
- Location: `src/tui/mod.rs:50`
- Triggers: `claudebar config` (requires `tui` feature)
- Responsibilities: Load config, enter alternate screen, run event loop, save on request

## Architectural Constraints

- **Threading:** Single-threaded. The render path is entirely synchronous. The TUI event loop polls crossterm events with a 200ms timeout; no background threads.
- **Global state:** None. `themes::get` and `styles::get` return owned values. No `lazy_static` or `once_cell` singletons.
- **Feature gate:** `ratatui` and `crossterm` are only compiled when `features = ["tui"]` (default). A `--no-default-features` build produces a render-only binary with no TUI dependency.
- **Infallible render:** `InputData::parse` never returns `Err`; `render_with` never panics; `println!` is the only I/O that can fail.
- **Git subprocess:** `src/segment/git.rs` spawns a `git` child process. This is the only non-stdin external I/O in the render path.

## Anti-Patterns

### Segments reading ambient environment

**What happens:** Segments must not call `std::env::var("HOME")` or `SystemTime::now()` directly.
**Why it's wrong:** Makes segments non-deterministic and untestable.
**Do this instead:** Read `ctx.home` and `ctx.now` from `RenderCtx` (`src/segment/mod.rs:21–30`).

### Raw ANSI codes in segments

**What happens:** A segment writing `\x1b[38;5;33m` directly into a string buffer.
**Why it's wrong:** Bypasses theme and style; colors are hardcoded and not overridable.
**Do this instead:** Use `SegmentWriter` methods (`colored`, `dim`, `icon`, `bar`) which resolve colors through the theme (`src/render/writer.rs`).

### Second render code path for TUI preview

**What happens:** TUI preview duplicating render logic instead of calling `render_with`.
**Why it's wrong:** Preview can diverge from the actual hook output.
**Do this instead:** Always call `render_with` from `src/tui/preview.rs` — this is already enforced.

## Error Handling

**Strategy:** Degradation over failure. The render path never returns an error; all failure modes produce partial or empty output.

**Patterns:**
- `InputData::parse` returns `InputData::default()` on any JSON failure
- `Coerce<T>` degrades wrong-typed fields to `None` without propagating errors
- `Config::load` returns `Config::default()` for missing files; surfaces `ConfigError` for malformed TOML
- `main.rs` prints warnings to stderr on config parse failure and continues with defaults
- Segment `render()` returns `false` to skip a segment that has no data to show

## Cross-Cutting Concerns

**Logging:** None. Warnings go to `stderr` via `eprintln!` in `main.rs` only.
**Validation:** Input is validated by `Coerce<T>` at deserialization; `used_percentage` is clamped to `<= 999` in segment code to exclude timestamp leakage.
**Authentication:** None. claudebar is a local CLI tool with no network calls (except the `git` subprocess for branch info).

---

*Architecture analysis: 2026-06-20*
