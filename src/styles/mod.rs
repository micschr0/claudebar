//! Built-in style registry. Like the theme registry, the `match` is complete up
//! front so a style worker only fills its own file body and never edits this
//! shared list.

pub mod ascii;
pub mod lean;
pub mod minimal;
pub mod plain;
pub mod powerline;
pub mod rounded;
pub mod unicode;

use crate::model::Style;

/// All built-in style names, in display order. Powerline is the default.
pub const NAMES: &[&str] = &[
    "powerline",
    "lean",
    "plain",
    "rounded",
    "minimal",
    "unicode",
    "ascii",
];

/// Resolve a style by name. Unknown names (and the default) fall back to
/// Powerline.
#[must_use]
pub fn get(name: &str) -> Style {
    match name {
        "lean" => lean::style(),
        "plain" => plain::style(),
        "rounded" => rounded::style(),
        "minimal" => minimal::style(),
        "unicode" => unicode::style(),
        "ascii" => ascii::style(),
        _ => powerline::style(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_get_returns_known_names() {
        for name in NAMES {
            let _ = get(name);
        }
    }

    #[test]
    fn style_get_none_on_unknown() {
        // Unknown names fall back to Powerline without panicking
        let _ = get("definitely-not-a-real-style");
    }
}
