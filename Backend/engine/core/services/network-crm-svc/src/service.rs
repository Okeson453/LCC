//! network-crm-svc service.

use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::domain::{
    Company, ConnectionStatus, Contact, ContactTier, Interaction, InteractionKind,
    RelationshipStage, RelationshipStrength, StalenessReport,
};
use crate::error::Error;
use crate::repository::PgRepository;

pub struct Service {
    repo: PgRepository,
    #[allow(dead_code)]
    redis: deadpool_redis::Pool,
}

/// The fields a caller may supply when creating a contact.
///
/// A struct rather than nine positional parameters: five of them are
/// `Option<&str>`, so a caller could silently pass `industry` where `linkedin_id`
/// belonged and nothing would stop it.
#[derive(Debug, Clone)]
pub struct NewContact<'a> {
    pub display_name: &'a str,
    pub title: Option<&'a str>,
    pub headline: Option<&'a str>,
    pub linkedin_id: Option<&'a str>,
    pub linkedin_url: Option<&'a str>,
    pub company_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub notes: Option<&'a str>,
    /// Canonical contract `ContactCreate.tier` (default `standard`).
    pub tier: Option<ContactTier>,
    /// Canonical contract `ContactCreate.first_contact_date`.
    pub first_contact_date: Option<NaiveDate>,
}

/// The fields needed to create a company. Same reasoning as [`NewContact`].
#[derive(Debug, Clone)]
pub struct NewCompany<'a> {
    pub name: &'a str,
    pub domain: Option<&'a str>,
    pub industry: Option<&'a str>,
    pub size_band: Option<&'a str>,
    pub funding_stage: Option<&'a str>,
    pub hq_location: Option<&'a str>,
    pub tech_stack: Vec<String>,
    pub trigger_events: JsonValue,
    pub public_signals: JsonValue,
    pub third_party_ttl_at: Option<DateTime<Utc>>,
}

impl Service {
    pub fn new(repo: PgRepository, redis: deadpool_redis::Pool) -> Self {
        Self { repo, redis }
    }
    pub fn repo(&self) -> &PgRepository {
        &self.repo
    }

    pub async fn list_contacts(
        &self,
        member_id: Uuid,
        q: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Contact>, Error> {
        self.repo.list_contacts(member_id, q, limit).await
    }

    pub async fn get_contact(&self, m: Uuid, id: Uuid) -> Result<Contact, Error> {
        self.repo.get_contact(m, id).await
    }

    pub async fn create_contact(
        &self,
        member_id: Uuid,
        new: NewContact<'_>,
    ) -> Result<Contact, Error> {
        let NewContact {
            display_name,
            title,
            headline,
            linkedin_id,
            linkedin_url,
            company_id,
            tags,
            notes,
            tier,
            first_contact_date,
        } = new;

        if display_name.trim().is_empty() {
            return Err(Error::Validation("display_name required".into()));
        }
        if let Some(url) = linkedin_url {
            // `contacts.linkedin_url` is contract-typed `format: uri` and is
            // the profile link the dashboard renders as an anchor. A value
            // that is not an absolute http(s) URL would be persisted and then
            // rendered as a dead link, so it is refused at the boundary.
            if !(url.starts_with("https://") || url.starts_with("http://")) {
                return Err(Error::Validation(
                    "linkedin_url must be an absolute http(s) URL".into(),
                ));
            }
        }

        let now = Utc::now();
        let metadata = metadata_with_note(JsonValue::Object(Default::default()), notes);
        let c = Contact {
            id: Uuid::new_v4(),
            member_id,
            company_id,
            display_name: display_name.into(),
            title: title.map(str::to_string),
            headline: headline.map(str::to_string),
            company: None,
            linkedin_id: linkedin_id.map(str::to_string),
            linkedin_url: linkedin_url.map(str::to_string),
            tier: tier.unwrap_or(ContactTier::Standard),
            is_vip: tier == Some(ContactTier::Vip),
            is_mutual: false,
            connection_status: ConnectionStatus::NotConnected,
            relationship_stage: RelationshipStage::Cold,
            relationship_strength: RelationshipStrength::None,
            tags,
            first_contact_date,
            last_interaction_at: None,
            last_interaction_kind: None,
            follow_up_date: None,
            stale: false,
            stale_since: None,
            notes: notes.map(str::to_string),
            metadata,
            version: 1,
            opportunity_id: None,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert_contact(&c).await?;
        Ok(c)
    }

    pub async fn update_contact(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        patch: ContactPatch,
    ) -> Result<Contact, Error> {
        // Read first: proves the caller owns the row (404 otherwise) and gives
        // the merge below a base to work from.
        let current = self.repo.get_contact(member_id, id).await?;
        // `ContactPatch` uses `Option<Option<T>>` for nullable columns so that
        // "field absent" (keep current) is distinct from "field sent as null"
        // (clear it). `unwrap_or` preserves that distinction; `or` would not,
        // because it cannot tell `Some(None)` from `None`.
        // Read `patch.notes` once: it is consumed both by the merge and by the
        // metadata rewrite, and `ContactPatch` is not `Copy`.
        let note_patch = patch.notes.clone();
        let notes = note_patch.clone().unwrap_or(current.notes.clone());
        let base_metadata = if notes == current.notes {
            current.metadata
        } else {
            // The note lives inside `contacts.metadata`; anything else the
            // caller had stored there survives the rewrite.
            strip_note(&current.metadata)
        };
        let updated = Contact {
            id: current.id,
            member_id: current.member_id,
            company_id: patch.company_id.unwrap_or(current.company_id),
            display_name: patch.display_name.unwrap_or(current.display_name),
            title: patch.title.unwrap_or(current.title),
            headline: patch.headline.unwrap_or(current.headline),
            company: patch.company.unwrap_or(current.company),
            linkedin_id: patch.linkedin_id.unwrap_or(current.linkedin_id),
            linkedin_url: patch.linkedin_url.unwrap_or(current.linkedin_url),
            tier: patch.tier.unwrap_or(current.tier),
            // Derived from the authoritative column, never taken from the
            // request body, so the 0008 mirror cannot drift from 0020's `tier`.
            is_vip: patch.tier.unwrap_or(current.tier) == ContactTier::Vip,
            is_mutual: patch.is_mutual.unwrap_or(current.is_mutual),
            connection_status: patch.connection_status.unwrap_or(current.connection_status),
            relationship_stage: patch
                .relationship_stage
                .unwrap_or(current.relationship_stage),
            relationship_strength: patch
                .relationship_strength
                .unwrap_or(current.relationship_strength),
            tags: patch.tags.unwrap_or(current.tags),
            first_contact_date: patch
                .first_contact_date
                .unwrap_or(current.first_contact_date),
            last_interaction_at: current.last_interaction_at,
            last_interaction_kind: current.last_interaction_kind,
            follow_up_date: patch.follow_up_date.unwrap_or(current.follow_up_date),
            stale: current.stale,
            stale_since: current.stale_since,
            notes,
            metadata: metadata_with_note(base_metadata, note_patch.flatten().as_deref()),
            version: current.version,
            opportunity_id: current.opportunity_id,
            created_at: current.created_at,
            updated_at: current.updated_at,
        };
        let new_v = self
            .repo
            .update_contact(member_id, id, expected_version, &updated)
            .await?;
        let mut out = self.repo.get_contact(member_id, id).await?;
        out.version = new_v;
        Ok(out)
    }

    pub async fn delete_contact(&self, m: Uuid, id: Uuid) -> Result<(), Error> {
        self.repo.delete_contact(m, id).await
    }

    pub async fn record_interaction(
        &self,
        member_id: Uuid,
        contact_id: Uuid,
        kind: InteractionKind,
        summary: &str,
        actor: &str,
        occurred_at: DateTime<Utc>,
    ) -> Result<Interaction, Error> {
        let i = Interaction {
            id: Uuid::new_v4(),
            member_id,
            contact_id,
            kind,
            summary: summary.into(),
            actor: actor.into(),
            occurred_at,
            created_at: Utc::now(),
        };
        self.repo.record_interaction(&i).await?;
        Ok(i)
    }

    pub async fn list_interactions(
        &self,
        member_id: Uuid,
        contact_id: Uuid,
    ) -> Result<Vec<Interaction>, Error> {
        self.repo.list_interactions(member_id, contact_id).await
    }

    pub async fn list_companies(
        &self,
        member_id: Uuid,
        q: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Company>, Error> {
        self.repo.list_companies(member_id, q, limit).await
    }

    pub async fn get_company(&self, member_id: Uuid, id: Uuid) -> Result<Company, Error> {
        self.repo.get_company(member_id, id).await
    }

    pub async fn create_company(
        &self,
        member_id: Uuid,
        new: NewCompany<'_>,
    ) -> Result<Company, Error> {
        let NewCompany {
            name,
            domain,
            industry,
            size_band,
            funding_stage,
            hq_location,
            tech_stack,
            trigger_events,
            public_signals,
            third_party_ttl_at,
        } = new;

        if name.trim().is_empty() {
            return Err(Error::Validation("name required".into()));
        }
        let now = Utc::now();
        let c = Company {
            id: Uuid::new_v4(),
            member_id,
            name: name.into(),
            domain: domain.map(str::to_string),
            industry: industry.map(str::to_string),
            size_band: size_band.map(str::to_string),
            funding_stage: funding_stage.map(str::to_string),
            hq_location: hq_location.map(str::to_string),
            tech_stack,
            trigger_events,
            public_signals,
            enrichment_meta: JsonValue::Object(Default::default()),
            third_party_ttl_at,
            version: 1,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert_company(&c).await?;
        Ok(c)
    }

    pub async fn staleness(&self, member_id: Uuid) -> Result<StalenessReport, Error> {
        self.repo.staleness(member_id).await
    }

    pub async fn link(&self, m: Uuid, contact_id: Uuid, company_id: Uuid) -> Result<(), Error> {
        self.repo
            .link_contact_company(m, contact_id, company_id)
            .await
    }
}

/// PATCH body. `Option<Option<T>>` distinguishes "absent" from "explicit null".
#[derive(Debug, Default)]
pub struct ContactPatch {
    pub display_name: Option<String>,
    pub title: Option<Option<String>>,
    pub headline: Option<Option<String>>,
    pub company: Option<Option<String>>,
    pub linkedin_id: Option<Option<String>>,
    pub linkedin_url: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
    pub company_id: Option<Option<Uuid>>,
    pub tier: Option<ContactTier>,
    pub is_mutual: Option<bool>,
    pub connection_status: Option<ConnectionStatus>,
    pub relationship_stage: Option<RelationshipStage>,
    pub relationship_strength: Option<RelationshipStrength>,
    pub first_contact_date: Option<Option<NaiveDate>>,
    pub follow_up_date: Option<Option<NaiveDate>>,
}

const NOTE_KEY: &str = "notes";

/// Puts `notes` into `metadata`, where `lcc.contacts` actually stores it.
///
/// See [`Contact::notes`]: the contract and the design both require a note on
/// a contact, and `metadata JSONB` is the one real free-form column the table
/// has. Everything else in `metadata` is preserved.
fn metadata_with_note(mut metadata: JsonValue, notes: Option<&str>) -> JsonValue {
    if !metadata.is_object() {
        // A non-object `metadata` cannot hold a key; refuse to silently
        // discard whatever the caller had stored there.
        metadata = JsonValue::Object(Default::default());
    }
    let JsonValue::Object(map) = &mut metadata else {
        unreachable!("just replaced a non-object with an object");
    };
    match notes {
        Some(n) => {
            map.insert(NOTE_KEY.to_string(), JsonValue::String(n.to_string()));
        }
        None => {
            map.remove(NOTE_KEY);
        }
    }
    metadata
}

/// `metadata` with the note key removed, leaving the caller's other keys.
fn strip_note(metadata: &JsonValue) -> JsonValue {
    let mut v = metadata.clone();
    if let JsonValue::Object(map) = &mut v {
        map.remove(NOTE_KEY);
    }
    v
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn note_is_stored_inside_metadata() {
        let m = metadata_with_note(json!({}), Some("met at a conference"));
        assert_eq!(m, json!({"notes": "met at a conference"}));
    }

    #[test]
    fn other_metadata_keys_survive_a_note_write() {
        let m = metadata_with_note(json!({"source": "linkedin"}), Some("hi"));
        assert_eq!(m, json!({"source": "linkedin", "notes": "hi"}));
    }

    #[test]
    fn clearing_the_note_leaves_the_rest() {
        let m = metadata_with_note(json!({"source": "linkedin"}), None);
        assert_eq!(m, json!({"source": "linkedin"}));
        assert_eq!(strip_note(&json!({"notes": "x", "a": 1})), json!({"a": 1}));
    }

    #[test]
    fn non_object_metadata_is_replaced_not_merged_into() {
        // A JSON array has no room for a key; the old value must not be
        // silently mangled into something else.
        let m = metadata_with_note(json!([1, 2]), Some("x"));
        assert_eq!(m, json!({"notes": "x"}));
    }
}
