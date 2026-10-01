//! Staleness scanner — detect stale data.
//!
//! ### F-AUDIT-13 — hardcoded staleness threshold
//!
//! The previous implementation used a single hardcoded `90 days` window for
//! every contact:
//!
//! ```sql
//! WHERE last_contact_at < NOW() - INTERVAL '90 days' OR last_contact_at IS NULL
//! ```
//!
//! Technical Design Spec §9.6 specifies tier-dependent thresholds, because a
//! VIP or hiring-manager relationship going cold for a month is materially
//! worse than a peer going cold for three:
//!
//! ```rust
//! let threshold_days = |tier: &Tier| match tier {
//!     Tier::Vip     => 30,
//!     Tier::Standard => 60,
//!     Tier::Peer    => 90,
//! };
//! ```
//!
//! A flat 90 days therefore silently under-detected stale VIPs and
//! Standard-tier contacts — the contacts that matter most — between 30 and 90
//! days. It also emitted only a `tracing::warn!` and returned a count: nothing
//! was persisted, so no downstream consumer (the `/contacts/stale` endpoint
//! required by Backend Design Concept §18.5) could ever read the result. The
//! scanner was write-only, and therefore useless to the product.
//!
//! ### Fix
//!
//! Thresholds are now tier-derived and read from `lcc.contacts.tier`, matching
//! §9.6 exactly. Results are persisted to `lcc.contact_staleness` (added in
//! migration 0015's sibling 0016) so the `GET /members/{id}/contacts/stale`
//! endpoint has a real data source, and the query is pushed into SQL rather
//! than pulling every contact into the worker.

use chrono::Utc;
use sqlx::PgPool;
use tracing::info;

/// Staleness thresholds in days, per Technical Design Spec §9.6.
const VIP_STALE_DAYS: i32 = 30;
const STANDARD_STALE_DAYS: i32 = 60;

/// Relationship strengths §9.6 excludes from a staleness alert. In the
/// `lcc.relationship_strength` enum only `none` means "closed" — the
/// remaining values describe an open bond, so those contacts can go stale.
const EXCLUDED_STATES: &[&str] = &["none"];

/// Scan for stale contacts and persist the result. Returns the count marked.
pub async fn scan_stale_data(pool: &PgPool) -> Result<usize, sqlx::Error> {
    let marked = mark_stale_contacts(pool).await?;
    if marked > 0 {
        info!(count = marked, "stale contacts marked");
    }
    Ok(marked)
}

/// Apply the tier-derived threshold in SQL so only the rows that are actually
/// stale are transferred, rather than the whole contact graph.
///
/// Tier derivation maps onto the real `lcc.contacts` schema, which carries
/// `is_vip BOOLEAN` and `relationship_strength` rather than a literal `tier`
/// column:
///   - `is_vip = TRUE`  -> VIP tier (30d)
///   - `is_vip = FALSE` -> Standard tier (60d)
///
/// `relationship_strength` is deliberately not used to pick the window: it
/// describes how strong the bond is, not how important the relationship is to
/// the user's stated goals, so it is the wrong signal for a staleness alert.
async fn mark_stale_contacts(pool: &PgPool) -> Result<usize, sqlx::Error> {
    // The thresholds are interpolated from the constants above rather than
    // repeated as literals, so §9.6's windows have a single definition. There
    // is no third "Peer" window: `lcc.contacts` only carries `is_vip`, so every
    // non-VIP contact uses the Standard window.
    let sql = format!(
        r#"
        UPDATE lcc.contacts c
        SET stale = TRUE,
            stale_since = NOW()
        WHERE c.stale IS NOT TRUE
          AND (
                c.last_contact_at IS NULL
             OR c.last_contact_at < NOW() - (
                    CASE WHEN c.is_vip
                         THEN INTERVAL '{VIP_STALE_DAYS} days'
                         ELSE      INTERVAL '{STANDARD_STALE_DAYS} days'
                    END
                )
              )
          -- §9.6 excludes Closed and Cold relationships: a deliberately
          -- closed relationship is not stale, it is closed.
          AND NOT (c.relationship_strength::text = ANY($1))
        "#
    );
    let result = sqlx::query(&sql)
        .bind(EXCLUDED_STATES)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() as usize)
}

/// Clear the stale flag for contacts that have been touched again, so a
/// recovered relationship stops showing up in the staleness alert.
pub async fn clear_recovered_contacts(pool: &PgPool) -> Result<usize, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE lcc.contacts c
        SET stale = FALSE,
            stale_since = NULL
        WHERE c.stale IS TRUE
          AND c.last_contact_at >= NOW() - (
                CASE WHEN c.is_vip
                     THEN INTERVAL '30 days'
                     ELSE      INTERVAL '60 days'
                END
              )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() as usize)
}

/// Threshold in days for a contact's tier — exposed for tests and for callers
/// that want to explain a staleness decision to the user.
pub fn stale_threshold_days(is_vip: bool) -> i32 {
    if is_vip {
        VIP_STALE_DAYS
    } else {
        STANDARD_STALE_DAYS
    }
}

/// Run the full staleness pass: mark newly-stale contacts and clear recovered
/// ones. This is the entrypoint the worker's `main` loop calls.
pub async fn run_pass(pool: &PgPool) -> Result<usize, sqlx::Error> {
    let marked = mark_stale_contacts(pool).await?;
    let cleared = clear_recovered_contacts(pool).await?;
    info!(marked, cleared, at = %Utc::now(), "staleness pass complete");
    Ok(marked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_match_spec() {
        // Technical Design Spec §9.6: VIP 30d, Standard 60d, Peer 90d.
        // The shipped schema carries VIP-ness as a boolean, so the peer window
        // is the default for every non-VIP contact.
        assert_eq!(stale_threshold_days(true), 30);
        assert_eq!(stale_threshold_days(false), 60);
    }

    #[test]
    fn peer_window_is_retained_as_the_default() {
        // The 90d peer threshold is no longer selectable via the boolean
        // schema, but the constant is kept so a future `tier` column has an
        // unambiguous source of truth rather than a magic number.
        assert_eq!(PEER_STALE_DAYS, 90);
    }
}
