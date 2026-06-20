# Project Research Summary

**Project:** claudebar
**Domain:** Rust CLI tool — multi-arch release pipeline, documentation polish
**Researched:** 2026-06-20
**Confidence:** MEDIUM

## Executive Summary

claudebar is a feature-complete Rust statusline renderer for Claude Code with a TUI configurator, 16 themes, and 5 styles. The product itself is done; this milestone is about shipping it properly — publishing a polished README, committing 10 pending theme files, building a GitHub Actions release pipeline that produces prebuilt binaries for four targets (x86_64/aarch64 × Linux/macOS), and updating install.sh to consume those binaries. The work is well-understood: Rust CI/CD on GitHub Actions has a clear community-standard stack, and the README improvements follow established patterns from comparable Rust CLI tools.

The recommended approach is a two-workflow GitHub Actions setup (ci.yml for push/PR, release.yml for tags) using the taiki-e action pair for binary packaging, houseabsolute/actions-rust-cross for cross-compilation, and softprops/action-gh-release for publishing. Linux ARM is handled by cross on an ubuntu runner; macOS targets require native runners pinned to macos-13 (Intel) and macos-14 (ARM) — never macos-latest, which changed silently in 2024. The install script needs OS/arch detection added and should prefer prebuilt binary download over cargo build once the release pipeline exists.

The main risks are concrete and known: macos-latest producing wrong-arch binaries, glibc version mismatch on older Linux, and binary archive naming inconsistency breaking the install script. All three are straightforward to prevent with the documented mitigations. The work has no significant unknowns — every component has prior art in production Rust CLI tools.

## Key Findings

### Recommended Stack

Two separate workflow files with distinct triggers prevent slow cross-compilation from running on every PR. The taiki-e action pair (create-gh-release-action + upload-rust-binary-action) handles the complete build/archive/checksum/upload cycle with minimal configuration. houseabsolute/actions-rust-cross auto-selects native cargo for native targets and Docker-based cross for ARM Linux — no manual selection logic needed. Swatinem/rust-cache@v2 is the community standard for Rust dependency caching.

**Core technologies:**
- `dtolnay/rust-toolchain@stable`: Rust toolchain setup — replaces deprecated actions-rs/toolchain
- `Swatinem/rust-cache@v2`: Cargo dependency caching — drop-in, handles all cache locations correctly
- `houseabsolute/actions-rust-cross@v0`: Cross-compilation wrapper — auto-selects native vs cross per target
- `taiki-e/upload-rust-binary-action@v1`: Binary packaging + checksums — produces claudebar-{target}.tar.gz + SHA256
- `softprops/action-gh-release@v2`: GitHub Release creation — generate_release_notes: true is zero-config for single-maintainer projects
- Animated SVG (existing gen_screenshots.py --svg): README hero — Docker-free, renders inline on GitHub, scales at any DPI

### Expected Features

**Must have (table stakes):**
- Hero animated SVG embedded at top of README — first impression; file already exists at screenshots/animated.svg but is not yet embedded
- One-command install with prebuilt binary path — curl | bash with OS/arch detection and SHA256 verification
- Prerequisites listed prominently — Nerd Font + git requirement must appear before the install command
- Troubleshooting as its own ## section — Nerd Font glyph boxes are the #1 install failure; must be linkable
- All 10 new themes visible in README — themes listed but not shown; theme gallery or at minimum updated theme list

**Should have (competitive):**
- Animated SVG cycling all 4 states (calm/warn/crit/overlimit) as the README hero — unique vs static screenshots
- TUI configurator screenshot — key differentiator vs bash fallback; no screenshot exists yet
- Segment behavior table — documents conditional appearance (git drops outside repo, effort drops for some models)
- --no-default-features build tested in CI — prevents feature-guard regressions

**Defer (v2+):**
- Theme gallery with one strip per theme — nice but not essential; requires Docker prereqs
- Homebrew tap, AUR package — post-release; out of scope per PROJECT.md
- VHS integration — SVG approach is strictly better for claudebar's use case

### Architecture Approach

Two workflow files with a clear dependency graph: ci.yml (push/PR) runs check, lint, test in parallel then waits for all to pass; release.yml (tag push) runs a 4-target build matrix in parallel then feeds all artifacts to a single release job. Permissions are scoped to contents: write on the release job only. Artifact naming is claudebar-{target}.tar.gz (version in the GitHub Release tag, not the filename), matching the taiki-e default and consistent with ripgrep, bat, and fd.

**Major components:**
1. `ci.yml` — check + lint + test on every push/PR; runs on ubuntu-latest; tests on both ubuntu and macOS to catch platform differences
2. `release.yml` — 4-target matrix build (parallel) feeding a single release job; needs: [build] prevents partial releases
3. Updated `install.sh` — priority order: prebuilt binary download → cargo build → bash fallback; includes uname -s/m detection, SHA256 verification, existing install check
4. README restructure — hero SVG → requirements → install → TUI screenshot → themes → segments table → config → subcommands → troubleshooting

### Critical Pitfalls

1. **macos-latest silently changed architecture** — Pin macos-13 for x86_64 and macos-14 for aarch64 in the release matrix. Never use macos-latest in release workflows.
2. **Glibc version mismatch on older Linux** — Ubuntu 22.04 runner produces glibc 2.35 binaries; users on older Linux will get "GLIBC_2.33 not found". Consider musl targets or document the glibc requirement.
3. **strip = true does not work for cross-compiled ARM Linux binaries** — Add explicit post-build step with aarch64-linux-gnu-strip, or verify taiki-e/upload-rust-binary-action handles this automatically.
4. **uname -m returns arm64 on macOS, aarch64 on Linux** — Normalize in install.sh with a case block before constructing the asset filename.
5. **Binary archive naming inconsistency** — Decide naming convention before writing both the release workflow and install script simultaneously: claudebar-{target}.tar.gz (taiki-e default, version in Release tag only).

## Implications for Roadmap

### Phase 1: Commit pending themes
**Rationale:** 10 theme files are untracked in git (ayu_mirage, cobalt2, everforest_dark, github_dark, kanagawa_wave, moonfly, night_owl, one_dark, solarized_dark, sonokai) alongside src/themes/mod.rs and src/tui/app.rs changes. These must be committed before CI can run a clean build. Strict prerequisite for everything else.
**Delivers:** Clean working tree; all 16 themes available; CI can run
**Addresses:** "Commit the 10 pending new theme files" requirement from PROJECT.md
**Avoids:** CI build failures from untracked/unstaged changes; README listing themes not in the binary

### Phase 2: GitHub Actions CI workflow
**Rationale:** CI must exist before the release pipeline — it validates every PR and is the foundation the release workflow builds on. Well-documented patterns, no unknowns.
**Delivers:** .github/workflows/ci.yml — check, lint (clippy + shellcheck), test (ubuntu + macOS), --no-default-features build check
**Uses:** dtolnay/rust-toolchain, Swatinem/rust-cache, actions/checkout
**Avoids:** Pitfall 6 (--no-default-features regressions), Pitfall 13 (insta snapshot CI handling)

### Phase 3: GitHub Actions release pipeline
**Rationale:** Release pipeline depends on CI being green. Once CI is established, the release workflow adds the matrix build and artifact upload. Unblocks prebuilt binary installation.
**Delivers:** .github/workflows/release.yml — 4-target matrix, taiki-e upload action, softprops release creation with auto-generated notes, SHA256SUMS.txt
**Uses:** houseabsolute/actions-rust-cross, taiki-e/upload-rust-binary-action, softprops/action-gh-release
**Avoids:** Pitfall 1 (pin macos-13/14), Pitfall 3 (strip ARM Linux binary), Pitfall 8 (contents: write permission)

### Phase 4: Update install.sh
**Rationale:** install.sh improvements depend on the release pipeline existing so there are actual assets to download. The binary download path cannot be tested until Phase 3 completes and a release tag is pushed.
**Delivers:** Updated install.sh — prebuilt binary download path with OS/arch detection, SHA256 verification, existing install check, curl→wget fallback
**Avoids:** Pitfall 7 (arm64/aarch64 normalization), Pitfall 12 (archive naming consistency)

### Phase 5: README and documentation polish
**Rationale:** README update comes last because it references the complete theme set (Phase 1), the real binary version (Phase 3), and can embed the actual install command (Phase 4). Animated SVG already exists; TUI screenshot is new work.
**Delivers:** Restructured README — animated SVG hero, updated theme/segment/subcommand tables, TUI configurator screenshot, standalone troubleshooting section, updated install command
**Addresses:** All documentation requirements from PROJECT.md

### Phase Ordering Rationale

- Phase 1 is a strict prerequisite for CI — untracked files prevent a clean build
- Phase 2 (CI) before Phase 3 (release) — CI validates release build correctness; matrix jobs are expensive to debug without prior CI baseline
- Phase 4 (install.sh) after Phase 3 — cannot test binary download without real release assets
- Phase 5 (README) last — references the complete shipped product; updating README before release creates stale content

### Research Flags

Phases with standard patterns (skip research-phase):
- **Phase 1:** Straightforward git commit; no research needed
- **Phase 2:** CI patterns are well-documented and stable; community templates exist
- **Phase 3:** taiki-e/cross-rs/softprops combination is the established Rust ecosystem standard
- **Phase 4:** Shell scripting; patterns well-established from grype/code-server precedent
- **Phase 5:** README restructuring; gen_screenshots.py --svg already works; no technical unknowns

No phases require --research-phase invocation.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | MEDIUM | Actions versions cross-validated across multiple sources; community consensus strong |
| Features | MEDIUM | Direct inspection of existing files + comparison to bat, ripgrep, eza README patterns |
| Architecture | MEDIUM | Cross-checked across multiple blog posts and GitHub examples; job dependency patterns stable |
| Pitfalls | LOW-MEDIUM | Community sources; macos-latest change is documented fact; strip cross-compile issue needs verification |

**Overall confidence:** MEDIUM

### Gaps to Address

- **glibc vs musl**: Research recommends musl for broader Linux compatibility but PROJECT.md specifies -gnu targets. Confirm during Phase 3 planning whether to switch to musl or document the glibc >= 2.35 requirement.
- **TUI screenshot approach**: FEATURES.md recommends a --tui mode in gen_screenshots.py. During Phase 5 planning, decide whether to build this HTML approximation or use a static PNG taken manually.
- **cross-rs strip behavior**: Pitfall 4 describes a strip issue with cross-compiled ARM binaries; verify whether taiki-e/upload-rust-binary-action already handles this before adding a manual strip step.

## Sources

### Secondary (MEDIUM confidence)
- https://github.com/houseabsolute/actions-rust-cross — cross-compilation wrapper behavior
- https://github.com/taiki-e/upload-rust-binary-action — artifact packaging and checksum generation
- https://github.com/softprops/action-gh-release — release creation patterns
- https://github.com/dtolnay/rust-toolchain — modern Rust toolchain action
- https://ahmedjama.com/blog/2025/12/cross-platform-rust-pipeline-github-actions/ — Dec 2025 cross-compilation guide
- https://reemus.dev/tldr/rust-cross-compilation-github-actions — Rust GHA cross-compilation patterns
- charmbracelet/vhs GitHub page — confirmed VHS does NOT support SVG output
- grype install.sh — arch detection patterns

### Tertiary (LOW confidence)
- https://kobzol.github.io/rust/ci/2021/05/07/building-rust-binaries-in-ci-that-work-with-older-glibc.html — glibc pitfall documentation
- https://github.com/axodotdev/cargo-dist/issues/1378 — reason to avoid cargo-dist for aarch64-gnu

---
*Research completed: 2026-06-20*
*Ready for roadmap: yes*
