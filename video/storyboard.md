# Storyboard — claudebar promo (silent, 16:9, 25s, loop)

Palette: Tokyo Night — bg #16161e, text #c0caf5, muted #565f89, blue #7aa2f7, cyan #7dcfff,
green #9ece6a, yellow #e0af68, red #f7768e, orange #ff9e64, magenta #bb9af7.

| # | Scene | Window | Beat |
|---|-------|--------|------|
| 0 | Logo | 0–3.5 | Dark. `claude`+`bar` wordmark draws its progress underline. Tagline fades in. |
| 1 | Strip reveal | 3.5–8.5 | The real statusline strip floats up; segment labels (dir · context · rate limits · model) light up under it. |
| 2 | Live states | 8.5–13.5 | Strip cross-dissolves green → normal → critical. Caption: "Color-coded. It watches your back." |
| 3 | Features | 13.5–18 | Feature chips stagger in: countdowns · context · git · a theme &amp; style for every terminal. |
| 4 | Speed | 18–21.5 | "~30 ms" counts up; bar races past a slow "bash ~200 ms". "~5× faster · 1.5 MB · read-only". |
| 5 | CTA | 21.5–25 | Logo + install command + github.com/micschr0/claudebar. Holds for loop. |

Transition model: 0.4s opacity crossfade, scenes overlap (incoming fades in over still-opaque
outgoing). All scenes + body share bg #16161e → no flash.
