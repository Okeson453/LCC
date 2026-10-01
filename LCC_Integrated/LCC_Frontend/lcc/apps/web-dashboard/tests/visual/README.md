# Visual regression tests

Snapshots are captured using Playwright's built-in screenshot API and compared
against baselines in `tests/visual/__snapshots__/`.

Update baselines with:

```bash
pnpm --filter @lcc/web-dashboard test:visual:update
```

Visual tests intentionally have a manual review step before PR merge; changes
to design tokens should be reviewed against the design system ADRs first.
