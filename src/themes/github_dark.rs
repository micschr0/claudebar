//! GitHub Dark — based on GitHub's 2023 dark mode color system. Clean,
//! familiar, high-readability contrast from GitHub / Primer.

use crate::model::{Color, Theme};

#[must_use]
pub fn theme() -> Theme {
    Theme {
        dir: Color(75),
        git_branch: Color(141),
        ahead: Color(71),
        behind: Color(209),
        modified: Color(172),
        untracked: Color(244),
        token: Color(111),
        bar_ok: Color(77),
        bar_warn: Color(172),
        bar_crit: Color(209),
        bar_track: Color(241),
        separator: Color(243),
        dim: Color(244),
        reset: Color(80),
        effort: Color(183),
        model: Color(172),
        project: Color(75),
        stash: Color(141),
        lines: Color(244),
        cost: Color(214), // Warm gold — informational, not alarm
        duration: Color(80),
        clock: Color(77),
        burn: Color(209),
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
