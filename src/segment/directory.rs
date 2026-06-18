//! Directory segment: the abbreviated current working directory.
//!
//! Matches the bash script's directory segment: fish-style abbreviation with
//! `~` for `$HOME`, the whole thing in `theme.dir`, no icon. Bash emitted
//! `printf "${C_DIR} %s${R}"` — a leading space inside the colored run, then the
//! path. The path is host-provided; `abbreviate_path` strips control bytes.
//!
//! (Implemented in the foundation so every theme/style/TUI worker has a live
//! rendering segment to verify against.)

use crate::render::SegmentWriter;
use crate::sanitize::abbreviate_path;
use crate::segment::{RenderCtx, Segment};

pub struct Directory;

impl Segment for Directory {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let cwd = match ctx.input.cwd.as_deref() {
            Some(c) if !c.is_empty() => c,
            _ => return false,
        };
        let path = abbreviate_path(cwd, ctx.home);
        // Leading space inside the colored run, matching bash's `${C_DIR} %s`.
        out.colored(ctx.theme.dir, &format!(" {path}"));
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Config, InputData, SegmentKind};
    use crate::render::render_with;
    use crate::{styles, themes};

    fn render_dir(cwd: Option<&str>, home: Option<&str>) -> String {
        let input = InputData {
            cwd: cwd.map(str::to_string),
            ..Default::default()
        };
        let cfg = Config {
            segments: vec![SegmentKind::Directory],
            ..Default::default()
        };
        let theme = themes::get(&cfg.theme);
        let style = styles::get(&cfg.style);
        render_with(&input, &cfg, &theme, &style, 0, home)
    }

    #[test]
    fn renders_abbreviated_path_in_dir_color() {
        let out = render_dir(Some("/home/me/projects/app"), Some("/home/me"));
        assert!(out.contains("~/p/app"), "got: {out:?}");
        assert!(out.contains("\x1b[38;5;33m"), "dir color missing: {out:?}");
    }

    #[test]
    fn empty_cwd_renders_nothing() {
        assert_eq!(render_dir(None, Some("/home/me")), "");
        assert_eq!(render_dir(Some(""), Some("/home/me")), "");
    }

    #[test]
    fn strips_injection_bytes() {
        let out = render_dir(Some("/x/\x1b[31mevil"), None);
        // Only our own two SGR codes (dir color + reset) carry an ESC byte; the
        // ESC injected through cwd must be stripped, so the count stays at 2.
        assert_eq!(out.matches('\u{1b}').count(), 2, "unexpected ESC: {out:?}");
    }
}
