---
type: "Reference"
title: "Clock segment"
openwiki_generated: true
---

# Clock segment

Current time with 12h/24h detection and tz offset.

- 12h/24h detection: `LC_TIME`/`LC_ALL`/`LANG` + country table.
- tz offset: via `time` crate `LazyLock`, or injected `clock.tz_offset_seconds`.
- `clock_mode`: `auto` / `12h` / `24h` / `off` (off → hidden).
- Source: `src/segment/clock.rs`.
- Tests: locale table, mode overrides, offset rendering.