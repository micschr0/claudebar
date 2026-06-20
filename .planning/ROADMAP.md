# Roadmap: claudebar

## Overview

claudebar is feature-complete; this milestone is about shipping it properly. The 10 pending theme files must land first so CI has a clean build. CI gates the release pipeline, which gates the install script improvements, which lets the README doc the final one-command install flow. Five phases, strict prerequisite ordering.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Commit Pending Themes** - Commit 10 untracked theme files so CI can run a clean build (completed 2026-06-20)
- [x] **Phase 2: GitHub Actions CI** - Add ci.yml running check/lint/test on every push and PR (completed 2026-06-20)
- [x] **Phase 3: Release Pipeline** - Add release.yml producing prebuilt binaries for 4 targets on tag push (completed 2026-06-20)
- [ ] **Phase 4: Install Script** - Rewrite install.sh with OS/arch detection, binary download, and SHA256 verification
- [ ] **Phase 5: Documentation Polish** - Update README to reflect all 16 themes, segments, TUI, and one-command install

## Phase Details

### Phase 1: Commit Pending Themes

**Goal**: All 16 themes are present in the repository and `claudebar list` reports them correctly on a clean checkout
**Mode:** mvp
**Depends on**: Nothing (first phase)
**Requirements**: THEME-01, THEME-02
**Success Criteria** (what must be TRUE):

  1. `git status` shows no untracked files under `src/themes/` or modified `src/tui/app.rs`
  2. `claudebar list` outputs exactly 16 theme names including all 10 new themes
  3. `cargo test` passes on a clean checkout (no missing module references)

**Plans**: 1/1 plans complete

Plans:

- [x] 01-01-PLAN.md — Quality gate (fmt/clippy/test) then atomic commit of 10 theme files + mod.rs + app.rs

### Phase 2: GitHub Actions CI

**Goal**: Every push and PR is validated by a passing CI workflow that enforces formatting, linting, tests, and both feature configurations
**Mode:** mvp
**Depends on**: Phase 1
**Requirements**: CI-01, CI-02, CI-03
**Success Criteria** (what must be TRUE):

  1. A green CI badge is visible on the repository after a passing push
  2. A PR with a clippy warning or fmt violation causes CI to fail
  3. CI tests both the default `tui` build and `--no-default-features` render-only build
  4. `shellcheck statusline-command.sh` runs in CI and a shell error causes failure

**Plans**: 1/1 plans complete

Plans:

- [x] 02-01-PLAN.md — Add --no-default-features CI steps to rust.yml + CI badge to README

### Phase 3: Release Pipeline

**Goal**: Pushing a `v*.*.*` tag automatically produces a GitHub Release with prebuilt binaries for all 4 targets and a SHA256 checksum file
**Mode:** mvp
**Depends on**: Phase 2
**Requirements**: REL-01, REL-02, REL-03, REL-04, REL-05, REL-06, REL-07, REL-08
**Success Criteria** (what must be TRUE):

  1. Pushing `v0.2.0` creates a GitHub Release with generated release notes automatically
  2. The release contains 4 `.tar.gz` archives named `claudebar-{version}-{target}.tar.gz` plus `SHA256SUMS.txt`
  3. macOS binaries are built on pinned runners (macos-13 for x86_64, macos-14 for aarch64) — never macos-latest
  4. Linux ARM binaries are produced via cross-rs and run on an aarch64 Linux host

**Plans**: 1/1 plans complete

Plans:

- [x] 03-01-PLAN.md — Create release.yml: three-job pipeline (create-release → build matrix → finalize) for 4-target binary releases with SHA256SUMS.txt

### Phase 4: Install Script

**Goal**: A new user on any supported OS/arch can install claudebar in one command with automatic fallback at each step
**Mode:** mvp
**Depends on**: Phase 3
**Requirements**: INST-01, INST-02, INST-03, INST-04, INST-05
**Success Criteria** (what must be TRUE):

  1. `bash install.sh` on x86_64 Linux downloads the correct prebuilt binary and verifies its SHA256 checksum
  2. On a machine with no prebuilt match but cargo available, `install.sh` falls back to `cargo install --path .`
  3. On a machine with no cargo, `install.sh` falls back to installing `statusline-command.sh` with a clear message
  4. Each fallback stage prints a descriptive message explaining what it tried and why it is falling back

**Plans**: 1 plan

Plans:
- [ ] 04-01-PLAN.md — Rewrite install.sh: three-tier fallback chain (prebuilt binary + SHA256 → cargo → bash script)

### Phase 5: Documentation Polish

**Goal**: A new user landing on the README understands what claudebar is, installs it in one command, and can configure every current feature
**Mode:** mvp
**Depends on**: Phase 4
**Requirements**: DOC-01, DOC-02, DOC-03, DOC-04, DOC-05, DOC-06, DOC-07
**Success Criteria** (what must be TRUE):

  1. The README hero section displays the animated SVG (existing `screenshots/animated.svg` or regenerated)
  2. The README install section shows a single `curl | bash` one-liner using the updated `install.sh`
  3. All 16 themes and all 5 styles are listed by name in the README
  4. The `dev-context` segment and TUI key bindings are documented, with a TUI screenshot or demo

**Plans**: TBD
**UI hint**: yes

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Commit Pending Themes | 1/1 | Complete   | 2026-06-20 |
| 2. GitHub Actions CI | 1/1 | Complete   | 2026-06-20 |
| 3. Release Pipeline | 1/1 | Complete   | 2026-06-20 |
| 4. Install Script | 0/TBD | Not started | - |
| 5. Documentation Polish | 0/TBD | Not started | - |
