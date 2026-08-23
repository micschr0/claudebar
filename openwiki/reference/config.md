---
type: "Reference"
title: "Config reference"
openwiki_generated: true
---

# Config reference

TOML at `$XDG_CONFIG_HOME/claudebar/config.toml` (fallback
`~/.config/claudebar/config.toml`). Managed by `src/model/config.rs`.

## Top-level keys

| Key | Default | Meaning |
|---|---|---|
| `theme` | `"tokyo-night"` | theme name |
| `style` | `"powerline"` | style name |
| `segments` | `DEFAULT` | ordered segment list (kebab-case) |
| `layout` | `"fixed"` | `fixed` / `auto` |
| `max_lines` | — | auto-wrap line cap |
| `wrap_margin` | — | auto-wrap margin |
| `thresholds` | object | see threshold-semantics |

## Segments list

`segments = ["directory", "git", "context", "rate-limits", "model-effort",
"dev-context", "cost", "lines", "duration", "clock", "burn"]`
(kebab-case, `#[serde(rename_all = "kebab-case")]`).

## Load/save semantics

- **Missing file** → `Config::default()` (no error).
- **Malformed TOML** → `ConfigError::Parse` surfaced to stderr; `main.rs`
  warns and continues with defaults.
- `default_path` resolution honors `$XDG_CONFIG_HOME`, falls back to
  `~/.config`.
- `Config::load` / `Config::save` are `#[must_use]`.

## Partial input

Every struct field carries `#[serde(default)]` — a partial TOML always
parses; absent keys keep defaults.

## sync semantics

`claudebar sync` inserts new segments into an existing `segments` list at
canonical positions, preserving user order.

## Inert keys

`name_max` and `model_show_effort` are **declared but not consumed** by any
renderer — inert config surface, no behavior. Do not rely on them.

## Tests

- `src/model/config.rs` unit tests (defaults, partial serde, path
  resolution).