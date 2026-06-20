//! GitHub Dark — based on GitHub's 2023 dark mode color system. Clean,
//! familiar, high-readability contrast from GitHub / Primer.

use crate::model::{Color, Theme};

pub fn theme() -> Theme {
    Theme {
        dir: Color(75),
        git_branch: Color(141),
        ahead: Color(71),
        behind: Color(209),
        modified: Color(172),
        untracked: Color(243),
        token: Color(111),
        bar_ok: Color(71),
        bar_warn: Color(172),
        bar_crit: Color(209),
        bar_track: Color(239),
        separator: Color(243),
        dim: Color(243),
        reset: Color(80),
        effort_max: Color(183),
        model: Color(172),
    }
}
