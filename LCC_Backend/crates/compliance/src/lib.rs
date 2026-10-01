//! `lcc-compliance` — Compliance math, types, and permit_token signing.
//!
//! This crate is the source of truth for the formulas referenced by the
//! Compliance Governor service: H_c (§4), AB_d (§4), φ (§6.3), ρ (§5) and the
//! load-time `ComplianceConfig` (§10–§15 of source spec).
//!
//! ### Module map
//! - [`action`] — `ActionType` and `RiskTier` enums (single source of truth).
//! - [`config`] — `ComplianceConfig` struct loaded from versioned YAML.
//! - [`h_c`] — H_c formula.
//! - [`ab_d`] — AB_d formula.
//! - [`phi`] — φ formula.
//! - [`rho`] — ρ formula (no default β coefficients; rule-based fallback).
//! - [`permit_token`] — Issue and verify `permit_token` JWTs.
//!
//! ### Boundary
//! No I/O. This crate never touches the network, the database, or Redis. The
//! `compliance-governor` service is the only thing that *invokes* these
//! formulas against live state. This separation keeps the math unit-testable
//! in isolation.

pub mod ab_d;
pub mod action;
pub mod config;
pub mod h_c;
pub mod permit_token;
pub mod phi;
pub mod rho;

pub use ab_d::ab_d;
pub use action::{ActionType, RiskTier};
pub use config::{
    ActionCaps, ComplianceConfig, ComplianceConfigError, Spacing, DEFAULT_VERSION,
};
pub use h_c::{h_c, H_cComponents, H_cInputs};
pub use permit_token::{
    generate_keypair, public_only_verifier, signing_only_signer, KeyStore, PermitClaims,
    PermitClaimsBuilder, PermitError, PermitSigner, PermitVerifier, StaticKeyStore,
};
// F-AUDIT-18: the previous file ended with
//     #[deprecated(note = "use PermitSigner instead")]
//     pub type PermitSigner = PermitSigner;
//     #[deprecated(note = "use PermitVerifier instead")]
//     pub type PermitVerifier = PermitVerifier;
//     #[deprecated(note = "use PermitError instead")]
//     pub type PermitError = PermitError;
// i.e. three self-referential aliases. Rust rejects a cyclic type alias
// (E0391: "cycle detected when computing type of ..."), so `lcc-compliance`
// — the shared dependency of the compliance-governor AND the
// integration-gateway — could not compile at all. The aliases were also
// pointlessly deprecated-to-themselves.
//
// The three names are already exported by the `pub use permit_token::{...}`
// above, so the correct fix is deletion, not renaming. All downstream
// `lcc_compliance::PermitSigner` / `::PermitVerifier` / `::PermitError`
// references keep resolving.
pub use phi::{phi, PhiComponents, PhiInputs};
pub use rho::{rho, ReplyProbabilityFeatures, RhoMode, RhoResult};
