//! Sonokai (桜海 / cherry-blossom sea) — Japanese-inspired theme by Sainnhe
//! Park. Dark slate base with pastel accents.

use crate::model::{Color, Theme};

#[must_use]
pub fn theme() -> Theme {
    Theme {
        dir: Color(116),
        git_branch: Color(147),
        ahead: Color(149),
        behind: Color(204),
        modified: Color(185),
        untracked: Color(102),
        token: Color(116),
        bar_ok: Color(149),
        bar_warn: Color(185),
        bar_crit: Color(204),
        bar_track: Color(241),
        separator: Color(102),
        dim: Color(102),
        reset: Color(209),
        effort: Color(141),
        model: Color(185),
        project: Color(116),
        stash: Color(147),
        lines: Color(102),
        cost: Color(222), // Warm gold — informational, not alarm
        duration: Color(209),
        clock: Color(149),
        burn: Color(204),
    }
}
