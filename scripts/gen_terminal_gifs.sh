#!/usr/bin/env bash
# gen_terminal_gifs.sh — record REAL terminal GIFs of claudebar and render them.
#
# Produces (transparent APNG via window_frame.py):
#   screenshots/intro.png  — the easter-egg transcript with a loading spinner (hero)
#   screenshots/tui.png    — navigating the `claudebar config` TUI
#
# Unlike gen_screenshots.py (which rebuilds the terminal in HTML), this records an
# actual PTY session with asciinema and rasterises it with agg, so every frame is
# genuine terminal output rendered with the real Nerd Font.
#
# Prerequisites:
#   - cargo build --release           (binary at target/release/claudebar)
#   - asciinema   (pip install asciinema)
#   - agg         (cargo install --git https://github.com/asciinema/agg)
#   - tmux
#   - Hack Nerd Font Mono in $NF_FONT_DIR (default /tmp/fonts)
set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
SHOTS="$REPO/screenshots"
NF_FONT_DIR="${NF_FONT_DIR:-/tmp/fonts}"
BIN="$REPO/target/release/claudebar"

# tokyo-night palette: bg,fg, then 16 ANSI slots
THEME="1a1b2e,c0caf5,15161e,f7768e,9ece6a,e0af68,7aa2f7,bb9af7,7dcfff,a9b1d6,414868,f7768e,9ece6a,e0af68,7aa2f7,bb9af7,7dcfff,c0caf5"
AGG=(agg --font-dir "$NF_FONT_DIR" --font-family "Hack Nerd Font Mono" --theme "$THEME")

mkdir -p "$SHOTS"

echo "── intro.png ──"
asciinema rec --cols 94 --rows 20 --overwrite \
  -c "python3 $REPO/scripts/demo_intro.py" /tmp/cb_intro.cast
"${AGG[@]}" --font-size 28 /tmp/cb_intro.cast /tmp/cb_intro_raw.gif
python3 "$REPO/scripts/window_frame.py" /tmp/cb_intro_raw.gif "$SHOTS/intro.png" \
  "claude — /var/skynet/defense-net/missile-command/launch" --no-hold

echo "── tui.png ──"
tmux kill-session -t cbgif 2>/dev/null || true
tmux new-session -d -s cbgif -x 100 -y 30 "TERM=xterm-256color $BIN config"
tmux set -t cbgif -g status off          # hide tmux's own status bar
tmux set -t cbgif -g remain-on-exit off
( sleep 2.2
  for k in 2 3 4 1; do tmux send-keys -t cbgif "$k"; sleep 2.4; done
  tmux send-keys -t cbgif '?'; sleep 2.4
  tmux send-keys -t cbgif Escape; sleep 0.8
  tmux send-keys -t cbgif q ) &
asciinema rec --cols 100 --rows 30 --overwrite \
  -c "tmux attach -t cbgif" /tmp/cb_tui.cast
"${AGG[@]}" --font-size 20 /tmp/cb_tui.cast /tmp/cb_tui_raw.gif
python3 "$REPO/scripts/window_frame.py" /tmp/cb_tui_raw.gif "$SHOTS/tui.png" "claudebar config"

echo "Done: $SHOTS/intro.png, $SHOTS/tui.png"
