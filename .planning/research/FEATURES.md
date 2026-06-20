# Feature Landscape: claudebar Documentation & Screenshots

**Domain:** CLI tool documentation, terminal screenshots, install script UX
**Researched:** 2026-06-20
**Confidence:** MEDIUM (web sources + direct file inspection of gen_screenshots.py and install.sh)

---

## Table Stakes

Features users expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Hero visual at the top of README | First impression; users scan before reading | Low | Animated SVG preferred over static PNG — renders inline on GitHub |
| One-command install, copy-pasteable | Every popular CLI tool does this | Low | Already exists; needs update for prebuilt binary path |
| Prerequisites listed before install | Users with wrong setup waste time otherwise | Low | Nerd Font + git requirement already documented |
| Config file location + example block | Users need to know where to edit | Low | Already present; needs new themes listed |
| Subcommand reference table | CLI tools with multiple commands need this | Low | Already present; needs `migrate` added |
| Screenshots showing key states (calm/warn/crit/overlimit) | Users want to see what they're getting | Low | Strip PNGs already exist and are embedded |
| License section | GitHub default expectation | Trivial | Already present |
| Troubleshooting section | Nerd Font glyph boxes are the #1 install failure | Low | Currently inline; should be its own section |

## Differentiators

Features that set the README apart from generic CLI docs.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Animated SVG demo in README hero | Shows all 4 states cycling live; no GIF jitter or color degradation | Low | `gen_screenshots.py --svg` already produces this; just needs embedding |
| TUI configurator screenshot | Shows `claudebar config` in action — unique selling point vs bash fallback | Medium | Not currently in README; needs a new screenshot mode in gen_screenshots.py or a static PNG |
| Theme gallery in README | 16 themes is a strong differentiator; users choose by visual | Medium | One strip per theme at a fixed state; gen_screenshots.py needs a `--themes` mode |
| Segment behavior table | Documents each segment's conditional appearance (git drops when outside repo, effort drops for some models) | Low | Good prose already exists; convert to table for scannability |
| Prebuilt binary install path | Rust toolchain is a real barrier; binary download removes it | High | Requires GitHub Actions release pipeline first |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Asciinema embed in README | GitHub strips `<script>` tags; recording won't play inline | Use animated SVG or VHS-generated GIF instead |
| VHS (.tape) for claudebar demo | VHS outputs GIF/MP4/WebM only — no SVG; requires ttyd + running emulator; existing gen_screenshots.py SVG approach is lighter, self-contained, and already works | Keep gen_screenshots.py SVG path; optionally add VHS for a GIF fallback |
| Git LFS for screenshots | Requires LFS on server and all cloners; overkill for <20 PNGs | Commit PNGs directly to screenshots/ in main branch |
| Screenshots in orphan assets branch | Adds workflow complexity; cross-branch URLs are fragile on forks | Keep in screenshots/ on main; it's the dominant real-world pattern |
| Separate docs site | Overkill for a statusline tool; README is the product page | Single README, well structured |
| Windows section | No demand signaled; bash fallback works via WSL | Note WSL in troubleshooting |

---

## README Structure Recommendation

Informed by patterns from charmbracelet, ripgrep, bat, eza, and similar Rust CLI tools with strong READMEs.

### Recommended Section Order

```
1. Title + one-line description
2. Hero animated SVG (cycling all 4 states)
3. Requirements (Nerd Font + git — keep brief)
4. Install (one-command curl | bash — prominence)
   └── <details> Manual install (cargo, bash fallback, settings.json snippet)
5. TUI configurator demo (screenshot of `claudebar config`)
6. Themes & styles (strip gallery — all 16 themes shown)
7. Segments (table: name | what it shows | when it appears)
8. Configuration (TOML block + threshold explanation)
9. Subcommands (table)
10. Build from source
11. Troubleshooting (Nerd Font boxes, blank line, macOS Terminal limitations)
12. License
```

### What the Current README Is Missing

- Hero animated SVG (the file exists at `screenshots/animated.svg` but is not embedded)
- TUI configurator screenshot — no existing screenshot of `claudebar config`
- New themes (10 pending): catppuccin et al. are listed but ayu_mirage, cobalt2, etc. are absent
- `migrate` subcommand missing from the subcommand table
- `dev-context` segment missing from segment list
- Troubleshooting is inline in Install — should be its own `##` section for linkability

---

## Terminal Screenshot Approach

### Decision: Keep existing gen_screenshots.py SVG + static PNG approach

**Why not VHS:** VHS (charmbracelet) outputs GIF/MP4/WebM/ASCII only — SVG is not a supported output format as of the current release. It also requires a running terminal emulator (ttyd or similar) and a real terminal environment to record against. The existing gen_screenshots.py already handles the harder problem (rendering ANSI with Nerd Font glyphs correctly) via Playwright+Chrome+Docker.

**Why not asciinema:** GitHub READMEs strip `<script>` tags. Asciinema recordings require an external page to play; they do not embed inline in README.

**Why SVG over GIF for the hero:**
- Animated SVG is pure text, committed to the repo alongside source
- Renders inline on GitHub with no external CDN dependency
- Scales perfectly at any DPI (2x retina, 4K)
- Much smaller than equivalent GIF (typically 10-50x)
- The existing gen_screenshots.py `--svg` path already works without Docker

**When to use static PNG strips:**
- For showing individual states (calm/warn/crit/overlimit) — each strip at a fixed moment
- For the theme gallery — one strip per theme
- These require Docker + Hack Nerd Font (existing prereqs documented in gen_screenshots.py)

### Where to Commit Screenshots

Commit directly to `screenshots/` on the main branch. No LFS, no assets branch. Rationale:
- The screenshots directory already exists and is already used by the current README
- PNG strips are small (~50-100 KB each at 2x DPI)
- SVG is text, effectively zero storage overhead
- Orphan branches add fork-maintenance complexity that offers no real benefit at this scale

### New Screenshot Needed: TUI Configurator

The TUI configurator (`claudebar config`) is a key differentiator but has no screenshot. gen_screenshots.py currently only renders the statusline output. A TUI screenshot requires either:
1. A new mode in gen_screenshots.py that renders a fake TUI layout in HTML/SVG (same approach as existing code)
2. A static VHS recording (GIF) of the real TUI — requires running the binary interactively

**Recommended approach:** Add a `--tui` mode to gen_screenshots.py that renders an approximation of the TUI list UI in HTML, consistent with the existing terminal chrome CSS. The actual ratatui widget layout is known from src/tui/ui.rs.

---

## Install Script UX Patterns

### Current State

The current `install.sh` handles:
- Local checkout + cargo (builds from source)
- Fallback to downloading statusline-command.sh (bash script)
- Patching `~/.claude/settings.json`

### What It Lacks

1. **Prebuilt binary download** — No GitHub Releases path. Once the release pipeline exists, the install priority order should be: (1) prebuilt binary download, (2) cargo build from source, (3) bash script fallback.

2. **OS + arch detection** — No `uname -s` / `uname -m` detection. Required for selecting the correct release asset.

3. **Existing install check** — No `command -v claudebar` check with version comparison. A reinstall should print current version and prompt, not silently overwrite.

4. **Checksum verification** — No SHA256 check on downloaded binary.

5. **curl → wget fallback** — Only curl is used; wget is common on minimal Linux systems.

### Recommended Install Priority Order (post-release-pipeline)

```
1. Prebuilt binary from GitHub Releases (detect OS+arch, curl/wget download, SHA256 verify)
2. cargo install --path . (if SRC_DIR and cargo available — local dev workflow)
3. cargo install --git (if only cargo available — no local checkout)
4. bash fallback script (last resort, no toolchain needed, requires jq)
```

### Arch Detection Pattern (from grype/code-server precedent)

```bash
os=$(uname -s | tr '[:upper:]' '[:lower:]')
arch=$(uname -m)
case "$arch" in
  x86_64)  arch="x86_64" ;;
  aarch64) arch="aarch64" ;;
  arm64)   arch="aarch64" ;;  # macOS M-series
  *)       arch="unsupported" ;;
esac
```

Asset naming convention (matches cross-compilation targets):
`claudebar-{version}-{arch}-{os}.tar.gz`
Examples: `claudebar-0.2.0-x86_64-linux.tar.gz`, `claudebar-0.2.0-aarch64-macos.tar.gz`

### Existing Install Detection

```bash
if command -v claudebar >/dev/null 2>&1; then
  current=$(claudebar --version 2>/dev/null || echo "unknown")
  printf 'Found existing claudebar %s — updating...\n' "$current"
fi
```

### Error Message Quality

Current error messages are minimal. Good patterns:
- Red text for errors (already done)
- Print the exact command that failed
- Suggest the next step inline with the error
- End with a single "Installation failed" summary line that links to the troubleshooting doc

---

## Feature Dependencies

```
Prebuilt binary install path → GitHub Actions release pipeline (must exist first)
Theme gallery in README → 10 pending themes committed to main
TUI screenshot → gen_screenshots.py --tui mode (new work)
Animated SVG in README → screenshots/animated.svg regenerated with current states
```

## MVP Recommendation

Prioritize in this order:

1. **Commit pending themes** — unblocks theme gallery and fixes the theme list in README
2. **Regenerate animated.svg** — embed as hero in README (zero new tooling needed)
3. **Update README** — add SVG hero, missing segments/themes/subcommands, troubleshooting section, TUI screenshot placeholder
4. **Improve install.sh** — add OS+arch detection + existing install check (even before prebuilt binaries exist; sets up the structure)
5. **GitHub Actions release pipeline** — enables prebuilt binary path in install.sh
6. **TUI screenshot** — add after release pipeline so screenshots show the real version

Defer: Theme gallery (nice but not essential for initial polished README), VHS integration (SVG approach is better for claudebar's use case).

---

## Sources

- charmbracelet/vhs GitHub page (fetched directly) — confirmed VHS does NOT support SVG output
- grype install.sh (fetched directly) — arch detection patterns
- joncardasis/e6494afd538a400722545163eb2e1fa5 gist — assets branch hosting approach
- matiassingers/awesome-readme — README structure best practices
- makeareadme.com — standard README section conventions
- Web search results on terminal recording tool comparison

**Confidence notes:** VHS/SVG finding cross-verified via direct GitHub page fetch. Install script patterns cross-verified across multiple real-world scripts (grype, code-server). README structure is well-established convention with LOW variation across the ecosystem. Overall confidence: MEDIUM.
