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

#[cfg(test)]
mod tests {
    use crate::model::input::{Coerce, Pr};
    use crate::model::{Config, InputData, SegmentKind};
    use crate::render::render_with;
    use crate::{styles, themes};

    fn render_dev_ctx(review_state: Option<&str>) -> String {
        let input = InputData {
            pr: Pr {
                number: Coerce(Some(42)),
                review_state: review_state.map(String::from),
            },
            ..Default::default()
        };
        let cfg = Config {
            segments: vec![SegmentKind::DevContext],
            ..Default::default()
        };
        let theme = themes::get(&cfg.theme);
        let style = styles::get(&cfg.style);
        render_with(&input, &cfg, &theme, &style, 0, None, 0)
    }

    /// PR #42 with review_state "changes_requested" → renders ✗ indicator.
    #[test]
    fn dev_context_requested_changes() {
        let out = render_dev_ctx(Some("changes_requested"));
        assert!(out.contains("#42"), "#42 missing: {out:?}");
        assert!(out.contains('\u{2717}'), "✗ missing: {out:?}");
    }

    /// PR #42 with review_state "commented" → renders ◦ indicator.
    #[test]
    fn dev_context_commented() {
        let out = render_dev_ctx(Some("commented"));
        assert!(out.contains("#42"), "#42 missing: {out:?}");
        assert!(out.contains('\u{25e6}'), "◦ missing: {out:?}");
    }

    /// PR #42 with review_state "pending" (draft) → renders · indicator.
    #[test]
    fn dev_context_draft() {
        let out = render_dev_ctx(Some("pending"));
        assert!(out.contains("#42"), "#42 missing: {out:?}");
        assert!(out.contains('\u{b7}'), "· missing: {out:?}");
    }

    /// PR #42 with no review_state → no review indicator emitted.
    #[test]
    fn dev_context_nil_review() {
        let out = render_dev_ctx(None);
        assert!(out.contains("#42"), "#42 missing: {out:?}");
        // No ✓, ✗, ◦, or · — just the PR number with no state indicator.
        assert!(
            !out.contains('\u{2713}')
                && !out.contains('\u{2717}')
                && !out.contains('\u{25e6}')
                && !out.contains('\u{b7}'),
            "unexpected review indicator in nil review: {out:?}"
        );
    }
    /// PR #42 with review_state "approved" → renders ✓ indicator.
    #[test]
    fn dev_context_approved() {
        let out = render_dev_ctx(Some("approved"));
        assert!(out.contains("#42"), "#42 missing: {out:?}");
        assert!(out.contains('\u{2713}'), "✓ missing: {out:?}");
    }

    /// PR #42 with review_state "unknown" → no review indicator emitted.
    #[test]
    fn dev_context_unknown_review() {
        let out = render_dev_ctx(Some("unknown"));
        assert!(out.contains("#42"), "#42 missing: {out:?}");
        // No ✓, ✗, ◦, or · for unknown states.
        assert!(
            !out.contains('\u{2713}')
                && !out.contains('\u{2717}')
                && !out.contains('\u{25e6}')
                && !out.contains('\u{b7}'),
            "unexpected review indicator in unknown review: {out:?}"
        );
    }

    /// No worktree, PR, or agent data → skipped (returns false).
    #[test]
    fn dev_context_skipped_when_all_absent() {
        let input = InputData {
            worktree: None,
            ..Default::default()
        };
        let cfg = Config {
            segments: vec![SegmentKind::DevContext],
            ..Default::default()
        };
        let theme = themes::get(&cfg.theme);
        let style = styles::get(&cfg.style);
        let out = render_with(&input, &cfg, &theme, &style, 0, None, 0);
        assert_eq!(out, "");
    }
}
