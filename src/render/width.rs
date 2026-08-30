//! Visible-width computation: strip ANSI SGR escapes, then count terminal
//! columns — 1 per codepoint, 2 for wide CJK/kana/Hangul/emoji, 0 for
//! combining marks. Pure Rust, no external tables or dependencies.
//!
//! Used by the responsive auto-layout path in [`super`] to decide where a
//! status line should wrap.

/// Compute the visible terminal width of `s` in columns.
///
/// Escape sequences are removed by [`crate::render::strip_ansi`] — the same
/// stripper the plain-text float readout uses, so the two cannot disagree about
/// what counts as visible. Each remaining codepoint counts as 1 column by
/// default; wide CJK/kana/Hangul/emoji ranges count as 2, and combining
/// diacritical marks (including variation selectors) count as 0.
#[must_use]
pub fn visible_width(s: &str) -> usize {
    crate::render::strip_ansi(s).chars().map(char_width).sum()
}

/// Column width of a single codepoint. Combining marks resolve to 0 before
/// the wide check so they never inflate a base character's width.
fn char_width(c: char) -> usize {
    let cp = u32::from(c);
    if is_combining(cp) {
        0
    } else if is_wide(cp) {
        2
    } else {
        1
    }
}

/// Combining diacritical marks and variation selectors — zero column width;
/// they modify the preceding base codepoint rather than occupying a cell.
fn is_combining(cp: u32) -> bool {
    matches!(
        cp,
        0x0300..=0x036F   // Combining Diacritical Marks
        | 0x1AB0..=0x1AFF // Combining Diacriticals Extended
        | 0x1DC0..=0x1DFF // Combining Diacriticals Supplement
        | 0x20D0..=0x20FF // Combining Diacriticals for Symbols
        | 0xFE00..=0xFE0F // Variation Selectors 1–16
        | 0xFE20..=0xFE2F // Combining Half Marks
    )
}

/// East-Asian Wide / Fullwidth and emoji ranges — two columns each.
fn is_wide(cp: u32) -> bool {
    matches!(
        cp,
        0x1100..=0x115F     // Hangul Jamo
        | 0x2E80..=0x303E   // CJK radicals, Kangxi, CJK symbols
        | 0x3041..=0x33FF   // Hiragana, Katakana, Bopomofo, Hangul Compat, CJK symbols
        | 0x3400..=0x4DBF   // CJK Unified Ideographs Extension A
        | 0x4E00..=0x9FFF   // CJK Unified Ideographs
        | 0xA000..=0xA4CF   // Yi
        | 0xA960..=0xA97F   // Hangul Jamo Extended-A
        | 0xAC00..=0xD7A3   // Hangul Syllables
        | 0xF900..=0xFAFF   // CJK Compatibility Ideographs
        | 0xFE10..=0xFE19   // Vertical Forms
        | 0xFE30..=0xFE6F   // CJK Compatibility Forms
        | 0xFF00..=0xFFE6   // Fullwidth ASCII / signs
        | 0x1F000..=0x1FAFF // Emoji & pictographs
        | 0x20000..=0x3FFFD // CJK Unified Ideographs Extension B+
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_one_per_char() {
        assert_eq!(visible_width("hello"), 5);
    }

    #[test]
    fn empty_is_zero() {
        assert_eq!(visible_width(""), 0);
    }

    #[test]
    fn cjk_is_two_per_char() {
        // 機 = U+6A5F (CJK Unified Ideograph).
        assert_eq!(visible_width("機"), 2);
        // Three CJK ideographs = six columns.
        assert_eq!(visible_width("日本語"), 6);
    }

    #[test]
    fn kana_and_hangul_are_wide() {
        // Hiragana あ (U+3042) and Hangul 가 (U+AC00).
        assert_eq!(visible_width("あ"), 2);
        assert_eq!(visible_width("가"), 2);
    }

    #[test]
    fn emoji_is_two() {
        // 🚀 = U+1F680; a single emoji occupies two columns.
        assert_eq!(visible_width("🚀"), 2);
        assert_eq!(visible_width("🚀🎉"), 4);
    }

    #[test]
    fn emoji_with_variation_selector_stays_two() {
        // 🚀 followed by U+FE0F (variation selector-16): the VS is width 0.
        assert_eq!(visible_width("🚀\u{FE0F}"), 2);
    }

    #[test]
    fn ansi_escapes_stripped() {
        // \x1b[31m red \x1b[0m — only "red" (3) counts.
        assert_eq!(visible_width("\x1b[31mred\x1b[0m"), 3);
    }

    #[test]
    fn mixed_ansi_and_wide() {
        // The SGR runs are stripped; only the CJK ideograph (2) counts.
        assert_eq!(visible_width("\x1b[32m機\x1b[0m"), 2);
    }

    #[test]
    fn combining_marks_are_zero() {
        // 'e' (1) + combining acute U+0301 (0) = one column.
        assert_eq!(visible_width("e\u{0301}"), 1);
    }

    /// A CSI sequence ends at its first byte in `0x40..=0x7E`, not specifically
    /// at `m`. `\x1b[31a` is therefore a *complete* sequence and only `bc`
    /// remains visible.
    ///
    /// The previous hand-rolled scanner skipped to the next `m` and so ate the
    /// rest of the string here, reporting 0. Nothing observable changed: the
    /// renderer only ever emits SGR (`…m`), so no real status line contains a
    /// sequence that terminates on another byte.
    #[test]
    fn a_csi_sequence_ends_at_its_final_byte_not_at_m() {
        assert_eq!(visible_width("\x1b[31abc"), 2);
    }

    /// A lone ESC at the very end has nothing to terminate it and contributes
    /// no columns.
    #[test]
    fn trailing_lone_escape_is_dropped() {
        assert_eq!(visible_width("ab\x1b"), 2);
    }
}
