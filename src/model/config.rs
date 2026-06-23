//! User configuration: which segments, in what order, which theme & style,
//! and the numeric thresholds. Persisted as TOML.
//!
//! Config-less operation uses [`Config::default`], which reproduces the original
//! bash look exactly: Tokyo Night palette, Powerline style, all five segments in
//! their historical order.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The renderable segments. `Vec<SegmentKind>` in [`Config`] encodes both
/// *which* are enabled (presence) and their *order* (render order).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum SegmentKind {
    Directory,
    Git,
    Context,
    RateLimits,
    DevContext,
    Model,
}

impl SegmentKind {
    /// All segments in canonical (default) order.
    pub const ALL: [SegmentKind; 6] = [
        SegmentKind::Directory,
        SegmentKind::Git,
        SegmentKind::Context,
        SegmentKind::RateLimits,
        SegmentKind::DevContext,
        SegmentKind::Model,
    ];

    /// Human label for the TUI list.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            SegmentKind::Directory => "Directory",
            SegmentKind::Git => "Git",
            SegmentKind::Context => "Context",
            SegmentKind::RateLimits => "Rate limits",
            SegmentKind::DevContext => "Dev context",
            SegmentKind::Model => "Model",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Thresholds {
    /// Bar turns warn-colored at or above this percent.
    pub warn: u16,
    /// Bar turns crit-colored at or above this percent.
    pub crit: u16,
    /// The weekly rate-limit window is only shown once usage reaches this percent.
    pub weekly_show_at: u16,
    /// Width, in cells, of every progress bar.
    pub bar_width: u8,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            warn: 50,
            crit: 80,
            weekly_show_at: 50,
            bar_width: 6,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub theme: String,
    pub style: String,
    pub segments: Vec<SegmentKind>,
    pub thresholds: Thresholds,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "tokyo-night".into(),
            style: "powerline".into(),
            segments: SegmentKind::ALL.to_vec(),
            thresholds: Thresholds::default(),
        }
    }
}

impl Config {
    /// Standard config path: `$XDG_CONFIG_HOME/claudebar/config.toml`,
    /// falling back to `$HOME/.config/claudebar/config.toml`.
    #[must_use]
    pub fn default_path() -> Option<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        Some(base.join("claudebar").join("config.toml"))
    }

    /// Load config from `path`. A missing file yields `Config::default()`
    /// (config-less operation is a supported, first-class state). A present but
    /// malformed file is a real error the caller should surface.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::Parse` when the file contains invalid TOML, and
    /// `ConfigError::Io` when the file exists but cannot be read (permissions,
    /// filesystem error). A missing file is **not** an error — it yields
    /// `Ok(Config::default())`.
    #[must_use = "returns default config on file-not-found, but parse errors must be surfaced"]
    pub fn load(path: &Path) -> Result<Config, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(s) => toml::from_str(&s).map_err(|e| ConfigError::Parse(e.to_string())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
            Err(e) => Err(ConfigError::Io(e.to_string())),
        }
    }

    /// Load from the explicit path if given, else the default path, else default.
    #[must_use]
    pub fn load_or_default(explicit: Option<&Path>) -> Config {
        let path = explicit.map(PathBuf::from).or_else(Config::default_path);
        match path {
            Some(p) => Config::load(&p).unwrap_or_default(),
            None => Config::default(),
        }
    }

    /// Serialize to pretty TOML, creating parent dirs.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::Io` when the parent directory cannot be created or
    /// the file cannot be written. Returns `ConfigError::Parse` when
    /// serialization fails (should not happen for valid data).
    #[must_use = "ignoring the save result means the config is silently not persisted"]
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ConfigError::Io(e.to_string()))?;
        }
        let body = toml::to_string_pretty(self).map_err(|e| ConfigError::Parse(e.to_string()))?;
        std::fs::write(path, body).map_err(|e| ConfigError::Io(e.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigError {
    #[error("config i/o error: {0}")]
    Io(String),
    #[error("config parse error: {0}")]
    Parse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_bash_layout() {
        let c = Config::default();
        assert_eq!(c.theme, "tokyo-night");
        assert_eq!(c.style, "powerline");
        assert_eq!(c.segments, SegmentKind::ALL.to_vec());
        assert_eq!(c.thresholds.warn, 50);
        assert_eq!(c.thresholds.crit, 80);
    }

    #[test]
    fn roundtrips_through_toml() {
        let c = Config::default();
        let s = toml::to_string_pretty(&c).unwrap();
        let back: Config = toml::from_str(&s).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn partial_toml_fills_defaults() {
        // serde(default) means a sparse file is still valid.
        let c: Config = toml::from_str(r#"theme = "nord""#).unwrap();
        assert_eq!(c.theme, "nord");
        assert_eq!(c.style, "powerline");
        assert_eq!(c.segments, SegmentKind::ALL.to_vec());
    }

    #[test]
    fn segments_kebab_case() {
        let c: Config = toml::from_str(r#"segments = ["rate-limits", "git"]"#).unwrap();
        assert_eq!(c.segments, vec![SegmentKind::RateLimits, SegmentKind::Git]);
    }

    /// Unique temp path; nanos + pid keep parallel test runs from colliding.
    /// No `tempfile` crate — `insta` is the only dev-dependency.
    fn unique_temp_path() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!(
            "claudebar-test-{}-{}.toml",
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn malformed_toml_is_parse_error() {
        // CR-14: a present-but-broken file is a real error, not a silent default.
        let path = unique_temp_path();
        std::fs::write(&path, "theme = \"unclosed").unwrap();
        let result = Config::load(&path);
        assert!(
            matches!(result, Err(ConfigError::Parse(_))),
            "expected Parse error, got: {result:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_or_default_falls_back_on_malformed() {
        // CR-14: the load-or-default path swallows the parse error and yields
        // Config::default() so rendering never breaks on a bad file.
        let path = unique_temp_path();
        std::fs::write(&path, "theme = \"unclosed").unwrap();
        assert_eq!(Config::load_or_default(Some(&path)), Config::default());
        let _ = std::fs::remove_file(&path);
    }
}
