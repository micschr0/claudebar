//! Git segment.
//!
//! Contract (matches the bash script's git segment):
//! - Only run when `ctx.input.cwd` is a non-empty **absolute** path (starts with
//!   `/`). Otherwise emit nothing, return `false`.
//! - Run exactly one git command:
//!   `git -C <cwd> -c gc.auto=0 status --branch --porcelain` (suppress stderr).
//!   If it fails or output is empty → return `false`.
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
//!   `theme.git_branch`; then ` ↑N` (theme.ahead) if ahead>0; ` ↓M`
//!   (theme.behind) if behind>0; ` MN` (theme.modified) if n_mod>0; ` ?N`
//!   (theme.untracked) if n_new>0. Glyphs come from `style.glyphs`
//!   (branch/ahead/behind/modified/untracked).
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
            rest.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
        })
        .and_then(|digits| digits.parse().ok())
        .unwrap_or(0)
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
        if status.ahead > 0 {
            out.colored(theme.ahead, &format!(" {}{}", glyphs.ahead, status.ahead));
        }
        if status.behind > 0 {
            out.colored(
                theme.behind,
                &format!(" {}{}", glyphs.behind, status.behind),
            );
        }
        if status.n_mod > 0 {
            out.colored(
                theme.modified,
                &format!(" {}{}", glyphs.modified, status.n_mod),
            );
        }
        if status.n_new > 0 {
            out.colored(
                theme.untracked,
                &format!(" {}{}", glyphs.untracked, status.n_new),
            );
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
