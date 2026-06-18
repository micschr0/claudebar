#!/bin/bash
# Performance benchmark + SLO guard for statusline-command.sh
# SLO: subprocesses <= 5 | p95 < 100ms
# Usage: bash scripts/benchmark.sh

set -euo pipefail

SCRIPT="$(cd "$(dirname "$0")/.." && pwd)/statusline-command.sh"
WARMUP=10
RUNS=50
SLO_PROCS=5
SLO_P95_MS=100

ESC=$'\e'; R="${ESC}[0m"
BOLD="${ESC}[1m"; DIM="${ESC}[38;5;245m"
GREEN="${ESC}[38;5;114m"; AMBER="${ESC}[38;5;221m"; RED="${ESC}[38;5;203m"

ok()   { printf "${GREEN}✓${R}"; }
warn() { printf "${AMBER}!${R}"; }
fail() { printf "${RED}✗${R}"; }

# ── Timing helper ─────────────────────────────────────────────────────────────
# Returns elapsed milliseconds; works on bash 3.2+ (macOS) and bash 5+ (Linux).
measure_ms() {
  local input=$1
  local t
  t=$( { TIMEFORMAT='%R'; time bash "$SCRIPT" <<< "$input" > /dev/null; } 2>&1 )
  awk -v t="$t" 'BEGIN{printf "%.0f", t * 1000}'
}

# Sort array of ints, print to stdout one per line
sort_ints() {
  printf '%s\n' "$@" | sort -n
}

# ── Preflight ─────────────────────────────────────────────────────────────────
if [[ "$(uname)" == "Linux" ]] && ! command -v strace &>/dev/null; then
  echo "ERROR: strace not found — subprocess SLO cannot be verified on Linux" >&2
  echo "Install it: sudo apt-get install -y strace" >&2
  exit 1
fi

# ── Subprocess counter (Linux only via strace) ─────────────────────────────────
count_procs() {
  local input=$1
  if command -v strace &>/dev/null; then
    strace -e trace=execve -f bash "$SCRIPT" <<< "$input" 2>&1 \
      | grep -c 'execve(' || true
  else
    echo "n/a"
  fi
}

# ── Run benchmark for one scenario ────────────────────────────────────────────
bench() {
  local name=$1 input=$2
  local -a vals=()

  # warmup
  for (( i=0; i<WARMUP; i++ )); do
    bash "$SCRIPT" <<< "$input" > /dev/null
  done

  # measure
  for (( i=0; i<RUNS; i++ )); do
    vals+=( "$(measure_ms "$input")" )
  done

  # sort
  local sorted
  IFS=$'\n' read -r -d '' -a sorted < <(sort_ints "${vals[@]}" && printf '\0') || true

  local p50 p95
  p50=${sorted[$((RUNS / 2))]}
  p95=${sorted[$((RUNS * 95 / 100))]}

  local procs
  procs=$(count_procs "$input")

  printf '%s %s %s %s\n' "$name" "$p50" "$p95" "$procs"
}

# ── Header ────────────────────────────────────────────────────────────────────
printf '\n%sBenchmark: statusline-command.sh%s\n' "$BOLD" "$R"
printf '%sSLO: subprocesses ≤ %d · p95 < %dms%s\n\n' "$DIM" "$SLO_PROCS" "$SLO_P95_MS" "$R"
printf '%-14s │ %5s │ %5s │ %12s │ SLO\n' "Scenario" "p50" "p95" "Subprocesses"
printf '%-14s─┼─%5s─┼─%5s─┼─%12s─┼─────\n' \
  "──────────────" "─────" "─────" "────────────"

# ── Run all scenarios ──────────────────────────────────────────────────────────
slo_ok=true

for name in minimal typical full; do
  case "$name" in
    minimal) input='{}' ;;
    typical) input='{"cwd":"/home/user/project","context_window":{"total_input_tokens":35000,"total_output_tokens":7300,"used_percentage":67.0},"model":{"display_name":"Claude Sonnet 4.6"}}' ;;
    full)    input='{"cwd":"/home/user/project","context_window":{"total_input_tokens":350000,"total_output_tokens":73000,"used_percentage":95.0},"rate_limits":{"five_hour":{"used_percentage":72.0,"resets_at":9999999999},"seven_day":{"used_percentage":55.0,"resets_at":9999999999}},"model":{"display_name":"Claude Sonnet 4.6"},"effort":{"level":"high"}}' ;;
  esac
  read -r _ p50 p95 procs <<< "$(bench "$name" "$input")"

  # SLO check
  proc_ok=true; time_ok=true
  if [[ "$procs" != "n/a" ]] && [ "$procs" -gt "$SLO_PROCS" ] 2>/dev/null; then
    proc_ok=false; slo_ok=false
  fi
  if [ "$p95" -gt "$SLO_P95_MS" ] 2>/dev/null; then
    time_ok=false; slo_ok=false
  fi

  # Format proc column
  if [[ "$procs" == "n/a" ]]; then
    proc_col="${DIM}n/a (no strace)${R}"
  elif $proc_ok; then
    proc_col="${GREEN}${procs}${R}"
  else
    proc_col="${RED}${procs}${R}"
  fi

  # SLO indicator
  if $proc_ok && $time_ok; then slo_sym="$(ok)"; else slo_sym="$(fail)"; fi

  # p95 color
  if $time_ok; then p95_col="${GREEN}${p95}ms${R}"; else p95_col="${RED}${p95}ms${R}"; fi

  printf '%-14s │ %4dms │ %s │ %-20b │ %b\n' \
    "$name" "$p50" "$p95_col" "$proc_col" "$slo_sym"
done

printf '\n'

if $slo_ok; then
  printf '%sAll scenarios within SLO.%s\n\n' "$GREEN" "$R"
  exit 0
else
  printf '%sSLO violated — see rows marked ✗ above.%s\n\n' "$RED" "$R"
  exit 1
fi
