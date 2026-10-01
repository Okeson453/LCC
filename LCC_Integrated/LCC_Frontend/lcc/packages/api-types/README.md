# @lcc/api-types

Generated TypeScript types from backend OpenAPI + protobuf contracts. Single source of truth for what flows over the wire between the frontend and the backend.

## Architecture

```
packages/api-types/
├── codegen/                  # codegen scripts (run via pnpm codegen)
│   ├── split-openapi.ts      # splits OpenAPI types into per-resource files
│   ├── mirror-grpc.ts        # mirrors proto/gen/typescript into here
│   ├── generate-zod.ts       # emits zod schemas from OpenAPI x-zod extensions
│   └── write-manifest.ts     # writes a generation timestamp manifest for CI
├── src/
│   ├── generated/            # ★ GITIGNORED — produced by codegen
│   │   ├── http/             # from schemas/openapi/api-gateway.yaml
│   │   ├── grpc/             # from proto/
│   │   └── events/           # from schemas/events/
│   ├── manual/               # hand-curated types not in OpenAPI
│   └── runtime/              # zod schemas + opaque brand types
└── tests/                    # schema-compat, generated-snapshot
```

## Usage

```ts
import type { Member, ContentItem, Approval } from '@lcc/api-types';
import { MemberSchema, ContentItemSchema } from '@lcc/api-types/zod-schemas';
import { isMemberId, memberId } from '@lcc/api-types/brand';

// Static types
function handleMember(m: Member) { /* ... */ }

// Runtime validation (defends against backend contract drift)
const member = MemberSchema.parse(apiResponse);

// Opaque brand types
const id: MemberId = memberId('0d9b2a5b-...');
```

## Codegen workflow

```bash
# 1. Edit schemas/openapi/api-gateway.yaml (or proto/) in the backend repo
# 2. Run codegen
pnpm codegen

# 3. Verify types and zod schemas
pnpm typecheck
pnpm test
```

If the codegen output is older than the source schema files, CI fails the build.

## Non-negotiable rules

- **Generated files are gitignored.** CI runs codegen before build.
- **zod schemas mirror DTOs** for runtime validation — never trust the wire.
- **Brand types are opaque.** `MemberId` is not `string`; you cannot pass a `ContactId` where a `MemberId` is expected.
- **No business state here.** This package contains types only, never API call logic.

## Adding a new endpoint

1. Edit `schemas/openapi/api-gateway.yaml` (or `proto/`).
2. Add the response/request shapes to `components/schemas`.
3. Run `pnpm codegen`.
4. Add a Zod schema extension via `x-zod` if you need runtime validation.
5. Update the API client in `apps/web-dashboard/src/lib/api/<resource>.ts`.
