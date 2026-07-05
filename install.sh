#!/usr/bin/env bash
# install.sh — one-command installer for claudebar
# Usage: curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh | bash
set -euo pipefail

BIN_DEST="$HOME/.claude/claudebar"
RELEASE_API="https://api.github.com/repos/micschr0/claudebar/releases/latest"

red()   { printf '\033[31m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
bold()  { printf '\033[1m%s\033[0m\n' "$*"; }

# Pinned to HTTPS + TLS 1.2+ so a redirect or MITM can't downgrade the connection.
curl_https() { curl --proto '=https' --tlsv1.2 -fsSL "$@"; }

install_hint() {
  case "$1" in
    curl) echo "  macOS:  brew install curl" ;;
    git)  echo "  macOS:  brew install git" ;;
    jq)   echo "  macOS:  brew install jq" ;;
  esac
  echo "  Linux:  sudo apt install $1   # or: sudo dnf install $1"
}

report_dependency() {
  local name="$1" consequence="$2"
  if command -v "$name" >/dev/null 2>&1; then
    printf '  %-6s %s\n' "$name" "$(command -v "$name")"
  else
    printf '\033[31m  %-6s not found — %s\033[0m\n' "$name" "$consequence"
    install_hint "$name"
  fi
}

check_dependencies() {
  bold "Checking dependencies..."
  echo ""
  report_dependency git  "the git segment stays hidden until you install it"
  report_dependency curl "prebuilt binary download will fail"
  report_dependency jq   "prebuilt binary download will fail"
  echo ""
}

detect_source_dir() {
  case "$0" in
    */*) cd "$(dirname "$0")" 2>/dev/null && pwd || true ;;
    *)   [ -f "${PWD}/Cargo.toml" ] && printf '%s' "$PWD" || true ;;
  esac
}

# Prints the Rust target triple for this OS/arch, or nothing if unsupported.
# CLAUDEBAR_TARGET overrides detection (used by verify-install.yml).
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
    *) return ;;
  esac
  case "$os" in
    Linux)  printf '%s-unknown-linux-musl' "$arch" ;;
    Darwin) printf '%s-apple-darwin'       "$arch" ;;
  esac
}

LATEST_RELEASE_JSON=""
fetch_latest_release() {
  if [ -z "$LATEST_RELEASE_JSON" ]; then
    LATEST_RELEASE_JSON=$(curl_https "$RELEASE_API")
  fi
  printf '%s' "$LATEST_RELEASE_JSON"
}

release_tag() {
  printf '%s' "$1" | jq -r '.tag_name // empty'
}

# Resolves asset names by pattern instead of hand-building filenames — the
# naming scheme is owned by the release workflow (cargo-dist) and has changed before.
find_asset_url() {
  local json="$1" regex="$2"
  printf '%s' "$json" | jq -r --arg re "$regex" \
    '[.assets[] | select(.name | test($re))][0].browser_download_url // empty'
}

# Download URLs come from the GitHub API response, which is otherwise trusted
# blindly — reject anything not hosted on github.com.
require_github_host() {
  case "$1" in
    https://github.com/*) return 0 ;;
    *)
      red "Refusing to download from untrusted host: $1"
      return 1
      ;;
  esac
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

# Accepts both sha256sum output formats: "hash  name" and "hash *name".
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
}

# Build-provenance check via GitHub artifact attestations. Defense-in-depth on
# top of the mandatory SHA256 gate — informative, never fatal. gh needs auth
# even for public repos, so unauthenticated gh counts as skipped, not failed.
verify_attestation() {
  local file="$1"
  if ! command -v gh >/dev/null 2>&1; then
    bold "Provenance check skipped (gh CLI not installed)"
    return 0
  fi
  if ! gh auth status >/dev/null 2>&1; then
    bold "Provenance check skipped (gh CLI not authenticated)"
    return 0
  fi
  bold "Verifying build provenance (gh attestation verify)..."
  if gh attestation verify "$file" --repo micschr0/claudebar >/dev/null 2>&1; then
    green "Build provenance verified — artifact was built by this repo's release workflow"
  else
    red "Provenance verification failed or no attestation found — continuing (SHA256 already verified)"
  fi
  return 0
}

archive_has_unsafe_paths() {
  tar -tf "$1" | grep -qE '(^|/)\.\.(/|$)|^/'
}

extract_archive() {
  local archive="$1" dest="$2"
  if archive_has_unsafe_paths "$archive"; then
    red "Archive contains unsafe paths (.. or absolute) — aborting"
    return 1
  fi
  tar --no-same-owner -xf "$archive" -C "$dest"
}

install_binary() {
  local src="$1"
  mv "$src" "$BIN_DEST"
  chmod +x "$BIN_DEST"
}

# Returns 0 on success, 1 on any recoverable failure (falls through to the
# cargo build tier). Checksum mismatch or an untrusted host is fatal.
install_prebuilt() {
  local target="$1" workdir="$2"
  local release tag archive url sums_url

  release=$(fetch_latest_release)
  tag=$(release_tag "$release")
  if [ -z "$tag" ]; then
    bold "No prebuilt release found — falling back to cargo build"
    return 1
  fi

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
  require_github_host "$url" || exit 1
  require_github_host "$sums_url" || exit 1
  archive="${url##*/}"

  bold "Downloading ${archive}..."
  if ! curl_https "$url" -o "${workdir}/${archive}"; then
    red "Download failed — binary may not exist for this release yet"
    bold "Falling back to cargo build..."
    return 1
  fi
  curl_https "$sums_url" -o "${workdir}/sha256.sum"
  verify_checksum "${workdir}/${archive}" "$archive" "${workdir}/sha256.sum" || exit 1
  verify_attestation "${workdir}/${archive}"

  extract_archive "${workdir}/${archive}" "$workdir" || exit 1
  install_binary "${workdir}/claudebar"
  green "claudebar ${tag} installed to ${BIN_DEST}"
}

install_from_source() {
  local src_dir="$1"
  if [ -z "$src_dir" ]; then
    bold "No local checkout detected — cargo build requires source."
    return 1
  fi
  if [ ! -f "${src_dir}/Cargo.toml" ]; then
    bold "No Cargo.toml in ${src_dir} — cannot build from source."
    return 1
  fi
  if ! command -v cargo >/dev/null 2>&1; then
    bold "cargo not found."
    return 1
  fi
  bold "Building from source with cargo..."
  if ! cargo build --release --manifest-path "${src_dir}/Cargo.toml"; then
    red "cargo build failed."
    return 1
  fi
  install -m 0755 "${src_dir}/target/release/claudebar" "$BIN_DEST"
  green "claudebar built and installed to ${BIN_DEST}"
}

# Makes `claudebar` runnable from any shell. Skips if it already resolves
# (e.g. a Homebrew install) so we never shadow it.
link_onto_path() {
  if command -v claudebar >/dev/null 2>&1; then
    return 0
  fi

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

  target="$HOME/.local/bin"
  mkdir -p "$target"
  ln -sf "$BIN_DEST" "$target/claudebar"
  green "Linked claudebar → ${target}/claudebar"
  red "  ${target} is not on your PATH — the 'claudebar' command won't resolve yet."
  echo "  Add it:  echo 'export PATH=\"${target}:\$PATH\"' >> ~/.zshrc   # or ~/.bashrc, then restart your shell"
}

check_nerd_font() {
  if command -v fc-list >/dev/null 2>&1; then
    if fc-list :family 2>/dev/null | grep -qi 'nerd'; then
      return 0
    fi
  fi
  local dir
  for dir in "/usr/share/fonts" "/usr/local/share/fonts" "$HOME/.local/share/fonts" "$HOME/.fonts"; do
    if [ -d "$dir" ] && find "$dir" -maxdepth 1 \( -name '*Nerd*' -o -name '*nerd*' \) \( -name '*.ttf' -o -name '*.otf' \) -print -quit 2>/dev/null | grep -q .; then
      return 0
    fi
  done
  return 1
}

report_nerd_font() {
  if check_nerd_font; then
    green "✓ Nerd Font detected"
  else
    red "⚠ No Nerd Font detected. The statusline uses powerline glyphs — install a Nerd Font for best results: https://www.nerdfonts.com"
    if [ "$(uname -s)" = "Darwin" ]; then
      echo "  macOS:  brew install --cask font-hack-nerd-font"
    fi
    echo "Tip: run 'claudebar config' and choose the 'ascii' style for glyph-free rendering."
  fi
}

main() {
  local workdir src_dir target installed=1

  check_dependencies
  mkdir -p "$HOME/.claude"

  workdir=$(mktemp -d)
  trap 'rm -rf "$workdir"' EXIT
  src_dir=$(detect_source_dir)

  target=$(detect_target)
  if [ -n "$target" ]; then
    if install_prebuilt "$target" "$workdir"; then
      installed=0
    fi
  else
    bold "No prebuilt binary for $(uname -s)/$(uname -m) — skipping download"
  fi

  if [ "$installed" -ne 0 ]; then
    if install_from_source "$src_dir"; then
      installed=0
    fi
  fi

  if [ "$installed" -ne 0 ]; then
    red "No installation method succeeded (no prebuilt binary for this platform and no local cargo build available)."
    echo "Install Rust (https://rustup.rs) and re-run from a checkout, or wait for a prebuilt release for your platform."
    exit 1
  fi

  "$BIN_DEST" setup --yes --binary-path "$BIN_DEST"
  link_onto_path

  echo ""
  bold "Installation complete."
  echo "Restart Claude Code — claudebar appears on the next turn."
  echo ""
  report_nerd_font
  echo ""
  echo "Troubleshooting: run 'claudebar doctor', or visit https://micschr0.github.io/claudebar/"
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
