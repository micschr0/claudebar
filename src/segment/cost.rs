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
            w.raw_fmt(format_args!("{:.*}", decimals, cost));
        });
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Config, InputData, SegmentKind};
    use crate::render::render_with;
    use crate::{styles, themes};

    fn render_cost(cost: f64) -> String {
        let input = InputData {
            cost: crate::model::input::CostInfo {
                total_cost_usd: crate::model::input::Coerce(Some(cost)),
                ..Default::default()
            },
            ..Default::default()
        };
        let cfg = Config {
            segments: vec![SegmentKind::Cost],
            ..Default::default()
        };
        let theme = themes::get(&cfg.theme);
        let style = styles::get(&cfg.style);
        render_with(&input, &cfg, &theme, &style, 0, None, 0)
    }

    #[test]
    fn cost_renders_usd_with_four_decimals() {
        let out = render_cost(9.0);
        assert!(out.contains('$'), "dollar sign missing: {out:?}");
        assert!(out.contains("9.00"), "amount '9.00' missing: {out:?}");

        let out2 = render_cost(1_234_567.89);
        assert!(out2.contains('$'), "dollar sign missing: {out2:?}");
        assert!(
            out2.contains("1234567.89"),
            "amount '1234567.89' missing: {out2:?}"
        );
    }
}
