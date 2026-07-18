//! Git segment.
//!
//! Contract (matches the bash script's git segment):
//! - Only run when `ctx.input.cwd` is a non-empty **absolute** path (starts with
//!   `/`). Otherwise emit nothing, return `false`.
//! - Run TWO git commands, both scoped to `cwd`:
//!   1. `git -C <cwd> -c gc.auto=0 status --branch --porcelain --no-optional-locks` (suppress stderr).
//!      If it fails or output is empty → return `false`.
//!   2. `git -C <cwd> rev-list --walk-reflogs --count refs/stash` (with
//!      `GIT_OPTIONAL_LOCKS=0`), only once (1) has confirmed a branch — for the
//!      stash flag. Skipped for non-repos and detached HEAD.
//! - Parse the `## ` branch line:
//!   - `## No commits yet on <branch>` → branch = `<branch>`.
//!   - `## HEAD (no branch)` → no branch (detached) → return `false`.
//!   - `## <branch>...<upstream> [ahead N, behind M]` → branch before `...`,
//!     ahead/behind from the bracketed counts (each optional).
//! - Count the remaining porcelain lines: `?? ` prefixed → untracked (`n_new`);
//!   any other non-`## ` non-empty line → modified (`n_mod`).
//! - Strip control bytes from the branch name (host-provided) via
//!   [`crate::sanitize::strip_control`].
//! - Emit (using the writer + style glyphs): branch icon + branch in
//!   `theme.git_branch`; then the stash flag (theme.stash / glyphs.stash)
//!   immediately after the branch name when stash count > 0; then ` ↑N`
//!   (theme.ahead) if ahead>0; ` ↓M` (theme.behind) if behind>0; ` MN`
//!   (theme.modified) if n_mod>0; ` ?N` (theme.untracked) if n_new>0. Glyphs
//!   come from `style.glyphs` (branch/stash/ahead/behind/modified/untracked).
//! - Return `true` once a branch was emitted.

use crate::render::SegmentWriter;
use crate::sanitize::strip_control;
use crate::segment::{RenderCtx, Segment};
use std::process::Command;

/// Parsed result of a `git status --branch --porcelain` run.
struct GitStatus {
    branch: String,
    ahead: u32,
    behind: u32,
    n_mod: u32,
    n_new: u32,
}

/// Parse the output of `git status --branch --porcelain`.
///
/// Returns `None` for detached HEAD (`## HEAD (no branch)`) or when no branch
/// can be determined.
fn parse_status(out: &str) -> Option<GitStatus> {
    let branch_line = out.lines().next()?;

    let mut branch = String::new();
    let mut ahead = 0u32;
    let mut behind = 0u32;

    if let Some(rest) = branch_line.strip_prefix("## No commits yet on ") {
        branch = rest.to_string();
    } else if branch_line.starts_with("## HEAD (no branch)") {
        return None;
    } else if let Some(rest) = branch_line.strip_prefix("## ") {
        // `<branch>...<upstream> [ahead N, behind M]`; branch is the text
        // before the first `...` (or the whole rest if no upstream).
        branch = rest.split("...").next().unwrap_or(rest).to_string();
        ahead = parse_count(branch_line, "ahead ");
        behind = parse_count(branch_line, "behind ");
    }

    let branch = strip_control(&branch);
    if branch.is_empty() {
        return None;
    }

    let mut n_mod = 0u32;
    let mut n_new = 0u32;
    for line in out.lines() {
        if line.starts_with("## ") || line.is_empty() {
            continue;
        }
        if line.starts_with("?? ") {
            n_new += 1;
        } else {
            n_mod += 1;
        }
    }

    Some(GitStatus {
        branch,
        ahead,
        behind,
        n_mod,
        n_new,
    })
}

/// Extract the integer following `key` (e.g. `"ahead "`) in the branch line.
fn parse_count(line: &str, key: &str) -> u32 {
    line.find(key)
        .map(|i| &line[i + key.len()..])
        .map(|rest| {
            // ASCII digits are single-byte, so the count of leading digit bytes
            // is a valid `&str` boundary — parse that subslice with no String.
            let n = rest.bytes().take_while(u8::is_ascii_digit).count();
            rest[..n].parse().unwrap_or(0)
        })
        .unwrap_or(0)
}

/// Parse the stdout of `git rev-list --walk-reflogs --count refs/stash`.
///
/// Returns 0 for empty or unparseable input — never panics.
fn parse_stash_count(out: &str) -> u64 {
    out.trim().parse().unwrap_or(0)
}

/// Count the git stashes in `cwd`. Returns 0 on any spawn or exit failure.
fn stash_count(cwd: &str) -> u64 {
    match Command::new("git")
        .args([
            "-C",
            cwd,
            "rev-list",
            "--walk-reflogs",
            "--count",
            "refs/stash",
        ])
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
    {
        Ok(o) if o.status.success() => parse_stash_count(&String::from_utf8_lossy(&o.stdout)),
        _ => 0,
    }
}

pub struct Git;

impl Segment for Git {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let cwd = match ctx.input.cwd.as_deref() {
            Some(c) if c.starts_with('/') => c,
            _ => return false,
        };

        // Like the bash reference (`git_out=$(…); [ -n "$git_out" ]`), gate on
        // non-empty stdout only — never on exit status. git can print a valid
        // `## ` line while exiting non-zero, and bash renders it.
        let git_out = match Command::new("git")
            .args([
                "--no-optional-locks",
                "-C",
                cwd,
                "-c",
                "gc.auto=0",
                "status",
                "--branch",
                "--porcelain",
            ])
            .stderr(std::process::Stdio::null())
            .output()
        {
            Ok(o) => String::from_utf8_lossy(&o.stdout).into_owned(),
            Err(_) => return false,
        };
        if git_out.is_empty() {
            return false;
        }

        let status = match parse_status(&git_out) {
            Some(s) => s,
            None => return false,
        };

        let theme = ctx.theme;
        let glyphs = ctx.style.glyphs;

        out.icon(glyphs.branch);
        out.colored(theme.git_branch, &status.branch);
        let stashes = stash_count(cwd);
        if stashes > 0 {
            out.colored_fmt(theme.stash, format_args!(" {}{}", glyphs.stash, stashes));
        }
        if status.ahead > 0 {
            out.colored_fmt(
                theme.ahead,
                format_args!(" {}{}", glyphs.ahead, status.ahead),
            );
        }
        if status.behind > 0 {
            out.colored_fmt(
                theme.behind,
                format_args!(" {}{}", glyphs.behind, status.behind),
            );
        }
        if status.n_mod > 0 {
            out.colored_fmt(
                theme.modified,
                format_args!(" {}{}", glyphs.modified, status.n_mod),
            );
        }
        if status.n_new > 0 {
            out.colored_fmt(
                theme.untracked,
                format_args!(" {}{}", glyphs.untracked, status.n_new),
            );
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Config;
    use crate::model::Thresholds;
    use crate::model::input::InputData;
    use crate::{styles, themes};

    #[test]
    fn clean_branch() {
        let s = parse_status("## main...origin/main\n").unwrap();
        assert_eq!(s.branch, "main");
        assert_eq!((s.ahead, s.behind, s.n_mod, s.n_new), (0, 0, 0, 0));
    }

    #[test]
    fn no_upstream_branch() {
        let s = parse_status("## feature/foo\n").unwrap();
        assert_eq!(s.branch, "feature/foo");
        assert_eq!((s.ahead, s.behind), (0, 0));
    }

    #[test]
    fn ahead_and_behind() {
        let s = parse_status("## main...origin/main [ahead 3, behind 1]\n").unwrap();
        assert_eq!(s.branch, "main");
        assert_eq!((s.ahead, s.behind), (3, 1));
    }

    #[test]
    fn ahead_only() {
        let s = parse_status("## main...origin/main [ahead 2]\n").unwrap();
        assert_eq!((s.ahead, s.behind), (2, 0));
    }

    #[test]
    fn behind_only() {
        let s = parse_status("## main...origin/main [behind 5]\n").unwrap();
        assert_eq!((s.ahead, s.behind), (0, 5));
    }

    #[test]
    fn no_commits_yet() {
        let s = parse_status("## No commits yet on main\n").unwrap();
        assert_eq!(s.branch, "main");
    }

    #[test]
    fn detached_head_is_none() {
        assert!(parse_status("## HEAD (no branch)\n").is_none());
    }

    #[test]
    fn counts_modified_and_untracked() {
        let out = "## main...origin/main\n M src/a.rs\nM  src/b.rs\n?? new.txt\n?? other.txt\n";
        let s = parse_status(out).unwrap();
        assert_eq!(s.n_mod, 2);
        assert_eq!(s.n_new, 2);
    }

    #[test]
    fn strips_control_bytes_from_branch() {
        let s = parse_status("## ma\x1b[31min\n").unwrap();
        assert!(!s.branch.contains('\x1b'), "got: {:?}", s.branch);
    }

    #[test]
    fn empty_output_is_none() {
        assert!(parse_status("").is_none());
    }

    #[test]
    fn parses_stash_count() {
        assert_eq!(parse_stash_count("2\n"), 2);
    }

    #[test]
    fn parses_stash_count_with_padding() {
        assert_eq!(parse_stash_count("  0 \n"), 0);
    }

    #[test]
    fn empty_stash_output_is_zero() {
        assert_eq!(parse_stash_count(""), 0);
    }

    #[test]
    fn garbage_stash_output_is_zero() {
        assert_eq!(parse_stash_count("garbage"), 0);
    }
    #[test]
    fn git_unavailable_returns_false() {
        // Mock an environment where git is not available.
        let original_path = std::env::var("PATH").ok();
        // SAFETY: using raw env manipulation is the only way to mock PATH in tests.
        unsafe { std::env::set_var("PATH", "") };
        // Also unset which(1) lookup which might short-circuit.
        unsafe { std::env::set_var("PATH", "") };

        let input = InputData {
            cwd: Some("/tmp".to_string()),
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
            now: 0,
            home: None,
            tz_offset_seconds: 0,
        };
        let mut out = SegmentWriter::new(&theme, &style);

        // Git should be unavailable in the mocked environment.
        // render() should return false (safe fallback, no panic).
        assert!(!Git.render(&ctx, &mut out));

        // Restore PATH.
        if let Some(p) = original_path {
            unsafe { std::env::set_var("PATH", p) };
        } else {
            unsafe { std::env::remove_var("PATH") };
        }
    }

    #[test]
    fn git_no_cwd_returns_false() {
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
        let mut out = SegmentWriter::new(&theme, &style);
        assert!(!Git.render(&ctx, &mut out));
    }

    #[test]
    fn git_non_absolute_cwd_returns_false() {
        let input = InputData {
            cwd: Some("relative/path".to_string()),
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
            now: 0,
            home: None,
            tz_offset_seconds: 0,
        };
        let mut out = SegmentWriter::new(&theme, &style);
        assert!(!Git.render(&ctx, &mut out));
    }

    #[test]
    fn git_render_clean_status() {
        // Create a temp git repo with one commit.
        let dir = std::env::temp_dir().join(format!("claudebar-git-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("README.md"), "# test").unwrap();

        // Use full /usr/bin/git path to avoid PATH-related concurrency issues.
        let git = "/usr/bin/git";
        let output = std::process::Command::new(git)
            .arg("init")
            .arg(&dir)
            .output()
            .unwrap();
        assert!(output.status.success(), "git init failed: {output:?}");

        let _ = std::process::Command::new(git)
            .args([
                "-C",
                &dir.to_string_lossy(),
                "symbolic-ref",
                "HEAD",
                "refs/heads/main",
            ])
            .output();
        let _ = std::process::Command::new(git)
            .args([
                "-C",
                &dir.to_string_lossy(),
                "config",
                "user.email",
                "test@test.com",
            ])
            .output();
        let _ = std::process::Command::new(git)
            .args(["-C", &dir.to_string_lossy(), "config", "user.name", "Test"])
            .output();

        let output = std::process::Command::new(git)
            .args(["-C", &dir.to_string_lossy(), "add", "."])
            .output()
            .unwrap();
        assert!(output.status.success(), "git add failed: {output:?}");

        let output = std::process::Command::new(git)
            .args(["-C", &dir.to_string_lossy(), "commit", "-m", "initial"])
            .env("GIT_AUTHOR_DATE", "2024-01-01T00:00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-01T00:00:00")
            .output()
            .unwrap();
        assert!(output.status.success(), "git commit failed: {:?}", output);

        let input = InputData {
            cwd: Some(dir.to_string_lossy().to_string()),
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
            now: 0,
            home: None,
            tz_offset_seconds: 0,
        };
        let mut out = SegmentWriter::new(&theme, &style);
        let rendered = Git.render(&ctx, &mut out);
        assert!(rendered, "expected render to return true for clean repo");
        let s = out.as_str();
        assert!(s.contains("main"), "expected branch name in output: {s:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn git_parse_status_with_non_git_dir_returns_none() {
        let dir =
            std::env::temp_dir().join(format!("claudebar-git-nonrepo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("readme.txt"), "hello").unwrap();

        let input = InputData {
            cwd: Some(dir.to_string_lossy().to_string()),
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
            now: 0,
            home: None,
            tz_offset_seconds: 0,
        };
        let mut out = SegmentWriter::new(&theme, &style);
        assert!(!Git.render(&ctx, &mut out), "expected false for non-repo");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
