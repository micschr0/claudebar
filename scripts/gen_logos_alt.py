#!/usr/bin/env python3
"""Three alternative claudebar logo directions — each a completely different
palette AND style, drawn from colours seen in the screenshots' themes:

  warm  — Gruvbox: warm retro, boxy, a blocky level-meter             (#282828)
  soft  — Catppuccin: soft pastel, rounded, a pill bar + knob          (#1e1e2e)
  light — status semaphore: flat, light background, green→amber→red    (#ffffff)

Writes assets/variants/{name}-mark.svg and {name}.svg. Pure SVG (no browser).
"""
import os

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(REPO, "assets", "variants")
os.makedirs(OUT, exist_ok=True)


def svg_mark(tile, glyph, defs=""):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" '
            f'viewBox="0 0 128 128" role="img" aria-label="claudebar logo mark">'
            f'{defs}{tile}{glyph}</svg>\n')


def svg_lockup(panel, tile, glyph, claude_fill, bar_fill, underline, defs=""):
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="520" height="160" '
        'viewBox="0 0 520 160" role="img" aria-label="claudebar">'
        f'{defs}{panel}'
        f'<g transform="translate(24 16)">{tile}{glyph}</g>'
        '<text x="190" y="92" font-family="\'SF Pro Display\',\'Segoe UI\',Helvetica,Arial,sans-serif" '
        f'font-size="56" font-weight="700" letter-spacing="-1">'
        f'<tspan fill="{claude_fill}">claude</tspan><tspan fill="{bar_fill}">bar</tspan></text>'
        f'<rect x="376" y="106" width="82" height="6" rx="3" fill="{underline[0]}"/>'
        f'<rect x="376" y="106" width="50" height="6" rx="3" fill="{underline[1]}"/>'
        '</svg>\n'
    )


# ── warm — Gruvbox retro ────────────────────────────────────────────────────
WARM_DEFS = ''
WARM_TILE = '<rect x="6" y="6" width="116" height="116" rx="18" fill="#282828" stroke="#3c3836" stroke-width="2"/>'
# chunky square-cut chevron + a blocky level-meter (retro)
WARM_GLYPH = (
    '<path d="M 46 32 L 70 56 L 46 80" fill="none" stroke="#fe8019" stroke-width="14" '
    'stroke-linecap="square" stroke-linejoin="miter"/>'
    + ''.join(
        f'<rect x="{26 + i*16}" y="94" width="12" height="14" fill="{c}"/>'
        for i, c in enumerate(["#fb4934", "#fe8019", "#fabd2f", "#b8bb26", "#8ec07c"]))
)
WARM_PANEL = ('<defs><linearGradient id="wp" x1="0" y1="0" x2="0" y2="1">'
              '<stop offset="0" stop-color="#3c3836"/><stop offset="1" stop-color="#282828"/></linearGradient></defs>'
              '<rect x="1" y="1" width="518" height="158" rx="34" fill="url(#wp)" stroke="#504945" stroke-width="2"/>')

# ── soft — Catppuccin pastel ────────────────────────────────────────────────
SOFT_DEFS = ('<defs><linearGradient id="sg" x1="0" y1="0" x2="1" y2="0">'
             '<stop offset="0" stop-color="#cba6f7"/><stop offset="0.5" stop-color="#f5c2e7"/>'
             '<stop offset="1" stop-color="#94e2d5"/></linearGradient></defs>')
SOFT_TILE = '<rect x="6" y="6" width="116" height="116" rx="36" fill="#1e1e2e" stroke="#313244" stroke-width="2"/>'
# soft rounded chevron + pill bar with a peach knob
SOFT_GLYPH = (
    '<path d="M 50 36 L 70 54 L 50 72" fill="none" stroke="#cba6f7" stroke-width="11" '
    'stroke-linecap="round" stroke-linejoin="round"/>'
    '<rect x="30" y="80" width="68" height="20" rx="10" fill="url(#sg)"/>'
    '<circle cx="74" cy="90" r="14" fill="#fab387" stroke="#1e1e2e" stroke-width="3"/>'
)
SOFT_PANEL = ('<defs><linearGradient id="sp" x1="0" y1="0" x2="0" y2="1">'
              '<stop offset="0" stop-color="#1e1e2e"/><stop offset="1" stop-color="#181825"/></linearGradient></defs>'
              '<rect x="1" y="1" width="518" height="158" rx="40" fill="url(#sp)" stroke="#313244" stroke-width="2"/>')

# ── light — status semaphore (flat, light) ──────────────────────────────────
LIGHT_DEFS = ''
LIGHT_TILE = '<rect x="6" y="6" width="116" height="116" rx="26" fill="#ffffff" stroke="#d0d7de" stroke-width="2"/>'
# minimal prompt + three ascending status bars (green→amber→red)
LIGHT_GLYPH = (
    '<path d="M 30 50 L 42 64 L 30 78" fill="none" stroke="#57606a" stroke-width="6" '
    'stroke-linecap="round" stroke-linejoin="round"/>'
    '<rect x="54" y="70" width="13" height="24" rx="6.5" fill="#2da44e"/>'
    '<rect x="75" y="56" width="13" height="38" rx="6.5" fill="#bf8700"/>'
    '<rect x="96" y="44" width="13" height="50" rx="6.5" fill="#cf222e"/>'
)
LIGHT_TILE_CHIP = '<rect x="0" y="0" width="128" height="128" rx="26" fill="#f6f8fa" stroke="#d0d7de" stroke-width="2"/>'
LIGHT_PANEL = '<rect x="1" y="1" width="518" height="158" rx="34" fill="#ffffff" stroke="#d0d7de" stroke-width="2"/>'

CONCEPTS = {
    "warm": dict(defs=WARM_DEFS, tile=WARM_TILE, glyph=WARM_GLYPH, panel=WARM_PANEL,
                 claude="#ebdbb2", bar="#fe8019", underline=("#504945", "#fe8019")),
    "soft": dict(defs=SOFT_DEFS, tile=SOFT_TILE, glyph=SOFT_GLYPH, panel=SOFT_PANEL,
                 claude="#cdd6f4", bar="#cba6f7", underline=("#313244", "#cba6f7")),
    "light": dict(defs=LIGHT_DEFS, tile=LIGHT_TILE, glyph=LIGHT_GLYPH, panel=LIGHT_PANEL,
                  tile_chip=LIGHT_TILE_CHIP, claude="#1f2328", bar="#2da44e",
                  underline=("#d0d7de", "#2da44e")),
}

for name, c in CONCEPTS.items():
    open(os.path.join(OUT, f"{name}-mark.svg"), "w").write(
        svg_mark(c["tile"], c["glyph"], c["defs"]))
    open(os.path.join(OUT, f"{name}.svg"), "w").write(
        svg_lockup(c["panel"], c.get("tile_chip", c["tile"].replace('x="6" y="6" width="116" height="116"', 'x="0" y="0" width="128" height="128"')),
                   c["glyph"], c["claude"], c["bar"], c["underline"], c["defs"]))
    print(f"wrote {name}-mark.svg, {name}.svg")
