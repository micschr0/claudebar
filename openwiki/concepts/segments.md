---
type: concept
title: "Segments: the composable statusline units"
description: "How the 12 renderable statusline segments work — the Segment trait seam, the injected RenderCtx, the SegmentWriter emission API, SegmentKind enable/order semantics, and each segment's contract."
tags: [segments, statusline, segment-trait, renderctx, segmentwriter, segmentkind, render, update-notice]
sources:
  - id: openwiki-source-c5edfb46b7c4acb766451a37
    resource: repo://src/model/config.rs
  - id: openwiki-source-1d33473d874a4090bb6026e0
    resource: repo://src/render/mod.rs
  - id: openwiki-source-d977bd28254dbfcf5d7fe3bb
    resource: repo://src/render/writer.rs
  - id: openwiki-source-3a6ff89030cacfe8ee730edf
    resource: repo://src/sanitize.rs
  - id: openwiki-source-451074c03cdeb781fd3dbe8e
    resource: repo://src/segment/burn.rs
  - id: openwiki-source-f2f421f1f8e75ffcc5953da6
    resource: repo://src/segment/clock.rs
  - id: openwiki-source-5e9d0d3a793adb710baab135
    resource: repo://src/segment/context.rs
  - id: openwiki-source-55299418ce7215e62b1fc874
    resource: repo://src/segment/cost.rs
  - id: openwiki-source-caed7c213e3bf4afc1854652
    resource: repo://src/segment/dev_context.rs
  - id: openwiki-source-36efb8afd8dc119a86c1105b
    resource: repo://src/segment/directory.rs
  - id: openwiki-source-4f00275a3586819a520cbe3d
    resource: repo://src/segment/duration.rs
  - id: openwiki-source-0700af36d25875a20e6db044
    resource: repo://src/segment/git.rs
  - id: openwiki-source-5bbd145b1c6fe30cc93223a1
    resource: repo://src/segment/limit_sync.rs
  - id: openwiki-source-ac5b3d1ae4cbd2dfd024bd54
    resource: repo://src/segment/lines.rs
  - id: openwiki-source-f4acb0adeb73aa0d54303a27
    resource: repo://src/segment/mod.rs
  - id: openwiki-source-fce92f63c4ad0d6ac9786bac
    resource: repo://src/segment/model.rs
  - id: openwiki-source-d4594996ae77710bcd28b71f
    resource: repo://src/segment/rate_limits.rs
  - id: openwiki-source-5d948000f74b098b1187bdc9
    resource: repo://src/segment/update_notice.rs
generated: {by: "openwiki/0.4.0", at: "2026-08-29T00:17:43.706Z"}
verified:
  - by: openwiki/0.4.0
    at: 2026-08-29T00:17:43.706Z
---

# Segments: the composable statusline units

The statusline is composed from **segments** — small, self-contained units that
each render one piece of Claude Code session state (directory, git branch, token
usage, rate limits, cost, time, a pending update, and so on). A segment
implements the `Segment` trait, receives an injected `RenderCtx` and writes its
colored spans into a `SegmentWriter`. The process of building one status line
from the enabled segments (choosing separators, layout, and the single render
entrypoint) is documented in
[render-pipeline](/openwiki/architecture/render-pipeline.md); this page is the
contract each segment must satisfy and how to add a new one.

## The `Segment` trait seam

The extension point is defined in `src/segment/mod.rs`:

```rust
pub trait Segment {
    /// Write this segment's body into `out`. Return `true` iff anything was
    /// emitted (an empty/absent segment returns `false` and is skipped, taking
    /// its separator with it).
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool;
}
```

Every segment is a **zero-sized struct**, resolved to a `&'static dyn Segment`
via `SegmentKind::as_segment()` (a `match` over all 12 variants). A segment
never knows its neighbors, never emits a separator, and never embeds a raw
ANSI/color code — it reads `ctx.theme` / `ctx.style` through the writer. This
is what keeps segments independently testable and composable.

## `RenderCtx` — the injected context

The `RenderCtx<'a>` bundle (`src/segment/mod.rs`) carries everything a segment
is allowed to see. **All ambient state is injected — never read from the
environment inside a segment** — which is what makes rendering deterministic
and testable:

- `input: &InputData` — the parsed hook JSON (`cwd`, `context_window`,
  `rate_limits`, `cost`, `model`, `effort`, `pr`, `agent`, …).
- `theme: &Theme` and `style: &Style` — the resolved color palette and glyph set.
- `th: &Thresholds` — warn/crit bands, `bar_width`, `cost_decimals`,
  `clock_mode`, `burn_lookback`, `weekly_show_at`, `limit_sync`, etc.
- `now: i64` — current epoch seconds (for reset countdowns and the clock).
- `home: Option<&str>` — `$HOME`, for `~` path abbreviation.
- `tz_offset_seconds: i32` — local timezone offset in seconds east of UTC
  (0 = UTC, the fallback when detection fails or for the TUI preview).
- `update: Option<&Version>` — a cached release strictly newer than this binary,
  only present when the `update-notice` segment is enabled; resolved **once per
  render** in `render::render_with` / `render::render_line`, never inside a
  segment.

`now`, `home`, `tz_offset_seconds`, and `update` are injected at the boundary
(see [render-pipeline](/openwiki/architecture/render-pipeline.md)). The clock
is the one segment that reads locale/the system only *through*
`tz_offset_seconds` and a `LazyLock`-cached 12h/24h preference — it never
shells out.

## `SegmentWriter` — the single emission point

Segments write into a `SegmentWriter` (`src/render/writer.rs`) that owns the
active theme × style and resolves colors/glyphs internally, so a segment never
hardcodes a color or decides whether icons render. Key methods:

- `colored(color, text)` / `colored_fmt(color, args)` / `dim(text)` — painted
  runs, always terminated by a reset (`RESET`).
- `colored_with(color, f)` — a closure body inside a color span; it restores an
  outer active color on pop so nested `colored_with`/`icon` runs nest correctly.
- `icon(glyph)` — a dim-colored leading glyph plus a trailing space, but only
  when the active style has `icons` enabled; with icons off it is a no-op.
- `raw(text)` / `raw_fmt(args)` — verbatim text the caller has already formed.
- `bar_pct(pct, width, color)` — a progress bar (via `render::bar::write_bar`)
  followed by `" <pct>%"`; the bar-fill/percent gap convention is owned here.
- `window_gap()` — a dim-colored glyph joining two related windows *inside* one
  segment, deliberately lighter than the composer's inter-segment separator.

Host-provided strings must be pre-sanitized with
[`strip_control`](/openwiki/concepts/security-and-sanitization.md) before
passing to the writer; the writer emits them verbatim.

## `SegmentKind` — which segments, and in what order

`Vec<SegmentKind>` in `Config` (`src/model/config.rs`) encodes **both** which
segments are enabled (presence) and their render order. The enum is serde
kebab-case (`rate-limits`, `dev-context`, `update-notice`), parsed back from a
name via `SegmentKind::from_kebab` (which reuses serde's rename mapping). Two
canonical arrays exist:

- `SegmentKind::ALL` — all 12 segments in canonical order:
  `Directory, Git, Model, Context, RateLimits, DevContext, Cost, Lines,
  Duration, Burn, Clock, UpdateNotice`.
- `SegmentKind::DEFAULT` — the 8-segment default used by `Config::default`
  (and therefore by config-less operation): `Directory, Git, Model, Context,
  Lines, RateLimits, Cost, Duration`. It **deliberately differs** from `ALL`
  order (placing `Lines` between `Context` and `RateLimits`) to mirror the bash
  statusline's segment order; this parity is guarded by the
  `default_matches_bash_layout` test, which also locks the default Tokyo Night +
  Powerline theme/style.

Adding a new segment means adding a `SegmentKind` variant (and its kebab name /
label) and registering it in `as_segment()`.

## Composition and separators

`render::render_fixed` (`src/render/mod.rs`) loops over `cfg.segments`, builds a
fresh `SegmentWriter` per segment, calls `render`, and — only when the segment
returned `true` **and** the writer is non-empty — appends it. A separator is
placed between two *adjacent non-empty* segments, so an empty/absent segment
takes its separator with it (there is never a dangling separator). The `auto`
layout path pre-collects only the non-empty segments before wrapping, so empty
segments cost neither a line nor a separator. The `Segments never emit a
separator` rule means a segment cannot invent a boundary of its own.

## The 12 segments

Each segment below follows a documented contract that mirrors the original bash
statusline block.

### Directory

`src/segment/directory.rs`. Renders the fish-style abbreviated `cwd` (via
`abbreviate_path`), collapsing `$HOME` to `~`, in `theme.dir` with no icon. It
emits a leading space inside the colored run to match bash's `${C_DIR} %s`.
Returns `false` (emits nothing) when `cwd` is absent or empty.

### Git

`src/segment/git.rs`. Only runs when `cwd` is a non-empty **absolute** path
(starts with `/`); otherwise returns `false`. It runs **two** git commands,
both scoped to `cwd`, and suppresses stderr on the first:

1. `git -C <cwd> --no-optional-locks -c gc.auto=0 status --branch --porcelain`
   — gated on non-empty stdout, **never** on exit status (git can print a
   valid `## ` line while exiting non-zero, matching the bash reference).
2. `git -C <cwd> rev-list --walk-reflogs --count refs/stash` with
   `GIT_OPTIONAL_LOCKS=0` — run only once (1) has confirmed a branch; its count
   is parsed defensively to 0 on any failure.

`parse_status` handles `## No commits yet on <branch>`, returns `None` for
detached HEAD (`## HEAD (no branch)`), and parses `<branch>...<upstream>
[ahead N, behind M]`; remaining porcelain lines are counted as untracked (`?? `)
or modified. It emits the branch in `theme.git_branch`, the stash flag when
`count > 0`, then `↑N` / `↓M` / `MN` / `?N` markers only when non-zero, and
strips control bytes from the branch name.

### Model

`src/segment/model.rs`. Renders model name and effort level side by side:
model icon (dim) + name in `theme.model`, then (after a separating space) an
effort icon + the level string colored by value — `high` → `bar_ok`, `xhigh` →
`bar_warn`, `max` → `theme.effort`, and **any other value** (including unknown
strings) → `theme.dim`. Effort is *absent* for models without the param, so the
segment gates on presence, never on a specific value. Emits nothing when both
name and level are absent.

### Context

`src/segment/context.rs`. Renders the session token count plus (when a usable
`used_percentage` is present) a usage bar and percent. The count is
`total_input_tokens + total_output_tokens` formatted via `fmt_tokens` and always
renders — **even at zero tokens**, so new users meet the segment in its most
benign form. When present, the percentage is rounded and accepted only when it
lands in `0..=999`; the bar color follows thresholds (`> 100` or `>= crit` →
`bar_crit`, `>= warn` → `bar_warn`, else `bar_ok`). Without a usable percentage
it renders just the token icon + count.

### RateLimits

`src/segment/rate_limits.rs`. One segment rendering both usage windows of the
Claude Code hook: a 5-hour window and a weekly (7-day) window, joined by
`SegmentWriter::window_gap()`. See
[rate-limits](/openwiki/concepts/rate-limits.md) for the full window semantics;
in short the 5-hour window shows whenever a percentage **or** a future
`resets_at` is present, the weekly window only once
`used_percentage >= th.weekly_show_at` (and `<= 999`), each with a reset
countdown via `fmt_reset`. Every percentage passes `pct_in_range` (rounds, then
accepts only `0..=999`, which rejects a leaked epoch timestamp while still
allowing over-limit values). The rate-limits segment also consults the
cross-session store (see below).

### DevContext

`src/segment/dev_context.rs`. Renders worktree name (from `worktree.name`,
falling back to `workspace.git_worktree`), PR number + review state, and
sub-agent name. Returns `false` when all three are absent. The PR renders
`#<n>` in `theme.git_branch` with a review-state indicator (✓ approved /
✗ changes-requested / ◦ commented / · pending), and the agent name uses
`theme.effort`.

### Cost

`src/segment/cost.rs`. Renders session cost in USD (`$1.23`), with decimal
places from `th.cost_decimals` (clamped to 4). Hides (returns `false`) when the
cost is zero or absent.

### Lines

`src/segment/lines.rs`. Renders lines added/removed this session (`+321 −87`)
in a single `theme.lines` color slot. The `+`/`−` signs carry the visual
distinction, so the segment avoids borrowing the `modified` color (semantically
for modified files). Hides when both counts are zero or absent.

### Duration

`src/segment/duration.rs`. Renders session wall-clock duration (`42s`, `47m`,
`1h02m`) in `theme.duration`. Hides when zero or absent; sub-second durations
render as `0s`.

### Burn

`src/segment/burn.rs`. Range-to-empty projection: on each render where burn is
in the segment list, a `(now, pct, resets_at)` sample is appended to a TSV cache
file (`$XDG_CACHE_HOME/claudebar/burn-5h.tsv`, overridable via
`CLAUDEBAR_BURN_FILE`, trimmed to ~1500 rows). A least-squares linear
regression over `th.burn_lookback` (default 600s) yields a slope
(pct/sec); the ETA is `(100 - current_pct) / slope`. States: `warming` (no
samples yet, `↗ …`), `idle` (slope ≤ 0, `↗ ✓`), or `active`
(`↗ {label} ⇢ {eta}`) colored red/yellow/green by urgency relative to the
window reset. Samples are only recorded when the `resets_at` is within 6 hours
(a plausibility guard against corrupt/sentinel snapshots).

### Clock

`src/segment/clock.rs`. Renders the current time in 12h or 24h from
`ctx.now` + `ctx.tz_offset_seconds` — pure computation with the `time` crate, no
subprocesses. Controlled by `th.clock_mode` (`auto`/`12h`/`24h`/`off`); in
`auto` it consults a `LazyLock`-cached locale preference (`LC_TIME` → `LC_ALL`
→ `LANG`, checked against a country table). `off` or a negative `now` returns
`false`.

### UpdateNotice

`src/segment/update_notice.rs`. Renders a badge (`↑ 2026.8.20`) for a release
newer than the running binary, using the `ahead` glyph in `theme.ahead`.
It is a **pure formatter**: the update check and version comparison happen in
`render_with` / `render_line`, and the segment itself does no I/O, showing
nothing when `ctx.update` is `None`. It is the **only segment that touches the
network** — `render_with` reads the update cache and (in `render_line`) spawns a
detached background check, both **gated on the `update-notice` segment being
enabled**, so a disabled segment keeps the cache read and network activity off
every other render path.

## Cross-session rate-limit sync

The `RateLimits` segment optionally consults the cross-session store in
`src/segment/limit_sync.rs` (see
[cross-session rate sync in rate-limits](/openwiki/concepts/rate-limits.md)).
With `th.limit_sync` enabled, each render records its session's 5-hour snapshot
into a shared cache (using `$CLAUDEBAR_LIMIT_SYNC_DIR` for tests, else
`$XDG_CACHE_HOME/claudebar`, else `~/.cache/claudebar`), and the displayed
value prefers the shared high-water mark when it still describes a live window
(reset in the future). The 7-day path only contributes when its own usage
crosses the weekly show threshold. Records are atomic `mkdir`s named
`<reset:%010d>_<pct:%07.3f>` so lexical sort matches `(reset, pct)` ordering;
`latest_*` keeps the single highest entry and GCs the rest. All filesystem
errors are swallowed — the cache is best-effort and never breaks rendering.

## Adding a segment safely

1. Add a variant to `SegmentKind` (`src/model/config.rs`) with its serde
   kebab-case name and `label()`; add it to `SegmentKind::ALL` (and `DEFAULT`
   only if the bash parity requires it — the `default_matches_bash_layout` test
   locks that set).
2. Create `src/segment/<name>.rs` implementing `Segment` as pure functions over
   `ctx`, writing only through `SegmentWriter`.
3. Register it in `SegmentKind::as_segment()` (`src/segment/mod.rs`).
4. Return `true` **iff** you emitted something, so the composer skips the
   separator for empty segments.
5. Never read env vars or system time, never embed a raw color, never emit a
   separator, and sanitize host-provided strings with `strip_control`.
6. Add `#[cfg(test)]` unit tests — table tests for pure helpers plus a render
   test through `render_with`; add/extend golden coverage in
   `tests/render_golden.rs` if the segment changes fixture output.
