//! Update-notice segment — a badge for a newer release.
//!
//! Renders `↑ 2026.8.20` when the cached update check found a stable release
//! newer than the running binary. Pure formatter: the cache read and the
//! version comparison happen in `render::render_with`, so this segment does no
//! I/O and shows nothing when `ctx.update` is `None`.

use crate::render::SegmentWriter;
use crate::segment::{RenderCtx, Segment};

pub struct UpdateNotice;

impl Segment for UpdateNotice {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let Some(version) = ctx.update else {
            return false;
        };
        // `icon()` is a no-op in icon-less styles, which would leave a bare
        // version number with nothing marking it as an update. The git segment
        // already emits its ahead/behind glyphs this way, ungated.
        out.colored_with(ctx.theme.ahead, |w| {
            w.raw_fmt(format_args!("{} {version}", ctx.style.glyphs.ahead));
        });
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Config, InputData, SegmentKind};
    use crate::render::SegmentWriter;
    use crate::segment::{RenderCtx, Segment, update_notice::UpdateNotice};
    use crate::update::Version;
    use crate::{styles, themes};

    fn render(update: Option<&Version>) -> Option<String> {
        let input = InputData::default();
        let cfg = Config {
            segments: vec![SegmentKind::UpdateNotice],
            ..Default::default()
        };
        let theme = themes::get(&cfg.theme);
        let style = styles::get(&cfg.style);
        let ctx = RenderCtx {
            input: &input,
            theme: &theme,
            style: &style,
            th: &cfg.thresholds,
            now: 0,
            home: None,
            tz_offset_seconds: 0,
            update,
        };
        let mut w = SegmentWriter::new(&theme, &style);
        UpdateNotice
            .render(&ctx, &mut w)
            .then(|| w.as_str().to_string())
    }

    #[test]
    fn shows_the_cached_version() {
        let v = Version::parse("2026.8.20").unwrap();
        let out = render(Some(&v)).expect("emitted");
        assert!(out.contains("2026.8.20"), "{out}");
    }

    /// Renders the badge in `style`, stripped of color, so the test sees the
    /// glyphs a user would.
    fn render_styled(style_name: &str) -> String {
        let input = InputData::default();
        let cfg = Config {
            segments: vec![SegmentKind::UpdateNotice],
            style: style_name.to_string(),
            ..Default::default()
        };
        let theme = themes::get(&cfg.theme);
        let style = styles::get(&cfg.style);
        let v = Version::parse("2026.8.20").unwrap();
        let ctx = RenderCtx {
            input: &input,
            theme: &theme,
            style: &style,
            th: &cfg.thresholds,
            now: 0,
            home: None,
            tz_offset_seconds: 0,
            update: Some(&v),
        };
        let mut w = SegmentWriter::new(&theme, &style);
        assert!(UpdateNotice.render(&ctx, &mut w), "{style_name}: no output");
        crate::render::strip_ansi(w.as_str())
    }

    #[test]
    fn every_style_marks_the_version_as_an_update() {
        for name in crate::styles::NAMES {
            let style = styles::get(name);
            let out = render_styled(name);
            // Icon-less styles (plain, ascii, minimal) must still show the
            // marker — a naked "2026.8.20" says nothing to the user.
            assert!(
                out.contains(style.glyphs.ahead),
                "{name}: badge is missing the '{}' marker, got: {out:?}",
                style.glyphs.ahead
            );
            assert!(out.contains("2026.8.20"), "{name}: {out:?}");
        }
    }

    #[test]
    fn emits_nothing_without_an_update() {
        assert_eq!(render(None), None);
    }
}
