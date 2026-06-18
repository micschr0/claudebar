//! ascii style — pure-ASCII fallback for fonts without Nerd glyphs.
//!
//! Pipe separator, `icons: false`, an ASCII-only glyph set, and `#`/`-` bar
//! chars. Safe everywhere; nothing here requires a special font.

use crate::model::{GlyphSet, Style};

pub fn style() -> Style {
    Style {
        separator: "|",
        icons: false,
        glyphs: GlyphSet {
            branch: "",
            ahead: "^",
            behind: "v",
            modified: "M",
            untracked: "?",
            context: "",
            token: "#",
            clock: "",
            weekly: "W",
            reset: "~",
            model: "@",
            effort: "*",
        },
        bar_fill: '#',
        bar_empty: '-',
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ascii_is_pure_ascii_fallback() {
        let s = super::style();
        assert!(!s.icons);
        assert_eq!(s.separator, "|");
        assert_eq!(s.bar_fill, '#');
        assert_eq!(s.bar_empty, '-');
        assert_eq!(s.glyphs.ahead, "^");
        assert_eq!(s.glyphs.behind, "v");
        assert_eq!(s.glyphs.token, "#");
        assert_eq!(s.glyphs.model, "@");
    }
}
