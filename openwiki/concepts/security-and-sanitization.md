---
type: Concept
title: "Security: terminal-injection hardening"
description: "How claudebar defends its statusline against ANSI/OSC escape injection: every host-provided string (cwd, git branch, model name, effort, dev-context) is stripped of terminal-control bytes by sanitize::strip_control before it reaches the rendered line, in both the Rust render path and the bash fallback."
tags: [security, sanitization, injection, ansi, terminal, statusline]
verified:
  - by: openwiki/0.4.0
    at: 2026-08-26T22:48:34.063Z
sources:
  - id: openwiki-source-5c39d26440648d6fcf80937e
    resource: repo://fixtures/injection.json
  - id: openwiki-source-03ffc32a0ca502ab67c54b25
    resource: repo://install.sh
  - id: openwiki-source-5d4fb36fe9d34b6bc366e220
    resource: repo://src/model/input.rs
  - id: openwiki-source-d977bd28254dbfcf5d7fe3bb
    resource: repo://src/render/writer.rs
  - id: openwiki-source-3a6ff89030cacfe8ee730edf
    resource: repo://src/sanitize.rs
  - id: openwiki-source-caed7c213e3bf4afc1854652
    resource: repo://src/segment/dev_context.rs
  - id: openwiki-source-36efb8afd8dc119a86c1105b
    resource: repo://src/segment/directory.rs
  - id: openwiki-source-0700af36d25875a20e6db044
    resource: repo://src/segment/git.rs
  - id: openwiki-source-fce92f63c4ad0d6ac9786bac
    resource: repo://src/segment/model.rs
  - id: openwiki-source-7a7338bb32b984db689c58ba
    resource: repo://statusline-command.sh
  - id: openwiki-source-d6e43e19ed4d1ddc97fba7dc
    resource: repo://tests/render_golden.rs
generated: {by: "openwiki/0.4.0", at: "2026-08-26T22:48:34.063Z"}
---

# Security: terminal-injection hardening

claudebar renders a statusline from host-provided strings that reach the
terminal. A hostile value smuggled through one of those strings — a
directory, a git branch, a model display name — can carry **ANSI/OSC escape
sequences** that would otherwise be interpreted by the terminal as control
input (cursor movement, OSC 8 hyperlinks, "ignore" cancels, even terminal
title changes). The render path treats this as an injection surface and
strips terminal-control bytes from every such string before it is emitted.

The rule is uniform across both implementations:

- **Rust render path:** `sanitize::strip_control` removes the four
  terminal-control bytes ESC (`\x1b`), BEL (`\x07`), CR (`\r`), and LF (`\n`)
  from host-provided strings before they enter the rendered line.
- **Bash fallback (`statusline-command.sh`) and `install.sh`:** the identical
  byte set is stripped with `${var//[$CTRL]/}`, and every host-provided value
  is always passed as a `printf %s/%d` argument — never interpolated into the
  format string.

## The injection surface and the sanitizer

`strip_control` (`src/sanitize.rs`) is a pure string transform: it filters
out exactly the four bytes that can start or terminate an escape sequence.
It deliberately does *not* attempt to parse or validate escape sequences — it
removes the trigger characters so no malformed sequence can begin.

```rust
pub fn strip_control(s: &str) -> String {
    s.chars()
        .filter(|&c| c != '\x1b' && c != '\x07' && c != '\r' && c != '\n')
        .collect()
}
```

Because the filter is applied per `char` (a Rust scalar value, never a raw
byte), it is safe on multi-byte UTF-8 input and never splits a codepoint. The
output is a `String`, so a fully-stripped value collapses to `""`, which
segments then treat as absent (they filter empty results out).

## Where sanitization is applied

Sanitization happens **at render time, per segment** — not at parse time.
`InputData::parse` (`src/model/input.rs`) is a plain `serde_json` deserialize
with no sanitization; the raw host strings flow through the parsed model
unmodified and only become safe when a segment calls `strip_control` on the
value it is about to emit. This keeps parsing dumb and places the defense at
the single emission point.

Every segment that renders a host-provided string must sanitize it first:

| Segment | Host-provided field stripped | Evidence |
|---|---|---|
| Directory | `cwd` (via `abbreviate_path`, which calls `strip_control`) | `src/segment/directory.rs` |
| Git | branch name | `src/segment/git.rs` |
| Model | `model.display_name`, `effort.level` | `src/segment/model.rs` |
| Dev-context | worktree name, `agent.name`, `pr.review_state` | `src/segment/dev_context.rs` |

The git segment is the primary injection surface: the branch name comes from
subprocess output (`git status --branch --porcelain`), so a repository can
host a malicious name. `parse_status` runs the branch through
`strip_control` and returns `None` if the result is empty — a fully-stripped
branch hides the segment rather than emitting a blank/detached line.

## Enforcement inside the writer

`SegmentWriter::colored` (`src/render/writer.rs`) emits its text **verbatim**
into the output buffer. It documents the invariant directly on the method:
callers must pre-sanitize host-provided strings with `strip_control`. This
means the defense is a *caller contract* rather than a second filter inside
the writer — the writer deliberately stays a dumb buffer so segments have one
place to enforce it, and the writer never guesses what is a "safe" string.

## The golden injection test and fixture

The end-to-end guarantee is pinned by `fixtures/injection.json`, whose `cwd`
and `model.display_name` carry ESC, BEL, CR, and LF:

```json
{
  "cwd": "/home/me/\u001b[31mevil/proj\u0007\r\n",
  "model": { "display_name": "Evil\u001b[5mModel" }
}
```

`injection_no_control_byte_leak` (`tests/render_golden.rs`) renders this
fixture through the full pipeline, strips only the renderer's own SGR runs
(`\x1b[...m`) by hand, and asserts the remaining residue contains **none** of
ESC/BEL/CR/LF. It is the explicit, end-to-end version of the per-segment unit
tests (`strips_control_bytes_from_branch` in git, `strips_injection_bytes`
in directory and model).

## The bash statusline and installer follow the same rule

`statusline-command.sh` defines the strip set once and applies it to each
host string:

```bash
CTRL=$'\e\a\r\n'   # terminal-control bytes stripped from host strings (injection guard)
...
fp=${fp//[$CTRL]/}          # directory
branch=${branch//[$CTRL]/}  # git branch
model_name=${model_name//[$CTRL]/}
effort_level=${effort_level//[$CTRL]/}
```

The file opens with a `shellcheck disable=SC2059` comment explaining that the
pre-defined ANSI color constants deliberately live in the `printf` format
string, while *every host-provided value* is passed as a `%s/%d` argument —
so the colored `printf` usage is an intentional, safe pattern rather than a
format-string defect. `install.sh` applies the same discipline: host values
such as `RELEASE_CHANNEL` are rendered through `printf '\033[31mUnknown
CLAUDEBAR_CHANNEL: %s ...' "$RELEASE_CHANNEL"` — constant format, value as a
`%s` argument.

## Shared formatting helpers (same module)

`src/sanitize.rs` is the *only* home for pure, no-I/O segment helpers, and it
is shared by both hardening and display formatting:

- `abbreviate_path` — fish-style path abbreviation (each component but the
  last shortened to its first char, or first two for a dotfile; `$HOME` →
  `~`). It calls `strip_control` on its result, so the directory segment is
  sanitized through this one helper.
- `fmt_tokens` — token totals as `<1000` raw / `N.Nk` / `N.NM`, with
  round-half-up and carry that can promote `k` → `M` (never past `M`).
- `fmt_reset` — adaptive "time until reset" (`Nd Nh` / `Nh Nm` / `Nm Ns` /
  `Ns`) relative to an injected `now`, returning `None` for absent/past
  resets.

The module is pure — no I/O, no color, no terminal interaction — so the
hardening and formatting helpers are deterministic and unit-testable in
isolation.

## Related

- `/openwiki/architecture/render-pipeline.md` — where sanitized segments
  compose into the final ANSI line.
- `/openwiki/architecture/input-parsing.md` — why the raw (unsanitized) model
  is produced before per-segment sanitization.
- `/openwiki/concepts/segment-seam.md` — the segment contract that makes
  per-emission sanitization the required pattern.
- `/openwiki/concepts/sanitize-formatting.md` — the shared helpers catalogued
  from the formatting side.
- `/openwiki/systems/segments/README.md` — the full segment catalogue and
  where each sanitized field is emitted.
