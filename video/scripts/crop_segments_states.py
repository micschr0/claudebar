#!/usr/bin/env python3
"""Crop the context/rate-limits per-segment screenshots out of the per-state
strip screenshots, for scene 2's live-states pills.

Companion to crop_segments.py (which crops the 5-segment strip-all.png for
scene 1's default pills). Segment boundaries are found programmatically: a
column is "content" if it differs from the pill's background fill anywhere
in a y-band that avoids the top/bottom border highlight; a run is a
"chevron" if it has a large gap on both sides (the powerline separator glyph
sits alone between adjacent segments' dense text/icon runs).

The strips now include git (5 segments: directory, git, model, context,
rate-limits -- see make_demo_repos.sh for the git states rendered), but this
script only needs the trailing two segments. Their chevron count is stable
regardless of git's variable-width branch text further left, so instead of
requiring an exact total chevron count, it takes the *last two* chevrons
found and crops context (between them) and rate-limits (after the last one
to the strip's end).
"""

from pathlib import Path

from PIL import Image

REPO_ROOT = Path(__file__).resolve().parents[2]
SCREENSHOTS = REPO_ROOT / "video" / "public" / "screenshots"

TOP = 48
BOTTOM = 152
BAND_TOP = 50
BAND_BOTTOM = 151
BG = (19, 20, 31)
DIFF_THRESHOLD = 12
GAP_THRESHOLD = 14
LEFT_PAD = 6
RIGHT_PAD = 6
EDGE_PAD = 14

STATES = ["green", "normal", "critical", "overlimit"]


def content_runs(im: Image.Image) -> list[tuple[int, int, int]]:
    """Return (start, end, gap_before) for each horizontal content run."""
    w = im.width
    px = im.load()

    def is_content(x: int) -> bool:
        for y in range(BAND_TOP, BAND_BOTTOM, 2):
            r, g, b = px[x, y]
            if abs(r - BG[0]) + abs(g - BG[1]) + abs(b - BG[2]) > DIFF_THRESHOLD:
                return True
        return False

    runs: list[list[int]] = []
    start = None
    left, right = 52, w - 52
    for x in range(left, right):
        c = is_content(x)
        if c and start is None:
            start = x
        if not c and start is not None:
            runs.append([start, x])
            start = None
    if start is not None:
        runs.append([start, right])

    # Drop the box's own left/right border runs (thin, right at the scan
    # edges) so they don't get mistaken for real segment content.
    runs = [r for r in runs if not (r[1] - r[0] <= 12 and (r[0] - left <= 15 or right - r[1] <= 15))]

    out = []
    prev_end = left
    for s, e in runs:
        out.append((s, e, s - prev_end))
        prev_end = e
    return out


def find_chevrons(runs: list[tuple[int, int, int]]) -> list[tuple[int, int]]:
    candidates = set()
    for i in range(len(runs) - 1):
        s, e, gap = runs[i]
        _, _, gap2 = runs[i + 1]
        if gap >= GAP_THRESHOLD and gap2 >= GAP_THRESHOLD:
            candidates.add((s, e))

    chevrons = []
    for i, (s, e, _gap) in enumerate(runs):
        if (s, e) not in candidates:
            continue
        if i + 1 < len(runs):
            nxt = (runs[i + 1][0], runs[i + 1][1])
            if nxt in candidates:
                chevrons.append((s, e))

    # A false-positive pair can appear right next to the real chevron (e.g.
    # an isolated icon glyph immediately after it also looks paired).
    # Collapse chevrons closer than 60px together, keeping the first.
    merged: list[tuple[int, int]] = []
    for s, e in chevrons:
        if merged and s - merged[-1][1] < 60:
            continue
        merged.append((s, e))
    return merged


def segment_boxes(runs: list[tuple[int, int, int]]) -> dict[str, tuple[int, int]]:
    chevrons = find_chevrons(runs)
    if len(chevrons) < 2:
        raise ValueError(f"expected at least 2 chevrons, found {len(chevrons)}: {chevrons}")

    # Only the trailing two segments (context, rate-limits) are used by
    # scene 2, so only the last two chevrons matter -- everything to their
    # left (directory/git/model, variable width) is irrelevant here.
    chev_context_end, chev_ratelimits_start = chevrons[-2], chevrons[-1]
    last_content = runs[-1][1]

    return {
        "context": (chev_context_end[1] + LEFT_PAD, chev_ratelimits_start[0] - RIGHT_PAD),
        "rate-limits": (chev_ratelimits_start[1] + LEFT_PAD, last_content + EDGE_PAD),
    }


def main() -> None:
    for state in STATES:
        strip = Image.open(SCREENSHOTS / f"strip-{state}.png").convert("RGB")
        runs = content_runs(strip)
        boxes = segment_boxes(runs)
        for name, (left, right) in boxes.items():
            crop = strip.crop((left, TOP, right, BOTTOM))
            out_path = SCREENSHOTS / f"seg-{name}-{state}.png"
            crop.save(out_path)
            print(f"wrote {out_path} ({crop.size[0]}x{crop.size[1]})")


if __name__ == "__main__":
    main()
