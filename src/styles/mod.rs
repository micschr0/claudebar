//! Built-in style registry. All styles live in this file as `pub const`
//! values; no per-style module, no merge-conflict surface.

use crate::model::{GlyphSet, Style};

/// powerline style.
pub const POWERLINE: Style = Style {
    separator: "\u{e0b1}",
    window_gap: "\u{b7}",
    icons: true,
    bar_fill: '\u{2501}',
    bar_empty: '\u{254c}',
    bar_dots: None,
    glyphs: GlyphSet {
        branch: "\u{e0a0}",
        ahead: "\u{2191}",
        behind: "\u{2193}",
        modified: "M",
        untracked: "?",
        context: "\u{f035b}",
        token: "\u{f0c29}",
        clock: "\u{f051f}",
        weekly: "\u{f00ed}",
        reset: "\u{21ba}",
        model: "\u{25c8}",
        effort: "\u{f0e7}",
        worktree: "\u{f126}",
        pull_request: "\u{f407}",
        review_ok: "\u{f00c}",
        review_fail: "\u{f00d}",
        agent: "\u{f013}",
        stash: "\u{f187}",
        lines: "\u{2013}",
        cost: "$",
        duration: "\u{f2f2}",
        time: "\u{f051f}",
        burn: "\u{2197}",
    },
};

/// lean style.
pub const LEAN: Style = Style {
    separator: "",
    window_gap: "\u{b7}",
    icons: true,
    bar_fill: '\u{2501}',
    bar_empty: '\u{254c}',
    bar_dots: None,
    glyphs: POWERLINE.glyphs,
};

/// plain style — ASCII with a different worktree marker.
///
/// Byte-identical to [`ASCII`] apart from `worktree`; spelling out all 23
/// glyphs again would just be a second copy to keep in sync.
pub const PLAIN: Style = Style {
    glyphs: GlyphSet {
        worktree: "+",
        ..ASCII.glyphs
    },
    ..ASCII
};

/// rounded style.
pub const ROUNDED: Style = Style {
    separator: "\u{e0b5}",
    window_gap: "\u{b7}",
    icons: true,
    bar_fill: '\u{2501}',
    bar_empty: '\u{254c}',
    bar_dots: None,
    glyphs: POWERLINE.glyphs,
};

/// minimal style.
pub const MINIMAL: Style = Style {
    separator: "\u{b7}",
    window_gap: ":",
    icons: false,
    bar_fill: '\u{2501}',
    bar_empty: '\u{254c}',
    bar_dots: None,
    // Icons-off style: everything else comes from POWERLINE, but the review
    // markers bypass `SegmentWriter::icon`, so they must not be Nerd Font PUA.
    glyphs: GlyphSet {
        review_ok: "\u{221a}",
        review_fail: "\u{d7}",
        ..POWERLINE.glyphs
    },
};

/// unicode style.
pub const UNICODE: Style = Style {
    separator: "❯",
    window_gap: "·",
    icons: true,
    bar_fill: '█',
    bar_empty: '░',
    bar_dots: None,
    glyphs: GlyphSet {
        branch: "↷",
        ahead: "↑",
        behind: "↓",
        modified: "±",
        untracked: "?",
        context: "◉",
        token: "◇",
        clock: "◷",
        weekly: "◈",
        reset: "↺",
        model: "▪",
        effort: "⚡",
        worktree: "↳",
        pull_request: "⇐",
        review_ok: "√",
        review_fail: "×",
        agent: "⊚",
        stash: "▩",
        lines: "–",
        cost: "$",
        duration: "◴",
        time: "◷",
        burn: "↗",
    },
};

/// ascii style.
pub const ASCII: Style = Style {
    separator: "|",
    window_gap: ":",
    icons: false,
    bar_fill: '#',
    bar_empty: '-',
    bar_dots: None,
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
        worktree: ">",
        pull_request: "#",
        review_ok: "+",
        review_fail: "x",
        agent: "&",
        stash: "s",
        lines: "-",
        cost: "$",
        duration: "d",
        time: "T",
        burn: "B",
    },
};

/// dots style — powerline decoration with quarter-step dot-meter bars.
pub const DOTS: Style = Style {
    separator: "\u{e0b1}",
    window_gap: "\u{b7}",
    icons: true,
    bar_fill: '\u{2501}',
    bar_empty: '\u{254c}',
    bar_dots: Some(['\u{25cb}', '\u{25d4}', '\u{25d1}', '\u{25d5}', '\u{25cf}']),
    glyphs: POWERLINE.glyphs,
};

crate::registry! { Style, POWERLINE,
    "powerline" => POWERLINE,
    "lean"      => LEAN,
    "plain"     => PLAIN,
    "rounded"   => ROUNDED,
    "minimal"   => MINIMAL,
    "unicode"   => UNICODE,
    "ascii"     => ASCII,
    "dots"      => DOTS,
}
