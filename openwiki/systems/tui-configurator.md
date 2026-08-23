# TUI configurator

`claudebar config` — interactive theme/style/segment setup with a **live
preview** that reuses the exact render path.

- Entry: `run(config_path)` in `src/tui/mod.rs` — load config, `TerminalGuard`
  (raw mode, alt screen, mouse), event loop (200ms poll).
- **App state** (`src/tui/app.rs`): flat cursor/selectable list + menu cursor/
  detail cursor/focused panel; `is_dirty`/`save`/`reset`; threshold
  nudge/cycling; reorder mode; samples; help overlay.
- Key dispatch priority: help overlay → pending reset → pending quit →
  reorder → normal.
- **Layout** (`ui.rs`): 3-zone (left menu / right detail / preview / status /
  hint); fallback below 80×20; mouse hit-testing via Cell areas; theme swatch
  cache.
- **Preview** (`preview.rs`): `render(cfg, sample)` → real `render_with` →
  `ansi_to_tui` Text; fixed `FIXED_NOW`, `PREVIEW_HOME`.
- **Samples** (`sample.rs`): 6 fixture samples.
- Save on `s` keypress; config persisted to the TOML path.

## Keymap

See the on-screen help overlay; primary keys: arrows/tab to move, enter to
select, `s` save, `q` quit, `?` help.

## Tests

- `app.rs` unit tests (~25): pure logic — `toggle_enables_absent_segment`,
  `move_up_swaps_with_predecessor`, `reset_restores_defaults_and_cursors`,
  `save_clears_dirty`, `toggle_cursor_sets_dirty_and_follows_segment`,
  `reorder_follows_cursor_in_display_order`, …
- `sample.rs`: one test (`six_samples_including_weekly_window`).
- `mod.rs`/`ui.rs`/`preview.rs` are **0%-coverage by design** (UI shell +
  ratatui draw; verified by screenshot scripts) per `CONTRIBUTING.md`.

## What to watch out for

- Preview must stay on the single render path (`render_with`) — never build a
  second composition for the TUI.
- All mutable state lives in `App`; no globals.