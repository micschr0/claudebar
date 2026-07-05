---
status: complete
phase: quick-260704-cuq
plan: 01
started: 2026-07-04T09:45:00Z
completed: 2026-07-04T09:49:23Z
---

# Quick Task 260704-cuq: Fix Right-Edge Clipping in Animated Strips

**Result:** Pass

## What Changed

- `scripts/gen_screenshots.py`: Added `ANIM_ORIGIN_LEFT_FRAC` (0.10) and `ANIM_OVERSHOOT_BUFFER` (1.2) constants. `render_animated_strips()` now measures `.stripwrap`'s live width via Playwright `getBoundingClientRect()` before the scale loop and applies a computed `paddingRight` on `.frame` — `pad = Math.ceil(w * overshoot_frac)` where `overshoot_frac = (1.12 - 1.0) * 0.90 * 1.2 = 0.1296`. The padding absorbs the 1.12x/10%-origin peak-scale overshoot.
- `screenshots/strip-critical.png`, `strip-overlimit.png`, `strip-nogit.png`: Regenerated with no right-edge clipping at peak zoom. Measured pads: critical=215px, overlimit=180px, nogit=136px.

## What Stayed the Same

- `ANIM_PEAK_SCALE` (1.12), `ANIM_N_FRAMES` (24), `ANIM_FPS` (6), `transform-origin:10% 50%` — unchanged
- `STRIP_CSS` shared `.strip{min-width:960px}` — untouched
- 4 static strips reverted via `git checkout --` — byte-identical to committed versions
- `README.md` — no changes

## Verification

- `python3 -m py_compile scripts/gen_screenshots.py` — passes
- All 9 grep invariants confirmed (PEAK_SCALE, N_FRAMES, FPS, transform-origin, min-width, new constants, paddingRight, getBoundingClientRect)
- All 3 animated strips: 46 frames, acTL chunk present, 7.7s loop — confirmed
- Peak frame (frame_023) right-edge content: 100% non-BG pixels in last 5px column — no clipping
- `git status --short screenshots/` — exactly 3 animated strips modified; static strips clean

## Commits

- `ccec2cf` fix(screenshots): pad anim frame for zoom overshoot
- `43fa094` fix(screenshots): regen animated strips with right-edge padding
