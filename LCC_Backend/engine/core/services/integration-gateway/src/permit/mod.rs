//! `permit` module — verify the Compliance Governor's `permit_token` before
//! executing ANY action (Technical Design Spec §10, axiom 1).
//!
//! F-AUDIT-21: this file re-exported `verify_permit_token`, a name that is
//! defined nowhere in the repository. `pub use verifier::{verify_permit_token,
//! ...}` is an unresolved import (E0432), so the integration-gateway crate
//! could not compile. The public surface is the `PermitVerifier` type, whose
//! `verify` method is what both Track A and Track B actually call.

pub mod replay;
pub mod verifier;

pub use replay::{ReplayGuard, ReplayGuardError};
pub use verifier::{PermitError, PermitVerifier};
