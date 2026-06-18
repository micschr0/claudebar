#!/bin/bash
# Claude Code statusline — dark theme
# Segments: directory | git | tokens+ctx-bar | rate-bar+timer | model

# shellcheck disable=SC2059
# Status segments deliberately put pre-defined ANSI color constants in the printf
# format string; every host-provided value is passed as a %s/%d argument, so this
# is safe and intentional rather than a format-string defect.

export LC_NUMERIC=C

command -v jq &>/dev/null || { printf 'statusline: jq not found — install it (brew install jq / apt install jq)\n' >&2; exit 1; }

input=$(cat)

ESC=$'\e'
R="${ESC}[0m"
CTRL=$'\e\a\r\n'   # terminal-control bytes stripped from host strings (injection guard)

# Colors (256-color) — tokyo-night aligned
C_DIR="${ESC}[38;5;33m"      # blue
C_GIT="${ESC}[38;5;141m"     # lavender
C_AHD="${ESC}[38;5;114m"      # green (ahead = positive state)
C_BHD="${ESC}[38;5;167m"     # muted red (behind — distinct from C_CRIT)
C_MOD="${ESC}[38;5;208m"     # amber (distinct from warning yellow)
C_TOK="${ESC}[38;5;117m"     # sky blue
C_OK="${ESC}[38;5;114m"      # green (resource healthy)
C_WARN="${ESC}[38;5;221m"    # yellow
C_CRIT="${ESC}[38;5;203m"    # red
C_SEP="${ESC}[38;5;238m"     # dark gray
C_DIM="${ESC}[38;5;245m"     # muted foreground
C_RST="${ESC}[38;5;73m"      # muted teal (reset countdown)
C_EFF_MAX="${ESC}[38;5;213m" # bright magenta (max effort)

# Powerline thin separator (U+E0B1 — right-pointing chevron, matches L→R segment
# flow; requires Nerd Font / powerline-patched font)
PL=$(printf '\xee\x82\xb1')
SEP=" ${C_SEP}${PL}${R} "
# Weekly-limit icon (Nerd Font MDI calendar, U+F00ED) — defined as bytes to keep
# the source ASCII-clean; swap the codepoint to taste.
WK=$(printf '\xf3\xb0\x83\xad')
EF=$(printf '\xef\x83\xa7')  # U+F0E7 nf-fa-bolt (effort indicator)

# make_bar <pct> <width> <fill-color>
# Emits a self-colored bar: filled run in <fill-color>, empty track dimmed in
# C_SEP, terminated with reset — so the fill level reads at a glance instead of
# the whole bar being one flat color.
make_bar() {
  local pct=$1 width=${2:-6} fill=$3
  local filled=$(( pct * width / 100 ))
  (( filled > width )) && filled=$width
  # ensure at least 1 filled char when pct > 0 (avoids indistinguishable zero-state)
  (( pct > 0 && filled == 0 )) && filled=1
  local empty=$(( width - filled ))
  local bar="$fill"
  for (( i=0; i<filled; i++ )); do bar="${bar}━"; done
  bar="${bar}${C_SEP}"
  for (( i=0; i<empty;  i++ )); do bar="${bar}╌"; done
  printf '%s\n' "${bar}${R}"
}

# fmt_reset <epoch> — adaptive time until reset (Nd Nh / Nh Nm / Nm Ns / Ns);
# empty if the timestamp is missing or already past. Reads global `now`.
fmt_reset() {
  local t=$1 diff d h m s
  [[ "$t" =~ ^[0-9]+$ ]] || return
  diff=$(( t - now )); (( diff <= 0 )) && return
  d=$(( diff / 86400 )); h=$(( (diff % 86400) / 3600 ))
  m=$(( (diff % 3600) / 60 )); s=$(( diff % 60 ))
  if   (( d > 0 )); then printf '%dd%dh' "$d" "$h"
  elif (( h > 0 )); then printf '%dh%dm' "$h" "$m"
  elif (( m > 0 )); then printf '%dm%ds' "$m" "$s"
  else                   printf '%ds' "$s"
  fi
}

# Parse all fields in one jq call — herestring avoids echo subprocess;
# delimiter US (0x1f): non-whitespace, preserves empty fields, safe in cwd/model names.
IFS=$'\x1f' read -r cwd s_in s_out used rl_pct resets_at wk_pct wk_resets_at model_name effort_level < <(
  jq -r '[
    (.cwd? // ""),
    ((.context_window?.total_input_tokens?  // 0) | (tonumber? // 0) | floor | tostring),
    ((.context_window?.total_output_tokens? // 0) | (tonumber? // 0) | floor | tostring),
    ((.context_window?.used_percentage?     // "") | (tonumber? // "") | tostring),
    ((.rate_limits?.five_hour?.used_percentage? // "") | (tonumber? // "") | tostring),
    ((.rate_limits?.five_hour?.resets_at?   // 0) | (tonumber? // 0) | floor | tostring),
    ((.rate_limits?.seven_day?.used_percentage? // "") | (tonumber? // "") | tostring),
    ((.rate_limits?.seven_day?.resets_at?   // 0) | (tonumber? // 0) | floor | tostring),
    (.model?.display_name? // ""),
    (.effort?.level? // "")
  ] | join("")' <<< "$input" 2>/dev/null
)

# ── Directory (fish style) ────────────────────────────────────────────────────
if [ -n "$cwd" ]; then
  rel="${cwd/#$HOME/\~}"
  IFS='/' read -ra parts <<< "$rel"
  total=${#parts[@]}
  fp=""
  for (( i=0; i<total-1; i++ )); do
    p="${parts[$i]}"
    if   [ -z "$p" ];              then fp="/"
    elif [ "${p:0:1}" = "." ];     then fp="${fp}${p:0:2}/"
    else                                fp="${fp}${p:0:1}/"; fi
  done
  fp="${fp}${parts[$((total-1))]}"
  fp=${fp//[$CTRL]/}            # strip terminal-control bytes (ANSI/OSC injection guard)
  printf "${C_DIR} %s${R}" "$fp"
fi

# ── Git ───────────────────────────────────────────────────────────────────────
# Single git call replaces 4 separate git subprocesses:
# symbolic-ref + 2x rev-list + status --porcelain → status --branch --porcelain
if [[ "$cwd" == /* ]]; then
  git_out=$(git -C "$cwd" -c gc.auto=0 status --branch --porcelain 2>/dev/null)
  if [ -n "$git_out" ]; then
    branch_line="${git_out%%$'\n'*}"
    branch=""; ahead=0; behind=0

    if [[ "$branch_line" == '## No commits yet on '* ]]; then
      branch="${branch_line#\#\# No commits yet on }"
    elif [[ "$branch_line" != '## HEAD (no branch)'* && "$branch_line" == '## '* ]]; then
      branch="${branch_line#\#\# }"; branch="${branch%%...*}"
      [[ "$branch_line" =~ ahead\ ([0-9]+) ]]  && ahead="${BASH_REMATCH[1]}"
      [[ "$branch_line" =~ behind\ ([0-9]+) ]] && behind="${BASH_REMATCH[1]}"
    fi

    branch=${branch//[$CTRL]/}   # strip terminal-control bytes (injection guard)
    if [ -n "$branch" ]; then
      printf "${SEP}${C_GIT}\xee\x82\xa0 %s${R}" "$branch"
      [ "${ahead}"  -gt 0 ] 2>/dev/null && printf " ${C_AHD}↑%s${R}" "$ahead"
      [ "${behind}" -gt 0 ] 2>/dev/null && printf " ${C_BHD}↓%s${R}" "$behind"

      # Count modified/new from already-fetched output — no extra forks
      n_mod=0; n_new=0
      while IFS= read -r line; do
        [[ "$line" == '## '* || -z "$line" ]] && continue
        if [[ "$line" == '?? '* ]]; then (( n_new++ ))
        else (( n_mod++ ))
        fi
      done <<< "$git_out"

      [ "$n_mod" -gt 0 ] && printf " ${C_WARN}M%s${R}" "$n_mod"
      [ "$n_new" -gt 0 ] && printf " ${C_DIM}?%s${R}" "$n_new"
    fi
  fi
fi

# ── Session tokens + context bar ─────────────────────────────────────────────
if [ "$s_in" -gt 0 ] 2>/dev/null || [ "$s_out" -gt 0 ] 2>/dev/null; then
  total=$(( s_in + s_out ))
  # Pure bash integer arithmetic replaces two awk subprocesses
  if   [ "$total" -ge 1000000 ]; then
    _i=$(( total / 1000000 ))
    _d=$(( (total % 1000000 * 10 + 500000) / 1000000 ))
    (( _d >= 10 )) && { (( _i++ )); _d=0; }
    fmt="${_i}.${_d}M"
  elif [ "$total" -ge 1000 ]; then
    _i=$(( total / 1000 ))
    _d=$(( (total % 1000 * 10 + 500) / 1000 ))
    (( _d >= 10 )) && { (( _i++ )); _d=0; }
    fmt="${_i}.${_d}k"
  else fmt="$total"; fi
  if [[ "$used" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    pct=$(printf '%.0f' "$used")
    # numeric pre-check above guarantees a real number; range-gate below
    if [ "$pct" -ge 0 ] && [ "$pct" -le 999 ] 2>/dev/null; then
      if   [ "$pct" -gt 100 ]; then cc="$C_CRIT"
      elif [ "$pct" -ge 80 ];  then cc="$C_CRIT"
      elif [ "$pct" -ge 50 ];  then cc="$C_WARN"
      else cc="$C_OK"; fi
      bar=$(make_bar "$pct" 6 "$cc")
      printf "${SEP}${C_DIM}󰍛 %s ${cc}%d%%${R} ${C_TOK}⬡ %s${R}" "$bar" "$pct" "$fmt"
    else
      printf "${SEP}${C_TOK}⬡ %s${R}" "$fmt"
    fi
  else
    printf "${SEP}${C_TOK}⬡ %s${R}" "$fmt"
  fi
fi

# ── Rate limits: 5h window (always) + weekly window (when it gets tight) ──────
if [ -n "$rl_pct" ] || [ "${resets_at:-0}" -gt 0 ] 2>/dev/null \
   || [ -n "$wk_pct" ] || [ "${wk_resets_at:-0}" -gt 0 ] 2>/dev/null; then
  now=$(date +%s)

  # 5-hour rolling window — shown whenever present, with live reset countdown
  if [ -n "$rl_pct" ] || [ "${resets_at:-0}" -gt 0 ] 2>/dev/null; then
    printf '%s' "$SEP"
    if [[ "$rl_pct" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
      rl_int=$(printf '%.0f' "$rl_pct")
      # allow >100 (you can be over the limit); the upper bound only rejects a
      # leaked Unix timestamp (~1.7e9), never a real over-limit percentage
      if [ "$rl_int" -ge 0 ] && [ "$rl_int" -le 999 ] 2>/dev/null; then
        if   [ "$rl_int" -ge 80 ]; then rlc="$C_CRIT"
        elif [ "$rl_int" -ge 50 ]; then rlc="$C_WARN"
        else rlc="$C_OK"; fi
        bar=$(make_bar "$rl_int" 6 "$rlc")
        printf "${C_DIM}󰔟 %s ${rlc}%d%%${R}" "$bar" "$rl_int"
      fi
    fi
    rrem=$(fmt_reset "$resets_at")
    [ -n "$rrem" ] && printf " ${C_DIM}↺${R} ${C_RST}%s${R}" "$rrem"
  fi

  # weekly (7-day) window — only surfaced once it becomes the binding constraint
  if [[ "$wk_pct" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    wk_int=$(printf '%.0f' "$wk_pct")
    if [ "$wk_int" -ge 50 ] && [ "$wk_int" -le 999 ] 2>/dev/null; then
      if [ "$wk_int" -ge 80 ]; then wkc="$C_CRIT"; else wkc="$C_WARN"; fi
      bar=$(make_bar "$wk_int" 6 "$wkc")
      printf "${SEP}${C_DIM}${WK} %s ${wkc}%d%%${R}" "$bar" "$wk_int"
      wrem=$(fmt_reset "$wk_resets_at")
      [ -n "$wrem" ] && printf " ${C_DIM}↺${R} ${C_RST}%s${R}" "$wrem"
    fi
  fi
fi

# ── Model + Effort (trailing — low-volatility metadata) ──────────────────────
model_name=${model_name//[$CTRL]/}   # strip terminal-control bytes (ANSI/OSC injection guard)
effort_level=${effort_level//[$CTRL]/}
if [ -n "$model_name" ] || [ -n "$effort_level" ]; then
  printf '%s' "$SEP"
  [ -n "$model_name" ] && printf "${C_MOD}◈ %s${R}" "$model_name"
  if [ -n "$effort_level" ]; then
    case "$effort_level" in
      low)    ec="$C_DIM"     ;;
      medium) ec="$C_DIM"     ;;
      high)   ec="$C_OK"      ;;
      xhigh)  ec="$C_WARN"    ;;
      max)    ec="$C_EFF_MAX" ;;
      *)      ec="$C_DIM"     ;;
    esac
    [ -n "$model_name" ] && printf ' '
    printf "${C_DIM}${EF} ${ec}%s${R}" "$effort_level"
  fi
fi

exit 0
