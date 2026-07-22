# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [Unreleased]

## [2026.7.21]

### Added
- Ship Homebrew prereleases to a versioned `@beta` formula: `brew install micschr0/tap/claudebar@beta` for beta users; the plain `claudebar` formula keeps tracking the latest stable, so `brew upgrade` cannot silently bump stable users onto a prerelease
- Document the `claudebar@beta` install path and the `brew uninstall && brew install` switch-back command in the README

### Changed
- Harden the custom Homebrew publish job in `release.yml`: `set -euo pipefail`, input-assertion of the dist artifact, a verified `class Claudebar` → `class ClaudebarATBeta` rename via `re.subn` substitution counting, and an informational `brew audit --new` step for formula-class / filename drift visibility
- Enable `mold` linker for x86_64-unknown-linux-musl release builds to cut link time

### CI
- Centralize zizmor suppressions in `zizmor.yml`; inline ignores for known false positives in the homebrew app-token workflow
- SHA-pin the docs deploy `actions/*` suite (`configure-pages`, `upload-pages-artifact`, `deploy-pages`, `settings`)
- Inline the homebrew tap publish into `release.yml` as a dedicated job (drops `homebrew-app-token.yml`) and auto-prune older prereleases after each publish via `gh release delete --cleanup-tag`
- Add `cargo-llvm-cov` + `scttnlsn/covrs` PR coverage reporting on `ubuntu-latest`

## [2026.7.7]

### Added
- `claudebar setup` prints a restart reminder after wiring up `settings.json`, matching the installer
- `install.sh` supports a beta channel (`CLAUDEBAR_CHANNEL=beta`) that installs the latest prerelease for testing before a stable release

### Fixed
- `install.sh`: add `--force` to `setup` call so an existing `statusLine` in `settings.json` doesn't abort the install script before `link_onto_path` and the success message
- `install.sh`: resolve cargo-dist archive nesting so the inner-directory layout extracts correctly
- `install.sh`: guard `main()` against unbound `BASH_SOURCE` when piped through `curl | bash`
- `verify-install.yml`: exercise piped-stdin execution alongside sourced execution to catch regressions
- Add a GitHub App token to `release.yml` so the release publish cascades to the Homebrew workflow
- Replace the Renovate PAT with a short-lived GitHub App token; scope its permissions and add `persist-credentials: false` to checkout steps
- `zizmor`: add artipacked ignore for the auto-generated `release.yml`

### Changed
- Update `time` to 0.3.53 and `clap_complete` to 4.6.7

### Docs
- Move the release-verification guide out of the README into `SECURITY.md`; note that Homebrew installs are not covered by attestation verification
- Streamline the README install section; add provenance and downloads badges

### Security
- Replaced long-lived Renovate PAT with a 1-hour GitHub App installation token
- Added `persist-credentials: false` to all `actions/checkout` steps

## [2026.7.5]

### Fixed
- Swap duration glyph to stopwatch in the powerline style
- Remove duplicate Nerd Font tip in `install.sh`
- Join the rate-limits segment's 5h and weekly windows with a dim-colored gap glyph; consolidate the bar-to-percent gap into `SegmentWriter::bar_pct()`

### Tests
- Add bats test suite for `statusline-command.sh` and `install.sh`; replace the smoke job in CI
- Exercise the weekly-window gap glyph in the golden-snapshot suite and live TUI preview by crossing the `weekly_show_at` threshold

### Security
- Attest release artifacts with GitHub build provenance (`actions/attest-build-provenance`); `install.sh` verifies it non-fatally via `gh attestation verify` after the SHA256 gate (#27, closes #19)
- Add `SECURITY.md` (supported versions, private vulnerability reporting) and document `gh attestation verify` in the README

### CI
- Switch GitHub Pages from workflow-based to branch-based deployment (main + /docs)
- Harden all workflows: pin every action to a commit SHA, pass zizmor, and scope job permissions
- Install `cargo-dist` as a prebuilt binary and unify all `checkout` actions to v7 (#28, closes #26)

## [2026.7.3]

### Added
- Add `claudebar setup` to wire `statusLine` into `settings.json` with a `--binary-path` override
- Add `claudebar doctor` to check `statusLine` wiring and cross-reference `setup`
- Print a live preview after `setup` succeeds

### Fixed
- Share Nerd Font bootstrap logic between `init` and `edit`

## [2026.7.2]

### Added
- Expand to 14 segments, TUI overhaul, promo assets, README rewrite (#18)
- Add Homebrew tap installer via `cargo-dist`
- Build dist binaries with `cargo-auditable`
- Fall back to the unicode style in `init` when no Nerd Font is available
- Serve the promo page via GitHub Pages from `docs/`
- Add `dist generate --check` guard to the release workflow

### Changed
- Replace `release.yml` with a `cargo-dist`-generated workflow
- Apply Strunk's *Elements of Style* to all prose
- Reword the features heading to "Your session, in one line."

### Fixed
- Move Homebrew publish to a custom dist job
- Use a GitHub App token for the homebrew-tap push
- Drop the deprecated `x86_64-apple-darwin` macOS-13 pin
- Define `[profile.dist]` for `cargo-dist`
- Point `install.sh` at the correct dist artifact names
- Drop duplicate hero heading; link the promo site from the README

### Docs
- Add a Homebrew cask hint for the Nerd Font on macOS
- Add `RELEASING.md` — inverted version model and smoke-tag ritual
- Re-render the demo video and sync it to GitHub Pages

## [2026.6.24] / [v0.2.0]

### Added
- Public-ready claudebar: TUI, installer, docs, demos, and branding (initial public release)
- Promo video via the hve-spielberg pipeline
- Product social preview card (logo wordmark + live statusline)

### Changed
- Trim README — drop What-you-see, key bindings, and theme/style lists (moved to the promo page)
- Upgrade to Rust edition 2024
- Buffer-writing render path — allocation-free bar, path, and count parsing for lower per-render overhead

### Fixed
- Guard `make_bar` against `u32` underflow when width is 0
- Strict 2^64 / 2^63 bounds in numeric coercion
- Lift low-contrast theme color slots; refresh README hero and screenshots
- Bump `crossterm` to 0.29, `toml` to 1.0/0.9, `thiserror` to v2, `ratatui`/`ansi-to-tui` to 0.30/v8

### CI
- Add a self-hosted Renovate workflow for dependency updates
- Run shellcheck directly, drop the abandoned ludeeus action

## [2026.6.20]

Initial development history: dev-context segment, `migrate` subcommand, 10 additional themes, CI badge, `--no-default-features` CI coverage, and the three-tier `install.sh` fallback rewrite.

[Unreleased]: https://github.com/micschr0/claudebar/compare/2026.7.7...HEAD
[2026.7.7]: https://github.com/micschr0/claudebar/compare/2026.7.5...2026.7.7
[2026.7.5]: https://github.com/micschr0/claudebar/compare/2026.7.3...2026.7.5
[2026.7.3]: https://github.com/micschr0/claudebar/compare/2026.7.2...2026.7.3
[2026.7.2]: https://github.com/micschr0/claudebar/compare/2026.6.24...2026.7.2
[2026.6.24]: https://github.com/micschr0/claudebar/compare/v0.2.0...2026.6.24
