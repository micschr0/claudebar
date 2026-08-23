---
type: "Reference"
title: "CLI reference"
openwiki_generated: true
---

# CLI reference

`Cli` + `Command` enum in `src/cli.rs` (clap derive); dispatch in
`src/main.rs`.

## Subcommands

| Subcommand | Purpose |
|---|---|
| `render` (default) | Read hook JSON from stdin, emit ANSI status line |
| `config` | Open the TUI configurator (`tui` feature; exit 1 without it) |
| `init` | Write a default config |
| `list` | List segments / themes / styles |
| `setup` | Patch Claude Code `settings.json` statusLine |
| `sync` | Add new segments to existing config |
| `doctor` | Environment checks |
| `edit` | Edit config in `$EDITOR` |

## Global flags

- `--config <path>` — override config path
- `--theme <name>` — override theme
- `--style <name>` — override style
- `--segments <list>` — override segments

## Render contract

- Reads stdin fully; `InputData::parse` (infallible).
- Injects `now` from system time (the only ambient read in the render path).
- Prints ANSI to stdout; errors to stderr; exits non-zero on config parse
  failure but still renders with defaults.

## Exit codes

- `0` success
- `1` usage/config error (e.g. `config` without `tui` feature)

## Tests

- `src/cli.rs` unit tests (arg parsing).
- `tests/cli_main_dispatch.rs` (subcommand dispatch matrix).
- `tests/cli_smoke.rs` (stdin/stdout contract).