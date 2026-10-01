# OpenAPI surface

The frontend consumes `schemas/openapi/api-gateway.yaml` as its single API
contract. Regenerate generated types via:

```bash
pnpm openapi:gen
```

That script invokes `openapi-typescript` and emits into
`packages/api-types/src/generated/http/`. The generated file is checked into
source control to keep builds deterministic.

## Versioning

- The OpenAPI document lists `info.version`. Bumps require a coordinated
  frontend release.
- Backwards-compatible changes (additive only) ship on Monday.
- Breaking changes require a major release and ADR.
