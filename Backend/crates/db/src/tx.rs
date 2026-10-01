//! Transaction wrapper that writes the audit_log row in the same transaction
//! as the entity write (Source Non-Negotiable §10, Backend Design Concept §47).
//!
//! The pattern is:
//! ```text
//! BEGIN
//!   -- audit_log row written FIRST (write-ahead)
//!   INSERT INTO audit_log (actor, action, resource_type, resource_id,
//!                          before_state, after_state, outcome, reason,
//!                          checksum_sha256)
//!     VALUES (...);
//!   -- entity update follows
//!   UPDATE <table> SET ... WHERE id = $1 AND version = $2;
//! COMMIT
//! ```
//!
//! Recovery: if the entity update fails, the audit row is rolled back too. If
//! the entity update succeeds but the commit fails, both are rolled back.
//! Either way, the system never has an audit row without the matching entity
//! transition (Source §17).

use chrono::Utc;
use lcc_audit_client::{AuditClient, AuditEvent, AuditOutcome};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuditTxError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("audit serialization error: {0}")]
    AuditSerialization(#[from] serde_json::Error),
    #[error("audit error: {0}")]
    Audit(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditTxResult {
    pub audit_id: i64,
    pub checksum: String,
}

/// Wraps the audit-in-transaction pattern.
///
/// The caller invokes [`AuditTx::begin`] to start a transaction, then runs
/// the audit INSERT and the entity UPDATE through the same `&mut Transaction`.
/// On commit, both are persisted atomically; on rollback, both are discarded.
pub struct AuditTx<'a> {
    tx: Transaction<'a, Postgres>,
    actor: String,
    action: String,
    /// F-AUDIT-44: the previous INSERT omitted `member_id` entirely, so
    /// `lcc_audit.events.member_id` was always NULL and per-member audit
    /// queries (the primary access pattern for the audit trail) had to scan
    /// the whole table. Also required to keep audit rows attributable.
    member_id: Option<Uuid>,
}

impl<'a> AuditTx<'a> {
    /// Read the checksum of the most recent audit row, so this one can be
    /// chained onto it.
    ///
    /// The `audit-integrity-worker` verifies the chain by comparing each row's
    /// `prev_checksum` against the previous row's `checksum_sha256`
    /// (`audit-integrity-worker/src/logic.rs`). `prev_checksum` is nullable in
    /// the schema but was never populated, so the worker would have flagged
    /// every row from the second onward as tampered.
    async fn prev_checksum(&mut self) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT checksum_sha256 FROM lcc_audit.events ORDER BY id DESC LIMIT 1",
        )
        .fetch_optional(&mut *self.tx)
        .await
    }
}

impl<'a> AuditTx<'a> {
    pub async fn begin(pool: &'a PgPool, actor: impl Into<String>, action: impl Into<String>) -> Result<Self, AuditTxError> {
        Ok(Self::begin_for_member(pool, actor, action, None).await?)
    }

    /// Begin an audit transaction attributed to a specific member.
    pub async fn begin_for_member(
        pool: &'a PgPool,
        actor: impl Into<String>,
        action: impl Into<String>,
        member_id: Option<Uuid>,
    ) -> Result<Self, AuditTxError> {
        let tx = pool.begin().await?;
        Ok(Self {
            tx,
            actor: actor.into(),
            action: action.into(),
            member_id,
        })
    }

    pub fn tx_mut(&mut self) -> &mut Transaction<'a, Postgres> {
        &mut self.tx
    }

    /// Write the audit_log row. Computes SHA-256 over
    /// `(entity_id, prior_state, new_state, actor, timestamp)`.
    pub async fn record(
        &mut self,
        resource_type: &str,
        resource_id: Uuid,
        before_state: Option<&serde_json::Value>,
        after_state: Option<&serde_json::Value>,
        outcome: AuditOutcome,
        reason: Option<&str>,
    ) -> Result<AuditTxResult, AuditTxError> {
        let before_json = before_state
            .map(|v| serde_json::to_string(v))
            .transpose()?
            .unwrap_or_else(|| "null".to_string());
        let after_json = after_state
            .map(|v| serde_json::to_string(v))
            .transpose()?
            .unwrap_or_else(|| "null".to_string());

        let ts = Utc::now();
        let checksum = compute_checksum(
            resource_id,
            &before_json,
            &after_json,
            &self.actor,
            ts.timestamp_nanos_opt().unwrap_or(0),
        );

        let outcome_str = match outcome {
            AuditOutcome::Success => "success",
            AuditOutcome::Denied => "denied",
            AuditOutcome::Failed => "failed",
        };

        // Fetch the previous checksum first: `prev_checksum()` borrows the
        // transaction, so it must be awaited before the INSERT re-borrows it.
        let prev_checksum = self.prev_checksum().await?;

        let row: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO lcc_audit.events (
                event_id, actor, action, resource_type, resource_id,
                member_id, outcome, reason, metadata,
                checksum_sha256, prev_checksum, occurred_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10, $11, $12)
            RETURNING id
            "#,
        )
        .bind(Uuid::now_v7())          // event_id  (NOT NULL UNIQUE)
        .bind(&self.actor)
        .bind(&self.action)
        .bind(resource_type)
        .bind(resource_id.to_string())  // resource_id is TEXT in lcc_audit.events
        .bind(self.member_id)           // member_id, for per-member RLS/queries
        .bind(outcome_str)
        .bind(reason)
        // before/after state live in `metadata`; the audit table has no
        // dedicated before_state/after_state columns.
        .bind(serde_json::json!({
            "before": before_state,
            "after": after_state,
        }))
        .bind(&checksum)
        .bind(prev_checksum)     // chain link for audit-integrity-worker
        .bind(ts)
        .fetch_one(&mut *self.tx)
        .await?;

        Ok(AuditTxResult {
            audit_id: row.0,
            checksum,
        })
    }

    pub async fn commit(self) -> Result<(), AuditTxError> {
        self.tx.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), AuditTxError> {
        self.tx.rollback().await?;
        Ok(())
    }
}

/// Convenience: full audit-in-tx wrapper for an idempotent record flow.
///
/// Use when you don't need to interleave other writes — just record an audit
/// row from an AuditClient outside a transaction.
pub async fn audit_tx<F, Fut, T>(
    pool: &PgPool,
    actor: &str,
    action: &str,
    record: F,
) -> Result<T, AuditTxError>
where
    F: for<'b> FnOnce(
        &'b mut AuditTx<'_>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, AuditTxError>> + Send + 'b>>,
{
    let mut atx = AuditTx::begin(pool, actor, action).await?;
    let result = record(&mut atx).await?;
    atx.commit().await?;
    Ok(result)
}

/// Compute the SHA-256 checksum mandated by Source §17 and Backend Design Concept §47.
pub fn compute_checksum(
    entity_id: Uuid,
    prior_state: &str,
    new_state: &str,
    actor: &str,
    timestamp_nanos: i64,
) -> String {
    let mut h = Sha256::new();
    h.update(entity_id.as_bytes());
    h.update(b"|");
    h.update(prior_state.as_bytes());
    h.update(b"|");
    h.update(new_state.as_bytes());
    h.update(b"|");
    h.update(actor.as_bytes());
    h.update(b"|");
    h.update(timestamp_nanos.to_be_bytes());
    hex::encode(h.finalize())
}

// Re-export AuditClient & types for convenience.
pub use lcc_audit_client::{AuditClient as AuditClientReExport, AuditEvent as AuditEventReExport};
#[allow(unused_imports)]
use AuditClient as _;
#[allow(unused_imports)]
use AuditEvent as _;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_is_deterministic() {
        let id = Uuid::nil();
        let c1 = compute_checksum(id, "{}", "{}", "system:audit-svc", 1_000_000);
        let c2 = compute_checksum(id, "{}", "{}", "system:audit-svc", 1_000_000);
        assert_eq!(c1, c2);
    }

    #[test]
    fn checksum_changes_with_actor() {
        let id = Uuid::nil();
        let c1 = compute_checksum(id, "{}", "{}", "system:svc-a", 1);
        let c2 = compute_checksum(id, "{}", "{}", "system:svc-b", 1);
        assert_ne!(c1, c2);
    }

    #[test]
    fn checksum_changes_with_timestamp() {
        let id = Uuid::nil();
        let c1 = compute_checksum(id, "{}", "{}", "actor", 1);
        let c2 = compute_checksum(id, "{}", "{}", "actor", 2);
        assert_ne!(c1, c2);
    }

    #[test]
    fn checksum_is_hex_64_chars() {
        let id = Uuid::nil();
        let c = compute_checksum(id, "{}", "{}", "actor", 0);
        assert_eq!(c.len(), 64);
        assert!(c.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
