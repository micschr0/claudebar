#!/usr/bin/env python3
"""Render + crop standalone screenshots for the 3 optional segments
(dev-context, burn, clock) that have no existing fixture/crop in the main
gen_screenshots.py pipeline.

Each is rendered alone via `--segments <name>` (single segment -> no chevron,
no boundary-detection needed, just crop the content bounding box) through the
same headless-Chromium path as scripts/gen_screenshots.py's strips.

burn needs a warm sample cache to reach its "active" (non-warming) state --
seeded via CLAUDEBAR_BURN_FILE pointing at a scratch TSV with two rising
samples 300s apart.

Requires the same env vars as gen_screenshots.py --strips:
  CLAUDEBAR_CHROME, PW_MODULES, NF_FONT_DIR
"""

import json
import os
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
BINARY = REPO_ROOT / "target" / "release" / "claudebar"
if not BINARY.exists():
    BINARY = REPO_ROOT / "target" / "debug" / "claudebar"
OUT_DIR = REPO_ROOT / "video" / "public" / "screenshots"

FONT_URL = "file://" + str(Path(os.environ.get("NF_FONT_DIR", "/tmp/fonts")) / "HackNerdFontMono-Regular.ttf")

STRIP_CSS = """
@font-face { font-family:'HackNF'; src:url('__FONT_URL__') format('truetype'); }
* { margin:0; padding:0; box-sizing:border-box; }
body { background: #13141f; }
.stripwrap { display:inline-block; padding:22px 26px; background:transparent; }
.strip {
  display:inline-block;
  background:#13141f; border-radius:8px;
  padding:16px 22px;
  font-family:'HackNF',monospace; font-size:14px; line-height:1.5;
  white-space:pre; color:#a9b1d6;
}
"""


def ansi256(n):
    if n < 16:
        base = [
            (0, 0, 0), (205, 0, 0), (0, 205, 0), (205, 205, 0),
            (0, 0, 238), (205, 0, 205), (0, 205, 205), (229, 229, 229),
            (127, 127, 127), (255, 0, 0), (0, 255, 0), (255, 255, 0),
            (92, 92, 255), (255, 0, 255), (0, 255, 255), (255, 255, 255),
        ]
        return "#%02x%02x%02x" % base[n]
    if n < 232:
        n -= 16
        r, g, b = n // 36, (n % 36) // 6, n % 6
        scale = [0, 95, 135, 175, 215, 255]
        return "#%02x%02x%02x" % (scale[r], scale[g], scale[b])
    v = 8 + (n - 232) * 10
    return "#%02x%02x%02x" % (v, v, v)


def parse_ansi(text):
    spans, color, pos = [], "#a9b1d6", 0
    for m in re.finditer(r"\x1b\[([0-9;]*)m", text):
        if m.start() > pos:
            spans.append((color, text[pos:m.start()]))
        p = m.group(1).split(";")
        if p[0] in ("0", ""):
            color = "#a9b1d6"
        elif len(p) >= 3 and p[0] == "38" and p[1] == "5":
            color = ansi256(int(p[2]))
        pos = m.end()
    if pos < len(text):
        spans.append((color, text[pos:]))
    return [(c, t) for c, t in spans if t]


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace('"', "&quot;")


def strip_html(sl_raw):
    spans = "".join(f'<span style="color:{c}">{esc(t)}</span>' for c, t in parse_ansi(sl_raw))
    css = STRIP_CSS.replace("__FONT_URL__", FONT_URL)
    return (f'<!DOCTYPE html><html><head><meta charset="utf-8"><style>{css}</style></head>'
            f'<body><div class="stripwrap"><div class="strip">{spans}</div></div></body></html>')


def render_claudebar(payload, segment, env_extra=None):
    env = {**os.environ}
    if env_extra:
        env.update(env_extra)
    cmd = [str(BINARY), "render", "--segments", segment]
    out = subprocess.run(cmd, input=json.dumps(payload), capture_output=True, text=True, env=env)
    return out.stdout.rstrip()


SHOTS = [
    ("dev-context", "dev-context", {
        "worktree": {"name": "feature-render-cache"},
        "pr": {"number": 482, "review_state": "ok"},
        "agent": {"name": "gsd-executor"},
    }, None),
    ("clock", "clock", {}, None),
]


def render_shots(html_files, selector, scale=2, wait=800, viewport=(900, 300)):
    core = os.path.join(os.environ["PW_MODULES"], "playwright-core")
    chrome = os.environ["CLAUDEBAR_CHROME"]
    shots_js = ",\n    ".join(f'{{ src:"file://{s}", out:"{o}" }}' for s, o in html_files)
    js = f"""
const {{ chromium }} = require("{core}");
(async () => {{
  const browser = await chromium.launch({{ executablePath:"{chrome}",
    args:["--no-sandbox","--disable-setuid-sandbox","--disable-gpu"] }});
  for (const {{ src, out }} of [{shots_js}]) {{
    const page = await browser.newPage({{ deviceScaleFactor:{scale},
      viewport:{{width:{viewport[0]},height:{viewport[1]}}} }});
    await page.goto(src);
    await page.waitForTimeout({wait});
    await page.locator("{selector}").screenshot({{ path: out, omitBackground: true }});
    await page.close();
    console.log("Saved:", out);
  }}
  await browser.close();
}})().catch(e => {{ console.error(e.message); process.exit(1); }});
"""
    with open("/tmp/claudebar_opt_render.js", "w") as f:
        f.write(js)
    subprocess.run(["node", "/tmp/claudebar_opt_render.js"], check=True)


BG = (19, 20, 31)
TARGET_HEIGHT = 112  # match the existing seg-directory/git/model crops


def bbox_crop(png_path):
    """Flatten the transparent corners onto the pill's own flat background
    (matches the existing opaque RGB crops -- no seam when embedded in a
    second rounded card, see PILL-DESIGN-NOTES.md) and normalize height."""
    from PIL import Image

    im = Image.open(png_path).convert("RGBA")
    bg = Image.new("RGBA", im.size, (*BG, 255))
    flat = Image.alpha_composite(bg, im).convert("RGB")

    bbox = im.split()[-1].getbbox()
    if not bbox:
        flat.save(png_path)
        return
    left, top, right, bottom = bbox
    h = bottom - top
    pad = (TARGET_HEIGHT - h) // 2
    top = max(0, top - pad)
    bottom = min(flat.height, top + TARGET_HEIGHT)
    flat.crop((left, top, right, bottom)).save(png_path)


def main():
    html_files = []

    # dev-context: needs worktree/pr/agent fields run_sl doesn't expose.
    dc_payload = {
        "cwd": "/tmp/demo-app",
        "pr": {"number": 482, "review_state": "ok"},
        "agent": {"name": "executor"},
    }
    raw = render_claudebar(dc_payload, "dev-context")
    print(f"  dev-context: {re.sub(chr(27)+r'[^m]*m', '', raw)}")
    tmp = "/tmp/opt_dev-context.html"
    with open(tmp, "w") as f:
        f.write(strip_html(raw))
    html_files.append((tmp, str(OUT_DIR / "seg-dev-context.png")))

    # clock: two timestamps 60s apart for a crossfade tick.
    clock_payload = {"cwd": "/tmp/demo-app"}
    for suffix, now_offset in (("a", 0), ("b", 60)):
        env_extra = {}
        raw = render_claudebar(clock_payload, "clock")
        print(f"  clock-{suffix}: {re.sub(chr(27)+r'[^m]*m', '', raw)}")
        tmp = f"/tmp/opt_clock_{suffix}.html"
        with open(tmp, "w") as f:
            f.write(strip_html(raw))
        html_files.append((tmp, str(OUT_DIR / f"seg-clock-{suffix}.png")))
        time.sleep(1)  # let the wall clock actually move a second so a-vs-b differ

    # burn: seed a warm rising-sample cache so it reaches "active" state.
    with tempfile.NamedTemporaryFile(suffix=".tsv", delete=False) as f:
        burn_file = f.name
    now = int(time.time())
    with open(burn_file, "w") as f:
        f.write(f"{now - 300}\t20.000\t{now + 8000}\n")
        f.write(f"{now}\t45.000\t{now + 8000}\n")
    burn_payload = {
        "cwd": "/tmp/demo-app",
        "rate_limits": {"five_hour": {"used_percentage": 45.0, "resets_at": now + 8000}},
    }
    raw = render_claudebar(burn_payload, "burn", env_extra={"CLAUDEBAR_BURN_FILE": burn_file})
    print(f"  burn: {re.sub(chr(27)+r'[^m]*m', '', raw)}")
    tmp = "/tmp/opt_burn.html"
    with open(tmp, "w") as f:
        f.write(strip_html(raw))
    html_files.append((tmp, str(OUT_DIR / "seg-burn.png")))
    os.unlink(burn_file)

    # duration: two ms values 92s apart for a tick crossfade (46m58s -> 48m30s).
    for suffix, ms in (("a", 2818000), ("b", 2910000)):
        raw = render_claudebar({"cwd": "/tmp/demo-app", "cost": {"total_duration_ms": ms}}, "duration")
        print(f"  duration-{suffix}: {re.sub(chr(27)+r'[^m]*m', '', raw)}")
        tmp = f"/tmp/opt_duration_{suffix}.html"
        with open(tmp, "w") as f:
            f.write(strip_html(raw))
        html_files.append((tmp, str(OUT_DIR / f"seg-duration-{suffix}.png")))

    # git: ahead-only -> diverged, same branch (demo-git-a / demo-git-b) so crop widths match.
    for suffix, cwd in (("a", "/tmp/demo-git-a"), ("b", "/tmp/demo-git-b")):
        raw = render_claudebar({"cwd": cwd}, "git")
        print(f"  git-{suffix}: {re.sub(chr(27)+r'[^m]*m', '', raw)}")
        tmp = f"/tmp/opt_git_{suffix}.html"
        with open(tmp, "w") as f:
            f.write(strip_html(raw))
        html_files.append((tmp, str(OUT_DIR / f"seg-git-{suffix}.png")))

    # cost: rising session USD.
    for suffix, usd in (("a", 0.42), ("b", 3.18)):
        raw = render_claudebar({"cwd": "/tmp/demo-app", "cost": {"total_cost_usd": usd}}, "cost")
        print(f"  cost-{suffix}: {re.sub(chr(27)+r'[^m]*m', '', raw)}")
        tmp = f"/tmp/opt_cost_{suffix}.html"
        with open(tmp, "w") as f:
            f.write(strip_html(raw))
        html_files.append((tmp, str(OUT_DIR / f"seg-cost-{suffix}.png")))

    # model: same model name, effort low -> high.
    for suffix, level in (("a", "low"), ("b", "high")):
        payload = {
            "cwd": "/tmp/demo-app",
            "model": {"display_name": "Claude Sonnet 4.6"},
            "effort": {"level": level},
        }
        raw = render_claudebar(payload, "model")
        print(f"  model-{suffix}: {re.sub(chr(27)+r'[^m]*m', '', raw)}")
        tmp = f"/tmp/opt_model_{suffix}.html"
        with open(tmp, "w") as f:
            f.write(strip_html(raw))
        html_files.append((tmp, str(OUT_DIR / f"seg-model-{suffix}.png")))

    print("\n  Rendering screenshots...")
    render_shots(html_files, ".strip")

    print("\n  Cropping to content bbox...")
    from PIL import Image
    for _, out_path in html_files:
        bbox_crop(out_path)
        with Image.open(out_path) as im:
            print(f"  {Path(out_path).name}: {im.size[0]}x{im.size[1]}")


if __name__ == "__main__":
    main()
