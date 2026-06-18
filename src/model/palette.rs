//! Color model and the [`Theme`] struct of named color slots.
//!
//! A theme is a *fixed struct* (not a map): adding a slot is a compile error in
//! every theme that omits it, so a theme can never silently miss a color.

/// A 256-color ANSI color index, used as `\e[38;5;<n>m`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8);

impl Color {
    /// SGR foreground sequence, e.g. `\x1b[38;5;33m`.
    pub fn fg(self) -> String {
        format!("\x1b[38;5;{}m", self.0)
    }
}

/// The SGR reset sequence — ends a colored run.
pub const RESET: &str = "\x1b[0m";

/// Named color slots, one per semantic role. Themes fill every slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Directory path.
    pub dir: Color,
    /// Git branch name.
    pub git_branch: Color,
    /// Ahead count (`↑N`).
    pub ahead: Color,
    /// Behind count (`↓N`).
    pub behind: Color,
    /// Modified-file count (`MN`).
    pub modified: Color,
    /// Untracked-file count (`?N`).
    pub untracked: Color,
    /// Token count (`⬡ 42.3k`).
    pub token: Color,
    /// Progress bar fill below the warn threshold.
    pub bar_ok: Color,
    /// Progress bar fill at/above warn, below crit.
    pub bar_warn: Color,
    /// Progress bar fill at/above crit (and over-limit).
    pub bar_crit: Color,
    /// Empty cells of a progress bar track.
    pub bar_track: Color,
    /// Powerline / pipe separator glyph.
    pub separator: Color,
    /// Dimmed icons and secondary symbols.
    pub dim: Color,
    /// Reset/countdown timer value.
    pub reset: Color,
    /// `max` effort highlight.
    pub effort_max: Color,
    /// Model display name.
    pub model: Color,
}
