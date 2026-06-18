//! plain style — pipe separators, full Nerd glyphs, same bars as powerline.
//!
//! Identical to powerline except the separator is a plain `|` (the composer adds
//! a space on each side, so it renders as ` | `). Glyphs and bar chars are
//! reused from powerline.

use crate::model::Style;

pub fn style() -> Style {
    Style {
        separator: "|",
        icons: true,
        glyphs: super::powerline::style().glyphs,
        bar_fill: '\u{2501}',  // ━ heavy horizontal
        bar_empty: '\u{254c}', // ╌ light double dash
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn plain_uses_pipe_separator_with_icons() {
        let s = super::style();
        assert!(s.icons);
        assert_eq!(s.separator, "|");
        assert_eq!(s.bar_fill, '\u{2501}');
        assert_eq!(s.bar_empty, '\u{254c}');
    }
}
