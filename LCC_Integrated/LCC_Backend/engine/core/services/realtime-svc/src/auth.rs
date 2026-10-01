//! Auth helpers — verifies the Bearer JWT using the same secret the
//! api-gateway and identity-svc use.

use crate::error::Error;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
    pub role: String,
}

pub fn verify(token: &str, secret: &str, issuer: &str, audience: &str) -> Result<JwtClaims, Error> {
    let mut validation = Validation::default();
    validation.set_audience(&[audience]);
    validation.set_issuer(&[issuer]);
    let data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| Error::Unauthorized(format!("invalid token: {e}")))?;
    Ok(data.claims)
}

/// Extract a Bearer token from an `Authorization` header value.
pub fn extract_bearer(header_value: Option<&axum::http::HeaderValue>) -> Result<String, Error> {
    let raw = header_value
        .ok_or_else(|| Error::Unauthorized("missing Authorization header".into()))?
        .to_str()
        .map_err(|_| Error::Unauthorized("invalid Authorization header".into()))?;
    let token = raw
        .strip_prefix("Bearer ")
        .ok_or_else(|| Error::Unauthorized("Authorization must be Bearer <jwt>".into()))?
        .trim();
    if token.is_empty() {
        return Err(Error::Unauthorized("empty bearer token".into()));
    }
    Ok(token.to_string())
}

impl JwtClaims {
    pub fn sub_uuid(&self) -> Result<uuid::Uuid, Error> {
        uuid::Uuid::parse_str(&self.sub).map_err(|_| Error::Unauthorized("invalid sub".into()))
    }
    pub fn role_is(&self, role: &str) -> bool {
        self.role.eq_ignore_ascii_case(role)
    }
}
