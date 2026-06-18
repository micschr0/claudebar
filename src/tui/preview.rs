//! Live preview: render a sample input through the *real* render path, then turn
//! the resulting ANSI string into a ratatui `Text` so the preview can never
//! diverge from the hook's output.

use crate::model::Config;
use crate::render::render_with;
use crate::tui::sample::Sample;
use crate::{styles, themes};
use ansi_to_tui::IntoText;
use ratatui::text::Text;

/// Fixed "now" (epoch seconds) so reset countdowns are stable across renders.
pub const FIXED_NOW: i64 = 1_899_990_000;

/// Home prefix used for fish-style directory abbreviation in the preview.
pub const PREVIEW_HOME: &str = "/home/me";

/// Render `sample` under `cfg` and convert the ANSI output to a ratatui `Text`.
pub fn render(cfg: &Config, sample: &Sample) -> Text<'static> {
    let theme = themes::get(&cfg.theme);
    let style = styles::get(&cfg.style);
    let ansi = render_with(
        &sample.input,
        cfg,
        &theme,
        &style,
        FIXED_NOW,
        Some(PREVIEW_HOME),
    );
    ansi.into_text()
        .unwrap_or_else(|_| Text::raw(ansi.replace('\u{1b}', "")))
}
