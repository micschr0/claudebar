# Context segment

Token usage: bar + percentage, fmt_tokens k/M rounding.

- Input: `context_window`, `used_percentage` (can exceed 100; clamped ≤999).
- Always renders (no skip path).
- Color bands per thresholds (`warn_at`/`crit_at`); bar via `make_bar` at
  `bar_width`.
- Source: `src/segment/context.rs`.
- Tests: threshold color bands, over-100, out-of-range.