# Coding Conventions

**Analysis Date:** 2026-06-20

## Naming Patterns

**Files:**
- Modules use `snake_case`: `rate_limits.rs`, `dev_context.rs`, `tokyo_night.rs`
- No barrel index files — each module is a named file
- Theme files named after the theme with underscores: `rose_pine.rs`, `tokyo_night.rs`

**Functions:**
- `snake_case` for all free functions: `make_bar()`, `render_line()`, `parse_status()`, `fmt_tokens()`, `fmt_reset()`
- Constructor-style methods named `new()` or `theme()` (for theme modules)
- Getter methods named descriptively: `as_str()`, `as_deref()`, `or_default()`, `get()`
- Boolean-returning methods named `is_*`: `is_empty()`, `is_dirty()`, `is_some()`
- Pure helpers prefixed with verb: `strip_control()`, `abbreviate_path()`, `fmt_scaled()`

**Types:**
- `PascalCase` for all structs, enums, traits: `InputData`, `SegmentKind`, `RenderCtx`, `Coerce<T>`
- Enums use `PascalCase` variants: `SegmentKind::RateLimits`, `StatusKind::Success`
- Traits named as nouns or roles: `Segment`, `Style`, `FromJsonNumber`

**Constants:**
- `SCREAMING_SNAKE_CASE`: `RESET`, `FIXED_NOW`, `ALL`

**Serde overrides:**
- `SegmentKind` uses `#[serde(rename_all = "kebab-case")]` for TOML: `rate-limits`, not `rate_limits`
- Structs use `#[serde(default)]` on all fields — partial input always accepted

## Module Structure

- Each module starts with a `//!` doc comment describing its purpose and contract
- Complex contracts (e.g., `rate_limits.rs`, `git.rs`) include inline pseudocode/spec in the module doc
- Private helpers defined in the same file as the type that uses them — no shared utilities module except `sanitize.rs`

## Code Style

**Formatting:**
- `cargo fmt` standard — no `.rustfmt.toml` overrides detected
- Trailing comments inline with color constants: `// Blue #89b4fa`

**Linting:**
- `cargo clippy -- -D warnings` is the lint gate (no `.clippy.toml`)
- `#[must_use]` on functions where ignoring the result is a likely bug: `Config::load()`, `Config::save()`, `render_line()`, `render_with()`
- `#[must_use = "..."]` with explanatory text when the reason is non-obvious

## Import Organization

**Order (standard Rust convention):**
1. `std` imports
2. External crate imports (`serde`, `ratatui`, etc.)
3. Internal `crate::` imports

**Pattern:**
```rust
use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use std::fmt;
use std::marker::PhantomData;
```

**Path Aliases:**
- None — all paths spelled out fully

## Error Handling

**Strategy:** Two-tier approach:
1. **Infallible rendering** — `InputData::parse()` never fails; returns `Default` on bad JSON. `Config::load_or_default()` swallows all errors. The render path never returns `Result`.
2. **Explicit errors for IO/config** — `Config::load()` and `Config::save()` return `Result<_, ConfigError>`

**Error types:**
- `thiserror::Error` derive for `ConfigError` with `#[error("...")]` messages
- Variants: `ConfigError::Io(String)` and `ConfigError::Parse(String)` — errors are strings, not boxed sources
- No `anyhow` — typed errors throughout

**Patterns:**
```rust
// Infallible parse with fallback
pub fn parse(s: &str) -> Self {
    serde_json::from_str(s).unwrap_or_default()
}

// IO result threading
match std::fs::read_to_string(path) {
    Ok(s) => toml::from_str(&s).map_err(|e| ConfigError::Parse(e.to_string())),
    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
    Err(e) => Err(ConfigError::Io(e.to_string())),
}
```

## Logging

**Framework:** None — no logging crate. Status messages go through the TUI `App.status` field for display.

**Patterns:**
- Errors surfaced to the user via stderr in `main.rs`; no structured logging

## Comments

**When to Comment:**
- Module-level `//!` doc comments are mandatory on every module
- Complex contracts (exact output format, edge cases) documented inline in the `//!` block, sometimes with pseudocode
- Inline `//` comments on color constant lines mapping hex colors to xterm-256 indices
- Non-obvious guards explained: `// At least one filled cell once there's any usage`
- No obvious/redundant comments

**Doc comments (`///`):**
- All `pub` functions, types, and constants carry `///` doc comments
- `/// # Errors` section on functions returning `Result`
- `pub(crate)` items use `///` only when the usage isn't obvious from the name

## Function Design

**Size:** Functions kept small; logic factored into private helpers (e.g., `pct_in_range()`, `write_reset()`, `fmt_scaled()`)

**Parameters:** Pass by reference for read-only data; mutable references for writers: `(ctx: &RenderCtx, out: &mut SegmentWriter)`

**Return Values:**
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

---

*Convention analysis: 2026-06-20*
