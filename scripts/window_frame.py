#!/usr/bin/env python3
"""Wrap each frame of an agg-rendered GIF in a macOS-style terminal window:
rounded corners, a titlebar with traffic-light dots, and a dark padded canvas.
Keeps per-frame timing and looping. Usage:

  python3 scripts/window_frame.py IN.gif OUT.gif "title text"
"""
import sys
from PIL import Image, ImageDraw, ImageFont, ImageSequence

IN, OUT = sys.argv[1], sys.argv[2]
TITLE = sys.argv[3] if len(sys.argv) > 3 else "claude"

PAD       = 40           # dark border around the window
TITLEBAR  = 52           # titlebar height
RADIUS    = 18
CANVAS_BG = (13, 13, 20)
BAR_BG    = (31, 32, 53)
BODY_BG   = (26, 27, 38)  # must match agg --theme background
TITLE_FG  = (170, 170, 170)
DOTS      = [((255, 95, 87)), (254, 188, 46), (40, 200, 64)]

def load_font(size):
    for p in ("/tmp/fonts/HackNerdFont-Regular.ttf",
              "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"):
        try:
            return ImageFont.truetype(p, size)
        except OSError:
            continue
    return ImageFont.load_default()

TITLE_FONT = load_font(22)

def frame_window(term_rgb):
    w = term_rgb.width
    win_w, win_h = w, TITLEBAR + term_rgb.height
    cv_w, cv_h = win_w + 2 * PAD, win_h + 2 * PAD

    # Window layer (RGBA) with rounded corners.
    win = Image.new("RGBA", (win_w, win_h), (0, 0, 0, 0))
    d = ImageDraw.Draw(win)
    d.rounded_rectangle([0, 0, win_w - 1, win_h - 1], radius=RADIUS, fill=BODY_BG + (255,))
    # Titlebar (top corners rounded, bottom square).
    d.rounded_rectangle([0, 0, win_w - 1, TITLEBAR + RADIUS], radius=RADIUS, fill=BAR_BG + (255,))
    d.rectangle([0, RADIUS, win_w - 1, TITLEBAR], fill=BAR_BG + (255,))
    d.line([0, TITLEBAR, win_w - 1, TITLEBAR], fill=(22, 23, 42, 255))
    for i, col in enumerate(DOTS):
        cx = 26 + i * 26
        d.ellipse([cx - 7, TITLEBAR // 2 - 7, cx + 7, TITLEBAR // 2 + 7], fill=col + (255,))
    tb = d.textbbox((0, 0), TITLE, font=TITLE_FONT)
    d.text(((win_w - (tb[2] - tb[0])) / 2, (TITLEBAR - (tb[3] - tb[1])) / 2 - tb[1]),
           TITLE, font=TITLE_FONT, fill=TITLE_FG)
    # Terminal content under the titlebar.
    win.paste(term_rgb.convert("RGBA"), (0, TITLEBAR))
    # Re-apply the rounded mask so the pasted content's corners stay rounded.
    mask = Image.new("L", (win_w, win_h), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, win_w - 1, win_h - 1], radius=RADIUS, fill=255)
    win.putalpha(mask)

    canvas = Image.new("RGB", (cv_w, cv_h), CANVAS_BG)
    canvas.paste(win, (PAD, PAD), win)
    return canvas

src = Image.open(IN)
frames, durations = [], []
for fr in ImageSequence.Iterator(src):
    frames.append(frame_window(fr.convert("RGB")))
    durations.append(fr.info.get("duration", 100))

frames[0].save(OUT, save_all=True, append_images=frames[1:],
               duration=durations, loop=0, disposal=2, optimize=True)
print(f"Wrote {OUT}: {len(frames)} frames, {frames[0].size[0]}x{frames[0].size[1]}")
