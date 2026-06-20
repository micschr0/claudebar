# External Integrations

**Analysis Date:** 2026-06-20

## APIs & External Services

**Git (subprocess):**
- `git` CLI invoked via `std::process::Command` in `src/segment/git.rs`
- Used to read: branch name, ahead/behind counts, modified file count
- No SDK — raw subprocess with `stdout` captured and `stderr` suppressed
- Requires `git` on PATH at runtime

**Claude Code host (stdin JSON):**
- claudebar is invoked by Claude Code as a `statusLine` command
- Receives session JSON on stdin — parsed in `src/model/input.rs`
- No outbound HTTP; this is a one-way data feed from the host process
- JSON schema: `rate_limits.{five_hour,seven_day}.{used_percentage,resets_at}`, `effort.level`, `cwd`, `model.display_name`, token counts

## Data Storage

**Databases:**
- None

**File Storage:**
- Local filesystem only
- User config: `$XDG_CONFIG_HOME/claudebar/config.toml` (read/write via `src/model/config.rs`)
- No temp files; no cache files written at runtime

**Caching:**
- None

## Authentication & Identity

**Auth Provider:**
- None — claudebar is a local CLI tool with no authentication

## Monitoring & Observability

**Error Tracking:**
- None

**Logs:**
- Errors written to stderr via Rust's standard error propagation (`thiserror`)
- No structured logging framework

## CI/CD & Deployment

**Hosting:**
- GitHub: `https://github.com/micschr0/claudebar`
- Binary distributed via `install.sh` — either `cargo install --path .` from checkout, or downloads `statusline-command.sh` from raw GitHub URL

**CI Pipeline:**
- No CI config detected (no `.github/workflows/` directory)

**Install script fetch URL:**
- `https://raw.githubusercontent.com/micschr0/claudebar/main` (used by `install.sh` when run via `curl | bash`)

## Environment Configuration

**Required env vars (runtime):**
- None required — all config is file-based

**Consulted env vars:**
- `XDG_CONFIG_HOME` — config directory location (falls back to `~/.config`)
- `HOME` — used in config path fallback

**Secrets:**
- None — no API keys or tokens in the Rust binary

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## External Tooling (dev/scripts only)

**Screenshot generation (`scripts/gen_screenshots.py`):**
- Docker (via `DOCKER_HOST=unix:///run/user/1002/docker.sock`)
- Playwright Node.js (`/tmp/pw/node_modules`)
- Hack Nerd Font (`/tmp/fonts/HackNerdFontMono-Regular.ttf`)
- These are local dev prerequisites, not runtime dependencies

**Bash fallback (`statusline-command.sh`):**
- Requires `jq` on PATH for JSON parsing
- Requires `git` on PATH for git segment
- Reads same stdin JSON as the Rust binary

---

*Integration audit: 2026-06-20*
