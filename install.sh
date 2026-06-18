#!/usr/bin/env bash
# install.sh — one-command installer for claudebar
# Usage: curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh | bash
set -euo pipefail

REPO="https://raw.githubusercontent.com/micschr0/claudebar/main"
SCRIPT_DEST="$HOME/.claude/statusline-command.sh"
BIN_DEST="$HOME/.claude/claudebar"
SETTINGS="$HOME/.claude/settings.json"

# Directory this script lives in, when run from a checkout (empty under `curl | bash`).
SRC_DIR=""
case "$0" in
  */*) SRC_DIR=$(cd "$(dirname "$0")" 2>/dev/null && pwd || true) ;;
esac

red()   { printf '\033[31m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
bold()  { printf '\033[1m%s\033[0m\n' "$*"; }

tmp_script=
tmp_cfg=
trap 'rm -f "$tmp_script" "$tmp_cfg"' EXIT

# ── Preflight ──────────────────────────────────────────────────────────────────
bold "Checking dependencies..."
echo "  Requires a Nerd Font set as your terminal font — https://www.nerdfonts.com"
echo ""
install_hint() {
  case "$1" in
    jq)  echo "  macOS:  brew install jq" ;;
    git) echo "  macOS:  brew install git" ;;
  esac
  echo "  Linux:  sudo apt install $1   # or: sudo dnf install $1"
}

ok=true
for cmd in jq git; do
  if command -v "$cmd" >/dev/null 2>&1; then
    printf '  %-6s %s\n' "$cmd" "$(command -v "$cmd")"
  else
    red "  $cmd    not found"
    install_hint "$cmd"
    ok=false
  fi
done
if [ "$ok" = false ]; then
  echo ""
  red "Install the missing tools above, then re-run this script."
  exit 1
fi
echo ""

mkdir -p "$HOME/.claude"

# ── Install ────────────────────────────────────────────────────────────────────
# Preferred: the Rust binary, built from a local checkout when `cargo` is
# available. Fallback: download the standalone bash script (no toolchain needed).
COMMAND_VALUE=""

if [ -n "$SRC_DIR" ] && [ -f "$SRC_DIR/Cargo.toml" ] && command -v cargo >/dev/null 2>&1; then
  bold "Building the Rust binary with cargo..."
  if cargo install --path "$SRC_DIR" --root "$HOME/.claude" --force; then
    # `cargo install --root DIR` puts the binary in DIR/bin/<name>.
    if [ -x "$HOME/.claude/bin/claudebar" ]; then
      cp "$HOME/.claude/bin/claudebar" "$BIN_DEST"
    fi
    chmod +x "$BIN_DEST"
    COMMAND_VALUE="$BIN_DEST render"
    green "claudebar binary installed to $BIN_DEST"
  else
    red "cargo install failed — falling back to the bash script."
  fi
fi

if [ -z "$COMMAND_VALUE" ]; then
  # ── Download script ──────────────────────────────────────────────────────────
  printf 'Downloading statusline-command.sh ... '
  tmp_script=$(mktemp)
  curl -fsSL "$REPO/statusline-command.sh" -o "$tmp_script"
  if [ ! -s "$tmp_script" ]; then
    red "Download failed — file is empty"
    exit 1
  fi
  mv "$tmp_script" "$SCRIPT_DEST"
  chmod +x "$SCRIPT_DEST"
  green "done"
  COMMAND_VALUE="bash ~/.claude/statusline-command.sh"
fi

# ── Patch settings.json ────────────────────────────────────────────────────────
STATUS_LINE_VALUE=$(jq -nc --arg c "$COMMAND_VALUE" '{type:"command",command:$c}')

if [ -f "$SETTINGS" ]; then
  backup="${SETTINGS}.backup.$(date +%s)"
  cp "$SETTINGS" "$backup"
  printf 'Backed up settings.json to %s\n' "$backup"
  tmp_cfg=$(mktemp)
  if ! jq --argjson v "$STATUS_LINE_VALUE" '.statusLine = $v' "$SETTINGS" > "$tmp_cfg"; then
    red "$SETTINGS is not valid JSON — fix it manually before re-running."
    exit 1
  fi
  mv "$tmp_cfg" "$SETTINGS"
else
  jq -n --argjson v "$STATUS_LINE_VALUE" '{statusLine:$v}' > "$SETTINGS"
fi
green "settings.json updated"

# ── Done ───────────────────────────────────────────────────────────────────────
echo ""
bold "Installation complete."
echo "Restart Claude Code — the claudebar appears on the next turn."
echo ""
echo "If glyphs show as boxes, install a Nerd Font and set it as your terminal font."
echo "Troubleshooting: https://github.com/micschr0/claudebar#troubleshooting"
