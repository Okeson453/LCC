//! Live-database integration tests for `network-crm-svc`.
//!
//! # These tests do not skip
//!
//! The remediation this file exists for was, in part, a set of queries that
//! looked plausible and named columns that do not exist. A test that quietly
//! skips when the database is unreachable cannot catch that class of bug — it
//! reports green for code that has never been executed. So there is no
//! `return` on a missing `LCC_TEST_DATABASE_URL`: the documented default is
//! used instead, and if the database cannot be reached the test **fails**.
//!
//! The connection is the one the project's verification steps name:
//!
//! ```text
//! LCC_TEST_DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/lcc_test
//! ```
//!
//! `postgres` is a superuser and therefore bypasses the forced RLS policies on
//! `lcc.contacts` / `lcc.companies` / `lcc.interactions`. That is the point:
//! with RLS out of the way, every assertion about tenant isolation below is
//! testing the service's own `member_id` predicates, which are the only thing
//! keeping members apart in this service (it never sets
//! `app.current_member_id`).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::{Duration, NaiveDate, Utc};
use lcc_network_crm_svc::domain::{
    Company, ConnectionStatus, Contact, ContactTier, Interaction, InteractionKind,
    RelationshipStage, RelationshipStrength,
};
use lcc_network_crm_svc::error::Error;
use lcc_network_crm_svc::repository::PgRepository;
use lcc_network_crm_svc::service::{ContactPatch, NewCompany, NewContact, Service};
use lcc_network_crm_svc::state::placeholder_redis;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

const DEFAULT_TEST_DATABASE_URL: &str = "postgres://postgres:postgres@127.0.0.1:5432/lcc_test";

fn database_url() -> String {
    std::env::var("LCC_TEST_DATABASE_URL").unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string())
}

/// A pool that panics — loudly — if the database is not there.
///
/// Panicking rather than returning `None` is deliberate and is the whole point
/// of this file's header comment: an unreachable database must fail the run,
/// not shrink it to zero tests.
async fn pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(4)
        .acquire_timeout(Duration::seconds(10).to_std().unwrap())
        .connect(&database_url())
        .await
        .expect(
            "network-crm-svc DB tests need a live migrated database at \
             LCC_TEST_DATABASE_URL (default postgres://postgres:postgres@127.0.0.1:5432/lcc_test). \
             These tests must not be skipped: every assertion here is about \
             semantics that a mocked or unexecuted query cannot check.",
        )
}

fn service() -> Service {
    Service::new(
        PgRepository::new(
            PgPoolOptions::new()
                .max_connections(1)
                .acquire_timeout(Duration::seconds(5).to_std().unwrap())
                .connect_lazy(&database_url())
                .expect("lazy pool"),
        ),
        placeholder_redis(),
    )
}

/// Creates a member that the test owns, and removes it (and everything hanging
/// off it) on drop. `linkedin_id` is UNIQUE NOT NULL, hence the uuid suffix.
struct TestMember(Uuid);

impl TestMember {
    async fn create(pool: &PgPool) -> Self {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO lcc.members (id, linkedin_id, email, display_name)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(format!("test-{id}"))
        .bind(format!("test-{id}@example.invalid"))
        .bind("Test Member")
        .execute(pool)
        .await
        .expect("insert member");
        Self(id)
    }

    fn id(&self) -> Uuid {
        self.0
    }
}

// No `Drop` cleanup on purpose: every test mints its own `TestMember`, so a
// leftover row can never be read by another test's scoped query, and the test
// database is disposable. A background thread doing best-effort deletes would
// be a second moving part that could flake for no benefit.

fn new_contact<'a>(display_name: &'a str) -> NewContact<'a> {
    NewContact {
        display_name,
        title: Some("Staff Engineer"),
        headline: Some("Distributed systems, ex-Linode"),
        linkedin_id: None,
        linkedin_url: None,
        company_id: None,
        tags: vec!["kubernetes".to_string()],
        notes: None,
        tier: None,
        first_contact_date: None,
    }
}

fn company(name: &str) -> Company {
    let now = Utc::now();
    Company {
        id: Uuid::now_v7(),
        member_id: Uuid::nil(),
        name: name.to_string(),
        domain: Some("acme.example".to_string()),
        industry: Some("infrastructure".to_string()),
        size_band: Some("51-200".to_string()),
        funding_stage: Some("series_a".to_string()),
        hq_location: Some("Berlin".to_string()),
        tech_stack: vec!["rust".to_string(), "postgres".to_string()],
        trigger_events: json!([{"type": "funding", "date": "2026-01-04"}]),
        public_signals: json!({"hiring": true, "blog_posts": 3}),
        enrichment_meta: json!({}),
        third_party_ttl_at: Some(now + Duration::hours(48)),
        version: 1,
        created_at: now,
        updated_at: now,
    }
}

// ---------------------------------------------------------------------------
// 1. Round-trip: every field lands in the column it claims to.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contact_round_trips_through_the_real_columns() {
    let p = pool().await;
    let m = TestMember::create(&p).await;
    let svc = service();

    let mut c = svc
        .create_contact(
            m.id(),
            NewContact {
                linkedin_id: Some("li-abc-123"),
                linkedin_url: Some("https://www.linkedin.com/in/abc123"),
                notes: Some("met at KubeCon"),
                tier: Some(ContactTier::Vip),
                first_contact_date: NaiveDate::from_ymd_opt(2026, 1, 2),
                ..new_contact("Ada Lovelace")
            },
        )
        .await
        .expect("create contact");

    assert_eq!(c.display_name, "Ada Lovelace");
    assert_eq!(c.tier, ContactTier::Vip);
    // `is_vip` is the 0008 mirror; it must agree with `tier` on the way in.
    assert!(c.is_vip);
    assert_eq!(c.relationship_strength, RelationshipStrength::None);
    assert_eq!(c.relationship_stage, RelationshipStage::Cold);
    assert_eq!(c.connection_status, ConnectionStatus::NotConnected);
    assert_eq!(c.notes.as_deref(), Some("met at KubeCon"));
    assert_eq!(c.version, 1);

    let read = svc.get_contact(m.id(), c.id).await.expect("get contact");
    assert_eq!(read.display_name, c.display_name);
    assert_eq!(read.linkedin_id.as_deref(), Some("li-abc-123"));
    assert_eq!(
        read.linkedin_url.as_deref(),
        Some("https://www.linkedin.com/in/abc123")
    );
    assert_eq!(read.tier, ContactTier::Vip);
    assert!(read.is_vip);
    assert_eq!(read.first_contact_date, NaiveDate::from_ymd_opt(2026, 1, 2));
    assert_eq!(read.notes.as_deref(), Some("met at KubeCon"));
    assert_eq!(read.title.as_deref(), Some("Staff Engineer"));
    assert_eq!(read.tags, vec!["kubernetes".to_string()]);

    // Prove the note really lives in `metadata`, and that nothing else in
    // `metadata` was clobbered by writing it there.
    let (meta,): (serde_json::Value,) =
        sqlx::query_as("SELECT metadata FROM lcc.contacts WHERE id = $1")
            .bind(read.id)
            .fetch_one(&p)
            .await
            .expect("read metadata");
    assert_eq!(meta["notes"], json!("met at KubeCon"));

    c.version = read.version;
    assert_eq!(c.version, 1);
}

/// The i16 that used to be here. The database stores an enum, so the test
/// asserts the enum round-trips *as its text* and that no numeric scale is
/// presented as if it meant something.
#[tokio::test]
async fn relationship_strength_is_the_database_enum_not_a_number() {
    let p = pool().await;
    let m = TestMember::create(&p).await;
    let svc = service();
    let c = svc
        .create_contact(m.id(), new_contact("Grace Hopper"))
        .await
        .expect("create");

    // Each successful update bumps the row's optimistic-concurrency version,
    // so the expected version must be tracked rather than pinned to 1.
    let mut expected: i32 = 1;
    for strength in [
        RelationshipStrength::None,
        RelationshipStrength::Weak,
        RelationshipStrength::Medium,
        RelationshipStrength::Strong,
        RelationshipStrength::StrongRecent,
    ] {
        svc.update_contact(
            m.id(),
            c.id,
            expected,
            ContactPatch {
                relationship_strength: Some(strength),
                ..Default::default()
            },
        )
        .await
        .expect("update strength");
        let back = svc.get_contact(m.id(), c.id).await.expect("reread");
        assert_eq!(back.relationship_strength, strength);
        expected = back.version;

        let (raw,): (String,) =
            sqlx::query_as("SELECT relationship_strength::TEXT FROM lcc.contacts WHERE id = $1")
                .bind(c.id)
                .fetch_one(&p)
                .await
                .expect("read raw strength");
        assert_eq!(raw, strength.as_str());
    }
}

// ---------------------------------------------------------------------------
// 2. Search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_matches_display_name_and_headline() {
    let p = pool().await;
    let m = TestMember::create(&p).await;
    let svc = service();
    svc.create_contact(
        m.id(),
        NewContact {
            headline: Some("Chief Marketing Officer at Globex".into()),
            ..new_contact("Grace Hopper")
        },
    )
    .await
    .expect("create");
    svc.create_contact(
        m.id(),
        NewContact {
            headline: Some("Compiler researcher".into()),
            ..new_contact("Barbara Liskov")
        },
    )
    .await
    .expect("create");

    let by_name = svc
        .list_contacts(m.id(), Some("hopper"), 50)
        .await
        .expect("by name");
    assert_eq!(by_name.len(), 1);
    assert_eq!(by_name[0].display_name, "Grace Hopper");

    // The dashboard searches headline too, so the server does now.
    let by_headline = svc
        .list_contacts(m.id(), Some("globex"), 50)
        .await
        .expect("by headline");
    assert_eq!(by_headline.len(), 1);
    assert_eq!(by_headline[0].display_name, "Grace Hopper");

    let all = svc.list_contacts(m.id(), None, 50).await.expect("all");
    assert_eq!(all.len(), 2);
}

// ---------------------------------------------------------------------------
// 3. Tenant isolation — the security invariant
// ---------------------------------------------------------------------------

#[tokio::test]
async fn one_member_cannot_reach_another_members_contact() {
    let p = pool().await;
    let alice = TestMember::create(&p).await;
    let bob = TestMember::create(&p).await;
    let svc = service();
    let c = svc
        .create_contact(alice.id(), new_contact("Alice Private"))
        .await
        .expect("create");

    // Every one of these is Bob asking for Alice's row by id. All must be
    // NotFound — not Forbidden, not an empty 200 — and never a 500.
    match svc.get_contact(bob.id(), c.id).await {
        Err(Error::NotFound(_)) => {}
        other => panic!("get_contact across tenants returned {other:?}"),
    }
    match svc
        .update_contact(
            bob.id(),
            c.id,
            1,
            ContactPatch {
                display_name: Some("Hijacked".into()),
                ..Default::default()
            },
        )
        .await
    {
        Err(Error::NotFound(_)) => {}
        other => panic!("update_contact across tenants returned {other:?}"),
    }
    match svc.delete_contact(bob.id(), c.id).await {
        Err(Error::NotFound(_)) => {}
        other => panic!("delete_contact across tenants returned {other:?}"),
    }
    match svc
        .record_interaction(
            bob.id(),
            c.id,
            InteractionKind::ManualNote,
            "snooping",
            "member",
            Utc::now(),
        )
        .await
    {
        Err(Error::NotFound(_)) => {}
        other => panic!("record_interaction across tenants returned {other:?}"),
    }
    match svc.list_interactions(bob.id(), c.id).await {
        Err(Error::NotFound(_)) => {}
        other => panic!("list_interactions across tenants returned {other:?}"),
    }

    // And the list endpoints are scoped too.
    assert!(svc
        .list_contacts(bob.id(), None, 50)
        .await
        .expect("bob list")
        .is_empty());
    assert!(svc
        .list_companies(bob.id(), None, 50)
        .await
        .expect("bob co")
        .is_empty());

    // Nothing was actually changed.
    let still = svc
        .get_contact(alice.id(), c.id)
        .await
        .expect("alice reread");
    assert_eq!(still.display_name, "Alice Private");
    assert_eq!(still.version, 1);
}

#[tokio::test]
async fn linking_a_company_across_tenants_is_refused() {
    let p = pool().await;
    let alice = TestMember::create(&p).await;
    let bob = TestMember::create(&p).await;
    let svc = service();

    let mut bob_co = company("Bob Industries");
    bob_co.member_id = bob.id();
    svc.repo()
        .insert_company(&bob_co)
        .await
        .expect("bob company");

    let c = svc
        .create_contact(alice.id(), new_contact("Alice"))
        .await
        .expect("create");

    // `contacts_company_id_fkey` would happily accept this id; only the
    // service's own EXISTS check refuses it.
    match svc.link(alice.id(), c.id, bob_co.id).await {
        Err(Error::NotFound(_)) => {}
        other => panic!("cross-tenant company link returned {other:?}"),
    }
    assert_eq!(
        svc.get_contact(alice.id(), c.id)
            .await
            .expect("reread")
            .company_id,
        None
    );

    // Alice's own company links fine.
    let mut alice_co = company("Alice Industries");
    alice_co.member_id = alice.id();
    svc.repo()
        .insert_company(&alice_co)
        .await
        .expect("alice company");
    svc.link(alice.id(), c.id, alice_co.id).await.expect("link");
    assert_eq!(
        svc.get_contact(alice.id(), c.id)
            .await
            .expect("reread")
            .company_id,
        Some(alice_co.id)
    );
}

// ---------------------------------------------------------------------------
// 4. Optimistic concurrency (design §52)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stale_expected_version_conflicts_and_a_fresh_one_wins() {
    let p = pool().await;
    let m = TestMember::create(&p).await;
    let svc = service();
    let c = svc
        .create_contact(m.id(), new_contact("Versioned"))
        .await
        .expect("create");

    let updated = svc
        .update_contact(
            m.id(),
            c.id,
            1,
            ContactPatch {
                display_name: Some("Versioned v2".into()),
                ..Default::default()
            },
        )
        .await
        .expect("first update");
    assert_eq!(updated.version, 2);
    assert_eq!(updated.display_name, "Versioned v2");

    // Replaying version 1 must be refused, not silently overwrite v2.
    match svc
        .update_contact(
            m.id(),
            c.id,
            1,
            ContactPatch {
                display_name: Some("Versioned v3".into()),
                ..Default::default()
            },
        )
        .await
    {
        Err(Error::Conflict(_)) => {}
        other => panic!("stale version returned {other:?}"),
    }
    assert_eq!(
        svc.get_contact(m.id(), c.id)
            .await
            .expect("reread")
            .display_name,
        "Versioned v2",
        "a rejected update must not have written"
    );
    assert_eq!(
        svc.get_contact(m.id(), c.id).await.expect("reread").version,
        2
    );
}

#[tokio::test]
async fn tier_write_keeps_the_legacy_is_vip_mirror_in_step() {
    let p = pool().await;
    let m = TestMember::create(&p).await;
    let svc = service();
    let c = svc
        .create_contact(m.id(), new_contact("Mirrored"))
        .await
        .expect("create");
    assert!(!c.is_vip);

    for (tier, want_vip) in [
        (ContactTier::Vip, true),
        (ContactTier::Peer, false),
        (ContactTier::Standard, false),
    ] {
        let out = svc
            .update_contact(
                m.id(),
                c.id,
                out_version(&svc, m.id(), c.id).await,
                ContactPatch {
                    tier: Some(tier),
                    ..Default::default()
                },
            )
            .await
            .expect("update");
        assert_eq!(out.tier, tier);
        assert_eq!(out.is_vip, want_vip, "is_vip must mirror tier for {tier:?}");
        let (raw,): (bool,) = sqlx::query_as("SELECT is_vip FROM lcc.contacts WHERE id = $1")
            .bind(c.id)
            .fetch_one(&p)
            .await
            .expect("raw is_vip");
        assert_eq!(raw, want_vip);
    }
}

async fn out_version(svc: &Service, m: Uuid, c: Uuid) -> i32 {
    svc.get_contact(m, c).await.expect("reread").version
}

// ---------------------------------------------------------------------------
// 5. Interactions — the derivation the brief asked for
// ---------------------------------------------------------------------------

#[tokio::test]
async fn logging_an_interaction_derives_the_last_kind_and_touch_stamps() {
    let p = pool().await;
    let m = TestMember::create(&p).await;
    let svc = service();
    let c = svc
        .create_contact(m.id(), new_contact("Interactive"))
        .await
        .expect("create");
    assert_eq!(c.last_interaction_kind, None);
    assert_eq!(c.last_interaction_at, None);

    let t0 = Utc::now() - Duration::days(40);
    svc.record_interaction(
        m.id(),
        c.id,
        InteractionKind::Call,
        "intro call",
        "member",
        t0,
    )
    .await
    .expect("log call");
    let after_call = svc.get_contact(m.id(), c.id).await.expect("reread");
    assert_eq!(after_call.last_interaction_kind.as_deref(), Some("call"));
    assert_eq!(after_call.first_contact_date, Some(t0.date_naive()));
    assert_eq!(
        after_call.last_interaction_at.map(|t| t.timestamp()),
        Some(t0.timestamp())
    );

    let t1 = Utc::now() - Duration::days(2);
    svc.record_interaction(
        m.id(),
        c.id,
        InteractionKind::Meeting,
        "quarterly sync",
        "member",
        t1,
    )
    .await
    .expect("log meeting");
    let after_meeting = svc.get_contact(m.id(), c.id).await.expect("reread");
    assert_eq!(
        after_meeting.last_interaction_kind.as_deref(),
        Some("meeting")
    );
    // `first_contact_date` is written once and never rewritten.
    assert_eq!(after_meeting.first_contact_date, Some(t0.date_naive()));

    // A back-dated note must not un-stale the contact: `last_contact_at` is
    // monotonic.
    svc.record_interaction(
        m.id(),
        c.id,
        InteractionKind::ManualNote,
        "backdated",
        "member",
        t0 - Duration::days(30),
    )
    .await
    .expect("log backdated");
    let after_backdate = svc.get_contact(m.id(), c.id).await.expect("reread");
    assert_eq!(
        after_backdate.last_interaction_at.map(|t| t.timestamp()),
        Some(t1.timestamp()),
        "an older interaction must not move last_contact_at backwards"
    );
    assert_eq!(after_backdate.first_contact_date, Some(t0.date_naive()));

    // The log is listable, newest first, and is not stored on the contact row.
    // "Newest" is by `occurred_at`, not by insertion order: the back-dated note
    // was typed in last but describes the oldest event, so it sorts last.
    let log = svc
        .list_interactions(m.id(), c.id)
        .await
        .expect("list interactions");
    assert_eq!(log.len(), 3);
    assert_eq!(log[0].kind, InteractionKind::Meeting);
    assert!(log[0].occurred_at >= log[2].occurred_at);
    assert!(log.iter().all(|i: &Interaction| i.contact_id == c.id));

    let (stored,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM lcc.interactions WHERE contact_id = $1")
            .bind(c.id)
            .fetch_one(&p)
            .await
            .expect("count");
    assert_eq!(stored, 3);
}

#[tokio::test]
async fn an_invalid_interaction_kind_is_refused_by_the_constraint() {
    let p = pool().await;
    let m = TestMember::create(&p).await;
    let svc = service();
    let c = svc
        .create_contact(m.id(), new_contact("Guarded"))
        .await
        .expect("create");

    // Bypassing the service layer to prove the database is the real gate: the
    // kind strings the old handler accepted would all fail this CHECK.
    for bad in ["outbound_message", "phone_call", "reaction", "other"] {
        let res = sqlx::query(
            "INSERT INTO lcc.interactions (id, member_id, contact_id, kind, summary, occurred_at)
             VALUES ($1,$2,$3,$4,'x',NOW())",
        )
        .bind(Uuid::now_v7())
        .bind(m.id())
        .bind(c.id)
        .bind(bad)
        .execute(&p)
        .await;
        assert!(res.is_err(), "{bad} must violate interactions_kind_check");
    }
    for good in [
        "manual_note",
        "call",
        "meeting",
        "sequence_step_sent",
        "reply_received",
    ] {
        sqlx::query(
            "INSERT INTO lcc.interactions (id, member_id, contact_id, kind, summary, occurred_at)
             VALUES ($1,$2,$3,$4,'x',NOW())",
        )
        .bind(Uuid::now_v7())
        .bind(m.id())
        .bind(c.id)
        .bind(good)
        .execute(&p)
        .await
        .unwrap_or_else(|e| panic!("{good} must satisfy interactions_kind_check: {e}"));
    }
}

// ---------------------------------------------------------------------------
// 6. opportunity_id derivation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn opportunity_id_is_only_populated_when_unambiguous() {
    let p = pool().await;
    let m = TestMember::create(&p).await;
    let svc = service();
    let c = svc
        .create_contact(m.id(), new_contact("Linked"))
        .await
        .expect("create");

    // No opportunity yet -> NULL, and certainly not a guess.
    assert_eq!(
        svc.get_contact(m.id(), c.id)
            .await
            .expect("reread")
            .opportunity_id,
        None
    );

    // Exactly one -> that one.
    let op1 = insert_opportunity(&p, m.id(), c.id, "Acme — SRE").await;
    assert_eq!(
        svc.get_contact(m.id(), c.id)
            .await
            .expect("reread")
            .opportunity_id,
        Some(op1)
    );

    // Two -> ambiguous, so NULL again rather than "the most recent".
    let _op2 = insert_opportunity(&p, m.id(), c.id, "Globex — SRE").await;
    assert_eq!(
        svc.get_contact(m.id(), c.id)
            .await
            .expect("reread")
            .opportunity_id,
        None,
        "with several linked opportunities the contract field has no single right answer"
    );

    // Another member's opportunity linked to the same contact is invisible to
    // the derivation, because the subquery is member-scoped.
    sqlx::query("DELETE FROM lcc.opportunities WHERE contact_id = $1")
        .bind(c.id)
        .execute(&p)
        .await
        .expect("clear");
    let other = TestMember::create(&p).await;
    insert_opportunity(&p, other.id(), c.id, "Initech — SRE").await;
    assert_eq!(
        svc.get_contact(m.id(), c.id)
            .await
            .expect("reread")
            .opportunity_id,
        None
    );
}

async fn insert_opportunity(p: &PgPool, member: Uuid, contact: Uuid, title: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO lcc.opportunities (id, member_id, kind, title, contact_id)
         VALUES ($1, $2, 'job_posting', $3, $4)",
    )
    .bind(id)
    .bind(member)
    .bind(title)
    .bind(contact)
    .execute(p)
    .await
    .expect("insert opportunity");
    id
}

// ---------------------------------------------------------------------------
// 7. Staleness thresholds
// ---------------------------------------------------------------------------

#[tokio::test]
async fn staleness_uses_the_per_tier_thresholds_from_the_contract() {
    let p = pool().await;
    let m = TestMember::create(&p).await;
    let svc = service();

    // 45 days ago: stale for standard (60d) and peer (90d)? No — 45 < 60, so
    // not stale for standard; and not stale for peer or VIP either. It is
    // stale only for... nothing. Use a 10-day-old VIP contact for the "not
    // stale" side and a 75-day-old standard contact for the "stale" side,
    // because 45 days is INSIDE the 60d standard window and asserting it was
    // stale would contradict the contract's own threshold.
    let fresh_vip = svc
        .create_contact(m.id(), new_contact("Fresh VIP"))
        .await
        .expect("create");
    set_last_contact(&p, fresh_vip.id, Utc::now() - Duration::days(10)).await;
    set_tier(&p, fresh_vip.id, "VIP").await;

    let stale_standard = svc
        .create_contact(m.id(), new_contact("Stale Standard"))
        .await
        .expect("create");
    set_last_contact(&p, stale_standard.id, Utc::now() - Duration::days(75)).await;
    set_tier(&p, stale_standard.id, "standard").await;

    // 45 days: inside the peer window (90d) but outside VIP (30d).
    let fresh_peer = svc
        .create_contact(m.id(), new_contact("Fresh Peer"))
        .await
        .expect("create");
    set_last_contact(&p, fresh_peer.id, Utc::now() - Duration::days(45)).await;
    set_tier(&p, fresh_peer.id, "peer").await;

    let stale_vip = svc
        .create_contact(m.id(), new_contact("Stale VIP"))
        .await
        .expect("create");
    set_last_contact(&p, stale_vip.id, Utc::now() - Duration::days(45)).await;
    set_tier(&p, stale_vip.id, "VIP").await;

    let never = svc
        .create_contact(m.id(), new_contact("Never Touched"))
        .await
        .expect("create");

    let report = svc.staleness(m.id()).await.expect("staleness");
    let ids: Vec<Uuid> = report.stale_contacts.iter().map(|s| s.contact_id).collect();

    assert!(
        !ids.contains(&fresh_vip.id),
        "VIP at 10d is inside the 30d window"
    );
    assert!(
        !ids.contains(&fresh_peer.id),
        "peer at 45d is inside the 90d window"
    );
    assert!(
        ids.contains(&stale_standard.id),
        "standard at 75d is past 60d"
    );
    assert!(ids.contains(&stale_vip.id), "VIP at 45d is past 30d");
    assert!(
        ids.contains(&never.id),
        "a never-contacted row is the most stale"
    );

    let never_row = report
        .stale_contacts
        .iter()
        .find(|s| s.contact_id == never.id)
        .expect("never-contacted row");
    // The old code substituted `days * 2 + 90` here. There is no such fact.
    assert_eq!(never_row.days_since_touch, None);
    assert!(never_row.suggested_action.contains("never contacted"));
}

async fn set_last_contact(p: &PgPool, id: Uuid, at: chrono::DateTime<Utc>) {
    sqlx::query("UPDATE lcc.contacts SET last_contact_at = $2 WHERE id = $1")
        .bind(id)
        .bind(at)
        .execute(p)
        .await
        .expect("set last_contact_at");
}

async fn set_tier(p: &PgPool, id: Uuid, tier: &str) {
    sqlx::query("UPDATE lcc.contacts SET tier = $2, is_vip = ($2 = 'VIP') WHERE id = $1")
        .bind(id)
        .bind(tier)
        .execute(p)
        .await
        .expect("set tier");
}

// ---------------------------------------------------------------------------
// 8. Companies
// ---------------------------------------------------------------------------

#[tokio::test]
async fn company_round_trips_jsonb_and_the_real_ttl_column() {
    let p = pool().await;
    let m = TestMember::create(&p).await;
    let svc = service();

    let c = svc
        .create_company(
            m.id(),
            NewCompany {
                name: "Globex",
                domain: Some("globex.example".into()),
                industry: Some("logistics".into()),
                size_band: Some("201-500".into()),
                funding_stage: Some("series_b".into()),
                hq_location: Some("Rotterdam".into()),
                tech_stack: vec!["go".into()],
                trigger_events: json!([{"type": "ipo_filing"}]),
                public_signals: json!({"hiring": false}),
                third_party_ttl_at: Some(Utc::now() + Duration::hours(24)),
            },
        )
        .await
        .expect("create company");

    let back = svc.get_company(m.id(), c.id).await.expect("get company");
    assert_eq!(back.name, "Globex");
    assert_eq!(back.hq_location.as_deref(), Some("Rotterdam"));
    // JSONB, not Vec<String>: an object and an array both survive.
    assert_eq!(back.trigger_events, json!([{"type": "ipo_filing"}]));
    assert_eq!(back.public_signals, json!({"hiring": false}));
    assert!(back.third_party_ttl_at.is_some());
    assert_eq!(back.version, 1);
}

#[tokio::test]
async fn soft_deleted_companies_are_not_listed() {
    let p = pool().await;
    let m = TestMember::create(&p).await;
    let svc = service();
    let c = svc
        .create_company(
            m.id(),
            NewCompany {
                name: "Soon Gone",
                domain: None,
                industry: None,
                size_band: None,
                funding_stage: None,
                hq_location: None,
                tech_stack: vec![],
                trigger_events: json!([]),
                public_signals: json!([]),
                third_party_ttl_at: None,
            },
        )
        .await
        .expect("create");
    assert_eq!(
        svc.list_companies(m.id(), None, 50)
            .await
            .expect("list")
            .len(),
        1
    );

    sqlx::query("UPDATE lcc.companies SET deleted_at = NOW() WHERE id = $1")
        .bind(c.id)
        .execute(&p)
        .await
        .expect("soft delete");
    assert!(
        svc.list_companies(m.id(), None, 50)
            .await
            .expect("list")
            .is_empty(),
        "a soft-deleted company must not be returned"
    );
    assert!(matches!(
        svc.get_company(m.id(), c.id).await,
        Err(Error::NotFound(_))
    ));
}

// ---------------------------------------------------------------------------
// 9. Delete semantics
// ---------------------------------------------------------------------------

#[tokio::test]
async fn deleting_a_missing_contact_is_not_found_not_a_silent_204() {
    let p = pool().await;
    let m = TestMember::create(&p).await;
    let svc = service();
    match svc.delete_contact(m.id(), Uuid::now_v7()).await {
        Err(Error::NotFound(_)) => {}
        other => panic!("delete of a missing contact returned {other:?}"),
    }
    let c = svc
        .create_contact(m.id(), new_contact("Doomed"))
        .await
        .expect("create");
    svc.delete_contact(m.id(), c.id).await.expect("delete");
    assert!(matches!(
        svc.get_contact(m.id(), c.id).await,
        Err(Error::NotFound(_))
    ));
}

// ---------------------------------------------------------------------------
// 10. Validation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_rejects_blank_names_and_non_url_linkedin_urls() {
    let p = pool().await;
    let m = TestMember::create(&p).await;
    let svc = service();
    assert!(matches!(
        svc.create_contact(m.id(), new_contact("   ")).await,
        Err(Error::Validation(_))
    ));
    assert!(matches!(
        svc.create_contact(
            m.id(),
            NewContact {
                linkedin_url: Some("javascript:alert(1)".into()),
                ..new_contact("Bad URL")
            }
        )
        .await,
        Err(Error::Validation(_))
    ));
}

#[tokio::test]
async fn unused_types_stay_compiled() {
    // Keeps the domain's public surface honest: if a struct field is removed,
    // this fails to compile.
    let c = Contact {
        id: Uuid::nil(),
        member_id: Uuid::nil(),
        company_id: None,
        display_name: String::new(),
        headline: None,
        title: None,
        linkedin_id: None,
        linkedin_url: None,
        company: None,
        tier: ContactTier::Standard,
        is_vip: false,
        is_mutual: false,
        connection_status: ConnectionStatus::NotConnected,
        relationship_stage: RelationshipStage::Cold,
        relationship_strength: RelationshipStrength::None,
        tags: vec![],
        notes: None,
        metadata: json!({}),
        first_contact_date: None,
        last_interaction_at: None,
        last_interaction_kind: None,
        follow_up_date: None,
        stale: false,
        stale_since: None,
        version: 1,
        opportunity_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    assert_eq!(c.version, 1);
}
