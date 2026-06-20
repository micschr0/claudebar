# Technology Stack

**Analysis Date:** 2026-06-20

## Languages

**Primary:**
- Rust 2021 edition — all application code in `src/`

**Secondary:**
- Bash — `statusline-command.sh` (zero-toolchain fallback), `install.sh`, `scripts/benchmark.sh`, `scripts/make_demo_repos.sh`
- Python 3 — `scripts/gen_screenshots.py` (screenshot generation only, not part of the binary)

## Runtime

**Environment:**
- Native binary — no runtime VM

**Toolchain:**
- Rust 1.95.0 (pinned in `.agents/skills/rust-skills/checks/rust-toolchain.toml` for CI harness)
- System Rust 1.96.0 available locally

**Package Manager:**
- Cargo 1.96.0
- Lockfile: `Cargo.lock` present and committed

## Frameworks

**Core:**
- `clap` 4 (derive feature) — CLI argument parsing (`src/cli.rs`)
- `ratatui` 0.29 — TUI widget layout (`src/tui/`) — optional, `tui` feature only
- `crossterm` 0.28 — terminal control / raw mode (`src/tui/`) — optional, `tui` feature only
- `ansi-to-tui` 7 — render ANSI preview inside ratatui (`src/tui/preview.rs`) — optional, `tui` feature only

**Testing:**
- `insta` 1 (filters + glob features) — snapshot testing (`tests/render_golden.rs`, `tests/snapshots/`)

**Build/Dev:**
- Task 3 (`Taskfile.yml`) — task runner wrapping cargo commands
- `shellcheck` — bash static analysis (lint task)

## Key Dependencies

**Critical:**
- `serde` 1 + derive — serialization/deserialization foundation for all JSON and TOML I/O
- `serde_json` 1 — parses Claude Code session JSON from stdin (`src/model/input.rs`)
- `toml` 0.8 — reads/writes user config at `$XDG_CONFIG_HOME/claudebar/config.toml`
- `thiserror` 1 — typed error definitions across the codebase

**Infrastructure:**
- `clap` 4 — subcommand dispatch: `render`, `config`, `init`, `list` (`src/main.rs`)

## Features

**`tui` (default):**
- Enables `ratatui`, `crossterm`, `ansi-to-tui`
- Includes `src/tui/` module
- Required for `claudebar config` subcommand

**Render-only build (`--no-default-features`):**
- Strips all TUI deps
- Produces a minimal binary for the Claude Code hook hot path
- Zero dependency on crossterm/ratatui

## Configuration

**User config:**
- TOML at `$XDG_CONFIG_HOME/claudebar/config.toml` (fallback: `~/.config/claudebar/config.toml`)
- Managed by `src/model/config.rs`
- Missing file falls back to `Config::default()`

**Build:**
- `Cargo.toml` — release profile: `opt-level=3`, `lto=true`, `strip=true`, `codegen-units=1`
- `Taskfile.yml` — task aliases for build, test, lint, render, screenshots

## Platform Requirements

**Development:**
- Rust toolchain (1.95+ pinned)
- `jq` and `git` on PATH (runtime deps for the bash fallback and `git` segment)
- Nerd Font in terminal for powerline glyphs

**Production:**
- Single statically-linked binary (Linux x86_64 primary target)
- No runtime dependencies for the Rust binary beyond a POSIX shell environment
- Bash fallback (`statusline-command.sh`) requires `jq` and `git`

---

*Stack analysis: 2026-06-20*
