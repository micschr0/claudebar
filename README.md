# claudebar

[![CI](https://github.com/micschr0/claudebar/actions/workflows/rust.yml/badge.svg)](https://github.com/micschr0/claudebar/actions/workflows/rust.yml)

**A fast, themeable statusline for Claude Code — your session at a glance, on every turn.**

claudebar reads Claude Code's session JSON and renders one clean ANSI line: where you are, what git's doing, how much context you've burned, and — the part you actually want — a live countdown to your next rate-limit reset. It's a single tiny binary with no runtime dependencies, 16 themes, 6 rendering styles, and a TUI to wire it up in under a minute.

<img src="screenshots/animated.svg" alt="claudebar demo">

## Why claudebar

- **Never get surprised by a rate limit** — live countdowns to your 5-hour *and* weekly reset windows, with the weekly window appearing only once it actually matters. Know how long you've got before you hit a wall.
- **Context usage at a glance** — a color-coded usage bar plus token count, turning yellow at your warn threshold and red when you're close to the edge.
- **Git state, inline** — branch name, ahead/behind counts, and modified-file count, read straight from your working tree. Drops out cleanly outside a repo.
- **Everything you need, nothing you don't** — directory, git, context, rate limits, dev context (worktree / PR / agent), and model + effort level. Toggle and reorder any of them.
- **16 themes, 6 styles** — Tokyo Night, Catppuccin, Gruvbox, Nord, Dracula, Rose Pine and more, in Powerline, rounded, minimal, unicode, plain, or ASCII.
- **A TUI to set it all up** — `claudebar config` lets you toggle segments, reorder them, pick a theme and style, and tune thresholds with a live preview. No hand-editing TOML required.
- **Tiny and dependency-free** — a single statically-linked binary with an LTO-stripped release build and a render-only mode for the absolute minimum footprint. Parsing never fails: malformed or partial session JSON degrades gracefully instead of breaking your prompt.
- **No toolchain? No problem** — a zero-dependency bash fallback (just `jq` + `git`) for environments without Rust.

### What each segment shows

| Segment | Shows |
|---------|-------|
| Directory | Fish-style abbreviated path |
| Git | Branch, ahead/behind, modified-file count |
| Context | Usage bar + token count, colored by threshold |
| Rate limits | 5-hour and weekly windows with live reset countdown |
| Dev context | Worktree name, PR number + review state, sub-agent name |
| Model | Model name and effort level |

### Configure it in seconds

`claudebar config` opens an interactive TUI — toggle and reorder segments, switch themes and styles, and nudge thresholds, all with a live render preview.

<img src="screenshots/config-tui.png" width="860" alt="The claudebar TUI configurator">

**Requirements:**
- [Nerd Font](https://www.nerdfonts.com/) — for the Powerline separator and status icon glyphs
- [git](https://git-scm.com/) — reads branch, ahead/behind, and modified-file counts

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh | bash
```

Restart Claude Code. If the claudebar is blank, verify `~/.claude/settings.json` contains `"statusLine": {"type": "command", ...}`. If glyphs show as boxes, install a Nerd Font — macOS Terminal does not support Nerd Font PUA glyphs, use iTerm2, Kitty, WezTerm, Ghostty, or Alacritty.

The installer tries three methods in order: (1) downloads a prebuilt binary from GitHub Releases (verified by SHA256), (2) builds from source with `cargo` if run from a local checkout, (3) falls back to a standalone bash script. The bash fallback requires [jq](https://jqlang.org/).

<details>
<summary>Manual install</summary>

**Prebuilt binary:**

Download the latest release for your platform from the [releases page](https://github.com/micschr0/claudebar/releases), extract, and place `claudebar` on your `$PATH` or at `~/.claude/claudebar`.

**Rust binary (build from source):**

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

| Key | Action |
|-----|--------|
| `j` / `k` or ↑↓ | Move cursor |
| `Tab` / `Shift-Tab` | Jump to next/previous section |
| `1`–`4` | Jump to section by number |
| `Space` | Toggle segment on/off |
| `m` | Enter reorder mode |
| `h` / `l` or ←→ | Nudge threshold ±1 |
| `H` / `L` | Nudge threshold ±5 |
| `s` | Save · `r` Reset to defaults · `?` Help · `q` Quit |

### Segments

| Segment | TOML key | What it shows |
|---------|----------|---------------|
| Directory | `directory` | Fish-style abbreviated path |
| Git | `git` | Branch name, ahead/behind, modified-file count |
| Context | `context` | Context usage bar and token count |
| Rate limits | `rate-limits` | 5-hour and weekly rate-limit windows with countdown |
| Dev context | `dev-context` | Dev context bar |
| Model | `model` | Model name and effort level |

Toggle segments with `claudebar config` or edit the `segments` list in `config.toml`.

### Themes and styles

- **Themes (16):** `tokyo-night` (default) · `ayu-mirage` · `catppuccin` · `cobalt2` · `everforest-dark` · `github-dark` · `gruvbox` · `kanagawa-wave` · `moonfly` · `night-owl` · `nord` · `one-dark` · `dracula` · `rose-pine` · `sonokai` · `solarized-dark`
- **Styles (6):** `powerline` (default) · `plain` · `rounded` · `minimal` · `unicode` · `ascii`

### Config file

TOML at `~/.config/claudebar/config.toml` (`$XDG_CONFIG_HOME/claudebar/config.toml` when set).

```toml
theme = "tokyo-night"
style = "powerline"
segments = ["directory", "git", "context", "rate-limits", "dev-context", "model"]

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

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| **Statusline is blank** | Check `~/.claude/settings.json` has `"statusLine": {"type": "command", ...}`, then restart Claude Code — it refreshes on the next turn. |
| **Glyphs show as boxes (□)** | Install a [Nerd Font](https://www.nerdfonts.com/) and set it as your terminal font. macOS Terminal.app does not render Nerd Font PUA glyphs — use iTerm2, Kitty, WezTerm, Ghostty, or Alacritty. |
| **Git segment missing** | The git segment only appears inside a git repository, and needs `git` on your `PATH`. |
| **Rate-limit windows missing** | These come from Claude Code's session JSON and only appear on Pro/Max plans; the weekly window shows only once it crosses `weekly_show_at`. |
| **`command not found: claudebar`** | The binary installs to `~/.claude/claudebar` — use the full path in `settings.json`, or move it onto your `PATH`. |
| **Bash fallback errors** | The fallback needs both [`jq`](https://jqlang.org/) and `git` installed. |

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

<details>
<summary>Easter egg</summary>

<img src="screenshots/skynet.png" width="860" alt="Easter egg">

</details>

## License

[MIT](LICENSE)
