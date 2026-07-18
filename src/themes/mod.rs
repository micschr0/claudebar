//! Built-in theme registry. The `match` here is **complete** — every built-in
//! theme name is listed up front pointing at its module constructor — so a theme
//! worker only fills in its own `themes/<name>.rs` body and never edits this
//! shared file. That removes the only cross-theme merge conflict.

pub mod ayu_mirage;
pub mod catppuccin;
pub mod cobalt2;
pub mod dracula;
pub mod everforest_dark;
pub mod github_dark;
pub mod gruvbox;
pub mod kanagawa_wave;
pub mod moonfly;
pub mod night_owl;
pub mod nord;
pub mod one_dark;
pub mod rose_pine;
pub mod solarized_dark;
pub mod sonokai;
pub mod tokyo_night;

use crate::model::Theme;

/// All built-in theme names, in display order. Tokyo Night remains the default.
pub const NAMES: &[&str] = &[
    "tokyo-night",
    "ayu-mirage",
    "catppuccin",
    "cobalt2",
    "everforest-dark",
    "github-dark",
    "gruvbox",
    "kanagawa-wave",
    "moonfly",
    "night-owl",
    "nord",
    "one-dark",
    "dracula",
    "rose-pine",
    "sonokai",
    "solarized-dark",
];

/// Resolve a theme by name. Unknown names (and the default) fall back to
/// Tokyo Night, the byte-parity anchor.
#[must_use]
pub fn get(name: &str) -> Theme {
    match name {
        "ayu-mirage" => ayu_mirage::theme(),
        "cobalt2" => cobalt2::theme(),
        "everforest-dark" => everforest_dark::theme(),
        "github-dark" => github_dark::theme(),
        "kanagawa-wave" => kanagawa_wave::theme(),
        "moonfly" => moonfly::theme(),
        "night-owl" => night_owl::theme(),
        "one-dark" => one_dark::theme(),
        "sonokai" => sonokai::theme(),
        "solarized-dark" => solarized_dark::theme(),
        "catppuccin" => catppuccin::theme(),
        "gruvbox" => gruvbox::theme(),
        "nord" => nord::theme(),
        "dracula" => dracula::theme(),
        "rose-pine" => rose_pine::theme(),
        _ => tokyo_night::theme(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Map a single xterm-256 colour index to its approximate sRGB hex string.
    /// Uses the standard 6×6×6 colour cube for indices 16–231 and the 24-step
    /// grey ramp for indices 232–255.
    fn xterm_to_srgb(idx: u8) -> [u8; 3] {
        match idx {
            0..=15 => {
                // Standard ANSI
                const ANSI: [[u8; 3]; 16] = [
                    [0x00, 0x00, 0x00],
                    [0xcd, 0x00, 0x00],
                    [0x00, 0xcd, 0x00],
                    [0xcd, 0xcd, 0x00],
                    [0x00, 0x00, 0xee],
                    [0xcd, 0x00, 0xcd],
                    [0x00, 0xcd, 0xcd],
                    [0xe5, 0xe5, 0xe5],
                    [0x7f, 0x7f, 0x7f],
                    [0xff, 0x00, 0x00],
                    [0x00, 0xff, 0x00],
                    [0xff, 0xff, 0x00],
                    [0x5c, 0x5c, 0xff],
                    [0xff, 0x00, 0xff],
                    [0x00, 0xff, 0xff],
                    [0xff, 0xff, 0xff],
                ];
                ANSI[idx as usize]
            }
            16..=231 => {
                let n = u32::from(idx - 16);
                let r = n / 36;
                let g = (n % 36) / 6;
                let b = n % 6;
                let comp = |v: u32| -> u8 { if v == 0 { 0 } else { (v * 40 + 55) as u8 } };
                [comp(r), comp(g), comp(b)]
            }
            _ => {
                // 232..=255 grey ramp
                let v = u32::from(idx - 232) * 10 + 8;
                let v = v as u8;
                [v, v, v]
            }
        }
    }

    /// WCAG 2.1 relative luminance of an sRGB colour triplet.
    fn relative_luminance(rgb: [u8; 3]) -> f64 {
        fn linear(c: u8) -> f64 {
            let v = f64::from(c) / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055_f64).powf(2.4)
            }
        }
        0.2126 * linear(rgb[0]) + 0.7152 * linear(rgb[1]) + 0.0722 * linear(rgb[2])
    }

    /// WCAG 2.1 contrast ratio between two sRGB colours.
    fn contrast_ratio(rgb1: [u8; 3], rgb2: [u8; 3]) -> f64 {
        let l1 = relative_luminance(rgb1);
        let l2 = relative_luminance(rgb2);
        let (light, dark) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (light + 0.05) / (dark + 0.05)
    }

    /// The canonical background for all theme contrast measurements.
    const BG: [u8; 3] = [0x10, 0x10, 0x18]; // #101018

    /// Text colour slots that MUST meet ≥4.5:1 contrast against the background.
    const TEXT_SLOTS: &[fn(&Theme) -> crate::model::Color] = &[
        |t| t.dir,
        |t| t.git_branch,
        |t| t.ahead,
        |t| t.behind,
        |t| t.modified,
        |t| t.untracked,
        |t| t.token,
        |t| t.dim,
        |t| t.reset,
        |t| t.effort,
        |t| t.model,
        |t| t.project,
        |t| t.stash,
        |t| t.lines,
        |t| t.cost,
        |t| t.duration,
        |t| t.clock,
        |t| t.burn,
    ];

    /// Decorative colour slots (separators, bar tracks/fills) that MUST meet
    /// ≥3:1 contrast against the background.
    const DECORATIVE_SLOTS: &[fn(&Theme) -> crate::model::Color] = &[
        |t| t.separator,
        |t| t.bar_track,
        |t| t.bar_ok,
        |t| t.bar_warn,
        |t| t.bar_crit,
    ];

    #[test]
    fn wcag_contrast() {
        for &name in NAMES {
            let theme = get(name);
            for slot_fn in TEXT_SLOTS {
                let color = slot_fn(&theme);
                let rgb = xterm_to_srgb(color.0);
                let cr = contrast_ratio(rgb, BG);
                assert!(
                    cr >= 4.5,
                    "theme {name}: text slot xterm {} ({rgb:?}) → {cr:.2}:1 < 4.5:1",
                    color.0
                );
            }
            for slot_fn in DECORATIVE_SLOTS {
                let color = slot_fn(&theme);
                let rgb = xterm_to_srgb(color.0);
                let cr = contrast_ratio(rgb, BG);
                assert!(
                    cr >= 3.0,
                    "theme {name}: decorative slot xterm {} ({rgb:?}) → {cr:.2}:1 < 3.0:1",
                    color.0
                );
            }
        }
    }

    #[test]
    fn theme_get_returns_known_names() {
        for name in NAMES {
            let _ = get(name);
        }
    }

    #[test]
    fn theme_get_none_on_unknown() {
        // Unknown names fall back to Tokyo Night without panicking
        let _ = get("definitely-not-a-real-theme");
    }
}
