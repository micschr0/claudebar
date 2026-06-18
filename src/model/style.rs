//! Style model: separators, glyph set, icon toggle, and bar characters.
//!
//! A style is pure data. The ASCII fallback style, for instance, is just a
//! [`GlyphSet`] of ASCII characters with `icons = false` and `#`/`-` bar chars —
//! segments never branch on the style, they only read these fields.

/// The set of glyphs a style uses. Nerd Font PUA glyphs for rich styles, ASCII
/// for the fallback. Stored as `&'static str` because some glyphs are multi-byte.
#[derive(Debug, Clone, Copy)]
pub struct GlyphSet {
    /// Branch icon preceding the branch name.
    pub branch: &'static str,
    /// Ahead marker (`↑` / `^`).
    pub ahead: &'static str,
    /// Behind marker (`↓` / `v`).
    pub behind: &'static str,
    /// Modified marker (`M`).
    pub modified: &'static str,
    /// Untracked marker (`?`).
    pub untracked: &'static str,
    /// Context/usage icon.
    pub context: &'static str,
    /// Token icon (`⬡`).
    pub token: &'static str,
    /// 5-hour window clock icon.
    pub clock: &'static str,
    /// Weekly window icon.
    pub weekly: &'static str,
    /// Reset/countdown icon (`↺`).
    pub reset: &'static str,
    /// Model icon (`◈`).
    pub model: &'static str,
    /// Effort icon (bolt).
    pub effort: &'static str,
}

/// A complete visual style: how segments are separated and decorated.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// Separator glyph placed between adjacent non-empty segments (painted in
    /// the theme's `separator` color, with a space on each side).
    pub separator: &'static str,
    /// When false, icons are suppressed entirely (minimal / pure-text styles).
    pub icons: bool,
    /// The glyph set this style draws from.
    pub glyphs: GlyphSet,
    /// Filled bar cell.
    pub bar_fill: char,
    /// Empty bar cell.
    pub bar_empty: char,
}
