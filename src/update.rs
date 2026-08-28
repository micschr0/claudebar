//! `claudebar update` — a manual, user-triggered version check.
//!
//! Deliberate design choice: the **render hot path never touches the network**.
//! Updating is an explicit command with its own exit codes, so a script or the
//! user can find out whether a newer release exists and how to install it.
//!
//! Version source: the GitHub releases API (`releases?per_page=30`), which is
//! the channel `install.sh` and the Homebrew tap publish to. claudebar ships
//! prereleases (CalVer `...-beta.N`) on a separate channel, so the comparison
//! is **channel-aware**: by default we compare against the newest *stable*
//! release (the default install path). Prereleases are only offered when the
//! user opts in with `--channel beta`.
//!
//! Exit codes (documented convention, useful in scripts):
//! - `0` — up to date (or, with `--check`, the check succeeded)
//! - `1` — could not check (network / parse error)
//! - `2` — an update is available (only without `--check`)

use serde::Deserialize;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use thiserror::Error;

/// Which release channel to compare against. The install path by default
/// targets stable releases; prereleases are opt-in via `--channel beta`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Channel {
    Stable,
    Beta,
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Stable => write!(f, "stable"),
            Channel::Beta => write!(f, "beta"),
        }
    }
}

/// A CalVer release tag, e.g. `2026.8.15` or `2026.8.15-beta.1`.
///
/// Ordering follows semver semantics: within the same `major.minor.patch`,
/// a release beats any prerelease, and prereleases compare field-by-field
/// (`beta.1` < `beta.2`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    prerelease: Option<String>,
}

impl Version {
    /// Parse a CalVer tag. Returns `None` for anything that doesn't match
    /// `N.N.N` with an optional `-prerelease` suffix.
    pub fn parse(s: &str) -> Option<Version> {
        // Split off an optional `-beta.N`-style prerelease suffix.
        let (core, prerelease) = match s.find('-') {
            Some(idx) => (&s[..idx], Some(s[idx + 1..].to_string())),
            None => (s, None),
        };
        let mut parts = core.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None; // more than three numeric components
        }
        Some(Version {
            major,
            minor,
            patch,
            prerelease,
        })
    }

    /// Whether this version is a prerelease (has a `-...` suffix).
    pub fn is_prerelease(&self) -> bool {
        self.prerelease.is_some()
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.prerelease {
            write!(f, "-{pre}")?;
        }
        Ok(())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
            .then_with(|| match (&self.prerelease, &other.prerelease) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater, // release > prerelease
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => cmp_prerelease(a, b),
            })
    }
}

/// Compare two prerelease suffixes. claudebar uses CalVer with a single
/// `-beta.N` shape (`2026.8.15-beta.1`), so we only need numeric beta levels;
/// a missing suffix (a full release) always sorts higher than a prerelease.
/// Kept deliberately simple — see ponytail: handles only `-beta.<u32>`; a
/// different prerelease shape would need the full semver comparison.
fn cmp_prerelease(a: &str, b: &str) -> Ordering {
    let a_num = a.strip_prefix("beta.").and_then(|n| n.parse::<u64>().ok());
    let b_num = b.strip_prefix("beta.").and_then(|n| n.parse::<u64>().ok());
    a_num.cmp(&b_num)
}

/// A single release entry from the GitHub releases API. We only need the tag;
/// whether it's a prerelease is determined by the tag's `-...` suffix.
#[derive(Debug, Deserialize)]
struct Release {
    #[serde(rename = "tag_name")]
    tag_name: String,
}

/// The newest releases we found.
#[derive(Debug, Clone)]
pub struct Latest {
    /// Newest release across all channels (stable and prerelease).
    pub overall: Version,
    /// Newest stable (non-prerelease) release, if any.
    pub stable: Option<Version>,
}

/// Errors that prevent an update check from completing.
#[derive(Debug, Error)]
pub enum UpdateError {
    /// The HTTP fetch failed or `curl` is unavailable.
    #[error("could not reach the release service: {0}")]
    Network(String),
    /// The response body could not be parsed as expected.
    #[error("unexpected release data: {0}")]
    Parse(String),
}

const RELEASES_URL: &str = "https://api.github.com/repos/micschr0/claudebar/releases?per_page=30";

/// Fetch the latest release information from the GitHub releases API.
///
/// Uses `curl` (the same tool `install.sh` relies on) rather than pulling in
/// an HTTP dependency; the render hot path never touches this code.
///
/// # Errors
///
/// Returns [`UpdateError::Network`] when `curl` is unavailable, exits
/// non-zero, or cannot reach the registry within the timeout. Returns
/// [`UpdateError::Parse`] when the response is not a JSON list of releases
/// or contains no parseable CalVer tag.
pub fn fetch_latest() -> Result<Latest, UpdateError> {
    let output = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--max-time",
            "15",
            RELEASES_URL,
        ])
        .output()
        .map_err(|e| UpdateError::Network(e.to_string()))?;

    if !output.status.success() {
        let hint = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(UpdateError::Network(if hint.is_empty() {
            "HTTP request failed".to_string()
        } else {
            hint
        }));
    }

    let releases: Vec<Release> =
        serde_json::from_slice(&output.stdout).map_err(|e| UpdateError::Parse(e.to_string()))?;

    let mut versions: Vec<Version> = Vec::with_capacity(releases.len());
    for r in releases {
        if let Some(v) = Version::parse(&r.tag_name) {
            versions.push(v);
        }
    }

    let overall = versions
        .iter()
        .max()
        .ok_or_else(|| UpdateError::Parse("no parseable releases found".to_string()))?;

    let stable = versions
        .iter()
        .filter(|v| !v.is_prerelease())
        .max()
        .cloned();

    Ok(Latest {
        overall: overall.clone(),
        stable,
    })
}

/// A day between background update checks.
const REFRESH_INTERVAL: i64 = 86_400;

/// The last update check's outcome, as persisted for the render path.
#[derive(Debug, Clone)]
pub struct CachedCheck {
    /// When a check was last started, epoch seconds — stamped before the
    /// network call and again when it finishes, so an offline machine retries
    /// daily instead of on every render.
    pub checked_at: i64,
    /// The newest stable release, if the check succeeded.
    pub latest: Option<Version>,
}

/// Where the update-check cache lives — beside the config file, so XDG
/// resolution stays in exactly one place.
#[must_use]
pub fn cache_path() -> Option<PathBuf> {
    Some(crate::model::Config::default_path()?.with_file_name("update-check.json"))
}

/// Persist an update check. Best-effort: any failure is silently dropped, and
/// `latest = None` records a failed check so the retry interval still applies.
pub fn write_cache(checked_at: i64, latest: Option<&Version>) {
    if let Some(path) = cache_path() {
        let _ = write_cache_at(&path, checked_at, latest);
    }
}

/// Returns whether the cache actually landed on disk. That is the one failure
/// the caller has to react to: an unwritable config directory means the daily
/// backoff can never be recorded.
fn write_cache_at(path: &Path, checked_at: i64, latest: Option<&Version>) -> bool {
    let body = serde_json::json!({
        "checked_at": checked_at,
        "latest": latest.map(ToString::to_string),
    });
    // Reuses the float readout's writer: a torn read here reads as "no cache",
    // which would spawn a redundant check on the next render.
    crate::render::float::write_atomic(path, &body.to_string()).is_ok()
}

/// Read the cache. Missing, unreadable, or malformed all yield `None`, the
/// same degrade-silently contract as `InputData::parse`.
#[must_use]
pub fn read_cache() -> Option<CachedCheck> {
    read_cache_at(&cache_path()?)
}

fn read_cache_at(path: &Path) -> Option<CachedCheck> {
    parse_cache(&std::fs::read_to_string(path).ok()?)
}

fn parse_cache(raw: &str) -> Option<CachedCheck> {
    let v: serde_json::Value = serde_json::from_str(raw).ok()?;
    Some(CachedCheck {
        checked_at: v.get("checked_at")?.as_i64()?,
        latest: v
            .get("latest")
            .and_then(serde_json::Value::as_str)
            .and_then(Version::parse),
    })
}

/// Whether a background check is due: no cache at all, or one older than
/// [`REFRESH_INTERVAL`].
///
/// `saturating_sub` is here for overflow, not for sign: an extreme `checked_at`
/// would otherwise panic on subtraction in debug builds. A stamp in the future
/// yields a negative age, which is simply never greater than the interval — so
/// clock skew suppresses checks until wall-clock time catches up.
fn is_refresh_due(cache: Option<&CachedCheck>, now: i64) -> bool {
    match cache {
        Some(c) => now.saturating_sub(c.checked_at) > REFRESH_INTERVAL,
        None => true,
    }
}

/// Spawn a detached `claudebar update --check` when the cache is missing or
/// older than [`REFRESH_INTERVAL`].
///
/// The caller never waits on the child and never sees its output — the render
/// returns immediately and the cache is updated whenever the child finishes.
/// Without a usable cache path, or when the cache cannot be written, there is
/// no way to rate-limit, so nothing is spawned at all.
pub fn refresh_in_background(now: i64) {
    let Some(path) = cache_path() else {
        return;
    };
    let cached = read_cache_at(&path);
    if !is_refresh_due(cached.as_ref(), now) {
        return;
    }
    // Resolve the child before claiming the slot: a stamp written for a spawn
    // that never happens burns a full day of backoff with no check performed.
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    // Claim the slot before the child starts its network call. `curl` waits up
    // to 15s, and until the child writes, every further render would see the
    // same stale cache and spawn a check of its own. Re-stamping keeps the
    // version already known so the badge survives the claim.
    if !write_cache_at(&path, now, cached.and_then(|c| c.latest).as_ref()) {
        return;
    }
    let _ = Command::new(exe)
        .args(["update", "--check"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

/// The outcome of comparing the installed version against the latest release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recommendation {
    /// Installed version is already the newest available.
    UpToDate,
    /// A newer release exists.
    Update {
        version: Version,
        /// Whether that newest release is a prerelease (beta channel).
        is_beta: bool,
        /// The newest stable release, if any (shown as context).
        stable: Option<Version>,
    },
}

/// Decide what to tell the user given their installed `current` version and
/// the requested [`Channel`].
///
/// On the `Stable` channel we compare against the newest stable release (the
/// default install path) and only fall back to a prerelease if no stable
/// release exists yet. On the `Beta` channel we compare against the newest
/// release across all channels.
pub fn recommend(current: &Version, latest: &Latest, channel: Channel) -> Recommendation {
    let target = match channel {
        Channel::Stable => latest.stable.as_ref().unwrap_or(&latest.overall),
        Channel::Beta => &latest.overall,
    };
    if current >= target {
        return Recommendation::UpToDate;
    }
    Recommendation::Update {
        version: target.clone(),
        is_beta: target.is_prerelease(),
        stable: latest.stable.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    /// Unique temp path; nanos + pid keep parallel test runs from colliding.
    /// No `tempfile` crate — `insta` is the only dev-dependency.
    fn unique_temp_path() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!(
            "claudebar-update-test-{}-{}/update-check.json",
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn cache_file_roundtrip() {
        let path = unique_temp_path();

        // Nothing written yet: reading degrades to None rather than erroring.
        assert!(read_cache_at(&path).is_none());

        // A successful check. The parent directory does not exist yet either.
        assert!(write_cache_at(&path, 1_700_000_000, Some(&v("2026.8.20"))));
        let got = read_cache_at(&path).expect("cache readable");
        assert_eq!(got.checked_at, 1_700_000_000);
        assert_eq!(got.latest, Some(v("2026.8.20")));

        // A failed check overwrites it: stamped, but nothing to show.
        assert!(write_cache_at(&path, 1_700_000_100, None));
        let got = read_cache_at(&path).expect("cache readable");
        assert_eq!(got.checked_at, 1_700_000_100);
        assert_eq!(got.latest, None);

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn cache_file_with_garbage_is_none() {
        let path = unique_temp_path();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "not json").unwrap();
        assert!(read_cache_at(&path).is_none());
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn unwritable_cache_path_reports_failure() {
        // A regular file where the cache's parent directory should be: nothing
        // can be created under it, on any platform and as any user.
        let blocker = unique_temp_path();
        std::fs::create_dir_all(blocker.parent().unwrap()).unwrap();
        std::fs::write(&blocker, "not a directory").unwrap();

        let path = blocker.join("update-check.json");
        assert!(
            !write_cache_at(&path, 1_700_000_000, Some(&v("2026.8.20"))),
            "an unwritable path must report failure, not swallow it"
        );
        // Failing closed matters because the caller uses this to decide whether
        // spawning a check can ever be rate-limited.
        assert!(read_cache_at(&path).is_none());

        let _ = std::fs::remove_dir_all(blocker.parent().unwrap());
    }

    #[test]
    fn refresh_is_due_without_a_cache() {
        assert!(is_refresh_due(None, 0));
    }

    #[test]
    fn refresh_backs_off_for_a_day() {
        let stamped = |at| CachedCheck {
            checked_at: at,
            latest: None,
        };
        let now = 1_700_000_000;
        assert!(!is_refresh_due(Some(&stamped(now)), now));
        assert!(!is_refresh_due(Some(&stamped(now - REFRESH_INTERVAL)), now));
        assert!(is_refresh_due(
            Some(&stamped(now - REFRESH_INTERVAL - 1)),
            now
        ));
        // A stamp from the future yields a negative age (not 0 — `saturating_sub`
        // saturates at `i64::MIN`), which is never past the interval, so no spin.
        assert!(!is_refresh_due(Some(&stamped(now + 10_000)), now));
    }

    #[test]
    fn cache_roundtrip_and_garbage() {
        let good = parse_cache(r#"{"checked_at":42,"latest":"2026.8.20"}"#).unwrap();
        assert_eq!(good.checked_at, 42);
        assert_eq!(good.latest, Some(v("2026.8.20")));

        // A failed check: stamped, but nothing to show.
        let failed = parse_cache(r#"{"checked_at":42,"latest":null}"#).unwrap();
        assert_eq!(failed.latest, None);

        // Malformed, wrong types, missing stamp, unparseable version.
        assert!(parse_cache("{").is_none());
        assert!(parse_cache(r#"{"checked_at":"soon"}"#).is_none());
        assert!(parse_cache(r#"{"latest":"2026.8.20"}"#).is_none());
        assert_eq!(
            parse_cache(r#"{"checked_at":42,"latest":"nope"}"#)
                .unwrap()
                .latest,
            None
        );
    }

    #[test]
    fn parse_calver_and_beta() {
        assert_eq!(v("2026.7.21").to_string(), "2026.7.21");
        assert_eq!(v("2026.8.15-beta.1").to_string(), "2026.8.15-beta.1");
        assert!(Version::parse("garbage").is_none());
        assert!(Version::parse("2026.8").is_none());
        assert!(Version::parse("a.b.c").is_none());
    }

    #[test]
    fn prerelease_sorts_below_release() {
        assert!(v("2026.7.21") > v("2026.7.21-beta.1"));
        assert!(v("2026.8.15-beta.1") > v("2026.7.21")); // newer minor beats beta
    }

    #[test]
    fn beta_increments_compare_numerically() {
        assert!(v("2026.8.15-beta.2") > v("2026.8.15-beta.1"));
        assert!(v("2026.8.15-beta.1") == v("2026.8.15-beta.1"));
    }

    #[test]
    fn recommend_up_to_date_when_equal() {
        let latest = Latest {
            overall: v("2026.8.15-beta.1"),
            stable: Some(v("2026.7.21")),
        };
        assert_eq!(
            recommend(&v("2026.8.15-beta.1"), &latest, Channel::Beta),
            Recommendation::UpToDate
        );
    }

    #[test]
    fn recommend_update_when_behind_on_beta_channel() {
        let latest = Latest {
            overall: v("2026.8.15-beta.1"),
            stable: Some(v("2026.7.21")),
        };
        assert_eq!(
            recommend(&v("2026.7.21"), &latest, Channel::Beta),
            Recommendation::Update {
                version: v("2026.8.15-beta.1"),
                is_beta: true,
                stable: Some(v("2026.7.21")),
            }
        );
    }

    #[test]
    fn stable_channel_ignores_prerelease_until_newer_release_exceeds_stable() {
        // Installed at the latest *stable*: even though a newer prerelease
        // exists, a Stable-channel user must be told they're up to date.
        let latest = Latest {
            overall: v("2026.8.15-beta.1"),
            stable: Some(v("2026.7.21")),
        };
        assert_eq!(
            recommend(&v("2026.7.21"), &latest, Channel::Stable),
            Recommendation::UpToDate
        );
    }

    #[test]
    fn stable_channel_still_updates_when_a_new_stable_exceeds_installed() {
        // A real newer stable release must still be offered on the Stable channel.
        let latest = Latest {
            overall: v("2026.8.20"),
            stable: Some(v("2026.8.20")),
        };
        assert_eq!(
            recommend(&v("2026.7.21"), &latest, Channel::Stable),
            Recommendation::Update {
                version: v("2026.8.20"),
                is_beta: false,
                stable: Some(v("2026.8.20")),
            }
        );
    }

    #[test]
    fn stable_channel_falls_back_to_overall_when_no_stable_exists() {
        // Only prereleases exist: Stable falls back to the newest overall.
        let latest = Latest {
            overall: v("2026.8.15-beta.1"),
            stable: None,
        };
        assert_eq!(
            recommend(&v("2026.7.21"), &latest, Channel::Stable),
            Recommendation::Update {
                version: v("2026.8.15-beta.1"),
                is_beta: true,
                stable: None,
            }
        );
    }
}
