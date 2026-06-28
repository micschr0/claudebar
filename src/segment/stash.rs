//! Stash segment — git stash count.
//!
//! Spawns `git rev-list --walk-reflogs --count refs/stash` in `cwd`.
//! Hides outside a git repo or when stash count is zero.

use crate::render::SegmentWriter;
use crate::segment::{RenderCtx, Segment};
use std::process::Command;

pub struct Stash;

impl Segment for Stash {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let cwd = match ctx.input.cwd.as_deref() {
            Some(c) if c.starts_with('/') => c,
            _ => return false,
        };

        let output = match Command::new("git")
            .args([
                "-C",
                cwd,
                "rev-list",
                "--walk-reflogs",
                "--count",
                "refs/stash",
            ])
            .env("GIT_OPTIONAL_LOCKS", "0")
            .output()
        {
            Ok(o) if o.status.success() => o.stdout,
            _ => return false,
        };

        let count_str = String::from_utf8_lossy(&output);
        let count: u64 = match count_str.trim().parse() {
            Ok(n) => n,
            Err(_) => return false,
        };

        if count == 0 {
            return false;
        }

        out.colored_with(ctx.theme.stash, |w| {
            w.icon(ctx.style.glyphs.stash);
            w.raw(" ");
            w.raw(&count.to_string());
            w.raw(" ");
        });
        true
    }
}
