# Files

- [Cross-session state](cross-session-state.md) - Best-effort local caches that persist across sessions so later renders can account for history: the limit_sync high-water marks, the burn TSV sample cache, the float readout file, and the update-check cache — all written atomically and never allowed to break the render.
