# Segments

Each segment is documented on its own page under `systems/segments/`. Common
contract: `render(&RenderCtx, &mut SegmentWriter) -> bool`; `true` = emitted,
`false` = skip (no separator). Output is sanitized; `now`/`home` are injected.

## Index

| Segment | What it shows | Key source | Emits nothing when |
|---|---|---|---|
| [directory](directory.md) | cwd, fish-abbreviation | `src/segment/directory.rs` | — |
| [git](git.md) | branch, ahead/behind, dirty, stash | `src/segment/git.rs` | no git repo |
| [context](context.md) | token usage + bar | `src/segment/context.rs` | — (always) |
| [rate-limits](rate-limits.md) | 5h/weekly windows + countdowns | `src/segment/rate_limits.rs` | no rate-limit data |
| [model-effort](model-effort.md) | model name + effort level | `src/segment/model.rs` | absent effort |
| [dev-context](dev-context.md) | worktree/PR/agent + review state | `src/segment/dev_context.rs` | all sub-elements absent |
| [cost](cost.md) | session cost USD | `src/segment/cost.rs` | zero |
| [lines](lines.md) | +N/−M | `src/segment/lines.rs` | both zero |
| [duration](duration.md) | session duration | `src/segment/duration.rs` | zero |
| [clock](clock.md) | time, 12h/24h, tz | `src/segment/clock.rs` | `clock_mode: off` |
| [burn](burn.md) | burn-down state from cache | `src/segment/burn.rs` | no cache file |

## Cross-session helpers

- `limit_sync.rs` — rate-limit synchronization across idle sessions (see
  [cross-session-state](../cross-session-state.md)).

## Reading a segment page

Each page covers: the input fields it reads, the emission rules, threshold /
theme / style dependencies, and focused tests.