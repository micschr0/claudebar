#!/usr/bin/env python3
"""Intro demo for the README hero GIF: cycles the claudebar statusline through
four states in a real terminal. Recorded with asciinema and rendered to GIF by
agg (see gen_terminal_gifs.sh). Every statusline line is genuine binary output."""
import subprocess, time, sys, os

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BINARY = os.path.join(REPO, "target/release/claudebar")
NOW = int(time.time())

DIM = "\x1b[38;5;240m"; FG = "\x1b[38;5;252m"; GRN = "\x1b[38;5;108m"
PUR = "\x1b[38;5;141m"; YEL = "\x1b[38;5;179m"; RST = "\x1b[0m"
SEP = "\x1b[38;5;236m" + "─" * 92 + RST

TRANSCRIPT = [
    f"{PUR}❯{RST} {DIM}# refactor auth middleware to use JWT validation{RST}",
    "",
    f"{GRN}●{RST} {DIM}Read(src/auth.rs){RST}",
    f"{GRN}●{RST} {DIM}Read(src/config/jwt.rs){RST}",
    "",
    f"{FG}Replacing DB-backed session validation with stateless JWT verification.{RST}",
    "",
    f"{GRN}●{RST} {DIM}Edit(src/auth.rs) +47 -23{RST}",
    f"{GRN}●{RST} {DIM}Bash(cargo test middleware){RST}",
    "",
    f"{FG}All 14 tests pass. Set {YEL}JWT_SECRET{FG} before deploying.{RST}",
]

# (ctx%, tok_in, tok_out, rl%, reset_offset_seconds, label)
STATES = [
    (67.0,  55000,  9200, 38.0, 12000, "normal"),
    (72.0,  90000, 18000, 62.0,  6300, "warning"),
    (88.0, 140000, 26000, 80.0,  2700, "critical"),
    (101.0,160000,  8000, 93.0,   900, "over limit"),
]

def statusline(ctx, ti, to, rl, off):
    j = (f'{{"cwd":"/home/dev/projects/demo-app",'
         f'"context_window":{{"total_input_tokens":{ti},"total_output_tokens":{to},'
         f'"used_percentage":{ctx}}},'
         f'"rate_limits":{{"five_hour":{{"used_percentage":{rl},"resets_at":{NOW+off}}}}},'
         f'"model":{{"display_name":"Claude Sonnet 4.6"}}}}')
    env = {**os.environ, "HOME": "/home/dev"}
    return subprocess.run([BINARY, "render"], input=j, capture_output=True,
                          text=True, env=env).stdout.rstrip()

def frame(ctx, ti, to, rl, off, label):
    sys.stdout.write("\x1b[2J\x1b[H")  # clear + home
    for line in TRANSCRIPT:
        sys.stdout.write(line + "\r\n")
    sys.stdout.write("\r\n" + SEP + "\r\n")
    sys.stdout.write(statusline(ctx, ti, to, rl, off) + "\r\n")
    sys.stdout.write(f"{DIM}  {label} — context {ctx:.0f}%  ·  rate limit {rl:.0f}%{RST}\r\n")
    sys.stdout.flush()

time.sleep(0.6)
for st in STATES:
    frame(*st)
    time.sleep(3.0)
time.sleep(0.8)
