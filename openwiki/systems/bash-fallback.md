# Bash fallback

`statusline-command.sh` — the zero-toolchain fallback for environments without
a Rust toolchain. Requires `jq` + `git`. **Divergence-aware contract**: it is
**not** byte-for-byte parity with the Rust binary.

## What it implements (fixed order)

directory, git, tokens+context bar, 5h/weekly rate windows, dev-context,
model+effort.

## What it omits / diverges

- Omits: lines, cost, duration, clock, burn, dev-context ordering parity.
- Hardcoded `◈` model glyph (vs Rust `style.glyphs.model`).
- Always emits chevron separators even around skipped segments (Rust skips
  separators around empty segments).
- Weekly window shows at ≥50 with warn-only colors; no weekly reset text
  unless pct exists.
- Own `date +%s` countdown (line 193) vs Rust's injected `now`.
- Single-jq-call parse; subprocess economy.

## Security

- ANSI injection guards: strips `CTRL` bytes from host strings.
- No cache files written (unlike Rust `limit_sync`/`burn`).

## SLO

- Subprocesses ≤ 5.
- p95 < 100 ms (`scripts/benchmark.sh`, strace on Linux).

## Tests

- `tests/statusline.bats` (contracts, guards).
- `scripts/benchmark.sh` (SLO guard).