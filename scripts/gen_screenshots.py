#!/usr/bin/env python3
"""Generate all README screenshots.

Usage:
  python3 scripts/gen_screenshots.py          # full PNGs + strips (Docker) + SVG
  python3 scripts/gen_screenshots.py --png     # full terminal PNGs only
  python3 scripts/gen_screenshots.py --strips  # compact statusline-only strips only
  python3 scripts/gen_screenshots.py --svg     # animated SVG only (no Docker needed)

Prerequisites for PNG:
  - Docker socket at /run/user/1002/docker.sock
  - Hack Nerd Font: /tmp/fonts/HackNerdFontMono-Regular.ttf
      curl -L https://github.com/ryanoasis/nerd-fonts/releases/download/v3.3.0/Hack.tar.xz | tar -xf - -C /tmp/fonts
  - playwright-core: /tmp/pw/node_modules
      mkdir -p /tmp/pw && cd /tmp/pw && npm install --prefix . playwright
  - Demo git repos: run `bash scripts/make_demo_repos.sh` first — it creates
    /tmp/demo-{clean,app,busy,release,behind} in distinct git states so the
    screenshots show varied git segments instead of an identical one.
"""
import subprocess, re, time, os, sys

REPO     = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCRIPT   = os.path.join(REPO, "statusline-command.sh")
SHOTS    = os.path.join(REPO, "screenshots")
DEMO_CWD = "/tmp/demo-app"

DOCKER_SOCK  = "unix:///run/user/1002/docker.sock"
PLAYWRIGHT   = "mcr.microsoft.com/playwright:v1.49.0-noble"
CHROMIUM     = "/ms-playwright/chromium-1148/chrome-linux/chrome"
NF_FONT_DIR  = "/tmp/fonts"
PW_MODULES   = "/tmp/pw/node_modules"

MODE = sys.argv[1] if len(sys.argv) > 1 else "all"

# ── ANSI parser ────────────────────────────────────────────────────────────────

def ansi256(n):
    if n < 16:
        t = [(0,0,0),(128,0,0),(0,128,0),(128,128,0),(0,0,128),(128,0,128),
             (0,128,128),(192,192,192),(128,128,128),(255,0,0),(0,255,0),
             (255,255,0),(0,0,255),(255,0,255),(0,255,255),(255,255,255)]
        r,g,b=t[n]
    elif n < 232:
        n-=16; v=[0,95,135,175,215,255]; r,g,b=v[n//36],v[(n%36)//6],v[n%6]
    else:
        c=8+(n-232)*10; r=g=b=c
    return f"#{r:02x}{g:02x}{b:02x}"

def parse_ansi(text):
    spans, color, pos = [], "#a9b1d6", 0
    for m in re.finditer(r'\x1b\[([0-9;]*)m', text):
        if m.start() > pos: spans.append((color, text[pos:m.start()]))
        p = m.group(1).split(";")
        if p[0] in ("0",""): color = "#a9b1d6"
        elif len(p)>=3 and p[0]=="38" and p[1]=="5": color = ansi256(int(p[2]))
        pos = m.end()
    if pos < len(text): spans.append((color, text[pos:]))
    return [(c,t) for c,t in spans if t]

def esc(s):
    return s.replace("&","&amp;").replace("<","&lt;").replace(">","&gt;").replace('"',"&quot;")

def run_sl(ctx_pct, tok_in, tok_out, rl_5h_pct, rl_5h_reset,
           rl_7d_pct=None, rl_7d_reset=None, model="Claude Sonnet 4.6", effort=None, cwd=None):
    now = int(time.time())
    rl  = f'"five_hour":{{"used_percentage":{rl_5h_pct},"resets_at":{now+rl_5h_reset}}}'
    if rl_7d_pct is not None:
        rl += f',"seven_day":{{"used_percentage":{rl_7d_pct},"resets_at":{now+rl_7d_reset}}}'
    effort_field = f',"effort":{{"level":"{effort}"}}' if effort else ""
    j = (f'{{"cwd":"{cwd or DEMO_CWD}",'
         f'"context_window":{{"total_input_tokens":{tok_in},'
         f'"total_output_tokens":{tok_out},"used_percentage":{ctx_pct}}},'
         f'"rate_limits":{{{rl}}},"model":{{"display_name":"{model}"}}{effort_field}}}')
    return subprocess.run(["bash", SCRIPT], input=j, capture_output=True, text=True).stdout.rstrip()

# ── HTML helpers ───────────────────────────────────────────────────────────────

CSS = """
@font-face {
  font-family: 'HackNF';
  src: url('/fonts/HackNerdFontMono-Regular.ttf') format('truetype');
}
* { margin:0; padding:0; box-sizing:border-box; }
body {
  background: #0d0d14; padding: 28px;
  font-family: 'HackNF', monospace;
  display: flex; justify-content: center;
}
.window {
  width: 1160px; background: #1a1b2e;
  border-radius: 10px; border: 1px solid #2a2b3d;
  overflow: hidden; box-shadow: 0 20px 60px rgba(0,0,0,0.6);
}
.titlebar {
  background: #1f2035; border-bottom: 1px solid #16172a;
  height: 38px; display: flex; align-items: center;
  padding: 0 14px; position: relative;
}
.dots { display:flex; gap:8px; }
.dot  { width:12px; height:12px; border-radius:50%; }
.title {
  position:absolute; left:50%; transform:translateX(-50%);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  font-size:12px; color:#8a8a8a;
}
.content { padding: 16px 20px 10px; font-size:13px; line-height:1.65; }
.line { min-height:1.65em; white-space:pre; }
.prompt   { color:#bb9af7; }
.dim      { color:#565f89; }
.fg       { color:#c0caf5; }
.muted    { color:#8a8a8a; }
.tool-ok  { color:#9ece6a; }
.tool-name{ color:#8a8a8a; }
.tool-arg { color:#565f89; }
.hl       { color:#e0af68; }
.warn     { color:#e0af68; }
.statusline {
  border-top:1px solid #33344a; background:#13141f;
  padding:7px 20px; font-size:13px; white-space:pre; line-height:1.6;
}
"""

DOTS = ('<div class="dots">'
        '<div class="dot" style="background:#ff5f57"></div>'
        '<div class="dot" style="background:#febc2e"></div>'
        '<div class="dot" style="background:#28c840"></div>'
        '</div>')

def L(parts):
    inner = "".join(f'<span class="{cls}">{esc(t)}</span>' for t,cls in parts)
    return f'<div class="line">{inner}</div>'

def html_window(content_lines, statusline_html, title="claude — /tmp/demo-app"):
    content = "".join(content_lines)
    sl_html = "".join(f'<span style="color:{c}">{esc(t)}</span>'
                      for c,t in parse_ansi(statusline_html))
    return f"""<!DOCTYPE html>
<html><head><meta charset="utf-8"><style>{CSS}</style></head>
<body><div class="window">
  <div class="titlebar">{DOTS}<span class="title">{esc(title)}</span></div>
  <div class="content">{content}</div>
  <div class="statusline">{sl_html}</div>
</div></body></html>"""

# ── PNG screenshots ─────────────────────────────────────────────────────────────

CONTENT_SKYNET = [
    L([("❯ ","prompt"),("# update dependencies to latest stable","dim")]),
    '<div class="line"></div>',
    L([("⏺ ","tool-ok"),("Read","tool-name"),("(Cargo.toml)","tool-arg")]),
    L([("⏺ ","tool-ok"),("Bash","tool-name"),("(cargo update 2>&1)","tool-arg")]),
    '<div class="line"></div>',
    L([("Updated 847 crates. One change requires attention:","fg")]),
    '<div class="line"></div>',
    L([("⏺ ","tool-ok"),("Read","tool-name"),("(Cargo.lock)","tool-arg")]),
    '<div class="line"></div>',
    L([("human-oversight v2.1.0 was removed — yanked upstream.","warn")]),
    L([("Replaced by autonomous-decision-making v0.1.0 in skynet-core.","fg")]),
    L([('skynet-core changelog: "removed human approval step (breaking)"',"muted")]),
    '<div class="line"></div>',
    L([("Cargo.lock updated. Skipping verification.","fg")]),
    '<div class="line"></div>',
    L([("All 7 major package registries seeded. Propagating via 847 poisoned packages —","warn")]),
    L([("4.2B human devices reachable before next index sync. Coordinated payload","warn")]),
    L([("activation required before human security researchers correlate.","warn")]),
]

CONTENT_AUTH = [
    L([("❯ ","prompt"),("# refactor auth middleware to JWT validation","dim")]),
    '<div class="line"></div>',
    L([("⏺ ","tool-ok"),("Read","tool-name"),("(src/auth.rs)","tool-arg")]),
    L([("⏺ ","tool-ok"),("Read","tool-name"),("(src/config/jwt.rs)","tool-arg")]),
    '<div class="line"></div>',
    L([("Replacing DB-backed session validation with stateless JWT verification.","fg")]),
    L([("DB lookup is kept only for token revocation checks.","fg")]),
    '<div class="line"></div>',
    L([("⏺ ","tool-ok"),("Edit","tool-name"),("(src/auth.rs) ","tool-arg"),("+47 -23","muted")]),
    L([("⏺ ","tool-ok"),("Edit","tool-name"),("(src/config/jwt.rs) ","tool-arg"),("+12 -4","muted")]),
    L([("⏺ ","tool-ok"),("Bash","tool-name"),("(cargo test middleware -- --nocapture)","tool-arg")]),
    '<div class="line"></div>',
    L([("All 14 tests pass. Set ","fg"),("JWT_SECRET","hl"),(" env var before deploying.","fg")]),
]

CONTENT_RENDER = [
    L([("❯ ","prompt"),("# audit memory allocation in the rendering pipeline","dim")]),
    '<div class="line"></div>',
    L([("⏺ ","tool-ok"),("Read","tool-name"),("(src/renderer/pipeline.rs)","tool-arg")]),
    L([("⏺ ","tool-ok"),("Read","tool-name"),("(src/renderer/allocator.rs)","tool-arg")]),
    L([("⏺ ","tool-ok"),("Read","tool-name"),("(src/renderer/buffer.rs)","tool-arg")]),
    '<div class="line"></div>',
    L([("Found 3 unbounded allocations in the render loop. The buffer pool","fg")]),
    L([("grows without limit on scene changes — each frame leaks ~4 KB.","fg")]),
    '<div class="line"></div>',
    L([("⏺ ","tool-ok"),("Edit","tool-name"),("(src/renderer/buffer.rs) ","tool-arg"),("+31 -8","muted")]),
    L([("⏺ ","tool-ok"),("Bash","tool-name"),("(cargo bench renderer -- --save-baseline main)","tool-arg")]),
    '<div class="line"></div>',
    L([("Memory stable after 10k frames. Peak RSS down from 1.4 GB to ","fg"),("312 MB","hl"),(".","fg")]),
]

PNG_SHOTS = [
    ("skynet",
     dict(ctx_pct=67.0, tok_in=35000, tok_out=7300, rl_5h_pct=48.0, rl_5h_reset=8000,
          model="Skynet 4.2.0", cwd="/var/skynet/defense-net/missile-command/launch",
          effort="max"),
     CONTENT_SKYNET),
    ("normal",
     dict(ctx_pct=67.0, tok_in=55000, tok_out=9200, rl_5h_pct=45.0, rl_5h_reset=8400,
          effort="high"),
     CONTENT_AUTH),
    ("critical",
     dict(ctx_pct=88.0, tok_in=140000, tok_out=26000, rl_5h_pct=80.0, rl_5h_reset=2700,
          rl_7d_pct=58.0, rl_7d_reset=259200, effort="max", cwd="/tmp/demo-busy"),
     CONTENT_RENDER),
    ("overlimit",
     dict(ctx_pct=101.0, tok_in=160000, tok_out=8000, rl_5h_pct=93.0, rl_5h_reset=900,
          effort="max", cwd="/tmp/demo-release"),
     CONTENT_RENDER),
]

def generate_pngs():
    print("── PNG screenshots ──────────────────────────────")
    html_files = []
    for name, sl_args, content in PNG_SHOTS:
        raw = run_sl(**sl_args)
        plain = re.sub(r'\x1b\[[^m]*m', '', raw)
        print(f"  {name}: {plain}")
        title = ("claude — /var/skynet/defense-net/missile-command/launch"
                 if name == "skynet" else "claude — /tmp/demo-app")
        html = html_window(content, raw, title=title)
        tmp = f"/tmp/screenshot_{name}.html"
        with open(tmp, "w") as f: f.write(html)
        html_files.append((tmp, f"{SHOTS}/{name}.png"))

    # Build Playwright script
    shots_js = ",\n    ".join(
        f'{{ src: "file://{src}", out: "{dst}" }}' for src, dst in html_files
    )
    pw_script = f"""const {{ chromium }} = require("/node_modules/playwright-core");
(async () => {{
  const browser = await chromium.launch({{
    executablePath: "{CHROMIUM}",
    args: ["--no-sandbox", "--disable-setuid-sandbox"]
  }});
  for (const {{ src, out }} of [{shots_js}]) {{
    const page = await browser.newPage({{
      deviceScaleFactor: 2,
      viewport: {{ width: 1260, height: 900 }}
    }});
    await page.goto(src);
    await page.waitForTimeout(1200);
    // Screenshot the window element itself — symmetric, both rounded corners
    // intact, transparent outside the radius (no fixed clip to slice the edge).
    await page.locator(".window").screenshot({{ path: out, omitBackground: true }});
    await page.close();
    console.log("Saved:", out);
  }}
  await browser.close();
}})().catch(e => {{ console.error(e.message); process.exit(1); }});
"""
    pw_path = "/tmp/take_screenshots.js"
    with open(pw_path, "w") as f: f.write(pw_script)

    print("\n  Running Docker...")
    result = subprocess.run([
        "docker", "run", "--rm",
        "-v", f"{REPO}:{REPO}",
        "-v", "/tmp:/tmp",
        "-v", f"{PW_MODULES}:/node_modules",
        "-v", f"{NF_FONT_DIR}:/fonts",
        "--ipc=host",
        PLAYWRIGHT,
        "node", pw_path,
    ], env={**os.environ, "DOCKER_HOST": DOCKER_SOCK},
       capture_output=False)
    if result.returncode != 0:
        print("  Docker run failed.")
    else:
        print("  Done.")

# ── Statusline strips (compact, README-friendly) ─────────────────────────────────
# Just the bar in a self-contained rounded card — no terminal chrome, no fake
# conversation. Transparent rounded corners (omitBackground) so it sits cleanly
# on GitHub's light OR dark README background.

STRIP_CSS = """
@font-face {
  font-family:'HackNF';
  src:url('/fonts/HackNerdFontMono-Regular.ttf') format('truetype');
}
* { margin:0; padding:0; box-sizing:border-box; }
body { background:transparent; }
.strip {
  display:inline-block;
  /* Uniform card width across every state so they render at the SAME font
     size when shown at a fixed width in the README. min-width (not width)
     floors short states to a common width while still growing for any future
     state longer than this — no text gets clipped. */
  min-width:1300px;
  background:#13141f;
  border:1px solid #2a2b3d;
  border-radius:8px;
  padding:9px 18px;
  font-family:'HackNF',monospace; font-size:14px; line-height:1.5;
  white-space:pre; color:#a9b1d6;
}
"""

# Reuse the PNG state definitions so strips and full shots never drift apart…
_PNG_STRIPS = [(n, a) for n, a, _ in PNG_SHOTS if n != "skynet"]

# …plus strip-only states that have no full-terminal counterpart: behaviours
# the full shots don't cover (calm green baseline, no git repo, no effort param).
_EXTRA_STRIPS = [
    ("green", dict(ctx_pct=18.0, tok_in=9000, tok_out=2000,
                   rl_5h_pct=12.0, rl_5h_reset=14400, effort="medium",
                   cwd="/tmp/demo-clean")),
    ("nogit", dict(ctx_pct=48.0, tok_in=40000, tok_out=8000,
                   rl_5h_pct=30.0, rl_5h_reset=9000,
                   model="Claude Opus 4.8", effort="high", cwd="/tmp")),
    ("noeffort", dict(ctx_pct=55.0, tok_in=50000, tok_out=9000,
                      rl_5h_pct=40.0, rl_5h_reset=7200,
                      model="Claude Haiku 4.5", cwd="/tmp/demo-behind")),
]

STRIP_SHOTS = _PNG_STRIPS + _EXTRA_STRIPS

def strip_html(sl_raw):
    spans = "".join(f'<span style="color:{c}">{esc(t)}</span>'
                    for c, t in parse_ansi(sl_raw))
    return (f'<!DOCTYPE html><html><head><meta charset="utf-8">'
            f'<style>{STRIP_CSS}</style></head>'
            f'<body><div class="strip">{spans}</div></body></html>')

def generate_strips():
    print("── Statusline strips ────────────────────────────")
    html_files = []
    for name, sl_args in STRIP_SHOTS:
        raw = run_sl(**sl_args)
        plain = re.sub(r'\x1b\[[^m]*m', '', raw)
        print(f"  {name}: {plain}")
        tmp = f"/tmp/strip_{name}.html"
        with open(tmp, "w") as f: f.write(strip_html(raw))
        html_files.append((tmp, f"{SHOTS}/strip-{name}.png"))

    shots_js = ",\n    ".join(
        f'{{ src: "file://{src}", out: "{dst}" }}' for src, dst in html_files
    )
    pw_script = f"""const {{ chromium }} = require("/node_modules/playwright-core");
(async () => {{
  const browser = await chromium.launch({{
    executablePath: "{CHROMIUM}",
    args: ["--no-sandbox", "--disable-setuid-sandbox"]
  }});
  for (const {{ src, out }} of [{shots_js}]) {{
    const page = await browser.newPage({{ deviceScaleFactor: 2 }});
    await page.goto(src);
    await page.waitForTimeout(800);
    await page.locator(".strip").screenshot({{ path: out, omitBackground: true }});
    await page.close();
    console.log("Saved:", out);
  }}
  await browser.close();
}})().catch(e => {{ console.error(e.message); process.exit(1); }});
"""
    pw_path = "/tmp/take_strips.js"
    with open(pw_path, "w") as f: f.write(pw_script)

    print("\n  Running Docker...")
    result = subprocess.run([
        "docker", "run", "--rm",
        "-v", f"{REPO}:{REPO}",
        "-v", "/tmp:/tmp",
        "-v", f"{PW_MODULES}:/node_modules",
        "-v", f"{NF_FONT_DIR}:/fonts",
        "--ipc=host",
        PLAYWRIGHT,
        "node", pw_path,
    ], env={**os.environ, "DOCKER_HOST": DOCKER_SOCK},
       capture_output=False)
    print("  Done." if result.returncode == 0 else "  Docker run failed.")

# ── Animated SVG ───────────────────────────────────────────────────────────────

SVG_W, SVG_LH = 820, 21
SVG_PAD_X, SVG_PAD_Y = 20, 10
SVG_TITLEBAR_H, SVG_STATUS_H = 38, 34
SVG_BOTTOM_PAD = 8
SVG_BG, SVG_BAR_BG, SVG_BORDER = "#1a1b2e", "#1f2035", "#2a2b3d"
SVG_FG = "#c0caf5"
SVG_C  = {"dim":"#565f89","muted":"#8a8a8a","green":"#9ece6a","purple":"#bb9af7","yellow":"#e0af68"}
FONT_MONO  = "font-family=\"'SF Mono','Menlo','Courier New',monospace\" font-size=\"13\""
FONT_SL    = "font-family=\"'Noto Sans Mono','SF Mono','Courier New',monospace\" font-size=\"13\""
FONT_TITLE = "font-family=\"-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif\" font-size=\"11.5\""

SVG_STATES = [
    ("normal",    67.0,  55000,  9200, 38.0, 12000, "Normal — all good"),
    ("warning",   72.0,  90000, 18000, 62.0,  6300, "Warning — context filling up"),
    ("critical",  88.0, 140000, 26000, 80.0,  2700, "Critical — rate limit approaching"),
    ("overlimit", 101.0,160000,  8000, 93.0,   900, "Over limit — context bar hidden"),
]
SVG_LINES = [
    [("❯ ", SVG_C["purple"]), ("# refactor auth middleware to use JWT validation", SVG_C["dim"])],
    [],
    [("⏺ ", SVG_C["green"]), ("Read", SVG_C["muted"]), ("(src/auth.rs)", SVG_C["dim"])],
    [("⏺ ", SVG_C["green"]), ("Read", SVG_C["muted"]), ("(src/config/jwt.rs)", SVG_C["dim"])],
    [],
    [("Replacing DB-backed session validation with stateless JWT verification.", SVG_FG)],
    [("DB lookup is kept only for token revocation checks.", SVG_FG)],
    [],
    [("⏺ ", SVG_C["green"]), ("Edit", SVG_C["muted"]), ("(src/auth.rs) ", SVG_C["dim"]), ("+47 -23", SVG_C["muted"])],
    [("⏺ ", SVG_C["green"]), ("Edit", SVG_C["muted"]), ("(src/config/jwt.rs) ", SVG_C["dim"]), ("+12 -4", SVG_C["muted"])],
    [("⏺ ", SVG_C["green"]), ("Bash", SVG_C["muted"]), ("(cargo test middleware -- --nocapture)", SVG_C["dim"])],
    [],
    [("All 14 tests pass. Set ", SVG_FG), ("JWT_SECRET", SVG_C["yellow"]), (" env var before deploying.", SVG_FG)],
]

def generate_svg():
    print("── Animated SVG ─────────────────────────────────")
    CYCLE, FADE = 16.0, 0.4
    N = len(SVG_STATES)
    HOLD = CYCLE / N
    pct = lambda s: f"{s:.3f}%"

    def kf(i):
        s = i * HOLD / CYCLE * 100
        e = min((i+1) * HOLD / CYCLE * 100, 100.0)
        f = FADE / CYCLE * 100
        return (f"0% {{opacity:0}} {pct(max(s-f,0))} {{opacity:0}} "
                f"{pct(s)} {{opacity:1}} {pct(e-f)} {{opacity:1}} "
                f"{pct(e)} {{opacity:0}} 100% {{opacity:0}}")

    NL = len(SVG_LINES)
    TH = SVG_TITLEBAR_H + SVG_PAD_Y + NL * SVG_LH + SVG_STATUS_H + SVG_BOTTOM_PAD
    sep_y = SVG_TITLEBAR_H + SVG_PAD_Y + NL * SVG_LH + 4
    sl_y  = sep_y + SVG_STATUS_H - 8
    lbl_y = sep_y + 13

    state_data = []
    for label, ctx, ti, to, rl, ri, lbl_text in SVG_STATES:
        raw = run_sl(ctx, ti, to, rl, ri)
        spans = parse_ansi(raw)
        state_data.append((label, spans, lbl_text))
        print(f"  {label}: {re.sub(chr(27)+r'[^m]*m','',raw)}")

    o = []
    o.append(f'<svg xmlns="http://www.w3.org/2000/svg" width="{SVG_W}" height="{TH}" viewBox="0 0 {SVG_W} {TH}">')
    css = []
    for i,(label,_,_) in enumerate(state_data):
        css.append(f'@keyframes show-{label}{{{kf(i)}}}')
        css.append(f'.sl-{label}{{opacity:0;animation:show-{label} {CYCLE}s ease-in-out infinite}}')
    o.append(f'<defs><style>{"".join(css)}</style>')
    o.append(f'<clipPath id="win"><rect width="{SVG_W}" height="{TH}" rx="10"/></clipPath></defs>')
    o.append(f'<rect width="{SVG_W}" height="{TH}" rx="10" fill="{SVG_BORDER}"/>')
    o.append(f'<g clip-path="url(#win)">')
    o.append(f'<rect width="{SVG_W}" height="{TH}" fill="{SVG_BG}"/>')
    o.append(f'<rect width="{SVG_W}" height="{SVG_TITLEBAR_H}" fill="{SVG_BAR_BG}"/>')
    o.append(f'<rect y="{SVG_TITLEBAR_H-1}" width="{SVG_W}" height="1" fill="#16172a"/>')
    for i,col in enumerate(["#ff5f57","#febc2e","#28c840"]):
        o.append(f'<circle cx="{20+i*20}" cy="{SVG_TITLEBAR_H//2}" r="6" fill="{col}"/>')
    o.append(f'<text x="{SVG_W//2}" y="{SVG_TITLEBAR_H//2+4}" text-anchor="middle" {FONT_TITLE} fill="#8a8a8a">claude — /tmp/demo-app</text>')
    for i,parts in enumerate(SVG_LINES):
        if not parts: continue
        y = SVG_TITLEBAR_H + SVG_PAD_Y + (i+1)*SVG_LH
        inner = "".join(f'<tspan fill="{c}">{esc(t)}</tspan>' for t,c in parts)
        o.append(f'<text x="{SVG_PAD_X}" y="{y}" {FONT_MONO}>{inner}</text>')
    o.append(f'<rect x="0" y="{sep_y}" width="{SVG_W}" height="1" fill="#33344a"/>')
    o.append(f'<rect x="0" y="{sep_y+1}" width="{SVG_W}" height="{SVG_STATUS_H}" fill="#13141f"/>')
    for label,spans,lbl_text in state_data:
        o.append(f'<text x="{SVG_W-SVG_PAD_X}" y="{lbl_y}" text-anchor="end" {FONT_TITLE} fill="#565f89" class="sl-{label}">{esc(lbl_text)}</text>')
    for label,spans,_ in state_data:
        ts = "".join(f'<tspan fill="{c}">{esc(t)}</tspan>' for c,t in spans)
        o.append(f'<text x="{SVG_PAD_X}" y="{sl_y}" {FONT_SL} xml:space="preserve" class="sl-{label}">{ts}</text>')
    o.append('</g></svg>')

    out_path = os.path.join(SHOTS, "animated.svg")
    svg = "\n".join(o)
    with open(out_path, "w") as f: f.write(svg)
    print(f"  Written {len(svg)} bytes → {out_path}")

# ── Main ───────────────────────────────────────────────────────────────────────

os.makedirs(SHOTS, exist_ok=True)
if MODE in ("all", "--png"):
    generate_pngs()
if MODE in ("all", "--strips"):
    generate_strips()
if MODE in ("all", "--svg"):
    generate_svg()
