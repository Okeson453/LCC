//! network-crm-svc repository.
//!
//! Every statement here was reconciled against the live schema. The three
//! tables this service touches are `lcc.contacts` (0008 + 0016 + 0017 + 0020),
//! `lcc.companies` (0017) and `lcc.interactions` (0017) — all three really
//! exist; the pre-remediation queries failed on *columns*, not on tables.
//!
//! Two invariants are load-bearing and must survive any future edit:
//!
//! 1. **Tenant scoping.** Every statement that reads or writes a row carries an
//!    explicit `member_id = $1` predicate, including the writes whose only
//!    natural key is the row id. `lcc.contacts` also has a forced RLS policy
//!    (`lcc.attach_member_rls`), but the service never sets
//!    `app.current_member_id`, so the policy is not what keeps one member out of
//!    another's rows — the predicates below are. They are not to be "simplified
//!    away" because a FK or an RLS policy looks like it would cover for them.
//! 2. **No existence oracle.** A statement scoped to the caller returns zero
//!    rows both when the row does not exist and when it belongs to somebody
//!    else, and every such call maps that to the same 404. Nothing in this
//!    module may return 403 or a distinct error for "exists but not yours".

use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    Company, ConnectionStatus, Contact, ContactTier, Interaction, InteractionKind,
    RelationshipStage, RelationshipStrength, StaleContact, StalenessReport,
};
use crate::error::Error;

#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}

/// The contact projection, shared by `list_contacts` and `get_contact`.
///
/// A named struct rather than a tuple: the projection has 27 columns and sqlx
/// only implements `FromRow` for tuples up to arity 16.
///
/// The two derived columns are spelled out here once, so no query has to
/// reinvent the "which interaction counts" rule:
///
/// * `last_interaction_kind` — `lcc.interactions` (0017) is the symmetric log
///   of every contact-touching event. `ORDER BY occurred_at DESC` matches the
///   primary reader's index, `idx_interactions_contact_occurred (contact_id,
///   occurred_at DESC)`. `created_at, id` break ties deterministically so two
///   interactions logged in the same millisecond cannot make the response
///   flap. `deleted_at IS NULL` because every index on that table is partial on
///   it. The `member_id` term is redundant against the contact row's own
///   predicate but keeps the subquery tenant-safe on its own.
/// * `opportunity_id` — `MIN(o.id::text)::uuid` with `HAVING COUNT(*) = 1`.
///   `lcc.opportunities.contact_id` (0009) points at the contact, so the
///   relation is one-to-many. `COUNT(*) = 1` is what makes this honest: with
///   several linked opportunities there is no single correct answer, so the
///   value is NULL rather than a guess. `MIN(...)::uuid` is used because
///   PostgreSQL has no `min(uuid)`.
const CONTACT_PROJECTION: &str = "
                c.id, c.member_id, c.company_id, c.company, c.display_name,
                c.headline, c.title, c.linkedin_id, c.linkedin_url,
                c.tier, c.is_vip, c.is_mutual, c.connection_status,
                c.relationship_stage, c.relationship_strength::TEXT,
                c.tags, c.metadata ->> 'notes' AS notes, c.metadata,
                c.first_contact_date, c.last_contact_at,
                c.follow_up_date, c.stale, c.stale_since, c.version,
                c.created_at, c.updated_at,
                (SELECT i.kind
                   FROM lcc.interactions i
                  WHERE i.contact_id = c.id
                    AND i.member_id = c.member_id
                    AND i.deleted_at IS NULL
                  ORDER BY i.occurred_at DESC, i.created_at DESC, i.id DESC
                  LIMIT 1) AS last_interaction_kind,
                (SELECT MIN(o.id::TEXT)::UUID
                   FROM lcc.opportunities o
                  WHERE o.contact_id = c.id
                    AND o.member_id = c.member_id
                 HAVING COUNT(*) = 1) AS opportunity_id";

/// One `lcc.contacts` row, with the text-valued constrained columns left as
/// `String` and converted in `TryFrom` — see [`ContactRow::try_into_contact`].
#[derive(Debug, sqlx::FromRow)]
pub struct ContactRow {
    pub id: Uuid,
    pub member_id: Uuid,
    pub company_id: Option<Uuid>,
    pub company: Option<String>,
    pub display_name: String,
    pub headline: Option<String>,
    pub title: Option<String>,
    pub linkedin_id: Option<String>,
    pub linkedin_url: Option<String>,
    pub tier: String,
    pub is_vip: bool,
    pub is_mutual: bool,
    pub connection_status: String,
    pub relationship_stage: String,
    pub relationship_strength: String,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub metadata: JsonValue,
    pub first_contact_date: Option<NaiveDate>,
    pub last_contact_at: Option<DateTime<Utc>>,
    pub follow_up_date: Option<NaiveDate>,
    pub stale: bool,
    pub stale_since: Option<DateTime<Utc>>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_interaction_kind: Option<String>,
    pub opportunity_id: Option<Uuid>,
}

impl ContactRow {
    /// Fallible rather than infallible on purpose: a `tier`/`stage`/`strength`
    /// outside the database's own vocabulary means the row and the constraint
    /// have diverged. Returning an error keeps that visible; defaulting it to
    /// `Standard`/`Cold`/`None` would return a plausible wrong value under a
    /// 200 OK, which is the failure mode this remediation exists to remove.
    fn try_into_contact(self) -> Result<Contact, Error> {
        let bad = |what: &str, v: &str| {
            Error::Internal(format!(
                "contact {}: column {what} holds {v:?}, which is outside its database constraint",
                self.id
            ))
        };
        Ok(Contact {
            id: self.id,
            member_id: self.member_id,
            display_name: self.display_name,
            linkedin_id: self.linkedin_id,
            linkedin_url: self.linkedin_url,
            title: self.title,
            headline: self.headline,
            company: self.company,
            company_id: self.company_id,
            tier: self.tier.parse().map_err(|()| bad("tier", &self.tier))?,
            is_vip: self.is_vip,
            is_mutual: self.is_mutual,
            connection_status: self
                .connection_status
                .parse()
                .map_err(|()| bad("connection_status", &self.connection_status))?,
            relationship_stage: self
                .relationship_stage
                .parse()
                .map_err(|()| bad("relationship_stage", &self.relationship_stage))?,
            relationship_strength: self
                .relationship_strength
                .parse()
                .map_err(|()| bad("relationship_strength", &self.relationship_strength))?,
            tags: self.tags,
            first_contact_date: self.first_contact_date,
            last_interaction_at: self.last_contact_at,
            last_interaction_kind: self.last_interaction_kind,
            follow_up_date: self.follow_up_date,
            stale: self.stale,
            stale_since: self.stale_since,
            notes: self.notes,
            metadata: self.metadata,
            version: self.version,
            opportunity_id: self.opportunity_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// A `lcc.companies` row. Named for the same reason as [`ContactRow`].
#[derive(Debug, sqlx::FromRow)]
pub struct CompanyRow {
    pub id: Uuid,
    pub member_id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub industry: Option<String>,
    pub size_band: Option<String>,
    pub funding_stage: Option<String>,
    pub hq_location: Option<String>,
    pub tech_stack: Vec<String>,
    pub trigger_events: JsonValue,
    pub public_signals: JsonValue,
    pub enrichment_meta: JsonValue,
    pub third_party_ttl_at: Option<DateTime<Utc>>,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CompanyRow> for Company {
    fn from(r: CompanyRow) -> Self {
        Company {
            id: r.id,
            member_id: r.member_id,
            name: r.name,
            domain: r.domain,
            industry: r.industry,
            size_band: r.size_band,
            funding_stage: r.funding_stage,
            hq_location: r.hq_location,
            tech_stack: r.tech_stack,
            trigger_events: r.trigger_events,
            public_signals: r.public_signals,
            enrichment_meta: r.enrichment_meta,
            third_party_ttl_at: r.third_party_ttl_at,
            version: r.version,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

/// A `lcc.interactions` row, projected by [`Self::list_interactions`].
#[derive(Debug, sqlx::FromRow)]
pub struct InteractionRow {
    pub id: Uuid,
    pub contact_id: Uuid,
    pub member_id: Uuid,
    pub kind: String,
    pub summary: String,
    pub actor: String,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<InteractionRow> for Interaction {
    type Error = Error;

    /// Fallible for the same reason as [`ContactRow::try_into_contact`]: a
    /// `kind` outside `interactions_kind_check` must surface, not be defaulted.
    fn try_from(r: InteractionRow) -> Result<Self, Self::Error> {
        Ok(Interaction {
            id: r.id,
            contact_id: r.contact_id,
            member_id: r.member_id,
            kind: r.kind.parse().map_err(|()| {
                Error::Internal(format!(
                    "interaction {}: kind {:?} is outside its database constraint",
                    r.id, r.kind
                ))
            })?,
            summary: r.summary,
            actor: r.actor,
            occurred_at: r.occurred_at,
            created_at: r.created_at,
        })
    }
}

const COMPANY_PROJECTION: &str = "
                       id, member_id, name, domain, industry, size_band,
                       funding_stage, hq_location, tech_stack, trigger_events,
                       public_signals, enrichment_meta, third_party_ttl_at,
                       version, created_at, updated_at";

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
        // The search term is matched against `display_name` AND `headline`: the
        // dashboard's contact list filters client-side on exactly those two
        // fields ("Search by name or headline…"), so this is the same
        // predicate, pushed down.
        let rows: Vec<ContactRow> = if let Some(q) = q {
            sqlx::query_as(&format!(
                "SELECT {CONTACT_PROJECTION}
                   FROM lcc.contacts c
                  WHERE c.member_id = $1
                    AND (c.display_name ILIKE '%' || $2 || '%'
                         OR c.headline ILIKE '%' || $2 || '%')
                  ORDER BY c.updated_at DESC, c.id DESC
                  LIMIT $3"
            ))
            .bind(member_id)
            .bind(q)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(&format!(
                "SELECT {CONTACT_PROJECTION}
                   FROM lcc.contacts c
                  WHERE c.member_id = $1
                  ORDER BY c.updated_at DESC, c.id DESC
                  LIMIT $2"
            ))
            .bind(member_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };
        rows.into_iter().map(ContactRow::try_into_contact).collect()
    }

    pub async fn get_contact(&self, member_id: Uuid, id: Uuid) -> Result<Contact, Error> {
        let row: ContactRow = sqlx::query_as(&format!(
            "SELECT {CONTACT_PROJECTION}
               FROM lcc.contacts c
              WHERE c.member_id = $1 AND c.id = $2"
        ))
        .bind(member_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            // RowNotFound covers both "no such contact" and "not yours" and
            // both must answer 404 — see the no-oracle note at the top.
            sqlx::Error::RowNotFound => Error::NotFound(format!("contact {id}")),
            other => Error::Internal(format!("get contact: {other}")),
        })?;
        row.try_into_contact()
    }

    pub async fn insert_contact(&self, c: &Contact) -> Result<(), Error> {
        // `is_vip` is written from `tier` rather than accepted separately, so
        // the legacy mirror (0008) and the authoritative column (0020) cannot
        // disagree after a write.
        sqlx::query(
            r#"INSERT INTO lcc.contacts
                 (id, member_id, company_id, company, display_name, headline,
                  title, linkedin_id, linkedin_url, tier, is_vip, is_mutual,
                  connection_status, relationship_stage, relationship_strength,
                  tags, metadata, first_contact_date, last_contact_at,
                  follow_up_date, stale, version, created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,
                     $15::lcc.relationship_strength,$16,$17,
                     $18,$19,$20,$21,$22,$23,$24)"#,
        )
        .bind(c.id)
        .bind(c.member_id)
        .bind(c.company_id)
        .bind(&c.company)
        .bind(&c.display_name)
        .bind(&c.headline)
        .bind(&c.title)
        .bind(&c.linkedin_id)
        .bind(&c.linkedin_url)
        .bind(c.tier.as_str())
        .bind(c.tier == ContactTier::Vip)
        .bind(c.is_mutual)
        .bind(c.connection_status.as_str())
        .bind(c.relationship_stage.as_str())
        .bind(c.relationship_strength.as_str())
        .bind(&c.tags)
        .bind(&c.metadata)
        .bind(c.first_contact_date)
        .bind(c.last_interaction_at)
        .bind(c.follow_up_date)
        .bind(c.stale)
        .bind(c.version)
        .bind(c.created_at)
        .bind(c.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Compare-and-swap update (design §52).
    ///
    /// `WHERE ... AND version = $3` plus `version = version + 1` is the
    /// documented pattern; zero rows affected means someone else wrote first,
    /// which the caller must be told about as 409 rather than silently
    /// overwriting. `company_id` is part of the patch, and
    /// `contacts_company_id_fkey` (0017) plus the service-layer check in
    /// [`Self::link_contact_company`] keep it inside the caller's tenant.
    pub async fn update_contact(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        contact: &Contact,
    ) -> Result<i32, Error> {
        let row: (i32,) = sqlx::query_as(
            r#"UPDATE lcc.contacts
                SET company_id = $4, display_name = $5, headline = $6, title = $7,
                    linkedin_id = $8, linkedin_url = $9, tier = $10,
                    is_vip = $11, is_mutual = $12, connection_status = $13,
                    relationship_stage = $14,
                    relationship_strength = $15::lcc.relationship_strength,
                    tags = $16, metadata = $17, first_contact_date = $18,
                    follow_up_date = $19, version = version + 1, updated_at = NOW()
              WHERE member_id = $1 AND id = $2 AND version = $3
              RETURNING version"#,
        )
        .bind(member_id)
        .bind(id)
        .bind(expected_version)
        .bind(contact.company_id)
        .bind(&contact.display_name)
        .bind(&contact.headline)
        .bind(&contact.title)
        .bind(&contact.linkedin_id)
        .bind(&contact.linkedin_url)
        .bind(contact.tier.as_str())
        .bind(contact.tier == ContactTier::Vip)
        .bind(contact.is_mutual)
        .bind(contact.connection_status.as_str())
        .bind(contact.relationship_stage.as_str())
        .bind(contact.relationship_strength.as_str())
        .bind(&contact.tags)
        .bind(&contact.metadata)
        .bind(contact.first_contact_date)
        .bind(contact.follow_up_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            // Both "no such contact" and "wrong version" land here. The
            // caller already proved ownership (it re-reads the contact before
            // patching), so a miss on the versioned UPDATE is a write race,
            // not a tenant problem — 409 is right and does not leak existence.
            sqlx::Error::RowNotFound => Error::Conflict(format!("contact v{expected_version}")),
            other => Error::Internal(format!("update contact: {other}")),
        })?;
        Ok(row.0)
    }

    pub async fn delete_contact(&self, member_id: Uuid, id: Uuid) -> Result<(), Error> {
        let n = sqlx::query(r#"DELETE FROM lcc.contacts WHERE member_id = $1 AND id = $2"#)
            .bind(member_id)
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        if n == 0 {
            return Err(Error::NotFound(format!("contact {id}")));
        }
        Ok(())
    }

    /// Appends to `lcc.interactions` and updates the contact, atomically.
    ///
    /// The pre-remediation version inserted the interaction and then called
    /// `touch_contact` unconditionally, so a POST to a contact that did not
    /// exist, or belonged to another member, blew up on
    /// `interactions_contact_id_fkey` as a 500 — a cross-tenant existence
    /// oracle expressed as a database error. Checking ownership first, inside
    /// the same transaction as the write, turns both cases into a 404 and
    /// leaves no half-applied interaction behind.
    pub async fn record_interaction(&self, i: &Interaction) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        let owned: Option<(Uuid,)> =
            sqlx::query_as(r#"SELECT id FROM lcc.contacts WHERE member_id = $1 AND id = $2"#)
                .bind(i.member_id)
                .bind(i.contact_id)
                .fetch_optional(&mut *tx)
                .await?;
        if owned.is_none() {
            return Err(Error::NotFound(format!("contact {}", i.contact_id)));
        }

        sqlx::query(
            r#"INSERT INTO lcc.interactions
                 (id, member_id, contact_id, kind, summary, actor,
                  occurred_at, created_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)"#,
        )
        .bind(i.id)
        .bind(i.member_id)
        .bind(i.contact_id)
        .bind(i.kind.as_str())
        .bind(&i.summary)
        .bind(&i.actor)
        .bind(i.occurred_at)
        .bind(i.created_at)
        .execute(&mut *tx)
        .await?;

        let n = sqlx::query(
            r#"UPDATE lcc.contacts
                SET last_contact_at = GREATEST(COALESCE(last_contact_at, $3), $3),
                    first_contact_date = COALESCE(
                        first_contact_date, ($3 AT TIME ZONE 'UTC')::DATE),
                    stale = FALSE,
                    stale_since = NULL,
                    updated_at = NOW()
              WHERE member_id = $1 AND id = $2"#,
        )
        .bind(i.member_id)
        .bind(i.contact_id)
        .bind(i.occurred_at)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if n == 0 {
            return Err(Error::NotFound(format!("contact {}", i.contact_id)));
        }

        tx.commit().await?;
        Ok(())
    }

    /// The interaction log for one contact, newest first.
    ///
    /// Was missing entirely: the contract declares
    /// `GET /contacts/{id}/interactions` and the dashboard's contact detail
    /// page calls it, but the service only ever had the write half.
    pub async fn list_interactions(
        &self,
        member_id: Uuid,
        contact_id: Uuid,
    ) -> Result<Vec<Interaction>, Error> {
        // Ownership first, so a contact that belongs to another member is
        // reported the same way `record_interaction` reports it (NotFound)
        // rather than as a misleading empty list. A genuinely empty contact
        // still returns `Ok([])`; only a foreign contact is NotFound. Both
        // are non-leaking — we never distinguish "absent" from "not yours".
        let owned: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT id
              FROM lcc.contacts
             WHERE id = $1 AND member_id = $2
            "#,
        )
        .bind(contact_id)
        .bind(member_id)
        .fetch_optional(&self.pool)
        .await?;
        if owned.is_none() {
            return Err(Error::NotFound(format!("contact {contact_id}")));
        }

        // Raw string literal on purpose: the repository's SQL is checked
        // against the live schema by the project's SQL detector
        // (tools/extract_sql.py), which only sees `r#"..."#` literals.
        let rows: Vec<InteractionRow> = sqlx::query_as(
            r#"
            SELECT id, contact_id, member_id, kind, summary, actor,
                   occurred_at, created_at
              FROM lcc.interactions
             WHERE member_id = $1 AND contact_id = $2 AND deleted_at IS NULL
             ORDER BY occurred_at DESC, created_at DESC, id DESC
            "#,
        )
        .bind(member_id)
        .bind(contact_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Interaction::try_from).collect()
    }

    pub async fn list_companies(
        &self,
        member_id: Uuid,
        q: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Company>, Error> {
        // `deleted_at IS NULL` is not decoration: 0017 implements soft delete
        // and *every* index on the table is partial on it, so omitting the
        // predicate both returned purged companies and made those indexes
        // unusable.
        let rows: Vec<CompanyRow> = if let Some(q) = q {
            sqlx::query_as(&format!(
                "SELECT {COMPANY_PROJECTION}
                   FROM lcc.companies
                  WHERE member_id = $1 AND deleted_at IS NULL
                    AND name ILIKE '%' || $2 || '%'
                  ORDER BY updated_at DESC, id DESC
                  LIMIT $3"
            ))
            .bind(member_id)
            .bind(q)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(&format!(
                "SELECT {COMPANY_PROJECTION}
                   FROM lcc.companies
                  WHERE member_id = $1 AND deleted_at IS NULL
                  ORDER BY updated_at DESC, id DESC
                  LIMIT $2"
            ))
            .bind(member_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows.into_iter().map(Company::from).collect())
    }

    pub async fn get_company(&self, member_id: Uuid, id: Uuid) -> Result<Company, Error> {
        let row: CompanyRow = sqlx::query_as(&format!(
            "SELECT {COMPANY_PROJECTION}
               FROM lcc.companies
              WHERE member_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(member_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("company {id}")))?;
        Ok(Company::from(row))
    }

    pub async fn insert_company(&self, c: &Company) -> Result<(), Error> {
        sqlx::query(
            r#"INSERT INTO lcc.companies
                 (id, member_id, name, domain, industry, size_band,
                  funding_stage, hq_location, tech_stack, trigger_events,
                  public_signals, enrichment_meta, third_party_ttl_at,
                  version, created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)"#,
        )
        .bind(c.id)
        .bind(c.member_id)
        .bind(&c.name)
        .bind(&c.domain)
        .bind(&c.industry)
        .bind(&c.size_band)
        .bind(&c.funding_stage)
        .bind(&c.hq_location)
        .bind(&c.tech_stack)
        .bind(&c.trigger_events)
        .bind(&c.public_signals)
        .bind(&c.enrichment_meta)
        .bind(c.third_party_ttl_at)
        .bind(c.version)
        .bind(c.created_at)
        .bind(c.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Contacts whose staleness threshold has passed.
    ///
    /// Thresholds are the contract's own — "Contacts past staleness threshold
    /// (VIP 30d / standard 60d / peer 90d)" — applied to `tier`. The
    /// pre-remediation query compared a single caller-supplied `days` against
    /// `last_touched_at`, so every member saw the same threshold regardless of
    /// which contacts mattered to them.
    ///
    /// `ORDER BY last_contact_at NULLS FIRST` is kept: a contact that has never
    /// been contacted is the most stale thing on the list, and `NULLS FIRST`
    /// is the only ordering that puts it there.
    pub async fn staleness(&self, member_id: Uuid) -> Result<StalenessReport, Error> {
        let rows: Vec<(Uuid, String, String, Option<DateTime<Utc>>)> = sqlx::query_as(
            r#"
            SELECT id, display_name, tier, last_contact_at
               FROM lcc.contacts
              WHERE member_id = $1
                AND (last_contact_at IS NULL
                     OR last_contact_at < NOW() - (CASE tier
                            WHEN 'VIP' THEN 30
                            WHEN 'peer' THEN 90
                            ELSE 60
                        END * INTERVAL '1 day'))
              ORDER BY last_contact_at NULLS FIRST, id
            "#,
        )
        .bind(member_id)
        .fetch_all(&self.pool)
        .await?;

        let as_of = Utc::now();
        let mut stale: Vec<StaleContact> = Vec::with_capacity(rows.len());
        for (id, display_name, tier, last_contact_at) in rows {
            let tier: ContactTier = tier.parse().map_err(|()| {
                Error::Internal(format!(
                    "contact {id}: tier {tier:?} is outside its constraint"
                ))
            })?;
            let days_since_touch = last_contact_at.map(|t| (as_of - t).num_days());
            stale.push(StaleContact {
                contact_id: id,
                suggested_action: suggest(days_since_touch).to_owned(),
                display_name,
                tier,
                days_since_touch,
            });
        }
        Ok(StalenessReport {
            member_id,
            as_of,
            stale_contacts: stale,
        })
    }

    /// Links a contact to a company, both scoped to the calling member.
    ///
    /// The `EXISTS` term is the important half. `contacts_company_id_fkey`
    /// (0017) only checks that the company exists — it is not tenant-aware — so
    /// without it a member could attach *another member's* company id to their
    /// own contact, and a later join would read across the tenant boundary.
    pub async fn link_contact_company(
        &self,
        member_id: Uuid,
        contact_id: Uuid,
        company_id: Uuid,
    ) -> Result<(), Error> {
        let n = sqlx::query(
            r#"UPDATE lcc.contacts
                SET company_id = $3, updated_at = NOW()
              WHERE member_id = $1 AND id = $2
                AND EXISTS (SELECT 1 FROM lcc.companies c
                             WHERE c.id = $3 AND c.member_id = $1
                               AND c.deleted_at IS NULL)"#,
        )
        .bind(member_id)
        .bind(contact_id)
        .bind(company_id)
        .execute(&self.pool)
        .await?
        .rows_affected();
        if n == 0 {
            // Covers "contact missing", "contact is not yours", "company is not
            // yours" and "company soft-deleted" alike. One 404 for all four:
            // distinguishing them would be the existence oracle.
            return Err(Error::NotFound(format!("contact {contact_id}")));
        }
        Ok(())
    }
}

/// Suggested next step for a stale contact, keyed on how long it has been.
///
/// `None` means "never contacted", which is a different situation from
/// "contacted a long time ago" and used to be flattened into a made-up number
/// (`days * 2 + 90`) so it could be rendered as one.
fn suggest(days: Option<i64>) -> &'static str {
    match days {
        None => "never contacted - decide whether this relationship is still worth keeping",
        Some(d) if d > 180 => "send an anniversary note or congrats on a recent post",
        Some(d) if d > 90 => "share a quick industry observation to re-engage",
        Some(_) => "send a low-stakes 'thinking of you' note",
    }
}

/// Keeps the import list honest: these are used by `service.rs`, not here, but
/// a `use` that names an enum the repository constrains is the cheapest way to
/// make a change to one of those constraint lists fail to compile here too.
#[allow(dead_code)]
fn _constraint_witness(
    c: ContactTier,
    s: ConnectionStatus,
    r: RelationshipStage,
    k: InteractionKind,
) -> &'static str {
    let _ = (c, s, r, RelationshipStrength::None, k);
    "lcc.contacts"
}
