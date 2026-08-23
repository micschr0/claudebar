# Escape emission

How ANSI is produced and measured.

## Colors

- `Color(u8)` — xterm-256 slot; SGR `\x1b[38;5;Nm`.
- `write_fg` is no-alloc; `RESET` constant (`\x1b[0m`).
- Colors live in `src/model/palette.rs`.

## `SegmentWriter` API

`colored`, `colored_with`, `colored_fmt`, `dim`, `icon`, `bar`, `bar_pct`,
`window_gap`, `raw`, `raw_fmt` — active-color stack, theme/style resolved
internally. Segments never embed raw escape codes.

## `strip_ansi`

A CSI state machine that removes ANSI escape sequences — used by
`render_float` (src/render/float.rs) to render the float marker through the
ASCII style and then strip color.

## `visible_width`

`src/render/width.rs` — visible (non-escape) width of an ANSI string, with
wide/combining character tables. Used by the auto layout to measure segments
before wrapping.

## Tests

- `src/render/writer.rs`, `width.rs`, `float.rs` unit tests.
- Golden snapshots pin exact ANSI output (no stray escapes).