//! Content-svc service — lifecycle + quality loop + scheduling.

use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::domain::{
    ContentItem, ContentKind, ContentMetrics, ContentState, QualityCheckResult, Schedule,
};
use crate::error::Error;
use crate::repository::PgRepository;

/// Max iterations of the auto-fix quality loop before content is blocked.
pub const MAX_QUALITY_LOOPS: i32 = 3;

/// The fields needed to create a content item.
///
/// Grouped into a struct rather than passed as eight positional parameters:
/// four adjacent `&str` arguments are easy to transpose at a call site, and
/// the list was over the workspace's too-many-arguments threshold.
#[derive(Debug, Clone)]
pub struct NewContentItem<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub kind: ContentKind,
    pub topic: &'a str,
    pub voice_style_kb_id: Option<Uuid>,
    pub pinned_kb_ids: Vec<Uuid>,
    pub idempotency_key: Option<String>,
}

pub struct Service {
    repo: PgRepository,
    redis: deadpool_redis::Pool,
}

impl Service {
    pub fn new(repo: PgRepository, redis: deadpool_redis::Pool) -> Self {
        Self { repo, redis }
    }
    pub fn repo(&self) -> &PgRepository {
        &self.repo
    }

    pub async fn list(
        &self,
        member_id: Uuid,
        state: Option<ContentState>,
        limit: i64,
    ) -> Result<Vec<ContentItem>, Error> {
        self.repo.list(member_id, state, limit).await
    }

    pub async fn get(&self, member_id: Uuid, id: Uuid) -> Result<ContentItem, Error> {
        self.repo.get(member_id, id).await
    }

    pub async fn create(
        &self,
        member_id: Uuid,
        new: NewContentItem<'_>,
    ) -> Result<ContentItem, Error> {
        let NewContentItem {
            title,
            body,
            kind,
            topic,
            voice_style_kb_id,
            pinned_kb_ids,
            idempotency_key,
        } = new;

        // Idempotency: if a key is supplied and a row already exists, return it.
        if let Some(key) = &idempotency_key {
            if let Ok(existing) = self.repo.get_by_idempotency_key(member_id, key).await {
                return Ok(existing);
            }
        }
        let now = Utc::now();
        let hash = Some(hash_body(body));
        let c = ContentItem {
            id: Uuid::new_v4(),
            member_id,
            state: ContentState::Idea,
            title: title.into(),
            body: body.into(),
            rendered_body_hash: hash,
            kind,
            topic: topic.into(),
            voice_style_kb_id,
            pinned_kb_ids,
            metrics: ContentMetrics::default(),
            quality_loop_count: 0,
            idempotency_key,
            expected_version: 1,
            version: 1,
            scheduled_at: None,
            published_at: None,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert(&c).await?;
        Ok(c)
    }

    pub async fn update_body(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        body: &str,
    ) -> Result<ContentItem, Error> {
        let hash = hash_body(body);
        let new_version = self
            .repo
            .update_body(member_id, id, expected_version, body, Some(&hash))
            .await?;
        self.repo.get(member_id, id).await.map(|mut c| {
            c.version = new_version;
            c
        })
    }

    pub async fn transition(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        new_state: ContentState,
        scheduled_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<ContentItem, Error> {
        let current = self.repo.get(member_id, id).await?;
        if !current.state.can_transition_to(new_state) {
            return Err(Error::InvalidTransition {
                from: current.state.as_str().into(),
                to: new_state.as_str().into(),
            });
        }
        let new_version = self
            .repo
            .transition(member_id, id, expected_version, new_state, scheduled_at)
            .await?;
        // Side-effects on entering specific states.
        match new_state {
            ContentState::Scheduled => {
                self.publish_audit_event(
                    member_id,
                    id,
                    "content.scheduled",
                    serde_json::json!({
                        "content_id":id,"scheduled_at":scheduled_at
                    }),
                )
                .await;
            }
            ContentState::Published => {
                self.publish_audit_event(
                    member_id,
                    id,
                    "content.published",
                    serde_json::json!({
                        "content_id":id,"at":Utc::now()
                    }),
                )
                .await;
                // Also publish realtime briefing refresh so the dashboard
                // surfaces the new post.
                self.publish_realtime(
                    "content.published",
                    member_id,
                    serde_json::json!({"content_id":id,"topic":current.topic}),
                )
                .await;
            }
            _ => {}
        }
        self.repo.get(member_id, id).await.map(|mut c| {
            c.version = new_version;
            c
        })
    }

    /// Quality loop: run auto-checks; if they fail, try auto-fix; repeat
    /// until either passed, or we hit `MAX_QUALITY_LOOPS`. Each loop is
    /// recorded in `lcc.content_quality_checks` for full traceability.
    pub async fn run_quality_loop(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
    ) -> Result<QualityCheckResult, Error> {
        let mut current = self.repo.get(member_id, id).await?;
        if current.version != expected_version {
            return Err(Error::Conflict(format!(
                "content version {expected_version} not current"
            )));
        }
        for loop_idx in 0..MAX_QUALITY_LOOPS {
            let mut issues = Vec::new();
            let mut auto_fixes = Vec::new();
            // Check 1: body length
            if current.body.len() < 80 {
                issues.push("body too short".into());
            }
            // Check 2: title is non-trivial
            if current.title.trim().len() < 5 {
                issues.push("title too short".into());
            }
            // Check 3: at least one pinned kb ref
            if current.pinned_kb_ids.is_empty() {
                issues.push("no kb grounding refs".into());
            }

            // Auto-fixes (very simple here): title-case, body trim, set default
            // voice_style_kb_id if missing.
            let mut next = current.clone();
            if issues.iter().any(|i| i == "title too short") {
                next.title = format!("{} — note", next.title);
                auto_fixes.push("title suffixed".into());
            }
            if issues.iter().any(|i| i == "body too short") {
                next.body = format!("{} (auto-extended)", next.body);
                auto_fixes.push("body auto-extended".into());
            }
            if current.pinned_kb_ids.is_empty() {
                next.pinned_kb_ids = vec![current.voice_style_kb_id.unwrap_or_else(Uuid::nil)];
                auto_fixes.push("added default voice-style kb ref".into());
            }

            let passed = issues.is_empty();
            let result = QualityCheckResult {
                passed,
                r#loop: loop_idx,
                issues: issues.clone(),
                auto_fixes,
                evaluated_at: Utc::now(),
            };
            self.repo
                .record_quality_check(member_id, id, &result)
                .await?;
            self.repo.increment_quality_loop(member_id, id).await?;
            if passed {
                return Ok(result);
            }
            // Apply auto-fixes and re-evaluate next iteration.
            let new_body_hash = hash_body(&next.body);
            let new_version = self
                .repo
                .update_body(
                    member_id,
                    id,
                    current.version,
                    &next.body,
                    Some(&new_body_hash),
                )
                .await?;
            current = self.repo.get(member_id, id).await?;
            current.version = new_version;
        }
        // Exhausted: block the item.
        let _ = self
            .repo
            .transition(member_id, id, current.version, ContentState::Blocked, None)
            .await?;
        Err(Error::QualityLoopExhausted {
            loop_count: MAX_QUALITY_LOOPS,
        })
    }

    pub async fn schedule(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        schedule: Schedule,
    ) -> Result<ContentItem, Error> {
        let scheduled_at: chrono::DateTime<Utc> = schedule.scheduled_at.and_utc();
        self.transition(
            member_id,
            id,
            expected_version,
            ContentState::Scheduled,
            Some(scheduled_at),
        )
        .await
    }

    pub async fn delete(&self, member_id: Uuid, id: Uuid) -> Result<(), Error> {
        self.repo.delete(member_id, id).await
    }

    async fn publish_audit_event(
        &self,
        member_id: Uuid,
        resource_id: Uuid,
        action: &str,
        payload: serde_json::Value,
    ) {
        let event = serde_json::json!({
            "event_id": Uuid::new_v4(),
            "event_name": action,
            "occurred_at": Utc::now(),
            "member_id": member_id,
            "producer_service": "content-svc",
            "actor": "content-svc",
            "action": action,
            "resource_type": "content_item",
            "resource_id": resource_id,
            "payload": payload,
        });
        if let Ok(mut conn) = self.redis.get().await {
            let _ = redis::cmd("XADD")
                .arg("lcc:audit:events")
                .arg("*")
                .arg("event")
                .arg(event.to_string())
                .query_async::<String>(&mut conn)
                .await;
        }
    }

    async fn publish_realtime(
        &self,
        event_name: &str,
        member_id: Uuid,
        payload: serde_json::Value,
    ) {
        let mut conn = match self.redis.get().await {
            Ok(c) => c,
            Err(_) => return,
        };
        let envelope = serde_json::json!({
            "event_id": Uuid::new_v4(),
            "event_name": event_name,
            "occurred_at": Utc::now(),
            "member_id": member_id,
            "producer_service": "content-svc",
            "payload": payload,
        });
        let _ = redis::cmd("XADD")
            .arg("lcc:realtime:events")
            .arg("*")
            .arg("envelope")
            .arg(envelope.to_string())
            .query_async::<String>(&mut conn)
            .await;
    }
}

impl PgRepository {
    /// Idempotency lookup — searches by the canonical idempotency_key column.
    pub async fn get_by_idempotency_key(
        &self,
        member_id: Uuid,
        key: &str,
    ) -> Result<ContentItem, Error> {
        let row = sqlx::query_as::<_, crate::repository::ContentRow>(
            r#"
            SELECT id, member_id, state::TEXT, title, body, rendered_body_hash,
                   kind::TEXT, topic, voice_style_kb_id, pinned_kb_ids,
                   metrics, quality_loop_count, idempotency_key,
                   expected_version, version, scheduled_at, published_at,
                   created_at, updated_at
            FROM lcc.content_items
            WHERE member_id = $1 AND idempotency_key = $2
            "#,
        )
        .bind(member_id)
        .bind(key)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("idempotency key {key}")),
            other => Error::Internal(format!("idempotency: {other}")),
        })?;
        crate::repository::map_row(row)
    }
}

pub(crate) fn hash_body(body: &str) -> String {
    let mut h = Sha256::new();
    h.update(body.as_bytes());
    let digest = h.finalize();
    format!("sha256:{}", hex::encode(digest))
}
