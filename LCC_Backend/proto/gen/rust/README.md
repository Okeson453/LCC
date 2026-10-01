# Generated Rust proto stubs

This directory contains the auto-generated Rust gRPC stubs from
`/workspace/lcc/proto/lcc/v1/*.proto`.

The Rust workspace's `crates/proto/` crate is generated via:

```bash
buf generate
# → /workspace/lcc/proto/gen/rust/lcc/v1/{common,compliance,...}.rs
```

The `crates/proto/src/lib.rs` includes these via `tonic::include_proto!`.
