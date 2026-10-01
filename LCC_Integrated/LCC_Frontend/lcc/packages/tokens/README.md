# @lcc/tokens

Design tokens for OKESON-LCC. Single source of truth for:

- **Color** — semantic palette, brand colors, dark/light/high-contrast themes, risk-tier palette (Tier 1–5)
- **Spacing** — 0–96 scale (Tailwind-aligned)
- **Typography** — sans/mono font families, type scale
- **Radius** — sm/md/lg/full
- **Shadow** — sm/md/lg/xl
- **Motion** — duration + easing tokens, respects `prefers-reduced-motion`

## Usage

### CSS

```css
@import '@lcc/tokens/css/colors.css';
@import '@lcc/tokens/css/spacing.css';
@import '@lcc/tokens/css/typography.css';

.button {
  background: var(--color-primary);
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-md);
}
```

### Tailwind

```js
// tailwind.config.ts
import preset from '@lcc/tokens/tailwind';

export default {
  presets: [preset],
  content: ['./src/**/*.{ts,tsx}'],
};
```

### TypeScript

```ts
import { tokens, riskTierColor, riskTierLabel } from '@lcc/tokens';

const tier1Color = riskTierColor(1); // "var(--color-tier-1)"
```

## Tier palette

The risk tier palette is critical to the approval-first UX. Tier 1 actions (draft-only) require no approval; Tier 5 actions (apply, proposal) require explicit human approval via the dialog.

| Tier | Color token | Usage |
|---|---|---|
| 1 | `--color-tier-1` (slate) | Drafts |
| 2 | `--color-tier-2` (blue) | Publish, comment |
| 3 | `--color-tier-3` (amber) | Connection request |
| 4 | `--color-tier-4` (orange) | Direct message |
| 5 | `--color-tier-5` (red) | Apply, proposal |

## Themes

- `dark.css` — default for "command center" aesthetic (per Frontend Design Concept §17.2)
- `light.css` — standard light theme
- `high-contrast.css` — WCAG AAA palette for the high-contrast accessibility variant

## Reduced motion

All motion tokens honor `@media (prefers-reduced-motion: reduce)`. The `--motion-duration-*` tokens resolve to `0ms` when the user has reduced-motion enabled.
