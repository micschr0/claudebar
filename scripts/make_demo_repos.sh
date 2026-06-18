#!/usr/bin/env bash
# Create deterministic demo git repos under /tmp for screenshot generation.
# Each repo lands in a distinct state so the README screenshots show varied
# git segments (ahead/behind, modified/untracked counts, branch names) instead
# of an identical "main ↑2 M1 ?1" on every shot.
#
#   repo            branch               state
#   demo-clean      main                 clean (no markers)
#   demo-app        main                 ↑2  M1 ?1
#   demo-busy       feature/render-cache ↑3 ↓1  M4 ?2   (diverged)
#   demo-release    release/2.0          ↑5  M2
#   demo-behind     fix/auth-token       ↓2  ?3          (behind only)
#
# Run before gen_screenshots.py:  bash scripts/make_demo_repos.sh
set -euo pipefail

ROOT=/tmp
GIT_AUTHOR="demo"; GIT_EMAIL="demo@example.com"
FILES=(src/main.rs src/lib.rs README.md Cargo.toml)

init() {  # init <dir> <branch>
  local d="$ROOT/$1" b=$2
  rm -rf "$d" "$d.up.git" "$d.clone"
  git init -q -b "$b" "$d"
  git -C "$d" config user.name  "$GIT_AUTHOR"
  git -C "$d" config user.email "$GIT_EMAIL"
}

seed() {  # seed <dir> : tracked files + base commit
  local d="$ROOT/$1" f
  for f in "${FILES[@]}"; do
    mkdir -p "$d/$(dirname "$f")"; printf '// %s\n' "$f" > "$d/$f"
  done
  git -C "$d" add -A; git -C "$d" commit -q -m "initial commit"
}

upstream() {  # upstream <dir> : give the repo a tracking branch via a bare remote
  local d="$ROOT/$1" bare="$ROOT/$1.up.git"
  git init -q --bare "$bare"
  git -C "$d" remote add origin "$bare"
  git -C "$d" push -q -u origin HEAD
}

ahead() {  # ahead <dir> <n> : n local commits not on the remote
  local d="$ROOT/$1" n=$2 i=0
  while (( i < n )); do git -C "$d" commit -q --allow-empty -m "local change $i"; i=$((i+1)); done
}

behind() {  # behind <dir> <m> : push m commits to the remote, then rewind local
  # Advance local + remote by m commits (updating the origin/<branch> tracking
  # ref), then hard-reset the local branch back m commits. The remote-tracking
  # ref stays ahead, so the branch reads as "behind m". Works on any branch
  # name (no clone, no remote-HEAD checkout issue).
  local d="$ROOT/$1" m=$2 i=0
  while (( i < m )); do git -C "$d" commit -q --allow-empty -m "upstream change $i"; i=$((i+1)); done
  git -C "$d" push -q origin HEAD
  git -C "$d" reset -q --hard "HEAD~$m"
}

modify() {  # modify <dir> <n> : dirty n tracked files (shows as M)
  local d="$ROOT/$1" n=$2 i=0 f
  for f in "${FILES[@]}"; do
    (( i < n )) || break; printf '// edit %d\n' "$i" >> "$d/$f"; i=$((i+1))
  done
}

untrack() {  # untrack <dir> <n> : n untracked files (shows as ?)
  local d="$ROOT/$1" n=$2 i=0
  while (( i < n )); do printf 'scratch\n' > "$d/scratch_$i.tmp"; i=$((i+1)); done
}

# demo-clean — calm baseline, nothing to report
init demo-clean main;            seed demo-clean

# demo-app — the original normal state
init demo-app main;              seed demo-app; upstream demo-app
ahead demo-app 2;                modify demo-app 1; untrack demo-app 1

# demo-busy — diverged feature branch under pressure
init demo-busy feature/render-cache; seed demo-busy; upstream demo-busy
behind demo-busy 1;              ahead demo-busy 3; modify demo-busy 4; untrack demo-busy 2

# demo-release — release branch, several commits ahead
init demo-release release/2.0;   seed demo-release; upstream demo-release
ahead demo-release 5;            modify demo-release 2

# demo-behind — behind the remote, a few stray files
init demo-behind fix/auth-token; seed demo-behind; upstream demo-behind
behind demo-behind 2;            untrack demo-behind 3

echo "Demo repos ready under $ROOT: demo-{clean,app,busy,release,behind}"
