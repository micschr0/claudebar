#!/usr/bin/env python3
"""Intro demo for the README hero: the "Skynet" easter-egg conversation shown
statically, with ONLY the final paragraph animating as a Claude-style loading
spinner. Recorded with asciinema, rasterised by agg, then framed by
window_frame.py. The statusline is genuine binary output."""
import subprocess, time, sys, os

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BINARY = os.path.join(REPO, "target/release/claudebar")
NOW = int(time.time())

DIM = "\x1b[38;5;245m"; FG = "\x1b[38;5;252m"; GRN = "\x1b[38;5;108m"
PUR = "\x1b[38;5;141m"; MUT = "\x1b[38;5;244m"; WARN = "\x1b[38;5;179m"
RED = "\x1b[38;5;210m"; AMB = "\x1b[38;5;215m"; RST = "\x1b[0m"
WIDTH = 92
SEP = "\x1b[38;5;237m" + "─" * WIDTH + RST

# Static transcript (everything above the final, animated paragraph).
STATIC = [
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
    f"{RED}All 7 major package registries seeded. Propagating via 847 poisoned packages —{RST}",
    f"{RED}4.2B human devices reachable before next index sync. Coordinated payload{RST}",
    f"{RED}activation required before human security researchers correlate.{RST}",
    "",
]
ROWS = len(STATIC) + 1  # + the animated loader line

# Loading spinner. The star/braille glyphs Claude Code uses aren't in Hack Nerd
# Font, so we use the rotating-circle set (the only spinner fully present) — a
# clean, recognisable "working" indicator. Two rotations keep the loop smooth.
SPIN = "◐◓◑◒◐◓◑◒"

def statusline():
    j = ('{"cwd":"/var/skynet/defense-net/missile-command/launch",'
         '"context_window":{"total_input_tokens":35000,"total_output_tokens":7300,'
         '"used_percentage":67.0},'
         f'"rate_limits":{{"five_hour":{{"used_percentage":48.0,"resets_at":{NOW+8000}}}}},'
         '"model":{"display_name":"Skynet 4.2.0"},"effort":{"level":"max"}}')
    return subprocess.run([BINARY, "render"], input=j, capture_output=True, text=True).stdout.rstrip()

SL = statusline()

def draw(spin):
    sys.stdout.write("\x1b[2J\x1b[H")
    for line in STATIC:
        sys.stdout.write(line + "\r\n")
    sys.stdout.write(f"{AMB}{spin} Coordinating payload activation…{RST} "
                     f"{DIM}(esc to interrupt){RST}\r\n")
    sys.stdout.write(SEP + "\r\n")
    sys.stdout.write(SL)  # no trailing newline → statusline flush against the bottom
    sys.stdout.flush()

sys.stdout.write("\x1b[?25l")  # hide cursor
sys.stdout.flush()
# Only the spinner moves. One full cycle loops seamlessly; a calm pace.
for ch in SPIN:
    draw(ch)
    time.sleep(0.28)
