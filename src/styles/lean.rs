//! lean style — Powerlevel10k-inspired flat look: icons on, no separator
//! glyph, segments flow with just a space between them.
//!
//! Each segment's color becomes its text accent (claudebar already uses
//! foreground-only coloring, so lean is primarily about the empty separator
//! and keeping icons). Reuses the powerline glyph set and bar characters.

use crate::model::Style;

#[must_use]
pub fn style() -> Style {
    Style {
        separator: "",
        window_gap: "\u{b7}", // · middle dot; lean's separator is empty so no collision
        icons: true,
        glyphs: super::powerline::style().glyphs,
        bar_fill: '\u{2501}',  // ━ heavy horizontal
        bar_empty: '\u{254c}', // ╌ light double dash
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn lean_has_empty_separator_with_icons() {
        let s = super::style();
        assert!(s.icons);
        assert_eq!(s.separator, "");
        assert_eq!(s.window_gap, "\u{b7}");
        assert_eq!(s.bar_fill, '\u{2501}');
        assert_eq!(s.bar_empty, '\u{254c}');
    }

    #[test]
    fn loads_default() {
        let _ = super::style();
    }
}
