<div align="center">

<img src="assets/logo.svg" width="300" alt="claudebar">

**A fast, themeable statusline for Claude Code.**

[![CI](https://github.com/micschr0/claudebar/actions/workflows/rust.yml/badge.svg)](https://github.com/micschr0/claudebar/actions/workflows/rust.yml)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![~5× faster than bash](https://img.shields.io/badge/render-~5%C3%97_faster_than_bash-9ece6a)

<img src="screenshots/intro.png" width="820" alt="claudebar statusline demo (rendered)">

<sub><i>Rendered demo for illustration.</i></sub>

</div>

<div align="center">

<img src="screenshots/strip-green.png" width="857" alt="Calm state — low usage, everything green">
<img src="screenshots/strip-normal.png" width="832" alt="Normal mid-session usage">
<img src="screenshots/strip-critical.png" width="1009" alt="Critical — context high, 5-hour limit tight, weekly window shown">
<img src="screenshots/strip-overlimit.png" width="874" alt="Over limit — past 100% context, both bars red">
<img src="screenshots/strip-nogit.png" width="756" alt="Outside a git repo — git segment drops out">
<img src="screenshots/strip-noeffort.png" width="781" alt="Model without an effort param — effort indicator omitted">

</div>

## Features

- Live rate-limit countdowns
- Color-coded context usage
- Inline git state
- 16 themes · 6 styles
- [~5× faster than bash scripts](scripts/benchmark.sh) (~30ms vs ~200ms)
- Read-only — never touches your session
- Tiny ~1.5 MB dependency-free binary

## Install

**Prerequisites**

- A [Nerd Font](https://www.nerdfonts.com/) set as your terminal font — for the glyphs
- `git` — for the git segment (optional; the segment just hides without it)
- `jq` — only if you already have a `~/.claude/settings.json` to merge into

```bash
curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh | bash
```

It installs the binary and wires up `~/.claude/settings.json` (backing up any existing file). Then **restart Claude Code** — the statusline appears on your next turn.

**Where it hooks in:** Claude Code reads the `statusLine` key in `~/.claude/settings.json`. The installer adds this for you:

```json
{
  "statusLine": { "type": "command", "command": "~/.claude/claudebar render" }
}
```

On every turn Claude Code runs that command, feeds it the session JSON on stdin, and prints whatever it writes to stdout as your statusline. (`cargo install` users use the bare `claudebar render`; the bash fallback uses `bash ~/.claude/statusline-command.sh`.)

<details>
<summary>Manual install</summary>

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

## Configure

Once installed, launch the configurator — this is the simplest way, there's no separate app or flag to remember:

```bash
claudebar config
```

Toggle and reorder segments, pick a theme and style, and nudge thresholds — all with a live render preview. It saves changes to `~/.config/claudebar/config.toml`. Press `?` inside for key bindings, `s` to save, `q` to quit.

> If you installed with the curl script, the binary lives at `~/.claude/claudebar`, so call `~/.claude/claudebar config` (or add `~/.claude` to your `PATH`).

<img src="screenshots/tui.png" width="860" alt="Navigating the claudebar TUI configurator">

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

Run `claudebar list` to see all built-in themes and styles. The `--theme`, `--style`, and `--config` flags override the file for a single invocation.

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
