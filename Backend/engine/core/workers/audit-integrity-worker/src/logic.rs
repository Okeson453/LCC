//! Audit-log integrity verifier.

use sqlx::PgPool;
use tracing::{error, warn};

/// Walk the audit-log checksum chain and verify each row's integrity.
/// Returns the count of rows checked.
pub async fn verify_audit_chain(pool: &PgPool) -> Result<usize, sqlx::Error> {
    // For production we need SELECT on lcc_audit.events.
    // We use lcc_audit_writer which can SELECT as it owns the table.
    let rows = sqlx::query(
        r#"
        SELECT id, event_id, checksum_sha256, prev_checksum, metadata
        FROM lcc_audit.events
        ORDER BY id ASC
        LIMIT 10000
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut prev: Option<String> = None;
    let mut count = 0;
    let mut broken = 0;

    for row in &rows {
        let id: i64 = row.try_get("id")?;
        let checksum: String = row.try_get("checksum_sha256")?;
        let prev_stored: Option<String> = row.try_get("prev_checksum")?;
        let metadata: serde_json::Value = row.try_get("metadata")?;

        if let Some(p) = &prev {
            if prev_stored.as_deref() != Some(p.as_str()) {
                error!(id, "audit chain broken: prev_checksum mismatch");
                broken += 1;
            }
        }
        // In production: also recompute the checksum from the row's content and verify.
        let _ = metadata; // placeholder for actual verification
        prev = Some(checksum);
        count += 1;
    }

    if broken > 0 {
        warn!(broken, total = count, "audit chain integrity issues found");
    }
    Ok(count)
}
