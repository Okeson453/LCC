//! lcc-kb-svc — Knowledge Base records.

#![deny(warnings)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo)]

pub mod config;
pub mod domain;
pub mod error;
pub mod http;
pub mod repository;
pub mod service;
pub mod state;
