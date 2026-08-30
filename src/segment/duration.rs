//! Duration segment — session wall-clock duration.
//!
//! Renders `⧖ 47m` or `⧖ 1h02m` or `⧖ 42s`. Hides when zero or absent.

use crate::render::SegmentWriter;
use crate::segment::{RenderCtx, Segment};

pub struct Duration;

impl Segment for Duration {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let ms = match ctx.input.cost.total_duration_ms.0 {
            Some(d) if d > 0 => d,
            _ => return false,
        };

        // Session wall-clock: hours are never folded into days.
        let formatted =
            crate::sanitize::fmt_span(i64::try_from(ms / 1000).unwrap_or(i64::MAX), false);

        out.colored_with(ctx.theme.duration, |w| {
            w.icon(ctx.style.glyphs.duration);
            w.raw(" ");
            w.raw(&formatted);
            w.raw(" ");
        });
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Config, InputData, SegmentKind};
    use crate::render::render_with;
    use crate::{styles, themes};

    fn render_dur(ms: u64) -> String {
        let input = InputData {
            cost: crate::model::input::CostInfo {
                total_duration_ms: crate::model::input::Coerce(Some(ms)),
                ..Default::default()
            },
            ..Default::default()
        };
        let cfg = Config {
            segments: vec![SegmentKind::Duration],
            ..Default::default()
        };
        let theme = themes::get(&cfg.theme);
        let style = styles::get(&cfg.style);
        render_with(&input, &cfg, &theme, &style, 0, None, 0)
    }

    /// The segment's own formatting contract, end to end. The formatter itself
    /// is property-tested against its pre-merge reference in `sanitize`.
    #[test]
    fn renders_seconds_minutes_and_padded_hours() {
        assert!(render_dur(42_000).contains("42s"));
        assert!(render_dur(2_820_000).contains("47m"));
        assert!(render_dur(3_720_000).contains("1h02m"));
    }

    /// A session past 24h keeps counting hours instead of folding into days —
    /// the reason `fmt_span` takes a flag rather than always showing days.
    #[test]
    fn long_session_does_not_fold_hours_into_days() {
        let out = render_dur(90_000_000); // 25h
        assert!(out.contains("25h00m"), "expected 25h00m, got: {out:?}");
    }

    /// Sub-second durations render as "0s" (the minimum granularity).
    #[test]
    fn duration_renders_below_one_ms() {
        let out = render_dur(500);
        assert!(out.contains("0s"), "expected '0s' for sub-second: {out:?}");
    }
}
