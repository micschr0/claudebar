//! Cobalt2 — vibrant blue-based theme by Wes Bos. High-contrast with bright
//! accents against a deep navy background.

use crate::model::{Color, Theme};

#[must_use]
pub fn theme() -> Theme {
    Theme {
        dir: Color(33),
        git_branch: Color(213),
        ahead: Color(76),
        behind: Color(204),
        modified: Color(220),
        untracked: Color(244),
        token: Color(39),
        bar_ok: Color(76),
        bar_warn: Color(220),
        bar_crit: Color(204),
        bar_track: Color(241),
        separator: Color(26),
        dim: Color(244),
        reset: Color(123),
        effort: Color(205),
        model: Color(220),
        project: Color(33),
        stash: Color(213),
        lines: Color(244),
        cost: Color(215), // Warm amber — informational, not alarm
        duration: Color(123),
        clock: Color(76),
        burn: Color(204),
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
