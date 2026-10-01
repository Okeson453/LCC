//! `lcc-audit-client` — used by every Rust Core Engine service for audit-log
//! writes.
//!
//! Two-tier design:
//! - **In-transaction writes** (via `lcc-db::AuditTx::record`): when the audit
//!   row must be committed atomically with an entity write (Non-Negotiable §10).
//! - **Spawn-and-forget** (via [`AuditClient::record_quick`]): for low-stakes
//!   audit-only events (e.g., a draft was composed, a draft was viewed) where
//!   the audit failure must not block the user-visible path.
//!
//! The audit-svc (service) is the long-running consumer of these writes; it
//! owns the INSERT-only DB role enforcement. The client is a thin gRPC
//! wrapper that handles retries + idempotency.
//!
//! ## Boundary
//! Every service `Cargo.toml` adds `lcc-audit-client = { workspace = true }`
//! in the workspace dependency array. A workspace lint enforces that no
//! service may write to `audit_log` directly (must go through this client).

pub mod client;

pub use client::{AuditClient, AuditClientConfig, AuditError, AuditEvent, AuditOutcome};
