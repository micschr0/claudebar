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

pub mod model;
pub mod render;
pub mod sanitize;
pub mod segment;
pub mod styles;
pub mod themes;

#[cfg(feature = "tui")]
pub mod tui;

pub use model::{Config, InputData};
pub use render::render_line;
