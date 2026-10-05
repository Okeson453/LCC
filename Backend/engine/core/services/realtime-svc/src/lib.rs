//! lcc-realtime-svc — Realtime + SSE service.
//!
//! Implements the 5 dashboard channels per
//! `contract_audit/realtime/lcc-realtime-contract.yaml`:
//!   - /api/v1/ws/briefing
//!   - /api/v1/ws/approvals
//!   - /api/v1/ws/engagement
//!   - /api/v1/ws/compliance
//!   - /api/v1/ws/sequence
//!
//! Plus SSE fallback for each at /api/v1/sse/<channel>.
//!
//! Backend domain events arrive via Redis Streams (topic pattern
//! `lcc:realtime:events`). Each event is deserialized, validated against the
//! channel's allowed event names, deduped by `event_id`, then fanned out to
//! the per-connection broadcast channels.

#![deny(warnings)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo)]

pub mod auth;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod events;
pub mod health;
pub mod http;
pub mod sse;
pub mod state;
pub mod ws;
