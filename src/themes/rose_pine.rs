//! Rosé Pine (main variant) — a soho-vibe palette of muted, natural tones.
//!
//! Canonical Rosé Pine hex → nearest xterm-256 index, preserving slot semantics:
//!   love #eb6f92 → 211 · gold #f6c177 → 222 · rose #ebbcba → 181
//!   pine #31748f → 66 · foam #9ccfd8 → 116 · iris #c4a7e7 → 183
//!   muted #6e6a86 → 60 · subtle #908caa → 103 · text #e0def4 → 189
//!
//! Semantic mapping:
//!   bar_ok = foam (green-leaning teal) · bar_warn = gold · bar_crit = love
//!   ahead = foam · behind = love · modified = gold · untracked = muted
//!   separator & bar_track = muted overlay tone · dim = subtle
//!   dir = foam · git_branch = iris · token = foam · model = rose
//!   reset = pine · effort_max = iris

use crate::model::{Color, Theme};

pub fn theme() -> Theme {
    Theme {
        dir: Color(116),        // foam  #9ccfd8
        git_branch: Color(183), // iris  #c4a7e7
        ahead: Color(116),      // foam  #9ccfd8 (green-ish)
        behind: Color(211),     // love  #eb6f92 (red-ish)
        modified: Color(222),   // gold  #f6c177
        untracked: Color(60),   // muted #6e6a86
        token: Color(116),      // foam  #9ccfd8
        bar_ok: Color(116),     // foam  #9ccfd8
        bar_warn: Color(222),   // gold  #f6c177
        bar_crit: Color(211),   // love  #eb6f92
        bar_track: Color(103), // subtle #908caa (overlay tone, lifted for contrast)
        separator: Color(103), // subtle #908caa
        dim: Color(103),        // subtle #908caa
        reset: Color(66),       // pine  #31748f
        effort_max: Color(183), // iris  #c4a7e7
        model: Color(181),      // rose  #ebbcba
    }
}

#[cfg(test)]
mod tests {
    use super::theme;
    use crate::model::Color;

    #[test]
    fn key_slots_match_rose_pine_mapping() {
        let t = theme();
        assert_eq!(t.dir, Color(116)); // foam
        assert_eq!(t.git_branch, Color(183)); // iris
        assert_eq!(t.behind, Color(211)); // love
        assert_eq!(t.model, Color(181)); // rose
        assert_eq!(t.reset, Color(66)); // pine
    }

    #[test]
    fn bar_thresholds_are_distinct() {
        let t = theme();
        assert_ne!(t.bar_ok, t.bar_warn);
        assert_ne!(t.bar_warn, t.bar_crit);
        assert_ne!(t.bar_ok, t.bar_crit);
    }
}
