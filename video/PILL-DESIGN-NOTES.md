# Pill crops & pill cards — design notes

Findings from the 2026-07-02 session on cropping real claudebar screenshots into
per-segment "pill" cards (scene 1 style) for use across scenes. Read this before
re-running `scripts/crop_segments.py` / `scripts/crop_segments_states.py` or
designing new pill-based scenes.

## Crop boundary detection (the hard part)

Don't eyeball crop boxes from zoomed screenshots — it's slow and produces
systematic errors (text cut off on one edge, chevron `>` bleeding in on the
other). `scripts/crop_segments_states.py` detects boundaries programmatically:

1. Scan a horizontal band (`y=50..151`, deliberately inside the pill's own
   top/bottom border-highlight lines at y≈44-45 and y≈152-153 — including
   those rows makes every column register as "content" and breaks gap
   detection entirely).
2. A column is "content" if any pixel in the band differs from the pill
   background `#13141f` (`(19, 20, 31)`) by more than a small threshold.
3. Collapse into runs; a run is a **chevron candidate** if it has a large gap
   (≥14px) on both sides (isolated). Plain in-segment icons (⚡ lightning,
   clock, hourglass) are also often isolated on *one* side only — that's not
   enough to qualify.
4. A candidate is a **real chevron** only if the *very next* run is *also* a
   candidate (chevron → icon is always a back-to-back isolated pair;
   solo in-segment icons aren't followed by another isolated run).
5. False-positive adjacent pairs still slip through occasionally (e.g. a
   clock icon coincidentally isolated next to a wide-kerned digit) — collapse
   any two detected chevrons closer than 60px into one.
6. Crop boundaries must pull *away* from each chevron, never toward it:
   `segment.right = chevron.start - RIGHT_PAD`, `segment.left = chevron.end +
   LEFT_PAD`. Getting the sign backwards (padding *outward* past the chevron)
   is what caused the very first bleed-in bug.

Even with all that, treat crops as needing a visual contact-sheet check after
every regeneration — don't trust the numbers blindly.

## Pill card styling — what looked wrong and why

- **No gradient card background.** A screenshot crop has a *flat* fill
  (`#13141f`). Pasting it onto a vertical-gradient card body creates a visible
  seam (the crop reads as a "window" instead of blending into the card) —
  violates Gestalt figure-ground unity. Fix: card body = flat, exact same hex
  as the crop background. No seam, no fix needed on the image side.
- **No glow.** A blurred colored glow behind the card looks like a generic
  "neon card" template and doesn't match scene 1's actual style (`4px
  border-top` + a shared `drop-shadow` on the whole composition, no per-card
  glow). Two different card treatments in the same video reads as two
  different design systems.
- **Caption font must be the monospace face** (Hack Nerd Font Mono), not a
  proportional UI font — the rest of the video is monospace throughout;
  switching fonts for captions breaks the typographic hierarchy.
- **Symmetric padding** top/bottom around the embedded screenshot — asymmetric
  padding (tighter below than above) reads as visually unbalanced even when
  the difference is only a few px.
- **One consistent shadow/light source** across scenes — don't mix a
  per-card shadow model in scene 2 with scene 1's single whole-strip shadow.

## Content choice for scene 2 (if rebuilt as pills)

Don't show all 4 segments × 4 states (16 pills) — cognitive overload for a
25s video. Show one pill per state, picking whichever segment carries that
state's signal: `context` for green/normal (calm/normal usage), `rate-limits`
for critical/overlimit (the segment that actually visualizes urgency).
Differentiate critical vs. overlimit via *motion* (the existing GSAP scale-pop
at the overlimit beat), not via a second color or layout — both are already
red by design (see `docs/model/config.rs` thresholds).

## rate-limits segment semantics (for caption/copy accuracy)

The `%` is usage of that limit window (5h or weekly); the time after it
(`45m0s`, `15m0s`) is the **countdown until that window's usage resets to
0%**, from `rate_limits.{five_hour,seven_day}.resets_at` (epoch seconds) in
the input JSON, rendered relative to `now`. Values >100% are real ("overlimit"
state) — the countdown still shows when the window resets even though you're
over it.

## Reusable assets

- `video/scripts/crop_segments.py` — 5-segment crops from `strip-all.png`
  (scene 1's default pills).
- `video/scripts/crop_segments_states.py` — 4-segment crops (no git) from
  `strip-{green,normal,critical,overlimit}.png`, one set per state.
- All `seg-*.png` outputs live in `video/public/screenshots/` (gitignored,
  regenerate via the scripts above — don't hand-edit).
