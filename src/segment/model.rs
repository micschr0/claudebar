//! Model + effort segment.
//!
//! Contract (matches the bash script's "model + effort" block):
//! ```text
//! Strip control bytes from display_name and effort.level (host-provided)
//!   via crate::sanitize::strip_control.
//! If both are empty/None -> emit nothing, return false.
//! If a model name is present: emit model icon (style.glyphs.model, dim) +
//!   name in theme.model. Bash: ${C_MOD}<model> %s${R}
//! If an effort level is present: when a model name was also emitted, emit a
//!   separating space first; then effort icon (style.glyphs.effort, dim) + the
//!   level string colored by level:
//!     low|medium -> theme.dim; high -> bar_ok; xhigh -> bar_warn;
//!     max -> theme.effort_max; anything else -> theme.dim.
//!   Note: effort is ABSENT for models without the param — gate on presence,
//!   never on a specific value.
//! Return true if anything was emitted.
//! ```

use crate::render::SegmentWriter;
use crate::sanitize::strip_control;
use crate::segment::{RenderCtx, Segment};

pub struct Model;

impl Segment for Model {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let name = ctx
            .input
            .model
            .display_name
            .as_deref()
            .map(strip_control)
            .filter(|s| !s.is_empty());
        let level = ctx
            .input
            .effort
            .level
            .as_deref()
            .map(strip_control)
            .filter(|s| !s.is_empty());

        if name.is_none() && level.is_none() {
            return false;
        }

        if let Some(name) = &name {
            out.icon(ctx.style.glyphs.model);
            out.colored(ctx.theme.model, name);
        }

        if let Some(level) = &level {
            // Separate the effort run from the model run when both render.
            if name.is_some() {
                out.raw(" ");
            }
            let color = match level.as_str() {
                "low" | "medium" => ctx.theme.dim,
                "high" => ctx.theme.bar_ok,
                "xhigh" => ctx.theme.bar_warn,
                "max" => ctx.theme.effort_max,
                _ => ctx.theme.dim,
            };
            out.icon(ctx.style.glyphs.effort);
            out.colored(color, level);
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::model::input::{Effort, Model as ModelData};
    use crate::model::{Config, InputData, SegmentKind};
    use crate::render::render_with;
    use crate::{styles, themes};

    fn render_model(name: Option<&str>, level: Option<&str>) -> String {
        let input = InputData {
            model: ModelData {
                display_name: name.map(str::to_string),
            },
            effort: Effort {
                level: level.map(str::to_string),
            },
            ..Default::default()
        };
        let cfg = Config {
            segments: vec![SegmentKind::Model],
            ..Default::default()
        };
        let theme = themes::get(&cfg.theme);
        let style = styles::get(&cfg.style);
        render_with(&input, &cfg, &theme, &style, 0, None)
    }

    #[test]
    fn renders_name_and_effort() {
        let out = render_model(Some("Opus 4.8"), Some("high"));
        assert!(out.contains("Opus 4.8"), "name missing: {out:?}");
        assert!(out.contains("high"), "effort missing: {out:?}");
        // model color 208 + bar_ok 114 (high).
        assert!(out.contains("\x1b[38;5;208m"), "model color: {out:?}");
        assert!(out.contains("\x1b[38;5;114m"), "high color: {out:?}");
    }

    #[test]
    fn effort_absent_renders_model_only() {
        let out = render_model(Some("Opus 4.8"), None);
        assert!(out.contains("Opus 4.8"), "name missing: {out:?}");
        assert!(out.contains("\x1b[38;5;208m"), "model color: {out:?}");
    }

    #[test]
    fn effort_only_renders_without_leading_space() {
        // name absent, level present: effort run stands alone, no leading space.
        let out = render_model(None, Some("high"));
        assert!(out.contains("high"), "effort missing: {out:?}");
        assert!(!out.starts_with(' '), "unexpected leading space: {out:?}");
        assert!(out.contains("\x1b[38;5;114m"), "high color: {out:?}");
    }

    #[test]
    fn max_uses_effort_max_color() {
        let out = render_model(Some("Opus 4.8"), Some("max"));
        assert!(out.contains("\x1b[38;5;213m"), "effort_max color: {out:?}");
    }

    #[test]
    fn both_absent_renders_nothing() {
        assert_eq!(render_model(None, None), "");
        assert_eq!(render_model(Some(""), Some("")), "");
    }

    #[test]
    fn strips_injection_bytes() {
        // ESC injected through the model name must be stripped; the only ESC
        // bytes left are our own SGR codes (icon dim+reset, model color+reset) —
        // exactly 4. If the injected ESC leaked through it would be 5.
        let out = render_model(Some("ev\x1bil"), None);
        assert!(out.contains("evil"), "name not stripped cleanly: {out:?}");
        assert_eq!(out.matches('\u{1b}').count(), 4, "unexpected ESC: {out:?}");
    }
}
