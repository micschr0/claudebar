//! Night Owl — deep blue-black background with bright, accessible accents.
//! Created by Sarah Drasner for VS Code.

use crate::model::{Color, Theme};

#[must_use]
pub fn theme() -> Theme { Theme {
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
    bar_track: Color(236),
    separator: Color(240),
    dim: Color(240),
    reset: Color(43),
    effort: Color(134),
    model: Color(186),
    project: Color(111),
    stash: Color(176),
    lines: Color(240),
    cost: Color(222),    // Warm gold — informational, not alarm
    duration: Color(43),
    clock: Color(41),
    burn: Color(203),
} }
