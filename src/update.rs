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
use std::process::Command;
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

/// Compare two full prerelease strings (e.g. `beta.1` vs `beta.2`).
fn cmp_prerelease(a: &str, b: &str) -> Ordering {
    let a_ids: Vec<&str> = a.split('.').collect();
    let b_ids: Vec<&str> = b.split('.').collect();
    let n = a_ids.len().min(b_ids.len());
    for i in 0..n {
        let ord = cmp_prerelease_id(a_ids[i], b_ids[i]);
        if ord != Ordering::Equal {
            return ord;
        }
    }
    // All shared identifiers equal: fewer identifiers is lower (semver rule).
    a_ids.len().cmp(&b_ids.len())
}

fn cmp_prerelease_id(a: &str, b: &str) -> Ordering {
    match (a.parse::<u64>(), b.parse::<u64>()) {
        (Ok(x), Ok(y)) => x.cmp(&y),       // numeric ids compare numerically
        (Ok(_), Err(_)) => Ordering::Less, // numeric < alphanumeric
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => a.cmp(b), // alphanumeric ids compare lexically
    }
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

    let mut versions: Vec<Version> = Vec::new();
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
