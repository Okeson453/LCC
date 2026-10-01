//! `lcc-db` — Postgres client wrapper with RLS context setter, optimistic
//! concurrency, and the audit-log-in-same-transaction helper.
//!
//! ### Boundary
//! This crate is consumed by every Rust Core service that needs DB access.
//! Python Intelligence services do NOT use this crate — they use
//! `packages/db` (asyncpg) which mirrors only the RLS setter, not the audit
//! helper (audit writes from Python go through `packages/intelligence-common`
//! which POSTs to `audit-svc` rather than calling audit-client directly).

pub mod optimistic;
pub mod pool;
pub mod rls;
pub mod tx;

pub use optimistic::{optimistic_update, OptimisticUpdateError, OptimisticUpdateResult};
pub use pool::{build_pool, PoolConfig};
pub use rls::{set_member_context, set_role_context, RlsContext, RlsError};
pub use tx::{audit_tx, AuditTx, AuditTxResult};
