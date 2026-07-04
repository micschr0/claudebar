//! plain style — zero Nerd Font dependency. Pipe separator, no icons, `#`/`-`
//! bar chars. Entirely safe on any terminal — no PUA codepoints, no special
//! glyphs. Segments emit only text labels.

use crate::model::{GlyphSet, Style};

#[must_use]
pub fn style() -> Style {
    Style {
        separator: "|",
        window_gap: ":",
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
        bar_fill: '#',
        bar_empty: '-',
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn plain_is_no_pua_no_icons() {
        let s = super::style();
        assert!(!s.icons);
        assert_eq!(s.separator, "|");
        assert_eq!(s.window_gap, ":");
        assert_eq!(s.bar_fill, '#');
        assert_eq!(s.bar_empty, '-');
        // Verify no PUA codepoints anywhere in the glyph set
        let all = [
            s.window_gap,
            s.glyphs.branch,
            s.glyphs.ahead,
            s.glyphs.behind,
            s.glyphs.context,
            s.glyphs.token,
            s.glyphs.clock,
            s.glyphs.weekly,
            s.glyphs.reset,
            s.glyphs.model,
            s.glyphs.effort,
            s.glyphs.worktree,
            s.glyphs.pull_request,
            s.glyphs.agent,
            s.glyphs.project,
            s.glyphs.stash,
            s.glyphs.lines,
            s.glyphs.cost,
            s.glyphs.duration,
            s.glyphs.time,
            s.glyphs.burn,
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
