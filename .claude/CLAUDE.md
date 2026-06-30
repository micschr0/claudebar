<!-- GSD:project-start source:PROJECT.md -->

## Project

**claudebar**

claudebar is a Rust-based statusline renderer for Claude Code — it reads session JSON from stdin and writes a themed, styled ANSI status line to stdout. It ships with a TUI configurator (`claudebar config`) for interactive theme/style/segment setup, and a bash fallback for zero-toolchain environments.

**Core Value:** Users can install claudebar in one command and get a polished, customizable statusline that just works — and new users discover it via clean documentation with real screenshots.

### Constraints

- **Tech stack**: Rust 2021, Cargo — no new runtime dependencies for the core binary
- **Binary size**: release build uses LTO + strip; keep it lean
- **Backward compat**: `claudebar render` stdin/stdout contract must not change
- **CI targets**: at minimum `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`
- **Git auth**: if `git push` fails with auth error, run `gh auth setup-git` — HTTPS remote uses GH_TOKEN

<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->

## Technology Stack

## Languages

- Rust 2021 edition — all application code in `src/`
- Bash — `statusline-command.sh` (zero-toolchain fallback), `install.sh`, `scripts/benchmark.sh`, `scripts/make_demo_repos.sh`
- Python 3 — `scripts/gen_screenshots.py` (screenshot generation only, not part of the binary)

## Runtime

- Native binary — no runtime VM
- Rust 1.95.0 (pinned in `.agents/skills/rust-skills/checks/rust-toolchain.toml` for CI harness)
- System Rust 1.96.0 available locally
- Cargo 1.96.0
- Lockfile: `Cargo.lock` present and committed

## Frameworks

- `clap` 4 (derive feature) — CLI argument parsing (`src/cli.rs`)
- `ratatui` 0.29 — TUI widget layout (`src/tui/`) — optional, `tui` feature only
- `crossterm` 0.28 — terminal control / raw mode (`src/tui/`) — optional, `tui` feature only
- `ansi-to-tui` 7 — render ANSI preview inside ratatui (`src/tui/preview.rs`) — optional, `tui` feature only
- `insta` 1 (filters + glob features) — snapshot testing (`tests/render_golden.rs`, `tests/snapshots/`)
- Task 3 (`Taskfile.yml`) — task runner wrapping cargo commands
- `shellcheck` — bash static analysis (lint task)

## Key Dependencies

- `serde` 1 + derive — serialization/deserialization foundation for all JSON and TOML I/O
- `serde_json` 1 — parses Claude Code session JSON from stdin (`src/model/input.rs`)
- `toml` 0.8 — reads/writes user config at `$XDG_CONFIG_HOME/claudebar/config.toml`
- `thiserror` 1 — typed error definitions across the codebase
- `clap` 4 — subcommand dispatch: `render`, `config`, `init`, `list` (`src/main.rs`)

## Features

- Enables `ratatui`, `crossterm`, `ansi-to-tui`
- Includes `src/tui/` module
- Required for `claudebar config` subcommand
- Strips all TUI deps
- Produces a minimal binary for the Claude Code hook hot path
- Zero dependency on crossterm/ratatui

## Configuration

- TOML at `$XDG_CONFIG_HOME/claudebar/config.toml` (fallback: `~/.config/claudebar/config.toml`)
- Managed by `src/model/config.rs`
- Missing file falls back to `Config::default()`
- `Cargo.toml` — release profile: `opt-level=3`, `lto=true`, `strip=true`, `codegen-units=1`
- `Taskfile.yml` — task aliases for build, test, lint, render, screenshots

## Platform Requirements

- Rust toolchain (1.95+ pinned)
- `jq` and `git` on PATH (runtime deps for the bash fallback and `git` segment)
- Nerd Font in terminal for powerline glyphs
- Single statically-linked binary (Linux x86_64 primary target)
- No runtime dependencies for the Rust binary beyond a POSIX shell environment
- Bash fallback (`statusline-command.sh`) requires `jq` and `git`

<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->

## Conventions

## Naming Patterns

- Modules use `snake_case`: `rate_limits.rs`, `dev_context.rs`, `tokyo_night.rs`
- No barrel index files — each module is a named file
- Theme files named after the theme with underscores: `rose_pine.rs`, `tokyo_night.rs`
- `snake_case` for all free functions: `make_bar()`, `render_line()`, `parse_status()`, `fmt_tokens()`, `fmt_reset()`
- Constructor-style methods named `new()` or `theme()` (for theme modules)
- Getter methods named descriptively: `as_str()`, `as_deref()`, `or_default()`, `get()`
- Boolean-returning methods named `is_*`: `is_empty()`, `is_dirty()`, `is_some()`
- Pure helpers prefixed with verb: `strip_control()`, `abbreviate_path()`, `fmt_scaled()`
- `PascalCase` for all structs, enums, traits: `InputData`, `SegmentKind`, `RenderCtx`, `Coerce<T>`
- Enums use `PascalCase` variants: `SegmentKind::RateLimits`, `StatusKind::Success`
- Traits named as nouns or roles: `Segment`, `Style`, `FromJsonNumber`
- `SCREAMING_SNAKE_CASE`: `RESET`, `FIXED_NOW`, `ALL`
- `SegmentKind` uses `#[serde(rename_all = "kebab-case")]` for TOML: `rate-limits`, not `rate_limits`
- Structs use `#[serde(default)]` on all fields — partial input always accepted

## Module Structure

- Each module starts with a `//!` doc comment describing its purpose and contract
- Complex contracts (e.g., `rate_limits.rs`, `git.rs`) include inline pseudocode/spec in the module doc
- Private helpers defined in the same file as the type that uses them — no shared utilities module except `sanitize.rs`

## Code Style

- `cargo fmt` standard — no `.rustfmt.toml` overrides detected
- Trailing comments inline with color constants: `// Blue #89b4fa`
- `cargo clippy -- -D warnings` is the lint gate (no `.clippy.toml`)
- `#[must_use]` on functions where ignoring the result is a likely bug: `Config::load()`, `Config::save()`, `render_line()`, `render_with()`
- `#[must_use = "..."]` with explanatory text when the reason is non-obvious

## Import Organization

- None — all paths spelled out fully

## Error Handling

- `thiserror::Error` derive for `ConfigError` with `#[error("...")]` messages
- Variants: `ConfigError::Io(String)` and `ConfigError::Parse(String)` — errors are strings, not boxed sources
- No `anyhow` — typed errors throughout

## Logging

- Errors surfaced to the user via stderr in `main.rs`; no structured logging

## Comments

- Module-level `//!` doc comments are mandatory on every module
- Complex contracts (exact output format, edge cases) documented inline in the `//!` block, sometimes with pseudocode
- Inline `//` comments on color constant lines mapping hex colors to xterm-256 indices
- Non-obvious guards explained: `// At least one filled cell once there's any usage`
- No obvious/redundant comments
- All `pub` functions, types, and constants carry `///` doc comments
- `/// # Errors` section on functions returning `Result`
- `pub(crate)` items use `///` only when the usage isn't obvious from the name

## Function Design

- `bool` from `Segment::render()` to signal "emitted anything" — no output in return value
- `Option<T>` for values that may be absent (`fmt_reset` returns `Option<String>`)
- `String` for owned output strings

## Feature Gating

- All ratatui/crossterm TUI code behind `#[cfg(feature = "tui")]`
- Render path must compile without the `tui` feature (`--no-default-features`)
- Feature-gated deps in `Cargo.toml` with `optional = true`

## Dependency Injection Pattern

- `RenderCtx` injects `now` (epoch seconds) and `home` path so segments are deterministic and testable without environment reads
- Segments never call `std::env::var` or system time directly

<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->

## Architecture

## System Overview

```text

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

- A single shared render path (`render_with`) is used by both the CLI hook and the TUI live preview — no divergence possible.
- Segments are stateless zero-sized types implementing `Segment`; `SegmentKind::as_segment()` returns `&'static dyn Segment`.
- All ambient state (current time, `$HOME`) is injected into `RenderCtx`, making rendering deterministic and testable.
- `InputData::parse` is infallible: any JSON shape (including invalid JSON) degrades to `InputData::default()` via `Coerce<T>`.
- Theme and Style are value types (structs, `Copy`), resolved by name in registry match arms.

## Layers

- Purpose: Parse args, read stdin, inject `now`, dispatch to subcommands
- Location: `src/main.rs`, `src/cli.rs`
- Contains: `clap` parsing, stdin read, `SystemTime` call, `Config::load`
- Depends on: model, render, tui
- Used by: nothing (binary entry point)
- Purpose: Data types — input, config, colors, styles
- Location: `src/model/`
- Contains: `InputData`, `Config`, `SegmentKind`, `Thresholds`, `Theme`, `Style`, `Coerce<T>`
- Depends on: serde, toml
- Used by: all other layers
- Purpose: Compose segments into ANSI output
- Location: `src/render/`
- Contains: `render_line`, `render_with`, `SegmentWriter`, `make_bar`
- Depends on: model, segment
- Used by: main.rs (render subcommand), tui/preview.rs
- Purpose: Domain logic for each status segment
- Location: `src/segment/`
- Contains: `Segment` trait, `RenderCtx`, six segment structs
- Depends on: model, render/writer (via SegmentWriter), sanitize, std::process (git subprocess)
- Used by: render layer
- Purpose: Name-to-value resolution for built-in themes and styles
- Location: `src/themes/`, `src/styles/`
- Contains: one file per theme/style, `mod.rs` with `get()` and `NAMES`
- Depends on: model (Theme, Style types)
- Used by: render layer, TUI
- Purpose: Interactive configurator, only compiled with `tui` feature
- Location: `src/tui/`
- Contains: terminal setup, event loop, widget layout, preview, sample fixtures
- Depends on: ratatui, crossterm, model, render
- Used by: main.rs (config subcommand)

## Data Flow

### Primary Request Path (render subcommand)

### TUI Preview Path

- No global mutable state. `App` in the TUI holds all mutable state for the configurator session.
- Config is loaded from disk at startup; saved explicitly on `s` keypress.
- Segment implementations are zero-sized; all data flows through `RenderCtx`.

## Key Abstractions

- Purpose: Uniform interface for all status bar segments
- Examples: `src/segment/directory.rs`, `src/segment/git.rs`, `src/segment/rate_limits.rs`
- Pattern: `fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool`
- Purpose: Encapsulate ANSI color emission so segments never embed raw escape codes
- File: `src/render/writer.rs`
- Pattern: Methods `colored`, `dim`, `icon`, `bar`, `raw` — theme and style resolved internally
- Purpose: Forgiving JSON number deserializer; wrong-typed or absent fields degrade to `None`
- File: `src/model/input.rs`
- Pattern: Custom `Deserialize` impl that maps bool/seq/map/garbage → `Coerce(None)`
- Purpose: Dependency injection bundle passed to every segment
- File: `src/segment/mod.rs`
- Pattern: `&InputData`, `&Theme`, `&Style`, `&Thresholds`, `now: i64`, `home: Option<&str>`
- Purpose: Named color slot structs and glyph character structs; resolved by `get(name)`
- Files: `src/model/palette.rs`, `src/model/style.rs`, `src/themes/mod.rs`, `src/styles/mod.rs`
- Pattern: Fixed struct fields = compile error if a theme omits a color

## Entry Points

- Location: `src/render/mod.rs:18`
- Triggers: `claudebar` / `claudebar render` invoked by Claude Code status line hook
- Responsibilities: Resolve theme/style, inject `$HOME`, delegate to `render_with`
- Location: `src/render/mod.rs:28`
- Triggers: Called by `render_line` and by TUI preview
- Responsibilities: Build `RenderCtx`, iterate segments, compose ANSI output string
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

### Raw ANSI codes in segments

### Second render code path for TUI preview

## Error Handling

- `InputData::parse` returns `InputData::default()` on any JSON failure
- `Coerce<T>` degrades wrong-typed fields to `None` without propagating errors
- `Config::load` returns `Config::default()` for missing files; surfaces `ConfigError` for malformed TOML
- `main.rs` prints warnings to stderr on config parse failure and continues with defaults
- Segment `render()` returns `false` to skip a segment that has no data to show

## Cross-Cutting Concerns

<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->

## Project Skills

| Skill | Description | Path |
|-------|-------------|------|
| code-review-excellence | Master effective code review practices to provide constructive feedback, catch bugs early, and foster knowledge sharing while maintaining team morale. Use when reviewing pull requests, establishing review standards, or mentoring developers. | `.agents/skills/code-review-excellence/SKILL.md` |
| deployment-pipeline-design | Design multi-stage CI/CD pipelines with approval gates, security checks, and deployment orchestration. Use this skill when designing zero-downtime deployment pipelines, implementing canary rollout strategies, setting up multi-environment promotion workflows, or debugging failed deployment gates in CI/CD. | `.agents/skills/deployment-pipeline-design/SKILL.md` |
| error-handling-patterns | Master error handling patterns across languages including exceptions, Result types, error propagation, and graceful degradation to build resilient applications. Use when implementing error handling, designing APIs, or improving application reliability. | `.agents/skills/error-handling-patterns/SKILL.md` |
| find-skills | Helps users discover and install agent skills when they ask questions like "how do I do X", "find a skill for X", "is there a skill that can...", or express interest in extending capabilities. This skill should be used when the user is looking for functionality that might exist as an installable skill. | `.agents/skills/find-skills/SKILL.md` |
| git-advanced-workflows | Master advanced Git workflows including rebasing, cherry-picking, bisect, worktrees, and reflog to maintain clean history and recover from any situation. Use when managing complex Git histories, collaborating on feature branches, or troubleshooting repository issues. | `.agents/skills/git-advanced-workflows/SKILL.md` |
| github-actions-templates | Create production-ready GitHub Actions workflows for automated testing, building, and deploying applications. Use when setting up CI/CD with GitHub Actions, automating development workflows, or creating reusable workflow templates. | `.agents/skills/github-actions-templates/SKILL.md` |
| rust-async-patterns | Master Rust async programming with Tokio, async traits, error handling, and concurrent patterns. Use when building async Rust applications, implementing concurrent systems, or debugging async code. | `.agents/skills/rust-async-patterns/SKILL.md` |
| rust-skills | > Comprehensive Rust coding guidelines with 265 rules across 26 categories. Use when writing, reviewing, or refactoring Rust code. Covers ownership, error handling, async patterns, concurrency, unsafe code, API design, memory optimization, performance, numeric safety, conversions, serde, pattern matching, macros, closures, observability, testing, and common anti-patterns. Invoke with /rust-skills. | `.agents/skills/rust-skills/SKILL.md` |
| skill-creator | Create new skills, modify and improve existing skills, and measure skill performance. Use when users want to create a skill from scratch, edit, or optimize an existing skill, run evals to test a skill, benchmark skill performance with variance analysis, or optimize a skill's description for better triggering accuracy. | `.agents/skills/skill-creator/SKILL.md` |
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:

- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work



<!-- GSD:screenshots-start -->

## Screenshot Pipeline

`scripts/gen_screenshots.py` generates README strips and full terminal PNGs.

**Prerequisites:**
- Host Chromium at `/usr/local/bin/chromium`
- Hack Nerd Font at `/tmp/fonts/HackNerdFontMono-Regular.ttf`
  `curl -sL https://github.com/ryanoasis/nerd-fonts/releases/download/v3.3.0/Hack.tar.xz | tar -xJf - -C /tmp/fonts --wildcards 'HackNerdFontMono-Regular.ttf'`
- playwright-core at `/tmp/pw/node_modules`
  `mkdir -p /tmp/pw && cd /tmp/pw && npm install --prefix . playwright-core`
- Release binary: `cargo build --release`

**Generate strips (fast, no Docker):**
```bash
CLAUDEBAR_CHROME=/usr/local/bin/chromium \
PW_MODULES=/tmp/pw/node_modules \
NF_FONT_DIR=/tmp/fonts \
python3 scripts/gen_screenshots.py --strips
```
Output: `screenshots/strip-{normal,critical,overlimit,green,nogit,noeffort,features}.png`

<!-- GSD:screenshots-end -->

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->

## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
