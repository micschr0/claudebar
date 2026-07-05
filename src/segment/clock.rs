//! Clock segment — current time, 12h or 24h, with system timezone via
//! `date +%z` and locale-based 12h/24h preference.
//!
//! Renders `◷ 02:45 pm` (12h) or `◷ 14:45:00` (24h). Controlled by
//! `thresholds.clock_mode` (`"auto"`, `"12h"`, `"24h"`, `"off"`) and
//! `thresholds.clock_seconds`. No JSON dependency — uses `ctx.now` plus
//! `ctx.tz_offset_seconds`.
//!
//! # Detection
//!
//! - **Timezone offset**: `date +%z` subprocess, cached in [`LazyLock`].
//!   Falls back to 0 (UTC). Uses `time` crate for time formatting only.
//! - **12h/24h preference**: inspected from `LC_TIME` → `LC_ALL` → `LANG`
//!   environment variables, checked against a static country-code table.

use crate::model::Thresholds;
use crate::render::SegmentWriter;
use crate::segment::{RenderCtx, Segment};
use std::sync::LazyLock;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};

/// Renders the current wall-clock time, with optional seconds and 12h/24h
/// auto-detection from the system locale. Controlled by
/// [`Thresholds::clock_mode`] and [`Thresholds::clock_seconds`].
pub struct Clock;

// ---------------------------------------------------------------------------
// 12h/24h detection
// ---------------------------------------------------------------------------

/// Countries whose predominant clock convention is 12-hour.
const TWELVE_H_COUNTRIES: &[&str] = &[
    "US", "GB", "AU", "NZ", "IE", "MT", "ZA", "IN", "PK", "BD", "PH", "KE", "NG", "GH", "JM", "TT",
    "BB", "BS", "BZ", "GY", "CA", "MX", "AR", "CL", "CO", "PE", "VE", "EC", "BO", "PY", "UY", "CR",
    "PA", "DO", "GT", "HN", "SV", "NI", "EG", "SA", "AE", "QA", "KW", "OM", "BH", "JO", "LB",
];

fn country_prefers_12h(language: Option<&str>, country: Option<&str>) -> bool {
    if language == Some("fr") && country == Some("CA") {
        return false;
    }
    TWELVE_H_COUNTRIES.contains(&country.unwrap_or(""))
}

fn extract_country(locale: &str) -> Option<&str> {
    let after_underscore = locale.split('_').nth(1)?;
    let before_dot = after_underscore.split('.').next().unwrap_or("");
    if before_dot.len() >= 2 {
        Some(&before_dot[..2])
    } else {
        None
    }
}

fn extract_language(locale: &str) -> Option<&str> {
    let lang = locale.split(['_', '.']).next()?;
    if !lang.is_empty() { Some(lang) } else { None }
}

fn detect_12h_preference() -> bool {
    let locale = std::env::var("LC_TIME")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();

    if locale.is_empty()
        || locale == "C"
        || locale == "C.UTF-8"
        || locale == "POSIX"
        || locale.starts_with("C.")
    {
        return false;
    }

    let language = extract_language(&locale);
    let country = extract_country(&locale);
    country_prefers_12h(language, country)
}

static PREFERS_12H: LazyLock<bool> = LazyLock::new(detect_12h_preference);

fn effective_clock_mode(th: &Thresholds) -> &str {
    match th.clock_mode.as_str() {
        "auto" => {
            if *PREFERS_12H {
                "12h"
            } else {
                "24h"
            }
        }
        "12h" | "24h" | "off" => &th.clock_mode,
        _ => "12h",
    }
}

// ---------------------------------------------------------------------------
// Timezone offset detection (date +%z subprocess)
// ---------------------------------------------------------------------------

fn parse_offset(s: &str) -> Option<i32> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (sign, rest) = match s.chars().next() {
        Some('+') => (1, &s[1..]),
        Some('-') => (-1, &s[1..]),
        _ => return None,
    };
    let rest = rest.replace(':', "");
    match rest.len() {
        4 => {
            let hh: i32 = rest[..2].parse().ok()?;
            let mm: i32 = rest[2..].parse().ok()?;
            if hh > 23 || mm > 59 {
                return None;
            }
            Some(sign * (hh * 3600 + mm * 60))
        }
        2 => {
            let hh: i32 = rest.parse().ok()?;
            if hh > 23 {
                return None;
            }
            Some(sign * hh * 3600)
        }
        _ => None,
    }
}

/// Detect the local timezone offset in seconds east of UTC via `date +%z`.
/// Cached in [`LazyLock`] for the process lifetime.
pub(crate) fn detect_tz_offset() -> i32 {
    static OFFSET: LazyLock<i32> = LazyLock::new(|| {
        std::process::Command::new("date")
            .args(["+%z"])
            .output()
            .ok()
            .and_then(|o| parse_offset(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or(0)
    });
    *OFFSET
}

// ---------------------------------------------------------------------------
// Time formatting (via `time` crate — pure computation, no syscalls)
// ---------------------------------------------------------------------------

const FMT_24H_S: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[hour repr:24]:[minute]:[second]");
const FMT_24H: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[hour repr:24]:[minute]");
const FMT_12H_S: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[hour repr:12]:[minute]:[second] [period case:lower]");
const FMT_12H: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[hour repr:12]:[minute] [period case:lower]");

// ---------------------------------------------------------------------------
// Segment impl
// ---------------------------------------------------------------------------

impl Segment for Clock {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let mode = effective_clock_mode(ctx.th);
        if mode == "off" || ctx.now < 0 {
            return false;
        }

        let offset = match UtcOffset::from_whole_seconds(ctx.tz_offset_seconds) {
            Ok(o) => o,
            Err(_) => return false,
        };
        let dt = match OffsetDateTime::from_unix_timestamp(ctx.now) {
            Ok(dt) => dt.to_offset(offset),
            Err(_) => return false,
        };

        let fmt: &[time::format_description::BorrowedFormatItem<'_>] = if mode == "24h" {
            if ctx.th.clock_seconds {
                FMT_24H_S
            } else {
                FMT_24H
            }
        } else {
            if ctx.th.clock_seconds {
                FMT_12H_S
            } else {
                FMT_12H
            }
        };

        let time_str = match dt.format(fmt) {
            Ok(s) => s,
            Err(_) => return false,
        };

        out.colored_with(ctx.theme.clock, |w| {
            w.icon(ctx.style.glyphs.time);
            w.raw(&time_str);
        });
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{styles, themes};

    // ------------------------------------------------------------------
    // 12h/24h country detection
    // ------------------------------------------------------------------

    #[test]
    fn country_prefers_12h_us() {
        assert!(country_prefers_12h(Some("en"), Some("US")));
    }
    #[test]
    fn country_prefers_24h_de() {
        assert!(!country_prefers_12h(Some("de"), Some("DE")));
    }
    #[test]
    fn country_prefers_12h_gb() {
        assert!(country_prefers_12h(Some("en"), Some("GB")));
    }
    #[test]
    fn country_prefers_24h_fr() {
        assert!(!country_prefers_12h(Some("fr"), Some("FR")));
    }
    #[test]
    fn country_prefers_12h_au() {
        assert!(country_prefers_12h(Some("en"), Some("AU")));
    }
    #[test]
    fn country_prefers_12h_in() {
        assert!(country_prefers_12h(Some("en"), Some("IN")));
    }
    #[test]
    fn country_prefers_24h_jp() {
        assert!(!country_prefers_12h(Some("ja"), Some("JP")));
    }
    #[test]
    fn country_prefers_24h_cn() {
        assert!(!country_prefers_12h(Some("zh"), Some("CN")));
    }

    #[test]
    fn fr_ca_override_twentyfour_h() {
        assert!(!country_prefers_12h(Some("fr"), Some("CA")));
        assert!(country_prefers_12h(Some("en"), Some("CA")));
    }

    #[test]
    fn unknown_country_defaults_24h() {
        assert!(!country_prefers_12h(None, None));
        assert!(!country_prefers_12h(Some("xx"), Some("XX")));
    }

    // ------------------------------------------------------------------
    // extract_country / extract_language
    // ------------------------------------------------------------------

    #[test]
    fn extract_country_en_us() {
        assert_eq!(extract_country("en_US.UTF-8"), Some("US"));
    }
    #[test]
    fn extract_country_de_de() {
        assert_eq!(extract_country("de_DE.utf8"), Some("DE"));
    }
    #[test]
    fn extract_country_no_underscore() {
        assert_eq!(extract_country("C.UTF-8"), None);
    }
    #[test]
    fn extract_language_en() {
        assert_eq!(extract_language("en_US.UTF-8"), Some("en"));
    }
    #[test]
    fn extract_language_de() {
        assert_eq!(extract_language("de_DE.utf8"), Some("de"));
    }
    #[test]
    fn extract_language_no_underscore() {
        assert_eq!(extract_language("C.UTF-8"), Some("C"));
    }

    // ------------------------------------------------------------------
    // effective_clock_mode
    // ------------------------------------------------------------------

    fn th_with_mode(mode: &str) -> Thresholds {
        Thresholds {
            clock_mode: mode.into(),
            ..Default::default()
        }
    }

    #[test]
    fn effective_mode_12h_passthrough() {
        assert_eq!(effective_clock_mode(&th_with_mode("12h")), "12h");
    }
    #[test]
    fn effective_mode_24h_passthrough() {
        assert_eq!(effective_clock_mode(&th_with_mode("24h")), "24h");
    }
    #[test]
    fn effective_mode_off_passthrough() {
        assert_eq!(effective_clock_mode(&th_with_mode("off")), "off");
    }
    #[test]
    fn effective_mode_auto_resolves() {
        let th = th_with_mode("auto");
        let mode = effective_clock_mode(&th);
        assert!(mode == "12h" || mode == "24h");
    }
    #[test]
    fn effective_mode_unknown_defaults_12h() {
        assert_eq!(effective_clock_mode(&th_with_mode("future-mode")), "12h");
    }

    // ------------------------------------------------------------------
    // parse_offset
    // ------------------------------------------------------------------

    #[test]
    fn offset_positive() {
        assert_eq!(parse_offset("+0200"), Some(7200));
    }
    #[test]
    fn offset_negative() {
        assert_eq!(parse_offset("-0500"), Some(-18000));
    }
    #[test]
    fn offset_utc() {
        assert_eq!(parse_offset("+0000"), Some(0));
    }
    #[test]
    fn offset_half_hour() {
        assert_eq!(parse_offset("+0530"), Some(19800));
    }
    #[test]
    fn offset_quarter_hour() {
        assert_eq!(parse_offset("+0545"), Some(20700));
    }
    #[test]
    fn offset_negative_quarter() {
        assert_eq!(parse_offset("-0945"), Some(-35100));
    }
    #[test]
    fn offset_colon_variant() {
        assert_eq!(parse_offset("+02:00"), Some(7200));
    }
    #[test]
    fn offset_hours_only() {
        assert_eq!(parse_offset("+02"), Some(7200));
    }
    #[test]
    fn offset_trailing_newline() {
        assert_eq!(parse_offset("+0200\n"), Some(7200));
    }
    #[test]
    fn offset_garbage() {
        assert_eq!(parse_offset("garbage"), None);
    }
    #[test]
    fn offset_empty() {
        assert_eq!(parse_offset(""), None);
    }
    #[test]
    fn offset_no_sign() {
        assert_eq!(parse_offset("0200"), None);
    }
    #[test]
    fn offset_hours_too_large() {
        assert_eq!(parse_offset("+2500"), None);
    }
    #[test]
    fn offset_minutes_too_large() {
        assert_eq!(parse_offset("+0260"), None);
    }

    // ------------------------------------------------------------------
    // Segment render
    // ------------------------------------------------------------------

    fn strip_ansi(raw: &str) -> String {
        let mut out = String::with_capacity(raw.len());
        let mut in_escape = false;
        for ch in raw.chars() {
            if in_escape {
                if ch == 'm' {
                    in_escape = false;
                }
            } else if ch == '\x1b' {
                in_escape = true;
            } else {
                out.push(ch);
            }
        }
        out
    }

    fn render_clock(epoch: i64, offset_seconds: i32, mode: &str, seconds: bool) -> String {
        let input = crate::model::InputData::default();
        let config = crate::model::Config::default();
        let theme = themes::get(&config.theme);
        let style = styles::get(&config.style);
        let th = Thresholds {
            clock_mode: mode.into(),
            clock_seconds: seconds,
            ..Default::default()
        };
        let ctx = RenderCtx {
            input: &input,
            theme: &theme,
            style: &style,
            th: &th,
            now: epoch,
            home: None,
            tz_offset_seconds: offset_seconds,
        };
        let mut w = SegmentWriter::new(&theme, &style);
        if Clock.render(&ctx, &mut w) {
            strip_ansi(w.as_str())
        } else {
            String::new()
        }
    }

    #[test]
    fn clock_utc_midnight_24h() {
        assert!(render_clock(0, 0, "24h", false).contains("00:00"));
    }
    #[test]
    fn clock_utc_midnight_24h_seconds() {
        assert!(render_clock(0, 0, "24h", true).contains("00:00:00"));
    }
    #[test]
    fn clock_with_positive_offset() {
        assert!(render_clock(36000, 7200, "24h", true).contains("12:00:00"));
    }
    #[test]
    fn clock_with_negative_offset() {
        assert!(render_clock(3600, -18000, "24h", true).contains("20:00:00"));
    }
    #[test]
    fn clock_12h_mode() {
        assert!(render_clock(50400, 0, "12h", true).contains("02:00:00 pm"));
    }
    #[test]
    fn clock_12h_midnight() {
        assert!(render_clock(0, 0, "12h", false).contains("12:00 am"));
    }
    #[test]
    fn clock_12h_noon() {
        assert!(render_clock(43200, 0, "12h", true).contains("12:00:00 pm"));
    }
    #[test]
    fn clock_24h_mode() {
        assert!(render_clock(86399, 0, "24h", true).contains("23:59:59"));
    }
    #[test]
    fn clock_negative_epoch_returns_false() {
        let input = crate::model::InputData::default();
        let config = crate::model::Config::default();
        let theme = themes::get(&config.theme);
        let style = styles::get(&config.style);
        let th = Thresholds::default();
        let ctx = RenderCtx {
            input: &input,
            theme: &theme,
            style: &style,
            th: &th,
            now: -1,
            home: None,
            tz_offset_seconds: 0,
        };
        let mut w = SegmentWriter::new(&theme, &style);
        assert!(!Clock.render(&ctx, &mut w));
    }
    #[test]
    fn clock_negative_offset_wraps_local_time() {
        // UTC 01:00:00 - 2h00m01s offset = local 22:59:59 previous day.
        assert!(render_clock(3600, -7201, "24h", true).contains("22:59:59"));
    }
    #[test]
    fn clock_off_mode_returns_false() {
        let input = crate::model::InputData::default();
        let config = crate::model::Config::default();
        let theme = themes::get(&config.theme);
        let style = styles::get(&config.style);
        let th = Thresholds {
            clock_mode: "off".into(),
            ..Default::default()
        };
        let ctx = RenderCtx {
            input: &input,
            theme: &theme,
            style: &style,
            th: &th,
            now: 0,
            home: None,
            tz_offset_seconds: 0,
        };
        let mut w = SegmentWriter::new(&theme, &style);
        assert!(!Clock.render(&ctx, &mut w));
    }
    #[test]
    fn clock_no_seconds() {
        let out = render_clock(50400, 0, "12h", false);
        assert!(out.contains("02:00 pm"), "got: {out:?}");
        assert!(!out.contains(":00:00"), "should not have seconds: {out:?}");
    }
}
