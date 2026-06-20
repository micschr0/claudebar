//! Night Owl — deep blue-black background with bright, accessible accents.
//! Created by Sarah Drasner for VS Code.

use crate::model::{Color, Theme};

pub fn theme() -> Theme {
    Theme {
        dir: Color(111),
        git_branch: Color(176),
        ahead: Color(41),
        behind: Color(203),
        modified: Color(186),
        untracked: Color(240),
        token: Color(111),
        bar_ok: Color(41),
        bar_warn: Color(186),
        bar_crit: Color(203),
        bar_track: Color(233),
        separator: Color(240),
        dim: Color(240),
        reset: Color(43),
        effort_max: Color(176),
        model: Color(186),
    }
}
