//! Pure helpers shared by segments: injection-hardening and number/time
//! formatting. No I/O, no color — just string transforms.

/// Strip terminal-control bytes (ESC `\x1b`, BEL `\x07`, CR `\r`, LF `\n`) from a
/// host-provided string. This blocks ANSI/OSC escape injection through fields
/// like `cwd`, the git branch name, or the model display name.
#[must_use]
pub fn strip_control(s: &str) -> String {
    s.chars()
        .filter(|&c| c != '\x1b' && c != '\x07' && c != '\r' && c != '\n')
        .collect()
}

/// Fish-style path abbreviation: every component except the last is shortened to
/// its first character (or first two if it begins with `.`); the final component
/// is kept in full. `$HOME` is collapsed to `~` first.
///
/// `/home/me/projects/claude-code-statusline` → `~/p/c/statusline`
#[must_use]
pub fn abbreviate_path(cwd: &str, home: Option<&str>) -> String {
    let rel = match home {
        Some(h) if !h.is_empty() && cwd == h => "~".to_string(),
        Some(h)
            if !h.is_empty()
                && cwd
                    .strip_prefix(h)
                    .is_some_and(|rest| rest.starts_with('/')) =>
        {
            // SAFETY of slice bounds: strip_prefix matched, so `h` is a prefix.
            let mut tilde = String::with_capacity(1 + cwd.len() - h.len());
            tilde.push('~');
            tilde.push_str(&cwd[h.len()..]);
            tilde
        }
        _ => cwd.to_string(),
    };

    let parts: Vec<&str> = rel.split('/').collect();
    if parts.is_empty() {
        return strip_control(&rel);
    }
    let last = parts.len() - 1;
    let mut out = String::with_capacity(rel.len());
    for (i, p) in parts.iter().enumerate() {
        if i == last {
            out.push_str(p);
            break;
        }
        if p.is_empty() {
            // Leading empty component from a root-absolute path → "/".
            out.push('/');
            continue;
        }
        if let Some(rest) = p.strip_prefix('.') {
            out.push('.');
            if let Some(c) = rest.chars().next() {
                out.push(c);
            }
        } else if let Some(c) = p.chars().next() {
            out.push(c);
        }
        out.push('/');
    }
    strip_control(&out)
}

/// Format a token total like the bash version: `< 1000` verbatim, `>= 1000`
/// as `N.Nk`, `>= 1_000_000` as `N.NM`, with round-half-up on the single
/// decimal and carry (`9.96k` → `10.0k`). A carry that crosses the `k`
/// ceiling promotes to `M` (`999_950` → `1.0M`); `M` is the top unit and has
/// no promotion (`999_950_000` → `1000.0M`).
#[must_use]
pub fn fmt_tokens(total: u64) -> String {
    if total >= 1_000_000 {
        fmt_scaled(total, 1_000_000, 'M')
    } else if total >= 1_000 {
        fmt_scaled(total, 1_000, 'k')
    } else {
        total.to_string()
    }
}

fn fmt_scaled(total: u64, unit: u64, suffix: char) -> String {
    let mut int = total / unit;
    let rem = total % unit;
    // One decimal, round half up.
    let mut dec = (rem * 10 + unit / 2) / unit;
    if dec >= 10 {
        int += 1;
        dec = 0;
    }
    if int >= 1000 && suffix == 'k' {
        return fmt_scaled(total, 1_000_000, 'M');
    }
    // "NNNN.Nc" ≤ 8 bytes.
    let mut s = String::with_capacity(8);
    use std::fmt::Write as _;
    write!(s, "{int}.{dec}{suffix}").unwrap();
    s
}

/// Adaptive "time until reset" relative to `now` (both epoch seconds):
/// `Nd Nh` / `Nh Nm` / `Nm Ns` / `Ns`. Returns `None` if the target is missing
/// (`<= 0` here means "no value") or already in the past.
///
/// # Panics
///
/// The internal `write!` to a `String` buffer is infallible and will never panic.
#[must_use]
pub fn fmt_reset(target: i64, now: i64) -> Option<String> {
    if target <= 0 {
        return None;
    }
    let diff = target - now;
    if diff <= 0 {
        return None;
    }
    let d = diff / 86_400;
    let h = (diff % 86_400) / 3_600;
    let m = (diff % 3_600) / 60;
    let s = diff % 60;
    let mut buf = String::with_capacity(8); // "23h59m" ≤ 7 bytes
    use std::fmt::Write as _;
    if d > 0 {
        write!(buf, "{d}d{h}h").unwrap();
    } else if h > 0 {
        write!(buf, "{h}h{m}m").unwrap();
    } else if m > 0 {
        write!(buf, "{m}m{s}s").unwrap();
    } else {
        write!(buf, "{s}s").unwrap();
    }
    Some(buf)
}

/// Compact span for a positive second count: `2d3h` / `1h58m` / `47m` / `42s`.
/// Zero or negative renders `0s`.
///
/// `with_days` folds spans of 24h or more into a day count. The burn-rate ETA
/// passes `true`; the session-duration readout passes `false` and renders a
/// 25-hour session as `25h00m` rather than `1d1h`.
///
/// Distinct from [`fmt_reset`], which shares the arithmetic but not the shape:
/// it leaves the minute unpadded and carries seconds alongside minutes
/// (`2m10s`), because a countdown ticking toward zero wants the finer grain.
#[must_use]
pub fn fmt_span(secs: i64, with_days: bool) -> String {
    if secs <= 0 {
        return String::from("0s");
    }
    let (d, h) = if with_days {
        (secs / 86_400, (secs % 86_400) / 3_600)
    } else {
        (0, secs / 3_600)
    };
    let m = (secs % 3_600) / 60;
    let s = secs % 60;
    let mut buf = String::with_capacity(8); // "1d23h" ≤ 7 bytes
    use std::fmt::Write as _;
    if d > 0 {
        write!(buf, "{d}d{h}h").unwrap();
    } else if h > 0 {
        write!(buf, "{h}h{m:02}m").unwrap();
    } else if m > 0 {
        write!(buf, "{m}m").unwrap();
    } else {
        write!(buf, "{s}s").unwrap();
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_escape_bytes() {
        let dirty = "ab\x1b[31mcd\x07ef\rgh\nij";
        assert_eq!(strip_control(dirty), "ab[31mcdefghij");
    }

    #[test]
    fn abbreviates_home() {
        assert_eq!(
            abbreviate_path("/home/me/projects/claude-code-statusline", Some("/home/me")),
            "~/p/claude-code-statusline"
        );
        assert_eq!(
            abbreviate_path("/home/me/p/c/statusline", Some("/home/me")),
            "~/p/c/statusline"
        );
        assert_eq!(abbreviate_path("/home/me", Some("/home/me")), "~");
    }

    #[test]
    fn abbreviates_absolute_no_home() {
        assert_eq!(abbreviate_path("/var/log/syslog", None), "/v/l/syslog");
    }

    #[test]
    fn abbreviates_dotfiles_keep_two_chars() {
        assert_eq!(
            abbreviate_path("/home/me/.config/statusline", Some("/home/me")),
            "~/.c/statusline"
        );
    }

    #[test]
    fn single_component() {
        assert_eq!(abbreviate_path("tmp", None), "tmp");
    }

    #[test]
    fn token_formatting() {
        assert_eq!(fmt_tokens(0), "0");
        assert_eq!(fmt_tokens(999), "999");
        assert_eq!(fmt_tokens(1000), "1.0k");
        assert_eq!(fmt_tokens(42300), "42.3k");
        assert_eq!(fmt_tokens(9960), "10.0k"); // carry
        assert_eq!(fmt_tokens(1_000_000), "1.0M");
        assert_eq!(fmt_tokens(1_550_000), "1.6M");
    }

    #[test]
    fn token_carry_boundary() {
        assert_eq!(fmt_tokens(999_949), "999.9k"); // below boundary, no promotion
        assert_eq!(fmt_tokens(999_950), "1.0M"); // carry promotes k -> M
        assert_eq!(fmt_tokens(999_999), "1.0M"); // carry promotes k -> M
        assert_eq!(fmt_tokens(9_996), "10.0k"); // in-unit carry still stays k
        assert_eq!(fmt_tokens(999_950_000), "1000.0M"); // M is the top unit, no further promotion
    }

    #[test]
    fn reset_formatting() {
        let now = 1_000_000;
        assert_eq!(fmt_reset(now + 90_000, now).as_deref(), Some("1d1h"));
        assert_eq!(fmt_reset(now + 8000, now).as_deref(), Some("2h13m"));
        assert_eq!(fmt_reset(now + 130, now).as_deref(), Some("2m10s"));
        assert_eq!(fmt_reset(now + 5, now).as_deref(), Some("5s"));
        assert_eq!(fmt_reset(now - 5, now), None); // past
        assert_eq!(fmt_reset(0, now), None); // absent
    }

    /// The three duration formatters this crate used to carry, verbatim, as
    /// reference implementations. `fmt_span` replaced two of them; these prove
    /// the replacement is byte-identical rather than merely plausible.
    mod reference {
        use std::fmt::Write as _;

        /// Former `segment::burn::fmt_eta`.
        pub fn fmt_eta(secs: i64) -> String {
            if secs <= 0 {
                return String::from("0s");
            }
            let days = secs / 86400;
            let h = (secs % 86400) / 3600;
            let m = (secs % 3600) / 60;
            let s = secs % 60;
            let mut buf = String::new();
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

        /// Former `segment::duration::fmt_duration`, in seconds.
        pub fn fmt_duration_secs(total_s: u64) -> String {
            let h = total_s / 3600;
            let m = (total_s % 3600) / 60;
            let s = total_s % 60;
            let mut buf = String::new();
            if h > 0 {
                write!(buf, "{h}h{m:02}m").unwrap();
            } else if m > 0 {
                write!(buf, "{m}m").unwrap();
            } else {
                write!(buf, "{s}s").unwrap();
            }
            buf
        }
    }

    /// Every second from 0 to 10_000_000 (just under 116 days), plus the day and
    /// hour boundaries, must format exactly as the old burn ETA did.
    #[test]
    fn fmt_span_with_days_matches_the_old_eta_formatter() {
        for secs in (0..10_000_000).step_by(97) {
            assert_eq!(
                fmt_span(secs, true),
                reference::fmt_eta(secs),
                "with_days mismatch at {secs}s"
            );
        }
        for secs in [0, 1, 59, 60, 61, 3599, 3600, 3601, 86_399, 86_400, 86_401] {
            assert_eq!(fmt_span(secs, true), reference::fmt_eta(secs), "at {secs}s");
        }
    }

    /// Same for the session-duration readout, which deliberately does *not*
    /// fold hours into days — a 25-hour session stays `25h00m`.
    #[test]
    fn fmt_span_without_days_matches_the_old_duration_formatter() {
        for secs in (1..10_000_000).step_by(97) {
            assert_eq!(
                fmt_span(secs, false),
                reference::fmt_duration_secs(secs as u64),
                "no-days mismatch at {secs}s"
            );
        }
        assert_eq!(fmt_span(90_000, false), "25h00m", "hours must not fold");
        assert_eq!(fmt_span(90_000, true), "1d1h", "unless days are requested");
    }

    #[test]
    fn fmt_span_clamps_non_positive_to_zero_seconds() {
        assert_eq!(fmt_span(0, true), "0s");
        assert_eq!(fmt_span(-1, true), "0s");
        assert_eq!(fmt_span(-100, false), "0s");
    }

    #[test]
    fn boundary_reset_cases() {
        // CR-16: exact diff-0 and diff-1 boundaries around the `diff <= 0` guard.
        let now = 1_000_000;
        assert_eq!(fmt_reset(now, now), None); // diff 0 → None
        assert_eq!(fmt_reset(now + 1, now).as_deref(), Some("1s")); // diff 1 → "1s"
    }
}
