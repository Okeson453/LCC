//! axum extractor + middleware for authenticated requests.

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use lcc_error::{LccError, LccResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::jwt::{JwtClaims, JwtVerifier};
use crate::rbac::{has_permission, Permission, Role};

/// AuthenticatedUser — extractor injected by the auth middleware.
/// Available on every handler after the middleware runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub member_id: Uuid,
    pub role: Role,
    pub session_id: Uuid,
    pub trace_id: String,
}

impl AuthenticatedUser {
    pub fn has(&self, perm: Permission) -> bool {
        has_permission(self.role, perm)
    }

    pub fn require(&self, perm: Permission) -> Result<(), LccError> {
        if self.has(perm) {
            Ok(())
        } else {
            Err(LccError::Forbidden(format!(
                "{:?} cannot {:?}",
                self.role, perm
            )))
        }
    }
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    Arc<AuthnState>: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_state = Arc::<AuthnState>::from_ref(state);
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let token = match auth_header.and_then(|h| h.strip_prefix("Bearer ")) {
            Some(t) => t,
            None => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    "missing or malformed Authorization header",
                )
                    .into_response());
            }
        };

        let claims = match auth_state.verifier.verify(token) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, "jwt verification failed");
                return Err((StatusCode::UNAUTHORIZED, "invalid token").into_response());
            }
        };

        let member_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "token subject is not a valid member_id",
            )
                .into_response()
        })?;

        let role: Role = claims
            .role
            .parse()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "unknown role").into_response())?;

        let session_id = Uuid::parse_str(&claims.session_id).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "token session_id is not a valid UUID",
            )
                .into_response()
        })?;

        let trace_id = parts
            .headers
            .get("x-trace-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        Ok(Self {
            member_id,
            role,
            session_id,
            trace_id,
        })
    }
}

/// State held by the Authn middleware. Lives in AppState.
#[derive(Clone)]
pub struct AuthnState {
    pub verifier: Arc<JwtVerifier>,
}

impl AuthnState {
    pub fn new(verifier: JwtVerifier) -> Self {
        Self {
            verifier: Arc::new(verifier),
        }
    }
}

/// AuthnLayer — convenience constructor for the auth middleware.
pub struct AuthnLayer;

impl AuthnLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AuthnLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_role_check() {
        let u = AuthenticatedUser {
            member_id: Uuid::now_v7(),
            role: Role::Owner,
            session_id: Uuid::now_v7(),
            trace_id: "trace_1".to_string(),
        };
        assert!(u.has(Permission::ApproveSend));
        assert!(u.require(Permission::ApproveSend).is_ok());

        let a = AuthenticatedUser {
            member_id: Uuid::now_v7(),
            role: Role::Assistant,
            session_id: Uuid::now_v7(),
            trace_id: "trace_1".to_string(),
        };
        assert!(!a.has(Permission::ApproveSend));
        assert!(a.require(Permission::ApproveSend).is_err());
    }
}
