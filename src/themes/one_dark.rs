//! One Dark — the iconic Atom editor dark theme. Balanced blue-gray base with
//! classic syntax-highlight colors by the Atom team.

use crate::model::{Color, Theme};

pub fn theme() -> Theme {
    Theme {
        dir: Color(75),
        git_branch: Color(176),
        ahead: Color(108),
        behind: Color(168),
        modified: Color(173),
        untracked: Color(241),
        token: Color(75),
        bar_ok: Color(108),
        bar_warn: Color(173),
        bar_crit: Color(168),
        bar_track: Color(235),
        separator: Color(241),
        dim: Color(241),
        reset: Color(73),
        effort_max: Color(176),
        model: Color(173),
    }
}
