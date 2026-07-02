//! Dev-context segment: worktree name, PR number + review state, sub-agent name.
//!
//! Contract (mirrors the bash branch `dev-context` section):
//! - Reads `worktree.name` (falling back to `workspace.git_worktree`), `pr.number`,
//!   `pr.review_state`, and `agent.name` from the hook JSON.
//! - Returns `false` (skipped) when all three are absent.
//! - Strips control bytes from all host-provided strings.
//! - Emits sub-elements separated by a single space:
//!   1. Worktree: icon (worktree glyph, dim) + name in dim.
//!   2. PR: icon (pull_request glyph, dim) + `#<n>` in `git_branch` color +
//!      optional review-state indicator (✓ ok, ✗ crit, ◦/· dim).
//!   3. Agent: icon (agent glyph, dim) + name in `effort_max` color.

use crate::render::SegmentWriter;
use crate::sanitize::strip_control;
use crate::segment::{RenderCtx, Segment};

pub struct DevContext;

impl Segment for DevContext {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let theme = ctx.theme;
        let glyphs = ctx.style.glyphs;

        let wt_name: Option<String> = ctx
            .input
            .worktree_name()
            .map(strip_control)
            .filter(|s| !s.is_empty());

        let pr_num: Option<u64> = ctx.input.pr.number.get();

        let agent_name: Option<String> = ctx
            .input
            .agent
            .name
            .as_deref()
            .map(strip_control)
            .filter(|s| !s.is_empty());

        if wt_name.is_none() && pr_num.is_none() && agent_name.is_none() {
            return false;
        }

        let mut need_space = false;

        if let Some(ref wt) = wt_name {
            out.icon(glyphs.worktree);
            out.dim(wt);
            need_space = true;
        }

        if let Some(num) = pr_num {
            if need_space {
                out.raw(" ");
            }
            out.icon(glyphs.pull_request);
            out.colored(theme.git_branch, &format!("#{num}"));
            let pr_state = ctx.input.pr.review_state.as_deref().map(strip_control);
            match pr_state.as_deref() {
                Some("approved") => out.colored(theme.bar_ok, " \u{2713}"),
                Some("changes_requested") => out.colored(theme.bar_crit, " \u{2717}"),
                Some("commented") => out.colored(theme.dim, " \u{25e6}"),
                Some("pending") => out.colored(theme.dim, " \u{b7}"),
                _ => {}
            }
            need_space = true;
        }

        if let Some(ref name) = agent_name {
            if need_space {
                out.raw(" ");
            }
            out.icon(glyphs.agent);
            out.colored(theme.effort, name);
        }

        true
    }
}
