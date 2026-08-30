//! `claudebar` — a Powerline-style status line for Claude Code, with a TUI
//! configurator, built-in themes and styles.
//!
//! The render path (`render_line`) and the TUI both build on the same
//! [`model`] contract and [`render`] composition, so the live preview can never
//! diverge from what the hook emits.

#![deny(clippy::correctness)]
#![warn(clippy::suspicious)]
#![warn(clippy::style)]
#![warn(clippy::complexity)]
#![warn(clippy::perf)]
// Prose mentions of identifiers like "C", "Rust", "JSON", "TOML", or segment
// names don't need backticks in this codebase; clippy's `doc_markdown` lint
// would flag ~80 false positives across module/function docs.
#![allow(clippy::doc_markdown)]
// These pedantic lints fire on patterns this codebase uses intentionally:
// - items_after_statements: imports after the first expression are common here
//   (coercion impls, fixture builders).
// - similar_names: i/j/k iterators and adjacent (a, b) bindings are intentional.
// - too_many_lines: a few render helpers and tui event handlers cross 100 lines
// - single_match / single_match_else: a few match blocks have meaningful
//   else arms (early-return, log + bail) that read better than nested if let.
// - unnested_or_patterns: the tui KeyCode matchers interleave Char with
//   named variants (Up/Down/Tab/BackTab/Esc); nesting them all would lose
//   the variant grouping that makes the dispatch readable.
#![allow(
    clippy::items_after_statements,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::single_match,
    clippy::single_match_else,
    clippy::unnested_or_patterns,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

/// Declare a name → value registry from one list.
///
/// Generates `NAMES` (display order) and `get()` (name lookup with a fallback)
/// from a single set of entries, so a name can never exist without a matching
/// arm. That failure mode is not hypothetical: `rounded` sat in `styles::NAMES`
/// with no arm in `get()` for its entire life, silently resolving to Powerline,
/// and the tests of the day could not see it.
macro_rules! registry {
    ($ty:ty, $fallback:ident, $($name:literal => $konst:ident),+ $(,)?) => {
        /// Every built-in name, in display order.
        pub const NAMES: &[&str] = &[$($name),+];

        /// Resolve a name. Unknown names fall back to the default.
        #[must_use]
        pub fn get(name: &str) -> $ty {
            match name {
                $($name => $konst,)+
                _ => $fallback,
            }
        }

        #[cfg(test)]
        mod registry_tests {
            use super::*;

            /// Every name must resolve to its *own* value. A name whose arm is
            /// missing falls through to the default and collides with it —
            /// exactly the `rounded` bug, now impossible to reintroduce.
            #[test]
            fn every_name_resolves_to_a_distinct_value() {
                let mut seen: Vec<(&str, $ty)> = Vec::new();
                for &name in NAMES {
                    let got = get(name);
                    if let Some((other, _)) = seen.iter().find(|(_, v)| *v == got) {
                        panic!(
                            "{name:?} resolves to the same value as {other:?} — \
                             missing match arm, or two entries share a constant"
                        );
                    }
                    seen.push((name, got));
                }
                assert_eq!(seen.len(), NAMES.len());
            }

            #[test]
            fn an_unknown_name_falls_back() {
                assert_eq!(get("no-such-entry"), $fallback);
            }
        }
    };
}
pub(crate) use registry;

pub mod model;
pub mod paths;
pub mod render;
pub mod sanitize;
pub mod segment;
pub mod setup;
pub mod styles;
pub mod themes;
pub mod update;

#[cfg(feature = "tui")]
pub mod tui;

pub use model::{Config, InputData};
pub use render::render_line;
