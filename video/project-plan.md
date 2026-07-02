# Project Plan — claudebar promo (README hero)

**Mode:** promo
**Aspect:** 16:9 1920×1080
**Audio:** none (silent, on-screen text — loop-friendly README hero)
**Visual identity strategy:** screenshots (real claudebar strips) + brand palette (Tokyo Night)
**Duration:** 25s
**Created:** 2026-06-24

## Phase Tracker

| Phase | Status | Notes |
|-------|--------|-------|
| 0. Discovery | ✅ done | context.md — audience, message, CTA |
| 1. Storytelling | ✅ done | storyboard.md — 6 scenes, 25s |
| 2. Capture | ✅ done | reused real strips in public/screenshots/ (no recapture needed) |
| 3. Design | ✅ done | brand palette inline in scenes; Tokyo Night |
| 4. Production | ✅ done | scenes/*.html + index.html, lint/inspect/validate — re-rendered 2026-06-29 |
| 5. Render | ✅ done | out/final.mp4 — 1.8 MB, H.264, 1920×1080@30, 25.0s. Copied to docs/assets/claudebar-demo.mp4 |

## Decision Log

| Decision | Rationale |
|----------|-----------|
| README/GitHub hero, 16:9 | primary placement is the repo README + social preview |
| Silent + on-screen text | autoplay-muted on GitHub; loop-friendly; no TTS/music deps |
| Reuse existing strip PNGs | 6 real states already captured + on-brand; skip Chrome capture |
| Tokyo Night palette | matches claudebar's own logo + default theme |
| 25s, 6 scenes | logo → strip → states → features → speed → CTA |
