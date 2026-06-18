//! Rate-limits segment — 5-hour + weekly windows in one segment.
//!
//! Contract (matches the bash script's "rate limits" block). Two windows; this
//! one segment renders both, separated by a single space (the segment is a unit;
//! the composer separator only appears around the whole segment).
//! ```text
//! 5-hour window (rate_limits.five_hour): shown whenever the window is present
//!   (a percentage OR a future resets_at).
//!   - If used_percentage present and in 0..=999 (rounded): color by
//!       pct >= th.crit -> bar_crit; pct >= th.warn -> bar_warn; else bar_ok.
//!       Emit clock icon (style.glyphs.clock, dim) + bar + " " + "<pct>%" colored.
//!   - Then the reset countdown via crate::sanitize::fmt_reset(resets_at, now):
//!       if Some, emit " " + reset icon (style.glyphs.reset, dim) + " " +
//!       "<countdown>" in theme.reset.
//!   Bash: ${C_DIM}<clock> %bar% ${rlc}%d%%${R} ${C_DIM}<reset>${R} ${C_RST}%s${R}
//!
//! Weekly window (rate_limits.seven_day): only surfaced once
//!   used_percentage >= th.weekly_show_at (and <= 999). Color: pct >= th.crit
//!   -> bar_crit, else bar_warn. Emit weekly icon (style.glyphs.weekly, dim) +
//!   bar + " " + "<pct>%" colored, then its own reset countdown like the 5h
//!   window. If shown after the 5h window, separate them with a single space.
//!
//! Emit nothing / return false when neither window has anything to show.
//! ```

use crate::model::input::Window;
use crate::model::Color;
use crate::render::SegmentWriter;
use crate::sanitize::fmt_reset;
use crate::segment::{RenderCtx, Segment};

pub struct RateLimits;

/// Round a percentage and accept it only when it lands in `0..=999` — the upper
/// bound rejects a leaked epoch timestamp while still allowing over-limit values.
fn pct_in_range(p: f64) -> Option<u32> {
    let n = p.round();
    if (0.0..=999.0).contains(&n) {
        Some(n as u32)
    } else {
        None
    }
}

/// Append a reset countdown (` ` + reset icon + ` ` + value) when present.
fn write_reset(ctx: &RenderCtx, out: &mut SegmentWriter, resets_at: i64) {
    if let Some(rem) = fmt_reset(resets_at, ctx.now) {
        write_reset_value(ctx, out, &rem);
    }
}

/// Append an already-formatted reset countdown: ` ` + reset icon + ` ` + value.
fn write_reset_value(ctx: &RenderCtx, out: &mut SegmentWriter, rem: &str) {
    out.raw(" ");
    out.icon(ctx.style.glyphs.reset);
    out.colored(ctx.theme.reset, rem);
}

impl Segment for RateLimits {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let mut emitted = false;

        // 5-hour window.
        if let Some(w) = ctx.input.rate_limits.five_hour.as_ref() {
            emitted |= render_five_hour(ctx, out, w);
        }

        // Weekly (7-day) window.
        if let Some(w) = ctx.input.rate_limits.seven_day.as_ref() {
            if let Some(pct) = w.used_percentage.get().and_then(pct_in_range) {
                if pct >= u32::from(ctx.th.weekly_show_at) {
                    let color = if pct >= u32::from(ctx.th.crit) {
                        ctx.theme.bar_crit
                    } else {
                        ctx.theme.bar_warn
                    };
                    if emitted {
                        out.raw(" ");
                    }
                    write_window(ctx, out, ctx.style.glyphs.weekly, pct, color);
                    write_reset(ctx, out, w.resets_at.get().unwrap_or(0));
                    emitted = true;
                }
            }
        }

        emitted
    }
}

/// Render the 5-hour window: shown whenever a percentage OR a future reset is
/// present. Returns whether anything was emitted.
fn render_five_hour(ctx: &RenderCtx, out: &mut SegmentWriter, w: &Window) -> bool {
    let pct = w.used_percentage.get().and_then(pct_in_range);
    let reset = fmt_reset(w.resets_at.get().unwrap_or(0), ctx.now);

    // Nothing to show for this window unless it has a renderable pct or a reset.
    if pct.is_none() && reset.is_none() {
        return false;
    }

    if let Some(pct) = pct {
        let color = if pct >= u32::from(ctx.th.crit) {
            ctx.theme.bar_crit
        } else if pct >= u32::from(ctx.th.warn) {
            ctx.theme.bar_warn
        } else {
            ctx.theme.bar_ok
        };
        write_window(ctx, out, ctx.style.glyphs.clock, pct, color);
    }

    if let Some(rem) = reset {
        write_reset_value(ctx, out, &rem);
    }

    true
}

/// Emit one window body: icon + bar + " " + "<pct>%" in `color`. The `icon`
/// method already appends a trailing space, so the bar follows it directly.
fn write_window(ctx: &RenderCtx, out: &mut SegmentWriter, glyph: &str, pct: u32, color: Color) {
    out.icon(glyph);
    out.bar(pct, ctx.th.bar_width, color);
    out.raw(" ");
    out.colored(color, &format!("{pct}%"));
}

#[cfg(test)]
mod tests {
    use crate::model::{Config, InputData, SegmentKind};
    use crate::render::render_with;
    use crate::{styles, themes};

    /// `now` is fixed well before every fixture's `resets_at` (which use epochs
    /// like 1.9e9 → year 2030), so countdowns render deterministically.
    const NOW: i64 = 1_700_000_000;

    fn render_rl(json: &str) -> String {
        let input = InputData::parse(json);
        let cfg = Config {
            segments: vec![SegmentKind::RateLimits],
            ..Default::default()
        };
        let theme = themes::get(&cfg.theme);
        let style = styles::get(&cfg.style);
        render_with(&input, &cfg, &theme, &style, NOW, None)
    }

    #[test]
    fn five_hour_shows_with_pct_and_reset() {
        let out = render_rl(
            r#"{"rate_limits":{"five_hour":{"used_percentage":48.0,"resets_at":1900000000}}}"#,
        );
        assert!(out.contains("48%"), "5h pct missing: {out:?}");
        // bar_ok (<50) color present.
        assert!(!out.is_empty());
        // reset countdown rendered (theme.reset color).
        let reset = themes::get("tokyo-night").reset.fg();
        assert!(out.contains(&reset), "reset color missing: {out:?}");
    }

    #[test]
    fn weekly_hidden_below_show_at() {
        let out = render_rl(
            r#"{"rate_limits":{"five_hour":{"used_percentage":30.0,"resets_at":1900000000},
                "seven_day":{"used_percentage":30.0,"resets_at":1905000000}}}"#,
        );
        assert!(out.contains("30%"), "5h pct missing: {out:?}");
        // Only one percentage value (the 5h one); weekly is hidden below 50%.
        assert_eq!(
            out.matches('%').count(),
            1,
            "weekly should be hidden: {out:?}"
        );
    }

    #[test]
    fn weekly_shown_at_or_above_show_at() {
        let out = render_rl(
            r#"{"rate_limits":{"five_hour":{"used_percentage":60.0,"resets_at":1900000000},
                "seven_day":{"used_percentage":64.0,"resets_at":1905000000}}}"#,
        );
        assert!(out.contains("60%"), "5h pct missing: {out:?}");
        assert!(out.contains("64%"), "weekly pct missing: {out:?}");
    }

    #[test]
    fn over_limit_renders() {
        let out = render_rl(
            r#"{"rate_limits":{"five_hour":{"used_percentage":105.0,"resets_at":1900000000}}}"#,
        );
        assert!(out.contains("105%"), "over-limit pct missing: {out:?}");
        // crit color for >= crit threshold.
        let crit = themes::get("tokyo-night").bar_crit.fg();
        assert!(out.contains(&crit), "crit color missing: {out:?}");
    }

    #[test]
    fn leaked_timestamp_pct_rejected() {
        // used_percentage > 999 (a leaked epoch) is rejected; with no reset the
        // window shows nothing.
        let out = render_rl(r#"{"rate_limits":{"five_hour":{"used_percentage":1900000000}}}"#);
        assert!(
            !out.contains('%'),
            "leaked timestamp must not render: {out:?}"
        );
    }

    #[test]
    fn five_hour_reset_only_no_pct() {
        // No percentage, but a future reset → window still shown (countdown only).
        let out = render_rl(r#"{"rate_limits":{"five_hour":{"resets_at":1900000000}}}"#);
        assert!(!out.is_empty(), "reset-only window should render: {out:?}");
        assert!(!out.contains('%'), "no pct expected: {out:?}");
    }

    #[test]
    fn empty_input_renders_nothing() {
        assert_eq!(render_rl("{}"), "");
    }
}
