//! Catppuccin Mocha — the soothing pastel dark variant. Each slot maps a
//! canonical Catppuccin Mocha hex color to its nearest xterm-256 index,
//! preserving slot semantics (green/yellow/red bars, blue dir, mauve branch …).

use crate::model::{Color, Theme};

#[must_use]
pub fn theme() -> Theme {
    Theme {
        // Blue #89b4fa
        dir: Color(111),
        // Mauve #cba6f7
        git_branch: Color(183),
        // Green #a6e3a1
        ahead: Color(150),
        // Red #f38ba8
        behind: Color(211),
        // Peach #fab387
        modified: Color(216),
        // Overlay1 #7f849c — dim grey
        untracked: Color(244),
        // Sapphire #74c7ec
        token: Color(117),
        // Green #a6e3a1
        bar_ok: Color(150),
        // Yellow #f9e2af
        bar_warn: Color(223),
        // Red #f38ba8
        bar_crit: Color(204),
        // Surface2 #585b70 — muted surface tone
        bar_track: Color(241),
        // Surface2 #585b70 — muted surface tone
        separator: Color(241),
        // Overlay0 #6c7086 — subtext/overlay grey
        dim: Color(244),
        // Teal #94e2d5
        reset: Color(115),
        // Pink #f5c2e7 — vivid accent
        effort: Color(218),
        // Lavender (Catppuccin Mauve neighbour, same family) — distinct from git_branch
        model: Color(140),
        project: Color(111),
        stash: Color(183),
        lines: Color(244),
        cost: Color(223), // Gold — informational, not alarm
        duration: Color(115),
        clock: Color(150),
        // same as bar_crit
        burn: Color(204),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_key_slots() {
        let t = theme();
        assert_eq!(t.dir, Color(111));
        assert_eq!(t.git_branch, Color(183));
        assert_eq!(t.bar_ok, Color(150));
        assert_eq!(t.bar_warn, Color(223));
        assert_eq!(t.bar_crit, Color(204));
        assert_eq!(t.effort, Color(218));
    }

    #[test]
    fn bar_thresholds_are_distinct() {
        let t = theme();
        assert_ne!(t.bar_ok, t.bar_warn);
        assert_ne!(t.bar_warn, t.bar_crit);
        assert_ne!(t.bar_ok, t.bar_crit);
    }

    #[test]
    fn dir_differs_from_tokyo_night() {
        assert_ne!(theme().dir, super::super::tokyo_night::theme().dir);
    }

    #[test]
    fn loads_default() {
        let _ = theme();
    }
}
