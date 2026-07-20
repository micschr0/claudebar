#!/usr/bin/env bats
# Bats coverage for scripts/make_demo_repos.sh — asserts the deterministic
# git-state contract the README screenshots depend on: per-repo branch name,
# ahead/behind counts, and modified/untracked tallies. The script is
# idempotent (init() rm -rf's each repo first), so setup_file runs it once
# and every test reads the resulting state.

setup_file() {
  command -v git &>/dev/null || skip "git not installed"
  REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/.." && pwd)"
  export DEMO_SCRIPT="$REPO_ROOT/scripts/make_demo_repos.sh"
  bash "$DEMO_SCRIPT" > "$BATS_FILE_TMPDIR/run.out"
}

repo_branch()    { git -C "/tmp/$1" branch --show-current; }
ahead_count()    { git -C "/tmp/$1" rev-list --count "@{u}..HEAD"; }
behind_count()   { git -C "/tmp/$1" rev-list --count "HEAD..@{u}"; }
modified_count() { git -C "/tmp/$1" status --porcelain | grep -c '^ M' || true; }
untracked_count() { git -C "/tmp/$1" status --porcelain | grep -c '^??' || true; }

assert_state() { # assert_state <repo> <branch> <ahead> <behind> <modified> <untracked>
  [ "$(repo_branch "$1")" = "$2" ]
  [ "$(ahead_count "$1")" -eq "$3" ]
  [ "$(behind_count "$1")" -eq "$4" ]
  [ "$(modified_count "$1")" -eq "$5" ]
  [ "$(untracked_count "$1")" -eq "$6" ]
}

@test "syntax check: bash -n make_demo_repos.sh" {
  run bash -n "$DEMO_SCRIPT"
  [ "$status" -eq 0 ]
}

@test "script reports completion" {
  [[ "$(cat "$BATS_FILE_TMPDIR/run.out")" == *"Demo repos ready"* ]]
}

@test "demo-clean: main, fully clean" {
  [ "$(repo_branch demo-clean)" = "main" ]
  [ -z "$(git -C /tmp/demo-clean status --porcelain)" ]
}

@test "demo-app: main ↑2 M1 ?1" {
  assert_state demo-app main 2 0 1 1
}

@test "demo-busy: feature/render-cache ↑3 ↓1 M4 ?2" {
  assert_state demo-busy feature/render-cache 3 1 4 2
}

@test "demo-release: release/2.0 ↑5 M2" {
  assert_state demo-release release/2.0 5 0 2 0
}

@test "demo-behind: fix/auth-token ↓2 ?3" {
  assert_state demo-behind fix/auth-token 0 2 0 3
}

@test "demo-git-a: main ↑2 M1" {
  assert_state demo-git-a main 2 0 1 0
}

@test "demo-git-b: main ↑1 ↓2 M1" {
  assert_state demo-git-b main 1 2 1 0
}

@test "re-run is idempotent: states do not drift" {
  bash "$DEMO_SCRIPT" >/dev/null
  assert_state demo-busy feature/render-cache 3 1 4 2
  assert_state demo-app main 2 0 1 1
}
