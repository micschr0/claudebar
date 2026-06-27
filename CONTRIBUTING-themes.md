# Contributing a theme

claudebar ships **16 built-in themes**. Adding one is a single small Rust file plus
three one-line registrations. No runtime config, no parsing — themes are compiled in.

## 1. Create the theme file

Themes live in [`src/themes/`](src/themes/). Copy an existing one as a template —
[`src/themes/catppuccin.rs`](src/themes/catppuccin.rs) is a clear, well-commented template.

```rust
// src/themes/my_theme.rs
use crate::model::{Color, Theme};

pub fn theme() -> Theme {
    Theme {
        dir: Color(111),        // directory segment
        git_branch: Color(183), // branch name
        ahead: Color(150),      // ↑ ahead count
        behind: Color(211),     // ↓ behind count
        modified: Color(216),
        untracked: Color(244),
        token: Color(117),      // context token count
        bar_ok: Color(150),     // bars: low usage (green)
        bar_warn: Color(223),   // bars: mid usage (yellow)
        bar_crit: Color(211),   // bars: high usage (red)
        bar_track: Color(240),  // empty part of a bar
        separator: Color(240),
        dim: Color(243),
        reset: Color(115),      // reset-time text
        effort_max: Color(218), // effort indicator at "max"
        model: Color(183),      // model name
    }
}
```

`Color(N)` is an **xterm-256 palette index** (0–255). Pick the index nearest to your
palette's hex — see the comments in `catppuccin.rs` for the hex→index mapping pattern.

## 2. Register it (3 lines in `src/themes/mod.rs`)

```rust
pub mod my_theme;                       // 1. declare the module
// ...
pub const NAMES: &[&str] = &[
    // ...
    "my-theme",                         // 2. add to the public name list
];
// ...
pub fn get(name: &str) -> Theme {
    match name {
        // ...
        "my-theme" => my_theme::theme(), // 3. add the match arm
        // ...
    }
}
```

The `get()` match is intentionally **complete** — every name in `NAMES` must have an arm.

## 3. Preview every style at once

Build and render your theme across all 6 styles:

```bash
cargo build --release
for s in powerline plain rounded minimal unicode ascii; do
  echo "── $s ──"
  ./target/release/claudebar render --theme my-theme --style "$s" < fixtures/typical.json
done
```

Or regenerate the full promo gallery (your theme appears automatically — it reads
`claudebar list`):

```bash
bash scripts/gen-gallery.sh   # → docs/index.html
```

## 4. Open a PR

- Confirm `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` pass.
- Include a screenshot of at least the `powerline` and `rounded` styles.
- Ensure the theme is readable across all six bar states (calm → over-limit).
