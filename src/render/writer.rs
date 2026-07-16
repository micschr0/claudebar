//! [`SegmentWriter`] — the single place segments emit colored, glyph-decorated
//! text. Centralizing emission here means a segment never embeds a raw ANSI
//! code, never hardcodes a theme color, and never decides whether icons render;
//! it just calls these methods and the active theme × style does the rest.

use crate::model::{Color, RESET, Style, Theme};
use crate::render::bar::write_bar;
use std::fmt::Write;

pub struct SegmentWriter<'a> {
    buf: String,
    theme: &'a Theme,
    style: &'a Style,
    /// Color of the innermost open `colored_with` span, if any. Nested resets
    /// (from `icon()` or a further-nested `colored_with`) restore this color
    /// instead of leaving trailing text at the terminal default.
    active: Option<Color>,
}

impl<'a> SegmentWriter<'a> {
    #[must_use]
    pub fn new(theme: &'a Theme, style: &'a Style) -> Self {
        Self {
            buf: String::with_capacity(64),
            theme,
            style,
            active: None,
        }
    }

    #[must_use]
    pub fn theme(&self) -> &Theme {
        self.theme
    }

    #[must_use]
    pub fn style(&self) -> &Style {
        self.style
    }

    /// True if nothing has been written yet.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Append a run of text in `color`, terminated by a reset. The text is
    /// emitted verbatim — callers must pre-sanitize host-provided strings with
    /// [`crate::sanitize::strip_control`].
    pub fn colored(&mut self, color: Color, text: &str) {
        color.write_fg(&mut self.buf);
        self.buf.push_str(text);
        self.buf.push_str(RESET);
    }

    /// Like [`SegmentWriter::colored`] but lets the caller write the body via a
    /// closure that has mutable access to this writer — avoids allocating a
    /// throwaway `String` for composed spans (icon + text). Emits the same byte
    /// order: fg color → closure body → reset.
    pub fn colored_with<F>(&mut self, color: Color, f: F)
    where
        F: FnOnce(&mut Self),
    {
        color.write_fg(&mut self.buf);
        let prev = self.active.replace(color);
        f(self);
        self.active = prev;
        self.buf.push_str(RESET);
        if let Some(c) = prev {
            c.write_fg(&mut self.buf);
        }
    }

    /// Like [`SegmentWriter::colored`] but takes pre-formatted [`std::fmt::Arguments`]
    /// so callers can pass `format_args!(...)` and write directly into the buffer
    /// instead of allocating a throwaway `String` per emission. Emits the same
    /// byte order as [`SegmentWriter::colored`]: fg color → args → reset.
    ///
    /// # Panics
    ///
    /// The internal `write_fmt` on a `String` buffer is infallible and will never panic.
    pub fn colored_fmt(&mut self, color: Color, args: std::fmt::Arguments) {
        color.write_fg(&mut self.buf);
        self.buf.write_fmt(args).unwrap();
        self.buf.push_str(RESET);
    }

    /// Append a run in the theme's dim color (secondary symbols).
    pub fn dim(&mut self, text: &str) {
        self.colored(self.theme.dim, text);
    }

    /// Append a leading icon in the dim color followed by a single space, but
    /// only when the active style enables icons. With icons off this is a no-op,
    /// so minimal/ASCII styles drop glyphs without any per-segment branching.
    pub fn icon(&mut self, glyph: &str) {
        if self.style.icons && !glyph.is_empty() {
            self.theme.dim.write_fg(&mut self.buf);
            self.buf.push_str(glyph);
            self.buf.push_str(RESET);
            if let Some(c) = self.active {
                c.write_fg(&mut self.buf);
            }
            self.buf.push(' ');
        }
    }

    /// Append raw, already-formed text (e.g. a single separating space).
    pub fn raw(&mut self, text: &str) {
        self.buf.push_str(text);
    }

    /// Like [`SegmentWriter::raw`] but formats directly into the buffer
    /// via [`std::fmt::Arguments`] — avoids allocating a throwaway
    /// `String` for numeric or formatted values.
    ///
    /// # Panics
    ///
    /// The internal `write!` to a `String` buffer is infallible and will never panic.
    pub fn raw_fmt(&mut self, args: std::fmt::Arguments) {
        write!(self.buf, "{}", args).unwrap();
    }

    /// Append a progress bar, using the style's bar characters, the theme's
    /// track color, and the given fill color.
    pub fn bar(&mut self, pct: u32, width: u8, fill: Color) {
        write_bar(
            &mut self.buf,
            pct,
            width,
            fill,
            self.theme.bar_track,
            self.style.bar_fill,
            self.style.bar_empty,
        );
    }

    /// Append a progress bar followed by its percentage — the bar-to-percent
    /// gap convention (`bar()` + `" "` + `"<pct>%"` in `color`) owned in one
    /// place so it can't drift between call sites, mirroring how `icon()`
    /// owns its own trailing-space convention.
    pub fn bar_pct(&mut self, pct: u32, width: u8, color: Color) {
        self.bar(pct, width, color);
        self.raw(" ");
        self.colored_fmt(color, format_args!("{pct}%"));
    }

    /// Join two related "windows" inside one segment (e.g. rate-limits'
    /// 5h/weekly gauges) with a dim-colored glyph — the intra-segment
    /// counterpart to the composer's inter-segment separator
    /// (see `render::mod`), deliberately using `theme.dim` (lighter) instead
    /// of `theme.separator` so the pair reads as one grouped unit rather than
    /// a segment boundary.
    pub fn window_gap(&mut self) {
        self.raw(" ");
        self.colored(self.theme.dim, self.style.window_gap);
        self.raw(" ");
    }

    /// The accumulated segment body.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.buf
    }
}

#[cfg(test)]
mod tests {
    use crate::{styles, themes};

    #[test]
    fn colored_with_restores_active_stack_on_pop() {
        let theme = themes::get("tokyo-night");
        let style = styles::get("powerline");
        let mut w = super::SegmentWriter::new(&theme, &style);

        // Single-level: active was None → after RESET, no color re-emission.
        let dir_fg = theme.dir.fg();
        w.colored_with(theme.dir, |w| w.raw("single"));
        let expected_single = format!("{dir_fg}single\x1b[0m");
        assert_eq!(
            w.as_str(),
            expected_single,
            "single colored_with: active restored from None"
        );

        // Now test nested: inner restores outer's color before its own RESET,
        // then outer ends with RESET. Buffer ends with "dir_color\x1b[0m".
        let model_fg = theme.model.fg();
        let mut w2 = super::SegmentWriter::new(&theme, &style);
        w2.colored_with(theme.dir, |w| {
            w.colored_with(theme.model, |w| w.raw("nested"));
        });
        let expected_nested = format!("{dir_fg}{model_fg}nested\x1b[0m{dir_fg}\x1b[0m");
        assert_eq!(
            w2.as_str(),
            expected_nested,
            "nested colored_with: inner restores outer, outer emits RESET"
        );
    }
}
