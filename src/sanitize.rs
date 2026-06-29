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
/// decimal and carry (`9.96k` → `10.0k`).
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
    fn reset_formatting() {
        let now = 1_000_000;
        assert_eq!(fmt_reset(now + 90_000, now).as_deref(), Some("1d1h"));
        assert_eq!(fmt_reset(now + 8000, now).as_deref(), Some("2h13m"));
        assert_eq!(fmt_reset(now + 130, now).as_deref(), Some("2m10s"));
        assert_eq!(fmt_reset(now + 5, now).as_deref(), Some("5s"));
        assert_eq!(fmt_reset(now - 5, now), None); // past
        assert_eq!(fmt_reset(0, now), None); // absent
    }

    #[test]
    fn boundary_reset_cases() {
        // CR-16: exact diff-0 and diff-1 boundaries around the `diff <= 0` guard.
        let now = 1_000_000;
        assert_eq!(fmt_reset(now, now), None); // diff 0 → None
        assert_eq!(fmt_reset(now + 1, now).as_deref(), Some("1s")); // diff 1 → "1s"
    }
}
