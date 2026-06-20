# Codebase Structure

**Analysis Date:** 2026-06-20

## Directory Layout

```
claudebar/
├── src/
│   ├── main.rs              # CLI entry — dispatches all subcommands
│   ├── cli.rs               # clap structs: Cli + Command enum
│   ├── lib.rs               # pub re-exports: Config, InputData, render_line
│   ├── sanitize.rs          # Strip ESC/BEL/CR/LF from host strings
│   ├── model/
│   │   ├── mod.rs           # Re-exports for the model layer
│   │   ├── config.rs        # Config, SegmentKind, Thresholds (TOML)
│   │   ├── input.rs         # InputData, Coerce<T> (JSON from stdin)
│   │   ├── palette.rs       # Color(u8), Theme struct, RESET constant
│   │   └── style.rs         # Style struct (glyphs, icon flag, bar chars)
│   ├── render/
│   │   ├── mod.rs           # render_line(), render_with() — top-level compose
│   │   ├── writer.rs        # SegmentWriter — ANSI buffer with theme-aware API
│   │   └── bar.rs           # make_bar() — progress bar string builder
│   ├── segment/
│   │   ├── mod.rs           # Segment trait, RenderCtx, SegmentKind::as_segment()
│   │   ├── directory.rs     # Fish-style path abbreviation
│   │   ├── git.rs           # Branch, ahead/behind, modified count (git subprocess)
│   │   ├── context.rs       # Context bar + token count
│   │   ├── rate_limits.rs   # 5h + weekly windows with live countdown
│   │   ├── dev_context.rs   # Dev context bar
│   │   └── model.rs         # Model name + effort level
│   ├── styles/
│   │   ├── mod.rs           # styles::get(name) registry + NAMES const
│   │   ├── powerline.rs     # Powerline glyphs (default)
│   │   ├── plain.rs         # Plain pipe separator
│   │   ├── rounded.rs       # Rounded corners
│   │   ├── minimal.rs       # Minimal, no icons
│   │   └── ascii.rs         # ASCII only, no glyphs
│   ├── themes/
│   │   ├── mod.rs           # themes::get(name) registry + NAMES const
│   │   ├── tokyo_night.rs   # Default theme
│   │   ├── catppuccin.rs
│   │   ├── dracula.rs
│   │   ├── gruvbox.rs
│   │   ├── nord.rs
│   │   ├── rose_pine.rs
│   │   ├── ayu_mirage.rs
│   │   ├── cobalt2.rs
│   │   ├── everforest_dark.rs
│   │   ├── github_dark.rs
│   │   ├── kanagawa_wave.rs
│   │   ├── moonfly.rs
│   │   ├── night_owl.rs
│   │   ├── one_dark.rs
│   │   ├── solarized_dark.rs
│   │   └── sonokai.rs
│   └── tui/                 # Feature-gated (feature = "tui")
│       ├── mod.rs           # run() — terminal setup, RAII guard, event loop
│       ├── app.rs           # App state — flat-list cursor, dirty-tracking
│       ├── ui.rs            # draw() — all ratatui widget layout
│       ├── preview.rs       # Live render preview pane
│       └── sample.rs        # Fixture-based sample inputs for preview
├── tests/
│   ├── render_golden.rs     # Integration snapshot tests
│   └── snapshots/           # insta snapshot files
├── fixtures/                # JSON edge-case inputs for manual renders and tests
├── scripts/
│   ├── gen_screenshots.py   # Screenshot generation (Docker + playwright)
│   ├── benchmark.sh
│   └── make_demo_repos.sh
├── screenshots/             # Generated PNG/SVG assets
├── statusline-command.sh    # Bash fallback (zero toolchain, needs jq + git)
├── install.sh               # Installer (cargo install or download bash fallback)
├── Cargo.toml
├── Cargo.lock
└── CLAUDE.md                # Project instructions for Claude Code
```

## Directory Purposes

**`src/model/`:**
- Purpose: All data types — no logic, no I/O
- Contains: `Config`, `SegmentKind`, `Thresholds`, `InputData`, `Coerce<T>`, `Theme`, `Style`
- Key files: `src/model/config.rs`, `src/model/input.rs`, `src/model/palette.rs`

**`src/render/`:**
- Purpose: Compose segments into a single ANSI string; no domain logic
- Contains: `render_line`, `render_with`, `SegmentWriter`, `make_bar`
- Key files: `src/render/mod.rs`, `src/render/writer.rs`

**`src/segment/`:**
- Purpose: One file per status segment; domain logic only
- Contains: `Segment` trait, `RenderCtx`, six segment implementations
- Key files: `src/segment/mod.rs` (trait + dispatch table)

**`src/themes/`:**
- Purpose: Built-in color palettes; one `theme()` function per file returning a `Theme` struct
- Contains: 16 theme files + `mod.rs` registry
- Key files: `src/themes/mod.rs` (name→Theme dispatch), `src/themes/tokyo_night.rs` (default)

**`src/styles/`:**
- Purpose: Built-in glyph sets; one `style()` function per file returning a `Style` struct
- Contains: 5 style files + `mod.rs` registry
- Key files: `src/styles/mod.rs`

**`src/tui/`:**
- Purpose: Interactive TOML configurator; feature-gated, not included in render-only builds
- Contains: Terminal lifecycle, event handler, ratatui widgets, preview, sample data
- Key files: `src/tui/mod.rs` (entry), `src/tui/app.rs` (state), `src/tui/ui.rs` (layout)

**`fixtures/`:**
- Purpose: JSON edge-case inputs for manual smoke testing and snapshot test data
- Contains: `typical.json`, `over_limit_5h.json`, `injection.json`, etc.
- Generated: No — hand-authored
- Committed: Yes

**`tests/`:**
- Purpose: Integration-level golden tests using `insta` snapshots
- Contains: `render_golden.rs` + `snapshots/` directory
- Key files: `tests/render_golden.rs`

## Key File Locations

**Entry Points:**
- `src/main.rs`: Binary entry; all subcommand dispatch lives here
- `src/lib.rs`: Library crate root; re-exports `Config`, `InputData`, `render_line`

**Core Render Path:**
- `src/render/mod.rs`: `render_line` and `render_with` — start here for render logic
- `src/segment/mod.rs`: `Segment` trait and `SegmentKind::as_segment()` dispatch

**Configuration:**
- `src/model/config.rs`: `Config`, `SegmentKind`, `Thresholds` — the TOML schema
- `Cargo.toml`: Features (`tui` = default), dependencies

**Testing:**
- `tests/render_golden.rs`: Snapshot integration tests
- `tests/snapshots/`: insta `.snap` files

## Naming Conventions

**Files:**
- Modules use `snake_case.rs`
- Theme files mirror the theme's kebab-case name with underscores: `tokyo_night.rs` → `"tokyo-night"`
- Style files likewise: `powerline.rs` → `"powerline"`

**Directories:**
- All lowercase, no separators: `model/`, `render/`, `segment/`, `styles/`, `themes/`, `tui/`

**Types:**
- Structs: `PascalCase` (`InputData`, `SegmentWriter`, `RenderCtx`)
- Enums: `PascalCase` variants (`SegmentKind::RateLimits`)
- TOML/CLI names: `kebab-case` (`rate-limits`, `tokyo-night`)
- Constants: `SCREAMING_SNAKE_CASE` (`RESET`, `NAMES`)

## Where to Add New Code

**New segment:**
1. Create `src/segment/<name>.rs` — implement `pub struct Name;` and `impl Segment for Name`
2. Add `pub mod <name>;` in `src/segment/mod.rs`
3. Add variant to `SegmentKind` enum in `src/model/config.rs` (include in `ALL` array at canonical position)
4. Add arm to `SegmentKind::as_segment()` match in `src/segment/mod.rs`
5. Add arm to `SegmentKind::label()` match in `src/model/config.rs`
6. Add test fixture JSON if the segment reads new input fields

**New theme:**
1. Create `src/themes/<name>.rs` — `pub fn theme() -> Theme { Theme { ... } }` filling all `Theme` slots
2. Add `pub mod <name>;` to `src/themes/mod.rs`
3. Add the kebab-case name to `NAMES` const and a match arm in `get()` in `src/themes/mod.rs`

**New style:**
1. Create `src/styles/<name>.rs` — `pub fn style() -> Style { Style { ... } }`
2. Add `pub mod <name>;` to `src/styles/mod.rs`
3. Add name to `NAMES` and match arm in `get()` in `src/styles/mod.rs`

**New config field:**
1. Add field to `Config` or `Thresholds` in `src/model/config.rs` with `#[serde(default)]`
2. Pass through `RenderCtx` in `src/segment/mod.rs` if segments need it
3. Add TUI row in `src/tui/app.rs` (`RowItem`) and `src/tui/ui.rs` (`draw`)

**New input JSON field:**
1. Add to the appropriate struct in `src/model/input.rs`, using `Coerce<T>` for numeric fields
2. Access in segment code via `ctx.input.<field>`

**Tests:**
- Unit tests: inline `#[cfg(test)]` module in the relevant source file
- Snapshot tests: add cases to `tests/render_golden.rs`, run `cargo insta review` to accept
- Fixtures: add JSON to `fixtures/` for manual smoke testing

## Special Directories

**`tests/snapshots/`:**
- Purpose: insta snapshot files for golden render tests
- Generated: Yes (by `cargo insta review`)
- Committed: Yes — snapshots are the source of truth

**`.planning/codebase/`:**
- Purpose: GSD codebase map documents consumed by planning/execution agents
- Generated: Yes (by `/gsd-map-codebase`)
- Committed: No (gitignored or ephemeral)

**`screenshots/`:**
- Purpose: PNG/SVG demo assets for README
- Generated: Yes (by `scripts/gen_screenshots.py`)
- Committed: Yes (binary assets)

---

*Structure analysis: 2026-06-20*
