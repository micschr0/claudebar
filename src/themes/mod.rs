//! Built-in theme registry. The `match` here is **complete** — every built-in
//! theme name is listed up front pointing at its module constructor — so a theme
//! worker only fills in its own `themes/<name>.rs` body and never edits this
//! shared file. That removes the only cross-theme merge conflict.

pub mod catppuccin;
pub mod dracula;
pub mod gruvbox;
pub mod nord;
pub mod rose_pine;
pub mod tokyo_night;

use crate::model::Theme;

/// All built-in theme names, in display order. Tokyo Night is the default.
pub const NAMES: &[&str] = &[
    "tokyo-night",
    "catppuccin",
    "gruvbox",
    "nord",
    "dracula",
    "rose-pine",
];

/// Resolve a theme by name. Unknown names (and the default) fall back to
/// Tokyo Night, the byte-parity anchor.
pub fn get(name: &str) -> Theme {
    match name {
        "catppuccin" => catppuccin::theme(),
        "gruvbox" => gruvbox::theme(),
        "nord" => nord::theme(),
        "dracula" => dracula::theme(),
        "rose-pine" => rose_pine::theme(),
        _ => tokyo_night::theme(),
    }
}
