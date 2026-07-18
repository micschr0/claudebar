//! Patches the `statusLine` key of Claude Code's `settings.json` — a
//! foreign, user-owned config file (not claudebar's own TOML) — to point at
//! `claudebar render`, without disturbing any other key.
//!
//! Unlike [`crate::model::Config`] (and `InputData::parse`), this module
//! intentionally does **not** follow the "infallible parse" philosophy:
//! malformed input here must surface as a typed error, per CLAUDE.md's Error
//! Handling conventions, rather than silently degrading to a default.

use serde_json::Value;
use std::path::{Path, PathBuf};

/// The command wired into `statusLine`. A fixed constant, never built by
/// formatting external/user input into the JSON `command` field.
pub const STATUSLINE_COMMAND: &str = "claudebar render";

/// Resolve the claudebar config file path. Pure — no environment reads, so
/// directly unit-testable. Mirrors the logic in [`crate::model::Config::default_path`]
/// but takes explicit parameters.
#[must_use]
pub fn resolve_config(xdg_config_home: Option<&Path>, home: Option<&Path>) -> PathBuf {
    if let Some(xdg) = xdg_config_home {
        xdg.join("claudebar").join("config.toml")
    } else if let Some(home) = home {
        home.join(".config").join("claudebar").join("config.toml")
    } else {
        PathBuf::from("config.toml")
    }
}

/// Resolve the editor command from `EDITOR` / `VISUAL` env var values passed
/// explicitly. Returns `None` when neither is set.
#[must_use]
pub fn resolve_editor_from(editor: Option<String>, visual: Option<String>) -> Option<String> {
    editor.or(visual)
}

/// Nerd Font check: look for .ttf/.otf files with "Nerd" or "nerd" in their name.
/// Uses `fc-list` if available; falls back to scanning common font dirs.
#[must_use]
pub fn check_nerd_font() -> bool {
    // Try fc-list first — fastest and most accurate.
    if let Ok(output) = std::process::Command::new("fc-list")
        .arg(":family")
        .output()
        && let Ok(stdout) = String::from_utf8(output.stdout)
        && stdout.to_lowercase().contains("nerd")
    {
        return true;
    }

    // Fallback: scan common font directories.
    let dirs: &[&str] = &[
        "/usr/share/fonts",
        "/usr/local/share/fonts",
        "~/.local/share/fonts",
        "~/.fonts",
    ];

    let home = std::env::var("HOME").unwrap_or_default();

    for dir in dirs {
        let path = if let Some(stripped) = dir.strip_prefix("~/") {
            PathBuf::from(home.clone()).join(stripped)
        } else {
            PathBuf::from(dir)
        };
        if path.is_dir()
            && let Ok(entries) = std::fs::read_dir(&path)
        {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if (name.ends_with(".ttf") || name.ends_with(".otf"))
                    && name.to_lowercase().contains("nerd")
                {
                    return true;
                }
            }
        }
    }

    false
}
/// Errors surfaced when reading, parsing, or writing settings.json.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SetupError {
    #[error("settings i/o error: {0}")]
    Io(String),
    #[error("settings parse error: {0}")]
    Parse(String),
}

/// Outcome of comparing the current `statusLine` value to the desired one.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// `statusLine` already equals the desired value — nothing to do.
    AlreadyConfigured,
    /// `statusLine` is absent (`previous: None`), or differs but `--force`
    /// was passed (`previous: Some(existing)`) — safe/requested to write.
    WillSet { previous: Option<Value> },
    /// `statusLine` differs from the desired value and `--force` was not
    /// passed — refuse to overwrite.
    Conflict { existing: Value },
}

/// Resolve the settings.json path: `$SETTINGS` env var (non-empty) wins;
/// else `$HOME/.claude/settings.json`; `None` if neither resolvable.
#[must_use]
pub fn default_settings_path() -> Option<PathBuf> {
    std::env::var_os("SETTINGS")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .or_else(|| {
            std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".claude").join("settings.json"))
        })
}

/// The desired `statusLine` value. With `binary_path: None`, returns the
/// unchanged default `{"type":"command","command":"claudebar render"}`. With
/// `binary_path: Some(path)`, builds `{"type":"command","command":"{path} render"}`
/// instead, so callers (e.g. `install.sh`) can point `statusLine` at an
/// absolute install location rather than assuming `claudebar` is on `$PATH`.
#[must_use]
pub fn desired_status_line(binary_path: Option<&str>) -> Value {
    let command = match binary_path {
        Some(path) => format!("{path} render"),
        None => STATUSLINE_COMMAND.to_string(),
    };
    serde_json::json!({
        "type": "command",
        "command": command,
    })
}

/// Load `settings.json` from `path`. A missing file yields an empty JSON
/// object (there is nothing to merge into yet). A present file that is not
/// valid JSON, or whose root is not a JSON object, is a real error the
/// caller must surface — never silently discarded.
///
/// # Errors
///
/// Returns `SetupError::Parse` when the file contains invalid JSON or its
/// root is not an object, and `SetupError::Io` when the file exists but
/// cannot be read for any other reason.
#[must_use = "parse errors must be surfaced, not silently discarded"]
pub fn load_settings(path: &Path) -> Result<Value, SetupError> {
    match std::fs::read_to_string(path) {
        Ok(s) => {
            let value: Value =
                serde_json::from_str(&s).map_err(|e| SetupError::Parse(e.to_string()))?;
            if value.is_object() {
                Ok(value)
            } else {
                Err(SetupError::Parse(
                    "settings.json root is not a JSON object".to_string(),
                ))
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(Value::Object(serde_json::Map::new()))
        }
        Err(e) => Err(SetupError::Io(e.to_string())),
    }
}

/// Compare the current `statusLine` value in `settings` to `desired`.
#[must_use]
pub fn classify(settings: &Value, desired: &Value, force: bool) -> Outcome {
    match settings.get("statusLine") {
        None => Outcome::WillSet { previous: None },
        Some(existing) if existing == desired => Outcome::AlreadyConfigured,
        Some(existing) if force => Outcome::WillSet {
            previous: Some(existing.clone()),
        },
        Some(existing) => Outcome::Conflict {
            existing: existing.clone(),
        },
    }
}

/// Set `settings["statusLine"] = desired` in place, leaving every other key
/// untouched.
///
/// # Panics
///
/// Panics if `settings` is not a `Value::Object` — callers must only ever
/// pass a value obtained from [`load_settings`], which guarantees this.
pub fn apply(settings: &mut Value, desired: Value) {
    settings
        .as_object_mut()
        .expect("settings must be a JSON object (guaranteed by load_settings)")
        .insert("statusLine".to_string(), desired);
}

/// Build the backup path for `path`: appends `.bak-{now}` to the file name.
/// Pure — no filesystem access, so directly unit-testable with a fixed `now`.
#[must_use]
pub fn backup_path(path: &Path, now: i64) -> PathBuf {
    let mut name = path
        .file_name()
        .map(std::ffi::OsStr::to_os_string)
        .unwrap_or_default();
    name.push(format!(".bak-{now}"));
    path.with_file_name(name)
}

/// Copy `path` to its backup location (see [`backup_path`]). Caller must
/// only invoke this when `path.exists()`.
///
/// # Errors
///
/// Returns `SetupError::Io` when the copy fails.
#[must_use = "the backup path must be reported to the user, and copy failures must be surfaced"]
pub fn backup_settings(path: &Path, now: i64) -> Result<PathBuf, SetupError> {
    let backup = backup_path(path, now);
    std::fs::copy(path, &backup).map_err(|e| SetupError::Io(e.to_string()))?;
    Ok(backup)
}

/// Serialize `value` as pretty JSON (plus a trailing newline) and write it to
/// `path`, creating parent directories as needed. Mirrors `Config::save`.
///
/// # Errors
///
/// Returns `SetupError::Io` when the parent directory cannot be created or
/// the file cannot be written. Returns `SetupError::Parse` on the
/// near-impossible serialization failure.
#[must_use = "ignoring the save result means settings.json is silently not persisted"]
pub fn save_settings(path: &Path, value: &Value) -> Result<(), SetupError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| SetupError::Io(e.to_string()))?;
    }
    let mut body =
        serde_json::to_string_pretty(value).map_err(|e| SetupError::Parse(e.to_string()))?;
    body.push('\n');
    std::fs::write(path, body).map_err(|e| SetupError::Io(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unique temp path; nanos + pid keep parallel test runs from colliding.
    /// No `tempfile` crate — `insta` is the only dev-dependency.
    fn unique_temp_path() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!(
            "claudebar-setup-test-{}-{}.json",
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn desired_status_line_with_binary_path_override() {
        assert_eq!(
            desired_status_line(Some("some/path")),
            serde_json::json!({"type":"command","command":"some/path render"})
        );
    }

    #[test]
    fn classify_missing_key_will_set_regardless_of_force() {
        let settings = serde_json::json!({});
        let desired = desired_status_line(None);
        assert_eq!(
            classify(&settings, &desired, false),
            Outcome::WillSet { previous: None }
        );
        assert_eq!(
            classify(&settings, &desired, true),
            Outcome::WillSet { previous: None }
        );
    }

    #[test]
    fn classify_matching_key_already_configured_regardless_of_force() {
        let desired = desired_status_line(None);
        let settings = serde_json::json!({ "statusLine": desired.clone() });
        assert_eq!(
            classify(&settings, &desired, false),
            Outcome::AlreadyConfigured
        );
        assert_eq!(
            classify(&settings, &desired, true),
            Outcome::AlreadyConfigured
        );
    }

    #[test]
    fn classify_conflicting_key_without_force_is_conflict() {
        let desired = desired_status_line(None);
        let existing = serde_json::json!({ "type": "command", "command": "other" });
        let settings = serde_json::json!({ "statusLine": existing.clone() });
        assert_eq!(
            classify(&settings, &desired, false),
            Outcome::Conflict { existing }
        );
    }

    #[test]
    fn classify_conflicting_key_with_force_will_set() {
        let desired = desired_status_line(None);
        let existing = serde_json::json!({ "type": "command", "command": "other" });
        let settings = serde_json::json!({ "statusLine": existing.clone() });
        assert_eq!(
            classify(&settings, &desired, true),
            Outcome::WillSet {
                previous: Some(existing)
            }
        );
    }

    #[test]
    fn malformed_existing_json_is_parse_error() {
        // Same style as config.rs's malformed_toml_is_parse_error test.
        let path = unique_temp_path();
        std::fs::write(&path, "not json").unwrap();
        let result = load_settings(&path);
        assert!(
            matches!(result, Err(SetupError::Parse(_))),
            "expected Parse error, got: {result:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn non_object_root_is_parse_error() {
        let path = unique_temp_path();
        std::fs::write(&path, "[1, 2, 3]").unwrap();
        let result = load_settings(&path);
        assert!(
            matches!(result, Err(SetupError::Parse(_))),
            "expected Parse error, got: {result:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_file_loads_as_empty_object() {
        let path = unique_temp_path();
        let result = load_settings(&path);
        assert_eq!(result.unwrap(), Value::Object(serde_json::Map::new()));
    }

    #[test]
    fn save_then_load_roundtrips() {
        let path = unique_temp_path();
        let mut value = serde_json::json!({ "otherKey": true });
        apply(&mut value, desired_status_line(None));
        save_settings(&path, &value).unwrap();
        let loaded = load_settings(&path).unwrap();
        assert_eq!(loaded, value);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn apply_preserves_unrelated_keys() {
        let mut settings = serde_json::json!({ "otherKey": true });
        apply(&mut settings, desired_status_line(None));
        assert_eq!(settings["otherKey"], serde_json::json!(true));
        assert_eq!(settings["statusLine"], desired_status_line(None));
    }

    #[test]
    fn backup_path_appends_bak_timestamp() {
        let path = PathBuf::from("/tmp/settings.json");
        let backup = backup_path(&path, 1_751_500_000);
        assert_eq!(backup, PathBuf::from("/tmp/settings.json.bak-1751500000"));
    }

    #[test]
    fn backup_settings_copies_file() {
        let path = unique_temp_path();
        std::fs::write(&path, "{}").unwrap();
        let backup = backup_settings(&path, 42).unwrap();
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), "{}");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&backup);
    }

    #[test]
    fn resolve_config_uses_xdg_var_when_set() {
        let xdg = Path::new("/tmp/xdg");
        let home = Path::new("/home/user");
        let result = resolve_config(Some(xdg), Some(home));
        assert_eq!(result, PathBuf::from("/tmp/xdg/claudebar/config.toml"));
    }

    #[test]
    fn resolve_config_falls_back_to_home_when_xdg_unset() {
        let home = Path::new("/home/user");
        let result = resolve_config(None, Some(home));
        assert_eq!(
            result,
            PathBuf::from("/home/user/.config/claudebar/config.toml")
        );
    }

    #[test]
    fn resolve_config_defaults_to_relative_when_neither_set() {
        let result = resolve_config(None, None);
        assert_eq!(result, PathBuf::from("config.toml"));
    }

    #[test]
    fn doctor_reports_missing_nerd_font_is_bool() {
        // Structural test: the function must return bool and not panic.
        let _: bool = check_nerd_font();
    }

    #[test]
    fn resolve_editor_from_uses_editor_before_visual() {
        assert_eq!(
            resolve_editor_from(Some("vim".into()), Some("code".into())),
            Some("vim".into())
        );
        assert_eq!(
            resolve_editor_from(None, Some("code".into())),
            Some("code".into())
        );
        assert_eq!(resolve_editor_from(None, None), None);
    }
}
