//! Built-in style registry. All styles live in this file as `pub const`
//! values; no per-style module, no merge-conflict surface.

use crate::model::{GlyphSet, Style};

/// All built-in style names, in display order. Powerline is the default.
pub const NAMES: &[&str] = &[
    "powerline",
    "lean",
    "plain",
    "rounded",
    "minimal",
    "unicode",
    "ascii",
];

/// powerline style.
pub const POWERLINE: Style = Style {
    separator: "\u{e0b1}",
    window_gap: "\u{b7}",
    icons: true,
    bar_fill: '\u{2501}',
    bar_empty: '\u{254c}',
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
        agent: "\u{2699}",
        project: "\u{2394}",
        stash: "\u{2691}",
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
    glyphs: POWERLINE.glyphs,
};

/// plain style.
pub const PLAIN: Style = Style {
    separator: "|",
    window_gap: ":",
    icons: false,
    bar_fill: '#',
    bar_empty: '-',
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
        worktree: "+",
        pull_request: "#",
        agent: "&",
        project: "P",
        stash: "s",
        lines: "-",
        cost: "$",
        duration: "d",
        time: "T",
        burn: "B",
    },
};

/// rounded style.
pub const ROUNDED: Style = Style {
    separator: "\u{e0b5}",
    window_gap: "\u{b7}",
    icons: true,
    bar_fill: '\u{2501}',
    bar_empty: '\u{254c}',
    glyphs: POWERLINE.glyphs,
};

/// minimal style.
pub const MINIMAL: Style = Style {
    separator: "\u{b7}",
    window_gap: ":",
    icons: false,
    bar_fill: '\u{2501}',
    bar_empty: '\u{254c}',
    glyphs: POWERLINE.glyphs,
};

/// unicode style.
pub const UNICODE: Style = Style {
    separator: "❯",
    window_gap: "·",
    icons: true,
    bar_fill: '█',
    bar_empty: '░',
    glyphs: GlyphSet {
        branch: "⎇",
        ahead: "↑",
        behind: "↓",
        modified: "±",
        untracked: "?",
        context: "◉",
        token: "⬡",
        clock: "◷",
        weekly: "◈",
        reset: "↺",
        model: "▪",
        effort: "⚡",
        worktree: "⑂",
        pull_request: "⇐",
        agent: "⚙",
        project: "⎔",
        stash: "⚑",
        lines: "–",
        cost: "$",
        duration: "⏱",
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
        agent: "&",
        project: "P",
        stash: "s",
        lines: "-",
        cost: "$",
        duration: "d",
        time: "T",
        burn: "B",
    },
};

/// Resolve a style by name. Unknown names fall back to Powerline.
#[must_use]
pub fn get(name: &str) -> Style {
    match name {
        "lean" => LEAN,
        "plain" => PLAIN,
        "minimal" => MINIMAL,
        "unicode" => UNICODE,
        "ascii" => ASCII,
        _ => POWERLINE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_known_names_resolve() {
        for n in NAMES {
            let _ = get(n);
        }
    }
    #[test]
    fn unknown_falls_back_to_powerline() {
        assert_eq!(get("nope").separator, POWERLINE.separator);
    }
}
