//! `lcc-proto` — message types and gRPC stubs for the LCC service contract.
//!
//! Two compile modes:
//!
//! 1. **Default (handwritten):** the message types in this crate are
//!    hand-written Rust structs that serialize to JSON via serde. This
//!    avoids the build-time dependency on `protoc` and `buf`. The
//!    gRPC services use `tonic::transport::Server` with `JsonCodec`,
//!    which is the right transport for JSON-over-tonic deployments
//!    (the default for `tonic` when no proto stubs are provided).
//!
//! 2. **`proto-binary` feature (opt-in):** when this feature is enabled,
//!    `tonic::include_proto!` is invoked against the `.proto` source
//!    files. The build script (`build.rs`) runs `buf generate` if
//!    available. This requires `protoc` to be installed locally.
//!
//! ## When to use which
//! - **Local dev / CI without protoc**: leave the default. Services
//!   run with JSON-over-tonic, which is wire-compatible with the
//!   Python intelligence services via `grpcio` + the python proto
//!   stubs in `proto/gen/python/`.
//! - **Production deployment with strict wire format**: enable
//!   `proto-binary`. This requires running `buf generate` first to
//!   populate `src/gen/`.

#![allow(clippy::all)]

pub mod handwritten;
pub mod scoring_handwritten;

// Default mode: hand-written types are the canonical types.
pub use scoring_handwritten::*;

// When the `proto-binary` feature is enabled, the real protobuf types are
// included from `src/gen/`. build.rs writes those stubs (via `buf` when it is
// on PATH, otherwise `tonic-build` + `protoc`).
//
// prost emits one file per proto *package* and writes cross-package
// references as `super::…` chains counted from the including module. The
// include tree below therefore mirrors the proto package hierarchy
// (`lcc.v1.<pkg>`) exactly — flattening it to `pub mod <pkg>` made
// `lcc.v1.network` emit `super::super::super::google::r#type::Date`, which
// resolves above the crate root and failed to compile (E0433).
#[cfg(feature = "proto-binary")]
pub mod google {
    pub mod r#type {
        include!("gen/google.r#type.rs");
    }
}

#[cfg(feature = "proto-binary")]
pub mod lcc {
    pub mod v1 {
        pub mod common {
            include!("gen/lcc.v1.common.rs");
        }
        pub mod intelligence {
            include!("gen/lcc.v1.intelligence.rs");
        }
        pub mod compliance {
            include!("gen/lcc.v1.compliance.rs");
        }
        pub mod integration {
            include!("gen/lcc.v1.integration.rs");
        }
        pub mod profile {
            include!("gen/lcc.v1.profile.rs");
        }
        pub mod content {
            include!("gen/lcc.v1.content.rs");
        }
        pub mod engagement {
            include!("gen/lcc.v1.engagement.rs");
        }
        pub mod network {
            include!("gen/lcc.v1.network.rs");
        }
        pub mod outreach {
            include!("gen/lcc.v1.outreach.rs");
        }
        pub mod analytics {
            include!("gen/lcc.v1.analytics.rs");
        }
        pub mod approval {
            include!("gen/lcc.v1.approval.rs");
        }
        pub mod identity {
            include!("gen/lcc.v1.identity.rs");
        }
        pub mod events {
            include!("gen/lcc.v1.events.rs");
        }
    }
}
