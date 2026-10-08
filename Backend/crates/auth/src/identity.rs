//! Reusable request-identity and authorization extraction for services.
//!
//! # Why this exists
//!
//! F-AUDIT-59: `Role` was parsed from the JWT by the gateway and by several
//! services, and `has_permission` in `rbac.rs` had a complete, tested policy
//! table — but nothing ever called it. `grep -rn "has_permission" engine/`
//! returned no call sites outside the crate's own unit tests. An `Assistant`
//! token could therefore drive any endpoint an `Owner` could, including
//! approving outbound sends, purely because the role was never compared.
//!
//! Rather than have each service re-implement "read the header, verify the
//! token, parse the subject, parse the role", every service extracts a
//! `Caller` from the request through this module and then checks a
//! `Permission` against it. Keeping the check in one place is what makes the
//! policy table auditable: if a handler calls `caller.require(...)`, the gate
//! exists, and a test can assert on it.
//!
//! # Threat model
//!
//! `member_id` is always taken from the verified `sub` claim, never from a
//! header or path parameter. A caller cannot reach another member's data by
//! supplying a different id, because the id is not attacker-controlled.

use axum::http::HeaderMap;
use lcc_error::LccError;
use uuid::Uuid;

use crate::rbac::{has_permission, Permission, Role};

/// A verified caller: the member identity and role from a validated JWT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caller {
    pub member_id: Uuid,
    pub role: Role,
}

impl Caller {
    /// Returns `Ok(())` when this caller holds `perm`, else `Forbidden`.
    ///
    /// `Forbidden` (403) rather than `Unauthorized` (401) is deliberate and
    /// matches RFC 9110: the request *was* authenticated, the identity just
    /// lacks the right to perform it.
    pub fn require(self, perm: Permission) -> Result<(), LccError> {
        if has_permission(self.role, perm) {
            Ok(())
        } else {
            Err(LccError::Forbidden(format!(
                "role {} lacks permission {:?}",
                self.role.as_ref(),
                perm
            )))
        }
    }

    /// True when the caller holds `perm`. Use for optional enrichment; use
    /// [`Caller::require`] for anything that must be gated.
    pub fn can(self, perm: Permission) -> bool {
        has_permission(self.role, perm)
    }
}

/// Extracts and verifies the caller from request headers.
///
/// Fails closed in every ambiguous case:
/// * missing or non-`Bearer` `Authorization` -> `Unauthorized`
/// * unverifiable / expired token              -> `Unauthorized`
/// * non-UUID `sub`                            -> `Unauthorized`
/// * unrecognised `role`                       -> `Unauthorized`
///
/// The last one matters: an unknown role must never be treated as a
/// least-privilege role that "probably has no permissions", because that
/// pattern invites accidentally widening a default later. Rejecting the token
/// outright means a newly-added role cannot be silently under-privileged
/// across the whole estate.
///
/// `LCC_AUTH_JWT_SECRET` supplies the HS256 secret. It is read per request
/// rather than cached so a rotated secret takes effect without a restart;
/// verification is cheap relative to the database round-trip that follows.
pub fn caller_from_headers(headers: &HeaderMap) -> Result<Caller, LccError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| LccError::Unauthorized("missing bearer token".into()))?;

    let secret = std::env::var("LCC_AUTH_JWT_SECRET")
        .map_err(|_| LccError::Unauthorized("auth secret not configured".into()))?;

    let claims = crate::verify_token(token, &secret)
        .map_err(|e| LccError::Unauthorized(format!("token rejected: {e}")))?;

    let member_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| LccError::Unauthorized("sub is not a uuid".into()))?;
    let role: Role = claims
        .role
        .parse()
        .map_err(|_| LccError::Unauthorized("unknown role in token".into()))?;

    Ok(Caller { member_id, role })
}

/// Returns the caller's member id, enforcing `perm` in the same step.
///
/// This is the shape most handlers want: the member id and the authorization
/// decision come from one verified source, so a handler cannot accidentally
/// use one without the other.
pub fn member_requiring(headers: &HeaderMap, perm: Permission) -> Result<Uuid, LccError> {
    let caller = caller_from_headers(headers)?;
    caller.require(perm)?;
    Ok(caller.member_id)
}

#[cfg(test)]
mod tests {
    // F-AUDIT-51: the workspace denies `clippy::unwrap_used` / `expect_used`
    // because an `unwrap` on a `Result` can take a production service down.
    // In a test binary the opposite holds — panicking is the failure signal.
    // Scoped to this test module so the production lints stay intact.
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    fn headers_with(token: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {token}").parse().unwrap(),
        );
        h
    }

    #[test]
    fn missing_header_is_unauthorized() {
        let h = HeaderMap::new();
        assert!(matches!(
            caller_from_headers(&h),
            Err(LccError::Unauthorized(_))
        ));
    }

    #[test]
    fn non_bearer_scheme_is_unauthorized() {
        let mut h = HeaderMap::new();
        h.insert(
            axum::http::header::AUTHORIZATION,
            "Basic abc123".parse().unwrap(),
        );
        assert!(matches!(
            caller_from_headers(&h),
            Err(LccError::Unauthorized(_))
        ));
    }

    #[test]
    fn garbage_token_is_unauthorized() {
        let h = headers_with("not-a-jwt");
        assert!(matches!(
            caller_from_headers(&h),
            Err(LccError::Unauthorized(_))
        ));
    }

    #[test]
    fn require_returns_forbidden_not_unauthorized() {
        let caller = Caller {
            member_id: Uuid::nil(),
            role: Role::Assistant,
        };
        // Authenticated but lacking the right -> 403 semantics.
        assert!(matches!(
            caller.require(Permission::ApproveSend),
            Err(LccError::Forbidden(_))
        ));
        assert!(caller.require(Permission::DraftContent).is_ok());
    }

    #[test]
    fn every_role_can_view_its_own_data() {
        // Self-service must never be a lockout.
        for role in [
            Role::Owner,
            Role::Assistant,
            Role::Reviewer,
            Role::Admin,
            Role::Auditor,
        ] {
            let caller = Caller {
                member_id: Uuid::nil(),
                role,
            };
            assert!(
                caller.require(Permission::ViewOwnData).is_ok(),
                "{role:?} must be able to read its own data"
            );
        }
    }

    #[test]
    fn only_owner_and_admin_manage_compliance_config() {
        for role in [Role::Owner, Role::Admin] {
            let caller = Caller {
                member_id: Uuid::nil(),
                role,
            };
            assert!(caller.can(Permission::ManageComplianceConfig));
        }
        for role in [Role::Assistant, Role::Reviewer, Role::Auditor] {
            let caller = Caller {
                member_id: Uuid::nil(),
                role,
            };
            assert!(
                !caller.can(Permission::ManageComplianceConfig),
                "{role:?} must not manage compliance config"
            );
        }
    }
}
