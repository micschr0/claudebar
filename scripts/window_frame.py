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

def empty_body(term):
    """True if the terminal frame is essentially blank (just background) — used
    to drop asciinema's initial empty frame so the loop doesn't flash."""
    small = term.resize((120, 70)); px = small.load(); c = 0
    for y in range(70):
        for x in range(120):
            r, g, b = px[x, y]
            if abs(r - BODY_BG[0]) > 28 or abs(g - BODY_BG[1]) > 28 or abs(b - BODY_BG[2]) > 28:
                c += 1
    return c < 30

src = Image.open(IN)
raw = [(fr.convert("RGB"), fr.info.get("duration", 100))
       for fr in ImageSequence.Iterator(src)]
# Drop the empty frames asciinema records at session start and after exit, so the
# loop neither flashes blank nor holds a blank screen at the end.
while len(raw) > 1 and empty_body(raw[0][0]):
    raw.pop(0)
while len(raw) > 1 and empty_body(raw[-1][0]):
    raw.pop()
# Hold the final (content) frame a beat so the loop reads cleanly.
raw[-1] = (raw[-1][0], max(raw[-1][1], 2500))

frames = [frame_window(term) for term, _ in raw]
durations = [dur for _, dur in raw]

# Quantise every frame to ONE shared palette and write FULL (un-optimised)
# frames. Optimised diff-frames + disposal break in strict GIF viewers
# (GitHub/browsers clear to background and show only the diff → flicker); full
# self-contained frames render identically everywhere. The richest frame (last)
# sources the palette so every colour is covered.
master = frames[-1].convert("P", palette=Image.ADAPTIVE, colors=255)
pal = [f.quantize(palette=master, dither=Image.NONE) for f in frames]
pal[0].save(OUT, save_all=True, append_images=pal[1:], duration=durations,
            loop=0, disposal=1, optimize=False)
print(f"Wrote {OUT}: {len(frames)} frames, {frames[0].size[0]}x{frames[0].size[1]}")
