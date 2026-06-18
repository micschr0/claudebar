//! Tokyo Night — the default theme and the **byte-parity anchor**. Every color
//! index here reproduces exactly the constant the original bash script used, so
//! `render --theme tokyo-night --style powerline` matches the shell output.
//!
//! Bash → slot mapping (note: the bash constant *names* don't all line up with
//! their use — these reflect actual usage in `statusline-command.sh`):
//!   C_DIR 33 dir · C_GIT 141 git_branch · C_AHD 114 ahead · C_BHD 167 behind
//!   C_WARN 221 modified(M) & bar_warn · C_DIM 245 untracked & dim
//!   C_TOK 117 token · C_OK 114 bar_ok · C_CRIT 203 bar_crit
//!   C_SEP 238 separator & bar_track · C_RST 73 reset · C_EFF_MAX 213 effort_max
//!   C_MOD 208 model(◈)

use crate::model::{Color, Theme};

pub fn theme() -> Theme {
    Theme {
        dir: Color(33),
        git_branch: Color(141),
        ahead: Color(114),
        behind: Color(167),
        modified: Color(221),
        untracked: Color(245),
        token: Color(117),
        bar_ok: Color(114),
        bar_warn: Color(221),
        bar_crit: Color(203),
        bar_track: Color(238),
        separator: Color(238),
        dim: Color(245),
        reset: Color(73),
        effort_max: Color(213),
        model: Color(208),
    }
}
