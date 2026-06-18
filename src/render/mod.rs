//! Composition: turn (input × config) into one ANSI status line.
//!
//! `render_line` is the single entrypoint shared by the hook and the TUI
//! preview — there is no second rendering code path.

pub mod bar;
pub mod writer;

pub use writer::SegmentWriter;

use crate::model::{Config, InputData, Style, Theme, RESET};
use crate::segment::RenderCtx;
use crate::{styles, themes};

/// Render the full status line for `input` under `cfg`, with `now` (epoch
/// seconds) injected for deterministic reset countdowns.
pub fn render_line(input: &InputData, cfg: &Config, now: i64) -> String {
    let theme = themes::get(&cfg.theme);
    let style = styles::get(&cfg.style);
    let home = std::env::var("HOME").ok();
    render_with(input, cfg, &theme, &style, now, home.as_deref())
}

/// Lower-level entry that takes resolved theme/style/home directly — used by the
/// TUI preview (which already holds them) and by tests (deterministic `home`).
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
                line.push_str(&separator(theme, style));
            }
            line.push_str(w.as_str());
            first = false;
        }
    }
    line
}

/// The separator placed between two adjacent non-empty segments: a space, the
/// style's separator glyph painted in the theme's separator color, then a space.
fn separator(theme: &Theme, style: &Style) -> String {
    format!(" {}{}{} ", theme.separator.fg(), style.separator, RESET)
}
