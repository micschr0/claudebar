#!/usr/bin/env python3
"""Crop the 5 default-segment screenshots out of strip-all.png.

The crop boxes below are calibrated to the current strip-all.png (11-segment
ALL order, 3756x198, 2x scale) and must be re-measured if that strip's
layout changes (segment order, font, scale, or padding).

Each box is (left, 32, right, 166) -- a fixed 134px-tall band that trims the
dead vertical padding and the card border while keeping ~12px around the
glyphs (glyph ink spans y=44..153). Horizontal bounds are chosen to sit
between the grey chevron separators so each crop contains exactly one
segment with no chevron bleed.
"""

from pathlib import Path

from PIL import Image

REPO_ROOT = Path(__file__).resolve().parents[2]
SRC = REPO_ROOT / "video" / "public" / "screenshots" / "strip-all.png"
OUT_DIR = REPO_ROOT / "video" / "public" / "screenshots"

TOP = 32
BOTTOM = 166

# name -> (left, right)
BOXES = {
    "directory": (105, 315),
    "git": (340, 618),
    "model": (642, 1100),
    "context": (1130, 1495),
    "rate-limits": (1516, 1883),
}


def main() -> None:
    strip = Image.open(SRC)
    for name, (left, right) in BOXES.items():
        crop = strip.crop((left, TOP, right, BOTTOM))
        out_path = OUT_DIR / f"seg-{name}.png"
        crop.save(out_path)
        print(f"wrote {out_path} ({crop.size[0]}x{crop.size[1]})")


if __name__ == "__main__":
    main()
