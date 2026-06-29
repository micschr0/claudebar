//! The [`Segment`] seam. Each segment is given a [`RenderCtx`] and writes its
//! spans into a [`SegmentWriter`]; it returns `true` if it emitted anything, so
//! the composer knows whether to place a separator before the next segment.
//!
//! Segments never know their neighbors, never emit a separator, and never embed
//! a raw color — they read `ctx.theme` / `ctx.style` through the writer.

pub mod burn;
pub mod clock;
pub mod context;
pub mod cost;
pub mod dev_context;
pub mod directory;
pub mod duration;
pub mod git;
pub mod limit_sync;
pub mod lines;
pub mod model;
pub mod rate_limits;
pub mod stash;

use crate::model::{InputData, SegmentKind, Style, Theme, Thresholds};
use crate::render::SegmentWriter;

/// Everything a segment needs to render. `now` and `home` are injected (never
/// read from the ambient environment inside a segment) so rendering is
/// deterministic and testable.
pub struct RenderCtx<'a> {
    pub input: &'a InputData,
    pub theme: &'a Theme,
    pub style: &'a Style,
    pub th: &'a Thresholds,
    /// Current time in epoch seconds (for reset countdowns).
    pub now: i64,
    /// `$HOME`, for path abbreviation.
    pub home: Option<&'a str>,
    /// Local timezone offset in seconds east of UTC.
    /// 0 = UTC (fallback when detection fails, or TUI preview).
    pub tz_offset_seconds: i32,
}

/// A renderable status segment.
pub trait Segment {
    /// Write this segment's body into `out`. Return `true` iff anything was
    /// emitted (an empty/absent segment returns `false` and is skipped, taking
    /// its separator with it).
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool;
}

impl SegmentKind {
    /// Resolve a [`SegmentKind`] to its (zero-sized) [`Segment`] implementation.
    pub fn as_segment(self) -> &'static dyn Segment {
        match self {
            SegmentKind::Clock => &clock::Clock,
            SegmentKind::Project => &NOOP,
            SegmentKind::Directory => &directory::Directory,
            SegmentKind::Git => &git::Git,
            SegmentKind::Stash => &stash::Stash,
            SegmentKind::Context => &context::Context,
            SegmentKind::RateLimits => &rate_limits::RateLimits,
            SegmentKind::DevContext => &dev_context::DevContext,
            SegmentKind::Model => &model::Model,
            SegmentKind::Effort => &NOOP,
            SegmentKind::Cost => &cost::Cost,
            SegmentKind::Lines => &lines::Lines,
            SegmentKind::Duration => &duration::Duration,
            SegmentKind::Burn => &burn::Burn,
        }
    }
}

/// No-op segment for deprecated variants (Project, Effort).
struct Noop;
impl Segment for Noop {
    fn render(&self, _ctx: &RenderCtx, _out: &mut SegmentWriter) -> bool { false }
}
const NOOP: Noop = Noop;
