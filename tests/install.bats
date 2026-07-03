#!/usr/bin/env bats
# Light Bats coverage for install.sh — syntax check only. install.sh runs its
# full install flow at top level (no `main` guard) and its happy path calls
# out to the GitHub releases API, so a live/networked run is deliberately out
# of scope here; that path is already covered by .github/workflows/verify-install.yml.

setup() {
  REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/.." && pwd)"
  INSTALL="$REPO_ROOT/install.sh"
}

@test "install.sh is syntactically valid" {
  run bash -n "$INSTALL"
  [ "$status" -eq 0 ]
}

# verify_checksum is tested in isolation by extracting the function (plus
# sha256_of) from the script — the top-level install flow never runs.
extract_verify_checksum() {
  sed -n '/^sha256_of()/,/^}/p; /^verify_checksum()/,/^}/p' "$INSTALL"
  echo 'red() { echo "$@"; }; green() { echo "$@"; }'
}

@test "verify_checksum accepts text-mode sums (hash  name)" {
  cd "$BATS_TEST_TMPDIR"
  echo hello > file.tar.gz
  hash=$(sha256sum file.tar.gz | awk '{print $1}')
  printf '%s  file.tar.gz\n' "$hash" > sha256.sum
  run bash -c "$(extract_verify_checksum); verify_checksum file.tar.gz file.tar.gz sha256.sum"
  [ "$status" -eq 0 ]
}

@test "verify_checksum accepts binary-mode sums (hash *name)" {
  cd "$BATS_TEST_TMPDIR"
  echo hello > file.tar.gz
  hash=$(sha256sum file.tar.gz | awk '{print $1}')
  printf '%s *file.tar.gz\n' "$hash" > sha256.sum
  run bash -c "$(extract_verify_checksum); verify_checksum file.tar.gz file.tar.gz sha256.sum"
  [ "$status" -eq 0 ]
}

@test "verify_checksum rejects a tampered file" {
  cd "$BATS_TEST_TMPDIR"
  echo hello > file.tar.gz
  hash=$(sha256sum file.tar.gz | awk '{print $1}')
  printf '%s *file.tar.gz\n' "$hash" > sha256.sum
  echo tampered >> file.tar.gz
  run bash -c "$(extract_verify_checksum); verify_checksum file.tar.gz file.tar.gz sha256.sum"
  [ "$status" -eq 1 ]
  [[ "$output" == *"SHA256 mismatch"* ]]
}

@test "verify_checksum fails on a missing entry" {
  cd "$BATS_TEST_TMPDIR"
  echo hello > file.tar.gz
  printf 'deadbeef *other.tar.gz\n' > sha256.sum
  run bash -c "$(extract_verify_checksum); verify_checksum file.tar.gz file.tar.gz sha256.sum"
  [ "$status" -eq 1 ]
  [[ "$output" == *"No checksum entry"* ]]
}
