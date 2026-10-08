//! Opportunity repository.
//!
//! Every statement below targets a table that actually exists, and every
//! statement carries its own `member_id` predicate. RLS on these tables is
//! enabled and forced (`lcc.attach_member_rls`), but the audit's standing
//! finding is that the session variable is not set by most services, so the
//! explicit predicate — not the policy — is the tenant boundary. Do not drop it.
//!
//! Tables used:
//!   * `lcc.opportunities` — 0009 + 0017 (contract-shape columns)
//!   * `lcc.applications`  — 0014 (job applications AND client proposals)
//!
//! `lcc.opportunity_applications` and `lcc.proposals` — the two tables the
//! previous version of this file used — do not exist and never did.

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

use crate::domain::{
    Application, ApplicationStatus, ApplicationType, Opportunity, OpportunityCursor,
    OpportunityKind, OpportunityStatus, OpportunityTrack, Page, Source,
};
use crate::error::Error;

/// Postgres `unique_violation`.
const UNIQUE_VIOLATION: &str = "23505";

fn map_write_err(e: sqlx::Error, ctx: &str, key: &str) -> Error {
    if let sqlx::Error::Database(ref d) = e {
        if d.code().as_deref() == Some(UNIQUE_VIOLATION) {
            return Error::Conflict(format!(
                "an application already exists for {key}; lcc.applications is \
                 UNIQUE (member_id, opportunity_id, application_type)"
            ));
        }
    }
    Error::Internal(format!("{ctx}: {e}"))
}

#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}

// ---------------------------------------------------------------------------
// lcc.opportunities
// ---------------------------------------------------------------------------

/// A `lcc.opportunities` row.
///
/// Named fields decoded by column name rather than by tuple position: the
/// previous 11-element tuple had to be read positionally to tell the `source`
/// string from the `status` string, which is exactly the class of bug this
/// whole remediation is about.
struct OpportunityRow {
    id: Uuid,
    member_id: Uuid,
    kind: String,
    company_id: Option<Uuid>,
    contact_id: Option<Uuid>,
    title: String,
    company: Option<String>,
    source: String,
    status: String,
    fit_score: Option<f64>,
    phi_components: JsonValue,
    discovered_at: DateTime<Utc>,
    last_evaluated_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    metadata: JsonValue,
    version: i32,
}

impl<'r> FromRow<'r, PgRow> for OpportunityRow {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            member_id: row.try_get("member_id")?,
            kind: row.try_get("kind")?,
            company_id: row.try_get("company_id")?,
            contact_id: row.try_get("contact_id")?,
            title: row.try_get("title")?,
            company: row.try_get("company")?,
            source: row.try_get("source")?,
            status: row.try_get("status")?,
            fit_score: row.try_get("fit_score")?,
            phi_components: row.try_get("phi_components")?,
            discovered_at: row.try_get("discovered_at")?,
            last_evaluated_at: row.try_get("last_evaluated_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            metadata: row.try_get("metadata")?,
            version: row.try_get("version")?,
        })
    }
}

impl OpportunityRow {
    fn into_opportunity(self) -> Result<Opportunity, Error> {
        let kind = OpportunityKind::parse(&self.kind)
            .ok_or_else(|| Error::Internal(format!("unknown opportunity kind {}", self.kind)))?;
        let source = Source::parse(&self.source)
            .ok_or_else(|| Error::Internal(format!("unknown source {}", self.source)))?;
        let status = OpportunityStatus::parse(&self.status)
            .ok_or_else(|| Error::Internal(format!("unknown status {}", self.status)))?;
        Ok(Opportunity {
            // The position is the opportunity's title. `kind` is the
            // CATEGORY and is never used here.
            position: self.title.clone(),
            track: OpportunityTrack::from_kind(kind),
            id: self.id,
            member_id: self.member_id,
            kind,
            company_id: self.company_id,
            contact_id: self.contact_id,
            title: self.title,
            company: self.company,
            source,
            status,
            fit_score: self.fit_score,
            fit_components: self.phi_components,
            discovered_at: self.discovered_at,
            last_evaluated_at: self.last_evaluated_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
            metadata: self.metadata,
            version: self.version,
        })
    }
}

/// The service's ordering is `fit_score DESC NULLS LAST, discovered_at DESC,
/// id DESC`, written out in each query rather than interpolated so that every
/// statement stays independently checkable. `NULLS LAST` matches
/// `idx_opportunities_member_status_fit` (0017) and keeps unscored
/// opportunities at the end rather than ahead of every scored one; the
/// `discovered_at, id` tail makes the order total, which is what makes the
/// keyset cursor correct rather than merely plausible.
impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// One page of the member's opportunities, best-and-newest first.
    ///
    /// Fetches `limit + 1` rows so `has_more` is a fact rather than a guess.
    ///
    /// The keyset predicate is split in two because `fit_score` is nullable and
    /// the sort is `NULLS LAST`: while the cursor still carries a score, every
    /// unscored row lies *after* it; once the cursor itself sits in the
    /// unscored block, only unscored rows newer than the cursor qualify. A
    /// single `(fit_score, …) < (cursor, …)` comparison would be wrong at that
    /// boundary and would silently skip or repeat rows.
    pub async fn list(
        &self,
        member_id: Uuid,
        status: Option<OpportunityStatus>,
        limit: i64,
        cursor: Option<OpportunityCursor>,
    ) -> Result<Page<Opportunity>, Error> {
        let take = limit + 1;
        let rows: Vec<OpportunityRow> = match (status, cursor) {
            (Some(s), None) => {
                sqlx::query_as(
                    r#"SELECT id, member_id, kind::TEXT, company_id, contact_id, title, company,
                              source, status, fit_score, phi_components, discovered_at,
                              last_evaluated_at, created_at, updated_at, metadata, version
                       FROM lcc.opportunities
                       WHERE member_id = $1 AND status = $2::text
                       ORDER BY fit_score DESC NULLS LAST, discovered_at DESC, id DESC
                       LIMIT $3"#,
                )
                .bind(member_id)
                .bind(s.as_str())
                .bind(take)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as(
                    r#"SELECT id, member_id, kind::TEXT, company_id, contact_id, title, company,
                              source, status, fit_score, phi_components, discovered_at,
                              last_evaluated_at, created_at, updated_at, metadata, version
                       FROM lcc.opportunities
                       WHERE member_id = $1
                       ORDER BY fit_score DESC NULLS LAST, discovered_at DESC, id DESC
                       LIMIT $2"#,
                )
                .bind(member_id)
                .bind(take)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(s), Some(c)) if c.fit.is_some() => {
                sqlx::query_as(
                    r#"SELECT id, member_id, kind::TEXT, company_id, contact_id, title, company,
                              source, status, fit_score, phi_components, discovered_at,
                              last_evaluated_at, created_at, updated_at, metadata, version
                       FROM lcc.opportunities
                       WHERE member_id = $1 AND status = $2::text
                         AND ( fit_score IS NULL
                               OR fit_score < $3::double precision
                               OR (fit_score = $3::double precision
                                   AND (discovered_at, id) < ($4::timestamptz, $5::uuid)) )
                       ORDER BY fit_score DESC NULLS LAST, discovered_at DESC, id DESC
                       LIMIT $6"#,
                )
                .bind(member_id)
                .bind(s.as_str())
                .bind(c.fit)
                .bind(c.discovered_at)
                .bind(c.id)
                .bind(take)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(c)) if c.fit.is_some() => {
                sqlx::query_as(
                    r#"SELECT id, member_id, kind::TEXT, company_id, contact_id, title, company,
                              source, status, fit_score, phi_components, discovered_at,
                              last_evaluated_at, created_at, updated_at, metadata, version
                       FROM lcc.opportunities
                       WHERE member_id = $1
                         AND ( fit_score IS NULL
                               OR fit_score < $2::double precision
                               OR (fit_score = $2::double precision
                                   AND (discovered_at, id) < ($3::timestamptz, $4::uuid)) )
                       ORDER BY fit_score DESC NULLS LAST, discovered_at DESC, id DESC
                       LIMIT $5"#,
                )
                .bind(member_id)
                .bind(c.fit)
                .bind(c.discovered_at)
                .bind(c.id)
                .bind(take)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(s), Some(c)) => {
                sqlx::query_as(
                    r#"SELECT id, member_id, kind::TEXT, company_id, contact_id, title, company,
                              source, status, fit_score, phi_components, discovered_at,
                              last_evaluated_at, created_at, updated_at, metadata, version
                       FROM lcc.opportunities
                       WHERE member_id = $1 AND status = $2::text
                         AND fit_score IS NULL
                         AND (discovered_at, id) < ($3::timestamptz, $4::uuid)
                       ORDER BY fit_score DESC NULLS LAST, discovered_at DESC, id DESC
                       LIMIT $5"#,
                )
                .bind(member_id)
                .bind(s.as_str())
                .bind(c.discovered_at)
                .bind(c.id)
                .bind(take)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(c)) => {
                sqlx::query_as(
                    r#"SELECT id, member_id, kind::TEXT, company_id, contact_id, title, company,
                              source, status, fit_score, phi_components, discovered_at,
                              last_evaluated_at, created_at, updated_at, metadata, version
                       FROM lcc.opportunities
                       WHERE member_id = $1
                         AND fit_score IS NULL
                         AND (discovered_at, id) < ($2::timestamptz, $3::uuid)
                       ORDER BY fit_score DESC NULLS LAST, discovered_at DESC, id DESC
                       LIMIT $4"#,
                )
                .bind(member_id)
                .bind(c.discovered_at)
                .bind(c.id)
                .bind(take)
                .fetch_all(&self.pool)
                .await?
            }
        };

        let has_more = rows.len() > limit.max(0) as usize;
        let mut items = rows;
        if has_more {
            items.truncate(limit.max(0) as usize);
        }
        let next_cursor = if has_more {
            items.last().map(|r| OpportunityCursor {
                fit: r.fit_score,
                discovered_at: r.discovered_at,
                id: r.id,
            })
        } else {
            None
        };

        let items = items
            .into_iter()
            .map(OpportunityRow::into_opportunity)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Page::new(items, has_more, next_cursor.map(|c| c.encode())))
    }

    /// Verify that a `lcc.companies` / `lcc.contacts` reference supplied on a
    /// write belongs to the same member as the row being written.
    ///
    /// `lcc.opportunities.company_id` and `.contact_id` are independent
    /// foreign keys, so nothing in the schema stops a member's opportunity
    /// from carrying another member's contact id. Reads are member-scoped so
    /// this would not leak a row today, but it would seed a cross-tenant
    /// pointer that every future join inherits. A missing or foreign
    /// reference is reported as a validation error, identically, so this is
    /// not a probe for the existence of another member's rows.
    pub async fn assert_references_owned(
        &self,
        member_id: Uuid,
        company_id: Option<Uuid>,
        contact_id: Option<Uuid>,
    ) -> Result<(), Error> {
        if let Some(id) = company_id {
            let found: Option<(i32,)> = sqlx::query_as(
                r#"SELECT 1 FROM lcc.companies WHERE id = $1::uuid AND member_id = $2::uuid"#,
            )
            .bind(id)
            .bind(member_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Internal(format!("check company: {e}")))?;
            if found.is_none() {
                return Err(Error::Validation(
                    "company_id does not belong to this member".into(),
                ));
            }
        }
        if let Some(id) = contact_id {
            let found: Option<(i32,)> = sqlx::query_as(
                r#"SELECT 1 FROM lcc.contacts WHERE id = $1::uuid AND member_id = $2::uuid"#,
            )
            .bind(id)
            .bind(member_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Internal(format!("check contact: {e}")))?;
            if found.is_none() {
                return Err(Error::Validation(
                    "contact_id does not belong to this member".into(),
                ));
            }
        }
        Ok(())
    }

    /// Member-scoped single read. A row belonging to another member is
    /// indistinguishable from a row that does not exist: both yield
    /// `Error::NotFound`, so this is not an existence oracle.
    pub async fn get(&self, member_id: Uuid, id: Uuid) -> Result<Opportunity, Error> {
        let row: OpportunityRow = sqlx::query_as(
            r#"SELECT id, member_id, kind::TEXT, company_id, contact_id, title, company,
                      source, status, fit_score, phi_components, discovered_at,
                      last_evaluated_at, created_at, updated_at, metadata, version
               FROM lcc.opportunities WHERE member_id = $1 AND id = $2"#,
        )
        .bind(member_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("opportunity {id}")),
            other => Error::Internal(format!("get opportunity: {other}")),
        })?;
        row.into_opportunity()
    }

    pub async fn insert(&self, o: &Opportunity) -> Result<(), Error> {
        // A member must not be able to attach another member's contact or
        // company to their own opportunity. Nothing in the schema stops this
        // (the id columns are plain UUIDs with no cross-member FK), so the
        // check has to happen here or the row becomes a live cross-tenant
        // pointer. Missing and foreign references fail identically, so this
        // is not a probe for another member's rows.
        self.assert_references_owned(o.member_id, o.company_id, o.contact_id)
            .await?;

        // `kind` is `NOT NULL` with no default: the previous INSERT omitted it,
        // so every discovery would have failed at runtime with a not-null
        // violation even though the statement PREPAREd cleanly. `phi_score` is
        // kept in step with `fit_score` because 0017 declares the pair to be
        // the same value under two names, and `phi_score` is itself
        // `NOT NULL DEFAULT 0`.
        //
        // `funnel` is deliberately not written: 0017 back-filled `status` from
        // it and then declared `status` the source of truth. The two enums do
        // not map onto each other (see `OpportunityFunnel` in `domain.rs`),
        // so writing one from the other would invent a mapping.
        sqlx::query(
            r#"INSERT INTO lcc.opportunities
                  (id, member_id, kind, company_id, contact_id, title, company,
                   source, status, fit_score, phi_score, phi_components,
                   discovered_at, last_evaluated_at, created_at, updated_at,
                   metadata, version)
               VALUES ($1,$2,$3::lcc.opportunity_kind,$4,$5,$6,$7,
                       $8::text,$9::text,$10,$11,$12,$13,$14,$15,$16,$17,$18)"#,
        )
        .bind(o.id)
        .bind(o.member_id)
        .bind(o.kind.as_str())
        .bind(o.company_id)
        .bind(o.contact_id)
        .bind(&o.title)
        .bind(&o.company)
        .bind(o.source.as_str())
        .bind(o.status.as_str())
        .bind(o.fit_score)
        .bind(o.fit_score.unwrap_or(0.0_f64))
        .bind(&o.fit_components)
        .bind(o.discovered_at)
        .bind(o.last_evaluated_at)
        .bind(o.created_at)
        .bind(o.updated_at)
        .bind(&o.metadata)
        .bind(o.version)
        .execute(&self.pool)
        .await
        .map_err(|e| map_write_err(e, "insert opportunity", &o.id.to_string()))?;
        Ok(())
    }

    /// Optimistic-concurrency status transition. Returns the new version.
    pub async fn qualify(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        fit_score: f64,
        status: OpportunityStatus,
        fit_components: &JsonValue,
    ) -> Result<i32, Error> {
        let row: (i32,) = sqlx::query_as(
            r#"UPDATE lcc.opportunities
               SET fit_score = $4, status = $5::text,
                   phi_score = $4, phi_components = $6,
                   last_evaluated_at = NOW(), updated_at = NOW(),
                   version = version + 1
               WHERE member_id = $1 AND id = $2 AND version = $3
               RETURNING version"#,
        )
        .bind(member_id)
        .bind(id)
        .bind(expected_version)
        .bind(fit_score)
        .bind(status.as_str())
        .bind(fit_components)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            // Stale version and unknown id are both "no row matched". Only the
            // version case is a write conflict; the foreign-member case falls
            // through to the `get` below, which reports NotFound.
            sqlx::Error::RowNotFound => Error::Conflict(format!("opp {id} v{expected_version}")),
            other => Error::Internal(format!("qualify: {other}")),
        })?;
        Ok(row.0)
    }

    // -----------------------------------------------------------------------
    // lcc.applications — job applications AND client proposals
    // -----------------------------------------------------------------------

    pub async fn insert_application(&self, a: &Application) -> Result<(), Error> {
        sqlx::query(
            r#"INSERT INTO lcc.applications
                  (id, member_id, opportunity_id, application_type, status,
                   submitted_at, response_received_at, payload, idempotency_key,
                   version, created_at, updated_at)
               VALUES ($1,$2,$3,$4::text,$5::text,$6,$7,$8,$9,$10,$11,$12)"#,
        )
        .bind(a.id)
        .bind(a.member_id)
        .bind(a.opportunity_id)
        .bind(a.application_type.as_str())
        .bind(a.status.as_str())
        .bind(a.submitted_at)
        .bind(a.response_received_at)
        .bind(&a.payload)
        .bind(&a.idempotency_key)
        .bind(a.version)
        .bind(a.created_at)
        .bind(a.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            map_write_err(
                e,
                "insert application",
                &format!("{} {}", a.opportunity_id, a.application_type.as_str()),
            )
        })?;
        Ok(())
    }

    /// List the member's applications, newest first.
    ///
    /// `application_type` selects the view: `Some(ClientProposal)` is what the
    /// proposals endpoint serves, and it reads the very same table rather than
    /// a `lcc.proposals` table that does not exist.
    ///
    /// `position` is `o.title` — the LEFT JOIN is on `member_id` as well as
    /// `id`, so even a corrupted `opportunity_id` cannot pull another member's
    /// title across the tenant boundary. `kind` is the opportunity CATEGORY and
    /// is never selected as the position.
    pub async fn list_applications(
        &self,
        member_id: Uuid,
        application_type: Option<ApplicationType>,
        limit: i64,
    ) -> Result<Vec<Application>, Error> {
        let rows: Vec<ApplicationRow> = match application_type {
            Some(t) => {
                sqlx::query_as(
                    r#"SELECT a.id, a.member_id, a.opportunity_id, a.application_type,
                              a.status, a.submitted_at, a.response_received_at, a.payload,
                              a.idempotency_key, a.version, a.created_at, a.updated_at,
                              o.title AS position
                       FROM lcc.applications a
                       LEFT JOIN lcc.opportunities o
                              ON o.id = a.opportunity_id AND o.member_id = a.member_id
                       WHERE a.member_id = $1 AND a.application_type = $2::text
                       ORDER BY a.created_at DESC, a.id DESC
                       LIMIT $3"#,
                )
                .bind(member_id)
                .bind(t.as_str())
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as(
                    r#"SELECT a.id, a.member_id, a.opportunity_id, a.application_type,
                              a.status, a.submitted_at, a.response_received_at, a.payload,
                              a.idempotency_key, a.version, a.created_at, a.updated_at,
                              o.title AS position
                       FROM lcc.applications a
                       LEFT JOIN lcc.opportunities o
                              ON o.id = a.opportunity_id AND o.member_id = a.member_id
                       WHERE a.member_id = $1
                       ORDER BY a.created_at DESC, a.id DESC
                       LIMIT $2"#,
                )
                .bind(member_id)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };
        rows.into_iter()
            .map(ApplicationRow::into_application)
            .collect()
    }
}

struct ApplicationRow {
    id: Uuid,
    member_id: Uuid,
    opportunity_id: Uuid,
    application_type: String,
    status: String,
    position: Option<String>,
    submitted_at: Option<DateTime<Utc>>,
    response_received_at: Option<DateTime<Utc>>,
    payload: JsonValue,
    idempotency_key: Option<String>,
    version: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl<'r> FromRow<'r, PgRow> for ApplicationRow {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            member_id: row.try_get("member_id")?,
            opportunity_id: row.try_get("opportunity_id")?,
            application_type: row.try_get("application_type")?,
            status: row.try_get("status")?,
            position: row.try_get("position")?,
            submitted_at: row.try_get("submitted_at")?,
            response_received_at: row.try_get("response_received_at")?,
            payload: row.try_get("payload")?,
            idempotency_key: row.try_get("idempotency_key")?,
            version: row.try_get("version")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl ApplicationRow {
    fn into_application(self) -> Result<Application, Error> {
        let application_type = ApplicationType::parse(&self.application_type).ok_or_else(|| {
            Error::Internal(format!(
                "unknown application_type {}",
                self.application_type
            ))
        })?;
        let status = ApplicationStatus::parse(&self.status).ok_or_else(|| {
            Error::Internal(format!("unknown application status {}", self.status))
        })?;
        Ok(Application {
            id: self.id,
            member_id: self.member_id,
            opportunity_id: self.opportunity_id,
            application_type,
            status,
            position: self.position,
            submitted_at: self.submitted_at,
            response_received_at: self.response_received_at,
            payload: self.payload,
            idempotency_key: self.idempotency_key,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
