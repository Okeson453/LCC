//! lcc-identity-svc — Identity Service.

#![deny(warnings)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo)]

pub mod config;
pub mod domain;
pub mod error;
pub mod events;
pub mod health;
pub mod http;
pub mod repository;
pub mod service;
pub mod state;
pub mod telemetry;
