//! Cobalt2 — vibrant blue-based theme by Wes Bos. High-contrast with bright
//! accents against a deep navy background.

use crate::model::{Color, Theme};

pub fn theme() -> Theme {
    Theme {
        dir: Color(33),
        git_branch: Color(213),
        ahead: Color(76),
        behind: Color(204),
        modified: Color(220),
        untracked: Color(25),
        token: Color(39),
        bar_ok: Color(76),
        bar_warn: Color(220),
        bar_crit: Color(204),
        bar_track: Color(0),
        separator: Color(25),
        dim: Color(25),
        reset: Color(123),
        effort: Color(205),
        model: Color(220),
        project: Color(33),
        stash: Color(213),
        lines: Color(25),
        cost: Color(215),     // Warm amber — informational, not alarm
        duration: Color(123),
        clock: Color(76),
        burn: Color(204),
    }
}
