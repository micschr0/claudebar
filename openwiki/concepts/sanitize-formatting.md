# Sanitize & formatting

Shared hardening + formatting helpers in `src/sanitize.rs`, used by segments
before emitting anything.

## `strip_control`

Strips ESC/BEL/CR/LF and other control bytes from host strings (`cwd`,
git branch names, model names, …) before they reach the status line. This is
an injection guard — without it a malicious `cwd` or git output could inject
escape sequences into the terminal. The golden test
`injection_no_control_byte_leak` pins this at the full-render level.

## `abbreviate_path` (fish-style)

- `$HOME` → `~`
- Compresses path components, dotfiles to 2 chars
- Used by the directory segment

## `fmt_tokens`

Formats token counts with k/M rounding and carry/promotion:

- `< 1000` → raw number
- `k` for thousands (1.5k, 12k), `M` for millions
- Handles promotion across units

## `fmt_reset`

Adaptive countdown formatting for rate-limit resets:

- `now` injected (never wall-clock)
- Under an hour → minutes/seconds (`12m`, `45s`); over → `1h 5m`
- Returns `None` when the reset is absent/expired (segment can hide)

## Callers

| Helper | Segments |
|---|---|
| `strip_control` | directory, git, model, dev-context, context, rate-limits, float |
| `abbreviate_path` | directory |
| `fmt_tokens` | context, rate-limits |
| `fmt_reset` | rate-limits |

## Contract

- All helpers are pure; output is deterministic given input.
- Nothing raw from host strings is ever emitted unsanitized.
- Failure contracts: `strip_control` on bad input yields empty, never panics.