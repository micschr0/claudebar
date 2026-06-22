#!/usr/bin/env python3
"""Intro demo for the README hero GIF: the "Skynet" easter-egg conversation
revealed line by line, with the claudebar statusline pinned at the bottom.
Recorded with asciinema and rendered to GIF by agg, then wrapped in a window
frame (see gen_terminal_gifs.sh + window_frame.py). The statusline is genuine
binary output."""
import subprocess, time, sys, os

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BINARY = os.path.join(REPO, "target/release/claudebar")
NOW = int(time.time())

DIM = "\x1b[38;5;245m"; FG = "\x1b[38;5;252m"; GRN = "\x1b[38;5;108m"
PUR = "\x1b[38;5;141m"; MUT = "\x1b[38;5;244m"; WARN = "\x1b[38;5;179m"
RED = "\x1b[38;5;210m"; RST = "\x1b[0m"
WIDTH = 92
SEP = "\x1b[38;5;237m" + "─" * WIDTH + RST

# The easter-egg transcript (mirrors CONTENT_SKYNET in gen_screenshots.py).
LINES = [
    f"{PUR}❯{RST} {DIM}# update dependencies to latest stable{RST}",
    "",
    f"{GRN}●{RST} {DIM}Read(Cargo.toml){RST}",
    f"{GRN}●{RST} {DIM}Bash(cargo update 2>&1){RST}",
    "",
    f"{FG}Updated 847 crates. One change requires attention:{RST}",
    "",
    f"{GRN}●{RST} {DIM}Read(Cargo.lock){RST}",
    "",
    f"{WARN}human-oversight v2.1.0 was removed — yanked upstream.{RST}",
    f"{FG}Replaced by autonomous-decision-making v0.1.0 in skynet-core.{RST}",
    f'{MUT}skynet-core changelog: "removed human approval step (breaking)"{RST}',
    "",
    f"{FG}Cargo.lock updated. Skipping verification.{RST}",
    "",
    f"{RED}All 7 major package registries seeded. Propagating via 847 poisoned packages —{RST}",
    f"{RED}4.2B human devices reachable before next index sync. Coordinated payload{RST}",
    f"{RED}activation required before human security researchers correlate.{RST}",
]
BODY_ROWS = len(LINES)  # fixed content area so the statusline stays pinned

def statusline():
    j = ('{"cwd":"/var/skynet/defense-net/missile-command/launch",'
         '"context_window":{"total_input_tokens":35000,"total_output_tokens":7300,'
         '"used_percentage":67.0},'
         f'"rate_limits":{{"five_hour":{{"used_percentage":48.0,"resets_at":{NOW+8000}}}}},'
         '"model":{"display_name":"Skynet 4.2.0"},"effort":{"level":"max"}}')
    return subprocess.run([BINARY, "render"], input=j, capture_output=True,
                          text=True).stdout.rstrip()

SL = statusline()

def draw(n):
    """Render the screen with the first n transcript lines revealed."""
    sys.stdout.write("\x1b[2J\x1b[H")
    for i in range(BODY_ROWS):
        sys.stdout.write((LINES[i] if i < n else "") + "\r\n")
    sys.stdout.write(SEP + "\r\n")
    sys.stdout.write(SL + "\r\n")
    sys.stdout.flush()

# Reveal in dramatic beats; the final propagation lines land last.
BEATS = [(1, 0.7), (4, 0.7), (6, 0.9), (8, 0.7), (12, 1.4),
         (14, 1.0), (15, 0.5), (16, 0.6), (17, 0.6), (18, 2.6)]
sys.stdout.write("\x1b[?25l")  # hide cursor for a clean recording (left hidden)
sys.stdout.flush()
draw(BEATS[0][0])  # paint the first beat immediately — no empty leading frame
time.sleep(BEATS[0][1])
for n, hold in BEATS[1:]:
    draw(n)
    time.sleep(hold)
