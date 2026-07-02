//! Cost segment — session cost in USD.
//!
//! Renders `$1.23` (with configurable decimal places). Hides when the cost is
//! zero or absent.

use crate::render::SegmentWriter;
use crate::segment::{RenderCtx, Segment};

pub struct Cost;

impl Segment for Cost {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let cost = match ctx.input.cost.total_cost_usd.0 {
            Some(c) if c > 0.0 => c,
            _ => return false,
        };
        let decimals = usize::from(ctx.th.cost_decimals.min(4));

        out.colored_with(ctx.theme.cost, |w| {
            w.icon(ctx.style.glyphs.cost);
            w.raw(" ");
            w.raw_fmt(format_args!("{:.*}", decimals, cost));
            w.raw(" ");
        });
        true
    }
}
