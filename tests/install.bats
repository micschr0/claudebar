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
