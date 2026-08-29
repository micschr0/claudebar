---
type: "Reference"
title: "Cross-session state"
description: "Best-effort local caches that persist across sessions so later renders can account for history: the limit_sync high-water marks, the burn TSV sample cache, the float readout file, and the update-check cache — all written atomically and never allowed to break the render."
tags: [persistence, best-effort, cache, atomicity, rate-limits]
verified:
  - by: openwiki/0.4.0
    at: 2026-08-29T00:17:43.706Z
sources:
  - id: openwiki-source-c5edfb46b7c4acb766451a37
    resource: repo://src/model/config.rs
  - id: openwiki-source-763738302a84ffdcefbc9913
    resource: repo://src/render/float.rs
  - id: openwiki-source-1d33473d874a4090bb6026e0
    resource: repo://src/render/mod.rs
  - id: openwiki-source-451074c03cdeb781fd3dbe8e
    resource: repo://src/segment/burn.rs
  - id: openwiki-source-5bbd145b1c6fe30cc93223a1
    resource: repo://src/segment/limit_sync.rs
  - id: openwiki-source-d4594996ae77710bcd28b71f
    resource: repo://src/segment/rate_limits.rs
  - id: openwiki-source-0ecba5538b5fd9860f10332f
    resource: repo://src/update.rs
generated: {by: "openwiki/0.4.0", at: "2026-08-29T00:17:43.706Z"}
---

# Cross-session state

Several pieces of local state persist across sessions so later renders can
account for history. They are all **best-effort and never break the render**: a
missing or corrupt cache degrades to defaults, and no filesystem access on the
render path ever raises into the status line. They differ in *how* they are
written atomically (rename for the float/update files, `mkdir` for
`limit_sync`, row-cap rewrite for the burn TSV), but share the same
degrade-silently contract.

<!-- openwiki: mermaid parse failed and this diagram was converted to a text fence so it does not break rendering. Fix the diagram source and restore the mermaid fence. Parser error: Heuristic: an unescaped angle bracket inside a label breaks rendering; rephrase the label. -->
```text
flowchart LR
    R["render_line<br>render_with"] --> F["float:<br>write_atomic rename"]
    R --> L["rate-limits:<br>limit_sync mkdir/rmdir"]
    R --> B["burn:<br>TSV append + trim"]
    R --> U["update-notice:<br>write_atomic rename"]
    F --> FS["float_file (~/.claude/claudebar-float.txt)"]
    L --> LS["limit-5h.d / limit-7d.d"]
    B --> BS["burn-5h.tsv"]
    U --> US["update-check.json (beside config)"]
```

## `limit_sync` (cross-session rate-limit sync)

Claude Code only re-renders a session's status line while that session is
active, so each session's 5-hour / 7-day rate-limit numbers are its own
last-seen snapshot. Idle sessions therefore drift, showing stale — and often
divergent — percentages. `limit_sync.rs` shares the **high-water mark** across
sessions on a host; each render records its `(reset, pct)` for a window, and
the displayed value is the highest `pct` any session has seen for the *current*
(highest-reset) window.

- **Opt-in**: gated by `thresholds.limit_sync` (default `false`).
- **Store layout**: `$CLAUDEBAR_LIMIT_SYNC_DIR` (highest priority, used by
  tests) overrides everything; otherwise `$XDG_CACHE_HOME/claudebar`, else
  `$HOME/.cache/claudebar`. Under the cache, per window there is a directory
  `limit-5h.d` / `limit-7d.d`.
- **Record format**: each record is itself a directory named
  `<reset:%010d>_<pct:%07.3f>`. Fixed-width fields make lexical sort match
  `(reset, pct)` ordering, so the lexically-last entry is the high-water mark.
- **Atomicity**: record creation is an atomic `mkdir` — concurrent sessions
  write distinct names and identical values are idempotent (collapse to one).
- **GC**: `latest_*` lists the window dir, takes the lexically-highest entry,
  and `rmdir`s the rest, keeping exactly one entry per window so the file count
  stays bounded across sessions and renders. GC is best-effort (a failed
  `rmdir` is ignored).
- **Plausibility caps**: `record_*` rejects non-finite or out-of-range `pct`
  and implausibly-far-future resets — the 5-hour window resets at most 6 hours
  ahead, the 7-day window at most 8 days ahead (corrupt/sentinel values from
  the input are dropped, not stored).
- **Integration**: `effective_5h`/`effective_7d` in `segment/rate_limits.rs`
  record the session's own snapshot first, then prefer the shared high-water
  mark when it describes a live window (reset in the future); otherwise the
  session's own values are used. For the 7-day window only sessions whose
  weekly usage crosses `thresholds.weekly_show_at` contribute to the store.

## `burn` TSV sample cache

The burn segment projects a range-to-empty ETA from recent usage by fitting a
linear regression over a sample history. Each render where `burn` is in the
segment list appends a `(now, pct, resets_at)` sample to a local TSV cache.

- **File**: `burn-5h.tsv` under `$XDG_CACHE_HOME/claudebar`, else
  `~/.cache/claudebar`. Override with `CLAUDEBAR_BURN_FILE` (used by tests).
  There is **no** separate 7-day TSV — the 7-day window is shown statelessly
  (color from pct/time-to-reset) and does not drive the regression.
- **Rows**: TSV lines `{now}\t{pct:.3}\t{resets_at}`; capped at `MAX_ROWS`
  (1500). Writes append then `trim_file` keeps only the last `MAX_ROWS` rows.
- **Lookback**: `read_samples` returns only samples with `t >= now - lookback`,
  where lookback is `thresholds.burn_lookback` (default 600s / 10 min).
- **Rejection**: a sample is only appended when the 5-hour window's `resets_at`
  is no more than 6h in the future (same corrupt/sentinel guard as `limit_sync`).
- **Degrade**: a missing/unreadable file yields an empty sample set, which
  renders the `warming` state rather than erroring.
- **States**: `warming` (no samples yet), `idle` (slope ≤ 0), `active`
  (slope > 0, colored ETA: red if it would run dry before reset, yellow within
  20% margin, green otherwise).

## Float readout file

The float is a plain-text side output of the status render: on every render,
when `thresholds.float` is enabled, `render_line` calls
`float::emit_float`, which renders the segments named in
`thresholds.float_segments` as ANSI-free, icon-free text joined by
`float_sep` and writes the result to `float_file`. This is the seam for piping
the status line to tmux, a terminal status bar, or a menu-bar app.

- **Defaults**: `float` off, `float_segments` `"model context cost"`,
  `float_sep` `"  ·  "`, `float_file` `"~/.claude/claudebar-float.txt"`.
- **Path resolution**: a leading `~` expands to `$HOME`; a `~` without a
  resolvable `$HOME` makes the write a no-op rather than guessing.
- **Rendering**: it reuses the exact same `Segment` implementations as the
  colored path (no second code path), rendered with the ASCII style
  (icons off) and then stripped of ANSI via `strip_ansi`, so the output is
  stable regardless of the user's theme/style. Unknown segment names in
  `float_segments` are silently dropped. The float never carries the update
  badge (that belongs on the status line only).
- **Atomic write**: `write_atomic` creates parent dirs, writes to a sibling
  temp file tagged with the process id (`.{name}.{pid}.tmp`) so concurrent
  writers do not clobber each other's staging files, then renames over the
  target (a same-filesystem rename is atomic on POSIX). Any I/O error is
  silently swallowed.

## Update-check cache

The update-notice badge and the background version check share one persisted
cache, `update-check.json` placed **beside the config file**
(`Config::default_path()` directory) so XDG path resolution stays in one place.

- **Schema**: `{"checked_at": <epoch>, "latest": "<calver>" | null}`; `latest`
  is `null` when a check failed, so the retry interval still applies.
- **Render hot path never touches the network**: `render_line` calls
  `refresh_in_background`, which — only when the cache is missing or older than
  `REFRESH_INTERVAL` (24h) — claims the slot atomically with the still-known
  version, then spawns a detached `claudebar update --check` subprocess. The
  caller never waits and never sees output; the badge reads the same cached
  value. Without a usable/writable cache path nothing is spawned (no way to
  rate-limit).
- **Atomic write**: it reuses the float module's `write_atomic`, so a torn read
  degrades to "no cache" and merely schedules a redundant check rather than
  crashing. Read failures, missing files, and malformed JSON all yield `None`.

## Shared invariants

- **Never break the render**: every read failure (missing file, unreadable,
  corrupt) on each cache degrades to its default. No filesystem error surfaces
  into the status line.
- **Atomic writes**: temp-file + rename (float, update cache), atomic `mkdir`
  + GC via `rmdir` (`limit_sync`), append + row-cap rewrite (burn).
- **No secrets stored**: all four caches hold only display/projection data
  (percentages, resets, sample rows, segment text, version strings) — nothing
  sensitive.
- **Bounded storage**: `limit_sync` GC keeps one record per window; burn trims
  to `MAX_ROWS`; the float and update caches are single fixed files.
