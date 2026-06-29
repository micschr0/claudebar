//! Everforest Dark — warm earthy greens and muted yellows. Designed for
//! reduced eye strain by Sainnhe Park.

use crate::model::{Color, Theme};

#[must_use]
pub fn theme() -> Theme { Theme {
    dir: Color(73),
    git_branch: Color(175),
    ahead: Color(108),
    behind: Color(174),
    modified: Color(180),
    untracked: Color(245),
    token: Color(115),
    bar_ok: Color(113),
    bar_warn: Color(180),
    bar_crit: Color(174),
    bar_track: Color(237),
    separator: Color(245),
    dim: Color(245),
    reset: Color(108),
    effort: Color(179),
    model: Color(180),
    project: Color(73),
    stash: Color(175),
    lines: Color(245),
    cost: Color(179),     // Warm tan — informational, not alarm
    duration: Color(116), // Cyan — distinct from clock (108)
    clock: Color(113),
    burn: Color(174),
} }
