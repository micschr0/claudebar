# Codebase Concerns

**Analysis Date:** 2026-06-20

## Tech Debt

**Swatch cache tied to field order and NAMES array order:**
- Issue: `swatch_cache` in `src/tui/app.rs:96` indexes `themes::NAMES` by position and hard-codes a `[u8; 5]` slot layout (`[separator, dir, git_branch, bar_ok, bar_crit]`). Any reordering of `NAMES` or a change to which palette fields are cached silently produces wrong colors in the TUI theme list.
- Files: `src/tui/app.rs:96`, `src/tui/ui.rs:363`
- Impact: Adding or reordering themes produces incorrect swatch colors with no compile-time or runtime error.
- Fix approach: Key the cache by theme name (HashMap) rather than position; derive slot indices from named theme fields to make the mapping explicit.

**`HOME` environment variable read per render call:**
- Issue: `render_line()` in `src/render/mod.rs:21` calls `std::env::var("HOME")` on every invocation. Claude Code invokes the hook frequently; a small but unnecessary syscall on every render.
- Files: `src/render/mod.rs:21`
- Impact: Negligible in isolation but signals the right hook is `render_with()` (which already accepts `home`) — the public `render_line()` wrapper could accept `home` instead.
- Fix approach: Pass `home` down from the CLI entry point (`run_render`) rather than reading env inside the library.

**Unstaged new themes not yet registered in `claudebar list`:**
- Issue: 10 new theme files (`ayu_mirage.rs`, `cobalt2.rs`, `everforest_dark.rs`, etc.) exist as untracked files. `src/themes/mod.rs` has already been updated to declare and route them, but they are not committed. The binary currently compiles with them included via the working-tree change, but a clean checkout from HEAD would miss them.
- Files: `src/themes/mod.rs` (modified), `src/themes/ayu_mirage.rs` … `src/themes/sonokai.rs` (untracked)
- Impact: Theme names listed in `claudebar list` would resolve to `tokyo-night` fallback on a HEAD build; developers pulling the branch get a broken build until files are staged.
- Fix approach: Commit the new theme files together with the `mod.rs` registration change.

**`src/tui/app.rs` modified but not committed (TUI changes pending):**
- Issue: `src/tui/app.rs` has uncommitted changes alongside the theme additions. The scope of the change is unclear without a diff review.
- Files: `src/tui/app.rs`
- Impact: Risk of lost context or accidental partial commit.
- Fix approach: Review and commit as part of the theme-addition batch.

**No timeout on the `git status` subprocess:**
- Issue: `src/segment/git.rs:114` spawns `git -C <cwd> status ...` with `Command::output()`, which blocks indefinitely. In pathological cases (network-mounted filesystems, very large repos, or git hooks that stall), this can freeze the status line render.
- Files: `src/segment/git.rs:114`
- Impact: The Claude Code hook will stall, blocking the shell prompt, with no user feedback.
- Fix approach: Use `Command::spawn()` + a timeout via `std::thread::spawn` + `child.wait_timeout()`, or the `wait-timeout` crate. A 500ms ceiling matches the bash reference behavior.

**`run_migrate` double-checks file existence after `Config::load`:**
- Issue: `src/main.rs:154–163` calls `Config::load(&path)` then immediately checks `path.exists()` — a TOCTOU pattern. If the file is deleted between the two calls, the error message is misleading.
- Files: `src/main.rs:154–163`
- Impact: Low probability; misleading error message in rare race.
- Fix approach: Check `path.exists()` before `Config::load`, or unify the "missing file" case into `Config::load`'s error type.

## Known Bugs

**`abbreviate_path` skips `strip_control` on the final path component:**
- Symptoms: The last component of a `cwd` path with injected control bytes (e.g. `\x1b`) is passed through `strip_control` only at the outer call on line `src/sanitize.rs:53`, but the path assembly loop appends components with `out.push_str(p)` without per-component sanitization. The outer `strip_control` on the assembled string does catch it, so this is not a live injection vector — but the comment in `src/sanitize.rs:7` says "Strip … from host-provided strings" while the path components are host-provided and only sanitized after joining.
- Files: `src/sanitize.rs:32–53`
- Trigger: `cwd` containing control bytes in intermediate components.
- Workaround: Final `strip_control` at line 53 covers the whole result.

**`pr.review_state` unknown values render nothing:**
- Symptoms: Any `review_state` value other than `approved`, `changes_requested`, `commented`, or `pending` is silently ignored (wildcard `_ => {}`). New Claude Code review states added in future API versions will silently drop.
- Files: `src/segment/dev_context.rs:65–71`
- Trigger: New `review_state` enum values from Claude Code.
- Workaround: None; the segment still renders the PR number, just without a state indicator.

## Security Considerations

**`cwd` is passed directly to `git -C <cwd>` subprocess:**
- Risk: `cwd` comes from the JSON hook input controlled by Claude Code's environment. Although `strip_control` removes ANSI escape bytes, it does not prevent shell metacharacter injection because the argument is passed via `Command::args()` (not a shell), which is safe. However, the `starts_with('/')` gate at `src/segment/git.rs:106–108` only requires an absolute path — paths like `/../etc/passwd` are not filtered.
- Files: `src/segment/git.rs:106–130`
- Current mitigation: `Command::args` bypasses shell interpretation; git itself will reject invalid or dangerous paths. The `starts_with('/')` check prevents relative paths.
- Recommendations: Add path canonicalization or at minimum reject paths containing `..` components to prevent git from running in unintended directories.

**Control-byte stripping is incomplete for OSC sequences:**
- Risk: `strip_control` removes `\x1b` but keeps the characters that follow it (e.g., `[31m` becomes literal `[31m`). This neutralizes ANSI SGR sequences but does not prevent injecting OSC-style payloads whose `\x1b` gets stripped while the remainder (`]0;payload\x07`) could still affect some terminals if misinterpreted.
- Files: `src/sanitize.rs:7–11`
- Current mitigation: `\x07` (BEL) is also stripped, which terminates OSC sequences. The combination of stripping both `\x1b` and `\x07` is sufficient for common terminal emulators.
- Recommendations: Add a test explicitly for `\x1b]0;injected\x07` sequences to document the expected output.

## Performance Bottlenecks

**`git status` subprocess on every render:**
- Problem: Every status-line render spawns a `git` subprocess via `Command::output()`.
- Files: `src/segment/git.rs:114`
- Cause: No caching; each render is stateless. Claude Code can invoke the hook many times per second during heavy use.
- Improvement path: Debounce or cache the git result with a file-mtime or timestamp guard. Alternatively, expose a `--no-git` flag or let users disable the git segment via config (already supported via segments config).

**`String::with_capacity(256)` in render path may reallocate for long lines:**
- Problem: `src/render/mod.rs:47` pre-allocates 256 bytes. A full powerline-style line with all segments, Nerd Font glyphs, and ANSI color codes can easily exceed 256 bytes, causing a reallocation.
- Files: `src/render/mod.rs:47`
- Cause: Conservative fixed estimate.
- Improvement path: Raise the hint (512–1024) or profile typical line lengths.

## Fragile Areas

**`src/tui/ui.rs` (1094 lines) — monolithic draw module:**
- Files: `src/tui/ui.rs`
- Why fragile: All TUI drawing logic, including title, list, description, preview, status, hint, and help overlay, lives in one file with 14 functions. Adding a new TUI section or row type requires touching this file, increasing merge conflict risk.
- Safe modification: Each `draw_*` function is isolated; add new functions without modifying existing ones. Changes to `RowItem` variants require updating `render_row` at line 190.
- Test coverage: TUI draw functions have no unit tests; correctness is verified only by visual inspection.

**`swatch_cache` slot magic numbers in `app.rs`:**
- Files: `src/tui/app.rs:96–119`, `src/tui/ui.rs:363`
- Why fragile: The five cache slots are positional (`[u8; 5]`) with meaning documented only in a comment. The UI side accesses specific indices without named constants.
- Safe modification: Do not change the order of fields built in `new()` without updating all consumers in `ui.rs`.
- Test coverage: None.

**`SegmentKind::ALL` canonical order drives `migrate` insertion logic:**
- Files: `src/main.rs:172–195`, `src/model/config.rs` (where `ALL` is defined)
- Why fragile: The migrate subcommand's insertion algorithm depends on `SegmentKind::ALL` being in the "intended" canonical segment order. Adding a new segment to `ALL` at the wrong position will insert it at an unintuitive location in existing user configs.
- Safe modification: New segments must be appended to `ALL` at the end unless their canonical position is intentionally before existing segments.
- Test coverage: No integration test for migrate behavior.

## Test Coverage Gaps

**TUI event handling (`src/tui/mod.rs`) has no tests:**
- What's not tested: The `handle_key()` dispatch loop, TUI save/quit confirmation flows, and terminal setup/teardown.
- Files: `src/tui/mod.rs`
- Risk: Key bindings could silently break; save confirmation could fail to trigger.
- Priority: Medium

**`migrate` subcommand has no integration test:**
- What's not tested: The segment insertion ordering algorithm, edge cases with all segments missing, or configs with unknown segment names.
- Files: `src/main.rs:141–217`
- Risk: Migration logic could insert segments at wrong positions with no detection.
- Priority: Medium

**New theme files lack snapshot coverage:**
- What's not tested: The 10 new themes (`ayu-mirage`, `cobalt2`, etc.) added in the current working tree are not represented in any golden snapshot in `tests/snapshots/`.
- Files: `tests/render_golden.rs`, `tests/snapshots/`
- Risk: A theme color regression would not be caught until a user reports it.
- Priority: Low (themes are purely cosmetic, but parity with the existing golden tests would be easy to add)

**`dev_context` segment has no render snapshot:**
- What's not tested: End-to-end ANSI output of the dev-context segment across all PR states and agent name edge cases.
- Files: `src/segment/dev_context.rs`, `tests/snapshots/render_golden__golden_lines@dev_context.json.snap`
- Risk: Color or layout regressions in dev-context output are not caught by CI.
- Priority: Low (a snapshot exists for the fixture but may not cover all branches)

---

*Concerns audit: 2026-06-20*
