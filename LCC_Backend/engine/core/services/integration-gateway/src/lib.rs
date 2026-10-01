//! Integration Gateway — the ONLY service permitted to call LinkedIn-facing
//! endpoints (Non-Negotiable §1, Source §10).
//!
//! Verifies `permit_token` from the Compliance Governor, then routes to
//! Track A (official API) or Track B (browser-assist WSS), enforces
//! idempotency, runs the circuit breaker, and detects restriction signals.
//!
//! ### Boundary
//! - Only this service may import `crates/integrations`.
//! - Only this service holds LinkedIn OAuth tokens (stored in Vault).

pub mod audit;
pub mod backoff;
pub mod circuit_breaker;
pub mod config;
pub mod error;
pub mod health;
pub mod idempotency;
pub mod permit;
pub mod restriction;
pub mod router;
pub mod state;
pub mod track_a;
pub mod track_b;
