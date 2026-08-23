# Rate-limits segment

Rate-limit windows with reset countdowns.

- 5h window: pct + countdown via `fmt_reset` (injected `now`).
- Weekly window: shown when pct ≥ `weekly_show_at`; joined via `window_gap`.
- `effective_5h`/`effective_7d` integrate `limit_sync` (cross-session
  high-water marks) — see cross-session-state.
- Leaked-timestamp rejection: timestamps in the future/implausible are
  ignored.
- Emits nothing when no rate-limit data.
- Source: `src/segment/rate_limits.rs`, `src/segment/limit_sync.rs`.
- Tests: window joining, reset formatting, limit_sync
  (`limit_sync_shows_highest_seen_across_sessions`), leaked timestamps.