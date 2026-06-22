<div align="center">

<img src="assets/logo.svg" width="440" alt="claudebar">

**A fast, themeable statusline for Claude Code — see your context usage and rate-limit countdowns at a glance, every turn.**

[![CI](https://github.com/micschr0/claudebar/actions/workflows/rust.yml/badge.svg)](https://github.com/micschr0/claudebar/actions/workflows/rust.yml)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

<img src="screenshots/intro.png" width="820" alt="claudebar statusline demo — a real terminal recording">

</div>

claudebar turns Claude Code's session JSON into one clean ANSI line — **directory · git · context · rate limits · dev context · model**. One dependency-free binary, **16 themes**, **6 styles**, and a TUI to set it up.

> [!NOTE]
> Read-only by design — claudebar never touches your session or Claude's behavior.

## Features

- **Never hit a wall blind** — live countdowns to your 5-hour *and* weekly resets; the weekly window appears only when it matters.
- **Context usage, color-coded** — a usage bar and token count that shift green → yellow → red as you near the edge.
- **Git state, inline** — branch, ahead/behind, and modified counts; disappears when you leave the repo.
- **Six segments, your order** — toggle and reorder directory, git, context, rate-limits, dev-context, and model.
- **16 themes, 6 styles** — Tokyo Night, Catppuccin, Gruvbox, Nord, Dracula, Rose Pine, and more — in `powerline`, `plain`, `rounded`, `minimal`, `unicode`, or `ascii`.
- **A TUI to set it up** — `claudebar config`: toggle, reorder, theme, style, thresholds, all with a live preview.
- **Small and resilient** — one LTO-stripped binary; malformed JSON degrades to a clean line and keeps your prompt intact.

## Install

**Prerequisites**

- A [Nerd Font](https://www.nerdfonts.com/) set as your terminal font — for the glyphs
- `git` — for the git segment (optional; the segment just hides without it)
- `jq` — only if you already have a `~/.claude/settings.json` to merge into

```bash
curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh | bash
```

It installs the binary, wires up `~/.claude/settings.json` (backing up any existing file), and picks the best method automatically:

**prebuilt binary** (SHA256-verified) → **`cargo` build** (from a checkout) → **bash fallback**

Then **restart Claude Code** — the statusline appears on your next turn.

> [!TIP]
> Glyphs showing as boxes (□)? Install a [Nerd Font](https://www.nerdfonts.com/) and set it as your terminal font. See [Troubleshooting](#troubleshooting).

> [!NOTE]
> Prefer to read the script first? Download it with `curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh -o install.sh`, inspect it, then run `bash install.sh`.

<details>
<summary>Manual install</summary>

If you used the curl script above, skip this — it is already done.

**Prebuilt binary:** download the latest release for your platform from the [releases page](https://github.com/micschr0/claudebar/releases), extract, and place `claudebar` on your `$PATH` or at `~/.claude/claudebar`.

**Build with cargo:**

```bash
cargo install --git https://github.com/micschr0/claudebar
```

`cargo install` places the binary on your `PATH` (`~/.cargo/bin`), so add it with the bare command:

```json
{
  "statusLine": { "type": "command", "command": "claudebar render" }
}
```

The curl installer instead installs to `~/.claude/claudebar` and writes that full path automatically. For the bash fallback, point the command at `bash ~/.claude/statusline-command.sh` instead.

</details>

## What you see

claudebar renders these segments left to right (enable and reorder any of them):

| Segment | Shows |
|---------|-------|
| Directory | Fish-style abbreviated path |
| Git | Branch, ahead/behind, modified-file count |
| Context | Usage bar + token count, colored by threshold |
| Rate limits | 5-hour and weekly windows with a live reset countdown |
| Dev context | Worktree name, PR number + review state, sub-agent name |
| Model | Model name and effort level |

> [!NOTE]
> Rate-limit countdowns come from Claude Code's session JSON and appear on Pro/Max plans only.

## Screenshots

**Calm** — low usage, everything green:

<img src="screenshots/strip-green.png" width="860" alt="Calm state, all green">

**Normal** — typical mid-session usage:

<img src="screenshots/strip-normal.png" width="860" alt="Normal mid-session state">

**Critical** — context filling up, 5-hour limit tight, weekly window now shown:

<img src="screenshots/strip-critical.png" width="860" alt="Critical state with weekly rate limit">

**Over limit** — past 100% context, both bars red:

<img src="screenshots/strip-overlimit.png" width="860" alt="Over context limit">

**Outside a git repo** — the git segment drops out:

<img src="screenshots/strip-nogit.png" width="860" alt="Outside a git repository">

**Model without effort** — effort indicator omitted when the model has no effort param:

<img src="screenshots/strip-noeffort.png" width="860" alt="Model without effort indicator">

<details>
<summary>Easter egg</summary>

<img src="screenshots/skynet.png" width="860" alt="Easter egg">

</details>

## Configure

```bash
claudebar config
```

Toggle and reorder segments, pick a theme and style, and nudge thresholds — all with a live render preview. It saves changes to `~/.config/claudebar/config.toml`.

<img src="screenshots/tui.png" width="860" alt="Navigating the claudebar TUI configurator">

| Key | Action |
|-----|--------|
| `j` / `k` or ↑↓ | Move cursor |
| `Tab` / `Shift-Tab` | Next / previous section |
| `1`–`4` | Jump to section |
| `Space` | Toggle segment |
| `m` | Reorder mode |
| `h` / `l` or ←→ | Nudge threshold ±1 (`H` / `L` for ±5) |
| `s` · `r` · `?` · `q` | Save · Reset · Help · Quit |

Prefer editing by hand? The config is plain TOML:

```toml
theme = "tokyo-night"
style = "powerline"
segments = ["directory", "git", "context", "rate-limits", "dev-context", "model"]

[thresholds]
warn           = 50   # bar turns yellow at this %
crit           = 80   # bar turns red at this %
weekly_show_at = 50   # weekly window shows at this % and above
bar_width      = 6    # bar width in terminal cells
```

- **Themes (16):** `tokyo-night` (default) · `ayu-mirage` · `catppuccin` · `cobalt2` · `everforest-dark` · `github-dark` · `gruvbox` · `kanagawa-wave` · `moonfly` · `night-owl` · `nord` · `one-dark` · `dracula` · `rose-pine` · `sonokai` · `solarized-dark`
- **Styles (6):** `powerline` (default) · `plain` · `rounded` · `minimal` · `unicode` · `ascii`

The `--theme`, `--style`, and `--config` flags override the file for a single invocation.

> [!TIP]
> Upgraded from an older version? Run `claudebar migrate` to append newly added segments to your existing `config.toml`.

## CLI

| Command | What it does |
|---------|--------------|
| `claudebar` / `claudebar render` | Read session JSON from stdin, write the ANSI line to stdout |
| `claudebar config` | Launch the interactive TUI configurator |
| `claudebar init [--print] [--force]` | Write a default config file |
| `claudebar migrate` | Add new segments from a newer version to an existing config |
| `claudebar list` | Print all built-in theme and style names |

## Build from source

```bash
cargo build --release                       # binary at target/release/claudebar
cargo install --path .                       # install to ~/.cargo/bin
cargo build --release --no-default-features  # render-only, no TUI (smaller)
```

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| **Statusline is blank** | Check `~/.claude/settings.json` has `"statusLine": {"type": "command", ...}`, then restart Claude Code. |
| **Glyphs show as boxes (□)** | Install a [Nerd Font](https://www.nerdfonts.com/). macOS Terminal.app can't render Nerd Font PUA glyphs — use iTerm2, Kitty, WezTerm, Ghostty, or Alacritty. |
| **Git segment missing** | Appears only inside a git repo; needs `git` on your `PATH`. |
| **Rate-limit windows missing** | Pro/Max plans only; the weekly window shows once weekly usage is at or above `weekly_show_at` (default 50%). |
| **`command not found: claudebar`** | The curl installer places the binary at `~/.claude/claudebar`; `cargo install` places it in `~/.cargo/bin`. Use the full path in `settings.json`, or ensure that directory is on your `PATH`. |

## License

[MIT](LICENSE)
