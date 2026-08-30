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
//! ## Storage
//!
//! One file per window under the cache dir (`$CLAUDEBAR_LIMIT_SYNC_DIR`, else
//! `$XDG_CACHE_HOME/claudebar`, else `~/.cache/claudebar`): `limit-5h` and
//! `limit-7d`, each a single line `<reset>\t<pct>`. Writes go through the
//! crate's atomic writer (temp file + rename), so a concurrent reader sees
//! either the old contents or the new, never a torn line.
//!
//! Recording is read-modify-write and keeps the higher `(reset, pct)`. Two
//! sessions writing at the same instant can therefore lose the higher value:
//! both read the old one, and the lower write can land last. That is benign
//! and self-correcting — the session holding the higher number re-records it
//! on its next render, which is at most one status-line refresh away. The
//! previous design encoded each record as its own directory name and relied on
//! `mkdir` atomicity to avoid this, at the cost of a listing, a lexical-sort
//! encoding, and a garbage-collection sweep on every read.
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
    crate::paths::cache_dir()
}

/// The file holding `window`'s high-water mark.
fn window_file(cache: &Path, window: &str) -> PathBuf {
    cache.join(format!("limit-{window}"))
}

/// Parse a stored line back into `(reset, pct)`. `None` for anything not
/// shaped like one of our own writes.
fn parse_line(line: &str) -> Option<(i64, f64)> {
    let (r, p) = line.trim().split_once('\t')?;
    let reset = r.parse::<i64>().ok()?;
    let pct = p.parse::<f64>().ok()?;
    if !pct.is_finite() {
        return None;
    }
    Some((reset, pct))
}

/// Read `window`'s stored `(reset, pct)`, or `None` when absent or unreadable.
fn read(cache: &Path, window: &str) -> Option<(i64, f64)> {
    parse_line(&fs::read_to_string(window_file(cache, window)).ok()?)
}

/// Record a `(pct, resets_at)` snapshot for `window` under `cache`, keeping
/// whichever `(reset, pct)` is higher.
///
/// No-op (rather than an error) when the value is implausible: a non-finite or
/// out-of-range `pct`, or a `resets_at` more than `max_ahead` in the future.
/// Filesystem errors are swallowed — a render that cannot write simply does not
/// contribute to the shared store.
fn record(cache: &Path, window: &str, now: i64, pct: f64, resets_at: i64, max_ahead: i64) {
    if !pct.is_finite() || !(0.0..=999.0).contains(&pct) {
        return;
    }
    if resets_at > now.saturating_add(max_ahead) {
        return;
    }
    // Nothing to do when the stored mark already covers this snapshot; skipping
    // the write also keeps idle sessions from touching the file every render.
    if let Some(stored) = read(cache, window)
        && stored >= (resets_at, pct)
    {
        return;
    }
    let _ = crate::render::float::write_atomic(
        &window_file(cache, window),
        &format!("{resets_at}\t{pct:.3}\n"),
    );
}

/// The highest `(pct, resets_at)` recorded for `window` under `cache`.
fn latest(cache: &Path, window: &str) -> Option<(f64, i64)> {
    read(cache, window).map(|(reset, pct)| (pct, reset))
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

    /// The store keeps the *current* window: a later reset wins even when its
    /// percentage is lower, so a fresh window is not masked by the previous
    /// window's high-water mark.
    #[test]
    fn a_newer_window_supersedes_the_previous_high_water_mark() {
        let dir = unique_temp_dir();
        let now = 1_700_000_000;
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
        let _ = fs::remove_dir_all(&dir);
    }

    /// A lower percentage within the same window must not overwrite a higher one.
    #[test]
    fn a_lower_percentage_does_not_lower_the_mark() {
        let dir = unique_temp_dir();
        let now = 1_700_000_000;
        let reset = now + 3600;
        record(&dir, "5h", now, 80.0, reset, FIVE_HOUR_MAX_AHEAD_SECS);
        record(&dir, "5h", now, 30.0, reset, FIVE_HOUR_MAX_AHEAD_SECS);
        assert_eq!(latest(&dir, "5h"), Some((80.0, reset)));
        let _ = fs::remove_dir_all(&dir);
    }

    /// One file per window, no matter how many records land.
    #[test]
    fn the_store_stays_one_file_per_window() {
        let dir = unique_temp_dir();
        let now = 1_700_000_000;
        for i in 0..25 {
            record(
                &dir,
                "5h",
                now,
                f64::from(i),
                now + 3600,
                FIVE_HOUR_MAX_AHEAD_SECS,
            );
        }
        let entries = fs::read_dir(&dir).map(std::fs::ReadDir::count).unwrap_or(0);
        assert_eq!(entries, 1, "expected exactly one file, found {entries}");
        let _ = fs::remove_dir_all(&dir);
    }

    /// Garbage in the file reads as "no mark" rather than panicking or
    /// resurrecting a bogus percentage.
    #[test]
    fn a_corrupt_file_reads_as_absent() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        for junk in [
            "",
            "not a line",
            "abc\tdef",
            "1700000000",
            "1700000000\tNaN",
        ] {
            fs::write(window_file(&dir, "5h"), junk).unwrap();
            assert_eq!(
                latest(&dir, "5h"),
                None,
                "junk {junk:?} should read as absent"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }

    /// A reader never observes a half-written line: `write_atomic` renames a
    /// complete temp file into place.
    ///
    /// Threads rather than processes on purpose, and this is the *harsher*
    /// case: `write_atomic` names its temp file after the pid, so eight threads
    /// in one process all contend for the same temp path — something separate
    /// claudebar renders never do. Even then the rename publishes a whole line,
    /// which is the property under test.
    #[test]
    fn concurrent_writers_leave_the_file_parseable() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        let now = 1_700_000_000;
        let reset = now + 3600;

        std::thread::scope(|scope| {
            for t in 0..8 {
                let dir = dir.clone();
                scope.spawn(move || {
                    for i in 0..40 {
                        let pct = f64::from((t * 40 + i) % 100);
                        record(&dir, "5h", now, pct, reset, FIVE_HOUR_MAX_AHEAD_SECS);
                        // Every read must land on a complete record.
                        if let Some(raw) = read(&dir, "5h") {
                            assert!(raw.1.is_finite(), "torn read: {raw:?}");
                        }
                    }
                });
            }
        });

        let (pct, r) = latest(&dir, "5h").expect("a mark must survive the race");
        assert_eq!(r, reset);
        assert!((0.0..=99.0).contains(&pct), "unexpected pct {pct}");
        let _ = fs::remove_dir_all(&dir);
    }

    /// The deliberate trade this storage makes, stated as a test rather than
    /// left as a surprise.
    ///
    /// Recording is read-modify-write, so two writers that both read the same
    /// old value can let the lower one land last. The previous directory-based
    /// store could not lose an update this way. It is accepted because the loss
    /// is transient: the session holding the higher number re-records it on its
    /// next render.
    #[test]
    fn a_lost_update_is_recovered_by_the_next_record() {
        let dir = unique_temp_dir();
        let now = 1_700_000_000;
        let reset = now + 3600;

        record(&dir, "5h", now, 80.0, reset, FIVE_HOUR_MAX_AHEAD_SECS);
        // Simulate the racing writer that read before the 80 landed.
        let _ = crate::render::float::write_atomic(
            &window_file(&dir, "5h"),
            &format!("{reset}\t{:.3}\n", 30.0),
        );
        assert_eq!(
            latest(&dir, "5h"),
            Some((30.0, reset)),
            "the update was lost"
        );

        // The next render from the busy session restores it.
        record(&dir, "5h", now, 80.0, reset, FIVE_HOUR_MAX_AHEAD_SECS);
        assert_eq!(latest(&dir, "5h"), Some((80.0, reset)), "and recovered");
        let _ = fs::remove_dir_all(&dir);
    }
}
