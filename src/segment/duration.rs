//! Duration segment — session wall-clock duration.
//!
//! Renders `⧖ 47m` or `⧖ 1h02m` or `⧖ 42s`. Hides when zero or absent.

use crate::render::SegmentWriter;
use crate::segment::{RenderCtx, Segment};

pub struct Duration;

fn fmt_duration(ms: u64) -> String {
    let s = ms / 1000;
    let h = s / 3600;
    let m = (s % 3600) / 60;
    let secs = s % 60;
    if h > 0 {
        format!("{}h{:02}m", h, m)
    } else if m > 0 {
        format!("{}m", m)
    } else {
        format!("{}s", secs)
    }
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
}
