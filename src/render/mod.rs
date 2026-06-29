//! Composition: turn (input × config) into one ANSI status line.
//!
//! `render_line` is the single entrypoint shared by the hook and the TUI
//! preview — there is no second rendering code path.

pub mod bar;
pub mod float;
pub mod width;
pub mod writer;
pub use writer::SegmentWriter;

use crate::model::{Config, InputData, RESET, Style, Theme};
use crate::segment::RenderCtx;
use crate::segment::clock;
use crate::{styles, themes};

/// Render the full status line for `input` under `cfg`, with `now` (epoch
/// seconds) injected for deterministic reset countdowns.
#[must_use]
pub fn render_line(input: &InputData, cfg: &Config, now: i64) -> String {
    let theme = themes::get(&cfg.theme);
    let style = styles::get(&cfg.style);
    let home = std::env::var("HOME").ok();
    let tz_offset = clock::detect_tz_offset();
    let line = render_with(input, cfg, &theme, &style, now, home.as_deref(), tz_offset);
    if cfg.thresholds.float {
        float::emit_float(input, cfg, now, home.as_deref());
    }
    line
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
    tz_offset_seconds: i32,
) -> String {
    let ctx = RenderCtx {
        input,
        theme,
        style,
        th: &cfg.thresholds,
        now,
        home,
        tz_offset_seconds,
    };

    if cfg.thresholds.layout == "auto" {
        render_auto(&ctx, cfg, theme, style)
    } else {
        render_fixed(&ctx, cfg, theme, style)
    }
}

/// Fixed layout: a single line, segments joined by separators. The original
/// behavior — used whenever `layout` is not `"auto"`.
fn render_fixed(ctx: &RenderCtx, cfg: &Config, theme: &Theme, style: &Style) -> String {
    let mut line = String::with_capacity(256);
    let mut first = true;
    for kind in &cfg.segments {
        let mut w = SegmentWriter::new(theme, style);
        let emitted = kind.as_segment().render(ctx, &mut w);
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

/// Responsive layout: wrap segments across up to `max_lines` lines so each
/// line stays within the terminal width minus `wrap_margin`. Greedy — each
/// line takes as many segments as fit; once `max_lines` is reached remaining
/// segments are packed onto the last line. A width of 0 (unknown terminal)
/// disables wrapping and falls back to a single line.
fn render_auto(ctx: &RenderCtx, cfg: &Config, theme: &Theme, style: &Style) -> String {
    // Collect each rendered segment as its own owned string; segments that
    // emit nothing are dropped so they never cost a line or a separator.
    let segments: Vec<String> = cfg
        .segments
        .iter()
        .filter_map(|kind| {
            let mut w = SegmentWriter::new(theme, style);
            let emitted = kind.as_segment().render(ctx, &mut w);
            (emitted && !w.is_empty()).then(|| w.as_str().to_owned())
        })
        .collect();
    if segments.is_empty() {
        return String::new();
    }

    let term_width = terminal_width().saturating_sub(usize::from(cfg.thresholds.wrap_margin));
    if term_width == 0 {
        // Unknown terminal: no wrapping, behave like the fixed path.
        let mut line = String::with_capacity(256);
        let mut first = true;
        for s in &segments {
            if !first {
                separator(&mut line, theme, style);
            }
            line.push_str(s);
            first = false;
        }
        return line;
    }

    let sep_w = separator_width(style);
    let max_lines = usize::from(cfg.thresholds.max_lines.max(1));

    let mut out = String::with_capacity(256);
    let mut line_w = 0usize;
    let mut line_idx = 1usize;
    let mut first_on_line = true;

    for s in &segments {
        let seg_w = width::visible_width(s);
        // Width this segment adds if it is not the first on the line: its own
        // width plus the separator that precedes it.
        let add = if first_on_line { seg_w } else { seg_w + sep_w };

        let overflows = !first_on_line && line_w + add > term_width;
        if overflows && line_idx < max_lines {
            // Start a new line.
            out.push('\n');
            line_idx += 1;
            line_w = 0;
            first_on_line = true;
        }

        if !first_on_line {
            separator(&mut out, theme, style);
            line_w += sep_w;
        }
        out.push_str(s);
        line_w += seg_w;
        first_on_line = false;
    }
    out
}

/// Terminal column count: `$COLUMNS` first, then `stty size`, else 0 (unknown).
fn terminal_width() -> usize {
    if let Some(n) = std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
    {
        return n;
    }
    // `stty size` prints "rows cols"; the second field is the width.
    if let Ok(out) = std::process::Command::new("stty")
        .arg("size")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        && let Some(cols) = String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .nth(1)
            .and_then(|f| f.parse::<usize>().ok())
    {
        return cols;
    }
    0
}

/// Visible width of a single separator between adjacent segments on the same
/// line. Mirrors [`separator`]: two spaces plus the glyph (or a single space
/// for the lean style's empty glyph).
fn separator_width(style: &Style) -> usize {
    if style.separator.is_empty() {
        1
    } else {
        2 + width::visible_width(style.separator)
    }
}

/// Append the separator between two adjacent non-empty segments directly into
/// `line`: a space, the style's separator glyph painted in the theme's separator
/// color, then a space.
fn separator(line: &mut String, theme: &Theme, style: &Style) {
    // Lean style uses an empty separator — just a single space, no color codes.
    if style.separator.is_empty() {
        line.push(' ');
        return;
    }
    line.push(' ');
    theme.separator.write_fg(line);
    line.push_str(style.separator);
    line.push_str(RESET);
    line.push(' ');
}
