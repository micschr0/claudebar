# Claude Code integration

How claudebar wires into Claude Code's `statusLine` and stays in sync with
`settings.json` — all in `src/setup.rs` + dispatch in `src/main.rs`.

## Subcommands

| Subcommand | Purpose |
|---|---|
| `setup` | Find/backup/apply the `statusLine.command` patch |
| `sync` | Add new segments to an existing config in canonical positions |
| `doctor` | Check PATH/font/git/config/statusLine |
| `edit` | Open config in `$EDITOR`, init-if-missing |

## setup flow

1. Resolve settings path (Claude Code `settings.json`).
2. `load_settings` — strict parse.
3. Classify: `AlreadyConfigured` / `WillSet` / `Conflict`.
4. Backup, apply, save `desired_status_line(binary_path)`.

## doctor checks

- PATH contains claudebar
- Nerd Font present (powerline glyphs)
- git available (git segment)
- config parses
- statusLine points at the binary

## Sync semantics

`sync` inserts newly added segments at their canonical positions in an
existing `segments` list, preserving user order for already-present segments.

## Tests

- `setup.rs` unit tests (classification, patch construction).
- `tests/cli_main_dispatch.rs` (dispatch wiring).