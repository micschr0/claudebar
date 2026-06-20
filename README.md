# claudebar

[![CI](https://github.com/micschr0/claudebar/actions/workflows/rust.yml/badge.svg)](https://github.com/micschr0/claudebar/actions/workflows/rust.yml)

A claudebar for Claude Code — context usage, rate limits, and git state on every turn.

<img src="screenshots/skynet.png" width="860" alt="Easter egg">

**Requirements:**
- [Nerd Font](https://www.nerdfonts.com/) — for the Powerline separator and status icon glyphs
- [git](https://git-scm.com/) — reads branch, ahead/behind, and modified-file counts

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh | bash
```

Restart Claude Code. If the claudebar is blank, verify `~/.claude/settings.json` contains `"statusLine": {"type": "command", ...}`. If glyphs show as boxes, install a Nerd Font — macOS Terminal does not support Nerd Font PUA glyphs, use iTerm2, Kitty, WezTerm, Ghostty, or Alacritty.

The installer builds the Rust binary with `cargo` when available. Without `cargo`, it falls back to a standalone bash script (requires [jq](https://jqlang.org/)).

<details>
<summary>Manual install</summary>

**Rust binary (recommended):**

```bash
cargo install --git https://github.com/micschr0/claudebar
```

Add to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "claudebar render"
  }
}
```

**Bash fallback** (no Rust toolchain needed, requires `jq`):

```bash
curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/statusline-command.sh \
  > ~/.claude/statusline-command.sh
chmod +x ~/.claude/statusline-command.sh
```

```json
{
  "statusLine": {
    "type": "command",
    "command": "bash ~/.claude/statusline-command.sh"
  }
}
```

</details>

## Updates

Re-run the install command. Updates take effect on the next turn.

## Configure

```bash
claudebar config      # interactive TUI configurator
```

The TUI lets you toggle and reorder segments, pick a theme and rendering style, and nudge the warning/critical thresholds. Changes are saved to `~/.config/claudebar/config.toml`.

### Themes and styles

- **Themes:** `tokyo-night` (default), `catppuccin`, `gruvbox`, `nord`, `dracula`, `rose-pine`
- **Styles:** `powerline` (default), `plain`, `rounded`, `minimal`, `ascii`

### Config file

TOML at `~/.config/claudebar/config.toml` (`$XDG_CONFIG_HOME/claudebar/config.toml` when set).

```toml
theme = "tokyo-night"
style = "powerline"
segments = ["directory", "git", "context", "rate-limits", "model"]

[thresholds]
warn           = 50   # bar turns yellow at this %
crit           = 80   # bar turns red at this %
weekly_show_at = 50   # show weekly rate-limit only above this %
bar_width      = 6    # bar width in terminal cells
```

Global flags `--theme`, `--style`, and `--config` override the file for a single invocation.

### All subcommands

| Command | What it does |
|---------|--------------|
| `claudebar` / `claudebar render` | Read session JSON from stdin, write ANSI status line to stdout |
| `claudebar config` | Launch the interactive TUI configurator |
| `claudebar init [--print] [--force]` | Write a default config file |
| `claudebar list` | Print all built-in theme and style names |

## Build from source

```bash
cargo build --release          # binary at target/release/claudebar
cargo install --path .         # install to ~/.cargo/bin/claudebar
```

To build without the TUI configurator (render-only, smaller binary):

```bash
cargo build --release --no-default-features
```

## Screenshots

**Calm** — low context and rate-limit usage, everything green:

<img src="screenshots/strip-green.png" width="860" alt="Calm state, all green">

**Normal** — context, 5-hour rate limit, and git state, working along:

<img src="screenshots/strip-normal.png" width="860" alt="Normal state">

**Critical** — context filling up, 5-hour limit tight, weekly window now shown:

<img src="screenshots/strip-critical.png" width="860" alt="Critical state with weekly rate limit">

**Over limit** — past 100% context, both bars red:

<img src="screenshots/strip-overlimit.png" width="860" alt="Over context limit">

**Outside a git repo** — the git segment simply drops out:

<img src="screenshots/strip-nogit.png" width="860" alt="Outside a git repository">

**Model without an effort parameter** — the effort indicator drops out:

<img src="screenshots/strip-noeffort.png" width="860" alt="Model without effort indicator">

## License

[MIT](LICENSE)
