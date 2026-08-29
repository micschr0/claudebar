---
type: concept
title: "Themes, styles, and the color model"
description: "How the fixed-struct Theme of named 256-color slots, the Style/GlyphSet definitions (including bar_dots quarter-step bars), and the built-in theme and style registries work, and how to add a theme or style consistently."
tags: [theme, style, color, palette, glyphset, theming, statusline, style-dots, bar-dots]
sources:
  - id: openwiki-source-058fc071169e55002128500a
    resource: repo://CONTRIBUTING-themes.md
  - id: openwiki-source-b55a21a31ede1b56cd31a6a6
    resource: repo://src/main.rs
  - id: openwiki-source-c5edfb46b7c4acb766451a37
    resource: repo://src/model/config.rs
  - id: openwiki-source-6676bf05b7330b243f0ed91f
    resource: repo://src/model/palette.rs
  - id: openwiki-source-3cc7f3c7b930ee9387b11a58
    resource: repo://src/model/style.rs
  - id: openwiki-source-f8dfbb6a10fc50e65133b3f7
    resource: repo://src/render/bar.rs
  - id: openwiki-source-763738302a84ffdcefbc9913
    resource: repo://src/render/float.rs
  - id: openwiki-source-1d33473d874a4090bb6026e0
    resource: repo://src/render/mod.rs
  - id: openwiki-source-d977bd28254dbfcf5d7fe3bb
    resource: repo://src/render/writer.rs
  - id: openwiki-source-f4acb0adeb73aa0d54303a27
    resource: repo://src/segment/mod.rs
  - id: openwiki-source-c5d0aaf913f55a00eb2e5796
    resource: repo://src/styles/mod.rs
  - id: openwiki-source-079dcbc1bf8b946f21372942
    resource: repo://src/themes/mod.rs
  - id: openwiki-source-d498d54938db40c967e5c84b
    resource: repo://src/tui/app.rs
verified:
  - by: openwiki/0.4.0
    at: 2026-08-29T00:17:43.706Z
generated: {by: "openwiki/0.4.0", at: "2026-08-29T00:17:43.706Z"}
---

# Themes, styles, and the color model

Claudebar separates **look-and-feel** into two orthogonal axes:

- **Themes** (`src/themes/mod.rs`) are a fixed struct of *named color slots* —
  one 256-color ANSI index per semantic role (`dir`, `git_branch`, `bar_ok`,
  `separator`, `dim`, …).
- **Styles** (`src/styles/mod.rs`) are pure data describing *how* segments are
  separated and decorated: the separator glyph, bar characters, the icon
  toggle, and a `GlyphSet` of per-segment glyphs.

A theme and a style are independently selectable in `config.toml` and combine at
render time: a segment picks *what* to show, a style picks *how* to glyph it,
and a theme picks *what color* it gets. The color and style definitions live in
`src/model/palette.rs` and `src/model/style.rs`; the built-in registries live in
`src/themes/mod.rs` and `src/styles/mod.rs`.

## The color model: `Color` and SGR emission

`src/model/palette.rs` defines a single `Color` type:

```rust
pub struct Color(pub u8);
```

It is a **256-color ANSI palette index** (0–255), rendered as the SGR
foreground sequence `\x1b[38;5;<n>m`. It implements `Copy`, `Eq`, and `Debug`,
so a theme can be a `Copy` value held by reference through a render pass.

Emission happens two ways, and the difference matters on the render hot path:

- `Color::fg(self) -> String` formats and **allocates** a fresh `String` —
  useful for tests and simple call sites.
- `Color::write_fg(self, buf: &mut String)` **pushes straight into a caller's
  `String` buffer**, avoiding the throwaway allocation. All render code paths
  (`SegmentWriter`, the composer's `separator()`, and `bar::write_bar` /
  `bar::write_bar_dots`) use `write_fg`, because a status line is built with
  many colored runs per frame.

`palette.rs` also exports `RESET: &str = "\x1b[0m"` — the SGR reset that ends
every colored run.

## The `Theme`: a fixed struct of named color slots

A theme is deliberately **not a map** — it is a fixed struct with one `Color`
field per semantic role (`src/model/palette.rs`):

```rust
pub struct Theme {
    pub dir: Color,
    pub git_branch: Color,
    pub ahead: Color,
    pub behind: Color,
    pub modified: Color,
    pub untracked: Color,
    pub token: Color,
    pub bar_ok: Color,
    pub bar_warn: Color,
    pub bar_crit: Color,
    pub bar_track: Color,
    pub separator: Color,
    pub dim: Color,
    pub reset: Color,
    pub model: Color,
    pub stash: Color,
    pub lines: Color,
    pub cost: Color,
    pub duration: Color,
    pub clock: Color,
    pub effort: Color,
    pub burn: Color,
}
```

**Adding a slot is a compile error in every theme that omits it**, so a theme can
never silently miss a color the renderer expects. The compiler is the safety
net that guarantees every built-in theme fills every slot. The doc comments on
each field document its semantic role — e.g. `dir` = directory path,
`bar_warn` = progress fill at/above warn, `stash` = stash count, and the
"background" slots `lines`, `cost`, `duration`, `clock`, `effort`, and `burn`
feed the composited background of the cost / duration / clock / effort /
burn-rate readouts.

One slot documents a **fallback convention** in commentary rather than code:
`stash` "falls back to `git_branch`" in themes that predate the slot — i.e. it
is intentionally set to the same index as `git_branch` in older palettes.

Themes are resolved by name via `themes::get(name)` (`src/themes/mod.rs`).
Unknown names **fall back to Tokyo Night**:

```rust
pub fn get(name: &str) -> Theme {
    match name {
        "ayu-mirage" => AYU_MIRAGE,
        // ... one arm per built-in theme ...
        _ => TOKYO_NIGHT,
    }
}
```

The default theme is `tokyo-night` — set in `Config::default()` and again in
the registry's fallback arm.

## The built-in theme registry

All built-in themes live in **`src/themes/mod.rs` as `pub const` values**,
with **no per-theme modules** and no merge-conflict surface. The `NAMES` slice
is the public, display-order list, and the default is Tokyo Night:

```rust
pub const NAMES: &[&str] = &[
    "tokyo-night", "ayu-mirage", "catppuccin", "cobalt2",
    "everforest-dark", "github-dark", "gruvbox", "kanagawa-wave",
    "moonfly", "night-owl", "nord", "one-dark", "dracula",
    "rose-pine", "sonokai", "solarized-dark",
];
```

There are **16 built-in themes**. Each is a `Theme` literal of `Color(n)`
indices (e.g. `TOKYO_NIGHT.dir = Color(39)`). The registry also holds three
focused unit tests: `all_known_names_resolve` (every `NAMES` entry resolves
through `get`), an unknown-name fallback test, and `bar_thresholds_distinct`
which enforces a real invariant: for every theme the three bar fill colors
`bar_ok`, `bar_warn`, and `bar_crit` must be pairwise distinct (so the
warn/crit bands are always visually distinguishable).

## Style: separators, glyphs, icons, and bar chars

`src/model/style.rs` defines a style as **pure data** — segments never branch
on the style; they read its fields. The two main types:

```rust
pub struct GlyphSet {
    pub branch: &'static str,
    pub ahead: &'static str,
    // ... one per segment/feature ...
}
pub struct Style {
    pub separator: &'static str,
    pub window_gap: &'static str,
    pub icons: bool,
    pub glyphs: GlyphSet,
    pub bar_fill: char,
    pub bar_empty: char,
    pub bar_dots: Option<[char; 5]>,
}
```

`Style`'s fields are:

- `separator` — the glyph placed **between adjacent non-empty segments** by the
  composer, painted in the theme's `separator` color with a space on each side.
  The "lean" style uses an **empty** separator, so the composer emits just a
  single space with no color codes.
- `window_gap` — a **lighter, intra-segment** separator joining two related
  "windows" inside one segment (e.g. rate-limits' 5h/weekly gauges), painted in
  the theme's `dim` color so the pair reads as one grouped unit rather than a
  segment boundary.
- `icons` — when `false`, icons are suppressed entirely (minimal / plain /
  ASCII / dots styles).
- `glyphs` — the `GlyphSet` this style draws from.
- `bar_fill` / `bar_empty` — the filled and empty cells of a **binary** progress
  bar track. These are single `char`s so `bar::write_bar` can push them directly.
- `bar_dots: Option<[char; 5]>` — selects the **quarter-step dot-meter bar**
  when `Some`; `None` selects the binary `bar_fill`/`bar_empty` bar. The five
  `char`s are the per-cell levels: index 0 = empty cell through index 4 = full
  cell (e.g. `['○', '◔', '◑', '◕', '●']`). When set, `SegmentWriter::bar`
  dispatches to `bar::write_bar_dots` instead of `write_bar`.

`GlyphSet` stores glyphs as `&'static str` (not `char`) because rich styles use
Nerd Font PUA / multi-byte glyphs (e.g. `\u{f0c29}` for the token icon), while
ASCII fallbacks use ASCII (`#`, `^`, `~`, …).

A style is pure data: the ASCII fallback, for instance, is just a `GlyphSet` of
ASCII characters with `icons = false` and `#`/`-` bar chars.

## The built-in style registry

All built-in styles live in **`src/styles/mod.rs` as `pub const` values**, again
with no per-style modules. The `NAMES` slice is the display-order list; Powerline
is the default:

```rust
pub const NAMES: &[&str] = &[
    "powerline", "lean", "plain", "rounded", "minimal", "unicode", "ascii", "dots",
];
```

There are **8 built-in styles**:

- **powerline** (default) — Nerd-Font powerline separator `\u{e0b1}`, icons on,
  heavy-solid bar chars.
- **lean** — same powerline glyphs but an **empty** separator (a single space
  between segments).
- **plain** — ASCII pipe `|` separator, icons off, `#`/`-` bar chars, ASCII
  glyphs.
- **rounded** — the rounded powerline separator `\u{e0b5}`, otherwise reuses
  `POWERLINE.glyphs`.
- **minimal** — a middot separator, icons off, but reuses `POWERLINE.glyphs`
  (icons are gated by the `icons` flag, not the glyph table). It overrides the
  two review markers (`✓`/`×`) to plain unicode because those bypass
  `SegmentWriter::icon` and must not be Nerd Font PUA.
- **unicode** — a plain-text `❯` separator with full-width unicode glyphs
  (`█`/`░` bars, `⎇`/`◉`/`⬡` icons).
- **ascii** — icons-off, ASCII-only glyphs and `#`/`-` bars; the base for the
  float readout.
- **dots** — powerline decoration (`\u{e0b1}` separator, `POWERLINE.glyphs`)
  but with `bar_dots: Some([…])` selecting the quarter-step dot-meter bar for
  progress readouts.

Styles are resolved via `styles::get(name)`; unknown names **fall back to
Powerline**. The module's tests assert every `NAMES` entry resolves to the
expected style (checking `separator`, `bar_fill`, and `bar_dots`), and that an
unknown name falls back to `POWERLINE`.

Several styles **reuse `POWERLINE.glyphs`** (lean, rounded, minimal, dots). This
is why `icons` is a separate flag: `minimal` draws powerline glyphs but
suppresses them via `icons = false`, proving segments read the `icons` flag
through the writer rather than branching on a hardcoded style identity.

## How theme × style reach a segment

A theme and style are resolved once at the render boundary
(`render::render_line` / `render_with`), then threaded through a `RenderCtx`
into every segment and into the composer (`src/render/mod.rs`):

```rust
let theme = themes::get(&cfg.theme);
let style = styles::get(&cfg.style);
```

Inside `RenderCtx` (`src/segment/mod.rs`) the segment sees
`theme: &'a Theme` and `style: &'a Style`. Segments never embed a raw color and
never decide whether icons render — they emit through `SegmentWriter`
(`src/render/writer.rs`), which centralizes emission:

```rust
pub fn colored(&mut self, color: Color, text: &str) { color.write_fg(&mut self.buf); ... }
pub fn icon(&mut self, glyph: &str) {
    if self.style.icons && !glyph.is_empty() { self.theme.dim.write_fg(&mut self.buf); ... }
}
pub fn dim(&mut self, text: &str) { self.colored(self.theme.dim, text); }
```

`SegmentWriter::icon` applies the `icons` gate centrally, so minimal/ASCII
styles drop every glyph with **no per-segment branching**. `window_gap()` and
`bar()`/`bar_pct()` likewise read `theme.dim`, `theme.separator`, `theme.bar_track`,
`theme.bar_*`, `style.window_gap`, `style.bar_fill`/`bar_empty`, and
`style.bar_dots` in one place. `SegmentWriter::bar` dispatches on `bar_dots`:
`Some(levels)` routes to `bar::write_bar_dots`, `None` to `bar::write_bar`, so a
bar's binary-vs-quarter-step shape is decided by the style alone.

```mermaid
flowchart LR
  CS["config.toml: theme and style names"]
  TG["themes::get resolves Theme slots"]
  SG["styles::get resolves Style fields"]
  WC["SegmentWriter: write_fg into buf"]
  OUT["ANSI status line"]
  CS --> TG
  CS --> SG
  TG --> WC
  SG --> WC
  WC --> OUT
```

*Config-selected `theme` and `style` names resolve through the registries once, then feed a single `SegmentWriter` that emits every colored run.*

The composer's inter-segment `separator()` and the writer's intra-segment
`window_gap()` implement the two distinct join styles from the shared `Style`
struct: the separator is painted in `theme.separator` (a segment boundary),
while `window_gap` is painted in `theme.dim` (related windows grouped as one).
`separator_width()` mirrors `separator()` for layout math (2 + the glyph's
visible width, or 1 for the empty lean separator).

### The dot-meter bar

`bar::write_bar_dots` (`src/render/bar.rs`) is the pure string-builder behind
the `dots` style and any style with `bar_dots: Some`. Given the five per-cell
levels (index 0 = empty … 4 = full) it converts `pct` into a **quarter-step
count** (`pct * width * 4 / 100`, rounded half-up, clamped) and fills each of
`width` cells by its remaining quarter level, switching the ANSI color from the
fill to the track color once a cell is less than full. It keeps the same
contract as `write_bar`: `pct` may exceed 100 and is clamped, and a non-zero
`pct` always shows at least one quarter so a live bar stays visually distinct
from an empty one.

### The float readout is deliberately style-independent

`render/float.rs` renders each `float_segments` entry with the **ASCII style**
(icons-off, ASCII-only glyphs) and **strips ANSI** afterwards, so the float
file's plain text is independent of the user's configured theme/style. It still
uses theme colors internally but they are immediately stripped away.

## Configuration and failure semantics

The theme and style are plain `String` fields on `Config` (`src/model/config.rs`)
whose defaults are `"tokyo-night"` and `"powerline"`:

```rust
Self {
    theme: "tokyo-night".into(),
    style: "powerline".into(),
    // ...
}
```

- **Config-less operation** is first-class: a missing config file yields
  `Config::default()` (Tokyo Night + Powerline).
- **Unknown CLI names** are warned about and fall back to the defaults
  (`src/main.rs`): an unknown `--theme` prints `unknown theme '<t>' — using
  tokyo-night`, an unknown `--style` similarly falls back to powerline, and the
  same `themes::get` / `styles::get` fallback arms make rendering robust even if
  an unknown name reaches them from TOML.
- The TUI config editor offers the registry `NAMES` lists for pickers, indexes
  theme swatches by `themes::NAMES` position (`src/tui/app.rs`), and keeps a
  swatch cache in `themes::NAMES` order — reordering `NAMES` or changing
  `Theme` fields requires updating that swatch code.

## Adding a theme or style

[`CONTRIBUTING-themes.md`](repo://CONTRIBUTING-themes.md) at the repo root is the
**canonical theme-contribution path**. It walks a contributor through cloning a
`src/themes/<name>.rs` template (e.g. `src/themes/catppuccin.rs`), picking a
`Color(n)` xterm-256 index (0–255) nearest to each intended hex color, and making
the **three one-line registrations** in `src/themes/mod.rs`:

1. `pub mod my_theme;` — declare the module;
2. add `"my-theme"` to the `NAMES` slice;
3. add a `"my-theme" => my_theme::theme()` arm to `get`'s match.

It then has you preview the theme across all styles (`claudebar render --theme
my-theme --style <s>`) and confirm `cargo test`, `clippy -D warnings`, and
`fmt --check` before opening a PR. The registry currently ships all 16 built-in
themes as inline `pub const` values in `src/themes/mod.rs`, but the contributed
`src/themes/<name>.rs` module pattern is the supported way to add a new one.

- **Theme:** follow `CONTRIBUTING-themes.md` — create a `src/themes/<name>.rs`
  `theme()` function returning a full `Theme` struct literal, then make the three
  one-line registrations in `src/themes/mod.rs`. Because `Theme` is a fixed
  struct and `get` returns a `Theme`, **every theme must fill every slot** — the
  compiler enforces completeness, so a new theme can never silently miss a color.
  Follow the fallback convention for the `stash` slot if you want older-palette
  behavior.
- **Style:** add a `Style` and, if needed, a `GlyphSet` to `src/styles/mod.rs`,
  register the name in `styles::NAMES`, and add a `get` match arm. Reuse
  `POWERLINE.glyphs` with `icons: false` for a minimal/plain variant rather
  than duplicating the glyph table. For a dot-meter variant, set
  `bar_dots: Some(['○', '◔', '◑', '◕', '●'])` (empty…full) and add the name to
  the registry's `every_name_resolves_to_its_own_style` test table.

The registry layout (one file, `pub const` values, no merge-conflict surface) and
the exhaustive `get` match arms plus the focused `all_known_names_resolve`,
fallback, and `bar_thresholds_distinct` tests keep the registries honest when they
change.
