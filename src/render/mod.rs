//! Composition: turn (input × config) into one ANSI status line.
//!
//! `render_line` is the single entrypoint shared by the hook and the TUI
//! preview — there is no second rendering code path.

pub mod bar;
pub mod writer;

pub use writer::SegmentWriter;

use crate::model::{Config, InputData, RESET, Style, Theme};
use crate::segment::RenderCtx;
use crate::{styles, themes};

/// Render the full status line for `input` under `cfg`, with `now` (epoch
/// seconds) injected for deterministic reset countdowns.
#[must_use]
pub fn render_line(input: &InputData, cfg: &Config, now: i64) -> String {
    let theme = themes::get(&cfg.theme);
    let style = styles::get(&cfg.style);
    let home = std::env::var("HOME").ok();
    render_with(input, cfg, &theme, &style, now, home.as_deref())
}

/// Lower-level entry that takes resolved theme/style/home directly — used by the
/// TUI preview (which already holds them) and by tests (deterministic `home`).
#[must_use]
pub fn render_with(
    input: &InputData,
    cfg: &Config,
    theme: &Theme,
    style: &Style,
    now: i64,
    home: Option<&str>,
) -> String {
    let ctx = RenderCtx {
        input,
        theme,
        style,
        th: &cfg.thresholds,
        now,
        home,
    };

    let mut line = String::with_capacity(256);
    let mut first = true;
    for kind in &cfg.segments {
        let mut w = SegmentWriter::new(theme, style);
        let emitted = kind.as_segment().render(&ctx, &mut w);
        if emitted && !w.is_empty() {
            if !first {
                separator(&mut line, theme, style);
            }
            line.push_str(w.as_str());
            first = false;
        }
    }
    line
}

/// Append the separator between two adjacent non-empty segments directly into
/// `line`: a space, the style's separator glyph painted in the theme's separator
/// color, then a space.
fn separator(line: &mut String, theme: &Theme, style: &Style) {
    line.push(' ');
    theme.separator.write_fg(line);
    line.push_str(style.separator);
    line.push_str(RESET);
    line.push(' ');
}
