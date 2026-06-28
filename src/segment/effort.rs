//! Effort segment — reasoning effort level (low/medium/high/xhigh/max).
//!
//! Renders `ψ high` (or `ψ max` etc.). Hides when the effort level is absent
//! (models without the param). The level string is colored by intensity:
//!   low|medium → dim · high → bar_ok · xhigh → bar_warn · max → effort.
//!
//! This is the standalone counterpart to the effort rendering embedded in the
//! model segment — use it when you want effort as a separate, reorderable field.

use crate::render::SegmentWriter;
use crate::sanitize::strip_control;
use crate::segment::{RenderCtx, Segment};

pub struct Effort;

impl Segment for Effort {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let level = match &ctx.input.effort.level {
            Some(l) if !l.is_empty() => strip_control(l),
            _ => return false,
        };

        // Compact label: "medium" → "med" to save space.
        let label = match level.as_str() {
            "medium" => "med",
            other => other,
        };

        let color = match level.as_str() {
            "low" | "medium" => ctx.theme.dim,
            "high" => ctx.theme.bar_ok,
            "xhigh" => ctx.theme.bar_warn,
            "max" => ctx.theme.effort,
            _ => ctx.theme.dim,
        };

        out.colored_with(color, |w| {
            w.icon(ctx.style.glyphs.effort);
            w.raw(" ");
            w.raw(label);
            w.raw(" ");
        });
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::input::{Effort as EffortData, InputData};
    use crate::model::{Config, Thresholds};
    use crate::{styles, themes};

    fn render_effort(level: Option<&str>) -> String {
        let input = InputData {
            effort: EffortData {
                level: level.map(String::from),
            },
            ..Default::default()
        };
        let config = Config::default();
        let theme = themes::get(&config.theme);
        let style = styles::get(&config.style);
        let th = Thresholds::default();
        let ctx = RenderCtx {
            input: &input,
            theme: &theme,
            style: &style,
            th: &th,
            now: 0,
            home: None,
            tz_offset_seconds: 0,
        };
        let mut writer = SegmentWriter::new(&theme, &style);
        let _ = Effort.render(&ctx, &mut writer);
        writer.as_str().to_string()
    }

    #[test]
    fn absent_renders_nothing() {
        assert_eq!(render_effort(None), "");
    }

    #[test]
    fn empty_renders_nothing() {
        assert_eq!(render_effort(Some("")), "");
    }

    #[test]
    fn high_renders_with_bar_ok_color() {
        let out = render_effort(Some("high"));
        assert!(out.contains("high"));
    }

    #[test]
    fn medium_compacted_to_med() {
        let out = render_effort(Some("medium"));
        assert!(out.contains("med"));
        assert!(!out.contains("medium"));
    }

    #[test]
    fn max_renders_with_effort_color() {
        let out = render_effort(Some("max"));
        assert!(out.contains("max"));
    }
    #[test]
    fn strips_injection_bytes() {
        // ESC injected through the level string must be stripped; the only ESC
        // bytes left are our own SGR codes (icon dim+reset, color+reset) —
        // exactly 4. If the injected ESC leaked through it would be 5.
        let out = render_effort(Some("hi\x1bgh"));
        assert!(out.contains("high"), "level not stripped cleanly: {out:?}");
        assert_eq!(
            out.matches('\u{1b}').count(),
            4,
            "unexpected ESC count: {out:?}"
        );
    }
}
