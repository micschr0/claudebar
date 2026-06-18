//! minimal style — middle-dot separator, no icons, no Nerd glyphs rendered.
//!
//! `icons: false` makes `SegmentWriter.icon()` a no-op, so the glyph set never
//! renders; it carries the powerline set only to satisfy the struct. Separator
//! is a middle dot (`·`).

use crate::model::Style;

pub fn style() -> Style {
    Style {
        separator: "\u{b7}", // · middle dot
        icons: false,
        glyphs: super::powerline::style().glyphs,
        bar_fill: '\u{2501}',  // ━ heavy horizontal
        bar_empty: '\u{254c}', // ╌ light double dash
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn minimal_drops_icons_and_uses_middle_dot() {
        let s = super::style();
        assert!(!s.icons);
        assert_eq!(s.separator, "\u{b7}");
        assert_eq!(s.bar_fill, '\u{2501}');
        assert_eq!(s.bar_empty, '\u{254c}');
    }
}
