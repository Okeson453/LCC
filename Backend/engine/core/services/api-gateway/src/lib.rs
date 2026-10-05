//! lcc-api-gateway — single public-facing HTTP entrypoint.

#![deny(warnings)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo)]

pub mod config;
pub mod error;
pub mod handlers;
pub mod health;
pub mod http;
pub mod middleware;
pub mod proxy;
pub mod routes;
pub mod state;
pub mod telemetry;
