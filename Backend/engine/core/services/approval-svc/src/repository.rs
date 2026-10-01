//! Approval repository.

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Approval, ApprovalStatus};
use crate::error::Error;

#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        member_id: Uuid,
        status: Option<ApprovalStatus>,
        limit: i64,
    ) -> Result<Vec<Approval>, Error> {
        let rows: Vec<(
            Uuid, Uuid, String, Uuid, JsonValue, i16,
            Option<String>, String, Uuid, Option<String>, Vec<Uuid>,
            Option<DateTime<Utc>>, i32, DateTime<Utc>, Option<DateTime<Utc>>,
        )> = if let Some(s) = status {
            sqlx::query_as(
                r#"
                SELECT id, member_id, resource_type, resource_id, requested_action,
                       tier::INT, rule_version, decision::TEXT, requested_by,
                       decided_reason, reviewer_ids, expires_at, version,
                       created_at, decided_at
                FROM lcc.approvals
                WHERE member_id = $1 AND decision = $2::text
                ORDER BY created_at DESC
                LIMIT $3
                "#,
            )
            .bind(member_id)
            .bind(s.as_str())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT id, member_id, resource_type, resource_id, requested_action,
                       tier::INT, rule_version, decision::TEXT, requested_by,
                       decided_reason, reviewer_ids, expires_at, version,
                       created_at, decided_at
                FROM lcc.approvals
                WHERE member_id = $1
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(member_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };
        rows.into_iter().map(map_row).collect()
    }

    pub async fn get(&self, member_id: Uuid, id: Uuid) -> Result<Approval, Error> {
        let row = sqlx::query_as::<_, (
            Uuid, Uuid, String, Uuid, JsonValue, i16,
            Option<String>, String, Uuid, Option<String>, Vec<Uuid>,
            Option<DateTime<Utc>>, i32, DateTime<Utc>, Option<DateTime<Utc>>,
        )>(
            r#"
            SELECT id, member_id, resource_type, resource_id, requested_action,
                   tier::INT, rule_version, decision::TEXT, requested_by,
                   decided_reason, reviewer_ids, expires_at, version,
                   created_at, decided_at
            FROM lcc.approvals
            WHERE member_id = $1 AND id = $2
            "#,
        )
        .bind(member_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("approval {id}")),
            other => Error::Internal(format!("get approval: {other}")),
        })?;
        map_row(row)
    }

    pub async fn insert(&self, a: &Approval) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO lcc.approvals
                (id, member_id, resource_type, resource_id, requested_action,
                 tier, rule_version, decision, requested_by,
                 decided_reason, reviewer_ids, expires_at, version,
                 created_at, decided_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8::text,$9,$10,$11,$12,$13,$14,$15)
            "#,
        )
        .bind(a.id)
        .bind(a.member_id)
        .bind(&a.resource_type)
        .bind(a.resource_id)
        .bind(&a.requested_action)
        .bind(a.tier)
        .bind(&a.rule_version)
        .bind(a.status.as_str())
        .bind(a.requested_by)
        .bind(&a.decided_reason)
        .bind(&a.reviewer_ids)
        .bind(a.expires_at)
        .bind(a.version)
        .bind(a.created_at)
        .bind(a.decided_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn decide(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        new_status: ApprovalStatus,
        reason: Option<&str>,
        reviewer_id: Uuid,
    ) -> Result<i32, Error> {
        // Atomic update with version check.
        let mut tx = self.pool.begin().await?;
        let row: Option<(i32, String, i32, Vec<Uuid>)> = sqlx::query_as(
            r#"
            UPDATE lcc.approvals
            SET decision = $4::text,
                decided_at = NOW(),
                decided_reason = $5,
                reviewer_ids = CASE
                  WHEN $6::uuid = ANY(reviewer_ids) THEN reviewer_ids
                  ELSE array_append(reviewer_ids, $6::uuid)
                END,
                version = version + 1
            WHERE member_id = $1 AND id = $2 AND version = $3
            RETURNING version, decision::TEXT, tier::INT, reviewer_ids
            "#,
        )
        .bind(member_id)
        .bind(id)
        .bind(expected_version)
        .bind(new_status.as_str())
        .bind(reason)
        .bind(reviewer_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| Error::Internal(format!("decide approval: {e}")))?;

        let Some(row) = row else {
            tx.rollback().await.ok();
            return Err(Error::Conflict(format!(
                "approval {id} v{expected_version} mismatch"
            )));
        };
        let new_version = row.0;
        let current_decision = row.1;
        let tier = row.2;
        // The UPDATE above appends `reviewer_id` if it was not already present,
        // so this array is the set of distinct reviewers who have now signed.
        let reviewer_count = row.3.len();

        // Two-reviewer gate: tier>=3 approvals require two distinct reviewers
        // before they can become "approved".
        if new_status == ApprovalStatus::Approved && tier >= 3 && reviewer_count < 2 {
            tx.rollback().await.ok();
            return Err(Error::TwoReviewerGateNotSatisfied);
        }
        let _ = current_decision;
        tx.commit().await?;
        Ok(new_version)
    }

    pub async fn bulk_decide(
        &self,
        member_id: Uuid,
        ids: &[Uuid],
        new_status: ApprovalStatus,
        reason: &str,
        reviewer_id: Uuid,
        expected_version: i32,
    ) -> Result<Vec<(Uuid, bool)>, Error> {
        // Each id gets its own transaction so that one failure does not
        // poison the rest of the batch.
        let mut results = Vec::with_capacity(ids.len());
        for id in ids {
            match self
                .decide(member_id, *id, expected_version, new_status, Some(reason), reviewer_id)
                .await
            {
                Ok(_) => results.push((*id, true)),
                Err(_) => results.push((*id, false)),
            }
        }
        Ok(results)
    }
}

fn map_row(
    row: (
        Uuid, Uuid, String, Uuid, JsonValue, i16,
        Option<String>, String, Uuid, Option<String>, Vec<Uuid>,
        Option<DateTime<Utc>>, i32, DateTime<Utc>, Option<DateTime<Utc>>,
    ),
) -> Result<Approval, Error> {
    let status = serde_json::from_value::<ApprovalStatus>(serde_json::Value::String(row.7.clone()))
        .map_err(|e| Error::Internal(format!("decision parse: {e}")))?;
    Ok(Approval {
        id: row.0, member_id: row.1, resource_type: row.2, resource_id: row.3,
        requested_action: row.4, tier: row.5, rule_version: row.6,
        status, requested_by: row.8, decided_reason: row.9,
        reviewer_ids: row.10, expires_at: row.11, version: row.12,
        created_at: row.13, decided_at: row.14,
    })
}
