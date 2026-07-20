//! Progress-bar rendering — pure string building, no allocation beyond the
//! result. Mirrors the bash `make_bar`: a filled run in `fill`, an empty track
//! in `track`, at least one filled cell whenever `pct > 0`.

use crate::model::{Color, RESET};

/// Append a self-colored bar of `width` cells for `pct` percent into `buf`,
/// avoiding the throwaway `String` that [`make_bar`] allocates.
///
/// `pct` may exceed 100 (over-limit); the filled run is clamped to `width`.
#[allow(clippy::too_many_arguments)]
pub fn write_bar(
    buf: &mut String,
    pct: u32,
    width: u8,
    fill: Color,
    track: Color,
    fill_ch: char,
    empty_ch: char,
) {
    let width = u32::from(width);
    let mut filled = pct.saturating_mul(width) / 100;
    if filled > width {
        filled = width;
    }
    // At least one filled cell once there's any usage, so a non-zero bar is
    // visually distinct from an empty one.
    if pct > 0 && width > 0 && filled == 0 {
        filled = 1;
    }
    let empty = width.saturating_sub(filled);

    fill.write_fg(buf);
    for _ in 0..filled {
        buf.push(fill_ch);
    }
    track.write_fg(buf);
    for _ in 0..empty {
        buf.push(empty_ch);
    }
    buf.push_str(RESET);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain_w(pct: u32, width: u8) -> String {
        let mut s = String::new();
        write_bar(&mut s, pct, width, Color(1), Color(2), '#', '-');
        // Strip ANSI to count cells deterministically.
        s.chars().filter(|c| *c == '#' || *c == '-').collect()
    }

    fn plain(pct: u32) -> String {
        plain_w(pct, 6)
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

    #[test]
    fn width_zero_with_zero_pct_is_empty() {
        assert_eq!(plain_w(0, 0), "");
    }

    #[test]
    fn width_zero_with_nonzero_pct_is_empty() {
        assert_eq!(plain_w(50, 0), "");
    }
}
