//! unicode style — rich Unicode glyphs, no Nerd Font required.
//!
//! Uses only standard Unicode codepoints (no PUA block). Suitable for any
//! terminal with a modern Unicode font. Separator is `❯`, branch is `⎇`,
//! bars use block-drawing characters. `icons: true` so glyphs render.

use crate::model::{GlyphSet, Style};

pub fn style() -> Style {
    Style {
        separator: "❯",       // U+276F HEAVY RIGHT-POINTING ANGLE QUOTATION MARK ORNAMENT
        icons: true,
        glyphs: GlyphSet {
            branch: "⎇",      // U+2387 ALTERNATIVE KEY SYMBOL
            ahead: "↑",       // U+2191 UPWARDS ARROW
            behind: "↓",      // U+2193 DOWNWARDS ARROW
            modified: "±",    // U+00B1 PLUS-MINUS SIGN
            untracked: "?",
            context: "◉",     // U+25C9 FISHEYE
            token: "⬡",       // U+2B21 WHITE HEXAGON
            clock: "◷",       // U+25F7 WHITE CIRCLE WITH UPPER RIGHT QUADRANT
            weekly: "◈",      // U+25C8 WHITE DIAMOND CONTAINING BLACK SMALL DIAMOND
            reset: "↺",       // U+21BA ANTICLOCKWISE OPEN CIRCLE ARROW
            model: "◆",       // U+25C6 BLACK DIAMOND
            effort: "⚡",      // U+26A1 HIGH VOLTAGE SIGN
            worktree: "⑂",    // U+2442 OCR FORK OR BRANCH
            pull_request: "⇐", // U+21D0 LEFTWARDS DOUBLE ARROW
            agent: "⚙",       // U+2699 GEAR
        },
        bar_fill: '█',        // U+2588 FULL BLOCK
        bar_empty: '░',       // U+2591 LIGHT SHADE
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn unicode_uses_no_pua_codepoints() {
        let s = super::style();
        assert!(s.icons);
        assert_eq!(s.separator, "❯");
        assert_eq!(s.glyphs.branch, "⎇");
        assert_eq!(s.bar_fill, '█');
        assert_eq!(s.bar_empty, '░');
        // Verify no PUA codepoints (U+E000–U+F8FF or U+F0000+)
        let all = [
            s.separator,
            s.glyphs.branch, s.glyphs.ahead, s.glyphs.behind,
            s.glyphs.context, s.glyphs.token, s.glyphs.clock,
            s.glyphs.weekly, s.glyphs.reset, s.glyphs.model,
            s.glyphs.effort, s.glyphs.worktree, s.glyphs.pull_request,
            s.glyphs.agent,
        ];
        for glyph in all {
            for c in glyph.chars() {
                let cp = c as u32;
                assert!(
                    !(0xE000..=0xF8FF).contains(&cp) && cp < 0xF0000,
                    "PUA codepoint U+{cp:04X} found in glyph {glyph:?}"
                );
            }
        }
    }
}
