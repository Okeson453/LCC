# proto/gen/typescript/

This directory is the codegen output for the TypeScript bindings of the backend's gRPC contracts. The `buf generate` command writes TS stubs here; they are then mirrored into `packages/api-types/src/generated/grpc/` by `tools/scripts/codegen-api-types.sh`.

For sources, see `../../` (parent `proto/` directory of the backend monorepo).
