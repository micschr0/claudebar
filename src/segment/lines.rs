//! Lines segment — added/removed this session.
//!
//! Renders `+321 −87`. Hides when both counts are zero or absent.
//! Uses a single color slot (`lines`) — the `+`/`−` signs provide visual
//! distinction without borrowing `modified` (semantically for modified files)
//! or requiring a new theme slot.

use crate::render::SegmentWriter;
use crate::segment::{RenderCtx, Segment};

pub struct Lines;

impl Segment for Lines {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let added = ctx.input.cost.total_lines_added.0.unwrap_or(0);
        let removed = ctx.input.cost.total_lines_removed.0.unwrap_or(0);

        if added == 0 && removed == 0 {
            return false;
        }

        out.colored_with(ctx.theme.lines, |w| {
            w.raw(" +");
            w.raw_fmt(format_args!("{}", added));
            w.raw(" \u{2212}");
            w.raw_fmt(format_args!("{}", removed));
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

    fn render_lines(added: u64, removed: u64) -> String {
        let input = InputData {
            cost: crate::model::input::CostInfo {
                total_lines_added: crate::model::input::Coerce(Some(added)),
                total_lines_removed: crate::model::input::Coerce(Some(removed)),
                ..Default::default()
            },
            ..Default::default()
        };
        let cfg = Config {
            segments: vec![SegmentKind::Lines],
            ..Default::default()
        };
        let theme = themes::get(&cfg.theme);
        let style = styles::get(&cfg.style);
        render_with(&input, &cfg, &theme, &style, 0, None, 0)
    }

    #[test]
    fn lines_added_only() {
        let out = render_lines(10, 0);
        assert!(out.contains("+10"), "+10 missing: {out:?}");
        assert!(out.contains("\u{2212}0"), "−0 missing: {out:?}");
    }

    #[test]
    fn lines_removed_only() {
        let out = render_lines(0, 5);
        assert!(out.contains("+0"), "+0 missing: {out:?}");
        assert!(out.contains("\u{2212}5"), "−5 missing: {out:?}");
    }

    #[test]
    fn lines_both_zero() {
        let out = render_lines(0, 0);
        assert!(
            out.is_empty(),
            "expected empty output for zero lines: {out:?}"
        );
    }

    #[test]
    fn lines_just_added() {
        let out = render_lines(1, 0);
        assert!(out.contains("+1"), "+1 missing: {out:?}");
        assert!(out.contains("\u{2212}0"), "−0 missing: {out:?}");
    }
}
