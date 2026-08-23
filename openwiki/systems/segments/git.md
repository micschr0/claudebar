---
type: "Reference"
title: "Git segment"
openwiki_generated: true
---

# Git segment

Shows branch state: branch name, ahead/behind counts, modified/untracked,
stash count, and no-commits-yet / detached states.

- Gate: absolute-path gate on `cwd`; spawns `git -C cwd -c gc.auto=0 status
  --branch --porcelain --no-optional-locks`.
- **Non-empty-stdout gate**, not exit status: empty stdout → `false` (skip).
  Git unavailable → `git_unavailable_returns_false`.
- Parsing: `parse_status` table (branch / ahead / behind / modified /
  untracked / detached / no-commits-yet).
- Stash: `git rev-list --walk-reflogs --count refs/stash` (cheap, separate
  subprocess).
- Output control-stripped; colors via theme git slots.
- Source: `src/segment/git.rs`.
- Tests: `parse_status` table tests, `git_unavailable_returns_false`, golden
  dirty/clean fixtures.