<div align="center">

<img src="assets/logo.svg" width="320" alt="claudebar">

**A powerline-style status line for Claude Code — segments, themes, and a live TUI configurator, in a single native binary.**

[![CI](https://img.shields.io/github/actions/workflow/status/micschr0/claudebar/rust.yml?style=flat-square&label=CI)](https://github.com/micschr0/claudebar/actions/workflows/rust.yml)
[![Claude Code](https://img.shields.io/badge/Built%20with-Claude%20Code-DA7857?style=flat-square&logo=anthropic)](https://claude.ai/code)
[![Claude Skills](https://img.shields.io/badge/Uses-Claude%20Skills-DA7857?style=flat-square&logo=anthropic)](.agents/skills)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024-orange?style=flat-square)](Cargo.toml)

**[Documentation & live demo](https://micschr0.github.io/claudebar/)**

[Screenshots](#screenshots) • [Install](#install) • [Configure](#configure) • [CLI](#cli-reference) • [Segments](#segments) • [Build from source](#build-from-source)

<a href="https://micschr0.github.io/claudebar/">
<img src="screenshots/skynet.png" width="820" alt="claudebar statusline showing all default segments">
</a>

</div>

## What it does

claudebar reads the session JSON that Claude Code's status line hook sends over stdin, and writes back one themed ANSI line: current directory, git state, active model, context usage, session cost, and rate-limit windows — all in a single native binary, no runtime and no background process.

- **Single binary** — no daemon, no subprocess forks. (A zero-toolchain bash fallback is included for environments without a Rust build, but it forks `jq`/`git`/`date` per render.)
- **TUI configurator** — toggle segments, preview themes and styles live, and tune thresholds without hand-editing TOML.
- **Segments hide themselves** — no git repo, no rate-limit data, no active effort level: the segment just doesn't render. See [Troubleshooting](#troubleshooting).

## Screenshots

<img src="screenshots/strip-critical.png" width="880" alt="Critical state — context near capacity, 5h window above warn threshold">

Context near capacity, with the 5-hour window past its warn threshold.

<img src="screenshots/strip-overlimit.png" width="880" alt="Over limit — both bars red, burn projection active">

Both windows past threshold, with the burn projection showing time-to-empty.

<img src="screenshots/strip-nogit.png" width="880" alt="Outside a git repo — git segment hidden">

Outside a git repo, the git segment is omitted entirely rather than shown empty.

## Install

**Prerequisites**
- A [Nerd Font](https://www.nerdfonts.com/) for powerline glyphs (or pick the `ascii` / `plain` / `unicode` style instead) — macOS: `brew install --cask font-hack-nerd-font`
- `git` on `PATH` (optional, only needed for the git segment)

```bash
brew install micschr0/tap/claudebar
```

Or without Homebrew:

```bash
curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh | bash
```

Restart Claude Code, then verify the install:

```bash
claudebar smoke     # renders a test fixture
claudebar doctor    # checks fonts, git, config
```

> [!NOTE]
> Homebrew installs to `$(brew --prefix)/bin`, already on `PATH`. The install script places the binary at `~/.claude/claudebar` instead — see [Troubleshooting](#troubleshooting) if `claudebar` isn't found afterward.

## Configure

```bash
claudebar config
```

Opens a full-screen TUI: toggle and reorder segments, live-preview all 16 themes and 7 styles against your own session data, and nudge thresholds with instant feedback. Press `?` for the key reference, `s` to save, `q` to quit.

<img src="screenshots/config-tui.png" width="860" alt="claudebar TUI configurator with live preview">

Prefer to edit by hand? The TUI writes plain TOML at `~/.config/claudebar/config.toml`:

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

A missing config file falls back to sensible defaults — `claudebar list` prints every built-in theme and style name.

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

## Reporting issues

Open an issue on [GitHub Issues](https://github.com/micschr0/claudebar/issues). Include:

- A clear description of the problem
- Steps to reproduce
- Expected vs. actual behavior
- Your terminal emulator and OS
- Your claudebar version (`claudebar --version`)
- When relevant, the output of `claudebar doctor` or the config in question

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

## Status & metrics

| Metric | Count |
|---|---|
| Segments | 11 (8 default + 3 optional) |
| Themes | 16 |
| Styles | 7 |
| Test cases | 232 passing |
| Snapshot tests | 46 |
| Edge-case fixtures | 14 |

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
- [shields.io](https://shields.io/) — badges
- [Claude Code](https://claude.ai/code) (Anthropic) — pair-programming this project
- [GitHub Pages](https://pages.github.com/) — hosting the live demo

## License

[MIT](LICENSE)

## Contact

Maintainer: [@micschr0](https://github.com/micschr0)
