//! Outreach service.

use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::domain::{Sequence, SequenceStatus, SequenceStep, Template, TemplateStep};
use crate::error::Error;
use crate::repository::PgRepository;

pub struct Service {
    repo: PgRepository,
    redis: deadpool_redis::Pool,
}

impl Service {
    pub fn new(repo: PgRepository, redis: deadpool_redis::Pool) -> Self {
        Self { repo, redis }
    }

    pub async fn list_sequences(
        &self,
        member_id: Uuid,
        status: Option<SequenceStatus>,
        limit: i64,
    ) -> Result<Vec<Sequence>, Error> {
        self.repo.list_sequences(member_id, status, limit).await
    }

    pub async fn create_sequence(
        &self,
        member_id: Uuid,
        contact_id: Uuid,
        template_id: Option<Uuid>,
        steps: Vec<TemplateStep>,
    ) -> Result<Sequence, Error> {
        if steps.is_empty() {
            return Err(Error::Validation("steps must be non-empty".into()));
        }
        let now = Utc::now();
        let s = Sequence {
            id: Uuid::new_v4(),
            member_id,
            contact_id,
            template_id,
            status: SequenceStatus::Draft,
            current_step: 0,
            paused_reason: None,
            last_step_sent_at: None,
            version: 1,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert_sequence(&s).await?;
        for (i, tstep) in steps.iter().enumerate() {
            let rendered = hash(&tstep.body);
            let step = SequenceStep {
                id: Uuid::new_v4(),
                sequence_id: s.id,
                step_index: i as i32,
                kind: tstep.kind,
                subject: tstep.subject.clone(),
                body: tstep.body.clone(),
                rendered_body_hash: Some(rendered),
                contact_id,
                scheduled_at: Some(
                    Utc::now().naive_utc() + chrono::Duration::hours(tstep.delay_hours as i64),
                ),
                sent_at: None,
                response_received_at: None,
                updated_at: now,
            };
            self.insert_step(&step).await?;
        }
        Ok(s)
    }

    async fn insert_step(&self, step: &SequenceStep) -> Result<(), Error> {
        sqlx::query(
            r#"INSERT INTO lcc.sequence_steps
                  (id, sequence_id, step_index, kind, subject, body,
                   rendered_body_hash, contact_id, scheduled_at, updated_at)
               VALUES ($1,$2,$3,$4::text,$5,$6,$7,$8,$9,$10)"#,
        )
        .bind(step.id)
        .bind(step.sequence_id)
        .bind(step.step_index)
        .bind(step.kind.as_str())
        .bind(&step.subject)
        .bind(&step.body)
        .bind(&step.rendered_body_hash)
        .bind(step.contact_id)
        .bind(step.scheduled_at)
        .bind(step.updated_at)
        .execute(&self.repo.pool)
        .await?;
        Ok(())
    }

    pub async fn pause(&self, m: Uuid, id: Uuid, v: i32, reason: &str) -> Result<Sequence, Error> {
        let new_v = self
            .repo
            .update_status(m, id, v, SequenceStatus::Paused, Some(reason))
            .await?;
        self.publish(
            "sequence.paused",
            m,
            &serde_json::json!({"sequence_id":id,"reason":reason}),
        )
        .await;
        Ok(Sequence {
            id,
            member_id: m,
            contact_id: Uuid::nil(),
            template_id: None,
            status: SequenceStatus::Paused,
            current_step: 0,
            paused_reason: Some(reason.into()),
            last_step_sent_at: None,
            version: new_v,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn resume(&self, m: Uuid, id: Uuid, v: i32) -> Result<Sequence, Error> {
        let new_v = self
            .repo
            .update_status(m, id, v, SequenceStatus::Active, None)
            .await?;
        self.publish(
            "sequence.resumed",
            m,
            &serde_json::json!({"sequence_id":id}),
        )
        .await;
        Ok(Sequence {
            id,
            member_id: m,
            contact_id: Uuid::nil(),
            template_id: None,
            status: SequenceStatus::Active,
            current_step: 0,
            paused_reason: None,
            last_step_sent_at: None,
            version: new_v,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn mark_step_sent(&self, step_id: Uuid, m: Uuid) -> Result<(), Error> {
        self.repo.mark_step_sent(step_id).await?;
        self.publish(
            "sequence.step.sent",
            m,
            &serde_json::json!({"step_id":step_id}),
        )
        .await;
        Ok(())
    }

    pub async fn mark_step_reply(&self, step_id: Uuid, m: Uuid) -> Result<(), Error> {
        self.repo.record_step_reply(step_id).await?;
        self.publish(
            "sequence.reply_detected",
            m,
            &serde_json::json!({"step_id":step_id}),
        )
        .await;
        Ok(())
    }

    pub async fn list_steps(&self, sequence_id: Uuid) -> Result<Vec<SequenceStep>, Error> {
        self.repo.list_steps(sequence_id).await
    }

    pub async fn create_template(
        &self,
        member_id: Uuid,
        name: &str,
        description: Option<&str>,
        steps: Vec<TemplateStep>,
    ) -> Result<Template, Error> {
        if name.trim().is_empty() {
            return Err(Error::Validation("name required".into()));
        }
        if steps.is_empty() {
            return Err(Error::Validation("steps required".into()));
        }
        let now = Utc::now();
        let t = Template {
            id: Uuid::new_v4(),
            member_id,
            name: name.into(),
            description: description.map(str::to_string),
            steps,
            is_published: false,
            version: 1,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert_template(&t).await?;
        Ok(t)
    }

    pub async fn list_templates(&self, member_id: Uuid) -> Result<Vec<Template>, Error> {
        self.repo.list_templates(member_id).await
    }

    async fn publish(&self, name: &str, m: Uuid, payload: &serde_json::Value) {
        let env = serde_json::json!({
            "event_id": Uuid::new_v4(),
            "event_name": name,
            "occurred_at": Utc::now(),
            "member_id": m,
            "producer_service": "outreach-svc",
            "payload": payload,
        });
        if let Ok(mut conn) = self.redis.get().await {
            let _ = redis::cmd("XADD")
                .arg("lcc:realtime:events")
                .arg("*")
                .arg("envelope")
                .arg(env.to_string())
                .query_async::<String>(&mut conn)
                .await;
        }
    }
}

impl PgRepository {
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

fn hash(body: &str) -> String {
    let mut h = Sha256::new();
    h.update(body.as_bytes());
    let digest = h.finalize();
    format!("sha256:{}", hex::encode(digest))
}
