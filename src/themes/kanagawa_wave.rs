//! Kanagawa Wave — inspired by the ukiyo-e painting "The Great Wave off
//! Kanagawa". Dark with muted jewel tones by rebelot.

use crate::model::{Color, Theme};

#[must_use]
pub fn theme() -> Theme {
    Theme {
        dir: Color(110),
        git_branch: Color(103),
        ahead: Color(101),
        behind: Color(167),
        modified: Color(143),
        untracked: Color(244),
        token: Color(110),
        bar_ok: Color(107),
        bar_warn: Color(143),
        bar_crit: Color(131),
        bar_track: Color(241),
        separator: Color(242),
        dim: Color(244),
        reset: Color(66),
        effort: Color(169),
        model: Color(143),
        project: Color(110),
        stash: Color(103),
        lines: Color(244),
        cost: Color(180), // Warm gold — informational, not alarm
        duration: Color(66),
        clock: Color(107),
        burn: Color(167),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_default() {
        let _ = theme();
    }
}
