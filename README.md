<div align="center">

<img src="assets/logo.svg" width="320" alt="claudebar">

**A powerline-style status line for Claude Code — segments, themes, and a live TUI configurator, in a single native binary.**

[![CI](https://img.shields.io/github/actions/workflow/status/micschr0/claudebar/rust.yml?style=flat-square&label=CI)](https://github.com/micschr0/claudebar/actions/workflows/rust.yml)
[![Security](https://img.shields.io/github/actions/workflow/status/micschr0/claudebar/security.yml?style=flat-square&label=Security)](https://github.com/micschr0/claudebar/actions/workflows/security.yml)
[![Release](https://img.shields.io/github/v/release/micschr0/claudebar?style=flat-square&label=release)](https://github.com/micschr0/claudebar/releases/latest)
[![Claude Code](https://img.shields.io/badge/Built%20with-Claude%20Code-DA7857?style=flat-square&logo=anthropic)](https://claude.ai/code)
[![Claude Skills](https://img.shields.io/badge/Uses-Claude%20Skills-DA7857?style=flat-square&logo=anthropic)](.agents/skills)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024-orange?style=flat-square)](Cargo.toml)
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos-lightgrey?style=flat-square)](CLAUDE.md)

**[Documentation & live demo](https://micschr0.github.io/claudebar/)**

[Screenshots](#screenshots) • [Install](#install) • [Configure](#configure) • [CLI](#cli-reference) • [Segments](#segments) • [Build from source](#build-from-source)

<a href="https://micschr0.github.io/claudebar/">
<img src="screenshots/segment-pills.png" width="820" alt="claudebar segments rendered as individual pill cards">
</a>

</div>

## What it does

claudebar renders the Claude Code status line: current directory, git state, active model, context usage, session cost, and rate-limit windows. It reads the session JSON that Claude Code's status line hook sends over stdin and writes back one themed ANSI line — a single ~1.6 MB native binary that renders in ~30 ms, no daemon, no background process. A zero-toolchain bash fallback is included for environments without a Rust build.

## Screenshots

### All segments at a glance

<img src="screenshots/chips.png" width="860" alt="All 11 claudebar segment feature chips">

### Color-coded states

| <img src="screenshots/thumb-normal.png" width="260" alt="Normal — green statusline"> | <img src="screenshots/thumb-critical.png" width="260" alt="Critical — yellow statusline"> | <img src="screenshots/thumb-overlimit.png" width="260" alt="Over limit — red statusline"> |
|:--:|:--:|:--:|
| Normal | Critical | Over limit |

## Install

Supported platforms: macOS and Linux (x86_64 / aarch64).

```bash
brew install micschr0/tap/claudebar
claudebar setup     # wires claudebar into Claude Code's settings.json
```

Or without Homebrew — the script downloads the release binary (checksum-verified), installs it to `~/.claude/claudebar`, and runs `setup` for you, backing up `settings.json` first:

```bash
curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh | bash
```

> [!NOTE]
> Powerline glyphs need a [Nerd Font](https://www.nerdfonts.com/) — macOS: `brew install --cask font-hack-nerd-font` — or pick the `ascii` / `plain` / `unicode` style instead. `git` on `PATH` is optional, only needed for the git segment.

Restart Claude Code, then verify:

```bash
claudebar smoke     # renders a test fixture
claudebar doctor    # checks fonts, git, config
```

## Configure

```bash
claudebar config
```

Launches a full-screen TUI against your own session data. Press `?` for the key reference, `s` to save, `q` to quit.

<img src="screenshots/config-tui.png" width="860" alt="claudebar TUI configurator with live preview">

The TUI writes plain TOML at `~/.config/claudebar/config.toml` — hand-edit it directly if you'd rather skip the UI:

```toml
theme = "tokyo-night"
style = "powerline"
segments = ["directory", "git", "model", "context", "lines", "rate-limits", "cost", "duration"]

[thresholds]
warn           = 50       # bar turns yellow at this %
crit           = 80       # bar turns red at this %
weekly_show_at = 75       # weekly window shown at this % and above
bar_width      = 6        # progress-bar width in cells
layout         = "fixed"  # "fixed" = single line, "auto" = responsive wrap
```

A missing config file falls back to sensible defaults. 16 themes and 7 styles are built in — `claudebar list` prints every name, and the [theme gallery](https://micschr0.github.io/claudebar/#gallery) shows every combination rendered.

## CLI reference

| Command | Does |
|---|---|
| `claudebar` / `claudebar render` | Read session JSON from stdin, write the ANSI line to stdout (the default, what Claude Code calls) |
| `claudebar config` | Launch the TUI configurator |
| `claudebar init [--print] [--force]` | Write a default config file |
| `claudebar sync` | Add newly introduced segments to an existing config |
| `claudebar list [--segments]` | List built-in themes and styles (or all segments) |
| `claudebar smoke` | Render a built-in fixture to verify the install |
| `claudebar doctor` | Diagnose Nerd Font, git, and config issues |
| `claudebar edit` | Open the config in `$EDITOR` (falls back to `vi`) |
| `claudebar completions <shell>` | Generate completions for bash, zsh, or fish |
| `claudebar setup [--settings-path] [--print] [--yes] [--force]` | Wire `claudebar render` into Claude Code's `settings.json` `statusLine` key |

Global flags — `--theme`, `--style`, `--segments`, `--config` — override the config file for a single invocation.

## Segments

### Enabled by default

| Segment | Shows |
|---|---|
| Directory | Working directory, abbreviated with `~` for `$HOME` |
| Git | Branch, ahead/behind, modified + untracked files, stash count |
| Model | Active Claude model with inline reasoning effort |
| Context | Context-window gauge with token counts |
| Lines | Lines added / removed this session (`+321 −87`) |
| Rate Limits | 5-hour + 7-day countdowns with color-coded bars |
| Cost | Session cost in USD |
| Duration | Session wall-clock time |

### Available, disabled by default

Toggle these via `claudebar config` or directly in `config.toml`:

| Segment | Key | Shows |
|---|---|---|
| Dev Context | `dev-context` | Active development context (worktree, PR, agent) |
| Burn | `burn` | Projected time until a rate-limit window empties, across 5 urgency levels |
| Clock | `clock` | Current time, 12h/24h auto-detected with timezone |

## Build from source

```bash
cargo build --release                        # binary at target/release/claudebar
cargo install --path .                       # install to ~/.cargo/bin
cargo build --release --no-default-features  # render-only build, no TUI (smaller)
```

## Troubleshooting

> [!TIP]
> `claudebar doctor` runs all of these checks automatically.

**Status line is blank**
Check that `~/.claude/settings.json` has a `"statusLine": {"type": "command", …}` entry, then restart Claude Code.

**Glyphs show as boxes (□)**
Install a [Nerd Font](https://www.nerdfonts.com/), or switch to the `ascii` / `plain` / `unicode` style. macOS Terminal.app can't render Nerd Font PUA glyphs — use iTerm2, Kitty, WezTerm, Ghostty, or Alacritty instead.

**Git segment missing**
It only appears inside a git repository, and needs `git` on `PATH`.

**Rate-limit windows missing**
Available on Pro/Max plans only; the weekly window appears once weekly usage reaches `weekly_show_at`.

**`command not found: claudebar`**
The install script places the binary at `~/.claude/claudebar`; `cargo install` uses `~/.cargo/bin`. Homebrew already puts it on `PATH`. Either add the right directory to `PATH`, or point `settings.json` at the full binary path.

## Uninstall

```bash
brew uninstall claudebar    # or: rm ~/.claude/claudebar  /  rm ~/.cargo/bin/claudebar
```

Then remove the `statusLine` entry from `~/.claude/settings.json` and, optionally, the config at `~/.config/claudebar/`.

## Reporting issues

Open an issue on [GitHub Issues](https://github.com/micschr0/claudebar/issues). Include:

- Steps to reproduce, expected vs. actual behavior
- Terminal emulator, OS, and claudebar version (`claudebar --version`)
- The output of `claudebar doctor` when relevant

## Contributing

Issues and pull requests are welcome. See [CONTRIBUTING-themes.md](CONTRIBUTING-themes.md) for adding a new theme.

## Project structure

```
src/
  model/      Input JSON, config, palette, style types
  render/     Segment composition → ANSI string
  segment/    One module per statusline segment
  styles/     Built-in glyph styles (powerline, ascii, …)
  themes/     Built-in color themes
  tui/        ratatui configurator (feature = "tui")
fixtures/     JSON edge-case inputs for testing
scripts/      Screenshot and benchmark tooling
tests/        Unit + insta snapshot tests
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release notes.

## Acknowledgements

claudebar builds on:

- [ratatui](https://ratatui.rs/) — TUI widget layout
- [crossterm](https://github.com/crossterm-rs/crossterm) — terminal control
- [clap](https://github.com/clap-rs/clap) — CLI argument parsing
- [serde](https://serde.rs/) — JSON/TOML serialization
- [insta](https://insta.rs/) — snapshot testing
- [Nerd Fonts](https://www.nerdfonts.com/) — powerline glyphs
- [Claude Code](https://claude.ai/code) (Anthropic) — pair-programming this project

## License

[MIT](LICENSE)

## Contact

Maintainer: [@micschr0](https://github.com/micschr0)
