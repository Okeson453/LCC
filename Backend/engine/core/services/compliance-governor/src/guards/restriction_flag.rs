//! Guard 6 — Restriction flag (Source §11 guard 6; axiom 6).
//!
//! `member_account.restricted_since IS NULL`. Any non-null value fails. The
//! flag is set by the Integration Gateway on detection of a 429 / CAPTCHA /
//! verification challenge / restriction banner (Source §39.4).

use async_trait::async_trait;

use super::{Guard, GuardResult};
use crate::state::GovernorDeps;
use crate::{AccountState, CandidateAction};

pub struct RestrictionFlagGuard {
    #[allow(dead_code)]
    deps: GovernorDeps,
}

impl RestrictionFlagGuard {
    pub fn new(deps: GovernorDeps) -> Self {
        Self { deps }
    }
}

#[async_trait]
impl Guard for RestrictionFlagGuard {
    fn name(&self) -> &'static str {
        "restriction_flag"
    }

    async fn check(
        &self,
        action: &CandidateAction,
        account: &AccountState,
        _deps: &GovernorDeps,
    ) -> GuardResult {
        if account.is_restricted {
            let reason = account
                .restricted_reason
                .clone()
                .unwrap_or_else(|| "unknown".into());
            return GuardResult::fail(format!(
                "account_restricted: {reason} (since {:?}); manual review required",
                account.restricted_since
            ));
        }

        // Read-only re-check against the database. The in-memory
        // `account.is_restricted` above is a cache and must not be trusted
        // alone (Technical Design Spec §9.1: "Stale state -> re-check").
        //
        // F-AUDIT-19: this previously ran
        //     SELECT is_restricted FROM member_account WHERE id = $1
        // `member_account` does not exist anywhere in the repo. Restrictions
        // live in `lcc.restrictions` (schemas/migrations/0010_compliance.sql),
        // where a restriction is "active" when `cleared_at IS NULL`. Because
        // the query always errored and the error arm is fail-closed, this
        // guard denied **100% of all actions** — including the two that guard
        // is supposed to be the only obstacle for. The governor could never
        // return Permit, which made the entire permit-issuance path
        // unreachable even once a caller existed.
        //
        // A member with no restriction rows is not restricted, so the correct
        // query is "does an uncleared restriction exist", answered with
        // EXISTS so at most one row crosses the wire.
        let active: Result<(bool,), sqlx::Error> = sqlx::query_as(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM lcc.restrictions
                WHERE member_id = $1
                  AND cleared_at IS NULL
            )
            "#,
        )
        .bind(action.member_id)
        .fetch_one(&_deps.db)
        .await;

        match active {
            Ok((true,)) => {
                // Surface the newest uncleared signal so the denial reason is
                // actionable in the audit trail.
                let kind: Option<(String,)> = sqlx::query_as(
                    "SELECT signal_kind FROM lcc.restrictions
                     WHERE member_id = $1 AND cleared_at IS NULL
                     ORDER BY detected_at DESC LIMIT 1",
                )
                .bind(action.member_id)
                .fetch_optional(&_deps.db)
                .await
                .ok()
                .flatten();
                let signal = kind.map(|(k,)| k).unwrap_or_else(|| "unknown".into());
                GuardResult::fail(format!(
                    "account_restricted: db_flag_set (signal={signal}); manual review required"
                ))
            }
            Ok((false,)) => GuardResult::pass(),
            Err(e) => {
                // Fail-closed by design (axiom 1: any guard failure -> deny).
                tracing::error!(error = %e, "restriction_flag: db lookup failed");
                GuardResult::fail("restriction_flag_check_failed")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Needs a Tokio runtime: the placeholder deps build a sqlx pool, and
    // `connect_lazy` spawns that pool's background task.
    #[tokio::test]
    async fn guard_name() {
        let g = RestrictionFlagGuard::new(GovernorDeps::placeholder());
        assert_eq!(g.name(), "restriction_flag");
    }
}
