//! Where claudebar keeps its files.
//!
//! Both directories follow the XDG base-directory spec: an explicit
//! `$XDG_*_HOME` wins, otherwise the conventional `$HOME` subdirectory. An
//! empty variable counts as unset, which is what the spec asks for and what a
//! shell that exports `XDG_CACHE_HOME=` actually produces.
//!
//! Resolution lives here rather than beside each consumer so the three callers
//! — the config file, the burn-rate samples, and the rate-limit sync store —
//! cannot drift apart.
//!
//! The environment read is split from the path arithmetic (`resolve`), the
//! same split this crate uses for its file-I/O helpers: the logic is then
//! testable directly, with no process-wide `set_var` that would make every
//! other test in the binary racy.

use std::ffi::OsString;
use std::path::PathBuf;

/// `$XDG_CONFIG_HOME/claudebar`, else `$HOME/.config/claudebar`.
/// `None` when neither variable is set.
#[must_use]
pub fn config_dir() -> Option<PathBuf> {
    resolve(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
        ".config",
    )
}

/// `$XDG_CACHE_HOME/claudebar`, else `$HOME/.cache/claudebar`.
/// `None` when neither variable is set.
#[must_use]
pub fn cache_dir() -> Option<PathBuf> {
    resolve(
        std::env::var_os("XDG_CACHE_HOME"),
        std::env::var_os("HOME"),
        ".cache",
    )
}

/// Pure resolution: the XDG override if it is set and non-empty, otherwise
/// `home/<home_subdir>`, with `claudebar` appended either way.
fn resolve(xdg: Option<OsString>, home: Option<OsString>, home_subdir: &str) -> Option<PathBuf> {
    xdg.map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .or_else(|| home.map(|h| PathBuf::from(h).join(home_subdir)))
        .map(|b| b.join("claudebar"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(s: &str) -> Option<OsString> {
        Some(OsString::from(s))
    }

    #[test]
    fn xdg_var_wins_over_home() {
        assert_eq!(
            resolve(os("/xdg"), os("/home/u"), ".cache"),
            Some(PathBuf::from("/xdg/claudebar"))
        );
    }

    #[test]
    fn falls_back_to_the_home_subdirectory() {
        assert_eq!(
            resolve(None, os("/home/u"), ".cache"),
            Some(PathBuf::from("/home/u/.cache/claudebar"))
        );
        assert_eq!(
            resolve(None, os("/home/u"), ".config"),
            Some(PathBuf::from("/home/u/.config/claudebar"))
        );
    }

    /// An exported-but-empty variable is treated as unset, not as the current
    /// directory — `XDG_CACHE_HOME=` in a shell profile is a real occurrence,
    /// and honouring it literally would scatter state into `./claudebar`.
    #[test]
    fn an_empty_xdg_var_counts_as_unset() {
        assert_eq!(
            resolve(os(""), os("/home/u"), ".cache"),
            Some(PathBuf::from("/home/u/.cache/claudebar"))
        );
    }

    /// No override and no `$HOME`: callers must no-op rather than guess.
    #[test]
    fn none_when_nothing_is_set() {
        assert_eq!(resolve(None, None, ".cache"), None);
    }

    /// An empty `$HOME` still yields a path, matching the pre-existing
    /// behaviour of every caller this replaced — only the XDG override is
    /// emptiness-checked.
    #[test]
    fn empty_home_is_not_emptiness_checked() {
        assert_eq!(
            resolve(None, os(""), ".cache"),
            Some(PathBuf::from(".cache/claudebar"))
        );
    }
}
