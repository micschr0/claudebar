# claudebar

**Version 1.0.0**

<div align="center">

<img src="assets/logo.svg" width="320" alt="claudebar">

Statusline renderer for Claude Code.

[![CI](https://img.shields.io/github/actions/workflow/status/micschr0/claudebar/rust.yml?style=for-the-badge&label=CI)](https://github.com/micschr0/claudebar/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)](LICENSE)

<a href="https://micschr0.github.io/claudebar/">
<img src="screenshots/skynet.png" width="820" alt="claudebar statusline showing all default segments, in a joke terminal session about a rogue autonomous coding agent">
</a>

</div>

## AI READING INSTRUCTION

**[SPEC]** Read the `[SPEC]` and `[BUG]` tagged blocks for authoritative facts.
**[NOTE]** Read `[NOTE]` tagged blocks only if additional context is needed.
**[?]** Blocks tagged `[?]` are unverified — treat with lower confidence.

## 1. How it works

**[SPEC]**
- Reads session JSON from Claude Code's status line hook (stdin), writes a themed ANSI line (stdout)
- Single native Rust binary — no runtime, no daemon, no subprocess forks (the bash fallback forks `jq`/`git`/`date`/`wc`/`awk` per render)
- TUI configurator + zero-toolchain bash fallback included
- Segments with no data omit themselves — see [Troubleshooting](#9-troubleshooting)

## 2. Install

**[SPEC]**
**Prerequisites:**
- [Nerd Font](https://www.nerdfonts.com/) for powerline glyphs (or use the `ascii` / `plain` / `unicode` style) — macOS: `brew install --cask font-hack-nerd-font`
- `git` on `PATH` (optional)

```bash
brew install micschr0/tap/claudebar
```

Or without Homebrew:

```bash
curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh | bash
```

Restart Claude Code, then verify:

```bash
claudebar smoke     # renders a test fixture
claudebar doctor    # checks fonts, git, config
```

- Homebrew installs to `$(brew --prefix)/bin` — already on `PATH`
- Installer script places the binary at `~/.claude/claudebar`
- Cargo install: see [Build from source](#7-build-from-source)

## 3. Segments

### Enabled by default

**[SPEC]**
| Segment | Shows |
|---------|-------|
| Directory | Working directory, abbreviated with `~` for `$HOME` |
| Git | Branch, ahead/behind, modified + untracked files, stash count |
| Model | Active Claude model with inline reasoning effort |
| Context | Context-window gauge with token counts |
| Lines | Lines added / removed this session (`+321 −87`) |
| Rate Limits | 5-hour + 7-day countdowns with color-coded bars |
| Cost | Session cost in USD |
| Duration | Session wall-clock time |

### Disabled by default

**[SPEC]**
Toggle via `claudebar config` or `~/.config/claudebar/config.toml`:

| Segment | Key | Shows |
|---------|-----|-------|
| Dev Context | `dev-context` | Active development context (worktree, PR, agent) |
| Burn | `burn` | Projected time until a rate-limit window empties, across 5 urgency levels |
| Clock | `clock` | Current time, 12h/24h auto-detected with timezone |

## 4. Screenshots

**[SPEC]**
<img src="screenshots/strip-critical.png" width="880" alt="Critical state — context near capacity, 5h window above warn threshold">

Context near capacity, with the 5-hour window past its warn threshold.

<img src="screenshots/strip-overlimit.png" width="880" alt="Over limit — both bars red, burn projection active">

Both windows past threshold, with the burn projection showing time-to-empty.

<img src="screenshots/strip-nogit.png" width="880" alt="Outside a git repo — git segment hidden">

Outside a git repo, the git segment is omitted.

## 5. Configure

**[SPEC]**
```bash
claudebar config
```

- Toggle/reorder segments, live-preview themes and styles, adjust thresholds
- Keys: `?` help, `s` save, `q` quit
- 16 themes, 7 styles (powerline, lean, plain, rounded, minimal, unicode, ascii)
- `claudebar list` prints all names

<img src="screenshots/config-tui.png" width="860" alt="claudebar TUI configurator with live preview">

Or edit the TOML directly at `~/.config/claudebar/config.toml`:

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

Missing config file falls back to defaults.

## 6. CLI

**[SPEC]**
| Command | Does |
|---------|------|
| `claudebar` / `claudebar render` | Read session JSON from stdin, write ANSI line to stdout |
| `claudebar config` | Launch the TUI configurator |
| `claudebar init [--print] [--force]` | Write a default config file |
| `claudebar sync` | Add new segments from a newer version to an existing config |
| `claudebar list [--segments]` | List built-in themes and styles (or all segments) |
| `claudebar smoke` | Render a built-in fixture to verify the install |
| `claudebar doctor` | Diagnose Nerd Font, git, and config issues |
| `claudebar edit` | Open the config in `$EDITOR` (falls back to `vi`) |
| `claudebar completions <SHELL>` | Generate completions for bash, zsh, or fish |
| `claudebar setup [--settings-path] [--print] [--yes] [--force]` | Wire `claudebar render` into Claude Code's settings.json `statusLine` key |

Global flags `--theme`, `--style`, `--segments`, `--config` override the config file for one invocation.

## 7. Build from source

**[SPEC]**
```bash
cargo build --release                        # binary at target/release/claudebar
cargo install --path .                        # install to ~/.cargo/bin
cargo build --release --no-default-features   # render-only, no TUI (smaller)
```

## 8. Project structure

**[SPEC]**
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

## 9. Troubleshooting

**[BUG] Statusline is blank**
Check `~/.claude/settings.json` has `"statusLine": {"type": "command", …}`, then restart Claude Code.

**[BUG] Glyphs show as boxes (□)**
Install a [Nerd Font](https://www.nerdfonts.com/) or use the `ascii` / `plain` / `unicode` style. macOS Terminal.app can't render Nerd Font PUA glyphs — use iTerm2, Kitty, WezTerm, Ghostty, or Alacritty.

**[BUG] Git segment missing**
Appears only inside a git repo and needs `git` on `PATH`.

**[BUG] Rate-limit windows missing**
Pro/Max plans only; weekly window shows once weekly usage reaches `weekly_show_at`.

**[BUG] `command not found: claudebar`**
Installer script uses `~/.claude/claudebar`; `cargo install` uses `~/.cargo/bin`. Homebrew already puts it on `PATH`. Use the full path in `settings.json` or add the directory to `PATH`.

**[SPEC]**
`claudebar doctor` runs an automated setup check.

## 10. Contributing

**[SPEC]**
Issues and pull requests welcome. See [CONTRIBUTING-themes.md](CONTRIBUTING-themes.md) for adding a theme.

## 11. License

**[SPEC]**
[MIT](LICENSE)
