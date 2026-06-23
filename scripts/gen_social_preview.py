#!/usr/bin/env python3
"""Generate the GitHub social-preview card (1280x640).

Recreates the wordmark from assets/logo.svg (so it stays font-independent),
adds the tagline, and composites a real statusline strip beneath it so the
card shows the product in action.

Writes assets/social-preview-product.png. Requires Pillow.
"""
import os

from PIL import Image, ImageDraw, ImageFont

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASSETS = os.path.join(REPO, "assets")
SHOTS = os.path.join(REPO, "screenshots")

W, H = 1280, 640
TL = (29, 30, 48)   # top-left bg
BR = (18, 19, 30)   # bottom-right bg

# Wordmark colors (from assets/logo.svg, tokyo-night palette)
CLAUDE = (192, 202, 245)   # #c0caf5
BAR = (122, 162, 247)      # #7aa2f7
TRACK = (47, 51, 77)       # #2f334d
GRAD_A = (122, 162, 247)   # #7aa2f7
GRAD_B = (125, 207, 255)   # #7dcfff

FONT_BOLD = "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf"
FONT_REG = "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf"

STRIP = os.path.join(SHOTS, "strip-critical.png")
TAGLINE = "A fast, themeable statusline for Claude Code"


def main():
    # background: diagonal gradient
    bg = Image.new("RGB", (W, H))
    px = bg.load()
    for y in range(H):
        for x in range(W):
            t = (x / W + y / H) / 2
            px[x, y] = tuple(int(TL[i] + (BR[i] - TL[i]) * t) for i in range(3))
    draw = ImageDraw.Draw(bg)

    def text_w(s, f):
        b = draw.textbbox((0, 0), s, font=f)
        return b[2] - b[0]

    # wordmark "claude" + "bar"
    logo_font = ImageFont.truetype(FONT_BOLD, 104)
    cw = text_w("claude", logo_font)
    bw = text_w("bar", logo_font)
    x0 = (W - (cw + bw)) // 2
    y_logo = 150
    draw.text((x0, y_logo), "claude", font=logo_font, fill=CLAUDE)
    draw.text((x0 + cw, y_logo), "bar", font=logo_font, fill=BAR)

    # progress-bar underline beneath "bar" (track + ~62% gradient fill)
    asc, _desc = logo_font.getmetrics()
    uy = y_logo + asc + 18
    uh = 11
    ux = x0 + cw
    uw = bw
    rad = uh // 2

    track = Image.new("RGBA", (uw, uh), (0, 0, 0, 0))
    ImageDraw.Draw(track).rounded_rectangle(
        [0, 0, uw - 1, uh - 1], radius=rad, fill=TRACK + (255,))
    bg.paste(track, (ux, uy), track)

    fw = int(uw * 0.62)
    grad = Image.new("RGBA", (fw, uh))
    gp = grad.load()
    for x in range(fw):
        t = x / max(fw - 1, 1)
        c = tuple(int(GRAD_A[i] + (GRAD_B[i] - GRAD_A[i]) * t) for i in range(3))
        for y in range(uh):
            gp[x, y] = c + (255,)
    mask = Image.new("L", (fw, uh), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, fw - 1, uh - 1], radius=rad, fill=255)
    bg.paste(grad, (ux, uy), mask)

    # tagline
    tag_font = ImageFont.truetype(FONT_REG, 30)
    draw.text(((W - text_w(TAGLINE, tag_font)) // 2, 320), TAGLINE,
              font=tag_font, fill=(120, 124, 145))

    # statusline strip
    strip = Image.open(STRIP).convert("RGBA")
    sw = 1040
    sh = int(strip.height * sw / strip.width)
    strip = strip.resize((sw, sh), Image.LANCZOS)
    bg.paste(strip, ((W - sw) // 2, 412), strip)

    out = os.path.join(ASSETS, "social-preview-product.png")
    bg.save(out)
    print("saved", out, bg.size)


if __name__ == "__main__":
    main()
