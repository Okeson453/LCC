//! Opportunity discovery worker — scans contacts for new signals.

use sqlx::PgPool;
use tracing::debug;

/// Discover opportunities from cached contact signals. Returns the count.
pub async fn discover_opportunities(pool: &PgPool) -> Result<usize, sqlx::Error> {
    // 1. Find contacts with recent interactions (job change, post about problem).
    let candidates = sqlx::query(
        r#"
        SELECT c.id AS contact_id, c.member_id, c.display_name, c.company, c.title
        FROM lcc.contacts c
        LEFT JOIN lcc.opportunities o
          ON o.contact_id = c.id AND o.funnel IN ('discovered', 'qualified', 'in_conversation')
        WHERE c.last_contact_at > NOW() - INTERVAL '14 days'
          AND o.id IS NULL
        LIMIT 50
        "#,
    )
    .fetch_all(pool)
    .await?;

    debug!(count = candidates.len(), "candidate contacts");

    let mut count = 0;
    for row in &candidates {
        let contact_id: uuid::Uuid = row.try_get("contact_id")?;
        let member_id: uuid::Uuid = row.try_get("member_id")?;
        let display_name: String = row.try_get("display_name")?;

        // Insert a discovered opportunity.
        sqlx::query(
            r#"
            INSERT INTO lcc.opportunities (member_id, kind, funnel, title, contact_id, phi_score)
            VALUES ($1, 'other', 'discovered', $2, $3, 0.0)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(member_id)
        .bind(format!("Discovery: {display_name}"))
        .bind(contact_id)
        .execute(pool)
        .await?;
        count += 1;
    }
    Ok(count)
}
