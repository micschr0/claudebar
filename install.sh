#!/usr/bin/env bash
# install.sh — one-command installer for claudebar
# Usage: curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh | bash
set -euo pipefail

REPO="https://raw.githubusercontent.com/micschr0/claudebar/main"
SCRIPT_DEST="$HOME/.claude/statusline-command.sh"
BIN_DEST="$HOME/.claude/claudebar"
SETTINGS="${SETTINGS:-$HOME/.claude/settings.json}"

# Directory this script lives in, when run from a checkout (empty under `curl | bash`).
SRC_DIR=""
case "$0" in
  */*) SRC_DIR=$(cd "$(dirname "$0")" 2>/dev/null && pwd || true) ;;
  *)   [ -f "${PWD}/Cargo.toml" ] && SRC_DIR="$PWD" ;;
esac

TMPDIR_WORK=$(mktemp -d)
trap 'rm -rf "$TMPDIR_WORK"' EXIT

red()   { printf '\033[31m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
bold()  { printf '\033[1m%s\033[0m\n' "$*"; }

# ── Preflight ──────────────────────────────────────────────────────────────────
# git is a soft runtime dependency (the git segment). curl and jq are needed for
# the prebuilt binary download path (fetch_latest_tag, download_prebuilt) and are
# also required for the bash fallback and settings.json merge respectively.
# The happy path (prebuilt binary) needs curl + jq. Nothing else is required.
bold "Checking dependencies..."
echo "  Tip: set a Nerd Font as your terminal font for the glyphs — https://www.nerdfonts.com"
echo ""
install_hint() {
  case "$1" in
    curl) echo "  macOS:  brew install curl" ;;
    git)  echo "  macOS:  brew install git" ;;
    jq)   echo "  macOS:  brew install jq" ;;
  esac
  echo "  Linux:  sudo apt install $1   # or: sudo dnf install $1"
}

if command -v git >/dev/null 2>&1; then
  printf '  %-6s %s\n' "git" "$(command -v git)"
else
  red "  git    not found — the git segment stays hidden until you install it"
  install_hint git
fi

if command -v curl >/dev/null 2>&1; then
  printf '  %-6s %s\n' "curl" "$(command -v curl)"
else
  red "  curl   not found — prebuilt binary download will fail"
  install_hint curl
fi

if command -v jq >/dev/null 2>&1; then
  printf '  %-6s %s\n' "jq" "$(command -v jq)"
else
  red "  jq     not found — prebuilt binary download and settings merge will fail"
  install_hint jq
fi
echo ""

mkdir -p "$HOME/.claude"

# ── Helper functions ───────────────────────────────────────────────────────────

# check_nerd_font — return 0 if a Nerd Font is installed, 1 otherwise.
# Uses fc-list when available; falls back to scanning common font directories.
check_nerd_font() {
    # Try fc-list first — fastest and most accurate.
    if command -v fc-list >/dev/null 2>&1; then
        if fc-list :family 2>/dev/null | grep -qi 'nerd'; then
            return 0
        fi
    fi

    # Fallback: scan common font directories (non-recursive, matching main.rs).
    local dir
    for dir in "/usr/share/fonts" "/usr/local/share/fonts" "$HOME/.local/share/fonts" "$HOME/.fonts"; do
        if [ -d "$dir" ] && find "$dir" -maxdepth 1 \( -name '*Nerd*' -o -name '*nerd*' \) \( -name '*.ttf' -o -name '*.otf' \) -print -quit 2>/dev/null | grep -q .; then
            return 0
        fi
    done

    return 1
}
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

# fetch_latest_release — print the full "latest" release JSON, or empty
# string if no release exists yet. Cached in LATEST_RELEASE_JSON so callers
# that need both the tag and its asset list only hit the API once.
LATEST_RELEASE_JSON=""
fetch_latest_release() {
  if [ -z "$LATEST_RELEASE_JSON" ]; then
    LATEST_RELEASE_JSON=$(curl -fsSL "https://api.github.com/repos/micschr0/claudebar/releases/latest")
  fi
  printf '%s' "$LATEST_RELEASE_JSON"
}

# fetch_latest_tag — print the latest GitHub release tag, or empty string if
# no release exists yet.
fetch_latest_tag() {
  fetch_latest_release | jq -r '.tag_name // empty'
}

# find_asset_url <json> <regex> — print the browser_download_url of the first
# release asset whose name matches <regex> (extended regex, jq `test`), or
# empty string if none match. Resolving by pattern instead of hand-building
# the filename keeps this in sync with whatever naming scheme the release
# workflow (cargo-dist) actually uses.
find_asset_url() {
  local json="$1" regex="$2"
  printf '%s' "$json" | jq -r --arg re "$regex" \
    '[.assets[] | select(.name | test($re))][0].browser_download_url // empty'
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
# the sha256.sum entry for <archive_name>. Returns 1 on mismatch (fatal).
verify_checksum() {
  local file="$1" archive_name="$2" sums_file="$3"
  local expected actual
  expected=$(grep "  ${archive_name}$" "$sums_file" | awk '{print $1}')
  if [ -z "$expected" ]; then
    red "No checksum entry for ${archive_name} in sha256.sum"
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
  local release tag archive url sums_url
  release=$(fetch_latest_release)
  tag=$(printf '%s' "$release" | jq -r '.tag_name // empty')
  if [ -z "$tag" ]; then
    bold "No prebuilt release found — falling back to cargo build"
    return 1
  fi

  # Resolve the actual asset names from the release instead of hand-building
  # them — the naming scheme is owned by the release workflow (cargo-dist),
  # not by this script, and has changed before (tag embedded in the archive
  # name, sums file renamed from sha256.sum to SHA256SUMS.txt).
  url=$(find_asset_url "$release" "^claudebar-.*-${target}\\.tar\\.gz$")
  sums_url=$(find_asset_url "$release" '^(SHA256SUMS\.txt|sha256\.sum)$')
  if [ -z "$url" ]; then
    red "No release asset found for target ${target} in ${tag}"
    bold "Falling back to cargo build..."
    return 1
  fi
  if [ -z "$sums_url" ]; then
    red "No checksum file found in release ${tag}"
    bold "Falling back to cargo build..."
    return 1
  fi
  archive="${url##*/}"

  bold "Downloading ${archive}..."
  if ! curl -fsSL "$url" -o "${TMPDIR_WORK}/${archive}"; then
    red "Download failed — binary may not exist for this release yet"
    bold "Falling back to cargo build..."
    return 1
  fi

  curl -fsSL "$sums_url" -o "${TMPDIR_WORK}/sha256.sum"
  if ! verify_checksum "${TMPDIR_WORK}/${archive}" "$archive" "${TMPDIR_WORK}/sha256.sum"; then
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
    if cargo build --release --manifest-path "${SRC_DIR}/Cargo.toml"; then
      install -m 0755 "${SRC_DIR}/target/release/claudebar" "$BIN_DEST"
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
# Merging into an existing config requires jq (hand-rolling a JSON merge is
# unsafe). A fresh install writes the file directly — no jq needed.
if [ -f "$SETTINGS" ]; then
  if ! command -v jq >/dev/null 2>&1; then
    red "$SETTINGS already exists but jq is not installed."
    echo "Install jq, or add this manually to $SETTINGS:"
    printf '  "statusLine": {"type":"command","command":"%s"}\n' "$COMMAND_VALUE"
    exit 1
  fi
  backup="${SETTINGS}.backup.$(date +%s)"
  cp "$SETTINGS" "$backup"
  printf 'Backed up settings.json to %s\n' "$backup"
  status_line=$(jq -nc --arg c "$COMMAND_VALUE" '{type:"command",command:$c}')
  tmp_cfg=$(mktemp)
  if ! jq --argjson v "$status_line" '.statusLine = $v' "$SETTINGS" > "$tmp_cfg"; then
    red "$SETTINGS is not valid JSON — fix it manually before re-running."
    rm -f "$tmp_cfg"
    exit 1
  fi
  mv "$tmp_cfg" "$SETTINGS"
else
  cat > "$SETTINGS" <<EOF
{
  "statusLine": { "type": "command", "command": "$COMMAND_VALUE" }
}
EOF
fi
green "settings.json updated"

# ── Done ───────────────────────────────────────────────────────────────────────
echo ""
bold "Installation complete."
echo "Restart Claude Code — claudebar appears on the next turn."
echo ""
# Nerd Font check (non-blocking — always runs after install)
if check_nerd_font; then
    green "✓ Nerd Font detected"
else
    red "⚠ No Nerd Font detected. The statusline uses powerline glyphs — install a Nerd Font for best results: https://www.nerdfonts.com"
    if [ "$(uname -s)" = "Darwin" ]; then
        echo "  macOS:  brew install --cask font-hack-nerd-font"
    fi
    echo "Tip: run 'claudebar config' and choose the 'ascii' style for glyph-free rendering."
fi
echo ""
echo "Troubleshooting: https://github.com/micschr0/claudebar#troubleshooting"
