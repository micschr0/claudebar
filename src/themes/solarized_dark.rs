//! Solarized Dark — Ethan Schoonover's meticulously designed color palette.
//! Based on CIELAB hue regularity and precision luminance contrast, it meets
//! the W3C AA contrast ratio guideline.

use crate::model::{Color, Theme};

#[must_use]
pub fn theme() -> Theme {
    Theme {
        dir: Color(32),
        git_branch: Color(168),
        ahead: Color(100),
        behind: Color(166),
        modified: Color(136),
        untracked: Color(244),
        token: Color(36),
        bar_ok: Color(108),
        bar_warn: Color(136),
        bar_crit: Color(160),
        bar_track: Color(241),
        separator: Color(241),
        dim: Color(244),
        reset: Color(36),
        effort: Color(69),
        model: Color(136),
        project: Color(32),
        stash: Color(168),
        lines: Color(244),
        cost: Color(178), // Warm gold — informational, not alarm
        duration: Color(36),
        clock: Color(108),
        burn: Color(167),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_default() {
        let _ = theme();
    }
}
