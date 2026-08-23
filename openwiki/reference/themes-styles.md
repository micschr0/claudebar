# Themes & styles

Named registries resolved to value structs. No globals — `themes::get` /
`styles::get` return owned values.

## Themes

- `Theme` has **fixed slots** for each semantic role (directory fg, git fg,
  usage fg, warn, crit, separator, …) — a compile error if a theme omits a
  slot.
- 16 built-in themes, one file per theme under `src/themes/`, `NAMES` list.
- `themes::get(name)` falls back to `tokyo-night` on unknown name.
- Tests: `themes` module (bar thresholds distinct per theme).

## Styles

- `Style` + `GlyphSet`; 7 built-in styles (`powerline`, `plain`, `ascii`,
  …).
- `styles::get(name)` falls back to `powerline`.
- Glyphs: PUA (private-use) glyphs for Nerd Font; ASCII substitutes for
  `plain`/`ascii`.
- Style controls separator glyph, bar fill/empty chars, model glyph.

## Contributing

- Add a theme: create `src/themes/<name>.rs` with a `const` Theme, register
  in `mod.rs`. See `CONTRIBUTING-themes.md`.
- Add a style: create `src/styles/<name>.rs`, register in `mod.rs`.

## Tests

- `src/themes/mod.rs`, `src/styles/mod.rs` unit tests (fallback, NAMES).