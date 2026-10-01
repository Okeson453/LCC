//! `lcc-auth` — OAuth2/PKCE helpers, JWT validation, RBAC enforcement.
//!
//! ### Boundary
//! Used by api-gateway and any other service that authenticates requests.
//! LinkedIn OAuth handshake (state + PKCE) lives in `identity-svc`; this crate
//! provides the helpers consumed by that service.

pub mod jwt;
pub mod middleware;
pub mod pkce;
pub mod rbac;

pub use jwt::{JwtClaims, JwtError, JwtIssuer, JwtVerifier};
pub use middleware::{AuthenticatedUser, AuthnLayer};
pub use pkce::{PkcePair, generate_code_verifier, derive_code_challenge};
pub use rbac::{has_permission, Permission, Role};

pub use lcc_error::{LccError, LccResult};

/// Default canonical issuer for HS256 tokens minted by `identity-svc`.
pub const DEFAULT_JWT_ISSUER: &str = "lcc.api-gateway";

/// Default canonical audience — every service expects this in the `aud` claim.
pub const DEFAULT_JWT_AUDIENCE: &str = "lcc-api";

/// Convenience helper: build a `JwtVerifier` from a raw HS256 secret using
/// the canonical issuer/audience, then verify a token.
///
/// Most hot paths should keep a `JwtVerifier` in `AppState`; this helper is
/// for one-off callers (worker jobs, internal callbacks) that read the secret
/// from env on every request.
pub fn verify_token(token: &str, secret: &str) -> Result<JwtClaims, JwtError> {
    let v = JwtVerifier::new(
        secret.as_bytes(),
        DEFAULT_JWT_ISSUER,
        DEFAULT_JWT_AUDIENCE,
    );
    v.verify(token)
}
