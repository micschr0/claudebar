//! Dracula theme — the canonical Dracula palette
//! (<https://draculatheme.com>) mapped to nearest xterm-256 indices, keeping
//! the slot semantics of [`crate::model::Theme`].
//!
//! Dracula hex → xterm-256 mapping used here:
//!   green #50fa7b → 84 · yellow #f1fa8c → 228 · orange #ffb86c → 215
//!   red #ff5555 → 203 · pink #ff79c6 → 212 · purple #bd93f9 → 141
//!   cyan #8be9fd → 117 · comment #6272a4 → 60
//!
//! Semantic slots: bar_ok green, bar_warn yellow, bar_crit red; ahead green,
//! behind red, modified orange, untracked comment grey; dir purple, git_branch
//! pink, token/reset cyan, model orange, effort pink; separator & bar_track
//! a muted background tone, dim a comment grey.

use crate::model::{Color, Theme};

#[must_use]
pub fn theme() -> Theme {
    Theme {
        dir: Color(141),        // purple #bd93f9
        git_branch: Color(212), // pink #ff79c6
        ahead: Color(84),       // green #50fa7b
        behind: Color(203),     // red #ff5555
        modified: Color(215),   // orange #ffb86c
        untracked: Color(67),   // bumped from 60 for WCAG ≥4.5:1
        token: Color(117),      // cyan #8be9fd
        bar_ok: Color(84),      // green #50fa7b
        bar_warn: Color(228),   // yellow #f1fa8c
        bar_crit: Color(203),   // red #ff5555
        bar_track: Color(241),  // muted background tone
        separator: Color(241),  // muted background tone
        dim: Color(245),        // comment grey
        reset: Color(117),      // cyan #8be9fd
        effort: Color(135),     // Magenta — distinct from git_branch (212)
        model: Color(215),      // orange #ffb86c
        project: Color(141),
        stash: Color(212),
        lines: Color(245),
        cost: Color(228), // Dracula yellow — informational, not alarm
        duration: Color(117),
        clock: Color(84),
        burn: Color(203),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dracula_indices() {
        let t = theme();
        assert_eq!(t.dir, Color(141));
        assert_eq!(t.git_branch, Color(212));
        assert_eq!(t.token, Color(117));
        assert_eq!(t.untracked, Color(67));
    }

    #[test]
    fn bar_thresholds_are_distinct() {
        let t = theme();
        assert_ne!(t.bar_ok, t.bar_warn);
        assert_ne!(t.bar_warn, t.bar_crit);
        assert_ne!(t.bar_ok, t.bar_crit);
    }

    #[test]
    fn loads_default() {
        let _ = theme();
    }
}
