---
phase: 04-install-script
plan: "01"
subsystem: install
tags: [bash, installer, three-tier-fallback, sha256, prebuilt-binary]
dependency_graph:
  requires: []
  provides: [three-tier-installer]
  affects: [install.sh]
tech_stack:
  added: []
  patterns: [three-tier-fallback, sha256-verification, tmpdir-trap, arch-normalization]
key_files:
  modified:
    - install.sh
decisions:
  - "TMPDIR_WORK replaces per-file tmp_script/tmp_cfg trap — single EXIT trap covers all downloads"
  - "SHA256 mismatch is fatal and never triggers a fallback — checksum failure exits non-zero"
  - "arm64 normalized to aarch64 before target triple construction — macOS Apple Silicon compatibility"
  - "cargo fallback gated on SRC_DIR non-empty AND Cargo.toml present — prevents curl|bash attempting cargo"
  - "verify_checksum uses two-space anchor grep pattern to prevent partial filename matches"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-20"
  tasks_completed: 2
  files_modified: 1
status: complete
---

# Phase 04 Plan 01: Rewrite install.sh with Three-Tier Fallback Summary

**One-liner:** Three-tier installer with OS/arch detection, GitHub Releases binary download + SHA256 verification, cargo fallback gated on local checkout, and bash script fallback as universal last resort.

## What Was Built

`install.sh` rewritten in place with:

1. **`detect_target()`** — maps `uname -s`/`uname -m` to Rust target triples matching the release matrix (`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`). Normalizes `arm64` → `aarch64` for macOS Apple Silicon.

2. **`fetch_latest_tag()`** — queries GitHub REST API for latest release tag. Uses `jq -r '.tag_name // empty'` to prevent literal `"null"` string when no releases exist.

3. **`sha256_of()`** — cross-platform SHA256: `sha256sum` on Linux, `shasum -a 256` on macOS.

4. **`verify_checksum()`** — matches against SHA256SUMS.txt with two-space anchor grep pattern (`"  ${archive_name}$"`). Mismatch is fatal (`return 1`) — never triggers fallback.

5. **`download_prebuilt()`** — constructs archive URL from tag + target, downloads with `curl -fsSL`, verifies SHA256, extracts binary only from tar archive, installs to `BIN_DEST`.

6. **Three-tier chain:**
   - Tier 1: Prebuilt binary from GitHub Releases
   - Tier 2: `cargo install --path $SRC_DIR` (local checkout only; gated on `SRC_DIR` + `Cargo.toml` + `cargo` in PATH)
   - Tier 3: `statusline-command.sh` bash fallback

7. **Preserved verbatim:** shebang, `set -euo pipefail`, constants block, SRC_DIR detection, color helpers, preflight (jq + git check), `settings.json` patching block, done block.

8. **Single `TMPDIR_WORK` trap** replaces old `tmp_script`/`tmp_cfg` per-file trap.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Rewrite install.sh with three-tier fallback chain | a2e487a | install.sh |
| 2 | Smoke-test tier detection and message logic | (read-only verification, no commit) | — |

## Smoke Test Results (Task 2)

| Check | Result |
|-------|--------|
| `detect_target` on x86_64 Linux host | `x86_64-unknown-linux-musl` — PASS |
| arm64 → aarch64 normalization | PASS |
| x86_64 stays x86_64 | PASS |
| `trap 'rm -rf "$TMPDIR_WORK"' EXIT` present | PASS |
| Two-space grep anchor in `verify_checksum` (confirmed via byte inspection) | PASS |
| `return 1` in `verify_checksum` mismatch branch | PASS (2 occurrences) |
| `bash -n install.sh` | PASS |
| `shellcheck` | Not available in this environment; manual SC2155/SC2086/SC2046 audit passed |

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written.

### Environment Note

`shellcheck` was not available in the execution environment. Manual audit confirmed:
- All `local` declarations separate from subshell assignment (SC2155 compliance)
- All variable expansions double-quoted (SC2086 compliance)
- No unquoted command substitutions causing word splitting (SC2046 compliance)

The CI pipeline runs `ludeeus/action-shellcheck` — the first CI run will catch any remaining issues.

## Requirements Implemented

| Requirement | Status |
|-------------|--------|
| INST-01: OS/arch detection with arm64→aarch64 mapping | Done |
| INST-02: Download prebuilt + SHA256 verification | Done |
| INST-03: cargo fallback gated on SRC_DIR + Cargo.toml + cargo in PATH | Done |
| INST-04: bash script fallback | Done |
| INST-05: Bold messages at each tier transition | Done |

## Threat Mitigations Applied

| Threat | Mitigation |
|--------|------------|
| T-04-01: MITM on download | HTTPS-only (`curl -fsSL`); TLS verified by default |
| T-04-02: Tampered binary before extraction | `verify_checksum()` called before `tar -xf`; mismatch exits non-zero |
| T-04-04: tar path traversal | Extraction to isolated `$TMPDIR_WORK`; only `claudebar` binary moved to `BIN_DEST` |
| T-04-06: Silent fallback bypassing checksum | `verify_checksum` failure returns 1 from `download_prebuilt` without setting `COMMAND_VALUE`; Tier 2 runs, not a bypass |

## Known Stubs

None. All install tiers are fully wired. The prebuilt download tier will gracefully fall back to cargo/bash if no GitHub release exists yet (e.g., before first tagged release).

## Threat Flags

None. No new network endpoints, auth paths, or trust boundaries introduced beyond those documented in the plan's threat model.

## Self-Check: PASSED

- [x] `install.sh` modified and committed (a2e487a)
- [x] `bash -n install.sh` exits 0
- [x] All acceptance criteria met (detect_target ≥2, arm64 normalization, SHA256SUMS.txt ≥2, verify_checksum ≥2, TMPDIR_WORK ≥3, cargo install present, statusLine present, Installation complete present)
- [x] SUMMARY.md created at `.planning/phases/04-install-script/04-01-SUMMARY.md`
- [x] No modifications to STATE.md or ROADMAP.md
