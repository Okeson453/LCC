//! `lcc-test-utils` — test harnesses for Rust Core services.
//!
//! - `postgres`: ephemeral Postgres via testcontainers or sqlx's "test" feature.
//! - `redis`: ephemeral Redis instance.
//! - `mock_linkedin`: wiremock-backed fake of the LinkedIn API surface.
//! - `governor`: helpers for evaluate_action test scenarios.

pub mod governor;
pub mod mock_linkedin;
pub mod postgres;
pub mod redis;
