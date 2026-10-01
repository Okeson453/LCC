//! Placeholder for additional handwritten message types. See
//! `scoring_handwritten.rs` for the canonical example.
//!
//! New domains that need message types should either:
//! 1. Run `buf generate proto` (requires `buf` installed) and let the
//!    generated `prost` types live under `src/gen/` (gated by the
//!    `proto-binary` feature).
//! 2. Hand-write the message types here, mirroring the `.proto` field
//!    tags 1:1 so the JSON serialization is wire-compatible.
//!
//! Files in this directory MUST keep the proto tag number in a comment
//! so a future `protoc` run can be diff'd against the handwritten code:
//!     `/// proto field 1`
//! This is non-negotiable for any new handwritten message type.
