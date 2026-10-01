//! Opportunity repository.

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    Application, Opportunity, OpportunityStatus, Proposal, ProposalStatus, Source,
};
use crate::error::Error;

#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn list(
        &self,
        member_id: Uuid,
        status: Option<OpportunityStatus>,
        limit: i64,
    ) -> Result<Vec<Opportunity>, Error> {
        let rows: Vec<(
            Uuid, Uuid, Option<Uuid>, String, String, String, Option<f64>,
            DateTime<Utc>, Option<DateTime<Utc>>, JsonValue, i32,
        )> = if let Some(s) = status {
            sqlx::query_as(
                r#"SELECT id, member_id, company_id, title, source::TEXT, status::TEXT,
                          fit_score, discovered_at, last_evaluated_at, metadata, version
                   FROM lcc.opportunities
                   WHERE member_id = $1 AND status = $2::text
                   ORDER BY fit_score DESC NULLS LAST, discovered_at DESC
                   LIMIT $3"#,
            )
            .bind(member_id).bind(s.as_str()).bind(limit).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as(
                r#"SELECT id, member_id, company_id, title, source::TEXT, status::TEXT,
                          fit_score, discovered_at, last_evaluated_at, metadata, version
                   FROM lcc.opportunities
                   WHERE member_id = $1
                   ORDER BY fit_score DESC NULLS LAST, discovered_at DESC
                   LIMIT $2"#,
            )
            .bind(member_id).bind(limit).fetch_all(&self.pool).await?
        };
        rows.into_iter().map(map_opp).collect()
    }

    pub async fn insert(&self, o: &Opportunity) -> Result<(), Error> {
        sqlx::query(
            r#"INSERT INTO lcc.opportunities
                  (id, member_id, company_id, title, source, status,
                   fit_score, discovered_at, last_evaluated_at, metadata, version)
               VALUES ($1,$2,$3,$4,$5::text,$6::text,$7,$8,$9,$10,$11)"#,
        )
        .bind(o.id).bind(o.member_id).bind(o.company_id).bind(&o.title)
        .bind(o.source.as_str()).bind(o.status.as_str())
        .bind(o.fit_score).bind(o.discovered_at).bind(o.last_evaluated_at)
        .bind(&o.metadata).bind(o.version).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get(&self, member_id: Uuid, id: Uuid) -> Result<Opportunity, Error> {
        let row: (Uuid, Uuid, Option<Uuid>, String, String, String, Option<f64>,
            DateTime<Utc>, Option<DateTime<Utc>>, JsonValue, i32) = sqlx::query_as(
            r#"SELECT id, member_id, company_id, title, source::TEXT, status::TEXT,
                      fit_score, discovered_at, last_evaluated_at, metadata, version
               FROM lcc.opportunities WHERE member_id = $1 AND id = $2"#,
        )
        .bind(member_id).bind(id).fetch_one(&self.pool).await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("opportunity {id}")),
            other => Error::Internal(format!("get opp: {other}")),
        })?;
        Ok(map_opp(row)?)
    }

    pub async fn qualify(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        fit_score: f64,
        status: OpportunityStatus,
    ) -> Result<i32, Error> {
        let row: (i32,) = sqlx::query_as(
            r#"UPDATE lcc.opportunities
               SET fit_score = $4, status = $5::text,
                   last_evaluated_at = NOW(), version = version + 1
               WHERE member_id = $1 AND id = $2 AND version = $3
               RETURNING version"#,
        )
        .bind(member_id).bind(id).bind(expected_version).bind(fit_score).bind(status.as_str())
        .fetch_one(&self.pool).await.map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::Conflict(format!("opp {id} v{expected_version}")),
            other => Error::Internal(format!("qualify: {other}")),
        })?;
        Ok(row.0)
    }

    pub async fn insert_application(&self, a: &Application) -> Result<(), Error> {
        sqlx::query(
            r#"INSERT INTO lcc.opportunity_applications
                  (id, member_id, opportunity_id, status,
                   submitted_at, resume_doc_id, cover_letter, version)
               VALUES ($1,$2,$3,$4::text,$5,$6,$7,$8)"#,
        )
        .bind(a.id).bind(a.member_id).bind(a.opportunity_id)
        .bind(a.status.as_str()).bind(a.submitted_at)
        .bind(a.resume_doc_id).bind(&a.cover_letter).bind(a.version)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_applications(
        &self, member_id: Uuid, limit: i64,
    ) -> Result<Vec<Application>, Error> {
        let rows: Vec<(Uuid, Uuid, Uuid, String, DateTime<Utc>,
            Option<Uuid>, Option<String>, i32)> = sqlx::query_as(
            r#"SELECT id, member_id, opportunity_id, status::TEXT,
                      submitted_at, resume_doc_id, cover_letter, version
               FROM lcc.opportunity_applications
               WHERE member_id = $1
               ORDER BY submitted_at DESC
               LIMIT $2"#,
        )
        .bind(member_id).bind(limit).fetch_all(&self.pool).await?;
        rows.into_iter().map(|(id, member_id, opportunity_id, status, submitted_at,
                              resume_doc_id, cover_letter, version)| {
            let s = serde_json::from_value::<OpportunityStatus>(serde_json::Value::String(status.clone()))
                .map_err(|e| Error::Internal(format!("status parse: {e}")))?;
            Ok::<Application, Error>(Application {
                id, member_id, opportunity_id, status: s,
                submitted_at, resume_doc_id, cover_letter, version,
            })
        }).collect()
    }

    pub async fn insert_proposal(&self, p: &Proposal) -> Result<(), Error> {
        sqlx::query(
            r#"INSERT INTO lcc.proposals
                  (id, member_id, opportunity_id, title, body,
                   price_cents, currency, status, kb_ref_ids,
                   sent_at, version, created_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8::text,$9,$10,$11,$12)"#,
        )
        .bind(p.id).bind(p.member_id).bind(p.opportunity_id).bind(&p.title)
        .bind(&p.body).bind(p.price_cents).bind(&p.currency)
        .bind(p.status.as_str()).bind(&p.kb_ref_ids).bind(p.sent_at)
        .bind(p.version).bind(p.created_at).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_proposals(
        &self, member_id: Uuid, limit: i64,
    ) -> Result<Vec<Proposal>, Error> {
        let rows: Vec<(Uuid, Uuid, Uuid, String, String, Option<i64>,
            Option<String>, String, Vec<Uuid>, Option<DateTime<Utc>>,
            i32, DateTime<Utc>)> = sqlx::query_as(
            r#"SELECT id, member_id, opportunity_id, title, body,
                      price_cents, currency, status::TEXT, kb_ref_ids,
                      sent_at, version, created_at
               FROM lcc.proposals
               WHERE member_id = $1
               ORDER BY created_at DESC
               LIMIT $2"#,
        )
        .bind(member_id).bind(limit).fetch_all(&self.pool).await?;
        rows.into_iter().map(|(id, member_id, opportunity_id, title, body,
                              price_cents, currency, status, kb_ref_ids,
                              sent_at, version, created_at)| {
            let s = serde_json::from_value::<ProposalStatus>(serde_json::Value::String(status.clone()))
                .map_err(|e| Error::Internal(format!("status parse: {e}")))?;
            Ok::<Proposal, Error>(Proposal {
                id, member_id, opportunity_id, title, body,
                price_cents, currency, status: s, kb_ref_ids,
                sent_at, version, created_at,
            })
        }).collect()
    }
}

fn map_opp(
    row: (Uuid, Uuid, Option<Uuid>, String, String, String, Option<f64>,
        DateTime<Utc>, Option<DateTime<Utc>>, JsonValue, i32),
) -> Result<Opportunity, Error> {
    let src = serde_json::from_value::<Source>(serde_json::Value::String(row.4.clone()))
        .map_err(|e| Error::Internal(format!("source parse: {e}")))?;
    let st = serde_json::from_value::<OpportunityStatus>(serde_json::Value::String(row.5.clone()))
        .map_err(|e| Error::Internal(format!("status parse: {e}")))?;
    Ok(Opportunity {
        id: row.0, member_id: row.1, company_id: row.2, title: row.3,
        source: src, status: st, fit_score: row.6,
        discovered_at: row.7, last_evaluated_at: row.8,
        metadata: row.9, version: row.10,
    })
}
