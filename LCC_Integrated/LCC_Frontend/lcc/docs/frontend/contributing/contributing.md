# Contributing

## Local setup

```bash
pnpm install
pnpm -r --filter './packages/*' build
pnpm --filter @lcc/web-dashboard dev
```

## Conventions

- Strict TS, ESLint clean, Prettier formatted.
- One changeset per feature.
- No emoji in commits.
- Update `IMPLEMENTATION_INVENTORY.md` with new file rows.

## Releasing

1. PR against `main`.
2. CI runs `lint`, `typecheck`, `test`, `e2e`, `bundle-size`, `lighthouse`.
3. Merge → release PR opens automatically.
4. Tag with semver; deploy image; verify smoke.
