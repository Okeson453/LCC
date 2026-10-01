//! Optimistic concurrency helper.
//!
//! Source: Backend Design Concept §52.
//!
//! Every mutable entity (`content_item`, `sequence`, `opportunity`, etc.) has
//! an integer `version` column. Updates use:
//! ```sql
//! UPDATE <table> SET ... , version = version + 1
//! WHERE id = $1 AND version = $expected
//! ```
//!
//! If rowcount = 0, the update is rejected as a conflict and the caller must
//! re-fetch + retry. This is enforced via [`optimistic_update`], not by
//! convention.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OptimisticUpdateError {
    #[error("optimistic concurrency conflict: expected version {expected}, current version was different")]
    Conflict { expected: i32 },
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptimisticUpdateResult {
    pub new_version: i32,
}

/// Run an optimistic UPDATE. Returns Ok(OptimisticUpdateResult{new_version}) on
/// success, Err(Conflict) if no row matched the version, Err(Database) on
/// other sqlx errors.
///
/// Caller is responsible for the SQL — pass an UPDATE statement that already
/// includes `version = version + 1` in the SET clause and `WHERE id = $1 AND
/// version = $2` in the WHERE clause.
pub async fn optimistic_update(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    sql: &str,
    id: uuid::Uuid,
    expected_version: i32,
) -> Result<OptimisticUpdateResult, OptimisticUpdateError> {
    let result = sqlx::query(sql)
        .bind(id)
        .bind(expected_version)
        .execute(&mut **tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(OptimisticUpdateError::Conflict {
            expected: expected_version,
        });
    }

    // New version is expected + 1 by construction.
    Ok(OptimisticUpdateResult {
        new_version: expected_version + 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_equality() {
        let r1 = OptimisticUpdateResult { new_version: 2 };
        let r2 = OptimisticUpdateResult { new_version: 2 };
        assert_eq!(r1, r2);
    }
}
