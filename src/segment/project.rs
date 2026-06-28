//! Project segment — repo-root name, stable across worktrees.
//!
//! Uses `git rev-parse --path-format=absolute --git-common-dir` to find the
//! main repo root (the one shared by all linked worktrees), falling back to
//! `--show-toplevel`. Hides outside a git repo. Falls back to the directory
//! segment when `directory` is not already shown.

use crate::render::SegmentWriter;
use crate::sanitize;
use crate::segment::{RenderCtx, Segment};
use std::path::Path;
use std::process::Command;

pub struct Project;

impl Segment for Project {
    fn render(&self, ctx: &RenderCtx, out: &mut SegmentWriter) -> bool {
        let cwd = match ctx.input.cwd.as_deref() {
            Some(c) if c.starts_with('/') => c,
            _ => return false,
        };

        let root = project_root(cwd);
        let name = match root {
            Some(ref r) => r,
            None => return false,
        };

        let mut display = name.clone();
        if ctx.th.name_max > 0 && display.len() > ctx.th.name_max as usize {
            display = truncate_middle(&display, ctx.th.name_max as usize);
        }

        out.colored_with(ctx.theme.project, |w| {
            w.icon(ctx.style.glyphs.project);
            w.raw(" ");
            w.raw(&sanitize::strip_control(&display));
            w.raw(" ");
        });
        true
    }
}

fn project_root(cwd: &str) -> Option<String> {
    let common_dir = Command::new("git")
        .args([
            "-C",
            cwd,
            "rev-parse",
            "--path-format=absolute",
            "--git-common-dir",
        ])
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .ok()?;
    let dir = if common_dir.status.success() {
        String::from_utf8_lossy(&common_dir.stdout).into_owned()
    } else {
        let top_level = Command::new("git")
            .args(["-C", cwd, "rev-parse", "--show-toplevel"])
            .env("GIT_OPTIONAL_LOCKS", "0")
            .output()
            .ok()?;
        if !top_level.status.success() {
            return None;
        }
        String::from_utf8_lossy(&top_level.stdout).into_owned()
    };
    let dir = dir.trim();
    if dir.is_empty() {
        return None;
    }
    let path = Path::new(dir);
    // Strip trailing /.git to get the repo root name
    let root = if path.ends_with(".git") {
        path.parent()?
    } else {
        path
    };
    root.file_name()?.to_str().map(String::from)
}

/// Truncate a string to `max` visible chars by keeping head and tail,
/// inserting `…` in the middle. `max` must be ≥ 3.
fn truncate_middle(s: &str, max: usize) -> String {
    if s.len() <= max || max < 3 {
        return s.to_string();
    }
    let head = (max - 1) / 2;
    let tail = max - 1 - head;
    let start = s.len() - tail;
    format!("{}…{}", &s[..head], &s[start..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_noop_when_short() {
        assert_eq!(truncate_middle("abc", 5), "abc");
    }

    #[test]
    fn truncate_long_name() {
        let result = truncate_middle("very-long-project-name", 10);
        assert_eq!(result.chars().count(), 10);
        assert!(result.contains('…'));
    }
}
