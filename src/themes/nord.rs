//! Nord theme — the arctic, north-bluish palette (Polar Night / Snow Storm /
//! Frost / Aurora). Each slot maps a canonical Nord hex to its nearest
//! xterm-256 index while preserving the slot *semantics* (see
//! `themes/tokyo_night.rs` and `model/palette.rs`).
//!
//! Nord hex → slot mapping:
//!   dir        nord10 #5e81ac → 67  (Frost blue)
//!   git_branch nord8  #88c0d0 → 110 (Frost cyan)
//!   ahead      nord14 #a3be8c → 150 (Aurora green)
//!   behind     nord11 #bf616a → 131 (Aurora red)
//!   modified   nord13 #ebcb8b → 222 (Aurora yellow)
//!   untracked  nord3  #4c566a → 240 (Polar Night grey)
//!   token      nord7  #8fbcbb → 108 (Frost teal)
//!   bar_ok     nord14 #a3be8c → 150 (Aurora green)
//!   bar_warn   nord13 #ebcb8b → 222 (Aurora yellow)
//!   bar_crit   nord11 #bf616a → 131 (Aurora red)
//!   bar_track  nord1  #3b4252 → 239 (Polar Night)
//!   separator  nord1  #3b4252 → 239 (Polar Night)
//!   dim        nord3-ish grey → 245
//!   reset      nord8  #88c0d0 → 110 (Frost cyan)
//!   effort_max nord15 #b48ead → 139 (Aurora purple)
//!   model      nord15 #b48ead → 139 (Aurora purple)

use crate::model::{Color, Theme};

pub fn theme() -> Theme {
    Theme {
        dir: Color(67),
        git_branch: Color(110),
        ahead: Color(150),
        behind: Color(131),
        modified: Color(222),
        untracked: Color(240),
        token: Color(108),
        bar_ok: Color(150),
        bar_warn: Color(222),
        bar_crit: Color(131),
        bar_track: Color(239),
        separator: Color(239),
        dim: Color(245),
        reset: Color(110),
        effort_max: Color(139),
        model: Color(139),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dir_is_frost_blue() {
        assert_eq!(theme().dir, Color(67));
    }

    #[test]
    fn git_branch_is_frost_cyan() {
        assert_eq!(theme().git_branch, Color(110));
    }

    #[test]
    fn model_is_aurora_purple() {
        assert_eq!(theme().model, Color(139));
    }

    #[test]
    fn bar_thresholds_are_three_distinct_values() {
        let t = theme();
        assert_ne!(t.bar_ok, t.bar_warn);
        assert_ne!(t.bar_warn, t.bar_crit);
        assert_ne!(t.bar_ok, t.bar_crit);
    }

    #[test]
    fn dir_differs_from_tokyo_night() {
        assert_ne!(theme().dir, super::super::tokyo_night::theme().dir);
    }
}
