# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Swap duration glyph to stopwatch in the powerline style
- Remove duplicate Nerd Font tip in `install.sh`

### Tests
- Add bats test suite for `statusline-command.sh` and `install.sh`; CI now runs it in place of the smoke job

## [2026.7.3] - 2026-07-03

### Added
- `claudebar setup` wires `statusLine` into `settings.json`, with a `--binary-path` override
- `claudebar doctor` checks `statusLine` wiring and cross-references `setup`
- `setup` prints a live preview on success

### Fixed
- Share Nerd Font bootstrap logic between `init` and `edit`

## [2026.7.2] - 2026-07-03

### Added
- Expand to 14 segments, TUI overhaul, promo assets, README rewrite (#18)
- Homebrew tap installer via `cargo-dist`
- `cargo-auditable` for dist builds
- `init` falls back to the unicode style when no Nerd Font is available
- Promo page served via GitHub Pages from `docs/`
- `dist generate --check` guard in the release workflow

### Changed
- Replace `release.yml` with a `cargo-dist`-generated workflow
- Apply Strunk's *Elements of Style* to all prose
- Features heading reworded to "Your session, in one line."

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
- Re-render the demo video, sync to GitHub Pages

## [2026.6.24] / [v0.2.0] - 2026-06-24

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

## [2026.6.20] - 2026-06-21

Initial development history: dev-context segment, `migrate` subcommand, 10 additional themes, CI badge, `--no-default-features` CI coverage, and the three-tier `install.sh` fallback rewrite.

[Unreleased]: https://github.com/micschr0/claudebar/compare/2026.7.3...HEAD
[2026.7.3]: https://github.com/micschr0/claudebar/compare/2026.7.2...2026.7.3
[2026.7.2]: https://github.com/micschr0/claudebar/compare/2026.6.24...2026.7.2
[2026.6.24]: https://github.com/micschr0/claudebar/compare/v0.2.0...2026.6.24
[v0.2.0]: https://github.com/micschr0/claudebar/compare/2026.6.20...v0.2.0
[2026.6.20]: https://github.com/micschr0/claudebar/releases/tag/2026.6.20
