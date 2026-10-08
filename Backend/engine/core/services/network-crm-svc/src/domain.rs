//! network-crm-svc domain: contacts + companies + interactions + staleness.
//!
//! # Shape of this module
//!
//! Every field below maps to a column that demonstrably exists in the live
//! schema (migrations 0006/0008/0014/0016/0017 + 0020), or to a value derived
//! from an authoritative relationship. Nothing is synthesised.
//!
//! Where a Rust-side enum stands in for a database enum or a `CHECK` list, the
//! enum is authoritative at the type level and the conversion is fallible
//! (`TryFrom<&str>`), so a row that somehow violates its constraint surfaces as
//! an explicit error rather than a plausible-looking wrong value under a 200.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Enums mirroring database constraints
// ---------------------------------------------------------------------------

macro_rules! db_enum {
    (
        $(#[$meta:meta])*
        $name:ident { $( $variant:ident => $wire:literal ),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $( #[serde(rename = $wire)] $variant ),+
        }

        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $( Self::$variant => $wire ),+
                }
            }
        }

        impl FromStr for $name {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $( $wire => Ok(Self::$variant), )+
                    _ => Err(()),
                }
            }
        }
    };
}

db_enum!(
    /// Design §11.5 + canonical contract: `tier`.
    /// Mirrors `contacts_tier_check` added in migration 0020.
    ContactTier { Vip => "VIP", Standard => "standard", Peer => "peer" }
);

db_enum!(
    /// Design §11.5 + canonical contract: `connection_status`.
    /// Mirrors `contacts_connection_status_check` (0020).
    ConnectionStatus {
        NotConnected => "not_connected",
        Pending => "pending",
        Connected => "connected",
    }
);

db_enum!(
    /// Design §11.5 + canonical contract: `relationship_stage`.
    /// Mirrors `contacts_relationship_stage_check` (0020).
    RelationshipStage {
        Cold => "cold",
        Connected => "connected",
        Engaged => "engaged",
        Conversation => "conversation",
        Opportunity => "opportunity",
        Closed => "closed",
    }
);

db_enum!(
    /// `lcc.relationship_strength` (migration 0008), a real PostgreSQL ENUM.
    ///
    /// This was an `i16` in the pre-remediation domain. It is deliberately NOT
    /// a number: the five values are ordered (none < weak < medium < strong <
    /// strong_recent) but the gaps between them carry no meaning, so any
    /// linear scale would invent distances that the schema does not assert —
    /// "medium" would have to be either 2 or 3 and neither is more true. The
    /// enum is carried as text, which is also what the database stores and what
    /// the contract/design name.
    RelationshipStrength {
        None => "none",
        Weak => "weak",
        Medium => "medium",
        Strong => "strong",
        StrongRecent => "strong_recent",
    }
);

db_enum!(
    /// `lcc.interactions.kind` — mirrors the `interactions_kind_check`
    /// constraint (migration 0017). These are the only values the database will
    /// accept; the pre-remediation enum (`outbound_message`, `phone_call`,
    /// `connection_request`, `reaction`, `public_comment`, `other`) would have
    /// been rejected by that CHECK on every write.
    InteractionKind {
        ManualNote => "manual_note",
        InboundMessage => "inbound_message",
        SentMessage => "sent_message",
        Call => "call",
        Meeting => "meeting",
        SequenceStepSent => "sequence_step_sent",
        ReplyReceived => "reply_received",
        ConnectionAccepted => "connection_accepted",
    }
);

// ---------------------------------------------------------------------------
// Contact
// ---------------------------------------------------------------------------

/// A `lcc.contacts` row plus two derived values.
///
/// The derivation rule, stated once so the repository does not have to
/// re-argue it:
///
/// * `last_interaction_kind` — the `kind` of the most recent non-deleted row
///   in `lcc.interactions` for this contact (migration 0017, the symmetric
///   interaction log). It is NOT stored on the contact row: 0017 created a log
///   precisely so interaction facts live in one place, and caching the last
///   kind on the contact would be a second source of truth to keep in sync.
/// * `opportunity_id` — `lcc.opportunities.contact_id` (migration 0009) is the
///   *inverse* of the `contact.opportunity_id` FK the design §11.5 declares,
///   and it is one-to-many. When exactly one opportunity is linked the id is
///   unambiguous and is returned; when zero or several are linked it is NULL.
///   Picking "the most recent" of several would be an invented rule that could
///   return a wrong id under a 200 OK.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    // --- identity ---
    pub id: Uuid,
    pub member_id: Uuid,
    /// `contacts.display_name` (0008, NOT NULL). The contract calls this
    /// `name`; the column is the real name of the concept, so the API field is
    /// named after the column and the contract schema was updated to match.
    pub display_name: String,
    /// `contacts.linkedin_id` (0008) — the platform member id.
    pub linkedin_id: Option<String>,
    /// `contacts.linkedin_url` (0020) — the human-facing profile URI.
    /// Distinct from `linkedin_id`; see migration 0020 §4.
    pub linkedin_url: Option<String>,

    // --- professional profile ---
    pub title: Option<String>,
    pub headline: Option<String>,
    /// `contacts.company` (0008) — the free-text company name kept alongside
    /// the structured link below.
    pub company: Option<String>,
    /// `contacts.company_id` (0017) — FK to `lcc.companies(id)`.
    pub company_id: Option<Uuid>,

    // --- relationship state ---
    pub tier: ContactTier,
    /// `contacts.is_vip` (0008). Legacy mirror of `tier`; the service writes
    /// both in the same statement so they cannot disagree.
    pub is_vip: bool,
    /// `contacts.is_mutual` (0008) — "we are connected to each other".
    pub is_mutual: bool,
    pub connection_status: ConnectionStatus,
    pub relationship_stage: RelationshipStage,
    pub relationship_strength: RelationshipStrength,
    pub tags: Vec<String>,

    // --- interaction history ---
    pub first_contact_date: Option<NaiveDate>,
    /// `contacts.last_contact_at` (0008).
    pub last_interaction_at: Option<DateTime<Utc>>,
    /// Derived — see the type-level note above.
    pub last_interaction_kind: Option<String>,
    pub follow_up_date: Option<NaiveDate>,

    // --- staleness (migration 0016) ---
    pub stale: bool,
    pub stale_since: Option<DateTime<Utc>>,

    // --- free-form / bookkeeping ---
    /// Free-text note, stored at `contacts.metadata->>'notes'`.
    ///
    /// The contract's `ContactUpdate` carries `notes` and the design's §18.5
    /// says PATCH updates "tags, tier, notes", but neither §11.5 nor any
    /// migration ever declared a `notes` column. `metadata JSONB` is the
    /// contact's real, purpose-built free-form column and is otherwise
    /// unconstrained, so the note lives there rather than in a new column that
    /// nothing else in the schema would agree with.
    pub notes: Option<String>,
    /// The rest of `contacts.metadata` (the note removed).
    pub metadata: serde_json::Value,
    /// `contacts.version` (0020) — optimistic concurrency, design §52.
    pub version: i32,

    /// Derived — see the type-level note above.
    pub opportunity_id: Option<Uuid>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Company
// ---------------------------------------------------------------------------

/// A `lcc.companies` row (migration 0017 / design §11.6).
///
/// `trigger_events`, `public_signals` and `enrichment_meta` are JSONB in the
/// schema. The pre-remediation domain typed them as `Vec<String>`, which would
/// have failed to decode even once the column name was right; they are exposed
/// as `serde_json::Value` so the service never claims a shape the column does
/// not guarantee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: Uuid,
    pub member_id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub industry: Option<String>,
    pub size_band: Option<String>,
    pub funding_stage: Option<String>,
    pub hq_location: Option<String>,
    pub tech_stack: Vec<String>,
    pub trigger_events: serde_json::Value,
    pub public_signals: serde_json::Value,
    pub enrichment_meta: serde_json::Value,
    /// `companies.third_party_ttl_at` (0017). The service previously selected
    /// a column called `ttl_at`, which has never existed.
    pub third_party_ttl_at: Option<DateTime<Utc>>,
    /// `companies.version` is BIGINT in the schema.
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Interaction
// ---------------------------------------------------------------------------

/// A `lcc.interactions` row (migration 0017).
///
/// `channel`, `direction` and `metadata` are gone: no such columns exist, the
/// canonical contract's `Interaction` does not declare them, and the design's
/// §3.4 log records `actor` instead — which the schema does have.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: Uuid,
    pub contact_id: Uuid,
    pub member_id: Uuid,
    pub kind: InteractionKind,
    pub summary: String,
    /// `interactions.actor` — `'member'` for human entries, `'system:<svc>'`
    /// for auto-logged events (0017).
    pub actor: String,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Staleness (design §18.5, contract `getStaleContacts`)
// ---------------------------------------------------------------------------

/// A contact past its staleness threshold.
///
/// Thresholds come from the canonical contract's
/// `getStaleContacts` summary: "VIP 30d / standard 60d / peer 90d", keyed on
/// `tier`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalenessReport {
    pub member_id: Uuid,
    pub as_of: DateTime<Utc>,
    pub stale_contacts: Vec<StaleContact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaleContact {
    pub contact_id: Uuid,
    pub display_name: String,
    pub tier: ContactTier,
    /// Days since the last recorded contact, or `None` for a contact that has
    /// never been contacted. The pre-remediation code substituted
    /// `days * 2 + 90` for the NULL case, inventing a number for a contact
    /// with no history at all.
    pub days_since_touch: Option<i64>,
    pub suggested_action: String,
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn interaction_kind_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&InteractionKind::InboundMessage).unwrap(),
            "\"inbound_message\""
        );
        assert_eq!(
            serde_json::to_string(&InteractionKind::SequenceStepSent).unwrap(),
            "\"sequence_step_sent\""
        );
    }

    /// Every wire value must round-trip through `FromStr`, because that is how
    /// a value read out of the database is turned back into an enum.
    #[test]
    fn every_enum_round_trips() {
        for k in [
            InteractionKind::ManualNote,
            InteractionKind::InboundMessage,
            InteractionKind::SentMessage,
            InteractionKind::Call,
            InteractionKind::Meeting,
            InteractionKind::SequenceStepSent,
            InteractionKind::ReplyReceived,
            InteractionKind::ConnectionAccepted,
        ] {
            assert_eq!(InteractionKind::from_str(k.as_str()).unwrap(), k);
        }
        for t in [ContactTier::Vip, ContactTier::Standard, ContactTier::Peer] {
            assert_eq!(ContactTier::from_str(t.as_str()).unwrap(), t);
        }
        for s in [
            RelationshipStrength::None,
            RelationshipStrength::Weak,
            RelationshipStrength::Medium,
            RelationshipStrength::Strong,
            RelationshipStrength::StrongRecent,
        ] {
            assert_eq!(RelationshipStrength::from_str(s.as_str()).unwrap(), s);
        }
        for c in [
            ConnectionStatus::NotConnected,
            ConnectionStatus::Pending,
            ConnectionStatus::Connected,
        ] {
            assert_eq!(ConnectionStatus::from_str(c.as_str()).unwrap(), c);
        }
        for r in [
            RelationshipStage::Cold,
            RelationshipStage::Connected,
            RelationshipStage::Engaged,
            RelationshipStage::Conversation,
            RelationshipStage::Opportunity,
            RelationshipStage::Closed,
        ] {
            assert_eq!(RelationshipStage::from_str(r.as_str()).unwrap(), r);
        }
    }

    /// The point of replacing the i16 `connection_strength`: a value outside the
    /// database's own vocabulary has to be rejected, not coerced to a number.
    #[test]
    fn unknown_strength_is_rejected_not_coerced() {
        assert!(RelationshipStrength::from_str("4").is_err());
        assert!(RelationshipStrength::from_str("vip").is_err());
        assert!(InteractionKind::from_str("phone_call").is_err());
    }

    #[test]
    fn tier_serializes_with_contract_casing() {
        assert_eq!(serde_json::to_string(&ContactTier::Vip).unwrap(), "\"VIP\"");
        assert_eq!(
            serde_json::to_string(&ContactTier::Standard).unwrap(),
            "\"standard\""
        );
    }
}
