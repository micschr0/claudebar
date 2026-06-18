//! Powerline — the default style and **byte-parity anchor** for glyphs. Every
//! glyph reproduces the exact codepoint the bash script emitted.
//!
//! Separator U+E0B1, branch U+E0A0, context U+F035B, token U+2B21,
//! clock U+F051F, weekly U+F00ED, reset U+21BA, model U+25C8, effort U+F0E7,
//! ahead U+2191, behind U+2193. Requires a Nerd Font / powerline-patched font.

use crate::model::{GlyphSet, Style};

pub fn style() -> Style {
    Style {
        separator: "\u{e0b1}",
        icons: true,
        glyphs: GlyphSet {
            branch: "\u{e0a0}",
            ahead: "\u{2191}",
            behind: "\u{2193}",
            modified: "M",
            untracked: "?",
            context: "\u{f035b}",
            token: "\u{2b21}",
            clock: "\u{f051f}",
            weekly: "\u{f00ed}",
            reset: "\u{21ba}",
            model: "\u{25c8}",
            effort: "\u{f0e7}",
        },
        bar_fill: '\u{2501}',  // ━ heavy horizontal
        bar_empty: '\u{254c}', // ╌ light double dash
    }
}
