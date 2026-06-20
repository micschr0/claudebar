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
