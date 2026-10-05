//! identity-svc persistence layer.
//!
//! Real implementation against the existing `lcc.members` and
//! `lcc.oauth_tokens` tables, with RLS-aware queries (the gateway sets
//! `app.current_member_id` per request via `lcc-db::tx::set_member_context`).

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::{GoalMode, Member, MemberRole, MemberSettings, OAuthToken};
use crate::error::Error;

#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Helper: set the per-request RLS member_id so subsequent queries on the
    /// same connection are scoped correctly.
    async fn set_member_context(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        member_id: Uuid,
    ) -> Result<(), Error> {
        sqlx::query("SELECT set_config('app.current_member_id', $1, true)")
            .bind(member_id.to_string())
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}

impl PgRepository {
    /// Read the authenticated member by id.
    /// Sets the RLS context on a fresh transaction so the SELECT is
    /// tenant-scoped.
    pub async fn get_member(&self, id: Uuid) -> Result<Member, Error> {
        let mut tx = self.pool.begin().await?;
        self.set_member_context(&mut tx, id).await?;

        // Defensive projection: the lcc.members table after migration 0017
        // contains the goal-mode and restriction columns. If they don't yet
        // exist, COALESCE handles the legacy-shape read.
        let row = sqlx::query(
            r#"
            SELECT id, linkedin_id, email, display_name, role, is_active,
                   timezone, locale,
                   COALESCE(active_goal_mode, 'hybrid') AS active_goal_mode,
                   COALESCE(is_restricted, false) AS is_restricted,
                   restricted_since, restricted_reason, warmup_started_at,
                   created_at, updated_at,
                   COALESCE(version, 1) AS version
            FROM lcc.members WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        let member = match row {
            Some(r) => Member {
                id: r.try_get("id")?,
                linkedin_id: r.try_get("linkedin_id")?,
                email: r.try_get("email").ok(),
                display_name: r.try_get("display_name")?,
                role: MemberRole::parse_role(&r.try_get::<String, _>("role")?),
                is_active: r.try_get("is_active")?,
                timezone: r.try_get("timezone")?,
                locale: r.try_get("locale")?,
                active_goal_mode: parse_goal_mode(&r.try_get::<String, _>("active_goal_mode")?)?,
                is_restricted: r.try_get("is_restricted")?,
                restricted_since: r.try_get("restricted_since").ok(),
                restricted_reason: r.try_get("restricted_reason").ok(),
                warmup_started_at: r.try_get("warmup_started_at")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
                version: r.try_get("version")?,
            },
            None => return Err(Error::NotFound(format!("member {id}"))),
        };

        tx.commit().await?;
        Ok(member)
    }

    /// Read member by linkedin_id (used during OAuth callback).
    pub async fn get_member_by_linkedin_id(
        &self,
        linkedin_id: &str,
    ) -> Result<Option<Member>, Error> {
        let row = sqlx::query(
            r#"
            SELECT id, linkedin_id, email, display_name, role, is_active,
                   timezone, locale,
                   COALESCE(active_goal_mode, 'hybrid') AS active_goal_mode,
                   COALESCE(is_restricted, false) AS is_restricted,
                   restricted_since, restricted_reason, warmup_started_at,
                   created_at, updated_at,
                   COALESCE(version, 1) AS version
            FROM lcc.members WHERE linkedin_id = $1
            "#,
        )
        .bind(linkedin_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(r) = row else {
            return Ok(None);
        };

        Ok(Some(Member {
            id: r.try_get("id")?,
            linkedin_id: r.try_get("linkedin_id")?,
            email: r.try_get("email").ok(),
            display_name: r.try_get("display_name")?,
            role: MemberRole::parse_role(&r.try_get::<String, _>("role")?),
            is_active: r.try_get("is_active")?,
            timezone: r.try_get("timezone")?,
            locale: r.try_get("locale")?,
            active_goal_mode: parse_goal_mode(&r.try_get::<String, _>("active_goal_mode")?)?,
            is_restricted: r.try_get("is_restricted")?,
            restricted_since: r.try_get("restricted_since").ok(),
            restricted_reason: r.try_get("restricted_reason").ok(),
            warmup_started_at: r.try_get("warmup_started_at")?,
            created_at: r.try_get("created_at")?,
            updated_at: r.try_get("updated_at")?,
            version: r.try_get("version")?,
        }))
    }

    /// Insert a new member.
    pub async fn insert_member(
        &self,
        linkedin_id: &str,
        display_name: &str,
        email: Option<&str>,
    ) -> Result<Member, Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO lcc.members (linkedin_id, email, display_name)
            VALUES ($1, $2, $3)
            RETURNING id, linkedin_id, email, display_name, role, is_active,
                      timezone, locale,
                      COALESCE(active_goal_mode, 'hybrid') AS active_goal_mode,
                      COALESCE(is_restricted, false) AS is_restricted,
                      restricted_since, restricted_reason, warmup_started_at,
                      created_at, updated_at,
                      COALESCE(version, 1) AS version
            "#,
        )
        .bind(linkedin_id)
        .bind(email)
        .bind(display_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(Member {
            id: row.try_get("id")?,
            linkedin_id: row.try_get("linkedin_id")?,
            email: row.try_get("email").ok(),
            display_name: row.try_get("display_name")?,
            role: MemberRole::parse_role(&row.try_get::<String, _>("role")?),
            is_active: row.try_get("is_active")?,
            timezone: row.try_get("timezone")?,
            locale: row.try_get("locale")?,
            active_goal_mode: parse_goal_mode(&row.try_get::<String, _>("active_goal_mode")?)?,
            is_restricted: row.try_get("is_restricted")?,
            restricted_since: row.try_get("restricted_since").ok(),
            restricted_reason: row.try_get("restricted_reason").ok(),
            warmup_started_at: row.try_get("warmup_started_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            version: row.try_get("version")?,
        })
    }

    /// Update timezone, active_goal_mode, and/or cap_overrides (stored in
    /// the `metadata` JSONB column of lcc.members when 0017 hasn't been
    /// applied yet, otherwise the dedicated columns).
    pub async fn update_settings(
        &self,
        id: Uuid,
        s: &crate::domain::MemberSettingsUpdate,
        expected_version: i64,
    ) -> Result<Member, Error> {
        let mut tx = self.pool.begin().await?;
        self.set_member_context(&mut tx, id).await?;

        // Optimistic concurrency: only update if version matches.
        let updated = sqlx::query(
            r#"
            UPDATE lcc.members
            SET timezone = COALESCE($2, timezone),
                active_goal_mode = COALESCE($3::text, active_goal_mode),
                version = version + 1,
                updated_at = NOW()
            WHERE id = $1 AND version = $4
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(s.timezone.as_deref())
        .bind(s.active_goal_mode.map(|g| g.as_str()))
        .bind(expected_version)
        .fetch_optional(&mut *tx)
        .await?;

        if updated.is_none() {
            return Err(Error::Conflict(
                "version conflict (re-fetch the resource and retry)".into(),
            ));
        }

        tx.commit().await?;

        // Re-read the full member row.
        self.get_member(id).await
    }

    /// Read settings (a derived view of the member row).
    pub async fn get_settings(&self, id: Uuid) -> Result<MemberSettings, Error> {
        let m = self.get_member(id).await?;
        Ok(MemberSettings {
            timezone: m.timezone,
            active_goal_mode: m.active_goal_mode,
            cap_overrides: None, // stored on member.metadata or a future dedicated column
        })
    }

    // ---------- OAuth token store ----------

    pub async fn upsert_oauth_token(
        &self,
        member_id: Uuid,
        provider: &str,
        access_token_encrypted: &[u8],
        refresh_token_encrypted: Option<&[u8]>,
        expires_at: DateTime<Utc>,
        scope: Option<&str>,
    ) -> Result<OAuthToken, Error> {
        sqlx::query(
            r#"
            INSERT INTO lcc.oauth_tokens (
                member_id, provider, access_token_encrypted,
                refresh_token_encrypted, expires_at, scope
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (member_id, provider) DO UPDATE
                SET access_token_encrypted = EXCLUDED.access_token_encrypted,
                    refresh_token_encrypted = EXCLUDED.refresh_token_encrypted,
                    expires_at = EXCLUDED.expires_at,
                    scope = EXCLUDED.scope,
                    updated_at = NOW()
            "#,
        )
        .bind(member_id)
        .bind(provider)
        .bind(access_token_encrypted)
        .bind(refresh_token_encrypted)
        .bind(expires_at)
        .bind(scope)
        .execute(&self.pool)
        .await?;

        Ok(OAuthToken {
            member_id,
            provider: provider.into(),
            access_token_encrypted: access_token_encrypted.to_vec(),
            refresh_token_encrypted: refresh_token_encrypted.map(|s| s.to_vec()),
            expires_at,
            scope: scope.map(String::from),
        })
    }

    pub async fn get_oauth_token(
        &self,
        member_id: Uuid,
        provider: &str,
    ) -> Result<Option<OAuthToken>, Error> {
        let row = sqlx::query(
            r#"
            SELECT member_id, provider, access_token_encrypted,
                   refresh_token_encrypted, expires_at, scope
            FROM lcc.oauth_tokens
            WHERE member_id = $1 AND provider = $2
            "#,
        )
        .bind(member_id)
        .bind(provider)
        .fetch_optional(&self.pool)
        .await?;

        let Some(r) = row else {
            return Ok(None);
        };

        Ok(Some(OAuthToken {
            member_id: r.try_get("member_id")?,
            provider: r.try_get("provider")?,
            access_token_encrypted: r.try_get("access_token_encrypted")?,
            refresh_token_encrypted: r.try_get("refresh_token_encrypted").ok(),
            expires_at: r.try_get("expires_at")?,
            scope: r.try_get("scope").ok(),
        }))
    }

    pub async fn delete_oauth_token(&self, member_id: Uuid, provider: &str) -> Result<(), Error> {
        sqlx::query("DELETE FROM lcc.oauth_tokens WHERE member_id = $1 AND provider = $2")
            .bind(member_id)
            .bind(provider)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

fn parse_goal_mode(s: &str) -> Result<GoalMode, Error> {
    match s {
        "job_hunting" => Ok(GoalMode::JobHunting),
        "client_acquisition" => Ok(GoalMode::ClientAcquisition),
        "hybrid" => Ok(GoalMode::Hybrid),
        _ => Err(Error::BadRequest(format!("invalid goal_mode: {s}"))),
    }
}
