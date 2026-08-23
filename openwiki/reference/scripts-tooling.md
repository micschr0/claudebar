---
type: "Reference"
title: "Scripts & tooling"
openwiki_generated: true
---

# Scripts & tooling

Python/JS/bash tooling under `scripts/` for docs, screenshots, benchmarks,
and demo assets. Not part of the binary.

| Script | Purpose | Output consumers |
|---|---|---|
| `gen_screenshots.py` (40KB) | Render the Rust binary → PNG/SVG strips/pills via Docker or host Chrome | README, docs |
| `gen-gallery.sh` | Theme×style gallery into `docs/index.html` between markers; fails fast on missing binary/themes | docs site |
| `make_demo_repos.sh` | 7 deterministic demo repos under `/tmp` with exact state contract | screenshots, demo |
| `benchmark.sh` | SLO guard (`p95 < 100ms`, subprocess ≤5) | CI benchmark |
| `gen_terminal_gifs.sh`, `gen_social_preview.py`, `demo_intro.py`, `window_frame.py`, `gen_tui_screenshot.py`, `gen_logos.py`, `ansi2html.py` | marketing/docs media | README, social, video |

## Tests

- `tests/scripts.bats` — gallery regeneration markers, binary-missing
  fast-fail.
- `tests/demo_repos.bats` — demo-repo state contract.

## Conventions

- All shell scripts pass `shellcheck` (lint task).
- Screenshot scripts need Chrome (host or Docker); deterministic fixtures.