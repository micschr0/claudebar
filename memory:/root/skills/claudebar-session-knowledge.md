# claudebar Session Knowledge (2026-06-29)

## Architecture State (final)
- 12 aktive Segmente + 2 deprecated (Project, Effort → no-op)
- ALL: [Directory, Git, Model, Context, RateLimits, DevContext, Stash, Cost, Lines, Duration, Burn, Clock]
- DEFAULT = 8 Core: [Directory, Git, Model, Context, Lines, RateLimits, Cost, Duration]
- Opt-in (4): DevContext, Stash, Burn, Clock
- Deprecated (2): Project, Effort — Varianten bleiben für TOML-Kompat, mappen auf Noop
- model_show_effort: true → Model rendert Effort inline
- weekly_show_at: 75 (Normie-Schutz: Weekly-Bar erst ab 75%, vorher nur 5h-Bar)
- Context rendert bei 0 Tokens (new-user onboarding: Segment von Anfang an sichtbar)
- RenderCtx hat `tz_offset_seconds: i32` (DI-Pattern)
- clock_mode = "auto" (Default), "12h", "24h", "off"; clock_seconds = true (Default)

## CLI (final)
- `claudebar` / `claudebar render` — stdin → statusline
- `claudebar config` — TUI
- `claudebar init [--print] [--force]`
- `claudebar list [--segments]` — themes/styles; --segments zeigt alle 14
- `claudebar sync` — neue Segmente zur Config hinzufügen (war `migrate`)
- `claudebar smoke` — Fixture rendern (war `test`)
- `claudebar doctor` — Font, Git, Config, PATH
- `claudebar edit` — $EDITOR
- `claudebar completions <SHELL>` — bash/zsh/fish
- Global flags: --theme, --style, --segments, --config
- --segments: Tippfehler warnen (nicht still ignorieren)

## Screenshot Pipeline (final)
- Script: `scripts/gen_screenshots.py --strips`
- Font: `/tmp/fonts/HackNerdFontMono-Regular.ttf` (Hack Nerd Font v3.3.0)
- Render: Host Chromium + playwright-core (`/tmp/pw/node_modules`)
- Befehl:
  ```bash
  CLAUDEBAR_CHROME=/usr/local/bin/chromium \
  PW_MODULES=/tmp/pw/node_modules \
  NF_FONT_DIR=/tmp/fonts \
  python3 scripts/gen_screenshots.py --strips
  ```
- Output: `screenshots/strip-{normal,critical,overlimit,green,nogit,noeffort,features}.png`
- Dokumentiert in `.claude/CLAUDE.md`

## Video Pipeline (final)
- hve-spielberg: 6-Phasen-Pipeline (Promo-Mode, 25s, silent, 1920×1080@30)
- Scenes: `video/scenes/00-05.html` (GSAP-Animationen, Tokyo Night #16161e)
- 00 Logo · 01 Strip reveal (8 Core-Labels) · 02 Live states · 03 Features (6 Chips) · 04 Speed (Datenfluss-Visualisierung) · 05 CTA
### Render-Workflow
1. `cd video && HYPERFRAMES_BROWSER_PATH=<chromium> npx hyperframes render . --fps 30`
2. Output: `video/renders/video_<timestamp>.mp4` (HyperFrames legt timestamp-Datei an)
3. `cp video/renders/video_*.mp4 docs/assets/claudebar-demo.mp4` (Produktion)
4. Alte `video/renders/video_*.mp4` + `video/renders/work-*/` löschen (nur aktuellen Render behalten)
5. `git add docs/assets/claudebar-demo.mp4 && git commit && git push`
- NIE `video/out/final.mp4` verwenden (veraltet)
- Nur `docs/assets/claudebar-demo.mp4` ist die Landing-Page-Quelle

## Release-Profil (final)
```toml
[profile.release]
opt-level = 3
lto = "fat"
panic = "abort"
strip = true
codegen-units = 1
```

## Lint-Attrs (lib.rs)
```rust
#![deny(clippy::correctness)]
#![warn(clippy::suspicious, clippy::style, clippy::complexity, clippy::perf)]
```

## Rust-Audit-Score: 79/100
- PARTIAL: panic="abort" gefehlt (✅ fixed), lto="thin"→"fat" (✅ fixed), opt-level 2→3 (✅ fixed), crate-level lints gefehlt (✅ fixed)
- DONE: 17 Kategorien (Ownership, Error-Handling, API-Design, Serde, Testing, Anti-Patterns, …)

## TUI — Lessons Learned
### Kritischer Bug: draw_right_panel render_widget fehlte
- Beim Einbau der Scroll-Indikatoren wurde `f.render_widget(Paragraph::new(Text::from(visible)), item_area)` ENTFERNT
- Symptom: rechter Panel komplett leer (nur Border + Footer)
- Fix: Zeile wiederherstellen NACH den Scroll-Indikator-Modifikationen
- Commit: `379e865`

### P3-Subagents — Vorsicht bei parallelen TUI-Änderungen
- gap_span doppelt verwendet → clone nötig. Fix: gap_a + gap_b statt clone
- context_help außerhalb `impl App` definiert → Compile-Fehler
- render_style_row: line-Rückgabe fehlte → Compile-Fehler
- Safe-Integration: erst app.rs-Änderungen, dann ui.rs, dann draw_status-Verdrahtung

## README — 3-Expert-Review Findings
- 7 kritische technische Fehler (6→8 default, 6→7 styles, migrate→sync, test→smoke, opt-in-Tabelle falsch)
- 4 strukturelle Probleme (Screenshots vor Features, Install zu spät, kein Verify-Schritt, 7→4 Screenshots reduziert)
- Kompletter Rewrite in `867c01d`

## Branch-Konvention
- KEINE externen Referenzen („coralline") in Code, Commits, Branch-Namen
- Branch-Name: `feat/latest-improvements`
- Main-Branch: NIE direkt pushen, nur über PR

## time Crate Integration
- `time = { version = "0.3", default-features = false, features = ["std", "formatting", "macros"] }`
- NICHT `UtcOffset::current_local_offset()` — blockiert in gVisor/sandboxed Umgebungen
- TZ-Offset: `date +%z` Subprocess via `detect_tz_offset()`, LazyLock-cached

## Erledigt (2026-06-29)
✅ 14→12 Segmente, Project+Effort deprecated, Noop für backward compat
✅ DEFAULT 8: Directory·Git·Model·Context·Lines·RateLimits·Cost·Duration
✅ 5-Runden Experten-Debatte (CC·Dev·Normie) → Konvergenz auf aktuellen DEFAULT
✅ weekly_show_at: 50→75 (Normie-Tag-1-Schutz)
✅ Context rendert bei 0 Tokens (new-user onboarding)
✅ Lines in DEFAULT befördert (selbst-unterdrückend, harmlos)
✅ Burn: Theme-Slots statt Hardcodes
✅ 4 Themes Deuteranopie-fix + 7 Kollisionen + 5 bar_track
✅ Icons: Duration ⏱ (U+23F1), Agent ⚙ (U+2699)
✅ TUI: h/l-Navigation, clock_mode/layout, Scroll-Indikatoren, Swatch-Cache, Tooltips
✅ CLI: stdin-Check, smoke, sync, completions, --segments-Warnung
✅ install.sh: Nerd-Font-Auto-Detect
✅ README: Komplett-Rewrite (3-Experten-Review)
✅ Screenshots: 7 Strips via gen_screenshots.py
✅ Video: 6 Scenes, Scene 04 Datenfluss-Visualisierung, Render-Workflow, .gitlab-ci.yml
✅ Rust-Audit: Release-Profil, Lint-Attrs, panic="abort", lto="fat"
✅ Code-Review: Rust Skills + CleanCode + TestReview — 0.92 confidence, Nits gefixt
✅ Branch: feat/latest-improvements, 213 Tests, Clippy clean
