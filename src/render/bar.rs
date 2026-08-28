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

/// Append a self-colored quarter-resolution bar of `width` cells for `pct`
/// percent into `buf`, using `levels` (index 0 = empty cell, 4 = full cell).
///
/// Same contract as [`write_bar`]: `pct` may exceed 100 and is clamped, and a
/// non-zero `pct` always shows at least one quarter so it stays visually
/// distinct from an empty bar.
pub fn write_bar_dots(
    buf: &mut String,
    pct: u32,
    width: u8,
    fill: Color,
    track: Color,
    levels: [char; 5],
) {
    let width = u32::from(width);
    let max = width.saturating_mul(4);
    // Round half up, integer-only, mirroring `write_bar`'s arithmetic.
    let mut quarters = pct
        .saturating_mul(width)
        .saturating_mul(4)
        .saturating_add(50)
        / 100;
    if quarters > max {
        quarters = max;
    }
    if pct > 0 && width > 0 && quarters == 0 {
        quarters = 1;
    }

    fill.write_fg(buf);
    let mut on_track = false;
    for i in 0..width {
        let q = quarters.saturating_sub(i * 4).min(4);
        if q == 0 && !on_track {
            track.write_fg(buf);
            on_track = true;
        }
        buf.push(levels[q as usize]);
    }
    buf.push_str(RESET);
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOTS: [char; 5] = ['o', 'p', 'q', 'r', '#'];

    fn dots_w(pct: u32, width: u8) -> String {
        let mut s = String::new();
        write_bar_dots(&mut s, pct, width, Color(1), Color(2), DOTS);
        s.chars().filter(|c| DOTS.contains(c)).collect()
    }

    fn dots(pct: u32) -> String {
        dots_w(pct, 6)
    }

    #[test]
    fn dots_zero_is_all_empty() {
        assert_eq!(dots(0), "oooooo");
    }

    #[test]
    fn dots_small_nonzero_shows_one_quarter() {
        assert_eq!(dots(1), "pooooo");
    }

    #[test]
    fn dots_quarter_steps() {
        // 6 cells = 24 quarters, so one quarter is ~4.17%.
        assert_eq!(dots(5), "pooooo");
        assert_eq!(dots(9), "qooooo");
        assert_eq!(dots(13), "rooooo");
        assert_eq!(dots(17), "#ooooo");
        assert_eq!(dots(21), "#poooo");
    }

    #[test]
    fn dots_half() {
        assert_eq!(dots(50), "###ooo");
    }

    #[test]
    fn dots_full() {
        assert_eq!(dots(100), "######");
    }

    #[test]
    fn dots_over_limit_clamps() {
        assert_eq!(dots(150), "######");
    }

    #[test]
    fn dots_width_zero_is_empty() {
        assert_eq!(dots_w(0, 0), "");
        assert_eq!(dots_w(50, 0), "");
    }

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
