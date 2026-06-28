//! Ayu Mirage — warm dusk palette with soft contrasts. Based on the Ayu theme
//! family by John-Paul Bader.

use crate::model::{Color, Theme};

pub fn theme() -> Theme {
    Theme {
        dir: Color(81),
        git_branch: Color(183),
        ahead: Color(113),
        behind: Color(210),
        modified: Color(221),
        untracked: Color(242),
        token: Color(117),
        bar_ok: Color(113),
        bar_warn: Color(221),
        bar_crit: Color(210),
        bar_track: Color(237),
        separator: Color(242),
        dim: Color(242),
        reset: Color(116),
        effort: Color(213),
        model: Color(215),
        project: Color(81),
        stash: Color(183),
        lines: Color(242),
        cost: Color(221),     // Warm yellow — informational, not alarm
        duration: Color(116),
        clock: Color(113),
        burn: Color(210),
    }
}
