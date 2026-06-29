//! One Dark — the iconic Atom editor dark theme. Balanced blue-gray base with
//! classic syntax-highlight colors by the Atom team.

use crate::model::{Color, Theme};

#[must_use]
pub fn theme() -> Theme { Theme {
    dir: Color(75),
    git_branch: Color(176),
    ahead: Color(108),
    behind: Color(168),
    modified: Color(173),
    untracked: Color(241),
    token: Color(75),
    bar_ok: Color(108),
    bar_warn: Color(173),
    bar_crit: Color(168),
    bar_track: Color(237),
    separator: Color(241),
    dim: Color(241),
    reset: Color(73),
    effort: Color(134),
    model: Color(173),
    project: Color(75),
    stash: Color(176),
    lines: Color(241),
    cost: Color(179),    // Warm gold — informational, not alarm
    duration: Color(73),
    clock: Color(108),
    burn: Color(168),
} }
