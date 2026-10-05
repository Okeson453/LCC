#!/usr/bin/env bash
# codegen-api-types.sh — Generate @lcc/api-types from backend contracts.
#
# Pipeline:
#   1. Run `openapi-typescript` on the CANONICAL contract,
#      `Contract/openapi/lcc-api-canonical.yaml`
#      → emits HTTP types to `packages/api-types/src/generated/http/`
#
#      This previously read `schemas/openapi/api-gateway.yaml` — a superseded
#      55-path document with no `/api/v1` namespace that the contract audit
#      explicitly replaced (70 paths / 82 operations). Generating the frontend's
#      types from it meant `@lcc/api-types` never described the API the gateway
#      and the services actually serve.
#   2. Run `buf generate` on `proto/` with TypeScript plugin
#      → emits gRPC stubs to `proto/gen/typescript/`, then mirror
#      → into `packages/api-types/src/generated/grpc/`
#   3. Run JSON-Schema → TypeScript for `schemas/events/*.schema.json`
#      → emits event types to `packages/api-types/src/generated/events/`
#   4. Generate zod schemas from OpenAPI (where `x-zod` is present)
#      → emits to `packages/api-types/src/runtime/zod-schemas.ts`
#
# Exit code 0 = success; 1 = any step failed.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

LOG_PREFIX="[codegen]"
say() { printf '%s %s\n' "$LOG_PREFIX" "$*"; }
die() { printf '%s ERROR: %s\n' "$LOG_PREFIX" "$*" >&2; exit 1; }

command -v pnpm >/dev/null 2>&1 || die "pnpm not found on PATH"
command -v buf >/dev/null 2>&1 || die "buf not found on PATH (https://buf.build)"

OUT_DIR="packages/api-types/src/generated"
mkdir -p "$OUT_DIR/http" "$OUT_DIR/grpc" "$OUT_DIR/events"

# 1. OpenAPI → HTTP types
CANONICAL_OPENAPI="../Contract/openapi/lcc-api-canonical.yaml"
[ -f "$CANONICAL_OPENAPI" ] || die "canonical contract not found at $CANONICAL_OPENAPI"

say "OpenAPI → TypeScript (lcc-api-canonical.yaml → $OUT_DIR/http/)"
pnpm exec openapi-typescript \
  "$CANONICAL_OPENAPI" \
  --output "$OUT_DIR/http/index.ts" \
  --enum \
  --immutable \
  --default-non-nullable \
  --empty-objects-unknown \
  --alphabetize \
  --export-type

# Split the monolithic index.ts into per-resource files (one HTTP type file per
# resource). This is the codegen step that mirrors `packages/api-types/src/generated/http/`
# into individual files: member.ts, profile.ts, content.ts, etc.
say "Splitting OpenAPI types into per-resource files"
pnpm exec tsx packages/api-types/codegen/split-openapi.ts

# 2. Protobuf → TypeScript gRPC stubs
say "Protobuf → TypeScript (proto/ → $OUT_DIR/grpc/)"
(cd proto && buf generate --template buf.gen.ts.yaml)

# Mirror the gRPC stubs from `proto/gen/typescript/` into `packages/api-types/src/generated/grpc/`
say "Mirroring gRPC stubs into packages/api-types/src/generated/grpc/"
pnpm exec tsx packages/api-types/codegen/mirror-grpc.ts

# 3. JSON-Schema → TypeScript for events
say "JSON-Schema → TypeScript (schemas/events/ → $OUT_DIR/events/)"
for schema in schemas/events/*.schema.json; do
  out="$OUT_DIR/events/$(basename "$schema" .schema.json).ts"
  pnpm exec json-schema-to-typescript "$schema" "$out" --bannerComment "/* eslint-disable */\n// @generated from $schema" >/dev/null
done

# 4. Generate zod schemas from OpenAPI extensions
say "OpenAPI → zod schemas → packages/api-types/src/runtime/zod-schemas.ts"
pnpm exec tsx packages/api-types/codegen/generate-zod.ts

# 5. Format
say "Formatting generated files"
pnpm exec prettier --write "$OUT_DIR" "packages/api-types/src/runtime" "packages/api-types/src/manual" >/dev/null

# 6. Compare generated timestamps against source for CI check
say "Writing generation manifest"
pnpm exec tsx packages/api-types/codegen/write-manifest.ts

say "Codegen complete"
