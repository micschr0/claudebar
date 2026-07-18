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

pub mod model;
pub mod render;
pub mod sanitize;
pub mod segment;
pub mod setup;
pub mod styles;
pub mod themes;

#[cfg(feature = "tui")]
pub mod tui;

pub use model::{Config, InputData};
pub use render::render_line;
