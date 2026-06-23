//! Solarized Dark — Ethan Schoonover's meticulously designed color palette.
//! Based on CIELAB hue regularity and precision luminance contrast, it meets
//! the W3C AA contrast ratio guideline.

use crate::model::{Color, Theme};

pub fn theme() -> Theme {
    Theme {
        dir: Color(32),
        git_branch: Color(168),
        ahead: Color(100),
        behind: Color(166),
        modified: Color(136),
        untracked: Color(242),
        token: Color(36),
        bar_ok: Color(100),
        bar_warn: Color(136),
        bar_crit: Color(166),
        bar_track: Color(240),
        separator: Color(240),
        dim: Color(242),
        reset: Color(36),
        effort_max: Color(62),
        model: Color(136),
    }
}
