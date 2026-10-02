//! lcc-orchestrator — Orchestrator / Briefing Service.

#![deny(warnings)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo)]

pub mod briefing_kind_enum;
pub mod case;
pub mod config;
pub mod domain;
pub mod error;
pub mod events_publisher;
pub mod health;
pub mod http;
pub mod http_handlers;
pub mod repository;
pub mod service;
pub mod state;
