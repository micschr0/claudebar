#!/usr/bin/env bats
# Bats coverage for the scripts/ tooling: syntax checks for every script plus
# the fail-fast guard contracts of gen-gallery.sh. Runtime behavior of
# benchmark.sh (SLO timing — flaky under CI load) and gen_terminal_gifs.sh
# (needs asciinema/agg/tmux/Nerd Fonts) is intentionally not exercised here.

setup() {
  REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/.." && pwd)"
  SCRIPTS="$REPO_ROOT/scripts"
}

@test "syntax check: bash -n benchmark.sh" {
  run bash -n "$SCRIPTS/benchmark.sh"
  [ "$status" -eq 0 ]
}

@test "syntax check: bash -n gen-gallery.sh" {
  run bash -n "$SCRIPTS/gen-gallery.sh"
  [ "$status" -eq 0 ]
}

@test "syntax check: bash -n gen_terminal_gifs.sh" {
  run bash -n "$SCRIPTS/gen_terminal_gifs.sh"
  [ "$status" -eq 0 ]
}

@test "gen-gallery fails fast when the binary is missing" {
  run env CLAUDEBAR_BIN="/nonexistent/claudebar" bash "$SCRIPTS/gen-gallery.sh"
  [ "$status" -eq 1 ]
  [[ "$output" == *"binary not found"* ]]
}

@test "gen-gallery fails when no themes parse from \`list\`" {
  fake_bin="$(mktemp)"
  printf '#!/usr/bin/env bash\necho garbage\n' > "$fake_bin"
  chmod +x "$fake_bin"
  run env CLAUDEBAR_BIN="$fake_bin" bash "$SCRIPTS/gen-gallery.sh"
  rm -f "$fake_bin"
  [ "$status" -eq 1 ]
  [[ "$output" == *"no themes parsed"* ]]
}
