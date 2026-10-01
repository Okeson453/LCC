# Design tokens

Single source of truth lives at `packages/tokens`.

## What is provided

- Color tokens (semantic, not literal).
- Spacing scale (4pt grid).
- Radius, font sizes, line-heights, motion durations.
- **Tier palette**: 5 risk levels, each with foreground + background + outline.

## Consumption

- Web Dashboard CSS via `@lcc/tokens/css` (selectors into `:root`).
- Tailwind via `tailwind.config.ts` `presets: [require('@lcc/tokens/tailwind-preset')]`.
- React components via `@lcc/ui` which consume tokens via Tailwind utility classes.

## Tier colors

| Tier | Color (light theme) | Color (dark theme) |
|------|---------------------|--------------------|
| 1    | green-500           | green-400          |
| 2    | sky-500             | sky-400            |
| 3    | amber-500           | amber-400          |
| 4    | orange-500          | orange-400         |
| 5    | red-500             | red-400            |

Use `<RiskTierBadge>` to surface tier; do not introduce ad-hoc tier color keys.

## Typography

- Inter as primary UI face.
- JetBrains Mono for code/IDs (trace IDs, etc.).
- Headings sizes follow the 8pt vertical scale (12, 14, 16, 20, 24, 32, 48).

## Motion

- Fade-in 150ms, slide-over 220ms ease-out.
- Respect `prefers-reduced-motion: reduce` — disable non-essential animation.
