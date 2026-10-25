# Contributing a theme

**Version 1.1.0**

**[SPEC]**
claudebar ships **16 built-in themes**. Adding one is a single `pub const` plus one
registry line — both in `src/themes/mod.rs`. No per-theme file, no runtime config,
no parsing: claudebar compiles themes into the binary.

## AI READING INSTRUCTION

**[SPEC]** Read the `[SPEC]` and `[BUG]` tagged blocks for authoritative facts.
**[NOTE]** Read `[NOTE]` tagged blocks only if additional context is needed.
**[?]** Blocks tagged `[?]` are unverified — treat with lower confidence.

## 1. Add the theme const

**[SPEC]**
All themes live in [`src/themes/mod.rs`](src/themes/mod.rs) as `pub const` values.
Copy an existing const (e.g. `CATPPUCCIN`) and change the indices. A `Theme` has
**22 colour slots**; omitting one is a compile error.

```rust
/// my-theme palette.
pub const MY_THEME: Theme = Theme {
    dir: Color(111),        // directory path
    git_branch: Color(183), // branch name
    ahead: Color(150),      // ↑N ahead count
    behind: Color(211),     // ↓N behind count
    modified: Color(216),   // MN modified-file count
    untracked: Color(244),  // ?N untracked-file count
    token: Color(117),      // token count
    bar_ok: Color(150),     // bar fill below warn
    bar_warn: Color(223),   // bar fill at/above warn, below crit
    bar_crit: Color(204),   // bar fill at/above crit (and over-limit)
    bar_track: Color(241),  // empty bar cells
    separator: Color(241),  // separator glyph
    dim: Color(244),        // dimmed icons and secondary symbols
    reset: Color(115),      // reset/countdown timer value
    effort: Color(218),     // effort level background
    model: Color(140),      // model display name
    stash: Color(183),      // stash count
    lines: Color(244),      // lines added/removed background
    cost: Color(223),       // cost in USD background
    duration: Color(115),   // session duration background
    clock: Color(150),      // clock background
    burn: Color(204),       // burn-rate background
};
```

`Color(N)` is an **xterm-256 palette index** (0–255). Pick the index nearest to
your hex colour.

**[SPEC]**
Two invariants are enforced by tests, so check them before opening a PR:

- `bar_ok`, `bar_warn` and `bar_crit` must be **pairwise distinct** — otherwise the
  warn/crit bands are indistinguishable (`bar_thresholds_distinct`).
- No two themes may share an identical palette — the `registry!` macro generates
  `every_name_resolves_to_a_distinct_value`, which also catches a missing match arm.

## 2. Register it (1 line)

**[SPEC]**
Add one arm to the `registry!` block at the bottom of `src/themes/mod.rs`. The macro
generates `NAMES`, `get()`, and the distinctness test from this list — there is
nothing else to update.

```rust
crate::registry! { Theme, TOKYO_NIGHT,
    "tokyo-night"     => TOKYO_NIGHT,
    // ...
    "my-theme"        => MY_THEME,
}
```

Unknown names fall back to `TOKYO_NIGHT`, which is also the default in `Config::default()`.

## 3. Preview every style at once

**[SPEC]**
Build and render your theme across all **8 styles**:

```bash
cargo build --release
for s in powerline lean plain rounded minimal unicode ascii dots; do
  echo "── $s ──"
  ./target/release/claudebar render --theme my-theme --style "$s" < fixtures/typical.json
done
```

Regenerate the promo gallery — it reads `claudebar list`, so your theme appears
automatically:

```bash
bash scripts/gen-gallery.sh   # → docs/index.html
```

## 4. Open a PR

**[SPEC]**
`main` is protected — branch first, then open a PR.

- Confirm `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` pass.
- A new theme adds **6 rows** to `tests/golden/render_matrix.txt` (3 palette fixtures ×
  2 palette styles). Review them, then accept.
- Include a screenshot of at least the `powerline` and `rounded` styles.
- Ensure the theme is readable across all bar states (calm → over-limit).
