# Testing Patterns

**Analysis Date:** 2026-06-20

## Test Framework

**Runner:**
- Rust built-in `cargo test`
- Snapshot testing: `insta` 1.x with `features = ["filters", "glob"]`
- Config: `[dev-dependencies]` in `Cargo.toml` — no separate test config file

**Assertion Library:**
- Rust standard `assert_eq!`, `assert_ne!`, `assert!`
- `insta::assert_snapshot!` for golden output

**Run Commands:**
```bash
cargo test                 # Run all tests
cargo test -- --nocapture  # Run with stdout
cargo insta review         # Review/accept snapshot changes
INSTA_UPDATE=always cargo test  # Auto-accept snapshots
```

## Test File Organization

**Location:**
- Unit tests: `#[cfg(test)]` module at the bottom of the source file they test (co-located)
- Integration tests: `tests/render_golden.rs` — a separate integration test file
- Snapshots: `tests/snapshots/` — committed alongside the test

**Naming:**
- Test functions use `snake_case`: `zero_is_all_empty`, `strips_escape_bytes`, `roundtrips_through_toml`
- Snapshot files auto-named by insta: `render_golden__golden_lines@<fixture>.json.snap`

**Structure:**
```
tests/
  render_golden.rs          # Integration golden-output tests
  snapshots/                # Committed insta snapshots (14 files)
    render_golden__golden_lines@typical.json.snap
    render_golden__golden_lines@injection.json.snap
    ... (one per fixture/*.json file)
src/
  sanitize.rs               # #[cfg(test)] at bottom — path/format helpers
  render/bar.rs             # #[cfg(test)] at bottom — bar builder
  model/input.rs            # #[cfg(test)] at bottom — Coerce<T> deserializer
  model/config.rs           # #[cfg(test)] at bottom — Config TOML roundtrip
  themes/catppuccin.rs      # #[cfg(test)] at bottom — theme slot tests
  themes/nord.rs            # #[cfg(test)] at bottom
  themes/gruvbox.rs         # #[cfg(test)] at bottom
  themes/dracula.rs         # #[cfg(test)] at bottom
  themes/rose_pine.rs       # #[cfg(test)] at bottom
  segment/{directory,git,context,rate_limits,model}.rs  # #[cfg(test)] at bottom
  styles/{ascii,minimal,plain,rounded}.rs               # #[cfg(test)] at bottom
  tui/app.rs                # #[cfg(test)] at bottom — pure TUI logic
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper for DRY test setup (common pattern)
    fn plain(pct: u32) -> String {
        let s = make_bar(pct, 6, Color(1), Color(2), '#', '-');
        s.chars().filter(|c| *c == '#' || *c == '-').collect()
    }

    #[test]
    fn zero_is_all_empty() {
        assert_eq!(plain(0), "------");
    }
}
```

**Patterns:**
- Helper functions inside the `mod tests` block for repetitive setup: `fn plain()`, `fn cw()`
- Describe-then-assert single-concept per test
- No `before_each`/`after_each` — Rust tests are stateless by default
- `use super::*;` imports the module under test directly

## Mocking

**Framework:** None — no mock library used.

**Patterns:**
- Dependency injection at the type level eliminates mocking: `now: i64` and `home: Option<&str>` are passed in rather than read from the environment
- Filesystem not mocked — integration tests use `fixtures/*.json` as real files on disk
- Git subprocess calls are not unit-tested; the `parse_status()` parser is tested directly with string inputs

**What to Mock:**
- Not applicable — design avoids I/O in hot path via `RenderCtx` injection

**What NOT to Mock:**
- File system: tests either use real fixture files or temp paths
- `serde_json::from_str` — tested directly with inline JSON strings

## Fixtures and Factories

**Test Data:**
```rust
// Integration tests glob all JSON fixtures
insta::glob!(env!("CARGO_MANIFEST_DIR"), "fixtures/*.json", |path| {
    let json = std::fs::read_to_string(path).unwrap();
    insta::assert_snapshot!(render_fixture(&json));
});

// Unit tests use inline JSON strings
fn cw(json: &str) -> ContextWindow {
    serde_json::from_str(json).unwrap()
}
let c = cw(r#"{"total_input_tokens": 35000, "used_percentage": 67.5}"#);
```

**Location:**
- JSON fixtures: `fixtures/*.json` (14 files covering edge cases)
- Edge cases covered: `bad_types.json`, `injection.json`, `over_limit_5h.json`, `over_100_context.json`, `empty.json`, `missing_resets.json`, `no_effort.json`, `no_git.json`, `huge_tokens.json`, `typical.json`, `dev_context.json`, `effort_max.json`

## Coverage

**Requirements:** No enforced coverage threshold.

**View Coverage:**
```bash
cargo test  # No built-in coverage; use cargo-llvm-cov or cargo-tarpaulin separately
```

## Test Types

**Unit Tests:**
- Co-located `#[cfg(test)]` modules
- Scope: pure functions — string formatters, parsers, bar builders, config roundtrips
- No I/O, no subprocesses

**Integration / Golden Tests:**
- `tests/render_golden.rs`
- Renders each `fixtures/*.json` file end-to-end with a fixed clock (`FIXED_NOW = 1_899_990_000`) and fixed `$HOME = /home/me`
- Output is ESC-escaped for readable diffs: `line.replace('\x1b', "\\e")`
- Committed snapshots in `tests/snapshots/` — must be reviewed with `cargo insta review` after any rendering change

**E2E Tests:**
- Not implemented — manual smoke tests via `cat fixtures/typical.json | cargo run --quiet -- render`

## Common Patterns

**Async Testing:**
- Not applicable — no async code in the codebase

**Snapshot Testing:**
```rust
// tests/render_golden.rs
const FIXED_NOW: i64 = 1_899_990_000;  // stable epoch for countdown determinism

fn render_fixture(json: &str) -> String {
    let input = InputData::parse(json);
    let cfg = Config::default();
    let theme = themes::get(&cfg.theme);
    let style = styles::get(&cfg.style);
    let line = render_with(&input, &cfg, &theme, &style, FIXED_NOW, Some("/home/me"));
    line.replace('\x1b', "\\e")
}

#[test]
fn golden_lines() {
    insta::glob!(env!("CARGO_MANIFEST_DIR"), "fixtures/*.json", |path| {
        let json = std::fs::read_to_string(path).unwrap();
        insta::assert_snapshot!(render_fixture(&json));
    });
}
```

**Error/Edge Case Testing:**
```rust
// Test degraded inputs — wrong types, None, out-of-range
#[test]
fn wrong_type_degrades_to_none() {
    let c = cw(r#"{"total_input_tokens": true, "used_percentage": "abc"}"#);
    assert_eq!(c.total_input_tokens.get(), None);
    assert_eq!(c.used_percentage.get(), None);
}

#[test]
fn whole_parse_never_fails() {
    let d = InputData::parse("not json at all");
    assert!(d.cwd.is_none());
}
```

**Theme Slot Tests (pattern for new themes):**
```rust
// themes/catppuccin.rs — every theme file has this pattern
#[test]
fn fills_key_slots() {
    let t = theme();
    assert_eq!(t.dir, Color(111));
    assert_eq!(t.git_branch, Color(183));
}

#[test]
fn bar_thresholds_are_distinct() {
    let t = theme();
    assert_ne!(t.bar_ok, t.bar_warn);
    assert_ne!(t.bar_warn, t.bar_crit);
}
```

## Adding New Tests

**New segment:** Add `#[cfg(test)]` block to the segment's `.rs` file. Use inline string inputs to `parse_status()` or equivalent parser. Add a corresponding `fixtures/<name>.json` so the golden test covers it end-to-end.

**New theme:** Add `#[cfg(test)]` block following the `fills_key_slots` + `bar_thresholds_are_distinct` pattern. Cross-reference against another theme with `assert_ne!(theme().dir, super::super::other_theme::theme().dir)`.

**New formatter:** Add exhaustive unit tests for boundary values (0, threshold, max, overflow) directly in the source file.

---

*Testing analysis: 2026-06-20*
