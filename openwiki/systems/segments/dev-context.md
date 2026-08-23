---
type: "Reference"
title: "Dev-context segment"
openwiki_generated: true
---

# Dev-context segment

Worktree / PR / agent sub-elements with review-state indicators.

- Elements: worktree name (fallback derivation), PR state, agent state/role.
- Review indicators: ✓ / ✗ / ◦ / ·.
- Skips entirely when all sub-elements absent.
- Source: `src/segment/dev_context.rs`.
- Tests: presence matrix, indicator rendering.