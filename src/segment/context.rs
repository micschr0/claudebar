//! Context (tokens + usage bar) segment.
//!
//! Contract (matches the bash script's "session tokens + context bar"):
//! ```text
//! total = context_window.total_input_tokens + total_output_tokens
//!   (each via Coerce::or_default). If total == 0 -> emit nothing, return false.
//! Format the count with crate::sanitize::fmt_tokens.
//! If used_percentage present and in range 0..=999 (round to nearest int first):
//!   pick the bar color by threshold:
//!     pct > 100 -> bar_crit; pct >= th.crit -> bar_crit;
//!     pct >= th.warn -> bar_warn; else bar_ok.
//!   Emit: context icon (style.glyphs.context, dim) + bar (width th.bar_width,
//!   fill = chosen color) + " " + "<pct>%" in the chosen color + " " +
//!   token icon (style.glyphs.token) + count in theme.token.
//!   Bash: ${C_DIM}<ctx> %bar% ${cc}%d%%${R} ${C_TOK}<tok> %s${R}
//! If used_percentage absent/out-of-range: emit only the token part
//!   (token icon + count in theme.token).
//! Return true.
//! ```

use crate::render::SegmentWriter;
use crate::sanitize::fmt_tokens;
use crate::segment::{RenderCtx, Segment};

pub struct Context;

impl Segment for Context {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let cw = &ctx.input.context_window;
        let total = cw.total_input_tokens.or_default() + cw.total_output_tokens.or_default();
        if total == 0 {
            return false;
        }
        let count = fmt_tokens(total);

        // The bar + percent only render when a usable percentage is present.
        if let Some(raw) = cw.used_percentage.get() {
            let pct = raw.round() as i64;
            if (0..=999).contains(&pct) {
                let pct = pct as u32;
                let color = if pct > 100 || pct >= ctx.th.crit as u32 {
                    ctx.theme.bar_crit
                } else if pct >= ctx.th.warn as u32 {
                    ctx.theme.bar_warn
                } else {
                    ctx.theme.bar_ok
                };
                out.icon(ctx.style.glyphs.context);
                out.bar(pct, ctx.th.bar_width, color);
                out.raw(" ");
                out.colored(color, &format!("{pct}%"));
                out.raw(" ");
            }
        }

        out.icon(ctx.style.glyphs.token);
        out.colored(ctx.theme.token, &count);
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::model::input::{Coerce, ContextWindow};
    use crate::model::{Config, InputData, SegmentKind};
    use crate::render::render_with;
    use crate::{styles, themes};

    fn render_ctx(
        input_tokens: Option<u64>,
        output_tokens: Option<u64>,
        pct: Option<f64>,
    ) -> String {
        let input = InputData {
            context_window: ContextWindow {
                total_input_tokens: Coerce(input_tokens),
                total_output_tokens: Coerce(output_tokens),
                used_percentage: Coerce(pct),
            },
            ..Default::default()
        };
        let cfg = Config {
            segments: vec![SegmentKind::Context],
            ..Default::default()
        };
        let theme = themes::get(&cfg.theme);
        let style = styles::get(&cfg.style);
        render_with(&input, &cfg, &theme, &style, 0, None)
    }

    #[test]
    fn zero_tokens_renders_nothing() {
        assert_eq!(render_ctx(None, None, Some(67.0)), "");
        assert_eq!(render_ctx(Some(0), Some(0), Some(67.0)), "");
    }

    #[test]
    fn renders_count_and_percent_in_token_color() {
        // 35000 + 7300 = 42300 -> 42.3k, 67% in warn band.
        let out = render_ctx(Some(35000), Some(7300), Some(67.0));
        assert!(out.contains("42.3k"), "count missing: {out:?}");
        assert!(out.contains("67%"), "percent missing: {out:?}");
        // token color 117 in tokyo-night.
        assert!(
            out.contains("\x1b[38;5;117m"),
            "token color missing: {out:?}"
        );
    }

    #[test]
    fn below_warn_uses_ok_color() {
        // bar_ok = 114 in tokyo-night.
        let out = render_ctx(Some(1000), Some(0), Some(10.0));
        assert!(out.contains("10%"), "percent missing: {out:?}");
        assert!(out.contains("\x1b[38;5;114m"), "ok color missing: {out:?}");
    }

    #[test]
    fn warn_threshold_uses_warn_color() {
        // bar_warn = 221 in tokyo-night; 50% >= warn(50), < crit(80).
        let out = render_ctx(Some(1000), Some(0), Some(50.0));
        assert!(
            out.contains("\x1b[38;5;221m"),
            "warn color missing: {out:?}"
        );
    }

    #[test]
    fn crit_threshold_uses_crit_color() {
        // bar_crit = 203 in tokyo-night; 95% >= crit(80).
        let out = render_ctx(Some(1000), Some(0), Some(95.0));
        assert!(
            out.contains("\x1b[38;5;203m"),
            "crit color missing: {out:?}"
        );
    }

    #[test]
    fn over_100_uses_crit_color() {
        let out = render_ctx(Some(1_600_000), Some(0), Some(142.0));
        assert!(out.contains("142%"), "percent missing: {out:?}");
        assert!(out.contains("1.6M"), "count missing: {out:?}");
        assert!(
            out.contains("\x1b[38;5;203m"),
            "crit color missing: {out:?}"
        );
    }

    #[test]
    fn missing_pct_renders_only_count() {
        let out = render_ctx(Some(42300), Some(0), None);
        assert!(out.contains("42.3k"), "count missing: {out:?}");
        assert!(!out.contains('%'), "should have no percent: {out:?}");
    }

    #[test]
    fn out_of_range_pct_renders_only_count() {
        let out = render_ctx(Some(42300), Some(0), Some(1000.0));
        assert!(out.contains("42.3k"), "count missing: {out:?}");
        assert!(!out.contains('%'), "should have no percent: {out:?}");
    }
}
