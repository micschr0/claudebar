//! Cross-session rate-limit sync store.
//!
//! Claude Code only re-renders a session's statusline while that session is
//! active, so each session's 5-hour / 7-day rate-limit numbers are its own
//! last-seen snapshot. Idle sessions therefore drift, showing stale — and
//! often divergent — percentages.
//!
//! This module shares the high-water mark across sessions on a host. Each
//! render records its `(reset, pct)` for a window; the displayed value is the
//! highest `pct` any session has seen for the *current* window (the highest
//! reset), so an idle session reflects another session's heavier usage.
//!
//! ## Storage layout
//!
//! The store lives under the cache dir (`$CLAUDEBAR_LIMIT_SYNC_DIR`, else
//! `$XDG_CACHE_HOME/claudebar`, else `~/.cache/claudebar`). Per window there is
//! a directory `limit-5h.d` / `limit-7d.d`; each record is itself a directory
//! (atomic `mkdir`) named `<reset:%010d>_<pct:%07.3f>`:
//!
//! - Fixed-width fields make lexical sort match `(reset, pct)` ordering, so the
//!   last entry is the highest reset, then the highest pct for that reset.
//! - `mkdir` is atomic: concurrent records from different sessions create
//!   distinct names and are idempotent for identical values.
//!
//! ## Reading & GC
//!
//! `latest_*` lists the window dir, takes the lexically-last (highest) entry,
//! and `rmdir`s the rest — keeping the store to a single entry per window so
//! the file count stays bounded across sessions and renders.
//!
//! ## Plausibility
//!
//! `record_*` rejects implausibly-far-future resets (corrupt or sentinel values
//! leaked from the input): the 5-hour window resets at most 6 hours ahead, the
//! 7-day window at most 8 days ahead. All filesystem errors are swallowed — the
//! cache is best-effort and never breaks rendering.

use std::fs;
use std::path::{Path, PathBuf};

/// A 5-hour window resets no more than 6 hours ahead of `now`.
const FIVE_HOUR_MAX_AHEAD_SECS: i64 = 6 * 60 * 60;
/// A 7-day window resets no more than 8 days ahead of `now`.
const SEVEN_DAY_MAX_AHEAD_SECS: i64 = 8 * 24 * 60 * 60;

/// Resolve the sync store directory.
///
/// `$CLAUDEBAR_LIMIT_SYNC_DIR` overrides everything (useful for tests);
/// otherwise `$XDG_CACHE_HOME/claudebar`, falling back to `$HOME/.cache/claudebar`.
/// Returns `None` when neither override nor `$HOME` is set — callers then no-op.
fn cache_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("CLAUDEBAR_LIMIT_SYNC_DIR")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
    {
        return Some(d);
    }
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("claudebar"))
}

/// The window subdirectory (`limit-5h.d` / `limit-7d.d`) under `cache`.
fn window_dir(cache: &Path, window: &str) -> PathBuf {
    cache.join(format!("limit-{window}.d"))
}

/// Encode a record as a fixed-width entry name `<reset:%010d>_<pct:%07.3f>`.
///
/// Fixed width means lexical sort matches `(reset, pct)` ordering, so the last
/// entry is the highest reset, then the highest pct for that reset.
fn entry_name(reset: i64, pct: f64) -> String {
    format!("{reset:010}_{pct:07.3}")
}

/// Parse an entry name back into `(reset, pct)`. Returns `None` for anything
/// not shaped like one of our own entries (defensive — the only writer is us).
fn parse_entry(name: &str) -> Option<(i64, f64)> {
    let (r, p) = name.split_once('_')?;
    let reset = r.parse::<i64>().ok()?;
    let pct = p.parse::<f64>().ok()?;
    Some((reset, pct))
}

/// Record a `(pct, resets_at)` snapshot for `window` under `cache`.
///
/// No-op (rather than an error) when the value is implausible: a non-finite or
/// out-of-range `pct`, or a `resets_at` more than `max_ahead` in the future
/// (corrupt/sentinel). `mkdir` is atomic, so concurrent records with distinct
/// names never collide and identical values are idempotent.
fn record(cache: &Path, window: &str, now: i64, pct: f64, resets_at: i64, max_ahead: i64) {
    if !pct.is_finite() || !(0.0..=999.0).contains(&pct) {
        return;
    }
    if resets_at > now.saturating_add(max_ahead) {
        return;
    }
    let dir = window_dir(cache, window);
    // Ignore errors: the cache is best-effort. A missing parent or a permission
    // failure simply means this render doesn't contribute to the shared store.
    let _ = fs::create_dir_all(&dir);
    let _ = fs::create_dir(dir.join(entry_name(resets_at, pct)));
}

/// The highest `(pct, resets_at)` recorded for `window` under `cache`, or `None`.
///
/// Lists the window dir, takes the lexically-last (highest) entry, and garbage
/// collects the rest so the store stays at a single entry. Read and GC errors
/// are swallowed: a missing dir yields `None`, and a failed `rmdir` is ignored.
fn latest(cache: &Path, window: &str) -> Option<(f64, i64)> {
    let dir = window_dir(cache, window);
    let mut names: Vec<String> = Vec::new();
    let mut best: Option<(i64, f64, String)> = None;
    for entry in fs::read_dir(&dir).ok()?.flatten() {
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        names.push(name.to_owned());
        if let Some((reset, pct)) = parse_entry(name) {
            let is_best = match &best {
                Some((br, bp, _)) => (reset, pct) > (*br, *bp),
                None => true,
            };
            if is_best {
                best = Some((reset, pct, name.to_owned()));
            }
        }
    }
    let (best_reset, best_pct, best_name) = best?;
    // GC: keep only the high-water mark; rmdir everything else (best-effort).
    for name in &names {
        if name != &best_name {
            let _ = fs::remove_dir(dir.join(name));
        }
    }
    Some((best_pct, best_reset))
}

/// Record this session's 5-hour `(pct, resets_at)` snapshot.
///
/// No-op when `pct` is non-finite/out of range or `resets_at` is implausibly far
/// in the future (more than 6 hours ahead), or when no cache dir can be resolved.
pub fn record_5h(now: i64, pct: f64, resets_at: i64) {
    if let Some(cache) = cache_dir() {
        record(&cache, "5h", now, pct, resets_at, FIVE_HOUR_MAX_AHEAD_SECS);
    }
}

/// Record this session's 7-day `(pct, resets_at)` snapshot.
///
/// No-op when `pct` is non-finite/out of range or `resets_at` is implausibly far
/// in the future (more than 8 days ahead), or when no cache dir can be resolved.
pub fn record_7d(now: i64, pct: f64, resets_at: i64) {
    if let Some(cache) = cache_dir() {
        record(&cache, "7d", now, pct, resets_at, SEVEN_DAY_MAX_AHEAD_SECS);
    }
}

/// The highest `(pct, resets_at)` any session has seen for the 5-hour window, or
/// `None` when the store is empty or unreadable.
#[must_use]
pub fn latest_5h() -> Option<(f64, i64)> {
    cache_dir().and_then(|c| latest(&c, "5h"))
}

/// The highest `(pct, resets_at)` any session has seen for the 7-day window, or
/// `None` when the store is empty or unreadable.
#[must_use]
pub fn latest_7d() -> Option<(f64, i64)> {
    cache_dir().and_then(|c| latest(&c, "7d"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unique temp dir; nanos + pid keep parallel test runs from colliding.
    fn unique_temp_dir() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!(
            "claudebar-limit-sync-{}-{}",
            std::process::id(),
            nanos,
        ))
    }

    #[test]
    fn record_then_read_roundtrips() {
        let dir = unique_temp_dir();
        let now = 1_700_000_000;
        record(&dir, "5h", now, 48.0, now + 3600, FIVE_HOUR_MAX_AHEAD_SECS);
        assert_eq!(latest(&dir, "5h"), Some((48.0, now + 3600)));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn keeps_highest_pct_for_window() {
        let dir = unique_temp_dir();
        let now = 1_700_000_000;
        let reset = now + 3600;
        // Three sessions, same window, different usage → the highest wins.
        record(&dir, "5h", now, 48.0, reset, FIVE_HOUR_MAX_AHEAD_SECS);
        record(&dir, "5h", now, 80.0, reset, FIVE_HOUR_MAX_AHEAD_SECS);
        record(&dir, "5h", now, 60.0, reset, FIVE_HOUR_MAX_AHEAD_SECS);
        assert_eq!(latest(&dir, "5h"), Some((80.0, reset)));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn latest_returns_none_for_empty_store() {
        let dir = unique_temp_dir();
        assert_eq!(latest(&dir, "5h"), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_far_future_reset_5h() {
        let dir = unique_temp_dir();
        let now = 1_700_000_000;
        // 7h ahead exceeds the 6h 5-hour cap → nothing is recorded.
        record(
            &dir,
            "5h",
            now,
            90.0,
            now + 7 * 3600,
            FIVE_HOUR_MAX_AHEAD_SECS,
        );
        assert_eq!(latest(&dir, "5h"), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_far_future_reset_7d() {
        let dir = unique_temp_dir();
        let now = 1_700_000_000;
        // 9 days ahead exceeds the 8-day 7-day cap → nothing is recorded.
        record(
            &dir,
            "7d",
            now,
            90.0,
            now + 9 * 24 * 3600,
            SEVEN_DAY_MAX_AHEAD_SECS,
        );
        assert_eq!(latest(&dir, "7d"), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn gc_removes_old_window_entries() {
        let dir = unique_temp_dir();
        let now = 1_700_000_000;
        // An older window (lower reset, higher pct) then the current window
        // (higher reset). The current window's entry wins; the stale 80% is GC'd.
        record(&dir, "5h", now, 80.0, now + 3600, FIVE_HOUR_MAX_AHEAD_SECS);
        record(
            &dir,
            "5h",
            now,
            30.0,
            now + 2 * 3600,
            FIVE_HOUR_MAX_AHEAD_SECS,
        );
        assert_eq!(latest(&dir, "5h"), Some((30.0, now + 2 * 3600)));
        // After GC exactly one entry remains; a second read is stable.
        let remaining = fs::read_dir(window_dir(&dir, "5h"))
            .map(std::fs::ReadDir::count)
            .unwrap_or(0);
        assert_eq!(remaining, 1, "GC should leave exactly one entry");
        assert_eq!(latest(&dir, "5h"), Some((30.0, now + 2 * 3600)));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn record_is_idempotent_for_identical_values() {
        let dir = unique_temp_dir();
        let now = 1_700_000_000;
        let reset = now + 3600;
        // Recording the same (reset, pct) twice yields one entry (mkdir is
        // idempotent) and a stable latest.
        record(&dir, "7d", now, 55.0, reset, SEVEN_DAY_MAX_AHEAD_SECS);
        record(&dir, "7d", now, 55.0, reset, SEVEN_DAY_MAX_AHEAD_SECS);
        assert_eq!(latest(&dir, "7d"), Some((55.0, reset)));
        let remaining = fs::read_dir(window_dir(&dir, "7d"))
            .map(std::fs::ReadDir::count)
            .unwrap_or(0);
        assert_eq!(remaining, 1, "identical records should collapse to one");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn entry_name_roundtrips_through_parse() {
        // Fixed-width names must round-trip and sort numerically.
        let a = entry_name(1_700_000_000, 48.0);
        let b = entry_name(1_900_000_000, 80.0);
        assert!(a < b, "lexical sort must match numeric sort: {a:?} < {b:?}");
        assert_eq!(parse_entry(&a), Some((1_700_000_000, 48.0)));
        assert_eq!(parse_entry(&b), Some((1_900_000_000, 80.0)));
    }
}
