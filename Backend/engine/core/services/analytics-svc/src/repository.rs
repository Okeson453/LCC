//! Analytics repository — read-only aggregate queries.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    ContentMetrics, DashboardSummary, EngagementMetrics, Granularity, OpportunityMetrics,
    OutreachMetrics, TimeSeries, TimeSeriesPoint,
};
use crate::error::Error;

#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn dashboard(
        &self,
        member_id: Uuid,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<DashboardSummary, Error> {
        let content = sqlx::query_as::<_, (i64, i64, i64, i64, Option<f64>)>(
            r#"SELECT
                  COUNT(*) FILTER (WHERE state = 'published')::BIGINT,
                  COUNT(*) FILTER (WHERE state = 'scheduled')::BIGINT,
                  COUNT(*) FILTER (WHERE state = 'drafted')::BIGINT,
                  COUNT(*) FILTER (WHERE state = 'in_review')::BIGINT,
                  AVG(quality_loop_count)::FLOAT8
              FROM lcc.content_items
              WHERE member_id = $1 AND created_at::DATE BETWEEN $2 AND $3"#,
        )
        .bind(member_id)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        let outreach = sqlx::query_as::<_, (i64, i64, i64)>(
            r#"SELECT
                  COUNT(*) FILTER (WHERE status = 'active')::BIGINT,
                  COUNT(*) FILTER (WHERE status = 'paused')::BIGINT,
                  COUNT(ss.id) FILTER (WHERE ss.response_received_at IS NOT NULL)::BIGINT
              FROM lcc.sequences s
              LEFT JOIN lcc.sequence_steps ss ON ss.sequence_id = s.id
              WHERE s.member_id = $1
                AND s.created_at::DATE BETWEEN $2 AND $3"#,
        )
        .bind(member_id)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        let engagement = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"SELECT
                  (SELECT COUNT(*) FROM lcc.engagement_inbox
                   WHERE member_id = $1 AND received_at::DATE BETWEEN $2 AND $3)::BIGINT,
                  COUNT(*) FILTER (WHERE status = 'queued')::BIGINT,
                  COUNT(*) FILTER (WHERE status = 'completed')::BIGINT,
                  COUNT(*) FILTER (WHERE status = 'drafted')::BIGINT
              FROM lcc.engagement_replies
              WHERE member_id = $1 AND created_at::DATE BETWEEN $2 AND $3"#,
        )
        .bind(member_id)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        let opportunity = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
            r#"SELECT
                  COUNT(*) FILTER (WHERE status = 'discovered')::BIGINT,
                  COUNT(*) FILTER (WHERE status = 'qualified')::BIGINT,
                  COUNT(*) FILTER (WHERE status = 'applied')::BIGINT,
                  COUNT(*) FILTER (WHERE status = 'interviewing')::BIGINT,
                  COUNT(*) FILTER (WHERE status = 'offer')::BIGINT
              FROM lcc.opportunities
              WHERE member_id = $1
                AND discovered_at::DATE BETWEEN $2 AND $3"#,
        )
        .bind(member_id)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        let outreach_sent: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*)::BIGINT
               FROM lcc.sequence_steps ss
               JOIN lcc.sequences s ON s.id = ss.sequence_id
               WHERE s.member_id = $1
                 AND ss.sent_at::DATE BETWEEN $2 AND $3"#,
        )
        .bind(member_id)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let reply_rate = if outreach_sent > 0 {
            Some(outreach.2 as f64 / outreach_sent as f64)
        } else {
            None
        };

        Ok(DashboardSummary {
            member_id,
            as_of: Utc::now(),
            window_start: start,
            window_end: end,
            content_metrics: ContentMetrics {
                published: content.0,
                scheduled: content.1,
                draft: content.2,
                in_review: content.3,
                average_quality_loop_count: content.4,
            },
            outreach_metrics: OutreachMetrics {
                active_sequences: outreach.0,
                paused_sequences: outreach.1,
                step_replies: outreach.2,
                reply_rate,
            },
            engagement_metrics: EngagementMetrics {
                inbound_count: engagement.0,
                queued_tasks: engagement.1,
                completed_tasks: engagement.2,
                draft_ready_tasks: engagement.3,
            },
            opportunity_metrics: OpportunityMetrics {
                discovered: opportunity.0,
                qualified: opportunity.1,
                applied: opportunity.2,
                interviewing: opportunity.3,
                offers: opportunity.4,
            },
        })
    }

    pub async fn time_series(
        &self,
        member_id: Uuid,
        metric: &str,
        granularity: Granularity,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<TimeSeries, Error> {
        let bucket = match granularity {
            Granularity::Day => "day",
            Granularity::Week => "week",
            Granularity::Month => "month",
        };
        // Whitelist of supported metrics (prevents SQL injection).
        let metric_sql = match metric {
            "content.published" => (
                "SELECT date_trunc($3, published_at)::DATE AS d, COUNT(*)::BIGINT
                  FROM lcc.content_items
                  WHERE member_id = $1 AND published_at::DATE BETWEEN $4 AND $5
                  GROUP BY 1 ORDER BY 1",
                "content.published",
            ),
            "engagement.completed" => (
                "SELECT date_trunc($3, completed_at)::DATE AS d, COUNT(*)::BIGINT
                  FROM lcc.engagement_replies
                  WHERE member_id = $1 AND completed_at::DATE BETWEEN $4 AND $5
                  GROUP BY 1 ORDER BY 1",
                "engagement.completed",
            ),
            "outreach.replies" => (
                "SELECT date_trunc($3, ss.response_received_at)::DATE AS d, COUNT(*)::BIGINT
                  FROM lcc.sequence_steps ss
                  JOIN lcc.sequences s ON s.id = ss.sequence_id
                  WHERE s.member_id = $1
                    AND ss.response_received_at::DATE BETWEEN $4 AND $5
                  GROUP BY 1 ORDER BY 1",
                "outreach.replies",
            ),
            other => return Err(Error::Validation(format!("unknown metric {other}"))),
        };

        let rows: Vec<(NaiveDate, i64)> = sqlx::query_as(metric_sql.0)
            .bind(member_id)
            .bind(metric)
            .bind(bucket)
            .bind(start)
            .bind(end)
            .fetch_all(&self.pool)
            .await?;

        Ok(TimeSeries {
            metric: metric_sql.1.to_string(),
            granularity,
            points: rows
                .into_iter()
                .map(|(d, v)| TimeSeriesPoint {
                    date: d,
                    value: v as f64,
                })
                .collect(),
        })
    }

    #[allow(dead_code)]
    pub async fn _t(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
