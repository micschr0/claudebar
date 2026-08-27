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
        out.colored_with(ctx.theme.ahead, |w| {
            w.icon(ctx.style.glyphs.ahead);
            w.raw_fmt(format_args!("{version}"));
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

    #[test]
    fn emits_nothing_without_an_update() {
        assert_eq!(render(None), None);
    }
}
