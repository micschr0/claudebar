# claudebar Session Knowledge (2026-06-27)

## Architecture State
- 14 Segmente: Directory, Git, Context, RateLimits, DevContext, Model, Effort, Clock, Cost, Lines, Duration, Stash, Project, Burn
- DEFAULT = alle 14 in neuer Reihenfolge: Clock, Project, Dir, Git, Stash, Context, RL, DevCtx, Model, Effort, Cost, Lines, Duration, Burn
- RenderCtx hat `tz_offset_seconds: i32` (DI-Pattern)
- clock_mode = "auto" (Default), "12h", "24h", "off"
- clock_seconds = true (Default)

## time Crate Integration
- `time = { version = "0.3", default-features = false, features = ["std", "formatting", "macros"] }`
- Formatierung: `format_description!("[hour repr:12]:[minute] [period case:lower]")` etc. — 4 const Format-Beschreibungen
- TZ-Offset: `date +%z` Subprocess via `detect_tz_offset()`, LazyLock-cached
- NICHT `UtcOffset::current_local_offset()` — blockiert in gVisor/sandboxed Umgebungen
- `OffsetDateTime::from_unix_timestamp(ctx.now).to_offset(offset).format(fmt)` im Render-Pfad

## 12h/24h Detection
- Country-Code-Tabelle (TWELVE_H_COUNTRIES) aus LC_TIME/LC_ALL/LANG
- fr_CA override → 24h
- C/C.UTF-8/POSIX → 24h
- LazyLock-cached: PREFERS_12H

## CLI Additions
- `--segments <kebab,case,list>` global flag auf Cli — comma-separated via clap `value_delimiter`
- `SegmentKind::from_kebab(s: &str) -> Option<Self>` (serde JSON roundtrip)
- `claudebar test` — render_line mit InputData::default() + Config::default()
- `claudebar doctor` — check: Nerd Font (fc-list + dir scan), git, config parse
- `claudebar edit` — $EDITOR auf config path
- `claudebar list --segments` — 14 Segmente mit kebab-case/Label/Default

## Icon Fixes
- Clock: ⊙ (U+2299) → ◷ (U+25F7) in unicode.rs
- Burn ASCII: ^ → B
- Project ASCII: # → P
- Alle 7 Style-Dateien angepasst

## Color Fixes
- Cost: bar_crit (rot) → warm amber/gold in ALLEN 16 Themes
- Nord: effort 139→133 (distinct von model)
- Dracula: effort 212→135 (distinct von git_branch)
- Everforest: duration 108→116 (distinct von clock)

## Screenshot Pipeline
- HTML-Generierung: Python mit base64-embedded Hack Nerd Font
- Rendering: Browser-Tool `tab.screenshot({ fullPage: true, save: path })`
- 7 Strips in video/public/screenshots/ + screenshots/: normal, critical, overlimit, green, nogit, noeffort, features
- features-Strip: `--segments directory,git,context,rate-limits,model,cost,lines,duration,clock`
- gen_screenshots.py: run_sl() erweitert um cost/lines_added/lines_removed/duration_ms/segments kwargs

## Noch offen (5 Items)
- P2: Project icon ⬢→⎔ (U+2394) — ✅ erledigt (2026-06-28)
- P3: TUI theme swatches + glyph preview (ratatui Farbquadrate neben Theme-Namen)
- P3: TUI "Try before Save"-Modus (Preview ohne Commit)
- P3: TUI Segment-Hilfetexte / Threshold-Tooltips (?-Overlay)
- P3: Nerd-Font-Auto-Detect bei Install (install.sh check)

## Gate Status
- 220 Tests pass
- cargo clippy -- -D warnings clean
- cargo check --no-default-features clean
- cargo fmt --check clean
- Release build mit LTO + strip

## Constraints
- Keine neuen Crate-Dependencies außer `time` (bewusste Ausnahme)
- `date +%z` Subprocess für TZ (kein `chrono`, kein `iana-time-zone`)
- `OnceLock`/`LazyLock` aus std (Rust 1.80+)
- Feature gate: TUI hinter `#[cfg(feature = "tui")]`
- Segmente lesen nie Environment direkt — alles via RenderCtx


## Review Findings (2026-06-28)
### Effort-Duplizierung
- Model-Segment rendert Effort inline: `◈ Opus 4.8 \u{f0e7} high`
- Effort-Segment rendert separat: `\u{f0e7}  high`
- Beide sind in DEFAULT → doppelte Anzeige: `◈ Opus 4.8 \u{f0e7} high  \u{f0e7}  high`
- Fix-Optionen:
  a) Effort aus DEFAULT entfernen (Model zeigt es bereits)
  b) Effort aus Model-Segment entfernen (nur separates Segment)
  c) Model prüft, ob Effort-Segment in der Liste ist, und unterdrückt dann Inline-Rendering
- UI/UX-Review-Teams wurden beauftragt (2026-06-28)


## Erledigt (2026-06-28)
- P2: Project icon ⬢→⎔ (U+2394) ✅
- P2: Duration icon ⧖→⏱ (U+23F1) ✅
- P2: Agent/DevContext icon → ⚙ (U+2699) ✅
- P0: Burn-Segment nutzt Theme-Slots statt Hardcodes ✅
- P0: 4 Themes Deuteranopie-fix (solarized, everforest, kanagawa, github) ✅
- P0: 7 model/effort/git_branch Kollisionen behoben ✅
- P0: 5 bar_track Sichtbarkeit verbessert ✅
- P0: DEFAULT 14→8 Core-Segmente, Effort-Duplizierung behoben ✅
- P0: TUI h/l-Überladung getrennt, clock_mode/layout editierbar ✅
- P0: CLI: stdin-Terminal-Check, --list-segments→--segments, test→smoke, migrate→sync ✅
- P1: CLI: --segments-Warnung für unbekannte Namen, init→stdout ✅
- P1: CLI: completions-Befehl hinzugefügt ✅
- Neue Screenshots (7 Strips) ✅

## Noch offen (4 Items)
- P3: TUI theme swatches + glyph preview (ratatui Farbquadrate neben Theme-Namen)
- P3: TUI "Try before Save"-Modus (Preview ohne Commit)
- P3: TUI Segment-Hilfetexte / Threshold-Tooltips (?-Overlay)
- P3: Nerd-Font-Auto-Detect bei Install (install.sh check)
