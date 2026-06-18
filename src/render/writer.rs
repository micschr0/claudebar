//! [`SegmentWriter`] — the single place segments emit colored, glyph-decorated
//! text. Centralizing emission here means a segment never embeds a raw ANSI
//! code, never hardcodes a theme color, and never decides whether icons render;
//! it just calls these methods and the active theme × style does the rest.

use crate::model::{Color, Style, Theme, RESET};
use crate::render::bar::make_bar;

pub struct SegmentWriter<'a> {
    buf: String,
    theme: &'a Theme,
    style: &'a Style,
}

impl<'a> SegmentWriter<'a> {
    pub fn new(theme: &'a Theme, style: &'a Style) -> Self {
        Self {
            buf: String::with_capacity(64),
            theme,
            style,
        }
    }

    pub fn theme(&self) -> &Theme {
        self.theme
    }

    pub fn style(&self) -> &Style {
        self.style
    }

    /// True if nothing has been written yet.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Append a run of text in `color`, terminated by a reset. The text is
    /// emitted verbatim — callers must pre-sanitize host-provided strings with
    /// [`crate::sanitize::strip_control`].
    pub fn colored(&mut self, color: Color, text: &str) {
        self.buf.push_str(&color.fg());
        self.buf.push_str(text);
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
            self.buf.push_str(&self.theme.dim.fg());
            self.buf.push_str(glyph);
            self.buf.push_str(RESET);
            self.buf.push(' ');
        }
    }

    /// Append raw, already-formed text (e.g. a single separating space).
    pub fn raw(&mut self, text: &str) {
        self.buf.push_str(text);
    }

    /// Append a progress bar, using the style's bar characters, the theme's
    /// track color, and the given fill color.
    pub fn bar(&mut self, pct: u32, width: u8, fill: Color) {
        self.buf.push_str(&make_bar(
            pct,
            width,
            fill,
            self.theme.bar_track,
            self.style.bar_fill,
            self.style.bar_empty,
        ));
    }

    /// The accumulated segment body.
    pub fn as_str(&self) -> &str {
        &self.buf
    }
}
