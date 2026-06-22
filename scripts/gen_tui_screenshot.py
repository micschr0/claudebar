#!/usr/bin/env python3
"""Generate screenshots/config-tui.png — a render of the `claudebar config` TUI.

Captures the live TUI from a fixed-size tmux pane (ANSI with colors), converts
it to HTML preserving foreground/background/bold, and rasterises it through a
host Chrome with the embedded Hack Nerd Font so the Powerline glyphs render.

Usage:
  CLAUDEBAR_CHROME=/path/to/chrome \
  PW_MODULES=/tmp/pw/node_modules NF_FONT_DIR=/tmp/fonts \
  python3 scripts/gen_tui_screenshot.py

Prereqs mirror gen_screenshots.py: a host Chrome, playwright-core, and the
Hack Nerd Font TTF. tmux must be on PATH.
"""
import subprocess, re, os, sys, base64, time

REPO       = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BINARY     = os.path.join(REPO, "target/release/claudebar")
SHOTS      = os.path.join(REPO, "screenshots")
OUT_PNG    = os.path.join(SHOTS, "config-tui.png")

NF_FONT_DIR = os.environ.get("NF_FONT_DIR", "/tmp/fonts")
PW_MODULES  = os.environ.get("PW_MODULES", "/tmp/pw/node_modules")
HOST_CHROME = os.environ.get("CLAUDEBAR_CHROME")
FONT_URL    = f"file://{NF_FONT_DIR}/HackNerdFontMono-Regular.ttf"

COLS, ROWS = 110, 34

# Tokyo-night-ish default fg when the TUI emits a bare reset.
DEFAULT_FG = (192, 202, 245)
DEFAULT_BG = (26, 27, 38)

def ansi256(n):
    if n < 16:
        t = [(0,0,0),(128,0,0),(0,128,0),(128,128,0),(0,0,128),(128,0,128),
             (0,128,128),(192,192,192),(128,128,128),(255,0,0),(0,255,0),
             (255,255,0),(0,0,255),(255,0,255),(0,255,255),(255,255,255)]
        return t[n]
    if n < 232:
        n -= 16; v = [0,95,135,175,215,255]
        return (v[n//36], v[(n%36)//6], v[n%6])
    c = 8 + (n-232)*10
    return (c, c, c)

def hexc(rgb):
    return "#%02x%02x%02x" % rgb

def esc(s):
    return (s.replace("&","&amp;").replace("<","&lt;")
             .replace(">","&gt;").replace('"',"&quot;"))

def capture_tui():
    """Drive the TUI in tmux and return the ANSI-colored pane capture."""
    subprocess.run(["tmux", "kill-session", "-t", "cbshot"],
                   stderr=subprocess.DEVNULL)
    subprocess.run(["tmux", "new-session", "-d", "-s", "cbshot",
                    "-x", str(COLS), "-y", str(ROWS)], check=True)
    subprocess.run(["tmux", "send-keys", "-t", "cbshot",
                    f"TERM=xterm-256color {BINARY} config", "Enter"], check=True)
    time.sleep(2.0)
    cap = subprocess.run(["tmux", "capture-pane", "-t", "cbshot", "-e", "-p"],
                         capture_output=True, text=True).stdout
    subprocess.run(["tmux", "send-keys", "-t", "cbshot", "q"],
                   stderr=subprocess.DEVNULL)
    time.sleep(0.3)
    subprocess.run(["tmux", "kill-session", "-t", "cbshot"],
                   stderr=subprocess.DEVNULL)
    return cap

def parse_line(line):
    """Yield (fg_rgb, bg_rgb, bold, text) runs for one ANSI line."""
    fg, bg, bold = DEFAULT_FG, None, False
    runs, buf = [], []
    def flush():
        if buf:
            runs.append((fg, bg, bold, "".join(buf)))
            buf.clear()
    pos = 0
    for m in re.finditer(r'\x1b\[([0-9;]*)m', line):
        if m.start() > pos:
            buf.append(line[pos:m.start()])
        params = [p for p in m.group(1).split(";")]
        i = 0
        # Bare ESC[m == reset
        if m.group(1) == "":
            params = ["0"]
        while i < len(params):
            p = params[i]
            if p in ("0", ""):
                flush(); fg, bg, bold = DEFAULT_FG, None, False
            elif p == "1":
                flush(); bold = True
            elif p == "2":
                flush(); bold = False
            elif p == "22":
                flush(); bold = False
            elif p == "39":
                flush(); fg = DEFAULT_FG
            elif p == "49":
                flush(); bg = None
            elif p == "38" and i+1 < len(params):
                flush()
                if params[i+1] == "5":
                    fg = ansi256(int(params[i+2])); i += 2
                elif params[i+1] == "2":
                    fg = (int(params[i+2]), int(params[i+3]), int(params[i+4])); i += 4
            elif p == "48" and i+1 < len(params):
                flush()
                if params[i+1] == "5":
                    bg = ansi256(int(params[i+2])); i += 2
                elif params[i+1] == "2":
                    bg = (int(params[i+2]), int(params[i+3]), int(params[i+4])); i += 4
            i += 1
        # flush text accumulated so far under the *previous* style, then apply
        flush()
        pos = m.end()
    if pos < len(line):
        buf.append(line[pos:])
    flush()
    return runs

def runs_to_html(lines):
    out = []
    for line in lines:
        spans = []
        for fg, bg, bold, text in parse_line(line):
            style = f"color:{hexc(fg)}"
            if bg is not None:
                style += f";background:{hexc(bg)}"
            if bold:
                style += ";font-weight:700"
            spans.append(f'<span style="{style}">{esc(text)}</span>')
        out.append('<div class="row">' + "".join(spans) + "</div>")
    return "\n".join(out)

def embed_nerd_font(chars):
    from fontTools import subset
    ttf_in = os.path.join(NF_FONT_DIR, "HackNerdFontMono-Regular.ttf")
    out = "/tmp/claudebar-nf-tui.woff2"
    opts = subset.Options(flavor="woff2", desubroutinize=True, layout_features=[],
                          notdef_outline=True, recalc_bounds=True, glyph_names=False)
    font = subset.load_font(ttf_in, opts)
    ss = subset.Subsetter(options=opts)
    ss.populate(unicodes=sorted({ord(c) for c in chars}))
    ss.subset(font)
    subset.save_font(font, out, opts)
    b64 = base64.b64encode(open(out, "rb").read()).decode()
    return ("@font-face{font-family:'ClaudebarNF';font-style:normal;font-weight:400;"
            f"src:url(data:font/woff2;base64,{b64}) format('woff2');}}")

DOTS = ('<div class="dots">'
        '<div class="dot" style="background:#ff5f57"></div>'
        '<div class="dot" style="background:#febc2e"></div>'
        '<div class="dot" style="background:#28c840"></div>'
        '</div>')

def build_html(lines):
    chars = set("".join(lines))
    face = embed_nerd_font(chars)
    body = runs_to_html(lines)
    css = f"""
{face}
* {{ margin:0; padding:0; box-sizing:border-box; }}
body {{ background:#0d0d14; padding:28px; display:flex; justify-content:center;
        font-family:'ClaudebarNF',monospace; }}
.window {{ background:{hexc(DEFAULT_BG)}; border-radius:10px;
           border:1px solid #2a2b3d; overflow:hidden;
           box-shadow:0 20px 60px rgba(0,0,0,0.6); display:inline-block; }}
.titlebar {{ background:#1f2035; border-bottom:1px solid #16172a; height:38px;
             display:flex; align-items:center; padding:0 14px; position:relative; }}
.dots {{ display:flex; gap:8px; }}
.dot {{ width:12px; height:12px; border-radius:50%; }}
.title {{ position:absolute; left:50%; transform:translateX(-50%);
          font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;
          font-size:12px; color:#8a8a8a; }}
.term {{ padding:14px 16px; font-size:13px; line-height:1.32; }}
.row {{ white-space:pre; min-height:1.32em; }}
"""
    return f"""<!DOCTYPE html><html><head><meta charset="utf-8"><style>{css}</style></head>
<body><div class="window">
  <div class="titlebar">{DOTS}<span class="title">claudebar config</span></div>
  <div class="term">{body}</div>
</div></body></html>"""

def render(html_path):
    core = os.path.join(PW_MODULES, "playwright-core")
    js = f"""const {{ chromium }} = require("{core}");
(async () => {{
  const browser = await chromium.launch({{ executablePath:"{HOST_CHROME}",
    args:["--no-sandbox","--disable-setuid-sandbox","--disable-gpu"] }});
  const page = await browser.newPage({{ deviceScaleFactor:2, viewport:{{width:1200,height:760}} }});
  await page.goto("file://{html_path}");
  await page.waitForTimeout(900);
  await page.locator(".window").screenshot({{ path:"{OUT_PNG}", omitBackground:true }});
  await browser.close();
  console.log("Saved:", "{OUT_PNG}");
}})().catch(e => {{ console.error(e.message); process.exit(1); }});"""
    with open("/tmp/claudebar_tui_render.js", "w") as f:
        f.write(js)
    return subprocess.run(["node", "/tmp/claudebar_tui_render.js"]).returncode == 0

def main():
    if not HOST_CHROME:
        sys.exit("Set CLAUDEBAR_CHROME to a host Chrome/Chromium binary.")
    cap = capture_tui()
    lines = cap.rstrip("\n").split("\n")
    # Trim trailing blank rows the pane pads with.
    while lines and not re.sub(r'\x1b\[[0-9;]*m', '', lines[-1]).strip():
        lines.pop()
    plain = "\n".join(re.sub(r'\x1b\[[0-9;]*m', '', l) for l in lines)
    print(plain)
    html = build_html(lines)
    tmp = "/tmp/claudebar_tui.html"
    with open(tmp, "w") as f:
        f.write(html)
    os.makedirs(SHOTS, exist_ok=True)
    print("\n  Rendering...")
    print("  Done." if render(tmp) else "  Render failed.")

if __name__ == "__main__":
    main()
