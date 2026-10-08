//! Orchestrator repository — reads pre-scored data from Postgres and
//! computes the briefing payload.
//!
//! The DAG-merge per Backend Design Concept §9.4 runs sequentially
//! after four parallel fetches (now done with `tokio::join!` instead of
//! the prior 4-serial implementation). All queries are read-only.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::lcc_approval::ApprovalSummary;
use crate::domain::lcc_engagement::TaskSummary;
use crate::domain::lcc_opportunity::OpportunitySummary;
use crate::domain::lcc_outreach::SequenceSummary;
use crate::domain::{Briefing, BriefingKind, BriefingSections};
use crate::error::Error;

#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}

/// An active sequence with a step due now. Column order must match the SELECT
/// below.
type ActiveSequenceRow = (
    Uuid,                  // sequence_id
    Uuid,                  // contact_id
    i32,                   // current_step
    Option<DateTime<Utc>>, // last_step_sent_at
    String,                // status
);

/// A pending approval as the briefing query returns it. Column order must match
/// the SELECT below.
type PendingApprovalRow = (
    Uuid,                  // id
    String,                // resource_type
    String,                // requested_action
    i16,                   // tier
    DateTime<Utc>,         // created_at
    Option<DateTime<Utc>>, // expires_at
);

/// A hot opportunity, with the company name joined in (nullable).
type HotOpportunityRow = (
    Uuid,           // opportunity_id
    Option<String>, // company name
    Option<String>, // position
    Option<f64>,    // fit_score
    String,         // status
    DateTime<Utc>,  // discovered_at
);

/// A queued or drafted reply awaiting action.
type QueuedReplyRow = (
    Uuid,                  // id
    String,                // action_type
    Option<f64>,           // priority_score
    Option<DateTime<Utc>>, // due_at
);

impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Fetch the pending approvals queue (read-only — actual mutation
    /// happens via `approval-svc`).
    pub async fn pending_approvals(
        &self,
        member_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ApprovalSummary>, Error> {
        let rows: Vec<PendingApprovalRow> = sqlx::query_as(
            r#"
                SELECT id, resource_type, requested_action->>'action_type',
                       tier::INT, created_at, expires_at
                FROM lcc.approvals
                WHERE member_id = $1 AND decision = 'pending'
                ORDER BY created_at ASC
                LIMIT $2
                "#,
        )
        .bind(member_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Internal(format!("approvals query: {e}")))?;

        Ok(rows
            .into_iter()
            .map(
                |(id, resource_type, action_type, tier, created_at, expires_at)| ApprovalSummary {
                    id,
                    resource_type,
                    action_type,
                    tier,
                    created_at,
                    expires_at,
                },
            )
            .collect())
    }

    /// Fetch hot opportunities (φ ≥ 0.85).
    pub async fn hot_opportunities(
        &self,
        member_id: Uuid,
        limit: i64,
    ) -> Result<Vec<OpportunitySummary>, Error> {
        let rows: Vec<HotOpportunityRow> = sqlx::query_as(
            r#"
            SELECT o.id, c.name,
                   -- The `position` is the opportunity's title. `lcc.opportunities.kind`
                   -- is the CATEGORY enum (job_posting, referral, ...) and must never
                   -- be used here: it answers "what sort of opportunity is this",
                   -- not "what is the role". `title` is already TEXT.
                   o.title AS position,
                   o.fit_score, o.status::TEXT AS status, o.discovered_at
                FROM lcc.opportunities o
                LEFT JOIN lcc.companies c ON c.id = o.company_id
                WHERE o.member_id = $1
                  AND o.status IN ('qualified', 'contacted', 'conversation')
                  AND o.fit_score >= 0.85
                ORDER BY o.fit_score DESC NULLS LAST, o.discovered_at DESC
                LIMIT $2
                "#,
        )
        .bind(member_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Internal(format!("opportunities query: {e}")))?;

        Ok(rows
            .into_iter()
            .map(
                |(id, company_name, position, fit_score, status, discovered_at)| {
                    OpportunitySummary {
                        id,
                        company_name,
                        position,
                        fit_score,
                        status,
                        discovered_at,
                    }
                },
            )
            .collect())
    }

    /// Fetch the engagement queue (overdue replies, planned comments).
    pub async fn engagement_queue(
        &self,
        member_id: Uuid,
        limit: i64,
    ) -> Result<Vec<TaskSummary>, Error> {
        let rows: Vec<QueuedReplyRow> = sqlx::query_as(
            r#"
            SELECT id, action_type, priority_score, due_at
            FROM lcc.engagement_replies
            WHERE member_id = $1
              AND status IN ('queued', 'drafted')
              AND (due_at IS NULL OR due_at <= NOW() + INTERVAL '1 day')
            ORDER BY priority_score DESC NULLS LAST, due_at NULLS LAST
            LIMIT $2
            "#,
        )
        .bind(member_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Internal(format!("engagement query: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|(id, action_type, priority_score, due_at)| TaskSummary {
                id,
                action_type,
                priority_score,
                due_at,
            })
            .collect())
    }

    /// Fetch due-follow-ups (active sequences whose next step is due).
    pub async fn due_followups(
        &self,
        member_id: Uuid,
        limit: i64,
    ) -> Result<Vec<SequenceSummary>, Error> {
        let rows: Vec<ActiveSequenceRow> = sqlx::query_as(
            r#"
            SELECT s.id, s.contact_id, s.current_step,
                   -- `lcc.sequences` has no `last_step_sent_at` column. The last
                   -- time a step actually went out is a fact about the steps, so
                   -- it is derived: the MAX of that sequence's `sent_at`. A
                   -- sequence with nothing sent yet yields NULL, which is the
                   -- honest answer rather than a stand-in. `sequences.started_at`
                   -- is NOT equivalent -- it is when the sequence was created.
                   (SELECT MAX(st.sent_at)
                      FROM lcc.sequence_steps st
                     WHERE st.sequence_id = s.id
                       AND st.sent_at IS NOT NULL) AS last_step_sent_at,
                   -- `lcc.sequences` names this column `state`, not `status`:
                   -- it is the lcc.sequence_state ENUM
                   -- (active|paused|completed|abandoned|replied).
                   s.state::TEXT AS status
            FROM lcc.sequences s
            WHERE s.member_id = $1
              AND s.state = 'active'
              AND EXISTS (
                  SELECT 1 FROM lcc.sequence_steps st
                  WHERE st.sequence_id = s.id
                    AND st.sent_at IS NULL
                    AND st.scheduled_at <= NOW()
              )
            -- Sequences that have never sent anything are the most overdue and
            -- so they lead. Among the rest, the longest-idle sequence is most
            -- overdue. `id` makes the order total and therefore stable.
            ORDER BY last_step_sent_at ASC NULLS FIRST, s.id
            LIMIT $2
            "#,
        )
        .bind(member_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Internal(format!("followups query: {e}")))?;

        Ok(rows
            .into_iter()
            .map(
                |(id, contact_id, current_step, last_step_sent_at, status)| SequenceSummary {
                    id,
                    contact_id,
                    current_step,
                    last_step_sent_at,
                    status,
                },
            )
            .collect())
    }

    /// Assemble the daily briefing using concurrent fan-out.
    pub async fn assemble_briefing(
        &self,
        member_id: Uuid,
        date: NaiveDate,
    ) -> Result<Briefing, Error> {
        // DAG-merge per Backend Design Concept §9.4 — concurrent fetch
        // then sequential merge. The orchestrator does not perform any
        // model inference itself; it only orders pre-scored data.
        let (approvals, opportunities, engagement, followups) = tokio::join!(
            self.pending_approvals(member_id, 25),
            self.hot_opportunities(member_id, 10),
            self.engagement_queue(member_id, 25),
            self.due_followups(member_id, 25),
        );

        Ok(Briefing {
            member_id,
            date,
            generated_at: Utc::now(),
            kind: BriefingKind::Morning,
            sections: BriefingSections {
                approvals_due: approvals?,
                hot_opportunities: opportunities?,
                engagement: engagement?,
                followups: followups?,
            },
        })
    }
}
