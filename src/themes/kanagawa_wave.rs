//! Kanagawa Wave — inspired by the ukiyo-e painting "The Great Wave off
//! Kanagawa". Dark with muted jewel tones by rebelot.

use crate::model::{Color, Theme};

pub fn theme() -> Theme {
    Theme {
        dir: Color(110),
        git_branch: Color(103),
        ahead: Color(101),
        behind: Color(131),
        modified: Color(143),
        untracked: Color(242),
        token: Color(110),
        bar_ok: Color(101),
        bar_warn: Color(143),
        bar_crit: Color(131),
        bar_track: Color(240),
        separator: Color(242),
        dim: Color(242),
        reset: Color(66),
        effort_max: Color(169),
        model: Color(143),
    }
}
