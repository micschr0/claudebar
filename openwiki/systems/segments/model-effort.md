# Model-effort segment

Model name + effort level.

- Input: `model.id`/`display_name`, `effort.level` (presence-gated).
- Color by level: low/medium dim, high ok, xhigh warn, max effort accent.
- Emits nothing when effort absent (or model absent).
- Source: `src/segment/model.rs`.
- Tests: level-color table, presence gates.