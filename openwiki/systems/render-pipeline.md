# Render pipeline

The composition layer that turns `(input × config)` into one ANSI status line.

## Entrypoints

```rust
pub fn render_line(input: &InputData, cfg: &Config, now: i64) -> String
pub fn render_with(input, cfg, theme, style, now, home: Option<&str>, tz_offset_seconds: i32) -> String
```

- `render_line` is the hook/TUI-shared entrypoint: resolves theme/style,
  injects `$HOME` and tz offset, delegates to `render_with`.
- `render_with` is lower-level; the TUI preview and tests call it with
  already-resolved values (deterministic `home`/`now`).

## Fixed vs auto layout

### `render_fixed` (default, `layout: "fixed"`)

Iterates `cfg.segments` in order, writes each via `SegmentWriter`, inserts a
separator between adjacent non-empty segments. One line. The original
behavior.

### `render_auto` (`layout: "auto"`)

Greedy wrapping across up to `max_lines` lines; each line takes as many
segments as fit within terminal width minus `wrap_margin`. Once `max_lines` is
reached, the rest pack onto the last line. Width 0 (unknown terminal) disables
wrapping → single line.

Terminal width: `$COLUMNS` first, then `stty size`, else 0.

## Separator rules

- `separator(line, theme, style)` appends: space + separator glyph in theme's
  separator color + space.
- Only between **two adjacent non-empty segments** — a segment returning
  `false` causes no separator around it (bash fallback differs here: it always
  emits chevrons).
- `separator_width(style)` mirrors the glyph (two spaces + glyph, or one space
  for lean's empty glyph).

## `SegmentWriter`

API: `colored`, `colored_with`, `colored_fmt`, `dim`, `icon`, `bar`,
`bar_pct`, `window_gap`, `raw`, `raw_fmt`; active-color stack; theme/style
resolved internally. Segments never embed raw escape codes.

## `make_bar`

`make_bar(pct, width, colors)` builds a progress bar string from pct, width,
and fg/bg colors (`src/render/bar.rs`). Used by context and rate-limits.

## Float side effect

`render_float` renders the float indicator **through the ASCII style then
strips ANSI** (`strip_ansi`, a CSI state machine) — the float writes a tiny
marker file so the next render can show a "float is active" indicator. See
[cross-session-state](cross-session-state.md).

## Tests

- `src/render/mod.rs` unit tests: separator width, terminal width, auto-wrap,
  empty-home.
- `tests/render_golden.rs` — full-output golden snapshots (`golden_lines`).