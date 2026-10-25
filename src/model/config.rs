//! User configuration: which segments, in what order, which theme & style,
//! and the numeric thresholds. Persisted as TOML.
//!
//! Config-less operation uses [`Config::default`], which renders the
//! 8-segment [`SegmentKind::DEFAULT`] layout with the Tokyo Night palette and
//! Powerline style. That layout mirrors the bash statusline's segment order
//! (`lines` placed between `context` and `rate-limits`) and therefore
//! deliberately differs from the canonical [`SegmentKind::ALL`] order; the
//! parity is guarded by the `default_matches_bash_layout` test.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The renderable segments. `Vec<SegmentKind>` in [`Config`] encodes both
/// *which* are enabled (presence) and their *order* (render order).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum SegmentKind {
    Directory,
    Git,
    Context,
    RateLimits,
    DevContext,
    Model,
    Clock,
    Cost,
    Lines,
    Duration,
    Burn,
    UpdateNotice,
}

impl SegmentKind {
    /// All segments in canonical order.
    pub const ALL: [SegmentKind; 12] = [
        SegmentKind::Directory,
        SegmentKind::Git,
        SegmentKind::Model,
        SegmentKind::Context,
        SegmentKind::RateLimits,
        SegmentKind::DevContext,
        SegmentKind::Cost,
        SegmentKind::Lines,
        SegmentKind::Duration,
        SegmentKind::Burn,
        SegmentKind::Clock,
        SegmentKind::UpdateNotice,
    ];

    pub const DEFAULT: [SegmentKind; 8] = [
        SegmentKind::Directory,
        SegmentKind::Git,
        SegmentKind::Model,
        SegmentKind::Context,
        SegmentKind::Lines,
        SegmentKind::RateLimits,
        SegmentKind::Cost,
        SegmentKind::Duration,
    ];

    /// Human label for the TUI list.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            SegmentKind::Directory => "Directory",
            SegmentKind::Git => "Git",
            SegmentKind::Context => "Context",
            SegmentKind::RateLimits => "Rate Limits",
            SegmentKind::DevContext => "Dev Context",
            SegmentKind::Model => "Model",
            SegmentKind::Clock => "Clock",
            SegmentKind::Cost => "Cost",
            SegmentKind::Lines => "Lines",
            SegmentKind::Duration => "Duration",
            SegmentKind::Burn => "Burn",
            SegmentKind::UpdateNotice => "Update Notice",
        }
    }

    /// Parse a kebab-case name (e.g. `"rate-limits"`, `"dev-context"`) into a
    /// [`SegmentKind`]. Returns `None` for unknown names.
    ///
    /// Uses serde's `rename_all = "kebab-case"` mapping so the name table stays
    /// in one place.
    #[must_use]
    pub fn from_kebab(s: &str) -> Option<Self> {
        // Single-value JSON deserialization reuses the existing rename mapping.
        serde_json::from_str(&format!("\"{s}\"")).ok()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Thresholds {
    /// Bar turns warn-colored at or above this percent.
    pub warn: u16,
    /// Bar turns crit-colored at or above this percent.
    pub crit: u16,
    /// The weekly rate-limit window is only shown once usage reaches this percent.
    pub weekly_show_at: u16,
    /// Width, in cells, of every progress bar.
    pub bar_width: u8,
    /// Decimal places for the cost segment (0–4).
    pub cost_decimals: u8,
    /// Clock display mode: `12h`, `24h`, or `off`.
    pub clock_mode: String,
    /// Burn-rate lookback window in seconds (default 600 = 10 min).
    pub burn_lookback: u32,
    /// Enable the plain-text float readout file (best-effort side effect).
    pub float: bool,
    /// Segments to render into the float readout (space-separated kebab-case names).
    pub float_segments: String,
    /// Separator placed between adjacent non-empty float segments.
    pub float_sep: String,
    /// Where to write the float readout (`~` expands to `$HOME`).
    pub float_file: String,
    /// Layout mode: "fixed" (one line) or "auto" (responsive wrap).
    pub layout: String,
    /// Auto only — max lines to wrap into.
    pub max_lines: u8,
    /// Auto only — columns kept free on the right.
    pub wrap_margin: u8,
    /// Enable cross-session rate-limit sync (opt-in).
    pub limit_sync: bool,
}
impl Default for Thresholds {
    fn default() -> Self {
        Self {
            warn: 50,
            crit: 80,
            weekly_show_at: 75,
            bar_width: 6,
            cost_decimals: 2,
            clock_mode: "auto".into(),
            burn_lookback: 600,
            float: false,
            float_segments: "model context cost".into(),
            float_sep: "  ·  ".into(),
            float_file: "~/.claude/claudebar-float.txt".into(),
            layout: "fixed".into(),
            max_lines: 3,
            wrap_margin: 4,
            limit_sync: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
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
            segments: SegmentKind::DEFAULT.to_vec(),
            thresholds: Thresholds::default(),
        }
    }
}

impl Config {
    /// Standard config path: `$XDG_CONFIG_HOME/claudebar/config.toml`,
    /// falling back to `$HOME/.config/claudebar/config.toml`.
    #[must_use]
    pub fn default_path() -> Option<PathBuf> {
        Some(crate::paths::config_dir()?.join("config.toml"))
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
        let path = explicit.map(PathBuf::from).or_else(Self::default_path);
        match path {
            Some(path) => Self::load(&path).unwrap_or_default(),
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
        let body = toml::to_string_pretty(self).map_err(|e| ConfigError::Parse(e.to_string()))?;
        let mut path = std::borrow::Cow::Borrowed(path);
        // Follow final-component links without requiring their target to exist.
        for links in 0..=40 {
            match std::fs::read_link(path.as_ref()) {
                Ok(target) if links < 40 => {
                    path = std::borrow::Cow::Owned(if target.is_absolute() {
                        target
                    } else {
                        path.parent().unwrap_or_else(|| Path::new(".")).join(target)
                    });
                }
                Ok(_) => break,
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidInput
                    ) =>
                {
                    return crate::render::float::write_atomic(&path, &body)
                        .map_err(|e| ConfigError::Io(e.to_string()));
                }
                Err(e) => return Err(ConfigError::Io(e.to_string())),
            }
        }
        Err(ConfigError::Io("too many configuration symlinks".into()))
    }
}

#[derive(Debug, thiserror::Error)]
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
        assert_eq!(c.segments, SegmentKind::DEFAULT.to_vec());
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
        assert_eq!(c.segments, SegmentKind::DEFAULT.to_vec());
    }

    /// A config written by an older claudebar still loads after a threshold key
    /// is retired. `name_max` and `model_show_effort` were both serialized into
    /// real user configs before being removed as dead (neither was ever read),
    /// so those files are out in the world and must not start erroring.
    ///
    /// This holds because neither `Config` nor `Thresholds` sets
    /// `deny_unknown_fields`. That is load-bearing rather than incidental —
    /// adding it would break every config file written before this commit —
    /// so the guarantee is pinned here instead of left to be rediscovered.
    #[test]
    fn retired_and_unknown_threshold_keys_are_ignored() {
        let c: Config = toml::from_str(
            r#"
            theme = "nord"
            [thresholds]
            warn = 42
            name_max = 30
            model_show_effort = false
            some_key_that_never_existed = "whatever"
            "#,
        )
        .expect("a config with retired/unknown threshold keys must still load");

        // The retired keys are ignored; every surviving key still applies.
        assert_eq!(c.theme, "nord");
        assert_eq!(c.thresholds.warn, 42);
        assert_eq!(c.thresholds.crit, Thresholds::default().crit);
    }

    #[test]
    fn segments_kebab_case() {
        let c: Config = toml::from_str(r#"segments = ["rate-limits", "git"]"#).unwrap();
        assert_eq!(c.segments, vec![SegmentKind::RateLimits, SegmentKind::Git]);
    }

    #[test]
    fn config_save_failure_preserves_existing_file() {
        let path = unique_temp_path();
        std::fs::write(&path, "previous contents").unwrap();
        let result = Config::default().save(&path.join("unwritable"));
        assert!(matches!(result, Err(ConfigError::Io(_))));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "previous contents");
        std::fs::remove_file(path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn config_save_preserves_existing_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let path = unique_temp_path();
        std::fs::write(&path, "old").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        Config::default().save(&path).unwrap();
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        std::fs::remove_file(path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn config_save_preserves_config_symlinks() {
        use std::os::unix::fs::symlink;

        let target = unique_temp_path();
        let link = target.with_extension("link");
        std::fs::write(&target, "old").unwrap();
        symlink(&target, &link).unwrap();
        let config = Config {
            theme: "nord".into(),
            ..Config::default()
        };
        config.save(&link).unwrap();
        assert!(
            std::fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(Config::load(&target).unwrap().theme, "nord");
        std::fs::remove_file(link).unwrap();
        std::fs::remove_file(target).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn config_save_follows_relative_symlink_chain_to_missing_target() {
        let dir = unique_temp_path();
        std::fs::create_dir(&dir).unwrap();
        let link = dir.join("config.toml");
        let middle = dir.join("middle");
        let target = dir.join("missing.toml");
        std::os::unix::fs::symlink("middle", &link).unwrap();
        std::os::unix::fs::symlink("missing.toml", &middle).unwrap();
        let config = Config {
            theme: "nord".into(),
            ..Config::default()
        };
        config.save(&link).unwrap();
        assert_eq!(Config::load(&target).unwrap(), config);
        assert_eq!(std::fs::read_link(&link).unwrap(), Path::new("middle"));
        assert_eq!(
            std::fs::read_link(&middle).unwrap(),
            Path::new("missing.toml")
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn config_save_rejects_symlink_cycles_without_replacing_links() {
        let dir = unique_temp_path();
        std::fs::create_dir(&dir).unwrap();
        let left = dir.join("left");
        let right = dir.join("right");
        std::os::unix::fs::symlink("right", &left).unwrap();
        std::os::unix::fs::symlink("left", &right).unwrap();
        assert!(Config::default().save(&left).is_err());
        assert_eq!(std::fs::read_link(&left).unwrap(), Path::new("right"));
        assert_eq!(std::fs::read_link(&right).unwrap(), Path::new("left"));
        std::fs::remove_dir_all(dir).unwrap();
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
    fn config_default_has_expected_theme() {
        let cfg = Config::default();
        assert_eq!(cfg.theme, "tokyo-night");
    }

    #[test]
    fn config_default_has_all_default_segments() {
        let cfg = Config::default();
        assert_eq!(cfg.segments, SegmentKind::DEFAULT.to_vec());
    }

    #[test]
    fn config_default_has_expected_style() {
        let cfg = Config::default();
        assert_eq!(cfg.style, "powerline");
    }

    #[test]
    fn config_load_missing_file_returns_default() {
        let path = std::path::PathBuf::from("/nonexistent/path/config.toml");
        let result = Config::load(&path);
        assert!(result.is_ok());
        let cfg = result.unwrap();
        assert_eq!(cfg.theme, Config::default().theme);
        assert_eq!(cfg.segments, Config::default().segments);
    }

    #[test]
    fn config_save_then_load_round_trips() {
        let dir = std::env::temp_dir();
        let path = dir.join("claudebar-test-config-roundtrip.toml");
        let _ = std::fs::remove_file(&path);
        let cfg = Config {
            theme: "rose_pine".to_string(),
            segments: vec![SegmentKind::Git, SegmentKind::DevContext],
            ..Default::default()
        };
        cfg.save(&path).expect("save");
        let loaded = Config::load(&path).expect("load");
        assert_eq!(loaded.theme, "rose_pine");
        assert_eq!(
            loaded.segments,
            vec![SegmentKind::Git, SegmentKind::DevContext]
        );
        let _ = std::fs::remove_file(&path);
    }
}
