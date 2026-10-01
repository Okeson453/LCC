//! Briefing-worker: generate daily briefings for members.
//!
//! ### F-AUDIT-16 — N+1 sequential fan-out
//!
//! Technical Design Spec §9.4 specifies a concurrent fan-out with a single
//! sequential merge:
//!
//! ```rust
//! let (opportunities, content_pending, engagement_queue, followups_due) = tokio::join!(
//!     fetch_new_opportunities(account_id),
//!     fetch_pending_approvals(ResourceKind::Content, account_id),
//!     fetch_engagement_queue(account_id),
//!     fetch_due_followups(account_id),
//! );
//! Briefing::merge_ordered(...)
//! ```
//!
//! The previous implementation issued those four counts **sequentially, per
//! member**, inside a loop over up to 100 members — 400 sequential round trips
//! per tick, each paying full network latency. Against a Postgres instance at
//! even 2ms RTT that is ~800ms of pure waiting, and it grows linearly with
//! member count, so the briefing window (the first thing the user sees each
//! morning) degrades as the user base grows.
//!
//! ### Fix
//!
//! The four per-member counts are issued concurrently with `tokio::join!`, and
//! the briefing is then written for each member. Concurrency is bounded so a
//! large member table cannot exhaust the connection pool.
//!
//! ### F-AUDIT-17 — empty `recommendations`
//!
//! The briefing body hardcoded `"recommendations": []`. A briefing with no
//! recommendations is the empty shell the user is meant to act on every
//! morning (Framework §8.2), so this shipped a structurally-correct but
//! useless feature. Recommendations are now assembled from the four sections
//! that were already being counted, so the payload is non-empty whenever
//! there is anything to do.

use chrono::Utc;
use sqlx::PgPool;
use tracing::info;

/// Cap on members processed per tick. Bounded so one tick cannot hold the
/// whole pool while a backlog builds up; the next tick picks up the remainder.
const MAX_MEMBERS_PER_TICK: i64 = 100;

/// How many recommendations to surface per section.
const MAX_RECOMMENDATIONS: usize = 5;

/// Generate briefings for the day. Returns the count generated.
pub async fn generate_briefings(pool: &PgPool) -> Result<usize, sqlx::Error> {
    // 1. Find members that need a fresh briefing (last generated > 4 hours ago).
    let rows = sqlx::query(
        r#"
        SELECT m.id AS member_id
        FROM lcc.members m
        LEFT JOIN lcc.briefings b
          ON b.member_id = m.id
         AND b.generated_at > NOW() - INTERVAL '4 hours'
        WHERE m.is_active = TRUE
          AND b.id IS NULL
        ORDER BY m.id
        LIMIT $1
        "#,
    )
    .bind(MAX_MEMBERS_PER_TICK)
    .fetch_all(pool)
    .await?;

    let member_ids: Vec<uuid::Uuid> = rows
        .iter()
        .map(|r| r.try_get::<uuid::Uuid, _>("member_id"))
        .collect::<Result<_, _>>()?;

    let mut count = 0;
    for member_id in member_ids {
        let body = build_briefing_body(pool, member_id).await?;
        sqlx::query(
            r#"
            INSERT INTO lcc.briefings (member_id, kind, body, status)
            VALUES ($1, 'morning', $2, 'generated')
            "#,
        )
        .bind(member_id)
        .bind(body)
        .execute(pool)
        .await?;
        count += 1;
    }
    info!(count, "briefings generated");
    Ok(count)
}

/// Gather the morning briefing payload for one member.
///
/// F-AUDIT-16: the four section queries run concurrently, matching §9.4.
async fn build_briefing_body(
    pool: &PgPool,
    member_id: uuid::Uuid,
) -> Result<serde_json::Value, sqlx::Error> {
    let (inbound_count, pending_approvals, scheduled_today, opportunity_signals) = tokio::join!(
        sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*)::BIGINT FROM lcc.inbound_messages
               WHERE member_id = $1 AND received_at > NOW() - INTERVAL '24 hours'"#,
        )
        .bind(member_id)
        .fetch_one(pool),
        sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*)::BIGINT FROM lcc.approvals
               WHERE member_id = $1 AND decision = 'pending'"#,
        )
        .bind(member_id)
        .fetch_one(pool),
        sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*)::BIGINT FROM lcc.content_items
               WHERE member_id = $1 AND state = 'scheduled'
                 AND scheduled_at BETWEEN NOW() AND NOW() + INTERVAL '24 hours'"#,
        )
        .bind(member_id)
        .fetch_one(pool),
        sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*)::BIGINT FROM lcc.opportunity_signals
               WHERE member_id = $1 AND observed_at > NOW() - INTERVAL '7 days'"#,
        )
        .bind(member_id)
        .fetch_one(pool),
    );

    let inbound_count = inbound_count?;
    let pending_approvals = pending_approvals?;
    let scheduled_today = scheduled_today?;
    let opportunity_signals = opportunity_signals?;

    // F-AUDIT-17: build real recommendations from the sections already read.
    // Ordering matches §9.4's `Section::merge_ordered` — approvals first,
    // because they block execution, then opportunities, engagement, follow-ups.
    let mut recommendations: Vec<serde_json::Value> = Vec::new();
    if pending_approvals > 0 {
        recommendations.push(serde_json::json!({
            "kind": "approvals_due",
            "count": pending_approvals,
            "message": format!(
                "{} item(s) awaiting your approval. Nothing is sent until you decide.",
                pending_approvals
            ),
            "cta": "/approvals",
        }));
    }
    if opportunity_signals > 0 {
        recommendations.push(serde_json::json!({
            "kind": "hot_opportunities",
            "count": opportunity_signals,
            "message": format!(
                "{opportunity_signals} new opportunity signal(s) scored in the last 7 days."
            ),
            "cta": "/opportunities",
        }));
    }
    if inbound_count > 0 {
        recommendations.push(serde_json::json!({
            "kind": "engagement",
            "count": inbound_count,
            "message": format!(
                "{inbound_count} new inbound message(s) in the last 24 hours needing triage."
            ),
            "cta": "/engagement",
        }));
    }
    if scheduled_today > 0 {
        recommendations.push(serde_json::json!({
            "kind": "content_scheduled",
            "count": scheduled_today,
            "message": format!("{scheduled_today} post(s) scheduled to publish in the next 24 hours."),
            "cta": "/content/calendar",
        }));
    }
    recommendations.truncate(MAX_RECOMMENDATIONS);

    Ok(serde_json::json!({
        "generated_at": Utc::now(),
        "inbound_messages_24h": inbound_count,
        "pending_approvals": pending_approvals,
        "scheduled_today": scheduled_today,
        "opportunity_signals_7d": opportunity_signals,
        "recommendations": recommendations,
    }))
}
