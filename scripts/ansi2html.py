#!/usr/bin/env python3
"""Convert a single line of ANSI (SGR, incl. 256-color 38;5;N) to inline HTML spans.

Reads ANSI from stdin, writes HTML to stdout. Used by gen-gallery.sh to render
claudebar statuslines into the promo page without producing image files.
Only the SGR codes claudebar emits are handled: reset (0), and 256-color
foreground/background (38;5;N / 48;5;N).
"""
import sys, re, html

# Standard xterm 16-color palette (0-15).
BASE16 = [
    "000000", "cd0000", "00cd00", "cdcd00", "0000ee", "cd00cd", "00cdcd", "e5e5e5",
    "7f7f7f", "ff0000", "00ff00", "ffff00", "5c5cff", "ff00ff", "00ffff", "ffffff",
]
CUBE = [0, 95, 135, 175, 215, 255]

def xterm_to_hex(n: int) -> str:
    if 0 <= n <= 15:
        return BASE16[n]
    if 16 <= n <= 231:
        n -= 16
        r, g, b = CUBE[n // 36], CUBE[(n // 6) % 6], CUBE[n % 6]
        return f"{r:02x}{g:02x}{b:02x}"
    if 232 <= n <= 255:
        v = 8 + 10 * (n - 232)
        return f"{v:02x}{v:02x}{v:02x}"
    return "c0caf5"

SGR = re.compile(r"\x1b\[([0-9;]*)m")

def convert(line: str) -> str:
    out, open_span = [], False
    pos = 0
    for m in SGR.finditer(line):
        text = line[pos:m.start()]
        if text:
            out.append(html.escape(text))
        pos = m.end()
        codes = [int(c) for c in m.group(1).split(";") if c != ""] or [0]
        i = 0
        while i < len(codes):
            c = codes[i]
            if c == 0:
                if open_span:
                    out.append("</span>"); open_span = False
            elif c in (38, 48) and i + 2 < len(codes) and codes[i + 1] == 5:
                hexc = xterm_to_hex(codes[i + 2])
                prop = "color" if c == 38 else "background"
                if open_span:
                    out.append("</span>")
                out.append(f'<span style="{prop}:#{hexc}">'); open_span = True
                i += 2
            i += 1
    tail = line[pos:]
    if tail:
        out.append(html.escape(tail))
    if open_span:
        out.append("</span>")
    return "".join(out)

if __name__ == "__main__":
    data = sys.stdin.read().rstrip("\n")
    sys.stdout.write(convert(data))
