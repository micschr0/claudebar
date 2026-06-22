# claudebar logo variants

Five logo concepts, all built from the default **tokyo-night** palette
(`#1a1b2e` tile · `#7aa2f7` blue · `#bb9af7` purple · `#7dcfff` cyan).
Each comes as a square `vN-mark.svg` (app icon) and a `vN.svg` wordmark lockup,
with matching PNG rasters. Regenerate the SVGs with `scripts/gen_logos.py`.

| # | Concept | Mark | Lockup |
|---|---------|------|--------|
| 1 | Prompt chevron + statusline bar | <img src="v1-mark.svg" width="72"> | <img src="v1.svg" width="260"> |
| 2 | Powerline segments (the literal statusline) | <img src="v2-mark.svg" width="72"> | <img src="v2.svg" width="260"> |
| 3 | Equalizer / status bars | <img src="v3-mark.svg" width="72"> | <img src="v3.svg" width="260"> |
| 4 | Context ring gauge + chevron | <img src="v4-mark.svg" width="72"> | <img src="v4.svg" width="260"> |
| 5 | Terminal window with a statusline | <img src="v5-mark.svg" width="72"> | <img src="v5.svg" width="260"> |

The marks are self-contained (dark tile) and the lockups sit on a dark panel, so
all variants stay legible on both light and dark README themes.

## Alternative directions (different palettes + styles)

Three concepts that drop the tokyo-night blue/purple for entirely different
colour worlds and visual styles, drawn from other themes seen in the
screenshots. Regenerate with `scripts/gen_logos_alt.py`.

| Name | Palette / style | Mark | Lockup |
|------|-----------------|------|--------|
| `warm` | Gruvbox — warm retro, boxy level-meter | <img src="warm-mark.svg" width="72"> | <img src="warm.svg" width="260"> |
| `soft` | Catppuccin — soft pastel, rounded pill + knob | <img src="soft-mark.svg" width="72"> | <img src="soft.svg" width="260"> |
| `light` | Status semaphore — flat, light, green→amber→red | <img src="light-mark.svg" width="72"> | <img src="light.svg" width="260"> |

`warm` and `soft` are self-contained dark tiles/panels; `light` is a deliberately
light design (white tile, light panel) for a completely different mood.
