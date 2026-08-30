//! Color model and the [`Theme`] struct of named color slots.
//!
//! A theme is a *fixed struct* (not a map): adding a slot is a compile error in
//! every theme that omits it, so a theme can never silently miss a color.

use std::fmt::Write;

/// A 256-color ANSI color index, used as `\e[38;5;<n>m`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8);

impl Color {
    /// SGR foreground sequence as an owned `String`, e.g. `\x1b[38;5;33m`.
    ///
    /// Test-only: the render path uses [`Color::write_fg`], which appends into
    /// an existing buffer instead of allocating. This one exists so assertions
    /// can build an expected escape sequence inline, and is gated so it does
    /// not sit in the public API or the shipped binary as a slower twin.
    #[cfg(test)]
    #[must_use = "returns ANSI escape string; ignoring it is a bug"]
    pub fn fg(self) -> String {
        format!("\x1b[38;5;{}m", self.0)
    }

    /// Append the SGR foreground sequence directly into `buf`, avoiding the
    /// throwaway `String` an owned-return variant would allocate on the render
    /// hot path.
    ///
    /// # Panics
    ///
    /// The internal `write!` to a `String` buffer is infallible and will never panic.
    #[inline]
    pub fn write_fg(self, buf: &mut String) {
        buf.push_str("\x1b[38;5;");
        write!(buf, "{}", self.0).unwrap();
        buf.push('m');
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
    /// Model display name.
    pub model: Color,
    /// Stash count (falls back to `git_branch` in themes that predate this slot).
    pub stash: Color,
    /// Lines added/removed background.
    pub lines: Color,
    /// Cost in USD background.
    pub cost: Color,
    /// Session duration background.
    pub duration: Color,
    /// Clock background.
    pub clock: Color,
    /// Effort level (reasoning effort) background.
    pub effort: Color,
    /// Burn-rate / range-to-empty background.
    pub burn: Color,
}
