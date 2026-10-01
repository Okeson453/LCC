# Monorepo tooling

- pnpm workspaces (`pnpm-workspace.yaml`).
- Turbo (`turbo.json`) for task orchestration and incremental builds.
- TypeScript path aliases: `@/*` (web-dashboard / extension) and `@lcc/*` (shared packages).
- ESLint config at `.eslintrc.cjs` enforces boundaries via `eslint-plugin-import` rules.
- Prettier configs in each package share `.prettierrc` at the root.

## Scripts (root)
- `pnpm -r --filter './packages/*' typecheck`
- `pnpm --filter @lcc/web-dashboard dev`
- `pnpm --filter @lcc/web-dashboard build`
- `pnpm --filter @lcc/browser-extension build:dev`
- `pnpm -r test`
- `pnpm lint && pnpm format:check`

## Boundary enforcement
The script `tools/scripts/check_boundaries_frontend.sh` fails the build if any
relative import crosses app↔package or relative↔workspace boundaries outside
the allowed paths.

## Codegen
- `tools/scripts/codegen-api-types.sh` runs `openapi-typescript` and emits into `packages/api-types/src/generated/`.
- Schemas live under `schemas/openapi`, `schemas/events`.
- Buf configuration at `proto/buf.yaml` — produces TS via `protoc-gen-ts_proto`.
