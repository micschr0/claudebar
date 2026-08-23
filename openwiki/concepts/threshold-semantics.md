---
type: "Reference"
title: "Threshold semantics"
openwiki_generated: true
---

# Threshold semantics

Thresholds control color bands and visibility of the rate-limit/context
segments. They live in the TOML config under `thresholds` and are defined in
`src/model/config.rs::Thresholds` with `#[serde(default)]`.

## Keys and defaults

| Key | Default | Meaning |
|---|---|---|
| `warn_at` | 70 | usage (pct) → warn color |
| `crit_at` | 90 | usage (pct) → crit color |
| `weekly_show_at` | 50 | weekly window appears from this pct |
| `bar_width` | 10 | progress bar cell count |
| `cost_decimals` | 2 | cost precision |
| `name_max` | — | **declared but not consumed** (inert) |
| `model_show_effort` | — | **declared but not consumed** (inert) |
| `clock_mode` | `"auto"` | `auto`/`12h`/`24h`/`off` |
| `burn_lookback` | — | burn regression window |
| `layout` | `"fixed"` | `fixed`/`auto` |
| `limit_sync` | opt-in | cross-session rate-limit sync |

> The two "declared-but-not-consumed" keys appear only in config declaration
> and defaults; no renderer reads them. They are inert config surface — do not
> rely on them for behavior. (Source: `src/model/config.rs`.)

## Color-band rules (context/rate-limits)

- `pct < warn_at` → normal (theme `usage_fg`)
- `warn_at <= pct < crit_at` → warn color
- `pct >= crit_at` → crit color
- `used_percentage` is clamped to `≤999`; ≥100 is possible (over-limit
  sessions).

## weekly window

- Shown when the weekly `pct >= weekly_show_at` (else hidden).
- Uses `window_gap` to join the 5h/weekly windows.
- Reset countdown via `fmt_reset` (see `sanitize-formatting`).

## TUI nudging clamps

The TUI configurator (`src/tui/app.rs::nudge_threshold`) enforces sane
relations — e.g. `warn_at < crit_at` — when the user edits thresholds.

## Tests

- `src/model/config.rs` unit tests (defaults, serde partial).
- Context/rate-limits segment tests for color-band boundaries and over-100.