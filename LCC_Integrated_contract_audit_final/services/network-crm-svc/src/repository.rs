//! network-crm-svc repository.

use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    Company, Contact, Interaction, InteractionDirection, InteractionKind, StaleContact,
    StalenessReport,
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

    pub async fn list_contacts(
        &self,
        member_id: Uuid,
        q: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Contact>, Error> {
        let rows: Vec<(
            Uuid, Uuid, Option<Uuid>, String, Option<String>, Option<String>,
            Option<String>, Option<String>, i16, Option<DateTime<Utc>>,
            Option<String>, Vec<String>, Option<String>, Option<DateTime<Utc>>,
            i32, DateTime<Utc>, DateTime<Utc>,
        )> = if let Some(q) = q {
            sqlx::query_as(
                r#"
                SELECT id, member_id, company_id, full_name, title, headline,
                       linkedin_url, email, connection_strength, last_touched_at,
                       last_interaction_kind, tags, notes, stale_at, version,
                       created_at, updated_at
                FROM lcc.contacts
                WHERE member_id = $1 AND full_name ILIKE '%' || $2 || '%'
                ORDER BY updated_at DESC
                LIMIT $3
                "#,
            )
            .bind(member_id)
            .bind(q)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT id, member_id, company_id, full_name, title, headline,
                       linkedin_url, email, connection_strength, last_touched_at,
                       last_interaction_kind, tags, notes, stale_at, version,
                       created_at, updated_at
                FROM lcc.contacts
                WHERE member_id = $1
                ORDER BY updated_at DESC
                LIMIT $2
                "#,
            )
            .bind(member_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter()
            .map(|(id, member_id, company_id, full_name, title, headline, linkedin_url, email,
                   connection_strength, last_touched_at, last_interaction_kind, tags, notes,
                   stale_at, version, created_at, updated_at)| {
                Ok::<Contact, Error>(Contact {
                    id, member_id, company_id, full_name, title, headline,
                    linkedin_url, email, connection_strength,
                    last_touched_at, last_interaction_kind, tags, notes,
                    stale_at, version, created_at, updated_at,
                })
            })
            .collect()
    }

    pub async fn get_contact(&self, member_id: Uuid, id: Uuid) -> Result<Contact, Error> {
        let row: (Uuid, Uuid, Option<Uuid>, String, Option<String>, Option<String>,
            Option<String>, Option<String>, i16, Option<DateTime<Utc>>,
            Option<String>, Vec<String>, Option<String>, Option<DateTime<Utc>>,
            i32, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            SELECT id, member_id, company_id, full_name, title, headline,
                   linkedin_url, email, connection_strength, last_touched_at,
                   last_interaction_kind, tags, notes, stale_at, version,
                   created_at, updated_at
            FROM lcc.contacts WHERE member_id = $1 AND id = $2
            "#,
        )
        .bind(member_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("contact {id}")),
            other => Error::Internal(format!("get contact: {other}")),
        })?;
        Ok(Contact {
            id: row.0, member_id: row.1, company_id: row.2, full_name: row.3,
            title: row.4, headline: row.5, linkedin_url: row.6, email: row.7,
            connection_strength: row.8, last_touched_at: row.9,
            last_interaction_kind: row.10, tags: row.11, notes: row.12,
            stale_at: row.13, version: row.14, created_at: row.15, updated_at: row.16,
        })
    }

    pub async fn insert_contact(&self, c: &Contact) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO lcc.contacts
                (id, member_id, company_id, full_name, title, headline,
                 linkedin_url, email, connection_strength, last_touched_at,
                 last_interaction_kind, tags, notes, stale_at, version,
                 created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            "#,
        )
        .bind(c.id).bind(c.member_id).bind(c.company_id).bind(&c.full_name)
        .bind(&c.title).bind(&c.headline).bind(&c.linkedin_url).bind(&c.email)
        .bind(c.connection_strength).bind(c.last_touched_at)
        .bind(&c.last_interaction_kind).bind(&c.tags).bind(&c.notes)
        .bind(c.stale_at).bind(c.version).bind(c.created_at).bind(c.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_contact(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        contact: &Contact,
    ) -> Result<i32, Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            UPDATE lcc.contacts
            SET company_id = $5, full_name = $6, title = $7, headline = $8,
                linkedin_url = $9, email = $10, connection_strength = $11,
                tags = $12, notes = $13, version = version + 1, updated_at = NOW()
            WHERE member_id = $1 AND id = $2 AND version = $3
            RETURNING version
            "#,
        )
        .bind(member_id)
        .bind(id)
        .bind(expected_version)
        .bind(contact.company_id)
        .bind(&contact.full_name)
        .bind(&contact.title)
        .bind(&contact.headline)
        .bind(&contact.linkedin_url)
        .bind(&contact.email)
        .bind(contact.connection_strength)
        .bind(&contact.tags)
        .bind(&contact.notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::Conflict(format!("contact v{expected_version}")),
            other => Error::Internal(format!("update contact: {other}")),
        })?;
        Ok(row.0)
    }

    pub async fn delete_contact(&self, member_id: Uuid, id: Uuid) -> Result<(), Error> {
        sqlx::query("DELETE FROM lcc.contacts WHERE member_id = $1 AND id = $2")
            .bind(member_id).bind(id)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn touch_contact(
        &self,
        member_id: Uuid,
        id: Uuid,
        kind: &str,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
            UPDATE lcc.contacts
            SET last_touched_at = NOW(),
                last_interaction_kind = $4,
                stale_at = NULL,
                updated_at = NOW()
            WHERE member_id = $1 AND id = $2
            "#,
        )
        .bind(member_id)
        .bind(id)
        .bind(kind)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_companies(
        &self,
        member_id: Uuid,
        q: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Company>, Error> {
        let rows: Vec<(
            Uuid, Uuid, String, Option<String>, Option<String>,
            Option<String>, Option<String>, Vec<String>, Vec<String>,
            Vec<String>, Option<NaiveDate>, i32, DateTime<Utc>, DateTime<Utc>,
        )> = if let Some(q) = q {
            sqlx::query_as(
                r#"
                SELECT id, member_id, name, domain, industry,
                       size_band, funding_stage, tech_stack,
                       trigger_events, public_signals, ttl_at,
                       version, created_at, updated_at
                FROM lcc.companies
                WHERE member_id = $1 AND name ILIKE '%' || $2 || '%'
                ORDER BY updated_at DESC
                LIMIT $3
                "#,
            )
            .bind(member_id).bind(q).bind(limit).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as(
                r#"
                SELECT id, member_id, name, domain, industry,
                       size_band, funding_stage, tech_stack,
                       trigger_events, public_signals, ttl_at,
                       version, created_at, updated_at
                FROM lcc.companies
                WHERE member_id = $1
                ORDER BY updated_at DESC
                LIMIT $2
                "#,
            )
            .bind(member_id).bind(limit).fetch_all(&self.pool).await?
        };
        rows.into_iter().map(|r| Ok::<Company, Error>(Company {
            id: r.0, member_id: r.1, name: r.2, domain: r.3, industry: r.4,
            size_band: r.5, funding_stage: r.6, tech_stack: r.7,
            trigger_events: r.8, public_signals: r.9, ttl_at: r.10,
            version: r.11, created_at: r.12, updated_at: r.13,
        })).collect()
    }

    pub async fn insert_company(&self, c: &Company) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO lcc.companies
                (id, member_id, name, domain, industry,
                 size_band, funding_stage, tech_stack,
                 trigger_events, public_signals, ttl_at, version, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            "#,
        )
        .bind(c.id).bind(c.member_id).bind(&c.name).bind(&c.domain)
        .bind(&c.industry).bind(&c.size_band).bind(&c.funding_stage)
        .bind(&c.tech_stack).bind(&c.trigger_events).bind(&c.public_signals)
        .bind(c.ttl_at).bind(c.version).bind(c.created_at).bind(c.updated_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn record_interaction(&self, i: &Interaction) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO lcc.interactions
                (id, member_id, contact_id, kind, summary, occurred_at,
                 channel, direction, metadata)
            VALUES ($1,$2,$3,$4::text,$5,$6,$7,$8::text,$9)
            "#,
        )
        .bind(i.id).bind(i.member_id).bind(i.contact_id)
        .bind(i.kind.as_str()).bind(&i.summary).bind(i.occurred_at)
        .bind(&i.channel).bind(i.direction.as_str()).bind(&i.metadata)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn staleness(
        &self,
        member_id: Uuid,
        days: i64,
    ) -> Result<StalenessReport, Error> {
        let rows: Vec<(Uuid, String, Option<DateTime<Utc>>)> = sqlx::query_as(
            r#"
            SELECT id, full_name, last_touched_at
            FROM lcc.contacts
            WHERE member_id = $1
              AND (last_touched_at IS NULL OR last_touched_at < NOW() - $2 * INTERVAL '1 day')
            ORDER BY last_touched_at NULLS FIRST
            "#,
        )
        .bind(member_id)
        .bind(days as f64)
        .fetch_all(&self.pool)
        .await?;
        let stale: Vec<StaleContact> = rows.into_iter().map(|(id, full_name, last_touched_at)| {
            let days_since_touch = last_touched_at
                .map(|t| (Utc::now() - t).num_days())
                .unwrap_or(days * 2 + 90);
            StaleContact {
                contact_id: id,
                full_name: full_name.clone(),
                days_since_touch,
                suggested_action: suggest(&full_name, days_since_touch),
            }
        }).collect();
        Ok(StalenessReport {
            member_id,
            as_of: Utc::now(),
            stale_contacts: stale,
        })
    }

    pub async fn link_contact_company(
        &self,
        member_id: Uuid,
        contact_id: Uuid,
        company_id: Uuid,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE lcc.contacts SET company_id = $3, updated_at = NOW()
             WHERE member_id = $1 AND id = $2",
        )
        .bind(member_id).bind(contact_id).bind(company_id)
        .execute(&self.pool).await?;
        Ok(())
    }
}

fn suggest(full_name: &str, days: i64) -> String {
    use std::fmt::Write;
    let mut buf = String::new();
    let _ = write!(&mut buf, "{}", full_name);
    buf.push_str(" — ");
    if days > 180 {
        buf.push_str("send an anniversary note or congrats on a recent post");
    } else if days > 90 {
        buf.push_str("share a quick industry observation to re-engage");
    } else {
        buf.push_str("send a low-stakes 'thinking of you' note");
    }
    buf
}

#[allow(dead_code)]
fn _types(_: JsonValue) { let _ = InteractionKind::Other; let _ = InteractionDirection::Inbound; }
