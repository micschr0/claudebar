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

TMPDIR_WORK=$(mktemp -d)
trap 'rm -rf "$TMPDIR_WORK"' EXIT

red()   { printf '\033[31m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
bold()  { printf '\033[1m%s\033[0m\n' "$*"; }

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

# ── Helper functions ───────────────────────────────────────────────────────────

# detect_target — print the Rust target triple for the current OS/arch, or
# empty string if no prebuilt binary is available for this platform.
detect_target() {
  local os arch
  os=$(uname -s)
  arch=$(uname -m)
  case "$arch" in
    arm64|aarch64) arch="aarch64" ;;
    x86_64)        arch="x86_64"  ;;
    *) printf ''; return ;;
  esac
  case "$os" in
    Linux)  printf '%s-unknown-linux-musl' "$arch" ;;
    Darwin) printf '%s-apple-darwin'       "$arch" ;;
    *)      printf '' ;;
  esac
}

# fetch_latest_tag — print the latest GitHub release tag, or empty string if
# no release exists yet.
fetch_latest_tag() {
  local tag
  tag=$(curl -fsSL "https://api.github.com/repos/micschr0/claudebar/releases/latest" \
        | jq -r '.tag_name // empty')
  printf '%s' "$tag"
}

# sha256_of <file> — print the SHA256 hex digest of <file>.
# Uses sha256sum (Linux) or shasum -a 256 (macOS).
sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

# verify_checksum <file> <archive_name> <sums_file> — verify <file> against
# the SHA256SUMS.txt entry for <archive_name>. Returns 1 on mismatch (fatal).
verify_checksum() {
  local file="$1" archive_name="$2" sums_file="$3"
  local expected actual
  expected=$(grep "  ${archive_name}$" "$sums_file" | awk '{print $1}')
  if [ -z "$expected" ]; then
    red "No checksum entry for ${archive_name} in SHA256SUMS.txt"
    return 1
  fi
  actual=$(sha256_of "$file")
  if [ "$actual" != "$expected" ]; then
    red "SHA256 mismatch — aborting (download may be tampered or corrupted)"
    red "  Expected: ${expected}"
    red "  Actual:   ${actual}"
    return 1
  fi
  green "SHA256 verified"
  return 0
}

# download_prebuilt <target> — download the prebuilt binary for <target>,
# verify its SHA256 checksum, extract it, and install to BIN_DEST.
# Returns 0 on success, 1 on any recoverable failure (triggers Tier 2).
# Checksum mismatch is fatal and does NOT trigger a fallback.
download_prebuilt() {
  local target="$1"
  local tag archive url sums_url
  tag=$(fetch_latest_tag)
  if [ -z "$tag" ]; then
    bold "No prebuilt release found — falling back to cargo build"
    return 1
  fi

  archive="claudebar-${tag}-${target}.tar.gz"
  url="https://github.com/micschr0/claudebar/releases/download/${tag}/${archive}"
  sums_url="https://github.com/micschr0/claudebar/releases/download/${tag}/SHA256SUMS.txt"

  bold "Downloading ${archive}..."
  if ! curl -fsSL "$url" -o "${TMPDIR_WORK}/${archive}"; then
    red "Download failed — binary may not exist for this release yet"
    bold "Falling back to cargo build..."
    return 1
  fi

  curl -fsSL "$sums_url" -o "${TMPDIR_WORK}/SHA256SUMS.txt"
  if ! verify_checksum "${TMPDIR_WORK}/${archive}" "$archive" "${TMPDIR_WORK}/SHA256SUMS.txt"; then
    exit 1
  fi

  tar -xf "${TMPDIR_WORK}/${archive}" -C "${TMPDIR_WORK}/"
  mv "${TMPDIR_WORK}/claudebar" "$BIN_DEST"
  chmod +x "$BIN_DEST"
  green "claudebar ${tag} installed to ${BIN_DEST}"
  return 0
}

# ── Install ────────────────────────────────────────────────────────────────────
COMMAND_VALUE=""

# Tier 1: Prebuilt binary
TARGET=$(detect_target)
if [ -n "$TARGET" ]; then
  if download_prebuilt "$TARGET"; then
    COMMAND_VALUE="${BIN_DEST} render"
  fi
else
  bold "No prebuilt binary for $(uname -s)/$(uname -m) — skipping download"
fi

# Tier 2: Cargo build (local checkout only — not available under curl | bash)
if [ -z "$COMMAND_VALUE" ]; then
  if [ -n "$SRC_DIR" ] && [ -f "${SRC_DIR}/Cargo.toml" ] && command -v cargo >/dev/null 2>&1; then
    bold "Building from source with cargo..."
    if cargo install --path "$SRC_DIR" --root "$HOME/.claude" --force; then
      if [ -x "$HOME/.claude/bin/claudebar" ]; then
        cp "$HOME/.claude/bin/claudebar" "$BIN_DEST"
      fi
      chmod +x "$BIN_DEST"
      COMMAND_VALUE="${BIN_DEST} render"
      green "claudebar built and installed to ${BIN_DEST}"
    else
      red "cargo build failed — falling back to statusline-command.sh"
    fi
  elif [ -z "$SRC_DIR" ]; then
    bold "No local checkout detected — cargo install requires source; falling back to statusline-command.sh"
  else
    bold "cargo not found — falling back to statusline-command.sh"
  fi
fi

# Tier 3: Bash script fallback
if [ -z "$COMMAND_VALUE" ]; then
  bold "Installing statusline-command.sh (bash fallback — requires jq + git at runtime)"
  curl -fsSL "$REPO/statusline-command.sh" -o "${TMPDIR_WORK}/statusline-command.sh"
  if [ ! -s "${TMPDIR_WORK}/statusline-command.sh" ]; then
    red "Download failed — file is empty"
    exit 1
  fi
  mv "${TMPDIR_WORK}/statusline-command.sh" "$SCRIPT_DEST"
  chmod +x "$SCRIPT_DEST"
  green "statusline-command.sh installed to ${SCRIPT_DEST}"
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
