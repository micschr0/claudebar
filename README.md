<div align="center">

<img src="assets/logo.svg" width="240" alt="claudebar">

**A powerline statusline for Claude Code: segments, themes, and a live TUI configurator in a single native binary.**

[![CI](https://img.shields.io/github/actions/workflow/status/micschr0/claudebar/rust.yml?style=flat-square&label=CI)](https://github.com/micschr0/claudebar/actions/workflows/rust.yml) [![Release](https://img.shields.io/github/v/release/micschr0/claudebar?style=flat-square&label=release)](https://github.com/micschr0/claudebar/releases/latest) [![Security](https://img.shields.io/github/actions/workflow/status/micschr0/claudebar/security.yml?style=flat-square&label=Security)](https://github.com/micschr0/claudebar/actions/workflows/security.yml) [![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos-lightgrey?style=flat-square)](CLAUDE.md) [![Rust 2024](https://img.shields.io/badge/rust-2024-%23CE422B?style=flat-square)](Cargo.toml) [![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)

**[Documentation & live demo](https://micschr0.github.io/claudebar/)**

</div>

<img src="screenshots/normal.png" alt="claudebar statusline pinned at the bottom of a Claude Code session">

## Install

macOS and Linux, x86_64 / aarch64. One command — the installer downloads the binary and wires it into Claude Code's `settings.json`:

```bash
curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh | bash
```

Or via Homebrew, then let `setup` do the wiring:

```bash
brew install micschr0/tap/claudebar
claudebar setup
```

Restart Claude Code and the statusline is live. If anything looks off, `claudebar doctor` checks fonts, git, config, and PATH.

> [!NOTE]
> Powerline glyphs need a [Nerd Font](https://www.nerdfonts.com/); the `ascii` style needs none.

## States (< 50% / 50–80% / > 80%)

<img src="screenshots/strip-normal.png" width="860" alt="Normal: calm baseline">

<img src="screenshots/strip-critical.png" width="860" alt="Critical: a rate limit is approaching">

<img src="screenshots/strip-overlimit.png" width="860" alt="Over limit: past the threshold">

## Segments

Eight on by default. dev-context, burn, and clock sit off by default — flip them on when you need them.

<img src="screenshots/segment-pills.png" width="860" alt="Every claudebar segment: directory, git, model, context, dev-context, rate limits, lines, cost, burn, duration, clock">

## Configure

```bash
claudebar config
```

Full-screen TUI on your own session data: live preview, theme and style pickers, threshold sliders. `?` for keys, `s` saves, `q` quits.

<img src="screenshots/config-tui.png" width="860" alt="claudebar TUI configurator with live preview, theme picker, and thresholds">

<img src="screenshots/config-tui-style.png" width="860" alt="claudebar TUI style picker with live preview for each style">

Or edit the TOML at `~/.config/claudebar/config.toml` directly (`claudebar edit`):

```toml
theme = "tokyo-night"
style = "powerline"
segments = ["directory", "git", "model", "context", "lines", "rate-limits", "cost", "duration"]

[thresholds]
warn = 50   # bar turns yellow
crit = 80   # bar turns red
```

## CLI reference

| Command | Action |
|---|---|
| `claudebar` / `claudebar render` | Read session JSON from stdin, write ANSI statusline to stdout |
| `claudebar config` | Launch the TUI configurator |
| `claudebar setup` | Wire claudebar into Claude Code's `settings.json` |
| `claudebar list` | List built-in themes and styles |
| `claudebar doctor` | Diagnose font, git, config, and PATH issues |

More commands and flags: `claudebar --help`.

## Uninstall

```bash
brew uninstall claudebar
# or: rm ~/.claude/claudebar   # script install
```

Then remove the `statusLine` entry from `~/.claude/settings.json` and, optionally, `~/.config/claudebar/`.

---

**More:** [documentation & live demo](https://micschr0.github.io/claudebar/) · [build from source](https://micschr0.github.io/claudebar/#build) · [contributing a theme](CONTRIBUTING-themes.md) · [changelog](CHANGELOG.md) · [report an issue](https://github.com/micschr0/claudebar/issues)

## License

[MIT](LICENSE)
