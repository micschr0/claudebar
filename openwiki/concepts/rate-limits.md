---
type: concept
title: "Rate limits: windows, thresholds, and cross-session sync"
description: "How the rate-limits segment renders Claude Code's 5-hour and 7-day usage windows — percentage rounding and color thresholds, weekly-show gating, reset countdowns — and how limit_sync shares a high-water mark across sessions on a host."
tags: [rate-limits, segment, thresholds, sync, cache, statusline]
verified:
  - by: openwiki/0.4.0
    at: 2026-08-29T00:17:43.706Z
sources:
  - id: openwiki-source-c5edfb46b7c4acb766451a37
    resource: repo://src/model/config.rs
  - id: openwiki-source-5d4fb36fe9d34b6bc366e220
    resource: repo://src/model/input.rs
  - id: openwiki-source-d977bd28254dbfcf5d7fe3bb
    resource: repo://src/render/writer.rs
  - id: openwiki-source-3a6ff89030cacfe8ee730edf
    resource: repo://src/sanitize.rs
  - id: openwiki-source-5bbd145b1c6fe30cc93223a1
    resource: repo://src/segment/limit_sync.rs
  - id: openwiki-source-d4594996ae77710bcd28b71f
    resource: repo://src/segment/rate_limits.rs
generated: {by: "openwiki/0.4.0", at: "2026-08-26T22:48:34.063Z"}
---

# Rate limits: windows, thresholds, and cross-session sync

The `rate_limits` segment (`src/segment/rate_limits.rs`) turns the `rate_limits`
field of the Claude Code hook JSON into two gauges rendered inside a single
statusline segment: a **5-hour window** and a **weekly (7-day) window**. Its
companion module `src/segment/limit_sync.rs` optionally shares the highest
usage observed across all sessions on a host so idle sessions don't show stale,
divergent numbers.

## Input shape

The input model (`src/model/input.rs`) defines `RateLimits` with two optional
`Window`s: `five_hour` and `seven_day`. Each `Window` carries `used_percentage`
(a `Coerce<f64>` that **can exceed 100** — over-limit is meaningful) and
`resets_at` (a Unix epoch in seconds). Every numeric field is wrapped in
`Coerce`, a forgiving deserializer that degrades a wrong-typed or unparseable
field to `None` instead of aborting the whole parse.

## Rendering the segment

The `RateLimits` segment implements the `Segment` trait
(`src/segment/mod.rs`): `render(&self, ctx, out) -> bool` writes spans into a
`SegmentWriter` and returns `true` iff anything was emitted. The 5-hour window
is rendered first; the weekly window follows, joined to it by
`SegmentWriter::window_gap()` (`src/render/writer.rs`) — a dim-colored glyph
(lighter than the composer's inter-segment separator) that signals the pair
belongs to one segment rather than marking a boundary.

Each window body (`write_window`) is an icon followed by a colored progress bar
and the percentage: `icon` + `bar_pct` (bar + `" "` + `"<pct>%"`). The 5-hour
window uses the clock glyph; the weekly window uses the weekly glyph. A reset
countdown is appended (`write_reset` / `write_reset_value`): `" "` + reset icon
+ `" "` + the formatted countdown in `theme.reset`.

### Percentage validation and rounding

`pct_in_range(p)` (`src/segment/rate_limits.rs`) rounds the percentage and
accepts it only when the result lands in `0..=999`. The upper bound rejects a
**leaked epoch timestamp** (e.g. a `used_percentage` of `1900000000`) while
still allowing over-limit values above 100. This range check is applied both in
the 5-hour and weekly render paths.

### Color thresholds

The bar color follows the `warn` / `crit` thresholds in `Thresholds`
(`src/model/config.rs`, defaults `warn: 50`, `crit: 80`):

- 5-hour window: `pct >= th.crit` → `bar_crit`; `pct >= th.warn` → `bar_warn`;
  otherwise `bar_ok`.
- Weekly window: `pct >= th.crit` → `bar_crit`; otherwise `bar_warn` (there is
  no "ok" state — the weekly gauge only appears once it has passed the show
  threshold, so it is at least warn-colored).

### Weekly-window gating

The weekly window is only surfaced once its rounded percentage is at or above
`th.weekly_show_at` (default `75`, `src/model/config.rs`). Below that threshold
the weekly gauge renders nothing, so the segment shows only the 5-hour window.

### Reset countdown

`crate::sanitize::fmt_reset(resets_at, now)` (`src/sanitize.rs`) formats the
time-until-reset adaptively as `Nd Nh` / `Nh Nm` / `Nm Ns` / `Ns`. It returns
`None` when the target is missing (`<= 0`) or already in the past, in which case
no countdown is emitted — but a window with a past reset still renders its
percentage (test `past_reset_shows_pct_without_countdown`).

## Cross-session sync (`limit_sync`)

Claude Code only re-renders a session's statusline while that session is active,
so each session's rate-limit numbers are its own last-seen snapshot; idle
sessions drift and diverge. `src/segment/limit_sync.rs` addresses this by
sharing the **high-water mark** across sessions on a host.

### Opt-in via configuration

The feature is gated by `th.limit_sync` (`Thresholds.limit_sync`, default
`false` — opt-in). In `effective_5h` / `effective_7d`
(`src/segment/rate_limits.rs`), when enabled, the session's own `(pct, reset)`
snapshot is first recorded into the shared store, then the shared high-water
mark is preferred **when it still describes a live window** (reset in the
future). Otherwise (or with sync disabled) the session's own values are used.
For the weekly window, only sessions whose own usage crosses the `weekly_show_at`
threshold contribute to the shared store.

### Storage layout

The store lives under the cache dir resolved by `cache_dir()`:
`$CLAUDEBAR_LIMIT_SYNC_DIR` overrides everything (used by tests); otherwise
`$XDG_CACHE_HOME/claudebar`, falling back to `~/.cache/claudebar`. Returns
`None` when neither override nor `$HOME` is set — callers then no-op.

Per window there is a directory `limit-5h.d` / `limit-7d.d` (via `window_dir`).
Each record is itself a directory (atomic `mkdir`) named
`<reset:%010d>_<pct:%07.3f>` (`entry_name`). The fixed-width fields make lexical
sort match `(reset, pct)` ordering, so the last entry is the highest reset then
the highest pct for that reset. `mkdir` is atomic: concurrent records from
different sessions create distinct names and identical values are idempotent.
`parse_entry` reads names back defensively, returning `None` for anything not
shaped like one of the module's own entries.

### Reading and GC

`latest()` lists the window dir, takes the highest `(reset, pct)` entry, and
`rmdir`s the rest — keeping the store to a single entry per window so the file
count stays bounded across sessions and renders. Read and GC errors are
swallowed: a missing dir yields `None` and a failed `rmdir` is ignored.

### Plausibility guards

`record()` rejects implausibly-far-future resets (corrupt or sentinel values
leaked from input): the 5-hour window resets at most **6 hours** ahead
(`FIVE_HOUR_MAX_AHEAD_SECS`), the 7-day window at most **8 days** ahead
(`SEVEN_DAY_MAX_AHEAD_SECS`). It also no-ops for a non-finite or out-of-range
`pct` (outside `0.0..=999.0`). All filesystem errors are swallowed — the cache
is best-effort and never breaks rendering.

## Failure semantics and invariants

- The segment never breaks rendering: absent fields degrade to `None`, wrong
  types are forgiven by `Coerce`, and the whole segment returns `false` (renders
  nothing) when neither window has anything to show (`empty_input_renders_nothing`).
- The weekly window only ever appears once `pct >= th.weekly_show_at` and is
  joined to the 5-hour window by `window_gap()`.
- Percentages over 999 are never trusted (timestamp-leak rejection); over-limit
  values up to 999 still render with crit colors.
- The sync store is best-effort: every FS error is swallowed, so a corrupt or
  unwritable cache degrades rendering gracefully to each session's own numbers.

## Configuration reference

| Key | Default | Meaning |
| --- | --- | --- |
| `thresholds.warn` | `50` | Bar turns warn-colored at/above this % (5h window) |
| `thresholds.crit` | `80` | Bar turns crit-colored at/above this % |
| `thresholds.weekly_show_at` | `75` | Weekly window shown once usage reaches this % |
| `thresholds.bar_width` | `6` | Width in cells of every progress bar |
| `thresholds.limit_sync` | `false` | Enable cross-session rate-limit sync (opt-in) |
| `CLAUDEBAR_LIMIT_SYNC_DIR` | — | Override cache dir for the sync store |

## Focused tests

- `src/segment/rate_limits.rs::tests`: weekly hidden below `show_at`, shown at/above
  it, gap uses dim color, over-limit renders with crit color, leaked-timestamp pct
  rejected, reset-only window renders, past reset shows pct without countdown,
  empty input renders nothing, and `limit_sync_shows_highest_seen_across_sessions`
  (which points the store at a unique temp dir and verifies a 20% session reflects
  another session's shared 80% high-water mark).
- `src/segment/limit_sync.rs::tests`: record→read round-trip, keeps highest pct for
  a window, empty store returns `None`, far-future resets rejected (both windows),
  GC leaves exactly one entry, identical records collapse to one via `mkdir`
  idempotency, and fixed-width entry names round-trip through `parse_entry`.
