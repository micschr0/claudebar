//! Gruvbox Dark theme. Canonical Gruvbox hex colors mapped to their nearest
//! xterm-256 indices, preserving slot semantics (green/yellow/red thresholds,
//! aqua/blue accents, muted bg tones for separators).
//!
//! Gruvbox Dark → xterm-256 mapping:
//!   green  #b8bb26 → 142 · yellow #fabd2f → 214 · red   #fb4934 → 167
//!   blue   #83a598 → 109 · aqua   #8ec07c → 108 · orange #fe8019 → 208
//!   gray   #928374 → 245 · bg3    #665c54 → 241 · bg4    #7c6f64 → 243
//!   purple #d3869b → 175

use crate::model::{Color, Theme};

#[must_use]
pub fn theme() -> Theme {
    Theme {
        dir: Color(109),        // gruvbox bright blue
        git_branch: Color(108), // gruvbox aqua
        ahead: Color(142),      // gruvbox green
        behind: Color(167),     // gruvbox red
        modified: Color(214),   // gruvbox yellow/orange
        untracked: Color(245),  // gruvbox gray
        token: Color(109),      // gruvbox bright blue
        bar_ok: Color(142),     // gruvbox green
        bar_warn: Color(214),   // gruvbox yellow
        bar_crit: Color(167),   // gruvbox red
        bar_track: Color(241),  // gruvbox bg3 (muted)
        separator: Color(241),  // gruvbox bg (dim gray)
        dim: Color(245),        // gruvbox gray
        reset: Color(108),      // gruvbox aqua
        effort: Color(175),     // gruvbox bright purple accent
        model: Color(208),      // gruvbox orange
        project: Color(109),
        stash: Color(108),
        lines: Color(245),
        cost: Color(214), // Gruvbox yellow — informational, not alarm
        duration: Color(108),
        clock: Color(142),
        burn: Color(167),
    }
}

#[cfg(test)]
mod tests {
    use super::theme;

    #[test]
    fn threshold_bars_are_distinct() {
        let t = theme();
        assert_ne!(t.bar_ok, t.bar_warn);
        assert_ne!(t.bar_warn, t.bar_crit);
        assert_ne!(t.bar_ok, t.bar_crit);
    }

    #[test]
    fn slots_use_gruvbox_indices() {
        let t = theme();
        assert_eq!(t.bar_ok.0, 142); // green
        assert_eq!(t.bar_warn.0, 214); // yellow
        assert_eq!(t.bar_crit.0, 167); // red
        assert_eq!(t.dir.0, 109); // blue
        assert_eq!(t.model.0, 208); // orange
    }

    #[test]
    fn loads_default() {
        let _ = theme();
    }
}
