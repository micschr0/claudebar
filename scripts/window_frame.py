#!/usr/bin/env python3
"""Wrap each frame of an agg-rendered GIF in a macOS-style terminal window:
rounded corners, a titlebar with traffic-light dots.

  python3 scripts/window_frame.py IN.gif OUT.{gif,png} "title" [--no-hold]

Output format follows the extension:
  .gif  → opaque dark canvas, shared-palette full frames
  .png  → APNG with a TRANSPARENT canvas + soft drop shadow (smooth rounded
          corners, floats on any README background)

--no-hold keeps every frame's own duration (use for a seamless spinner loop);
without it the final content frame is held briefly (nicer for step-through demos).
"""
import sys
from PIL import Image, ImageDraw, ImageFont, ImageSequence, ImageFilter

args = [a for a in sys.argv[1:] if not a.startswith("--")]
flags = {a for a in sys.argv[1:] if a.startswith("--")}
IN, OUT = args[0], args[1]
TITLE = args[2] if len(args) > 2 else "claude"
APNG = OUT.endswith(".png")
HOLD = "--no-hold" not in flags

PAD       = 40
TITLEBAR  = 52
RADIUS    = 18
CANVAS_BG = (13, 13, 20)
BAR_BG    = (31, 32, 53)
BODY_BG   = (26, 27, 38)  # must match agg --theme background
TITLE_FG  = (170, 170, 170)
DOTS      = [(255, 95, 87), (254, 188, 46), (40, 200, 64)]


def load_font(size):
    for p in ("/tmp/fonts/HackNerdFont-Regular.ttf",
              "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"):
        try:
            return ImageFont.truetype(p, size)
        except OSError:
            continue
    return ImageFont.load_default()


TITLE_FONT = load_font(22)


def window(term_rgb):
    """Build the opaque rounded terminal window (RGBA) for one frame."""
    win_w, win_h = term_rgb.width, TITLEBAR + term_rgb.height
    win = Image.new("RGBA", (win_w, win_h), (0, 0, 0, 0))
    d = ImageDraw.Draw(win)
    d.rounded_rectangle([0, 0, win_w - 1, win_h - 1], radius=RADIUS, fill=BODY_BG + (255,))
    d.rounded_rectangle([0, 0, win_w - 1, TITLEBAR + RADIUS], radius=RADIUS, fill=BAR_BG + (255,))
    d.rectangle([0, RADIUS, win_w - 1, TITLEBAR], fill=BAR_BG + (255,))
    d.line([0, TITLEBAR, win_w - 1, TITLEBAR], fill=(22, 23, 42, 255))
    for i, col in enumerate(DOTS):
        cx = 26 + i * 26
        d.ellipse([cx - 7, TITLEBAR // 2 - 7, cx + 7, TITLEBAR // 2 + 7], fill=col + (255,))
    tb = d.textbbox((0, 0), TITLE, font=TITLE_FONT)
    d.text(((win_w - (tb[2] - tb[0])) / 2, (TITLEBAR - (tb[3] - tb[1])) / 2 - tb[1]),
           TITLE, font=TITLE_FONT, fill=TITLE_FG)
    win.paste(term_rgb.convert("RGBA"), (0, TITLEBAR))
    mask = Image.new("L", (win_w, win_h), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, win_w - 1, win_h - 1], radius=RADIUS, fill=255)
    win.putalpha(mask)
    return win


def frame(term_rgb):
    win = window(term_rgb)
    cv = (win.width + 2 * PAD, win.height + 2 * PAD)
    if APNG:
        canvas = Image.new("RGBA", cv, (0, 0, 0, 0))
        shadow = Image.new("RGBA", cv, (0, 0, 0, 0))
        ImageDraw.Draw(shadow).rounded_rectangle(
            [PAD, PAD + 12, PAD + win.width - 1, PAD + win.height - 1 + 12],
            radius=RADIUS, fill=(0, 0, 0, 110))
        shadow = shadow.filter(ImageFilter.GaussianBlur(18))
        canvas = Image.alpha_composite(canvas, shadow)
        canvas.paste(win, (PAD, PAD), win)
        return canvas
    canvas = Image.new("RGB", cv, CANVAS_BG)
    canvas.paste(win, (PAD, PAD), win)
    return canvas


def empty_body(term):
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
while len(raw) > 1 and empty_body(raw[0][0]):
    raw.pop(0)
while len(raw) > 1 and empty_body(raw[-1][0]):
    raw.pop()
if HOLD:
    raw[-1] = (raw[-1][0], max(raw[-1][1], 2500))

frames = [frame(term) for term, _ in raw]
durations = [dur for _, dur in raw]
if not HOLD:
    # Even out the cadence for a seamless spinner loop — asciinema records a long
    # idle on the final frame that would otherwise freeze the rotation each loop.
    durations = [min(d, 320) for d in durations]

if APNG:
    frames[0].save(OUT, save_all=True, append_images=frames[1:], duration=durations,
                   loop=0, disposal=1, blend=0, format="PNG")
else:
    master = frames[-1].convert("P", palette=Image.ADAPTIVE, colors=255)
    pal = [f.quantize(palette=master, dither=Image.NONE) for f in frames]
    pal[0].save(OUT, save_all=True, append_images=pal[1:], duration=durations,
                loop=0, disposal=1, optimize=False)
print(f"Wrote {OUT}: {len(frames)} frames, {frames[0].size[0]}x{frames[0].size[1]}")
