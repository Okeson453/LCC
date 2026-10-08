//! lcc-kb-svc — Knowledge Base records.

#![deny(warnings)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo)]

pub mod config;
pub mod domain;
pub mod error;
// F-AUDIT-56: kb-svc is the one gateway upstream with no Deployment manifest
// (the gateway binds prefix "kb" -> kb-svc), and it also had no health module
// at all. Declared here so its router can serve /healthz and /readyz, which
// the manifest added alongside it probes.
pub mod health;
pub mod http;
pub mod repository;
pub mod service;
pub mod state;
