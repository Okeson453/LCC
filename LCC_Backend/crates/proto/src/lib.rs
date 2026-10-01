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

// When the `proto-binary` feature is enabled, attempt to include the
// real protobuf types from src/gen/. The build.rs script writes those
// stubs when `buf` is available.
#[cfg(feature = "proto-binary")]
pub mod common {
    pub mod trace {
        include!("gen/lcc.v1.common.trace.rs");
    }
    pub mod pagination {
        include!("gen/lcc.v1.common.pagination.rs");
    }
}

#[cfg(feature = "proto-binary")]
pub mod intelligence {
    pub mod scoring {
        include!("gen/lcc.v1.intelligence.scoring.rs");
    }
    pub mod ai {
        include!("gen/lcc.v1.intelligence.ai.rs");
    }
    pub mod opportunity {
        include!("gen/lcc.v1.intelligence.opportunity.rs");
    }
    pub mod kb {
        include!("gen/lcc.v1.intelligence.kb.rs");
    }
    pub mod voice {
        include!("gen/lcc.v1.intelligence.voice.rs");
    }
}

#[cfg(feature = "proto-binary")]
pub mod compliance {
    pub mod governor {
        include!("gen/lcc.v1.compliance.governor.rs");
    }
    pub mod admin {
        include!("gen/lcc.v1.compliance.admin.rs");
    }
}

#[cfg(feature = "proto-binary")]
pub mod integration {
    pub mod execute {
        include!("gen/lcc.v1.integration.execute.rs");
    }
}

#[cfg(feature = "proto-binary")]
pub mod profile {
    pub mod profile_svc {
        include!("gen/lcc.v1.profile.profile.rs");
    }
}

#[cfg(feature = "proto-binary")]
pub mod content {
    pub mod content_svc {
        include!("gen/lcc.v1.content.content.rs");
    }
}

#[cfg(feature = "proto-binary")]
pub mod engagement {
    pub mod engagement_svc {
        include!("gen/lcc.v1.engagement.engagement.rs");
    }
}

#[cfg(feature = "proto-binary")]
pub mod network {
    pub mod network_svc {
        include!("gen/lcc.v1.network.network.rs");
    }
}

#[cfg(feature = "proto-binary")]
pub mod outreach {
    pub mod outreach_svc {
        include!("gen/lcc.v1.outreach.outreach.rs");
    }
}

#[cfg(feature = "proto-binary")]
pub mod analytics {
    pub mod analytics_svc {
        include!("gen/lcc.v1.analytics.analytics.rs");
    }
}

#[cfg(feature = "proto-binary")]
pub mod approval {
    pub mod approval_svc {
        include!("gen/lcc.v1.approval.approval.rs");
    }
}

#[cfg(feature = "proto-binary")]
pub mod identity {
    pub mod identity_svc {
        include!("gen/lcc.v1.identity.identity.rs");
    }
}

#[cfg(feature = "proto-binary")]
pub mod events {
    pub mod events {
        include!("gen/lcc.v1.events.events.rs");
    }
}
