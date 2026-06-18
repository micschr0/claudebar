//! rounded style — rounded powerline cap separator, full Nerd glyphs.
//!
//! Like plain but the separator is the rounded powerline right-cap (U+E0B5).
//! Glyphs and bar chars match powerline.

use crate::model::Style;

pub fn style() -> Style {
    Style {
        separator: "\u{e0b5}", // rounded powerline right-cap
        icons: true,
        glyphs: super::powerline::style().glyphs,
        bar_fill: '\u{2501}',  // ━ heavy horizontal
        bar_empty: '\u{254c}', // ╌ light double dash
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rounded_uses_rounded_cap_with_icons() {
        let s = super::style();
        assert!(s.icons);
        assert_eq!(s.separator, "\u{e0b5}");
        assert_eq!(s.bar_fill, '\u{2501}');
        assert_eq!(s.bar_empty, '\u{254c}');
    }
}
