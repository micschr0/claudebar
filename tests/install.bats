#!/usr/bin/env bats
# Bats coverage for install.sh. The script guards its entry point
# (main runs only when executed, not sourced), so tests source it directly
# and exercise individual functions. The networked happy path is covered
# by .github/workflows/verify-install.yml.

setup() {
  REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/.." && pwd)"
  INSTALL="$REPO_ROOT/install.sh"
  source "$INSTALL"
  cd "$BATS_TEST_TMPDIR"
}

@test "install.sh is syntactically valid" {
  run bash -n "$INSTALL"
  [ "$status" -eq 0 ]
}

@test "sourcing install.sh does not run the installer" {
  run bash -c "source '$INSTALL'"
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}

# ── detect_target ──────────────────────────────────────────────────────────────

@test "detect_target honors CLAUDEBAR_TARGET override" {
  CLAUDEBAR_TARGET="aarch64-apple-darwin" run detect_target
  [ "$output" = "aarch64-apple-darwin" ]
}

@test "detect_target maps Linux x86_64 to musl triple" {
  uname() { case "$1" in -s) echo Linux ;; -m) echo x86_64 ;; esac; }
  export -f uname
  CLAUDEBAR_TARGET="" run detect_target
  [ "$output" = "x86_64-unknown-linux-musl" ]
}

@test "detect_target maps Darwin arm64 to apple triple" {
  uname() { case "$1" in -s) echo Darwin ;; -m) echo arm64 ;; esac; }
  export -f uname
  CLAUDEBAR_TARGET="" run detect_target
  [ "$output" = "aarch64-apple-darwin" ]
}

@test "detect_target prints nothing for unsupported arch" {
  uname() { case "$1" in -s) echo Linux ;; -m) echo riscv64 ;; esac; }
  export -f uname
  CLAUDEBAR_TARGET="" run detect_target
  [ -z "$output" ]
}

# ── release JSON parsing ───────────────────────────────────────────────────────

@test "release_tag extracts the tag" {
  run release_tag '{"tag_name":"v1.2.3"}'
  [ "$output" = "v1.2.3" ]
}

@test "release_tag is empty when no release exists" {
  run release_tag '{"message":"Not Found"}'
  [ -z "$output" ]
}

@test "find_asset_url resolves an asset by regex" {
  json='{"assets":[{"name":"claudebar-x86_64-unknown-linux-musl.tar.gz","browser_download_url":"https://github.com/x/y/releases/download/v1/claudebar-x86_64-unknown-linux-musl.tar.gz"}]}'
  run find_asset_url "$json" '^claudebar-x86_64-unknown-linux-musl\.tar\.gz$'
  [[ "$output" == https://github.com/*musl.tar.gz ]]
}

@test "find_asset_url is empty when nothing matches" {
  run find_asset_url '{"assets":[{"name":"other.zip","browser_download_url":"https://github.com/x"}]}' '^claudebar-.*\.tar\.gz$'
  [ -z "$output" ]
}

# ── require_github_host ────────────────────────────────────────────────────────

@test "require_github_host accepts github.com URLs" {
  run require_github_host "https://github.com/micschr0/claudebar/releases/download/v1/x.tar.gz"
  [ "$status" -eq 0 ]
}

@test "require_github_host rejects other hosts" {
  run require_github_host "https://evil.example.com/x.tar.gz"
  [ "$status" -eq 1 ]
  [[ "$output" == *"untrusted host"* ]]
}

@test "require_github_host rejects plain http" {
  run require_github_host "http://github.com/micschr0/claudebar/x.tar.gz"
  [ "$status" -eq 1 ]
}

@test "require_github_host rejects lookalike domains" {
  run require_github_host "https://github.com.evil.example/x.tar.gz"
  [ "$status" -eq 1 ]
}

# ── verify_checksum ────────────────────────────────────────────────────────────

@test "verify_checksum accepts text-mode sums (hash  name)" {
  echo hello > file.tar.gz
  printf '%s  file.tar.gz\n' "$(sha256_of file.tar.gz)" > sha256.sum
  run verify_checksum file.tar.gz file.tar.gz sha256.sum
  [ "$status" -eq 0 ]
}

@test "verify_checksum accepts binary-mode sums (hash *name)" {
  echo hello > file.tar.gz
  printf '%s *file.tar.gz\n' "$(sha256_of file.tar.gz)" > sha256.sum
  run verify_checksum file.tar.gz file.tar.gz sha256.sum
  [ "$status" -eq 0 ]
}

@test "verify_checksum rejects a tampered file" {
  echo hello > file.tar.gz
  printf '%s *file.tar.gz\n' "$(sha256_of file.tar.gz)" > sha256.sum
  echo tampered >> file.tar.gz
  run verify_checksum file.tar.gz file.tar.gz sha256.sum
  [ "$status" -eq 1 ]
  [[ "$output" == *"SHA256 mismatch"* ]]
}

@test "verify_checksum fails on a missing entry" {
  echo hello > file.tar.gz
  printf 'deadbeef *other.tar.gz\n' > sha256.sum
  run verify_checksum file.tar.gz file.tar.gz sha256.sum
  [ "$status" -eq 1 ]
  [[ "$output" == *"No checksum entry"* ]]
}

# ── verify_attestation ──────────────────────────────────────────────────────────
# Always non-fatal by design (status 0 on every branch) — assert that explicitly
# alongside each outcome's message, since a regressed `return 1` would still let
# the SHA256 gate mask the break in install_prebuilt().

@test "verify_attestation skips when gh is not installed" {
  command() { if [ "$1" = -v ] && [ "$2" = gh ]; then return 1; fi; builtin command "$@"; }
  run verify_attestation file.tar.gz
  [ "$status" -eq 0 ]
  [[ "$output" == *"gh CLI not installed"* ]]
}

@test "verify_attestation skips when gh predates the attestation subcommand" {
  gh() { [ "$*" = "attestation --help" ] && return 1; return 1; }
  export -f gh
  run verify_attestation file.tar.gz
  [ "$status" -eq 0 ]
  [[ "$output" == *"gh CLI too old"* ]]
}

@test "verify_attestation skips when gh is not authenticated" {
  gh() {
    case "$*" in
      "attestation --help") return 0 ;;
      "auth status") return 1 ;;
    esac
    return 1
  }
  export -f gh
  run verify_attestation file.tar.gz
  [ "$status" -eq 0 ]
  [[ "$output" == *"not authenticated"* ]]
}

@test "verify_attestation reports success when gh verifies" {
  gh() {
    case "$*" in
      "attestation --help") return 0 ;;
      "auth status") return 0 ;;
      "attestation verify "*"--signer-workflow micschr0/claudebar/.github/workflows/release.yml") return 0 ;;
    esac
    return 1
  }
  export -f gh
  run verify_attestation file.tar.gz
  [ "$status" -eq 0 ]
  [[ "$output" == *"Build provenance verified"* ]]
}

@test "verify_attestation reports failure without aborting the install" {
  gh() {
    case "$*" in
      "attestation --help") return 0 ;;
      "auth status") return 0 ;;
      "attestation verify "*) return 1 ;;
    esac
    return 1
  }
  export -f gh
  run verify_attestation file.tar.gz
  [ "$status" -eq 0 ]
  [[ "$output" == *"Provenance verification failed"* ]]
}

# ── archive safety ─────────────────────────────────────────────────────────────

@test "archive_has_unsafe_paths passes a clean archive" {
  echo bin > claudebar
  tar -czf clean.tar.gz claudebar
  run archive_has_unsafe_paths clean.tar.gz
  [ "$status" -eq 1 ]
}

@test "archive_has_unsafe_paths flags path traversal" {
  mkdir -p sub
  echo evil > sub/x
  tar -czf evil.tar.gz --transform 's|sub/x|../escape|' sub/x
  run archive_has_unsafe_paths evil.tar.gz
  [ "$status" -eq 0 ]
}

@test "extract_archive extracts a clean archive" {
  echo bin > claudebar
  tar -czf clean.tar.gz claudebar
  mkdir out
  run extract_archive clean.tar.gz out
  [ "$status" -eq 0 ]
  [ -f out/claudebar ]
}

@test "extract_archive refuses a traversal archive" {
  mkdir -p sub
  echo evil > sub/x
  tar -czf evil.tar.gz --transform 's|sub/x|../escape|' sub/x
  mkdir out
  run extract_archive evil.tar.gz out
  [ "$status" -eq 1 ]
  [[ "$output" == *"unsafe paths"* ]]
  [ ! -e escape ]
}

# ── install_from_source guards ─────────────────────────────────────────────────

@test "install_from_source fails without a source dir" {
  run install_from_source ""
  [ "$status" -eq 1 ]
  [[ "$output" == *"No local checkout"* ]]
}

@test "install_from_source fails without Cargo.toml" {
  mkdir empty-checkout
  run install_from_source "$PWD/empty-checkout"
  [ "$status" -eq 1 ]
  [[ "$output" == *"No Cargo.toml"* ]]
}
