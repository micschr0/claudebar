---
type: "Reference"
title: "Segment seam"
openwiki_generated: true
---

# Segment seam

The `Segment` trait is the extension point for new status segments.

```rust
pub trait Segment {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool;
}
```

- **Return value** is a bool: `true` = emitted something, `false` = skip this
  segment (no separator, no content).
- Segments are **zero-sized structs**; `SegmentKind::as_segment()` returns
  `&'static dyn Segment`.
- Segments never know their neighbors; separators are added by the composer
  only between two non-empty segments.

## `RenderCtx`

Injected bundle passed to every segment (`src/segment/mod.rs`):

- `input: &InputData` — the hook state
- `theme: &Theme`, `style: &Style` — resolved colors/glyphs
- `thresholds: &Thresholds` — warn/crit bands etc.
- `now: i64` — epoch seconds (deterministic, injected by `main.rs`)
- `home: Option<&str>` — for `~` abbreviation (injected for tests)

Segments must **not** read env vars or system time directly — that is what
makes rendering deterministic and testable.

## `SegmentWriter`

ANSI output buffer (`src/render/writer.rs`). Segments never embed raw escape
codes; they call:

- `colored(color, text)` / `colored_with(color, text)` — paint in theme color
- `dim`, `icon`, `bar`, `bar_pct`, `window_gap`, `raw`, `raw_fmt`
- active-color stack, theme/style resolved internally

## `SegmentKind`

Enum of all segments, kebab-case serde names (`rate-limits`, `dev-context`).
`SegmentKind::ALL` / `DEFAULT` define canonical ordering; `from_kebab` and
`as_segment` dispatch. `DEFAULT` is the bash-parity default order.

## Extension recipe

1. Add a variant to `SegmentKind` + label + kebab name.
2. Create `src/segment/<name>.rs` implementing `Segment` (pure functions,
   injected ctx).
3. Register in `as_segment()` dispatch.
4. Write unit tests (`#[cfg(test)]`) in the module — table tests for pure
   helpers, a render test for the emission path.
5. Add golden coverage in `tests/render_golden.rs` if the segment changes the
   output for fixtures.
6. Wire into `claudebar setup sync` canonical positions if desired.

## Watch out

- Emit-nothing must return `false` so the composer skips the separator.
- Never read env/time; add fields to `RenderCtx` instead.
- Keep output host strings sanitized (see `concepts/sanitize-formatting.md`).