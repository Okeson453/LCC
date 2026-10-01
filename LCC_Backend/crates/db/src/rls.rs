//! RLS context setter.
//!
//! Source: Backend Design Concept §43, §46; Non-Negotiable §14.
//!
//! Per request, the API Gateway validates JWT claims and calls
//! [`set_member_context`] to set the Postgres session variable
//! `app.current_member_id`. The RLS policies on every tenant-scoped table
//! filter rows based on this setting — so cross-account queries return empty
//! automatically at the database layer.
//!
//! ## Usage
//! ```rust,ignore
//! use lcc_db::rls::{set_member_context, RlsContext};
//!
//! let mut tx = pool.begin().await?;
//! set_member_context(&mut *tx, &RlsContext::for_member("m_001")).await?;
//! // any query on a tenant-scoped table now sees only `m_001` rows
//! ```
//!
//! ## Why per-request
//! The pg connection pool returns arbitrary connections. Without explicit
//! `SET LOCAL app.current_member_id = ...` per transaction, the connection's
//! session var from a previous request could leak into the next. Always use
//! `tx`-scoped session vars via [`sqlx::Transaction`].

use serde::{Deserialize, Serialize};
use sqlx::Executor;
use sqlx::PgExecutor;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RlsContext {
    pub member_id: Uuid,
    pub role: String,
    pub request_id: String,
    pub trace_id: String,
}

impl RlsContext {
    /// Build a new RLS context for an authenticated member.
    pub fn new(
        member_id: Uuid,
        role: impl Into<String>,
        request_id: impl Into<String>,
        trace_id: impl Into<String>,
    ) -> Self {
        Self {
            member_id,
            role: role.into(),
            request_id: request_id.into(),
            trace_id: trace_id.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum RlsError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("invalid member_id (not a UUID): {0}")]
    InvalidMemberId(String),
}

/// Set the RLS context for the current transaction/connection.
///
/// Sets the Postgres session variables:
/// - `app.current_member_id` (used by RLS policies)
/// - `app.current_role` (used by RBAC policies)
/// - `app.current_request_id` (for tracing)
/// - `app.current_trace_id` (for log correlation)
///
/// Uses `SET LOCAL` so the setting is bound to the current transaction.
/// MUST be called at the start of every transaction that touches tenant-scoped
/// tables — the API Gateway calls this for every authenticated request.
pub async fn set_member_context<'e, E>(executor: E, ctx: &RlsContext) -> Result<(), RlsError>
where
    E: PgExecutor<'e>,
{
    // Defense-in-depth: validate member_id is a valid UUID before SETting it.
    let member_id_str = ctx.member_id.to_string();

    // SET LOCAL is transaction-scoped — releases on COMMIT/ROLLBACK.
    // We use parameterized queries where possible; some session vars need
    // string concatenation because they aren't parameterizable.
    let stmt = format!(
        "SET LOCAL app.current_member_id = '{member_id_str}';\n\
         SET LOCAL app.current_role = '{}';\n\
         SET LOCAL app.current_request_id = '{}';\n\
         SET LOCAL app.current_trace_id = '{}';",
        escape_sql_string(&ctx.role),
        escape_sql_string(&ctx.request_id),
        escape_sql_string(&ctx.trace_id),
    );

    // sqlx's executor requires us to run the multi-statement via execute_many.
    executor.execute(stmt.as_str()).await?;
    Ok(())
}

/// Set the role context for an admin/auditor session (no member_id).
pub async fn set_role_context<'e, E>(
    executor: E,
    role: &str,
    request_id: &str,
    trace_id: &str,
) -> Result<(), RlsError>
where
    E: PgExecutor<'e>,
{
    let stmt = format!(
        "SET LOCAL app.current_member_id = '';\n\
         SET LOCAL app.current_role = '{}';\n\
         SET LOCAL app.current_request_id = '{}';\n\
         SET LOCAL app.current_trace_id = '{}';",
        escape_sql_string(role),
        escape_sql_string(request_id),
        escape_sql_string(trace_id),
    );
    executor.execute(stmt.as_str()).await?;
    Ok(())
}

/// Escape single quotes for safe SET LOCAL statements. Anything more exotic
/// is rejected at the API Gateway boundary.
fn escape_sql_string(s: &str) -> String {
    s.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_handles_quotes() {
        assert_eq!(escape_sql_string("hello"), "hello");
        assert_eq!(escape_sql_string("O'Brien"), "O''Brien");
        assert_eq!(escape_sql_string("'"), "''");
    }

    #[test]
    fn rls_context_constructs() {
        let m = Uuid::now_v7();
        let ctx = RlsContext::new(m, "owner", "req_1", "trace_1");
        assert_eq!(ctx.member_id, m);
        assert_eq!(ctx.role, "owner");
    }
}
