//! Powerline — the default style and **byte-parity anchor** for glyphs. Every
//! glyph reproduces the exact codepoint the bash script emitted.
//!
//! Separator U+E0B1, branch U+E0A0, context U+F035B, token U+F0C29,
//! clock U+F051F, weekly U+F00ED, reset U+21BA, model U+25C8, effort U+F0E7,
//! ahead U+2191, behind U+2193, project U+2394, agent U+2699, duration U+F2F2.
//! Requires a Nerd Font / powerline-patched font.

use crate::model::{GlyphSet, Style};

#[must_use]
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
            token: "\u{f0c29}", // nf-md-hexagon_outline — outline hexagon present in Nerd Fonts (U+2B21 ⬡ is not)
            clock: "\u{f051f}",
            weekly: "\u{f00ed}",
            reset: "\u{21ba}",
            model: "\u{25c8}",
            effort: "\u{f0e7}",
            worktree: "\u{f126}",     // U+F126 nf-fa-code-fork
            pull_request: "\u{f407}", // U+F407 nf-oct-git-pull-request
            agent: "\u{2699}",        // U+2699 ⚙ gear
            project: "\u{2394}",      // U+2394 ⎔ software-function symbol
            stash: "\u{2691}",        // U+2691 ⚑
            lines: "\u{2013}",        // U+2013 – en dash (lines removed marker)
            cost: "$",                // plain dollar
            duration: "\u{f2f2}", // nf-fa-stopwatch
            time: "\u{f051f}",     // nf-fa-clock — same Nerd Font clock as rate-limits window
            burn: "\u{2197}",      // U+2197 ↗
        },
        bar_fill: '\u{2501}',  // ━ heavy horizontal
        bar_empty: '\u{254c}', // ╌ light double dash
    }
}
