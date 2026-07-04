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
