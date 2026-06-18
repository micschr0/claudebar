//! Data shapes shared across the crate: parsed input, user config, theme, style.
//! This module owns no I/O and no rendering — it is the contract every segment,
//! theme, and style is written against.

pub mod config;
pub mod input;
pub mod palette;
pub mod style;

pub use config::{Config, ConfigError, SegmentKind, Thresholds};
pub use input::InputData;
pub use palette::{Color, Theme, RESET};
pub use style::{GlyphSet, Style};
