#!/usr/bin/env python3
"""Generate the claudebar logo variants (tokyo-night palette).

For each variant it writes, under assets/variants/:
  vN-mark.svg  — square app-icon mark (self-contained dark tile)
  vN.svg       — full wordmark lockup on a dark panel (readable on light or dark)

PNG rasters are produced separately (see the render step in the repo history);
this generator is pure SVG and needs no browser.
"""
import os

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(REPO, "assets", "variants")
os.makedirs(OUT, exist_ok=True)

DEFS = (
    '<defs>'
    '<linearGradient id="cv" x1="0" y1="0" x2="0" y2="1">'
    '<stop offset="0" stop-color="#bb9af7"/><stop offset="1" stop-color="#7aa2f7"/></linearGradient>'
    '<linearGradient id="br" x1="0" y1="0" x2="1" y2="0">'
    '<stop offset="0" stop-color="#7aa2f7"/><stop offset="1" stop-color="#7dcfff"/></linearGradient>'
    '<linearGradient id="panel" x1="0" y1="0" x2="0" y2="1">'
    '<stop offset="0" stop-color="#1d1e30"/><stop offset="1" stop-color="#181926"/></linearGradient>'
    '</defs>'
)
TILE = '<rect x="6" y="6" width="116" height="116" rx="30" fill="#1a1b2e" stroke="#2a2b3d" stroke-width="2"/>'

# Each glyph is drawn in the 128×128 tile coordinate space.
GLYPHS = {
    "1": (  # prompt chevron + statusline bar
        '<path d="M 50 36 L 74 58 L 50 80" fill="none" stroke="url(#cv)" stroke-width="12" '
        'stroke-linecap="round" stroke-linejoin="round"/>'
        '<rect x="40" y="90" width="48" height="11" rx="5.5" fill="#2f334d"/>'
        '<rect x="40" y="90" width="30" height="11" rx="5.5" fill="url(#br)"/>'
    ),
    "2": (  # powerline segments — the literal statusline
        '<polygon points="26,48 56,48 70,64 56,80 26,80" fill="#7aa2f7"/>'
        '<polygon points="52,48 80,48 94,64 80,80 52,80" fill="#bb9af7"/>'
        '<polygon points="78,48 100,48 104,64 100,80 78,80" fill="#7dcfff"/>'
    ),
    "3": (  # equalizer / status bars
        '<rect x="34" y="62" width="14" height="32" rx="7" fill="#9ece6a"/>'
        '<rect x="54" y="44" width="14" height="50" rx="7" fill="#7dcfff"/>'
        '<rect x="74" y="34" width="14" height="60" rx="7" fill="#7aa2f7"/>'
        '<rect x="94" y="54" width="14" height="40" rx="7" fill="#bb9af7"/>'
    ),
    "4": (  # context ring gauge + chevron
        '<circle cx="64" cy="64" r="34" fill="none" stroke="#2f334d" stroke-width="10"/>'
        '<circle cx="64" cy="64" r="34" fill="none" stroke="url(#br)" stroke-width="10" '
        'stroke-linecap="round" stroke-dasharray="150 64" transform="rotate(135 64 64)"/>'
        '<path d="M 56 50 L 72 64 L 56 78" fill="none" stroke="url(#cv)" stroke-width="9" '
        'stroke-linecap="round" stroke-linejoin="round"/>'
    ),
    "5": (  # terminal window with a statusline
        '<rect x="28" y="32" width="72" height="64" rx="11" fill="#24283b"/>'
        '<circle cx="40" cy="44" r="3.2" fill="#f7768e"/>'
        '<circle cx="51" cy="44" r="3.2" fill="#e0af68"/>'
        '<circle cx="62" cy="44" r="3.2" fill="#9ece6a"/>'
        '<path d="M 39 60 L 49 67 L 39 74" fill="none" stroke="url(#cv)" stroke-width="5" '
        'stroke-linecap="round" stroke-linejoin="round"/>'
        '<rect x="56" y="64" width="28" height="6" rx="3" fill="#3b4261"/>'
        '<rect x="36" y="82" width="56" height="7" rx="3.5" fill="url(#br)"/>'
    ),
}

def mark(glyph):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" '
            f'viewBox="0 0 128 128" role="img" aria-label="claudebar logo mark">'
            f'{DEFS}{TILE}{glyph}</svg>\n')

def lockup(glyph):
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="520" height="160" '
        'viewBox="0 0 520 160" role="img" aria-label="claudebar">'
        f'{DEFS}'
        '<rect x="1" y="1" width="518" height="158" rx="34" fill="url(#panel)" stroke="#2a2b3d" stroke-width="2"/>'
        f'<g transform="translate(24 16)">{TILE}{glyph}</g>'
        '<text x="190" y="92" font-family="\'SF Pro Display\',\'Segoe UI\',Helvetica,Arial,sans-serif" '
        'font-size="56" font-weight="700" letter-spacing="-1">'
        '<tspan fill="#c0caf5">claude</tspan><tspan fill="#7aa2f7">bar</tspan></text>'
        '<rect x="376" y="106" width="82" height="6" rx="3" fill="#2f334d"/>'
        '<rect x="376" y="106" width="50" height="6" rx="3" fill="url(#br)"/>'
        '</svg>\n'
    )

for n, glyph in GLYPHS.items():
    open(os.path.join(OUT, f"v{n}-mark.svg"), "w").write(mark(glyph))
    open(os.path.join(OUT, f"v{n}.svg"), "w").write(lockup(glyph))
    print(f"wrote v{n}-mark.svg, v{n}.svg")
