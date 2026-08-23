---
type: "Reference"
title: "Cross-session state"
openwiki_generated: true
---

# Cross-session state

Two pieces of local state persist across sessions so later renders can account
for history. Both are **best-effort and never break the render** — a missing or
corrupt cache degrades to defaults.

## `limit_sync` (rate-limit synchronization)

Idle sessions drift: a session that sat open while another consumed the limit
would show a stale rate. `limit_sync.rs` records high-water marks.

- Store layout: `limit-5h.d` / `limit-7d.d` directories under the cache dir
  (`$CLAUDEBAR_LIMIT_SYNC_DIR` override, else `$XDG_CACHE_HOME`/`$HOME/.cache`).
- Entry format: `<reset:%010d>_<pct:%07.3f>`.
- Atomicity: `mkdir` record creation; `latest()` high-water + GC via `rmdir`.
- Plausibility caps: 6h / 8d (rejects garbage timestamps).
- Opt-in: gated by `thresholds.limit_sync`.

## `burn` TSV cache

- File: `burn-5h.tsv` (plus 7d fallback), `MAX_ROWS` 1500, row-cap GC.
- `CLAUDEBAR_BURN_FILE` override.
- Consumed by the burn segment (regression slope, states, urgency).

## Rules

- Never break the render: read failures → default.
- Atomic writes (rename for float, mkdir for limit-sync, row-cap GC for burn).
- No secrets stored.