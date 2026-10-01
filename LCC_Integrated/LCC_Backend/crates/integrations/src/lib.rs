//! `lcc-integrations` — LinkedIn API clients (Track A + Track B).
//!
//! ★ WHITELISTED TO integration-gateway ONLY.
//!
//! Per Non-Negotiable §1 (Source) / §15 (Backend Design Concept):
//! - No service other than the Integration Layer may call LinkedIn-facing endpoints.
//! - No service other than the Integration Layer may hold LinkedIn credentials.
//!
//! ### Boundary enforcement
//! 1. Cargo workspace lints + a custom lint deny `crates/integrations` from
//!    being added as a dependency anywhere except `engine/core/services/integration-gateway`.
//! 2. CI grep: `tools/scripts/check_boundaries.sh` scans the repo for imports
//!    of `lcc-integrations` and fails the build if any are outside the gateway.
//! 3. NetworkPolicy per service in `infra/k8s/base/*/networkpolicy.yaml` only
//!    permits egress to `linkedin.com` from the integration-gateway pod.

#![deny(unused)]
#![deny(unsafe_code)]

pub mod limiter;
pub mod track_a;
pub mod track_b;

pub use limiter::{EndpointLimit, RateLimiter};
pub use track_a::{jobs_client, oauth_client, profile_client, share_client, TrackAError};
pub use track_b::{BrowserExtensionMessage, TrackBError};
