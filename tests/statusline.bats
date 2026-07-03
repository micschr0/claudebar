#!/usr/bin/env bats
# Bats suite for statusline-command.sh — ports the eight security.yml smoke
# cases into named, fixture-driven tests and adds error/edge cases the smoke
# job never covered (malformed JSON, empty stdin, wrong-typed fields).

setup() {
  REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/.." && pwd)"
  SCRIPT="$REPO_ROOT/statusline-command.sh"
  command -v jq &>/dev/null || skip "jq not installed"
}

@test "syntax check: bash -n statusline-command.sh" {
  run bash -n "$SCRIPT"
  [ "$status" -eq 0 ]
}

@test "empty input {} degrades gracefully" {
  run bash "$SCRIPT" < "$REPO_ROOT/fixtures/empty.json"
  [ "$status" -eq 0 ]
}

@test "typical input renders non-empty output" {
  run bash "$SCRIPT" < "$REPO_ROOT/fixtures/typical.json"
  [ "$status" -eq 0 ]
  [ -n "$output" ]
}

@test "over-limit context (used_percentage > 100) renders" {
  run bash "$SCRIPT" < "$REPO_ROOT/fixtures/over_100_context.json"
  [ "$status" -eq 0 ]
  [ -n "$output" ]
}

@test "5h rate-limit segment renders" {
  run bash "$SCRIPT" < "$REPO_ROOT/fixtures/over_limit_5h.json"
  [ "$status" -eq 0 ]
  [ -n "$output" ]
}

@test "weekly rate-limit segment (>=50%) renders" {
  run bash "$SCRIPT" < "$REPO_ROOT/fixtures/weekly_at_50.json"
  [ "$status" -eq 0 ]
  [ -n "$output" ]
}

@test "effort-level segment renders" {
  run bash "$SCRIPT" < "$REPO_ROOT/fixtures/effort_max.json"
  [ "$status" -eq 0 ]
  [ -n "$output" ]
}

@test "ANSI injection in cwd/model is stripped" {
  run bash "$SCRIPT" < "$REPO_ROOT/fixtures/injection.json"
  [ "$status" -eq 0 ]
  # The script only ever emits 256-color `[38;5;...m` sequences — it never
  # emits the 16-color codes the fixture tries to smuggle in. Their absence
  # proves the ESC bytes were stripped by the CTRL sanitize guard.
  [[ "$output" != *$'\e'"[31m"* ]]
  [[ "$output" != *$'\e'"[5m"* ]]
}

@test "no-git-repo path renders" {
  run bash "$SCRIPT" < "$REPO_ROOT/fixtures/no_git.json"
  [ "$status" -eq 0 ]
  [ -n "$output" ]
}

@test "malformed/non-JSON stdin degrades gracefully" {
  run bash -c "printf 'not valid json {' | bash \"$SCRIPT\""
  [ "$status" -eq 0 ]
}

@test "empty stdin degrades gracefully" {
  run bash "$SCRIPT" < /dev/null
  [ "$status" -eq 0 ]
}

@test "wrong-typed fields degrade gracefully" {
  run bash "$SCRIPT" < "$REPO_ROOT/fixtures/bad_types.json"
  [ "$status" -eq 0 ]
  [ -n "$output" ]
}
