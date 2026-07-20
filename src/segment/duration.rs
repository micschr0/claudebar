//! Duration segment — session wall-clock duration.
//!
//! Renders `⧖ 47m` or `⧖ 1h02m` or `⧖ 42s`. Hides when zero or absent.

use crate::render::SegmentWriter;
use crate::segment::{RenderCtx, Segment};

pub struct Duration;

fn fmt_duration(ms: u64) -> String {
    let total_s = ms / 1000;
    let h = total_s / 3600;
    let m = (total_s % 3600) / 60;
    let s = total_s % 60;
    let mut buf = String::with_capacity(7); // "1h02m" ≤ 7 bytes
    use std::fmt::Write as _;
    if h > 0 {
        write!(buf, "{h}h{m:02}m").unwrap();
    } else if m > 0 {
        write!(buf, "{m}m").unwrap();
    } else {
        write!(buf, "{s}s").unwrap();
    }
    buf
}

impl Segment for Duration {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let ms = match ctx.input.cost.total_duration_ms.0 {
            Some(d) if d > 0 => d,
            _ => return false,
        };

        let formatted = fmt_duration(ms);

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
    use super::*;
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

    #[test]
    fn fmt_seconds() {
        assert_eq!(fmt_duration(42_000), "42s");
    }

    #[test]
    fn fmt_minutes() {
        assert_eq!(fmt_duration(2_820_000), "47m");
    }

    #[test]
    fn fmt_hours() {
        assert_eq!(fmt_duration(3_720_000), "1h02m");
    }

    /// Sub-second durations render as "0s" (the minimum granularity).
    #[test]
    fn duration_renders_below_one_ms() {
        let out = render_dur(500);
        assert!(out.contains("0s"), "expected '0s' for sub-second: {out:?}");
    }
}
