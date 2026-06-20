//! Sonokai (桜海 / cherry-blossom sea) — Japanese-inspired theme by Sainnhe
//! Park. Dark slate base with pastel accents.

use crate::model::{Color, Theme};

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
        bar_track: Color(234),
        separator: Color(102),
        dim: Color(102),
        reset: Color(209),
        effort_max: Color(147),
        model: Color(185),
    }
}
