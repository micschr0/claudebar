---
type: "Reference"
title: "Directory segment"
openwiki_generated: true
---

# Directory segment

Shows the current working directory, abbreviated fish-style.

- Input: `input.cwd`, `ctx.home`.
- Emission: `abbreviate_path` (dotfiles → 2 chars, `~` for home),
  control-stripped, painted in the theme's directory color with a leading
  space inside the dir color slot.
- Source: `src/segment/directory.rs`, `src/sanitize.rs::abbreviate_path`.
- Tests: directory module tests (abbreviation table), golden covers the
  leading-space + color.