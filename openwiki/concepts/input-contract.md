# Input contract

Claude Code invokes `claudebar render --hook` (or `claudebar` directly) with
its session state as JSON on **stdin**. `InputData::parse` converts that into a
typed struct. The parse is **infallible**: any JSON shape — including invalid
JSON — degrades to `InputData::default()`.

## Fields

The hook JSON surfaces these fields (all optional; `#[serde(default)]`):

| Field | Type | Meaning | Gotcha |
|---|---|---|---|
| `cwd` | string | working directory | used by directory/git segments |
| `context_window` | number | token context window | `used_percentage` computed against it |
| `rate_limits` | object | `{used, limit, resets_at, ...}` | epoch seconds; can exceed 100% |
| `model` | object | `{id, display_name}` | model name |
| `effort` | object | `{level}` | presence-based gate |
| `pr` | object | `{state}` | review state indicator |
| `worktree` | object | `{state}` | worktree name fallback |
| `workspace` | object | `{current_dir}` | used in dev context |
| `agent` | object | `{state, role}` | agent/dev-context |
| `cost` | number | session cost USD | hidden at zero |
| `output_style` | object | `{header}` | — |
| `duration_ms` | number | session duration | — |
| `clock` | object | `{now, tz_offset_seconds}` | tz offset for clock segment |

## `Coerce<T>` degradation semantics

`Coerce<T>` is a custom `Deserialize` adapter: it maps wrong-typed or absent
fields to `None` instead of failing the whole parse.

- Number fields: numeric values and numeric strings → `Some`; bools/seqs/maps →
  `None`; JSON garbage → `None`.
- `f64 → i64` conversions are range-checked; out-of-range → `None`.
- The whole hook parse never returns `Err` — worst case is `default()`.

## Edge cases

- **`used_percentage` can exceed 100** (Claude Code reports over-limit
  sessions). The context bar caps at `≤999` and colors shift through warn/crit
  bands.
- **`resets_at` is epoch seconds** — countdown is computed against injected
  `now`, never wall-clock.
- **`effort.level` is presence-based** — absent effort means no effort segment
  element, not "low".
- **`worktree.name()` fallback** — derives a readable name when the raw field
  is missing.
- Control characters from any host string (e.g. a malicious `cwd`) are
  stripped by `sanitize::strip_control` before emission — see
  `concepts/sanitize-formatting.md`.

## Tests

- `src/model/input.rs` unit tests (coercion table, range checks).
- `fixtures/bad_types.json`, `fixtures/injection.json` — wrong types and
  injection attempts.
- `tests/render_golden.rs::injection_no_control_byte_leak` — no ESC/BEL leak
  through the full render path.