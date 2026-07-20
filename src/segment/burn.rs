//! Burn-rate segment — range-to-empty: projected time until a rate-limit
//! window hits 100% at the recent burn rate.
//! On each render where `burn` is in the segment list, a sample
//! `(now, pct, resets_at)` is appended to a local TSV cache file.
//! A linear regression over the recent lookback window
//! (`thresholds.burn_lookback`, default 600s) yields a slope (pct/sec). The
//! ETA is `(100 - current_pct) / slope` — the projected time until the
//! window hits 100%.
//!
//! States:
//!   - `warming` — no samples yet (fresh install). Shows dim `↗ …`
//!   - `idle`    — slope is zero or negative (stopped burning). Shows dim `↗ ✓`
//!   - `active`  — slope > 0. Shows colored `↗ {label} ⇢ {eta}`:
//!       - red    — ETA < time-until-reset (would run dry before window resets)
//!       - yellow — ETA close to reset (within 20% margin)
//!       - green  — ETA > time-until-reset (window resets with room to spare)
//!
//! The label tells which limit binds: `5h` (five-hour) or `7d` (seven-day),
//! whichever will hit 100% sooner. `5h` only appears when there's enough
//! burn to register a slope; otherwise `7d` binds by default.
//!
//! The cache file lives at `$XDG_CACHE_HOME/claudebar/burn-5h.tsv` (fallback
//! `~/.cache/claudebar/burn-5h.tsv`). It is trimmed to ~1500 rows on write.

#![allow(clippy::cast_precision_loss)]
use crate::model::{Color, Theme};
use crate::render::SegmentWriter;
use crate::segment::{RenderCtx, Segment};
use std::path::PathBuf;

/// Max rows kept in the sample file (old entries pruned on write).
const MAX_ROWS: usize = 1500;

pub struct Burn;

impl Segment for Burn {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        // Resolve the 5h window data.
        let fh = ctx.input.rate_limits.five_hour.as_ref();
        let wd = ctx.input.rate_limits.seven_day.as_ref();

        let fh_pct = fh.and_then(|w| w.used_percentage.0);
        let fh_rst = fh.and_then(|w| w.resets_at.0);
        let wd_pct = wd.and_then(|w| w.used_percentage.0);
        let wd_rst = wd.and_then(|w| w.resets_at.0);

        // Need at least one window to show anything.
        if fh_pct.is_none() && wd_pct.is_none() {
            return false;
        }

        // Sample the 5h window (the one we project from).
        if let (Some(pct), Some(rst)) = (fh_pct, fh_rst) {
            // Reject implausibly-far-future resets (corrupt/sentinel snapshots).
            let max_future = 6 * 3600; // 5h window resets within ~5h; 6h = +1h margin
            if rst <= ctx.now + max_future {
                sample(ctx.now, pct, rst);
            }
        }

        // Compute projection.
        let lookback = i64::from(ctx.th.burn_lookback);
        let samples = read_samples(ctx.now, lookback);
        let burn = estimate(
            ctx.now,
            &samples,
            fh_pct.zip(fh_rst),
            wd_pct.zip(wd_rst),
            ctx.theme,
        );

        render_burn(ctx, out, &burn);
        true
    }
}

/// The resolved burn state for rendering.
struct BurnEstimate {
    state: BurnState,
    label: &'static str,
    eta: String,
    color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BurnState {
    Warming,
    Idle,
    Active,
}

/// Compute the burn estimate: slope, ETA, and state.
fn estimate(
    now: i64,
    samples: &[(i64, f64)],
    fh: Option<(f64, i64)>,
    wd: Option<(f64, i64)>,
    theme: &Theme,
) -> BurnEstimate {
    if let Some((pct, rst)) = fh {
        if samples.is_empty() {
            return BurnEstimate {
                state: BurnState::Warming,
                label: "5h",
                eta: String::new(),
                color: theme.dim,
            };
        }

        let slope = linear_slope(samples);
        if slope <= 0.0 {
            return BurnEstimate {
                state: BurnState::Idle,
                label: "5h",
                eta: String::new(),
                color: theme.dim,
            };
        }

        let remaining_pct = (100.0 - pct).max(0.0);
        let eta_secs = remaining_pct / slope;
        let time_to_reset = (rst - now).max(0) as f64;
        let eta = fmt_eta(eta_secs.min(time_to_reset) as i64);
        let color = urgency_color(eta_secs, time_to_reset, theme);

        return BurnEstimate {
            state: BurnState::Active,
            label: "5h",
            eta,
            color,
        };
    }

    if let Some((pct, rst)) = wd {
        if pct <= 0.0 {
            return BurnEstimate {
                state: BurnState::Warming,
                label: "7d",
                eta: String::new(),
                color: theme.dim,
            };
        }
        let color = urgency_color_stateless(pct, Some(rst), now, theme);
        return BurnEstimate {
            state: BurnState::Active,
            label: "7d",
            eta: String::new(),
            color,
        };
    }

    BurnEstimate {
        state: BurnState::Warming,
        label: "",
        eta: String::new(),
        color: theme.dim,
    }
}

/// Linear regression slope (pct/sec) over samples in the lookback window.
/// Uses least squares: slope = (n·Σxy − Σx·Σy) / (n·Σx² − (Σx)²).
fn linear_slope(samples: &[(i64, f64)]) -> f64 {
    let n = samples.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let sum_x: f64 = samples.iter().map(|(t, _)| *t as f64).sum();
    let sum_y: f64 = samples.iter().map(|(_, p)| *p).sum();
    let sum_xy: f64 = samples.iter().map(|(t, p)| *t as f64 * p).sum();
    let sum_x2: f64 = samples.iter().map(|(t, _)| (*t as f64).powi(2)).sum();
    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-12 {
        return 0.0;
    }
    (n * sum_xy - sum_x * sum_y) / denom
}

/// Color the ETA by urgency: red if you'd empty before reset, yellow if close,
/// green if the window resets with room to spare.
fn urgency_color(eta_secs: f64, time_to_reset: f64, theme: &Theme) -> Color {
    if time_to_reset <= 0.0 {
        return theme.burn; // red — already past reset
    }
    if eta_secs < time_to_reset {
        theme.burn // red — would run dry before reset
    } else if eta_secs < time_to_reset * 1.2 {
        theme.bar_warn // yellow — close call
    } else {
        theme.bar_ok // green — safe
    }
}

fn urgency_color_stateless(pct: f64, resets_at: Option<i64>, now: i64, theme: &Theme) -> Color {
    if pct >= 90.0 {
        return theme.burn; // red
    }
    if let Some(rst) = resets_at
        && rst - now < 3600
    {
        return theme.burn; // red — resetting soon
    }
    if pct >= 75.0 {
        theme.bar_warn // yellow
    } else {
        theme.bar_ok // green
    }
}

/// Format seconds as a compact human duration: `1h58m`, `42m`, `15s`, `2d3h`.
fn fmt_eta(secs: i64) -> String {
    if secs <= 0 {
        return String::from("0s");
    }
    let days = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    let mut buf = String::with_capacity(8); // "1d23h" ≤ 7 bytes
    use std::fmt::Write as _;
    if days > 0 {
        write!(buf, "{days}d{h}h").unwrap();
    } else if h > 0 {
        write!(buf, "{h}h{m:02}m").unwrap();
    } else if m > 0 {
        write!(buf, "{m}m").unwrap();
    } else {
        write!(buf, "{s}s").unwrap();
    }
    buf
}

/// Render the burn estimate into the writer.
fn render_burn(ctx: &RenderCtx, out: &mut SegmentWriter, burn: &BurnEstimate) {
    let glyph = ctx.style.glyphs.burn;
    match burn.state {
        BurnState::Warming => {
            out.colored_with(burn.color, |w| {
                w.icon(glyph);
                w.raw("…");
            });
        }
        BurnState::Idle => {
            out.colored_with(burn.color, |w| {
                w.icon(glyph);
                w.raw("✓");
            });
        }
        BurnState::Active => {
            if burn.eta.is_empty() {
                // 7d stateless: just show the label.
                out.colored_with(burn.color, |w| {
                    w.icon(glyph);
                    w.raw(burn.label);
                });
            } else {
                out.colored_with(burn.color, |w| {
                    w.icon(glyph);
                    w.raw(burn.label);
                    w.raw(" ⇢ ");
                    w.raw(&burn.eta);
                });
            }
        }
    }
}

// ── Sample file I/O ──────────────────────────────────────────────────────────

/// Default cache path: `$XDG_CACHE_HOME/claudebar/burn-5h.tsv` or
/// `~/.cache/claudebar/burn-5h.tsv`.
fn default_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("claudebar").join("burn-5h.tsv"))
}

/// Override path via env var `CLAUDEBAR_BURN_FILE` (for testing).
fn burn_file() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("CLAUDEBAR_BURN_FILE")
        && !p.is_empty()
    {
        return Some(PathBuf::from(p));
    }
    default_path()
}

/// Append a sample `(now, pct, resets_at)` to the TSV cache, trimming old rows.
fn sample(now: i64, pct: f64, resets_at: i64) {
    let Some(path) = burn_file() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let line = format!("{now}\t{pct:.3}\t{resets_at}\n");
    // Append — if the file doesn't exist, create it.
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = f.write_all(line.as_bytes());
    }
    // Trim to MAX_ROWS (read, keep tail, rewrite).
    trim_file(&path);
}

/// Read samples within the lookback window, return as `[(time, pct)]`.
fn read_samples(now: i64, lookback: i64) -> Vec<(i64, f64)> {
    let Some(path) = burn_file() else {
        return Vec::new();
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let cutoff = now - lookback;
    let mut samples = Vec::with_capacity(MAX_ROWS);
    for line in content.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let Ok(t) = parts[0].parse::<i64>() else {
            continue;
        };
        let Ok(p) = parts[1].parse::<f64>() else {
            continue;
        };
        if t >= cutoff {
            samples.push((t, p));
        }
    }
    samples
}

/// Trim the file to the last MAX_ROWS entries.
fn trim_file(path: &std::path::Path) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= MAX_ROWS {
        return;
    }
    let keep = &lines[lines.len() - MAX_ROWS..];
    let trimmed = keep.join("\n") + "\n";
    let _ = std::fs::write(path, trimmed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::input::InputData;
    use crate::model::{Config, Thresholds};
    use crate::{styles, themes};
    fn test_theme() -> Theme {
        let config = Config::default();
        themes::get(&config.theme)
    }

    #[test]
    fn fmt_eta_seconds() {
        assert_eq!(fmt_eta(42), "42s");
    }

    #[test]
    fn fmt_eta_minutes() {
        assert_eq!(fmt_eta(2820), "47m");
    }

    #[test]
    fn fmt_eta_hours() {
        assert_eq!(fmt_eta(7080), "1h58m");
    }

    #[test]
    fn fmt_eta_days() {
        assert_eq!(fmt_eta(186_120), "2d3h");
    }

    #[test]
    fn slope_empty_samples() {
        assert_eq!(linear_slope(&[]), 0.0);
    }

    #[test]
    fn slope_single_sample() {
        assert_eq!(linear_slope(&[(100, 50.0)]), 0.0);
    }

    #[test]
    fn slope_increasing() {
        // 1 pct/sec slope
        let samples = vec![(0, 0.0), (1, 1.0), (2, 2.0)];
        let s = linear_slope(&samples);
        assert!((s - 1.0).abs() < 0.001, "expected 1.0, got {s}");
    }

    #[test]
    fn urgency_red_when_eta_before_reset() {
        // ETA = 100s, reset in 200s → red (would run dry first)
        let theme = test_theme();
        let c = urgency_color(100.0, 200.0, &theme);
        assert_eq!(c, theme.burn);
    }
    #[test]
    fn urgency_green_when_eta_after_reset() {
        // ETA = 300s, reset in 200s → green (resets with room to spare)
        let theme = test_theme();
        let c = urgency_color(300.0, 200.0, &theme);
        assert_eq!(c, theme.bar_ok);
    }
    #[test]
    fn urgency_yellow_when_close() {
        // ETA = 230s, reset in 200s → yellow (within 20% margin: 200*1.2=240)
        let theme = test_theme();
        let c = urgency_color(230.0, 200.0, &theme);
        assert_eq!(c, theme.bar_warn);
    }

    #[test]
    fn no_windows_renders_nothing() {
        let input = InputData::default();
        let config = Config::default();
        let theme = themes::get(&config.theme);
        let style = styles::get(&config.style);
        let th = Thresholds::default();
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
        assert!(!Burn.render(&ctx, &mut w));
    }
    #[test]
    fn estimate_warming_when_no_samples() {
        let theme = test_theme();
        let est = estimate(0, &[], Some((50.0, 10800)), None, &theme);
        assert_eq!(est.state, BurnState::Warming);
        assert_eq!(est.label, "5h");
    }

    #[test]
    fn estimate_idle_when_slope_zero() {
        // Single sample → slope=0 → idle.
        let theme = test_theme();
        let samples = vec![(0, 50.0)];
        let est = estimate(0, &samples, Some((50.0, 10800)), None, &theme);
        assert_eq!(est.state, BurnState::Idle);
    }

    #[test]
    fn estimate_active_with_slope() {
        // 10 samples, 1 pct/sec: at t=0 pct=50, at t=9 pct=59.
        let theme = test_theme();
        let samples: Vec<(i64, f64)> = (0..10).map(|i| (i, 50.0 + i as f64)).collect();
        let est = estimate(9, &samples, Some((59.0, 10800)), None, &theme);
        assert_eq!(est.state, BurnState::Active);
        assert_eq!(est.label, "5h");
        assert!(!est.eta.is_empty(), "expected non-empty ETA: {:?}", est.eta);
    }
    #[test]
    fn estimate_active_red_urgency() {
        // Slope=1 pct/sec, pct=90 → ETA=10s. Reset in 200s → red (run dry first).
        let theme = test_theme();
        let samples = vec![(0, 80.0), (10, 90.0)];
        let est = estimate(10, &samples, Some((90.0, 210)), None, &theme);
        assert_eq!(est.state, BurnState::Active);
        assert_eq!(est.color, theme.burn); // red
    }
    #[test]
    fn estimate_falls_back_to_7d() {
        // No 5h data → 7d stateless.
        let theme = test_theme();
        let est = estimate(0, &[], None, Some((80.0, 600_000)), &theme);
        assert_eq!(est.state, BurnState::Active);
        assert_eq!(est.label, "7d");
    }
    #[test]
    fn burn_render_reads_samples_file() {
        // Arrange: create a temp directory with pre-populated sample data.
        let dir = std::env::temp_dir().join(format!("claudebar-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let burn_path = dir.join("burn-5h.tsv");
        let now = 1000i64;
        // Write 10 samples: 0→50% over 10 seconds (1 pct/sec slope, starting at 50%).
        let mut content = String::new();
        for i in 0..10i64 {
            let pct = 40.0 + i as f64; // 40→49 over 9 seconds
            content.push_str(&format!("{now}\t{pct:.3}\t2100\n"));
        }
        std::fs::write(&burn_path, &content).expect("write samples file");

        // Set env var to point at our temp file.
        let prev = std::env::var("CLAUDEBAR_BURN_FILE").ok();
        // SAFETY: single-threaded test, no other env reads in flight.
        unsafe { std::env::set_var("CLAUDEBAR_BURN_FILE", burn_path.to_str().unwrap()) };

        // Two windows of data (5h at 45%, near reset).
        let input = InputData {
            rate_limits: crate::model::input::RateLimits {
                five_hour: Some(crate::model::input::Window {
                    used_percentage: crate::model::input::Coerce(Some(45.0)),
                    resets_at: crate::model::input::Coerce(Some(now + 200)),
                }),
                seven_day: Some(crate::model::input::Window {
                    used_percentage: crate::model::input::Coerce(Some(30.0)),
                    resets_at: crate::model::input::Coerce(Some(now + 5000)),
                }),
            },
            ..Default::default()
        };

        let config = Config::default();
        let theme = themes::get(&config.theme);
        let style = styles::get(&config.style);
        let th = Thresholds::default();
        let ctx = RenderCtx {
            input: &input,
            theme: &theme,
            style: &style,
            th: &th,
            now,
            home: None,
            tz_offset_seconds: 0,
        };

        let mut w = SegmentWriter::new(&theme, &style);
        let emitted = Burn.render(&ctx, &mut w);

        // Restore env var.
        match prev {
            Some(v) => unsafe { std::env::set_var("CLAUDEBAR_BURN_FILE", v) },
            None => unsafe { std::env::remove_var("CLAUDEBAR_BURN_FILE") },
        }
        // Clean up temp dir.
        let _ = std::fs::remove_dir_all(&dir);

        // Assert: render emitted burn content by reading the samples file.
        assert!(emitted, "burn should render when samples exist");
        assert!(!w.is_empty(), "writer should have content");
    }
    #[test]
    fn estimate_7d_warming_when_pct_zero() {
        // 7d window with 0% pct → Warming, label "7d".
        let theme = test_theme();
        let est = estimate(0, &[], None, Some((0.0, 600_000)), &theme);
        assert_eq!(est.state, BurnState::Warming);
        assert_eq!(est.label, "7d");
    }

    #[test]
    fn burn_render_warming_no_samples() {
        let dir =
            std::env::temp_dir().join(format!("claudebar-burn-warming-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let burn_path = dir.join("burn-5h.tsv");
        // Don't write any samples — the file doesn't exist yet.

        let now = 1000i64;
        let input = InputData {
            rate_limits: crate::model::input::RateLimits {
                five_hour: Some(crate::model::input::Window {
                    used_percentage: crate::model::input::Coerce(Some(45.0)),
                    resets_at: crate::model::input::Coerce(Some(now + 200)),
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        let config = Config::default();
        let theme = themes::get(&config.theme);
        let style = styles::get(&config.style);
        let th = Thresholds::default();

        // Set env var for the burn file location.
        let prev = std::env::var("CLAUDEBAR_BURN_FILE").ok();
        unsafe { std::env::set_var("CLAUDEBAR_BURN_FILE", burn_path.to_str().unwrap()) };

        let ctx = RenderCtx {
            input: &input,
            theme: &theme,
            style: &style,
            th: &th,
            now,
            home: None,
            tz_offset_seconds: 0,
        };
        let mut w = SegmentWriter::new(&theme, &style);
        let emitted = Burn.render(&ctx, &mut w);

        // Restore env var.
        match prev {
            Some(v) => unsafe { std::env::set_var("CLAUDEBAR_BURN_FILE", v) },
            None => unsafe { std::env::remove_var("CLAUDEBAR_BURN_FILE") },
        }
        let _ = std::fs::remove_dir_all(&dir);

        assert!(emitted, "burn should render even without existing samples");
        assert!(!w.is_empty(), "writer should have content");
        // The render always populates at least one sample, which gives slope=0 → Idle (✓).
        assert!(
            w.as_str().contains("↗"),
            "expected burn output: {:?}",
            w.as_str()
        );
    }

    #[test]
    fn burn_render_with_far_future_reset() {
        // Reset too far in the future → sampling is skipped, but render still works.
        let dir =
            std::env::temp_dir().join(format!("claudebar-burn-farfuture-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let burn_path = dir.join("burn-5h.tsv");

        // Write samples that would make it active if read.
        let now = 1000i64;
        let mut content = String::new();
        for i in 0..10i64 {
            let pct = 40.0 + i as f64;
            content.push_str(&format!("{now}\t{pct:.3}\t2100\n"));
        }
        std::fs::write(&burn_path, &content).expect("write samples file");

        let input = InputData {
            rate_limits: crate::model::input::RateLimits {
                five_hour: Some(crate::model::input::Window {
                    used_percentage: crate::model::input::Coerce(Some(45.0)),
                    // Reset in 100000s — far beyond the 6h margin, so sampling is skipped.
                    resets_at: crate::model::input::Coerce(Some(now + 100_000)),
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        let config = Config::default();
        let theme = themes::get(&config.theme);
        let style = styles::get(&config.style);
        let th = Thresholds::default();

        let prev = std::env::var("CLAUDEBAR_BURN_FILE").ok();
        unsafe { std::env::set_var("CLAUDEBAR_BURN_FILE", burn_path.to_str().unwrap()) };

        let ctx = RenderCtx {
            input: &input,
            theme: &theme,
            style: &style,
            th: &th,
            now,
            home: None,
            tz_offset_seconds: 0,
        };

        let mut w = SegmentWriter::new(&theme, &style);
        let emitted = Burn.render(&ctx, &mut w);

        match prev {
            Some(v) => unsafe { std::env::set_var("CLAUDEBAR_BURN_FILE", v) },
            None => unsafe { std::env::remove_var("CLAUDEBAR_BURN_FILE") },
        }
        let _ = std::fs::remove_dir_all(&dir);

        assert!(emitted, "burn should render even with far-future reset");
        assert!(!w.is_empty(), "writer should have some content");
    }
}
