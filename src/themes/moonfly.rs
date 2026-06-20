//! Moonfly — dark theme with a slightly bluish-gray tint by bluz71. Green and
//! blue accents on near-black background.

use crate::model::{Color, Theme};

pub fn theme() -> Theme {
    Theme {
        dir: Color(111),
        git_branch: Color(176),
        ahead: Color(113),
        behind: Color(203),
        modified: Color(186),
        untracked: Color(246),
        token: Color(111),
        bar_ok: Color(113),
        bar_warn: Color(186),
        bar_crit: Color(203),
        bar_track: Color(236),
        separator: Color(246),
        dim: Color(246),
        reset: Color(116),
        effort_max: Color(141),
        model: Color(186),
    }
}
