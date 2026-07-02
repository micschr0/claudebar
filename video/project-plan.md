# Project Plan — claudebar promo (README hero)

**Mode:** promo
**Aspect:** 16:9 1920×1080
**Audio:** none (silent, on-screen text — loop-friendly README hero)
**Visual identity strategy:** screenshots (real claudebar strips) + brand palette (Tokyo Night)
**Duration:** 44.1s
**Created:** 2026-06-24

## Phase Tracker

| Phase | Status | Notes |
|-------|--------|-------|
| 0. Discovery | ✅ done | context.md — audience, message, CTA |
| 1. Storytelling | ✅ done | storyboard.md — updated for 5-scene rework (bd679c2, drop live-states) |
| 2. Capture | ✅ done | seg-*/strip-* screenshots in public/screenshots/ cover the 8-segment default + opt-ins |
| 3. Design | ✅ done | brand palette inline in scenes; Tokyo Night |
| 4. Production | ✅ done | scenes/*.html + index.html — lint (0 errors, 5 cosmetic warnings) + validate clean, 2026-07-02 |
| 5. Render | ✅ done | out/final19.mp4 — 4.3 MB, 44.1s, rendered 2026-07-02. Copied to docs/assets/claudebar-demo.mp4 (previous copy was stale, predated the scene rework) |

## Decision Log

| Decision | Rationale |
|----------|-----------|
| README/GitHub hero, 16:9 | primary placement is the repo README + social preview |
| Silent + on-screen text | autoplay-muted on GitHub; loop-friendly; no TTS/music deps |
| Reuse existing strip PNGs | 6 real states already captured + on-brand; skip Chrome capture |
| Tokyo Night palette | matches claudebar's own logo + default theme |
| 25s, 6 scenes | logo → strip → states → features → speed → CTA |
