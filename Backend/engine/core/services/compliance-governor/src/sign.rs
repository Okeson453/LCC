//! Permit token signing — re-exports `lcc-compliance::permit_token` for use
//! from the Compliance Governor service.

pub use lcc_compliance::permit_token::{PermitClaims, PermitError, PermitSigner, PermitVerifier};
