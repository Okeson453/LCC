//! Data purge worker — removes TTL-expired third-party data.

use sqlx::PgPool;
use tracing::info;

/// Purge TTL-expired rows. Returns the count of rows removed.
pub async fn purge_expired_data(pool: &PgPool) -> Result<usize, sqlx::Error> {
    let mut count = 0;

    // Purge expired opportunity signals.
    let opp_signals = sqlx::query(
        r#"DELETE FROM lcc.opportunity_signals WHERE ttl_expires_at IS NOT NULL AND ttl_expires_at < NOW()"#,
    )
    .execute(pool)
    .await?
    .rows_affected();
    info!(deleted = opp_signals, "expired opportunity_signals purged");
    count += opp_signals as usize;

    // Purge expired profile snapshots (third-party only).
    let profile_snaps = sqlx::query(
        r#"DELETE FROM lcc.profile_snapshots WHERE third_party_ttl_expires_at IS NOT NULL AND third_party_ttl_expires_at < NOW()"#,
    )
    .execute(pool)
    .await?
    .rows_affected();
    info!(deleted = profile_snaps, "expired profile_snapshots purged");
    count += profile_snaps as usize;

    // Purge expired idempotency keys.
    let idem = sqlx::query(r#"DELETE FROM lcc.idempotency_keys WHERE expires_at < NOW()"#)
        .execute(pool)
        .await?
        .rows_affected();
    info!(deleted = idem, "expired idempotency_keys purged");
    count += idem as usize;

    Ok(count)
}
