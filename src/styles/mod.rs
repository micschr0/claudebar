//! Built-in style registry. Like the theme registry, the `match` is complete up
//! front so a style worker only fills its own file body and never edits this
//! shared list.

pub mod ascii;
pub mod minimal;
pub mod plain;
pub mod powerline;
pub mod rounded;

use crate::model::Style;

/// All built-in style names, in display order. Powerline is the default.
pub const NAMES: &[&str] = &["powerline", "plain", "rounded", "minimal", "ascii"];

/// Resolve a style by name. Unknown names (and the default) fall back to
/// Powerline.
pub fn get(name: &str) -> Style {
    match name {
        "plain" => plain::style(),
        "rounded" => rounded::style(),
        "minimal" => minimal::style(),
        "ascii" => ascii::style(),
        _ => powerline::style(),
    }
}
