# Discovery — claudebar

## Product
**claudebar** — a fast, themeable statusline for Claude Code. Rust binary that renders your
working directory, git state, context usage, and live rate-limit countdowns into the Claude Code
status line. Read-only, ~1.5 MB, renders in ~30 ms (~5× faster than the bash equivalent).

## Audience
Developers who use Claude Code daily. Terminal-native, care about speed, minimalism, and at-a-glance
signal during long sessions (context budget + rate limits).

## Core message
"See your session at a glance — context, rate limits, git, model — without leaving the terminal."

## Problems addressed
- Running out of context mid-task with no warning.
- Hitting rate limits unexpectedly.
- Slow / cluttered bash statuslines.

## Features to showcase
- Live rate-limit countdowns
- Color-coded context usage (green → yellow → red)
- Inline git state
- 16 themes · 6 styles
- ~30 ms render (~5× faster than bash), ~1.5 MB, read-only

## Emotional arc
Calm confidence → "it just watches your back." Premium, quiet, fast.

## CTA
Install: `curl -fsSL https://raw.githubusercontent.com/micschr0/claudebar/main/install.sh | bash`
Repo: github.com/micschr0/claudebar
