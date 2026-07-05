#!/usr/bin/env bash
# install.sh — one-command installer for claudebar
# Usage: curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh | bash
set -euo pipefail

BIN_DEST="$HOME/.claude/claudebar"

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

# curl_https — curl wrapper pinned to HTTPS + TLS 1.2+ so a redirect or
# MITM can't downgrade the connection to plaintext or an older TLS version.
curl_https() { curl --proto '=https' --tlsv1.2 -fsSL "$@"; }

# ── Preflight ──────────────────────────────────────────────────────────────────
# git is a soft runtime dependency (the git segment). curl and jq are needed for
# the prebuilt binary download path (fetch_latest_tag, download_prebuilt).
bold "Checking dependencies..."
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
  red "  jq     not found — prebuilt binary download will fail"
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
# CLAUDEBAR_TARGET overrides detection (used by verify-install.yml to exercise
# the aarch64 asset from an x86_64 runner without needing emulation).
detect_target() {
  if [ -n "${CLAUDEBAR_TARGET:-}" ]; then
    printf '%s' "$CLAUDEBAR_TARGET"
    return
  fi
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
    LATEST_RELEASE_JSON=$(curl_https "https://api.github.com/repos/micschr0/claudebar/releases/latest")
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

# require_github_host <url> — abort if <url> does not point at github.com.
# The GitHub API response is otherwise trusted blindly for download URLs; this
# guards against a compromised/spoofed API response redirecting the install
# to an arbitrary host.
require_github_host() {
  case "$1" in
    https://github.com/*) ;;
    *)
      red "Refusing to download from untrusted host: $1"
      exit 1
      ;;
  esac
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
# Matches both sha256sum output formats: text mode ("hash  name") and
# binary mode ("hash *name") — cargo-dist emits the latter.
verify_checksum() {
  local file="$1" archive_name="$2" sums_file="$3"
  local expected actual
  expected=$(awk -v name="$archive_name" \
    '{ f = $2; sub(/^\*/, "", f); if (f == name) { print $1; exit } }' \
    "$sums_file")
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
# Returns 0 on success, 1 on any recoverable failure (falls through to Tier 2).
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
  url=$(find_asset_url "$release" "^claudebar-${target}\\.tar\\.gz$")
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
  require_github_host "$url"
  require_github_host "$sums_url"
  archive="${url##*/}"

  bold "Downloading ${archive}..."
  if ! curl_https "$url" -o "${TMPDIR_WORK}/${archive}"; then
    red "Download failed — binary may not exist for this release yet"
    bold "Falling back to cargo build..."
    return 1
  fi

  curl_https "$sums_url" -o "${TMPDIR_WORK}/sha256.sum"
  if ! verify_checksum "${TMPDIR_WORK}/${archive}" "$archive" "${TMPDIR_WORK}/sha256.sum"; then
    exit 1
  fi

  # Defense-in-depth: reject archives containing path-traversal or absolute
  # entries before extracting, even though the checksum above already ties
  # the archive to the signed release.
  if tar -tf "${TMPDIR_WORK}/${archive}" | grep -qE '(^|/)\.\.(/|$)|^/'; then
    red "Archive ${archive} contains unsafe paths (.. or absolute) — aborting"
    exit 1
  fi

  tar --no-same-owner -xf "${TMPDIR_WORK}/${archive}" -C "${TMPDIR_WORK}/"
  mv "${TMPDIR_WORK}/claudebar" "$BIN_DEST"
  chmod +x "$BIN_DEST"
  green "claudebar ${tag} installed to ${BIN_DEST}"
  return 0
}

# link_onto_path — make `claudebar` runnable from any shell. The binary lives at
# $BIN_DEST (~/.claude/claudebar), which is not on PATH, so `claudebar
# config|doctor|edit|list` would be "command not found". The statusLine uses the
# absolute $BIN_DEST path and is unaffected either way. Skips entirely if
# `claudebar` already resolves (e.g. a Homebrew install) so we never shadow it.
link_onto_path() {
  if command -v claudebar >/dev/null 2>&1; then
    return 0
  fi

  # Prefer a directory already on PATH and writable — then the command works
  # immediately, no shell-rc edit needed.
  local dir target=""
  for dir in "/usr/local/bin" "$HOME/.local/bin" "$HOME/bin"; do
    case ":$PATH:" in *":$dir:"*) ;; *) continue ;; esac
    if [ -d "$dir" ] && [ -w "$dir" ]; then
      target="$dir"
      break
    fi
  done

  if [ -n "$target" ]; then
    ln -sf "$BIN_DEST" "$target/claudebar"
    green "Linked claudebar onto PATH: ${target}/claudebar"
    return 0
  fi

  # Nothing writable on PATH — link into ~/.local/bin and say how to reach it.
  target="$HOME/.local/bin"
  mkdir -p "$target"
  ln -sf "$BIN_DEST" "$target/claudebar"
  green "Linked claudebar → ${target}/claudebar"
  red "  ${target} is not on your PATH — the 'claudebar' command won't resolve yet."
  echo "  Add it:  echo 'export PATH=\"${target}:\$PATH\"' >> ~/.zshrc   # or ~/.bashrc, then restart your shell"
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
      red "cargo build failed."
    fi
  elif [ -z "$SRC_DIR" ]; then
    bold "No local checkout detected — cargo build requires source."
  else
    bold "cargo not found."
  fi
fi

if [ -z "$COMMAND_VALUE" ]; then
  red "No installation method succeeded (no prebuilt binary for this platform and no local cargo build available)."
  echo "Install Rust (https://rustup.rs) and re-run from a checkout, or wait for a prebuilt release for your platform."
  exit 1
fi

# ── Configure statusLine ─────────────────────────────────────────────────────────
"$BIN_DEST" setup --yes --binary-path "$BIN_DEST"
link_onto_path

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
echo "Troubleshooting: run 'claudebar doctor', or visit https://micschr0.github.io/claudebar/"
