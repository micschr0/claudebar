//! Progress-bar rendering — pure string building, no allocation beyond the
//! result. Mirrors the bash `make_bar`: a filled run in `fill`, an empty track
//! in `track`, at least one filled cell whenever `pct > 0`.

use crate::model::{Color, RESET};

/// Build a self-colored bar of `width` cells for `pct` percent.
///
/// `pct` may exceed 100 (over-limit); the filled run is clamped to `width`.
pub fn make_bar(
    pct: u32,
    width: u8,
    fill: Color,
    track: Color,
    fill_ch: char,
    empty_ch: char,
) -> String {
    let width = width as u32;
    let mut filled = pct.saturating_mul(width) / 100;
    if filled > width {
        filled = width;
    }
    // At least one filled cell once there's any usage, so a non-zero bar is
    // visually distinct from an empty one.
    if pct > 0 && filled == 0 {
        filled = 1;
    }
    let empty = width - filled;

    let mut out = String::with_capacity(width as usize * 4 + 16);
    out.push_str(&fill.fg());
    for _ in 0..filled {
        out.push(fill_ch);
    }
    out.push_str(&track.fg());
    for _ in 0..empty {
        out.push(empty_ch);
    }
    out.push_str(RESET);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(pct: u32) -> String {
        // Strip ANSI to count cells deterministically.
        let s = make_bar(pct, 6, Color(1), Color(2), '#', '-');
        s.chars().filter(|c| *c == '#' || *c == '-').collect()
    }

    #[test]
    fn zero_is_all_empty() {
        assert_eq!(plain(0), "------");
    }

    #[test]
    fn small_nonzero_gets_one_cell() {
        assert_eq!(plain(1), "#-----");
    }

    #[test]
    fn half() {
        assert_eq!(plain(50), "###---");
    }

    #[test]
    fn full() {
        assert_eq!(plain(100), "######");
    }

    #[test]
    fn over_limit_clamps() {
        assert_eq!(plain(150), "######");
    }
}
