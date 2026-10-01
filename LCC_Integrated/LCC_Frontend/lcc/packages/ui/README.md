# @lcc/ui

Design system primitives for OKESON-LCC. Built on shadcn/ui.

## What's here

- **primitives/** — shadcn wrappers (button, dialog, input, etc.). All carry forwardRef, displayName, ARIA attributes.
- **patterns/** — composed patterns (empty-state, error-state, data-table, form-field).
- **theme/** — ThemeProvider + useTheme + ModeToggle.
- **icons/** — Lucide re-exports + custom LinkedIn-domain icons.
- **a11y/** — skip-nav, live-region, focus-trap, visually-hidden.
- **utils/** — cn, format-date, format-number, format-currency, truncate.

## Tailwind preset

```js
// tailwind.config.ts
import preset from '@lcc/ui/tailwind-preset';

export default {
  presets: [preset],
  content: ['./src/**/*.{ts,tsx}'],
};
```

## Usage

```tsx
import { Button, Card, Input } from '@lcc/ui';

<Card>
  <Card.Header>
    <Card.Title>Members</Card.Title>
  </Card.Header>
  <Card.Content>
    <Input placeholder="Search members…" />
    <Button variant="default">Search</Button>
  </Card.Content>
</Card>
```

## Why this exists

Apps import from `@lcc/ui`, never re-implement. ESLint + review blocks duplicate primitives. Tailwind preset lives here, not in each app.
