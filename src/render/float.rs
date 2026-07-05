//! Plain-text float readout: on each render, write a one-line ANSI-free summary
//! of the selected segments to a file. This is the seam for piping the status
//! line to tmux, a terminal status bar, a menu-bar app, etc.
//!
//! Unlike the colored [`crate::render::render_line`] output, the float readout
//! contains no ANSI escape codes and no icons — just the segment text, joined by
//! a configurable separator. It is best-effort: any I/O error is silently
//! swallowed so a float failure can never break the status render.
//!
//! Rendering reuses the exact same [`Segment`] implementations as the colored
//! path (no second code path): each selected segment is rendered with the ASCII
//! style — `icons: false`, ASCII-only glyphs — and then stripped of ANSI color,
//! so the result is stable regardless of the user's configured theme/style.

use crate::model::{Config, InputData, SegmentKind};
use crate::render::SegmentWriter;
use crate::segment::RenderCtx;
use crate::{styles, themes};
use std::path::{Path, PathBuf};

/// Emit the plain-text float readout for `input` under `cfg`.
///
/// Renders the segments named in `cfg.thresholds.float_segments` (space-separated
/// kebab-case [`SegmentKind`] names) as plain text — no ANSI color, no icons —
/// joined by `float_sep`, and writes the result atomically to `float_file`
/// (expanding a leading `~` via `home`). Best-effort: the float is a side
/// effect of the status render, so any failure is silently ignored.
pub fn emit_float(input: &InputData, cfg: &Config, now: i64, home: Option<&str>) {
    let path = match resolve_path(&cfg.thresholds.float_file, home) {
        Some(p) => p,
        None => return,
    };
    let line = render_float(input, cfg, now, home);
    // Best-effort: a float write failure must never break the status line.
    let _ = write_atomic(&path, &line);
}

/// Render the float readout to a plain-text string (no I/O). Each segment named
/// in `float_segments` is rendered with the ASCII style and then stripped of
/// ANSI color, so the result is independent of the user's configured theme/style.
fn render_float(input: &InputData, cfg: &Config, now: i64, home: Option<&str>) -> String {
    let theme = themes::get(&cfg.theme);
    // The ASCII style is icons-off with ASCII-only glyphs — the cleanest base for
    // a plain-text readout. Colors are stripped afterwards in any case.
    let style = styles::ascii::style();
    let ctx = RenderCtx {
        input,
        theme: &theme,
        style: &style,
        th: &cfg.thresholds,
        now,
        home,
        tz_offset_seconds: 0,
    };

    let mut parts: Vec<String> = Vec::with_capacity(SegmentKind::ALL.len());
    for name in cfg.thresholds.float_segments.split_whitespace() {
        let Some(kind) = parse_segment(name) else {
            continue;
        };
        let mut w = SegmentWriter::new(&theme, &style);
        let emitted = kind.as_segment().render(&ctx, &mut w);
        if emitted && !w.is_empty() {
            parts.push(strip_ansi(w.as_str()));
        }
    }
    parts.join(&cfg.thresholds.float_sep)
}

/// Parse a single kebab-case segment name into a [`SegmentKind`], reusing the
/// same serde rename mapping the config/TOML path uses. Returns `None` for
/// unknown names so a typo in `float_segments` is silently dropped.
fn parse_segment(name: &str) -> Option<SegmentKind> {
    // Deserializing a JSON string reuses SegmentKind's `rename_all = "kebab-case"`
    // mapping without duplicating the name table.
    serde_json::from_str(&format!("\"{name}\"")).ok()
}

/// Strip ANSI CSI escape sequences (e.g. `\x1b[38;5;33m`, `\x1b[0m`) from `s`,
/// leaving only the visible text. Handles any CSI sequence: from ESC `[` it
/// skips the parameter bytes (0x30–0x3F) and intermediate bytes (0x20–0x2F) up
/// to and including the final byte (0x40–0x7E). A lone ESC not followed by `[`
/// drops only itself and its single follower (the writer never emits these).
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.next() {
                Some('[') => {
                    for inner in chars.by_ref() {
                        if ('\x40'..='\x7e').contains(&inner) {
                            break;
                        }
                    }
                }
                // Non-CSI escape: drop the ESC and its lone follower.
                Some(_) => {}
                None => break,
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Resolve `path`, expanding a leading `~` to `home`. Returns `None` when a `~`
/// is present but `home` is missing/empty — the expansion is impossible, so the
/// caller (best-effort) skips the write rather than guessing a location.
fn resolve_path(path: &str, home: Option<&str>) -> Option<PathBuf> {
    let home = home.filter(|h| !h.is_empty());
    if let Some(rest) = path.strip_prefix("~/") {
        Some(PathBuf::from(home?).join(rest))
    } else if path == "~" {
        Some(PathBuf::from(home?))
    } else {
        Some(PathBuf::from(path))
    }
}

/// Write `content` to `path` atomically: create parent dirs, write to a sibling
/// temp file, then rename over the target (a same-filesystem rename is atomic on
/// POSIX). The temp file is tagged with the process id so concurrent writers do
/// not clobber each other's staging files.
///
/// # Errors
///
/// Returns the underlying [`std::io::Error`]; the caller treats it as best-effort.
fn write_atomic(path: &Path, content: &str) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("claudebar-float");
    let tmp = parent.join(format!(".{name}.{}.tmp", std::process::id()));
    if let Err(e) = std::fs::write(&tmp, content) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    match std::fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{emit_float, parse_segment, render_float, resolve_path, strip_ansi};
    use crate::model::input::{Coerce, CostInfo, Model as ModelData};
    use crate::model::{Config, InputData, SegmentKind};
    use std::path::PathBuf;

    /// A config with the float readout enabled and `segments` set to `segs`.
    fn float_cfg(segs: &str) -> Config {
        let mut cfg = Config::default();
        cfg.thresholds.float = true;
        cfg.thresholds.float_segments = segs.to_string();
        cfg
    }

    #[test]
    fn strip_ansi_removes_color_codes() {
        assert_eq!(strip_ansi("\x1b[38;5;33mhi\x1b[0m"), "hi");
        assert_eq!(strip_ansi("\x1b[0m"), "");
        assert_eq!(strip_ansi("plain"), "plain");
        assert_eq!(strip_ansi(""), "");
        assert_eq!(
            strip_ansi("\x1b[38;5;33ma\x1b[0m\x1b[38;5;31mb\x1b[0m"),
            "ab"
        );
    }

    #[test]
    fn parse_segment_reuses_serde_kebab_names() {
        assert_eq!(parse_segment("model"), Some(SegmentKind::Model));
        assert_eq!(parse_segment("rate-limits"), Some(SegmentKind::RateLimits));
        assert_eq!(parse_segment("dev-context"), Some(SegmentKind::DevContext));
        assert_eq!(parse_segment("cost"), Some(SegmentKind::Cost));
        assert_eq!(parse_segment("bogus"), None);
        assert_eq!(parse_segment(""), None);
    }

    #[test]
    fn resolve_path_expands_tilde() {
        let home = "/home/user";
        assert_eq!(
            resolve_path("~/foo.txt", Some(home)),
            Some(PathBuf::from("/home/user/foo.txt"))
        );
        assert_eq!(resolve_path("~", Some(home)), Some(PathBuf::from(home)));
        assert_eq!(
            resolve_path("/abs/path.txt", Some(home)),
            Some(PathBuf::from("/abs/path.txt"))
        );
        assert_eq!(resolve_path("~/foo.txt", None), None);
        assert_eq!(resolve_path("~/foo.txt", Some("")), None);
    }

    #[test]
    fn produces_plain_text_no_ansi() {
        let input = InputData {
            model: ModelData {
                display_name: Some("claude-sonnet".into()),
            },
            cost: CostInfo {
                total_cost_usd: Coerce(Some(1.23)),
                ..Default::default()
            },
            ..Default::default()
        };
        let cfg = float_cfg("model cost");
        let line = render_float(&input, &cfg, 0, None);
        assert!(
            !line.contains('\x1b'),
            "float must contain no ANSI: {line:?}"
        );
        assert!(line.contains("claude-sonnet"));
        assert!(line.contains("1.23"));
    }

    #[test]
    fn respects_segment_selection() {
        let input = InputData {
            model: ModelData {
                display_name: Some("claude-sonnet".into()),
            },
            cost: CostInfo {
                total_cost_usd: Coerce(Some(9.99)),
                ..Default::default()
            },
            ..Default::default()
        };
        // Only `model` selected — the `cost` data must NOT appear.
        let cfg = float_cfg("model");
        let line = render_float(&input, &cfg, 0, None);
        assert!(line.contains("claude-sonnet"));
        assert!(!line.contains("9.99"));
        assert!(
            !line.contains('·'),
            "no separator with a single segment: {line:?}"
        );
    }

    #[test]
    fn handles_empty_segments() {
        // No model/context/cost data at all — every selected segment is empty.
        let input = InputData::default();
        let cfg = float_cfg("model context cost");
        let line = render_float(&input, &cfg, 0, None);
        // Context renders "0" at zero tokens (new-user onboarding).
        assert_eq!(
            line, "0",
            "all-empty input yields just context zero: {line:?}"
        );

        // Mixed: model present, cost empty — the empty segment is dropped, leaving
        // no dangling separator around the model text.
        let input = InputData {
            model: ModelData {
                display_name: Some("claude-sonnet".into()),
            },
            ..Default::default()
        };
        let cfg = float_cfg("model cost");
        let line = render_float(&input, &cfg, 0, None);
        assert_eq!(line, "claude-sonnet");
    }

    #[test]
    fn unknown_segment_names_are_ignored() {
        let input = InputData {
            model: ModelData {
                display_name: Some("claude-sonnet".into()),
            },
            ..Default::default()
        };
        let cfg = float_cfg("model bogus not-a-segment");
        let line = render_float(&input, &cfg, 0, None);
        assert_eq!(line, "claude-sonnet");
    }

    #[test]
    fn uses_custom_separator() {
        // model = "m" (icons off → bare name); cost = "2.50" (no extra
        // padding — spacing is the separator's job). Joined by "::".
        let input = InputData {
            model: ModelData {
                display_name: Some("m".into()),
            },
            cost: CostInfo {
                total_cost_usd: Coerce(Some(2.5)),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut cfg = float_cfg("model cost");
        cfg.thresholds.float_sep = "::".into();
        let line = render_float(&input, &cfg, 0, None);
        assert_eq!(line, "m::2.50");
    }

    #[test]
    fn emit_float_writes_plain_text_file() {
        let path = std::env::temp_dir().join(format!(
            "claudebar-float-test-{}-{}.txt",
            std::process::id(),
            line!()
        ));
        let _ = std::fs::remove_file(&path);

        let input = InputData {
            model: ModelData {
                display_name: Some("claude-sonnet".into()),
            },
            ..Default::default()
        };
        let mut cfg = float_cfg("model");
        cfg.thresholds.float_file = path.to_string_lossy().into_owned();

        emit_float(&input, &cfg, 0, None);

        let written = std::fs::read_to_string(&path).expect("float file should be written");
        assert!(
            !written.contains('\x1b'),
            "file must contain no ANSI: {written:?}"
        );
        assert!(written.contains("claude-sonnet"));

        let _ = std::fs::remove_file(&path);
    }
}
