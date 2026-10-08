//! Live-database tests for `lcc-opportunity-svc`.
//!
//! These run against a real PostgreSQL 15 instance with the real migration set
//! applied. They are the tests that would have caught the defects this
//! remediation fixes: `PREPARE`-based SQL validation resolves names but does
//! not execute a statement, so a `NOT NULL` column missing from an INSERT, or
//! a `CHECK` constraint that a written value violates, both look clean to the
//! detector and only fail when a request is actually served.
//!
//! Point `LCC_TEST_DATABASE_URL` at a database built from
//! `Backend/schemas/migrations/*.sql`:
//!
//! ```text
//! LCC_TEST_DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/lcc_test \
//!   cargo test -p lcc-opportunity-svc
//! ```
//!
//! Without the variable the database-backed tests compile and skip, so a
//! developer with no Postgres can still run `cargo test`.
//!
//! Note on RLS: `lcc.attach_member_rls` enables and FORCES row-level security on
//! every table here. When the suite runs as a superuser (the usual local/CI
//! case) PostgreSQL bypasses RLS, which is why the tenant assertions below
//! check the *application* predicate — the `WHERE member_id = $1` the service
//! itself is responsible for — rather than relying on the policy alone.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use lcc_opportunity_svc::domain::{
    Opportunity, OpportunityCursor, OpportunityFunnel, OpportunityKind, OpportunityStatus,
    OpportunityTrack, Page, ProposalPayload, Source,
};
use lcc_opportunity_svc::error::Error;
use lcc_opportunity_svc::repository::PgRepository;
use lcc_opportunity_svc::service::Service;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::time::Duration;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

async fn pool() -> Option<PgPool> {
    let url = std::env::var("LCC_TEST_DATABASE_URL").ok()?;
    Some(
        PgPoolOptions::new()
            .max_connections(4)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&url)
            .await
            .expect("LCC_TEST_DATABASE_URL is set but unreachable"),
    )
}

/// The service under test. The Redis pool is never used by these paths and
/// `deadpool_redis` connects lazily, so no Redis is needed to run the suite.
fn repo(pool: &PgPool) -> PgRepository {
    PgRepository::new(pool.clone())
}

fn service(pool: &PgPool) -> Service {
    let redis = deadpool_redis::Config::from_url("redis://127.0.0.1:6379")
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("build lazy redis pool");
    Service::new(PgRepository::new(pool.clone()), redis)
}

/// Create a throwaway member. `linkedin_id` and `display_name` are `NOT NULL`
/// with no default, and `linkedin_id` is UNIQUE.
async fn member(pool: &PgPool, tag: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO lcc.members (id, linkedin_id, display_name)
           VALUES ($1::uuid, $2::text, $3::text)"#,
    )
    .bind(id)
    .bind(format!("linkedin-{tag}-{id}"))
    .bind(format!("Test {tag}"))
    .execute(pool)
    .await
    .expect("insert member");
    id
}

fn opportunity(
    member_id: Uuid,
    kind: OpportunityKind,
    title: &str,
    fit_score: Option<f64>,
    discovered_at: chrono::DateTime<chrono::Utc>,
) -> Opportunity {
    Opportunity {
        id: Uuid::new_v4(),
        member_id,
        kind,
        track: OpportunityTrack::from_kind(kind),
        company_id: None,
        contact_id: None,
        position: title.to_string(),
        title: title.to_string(),
        company: None,
        source: Source::Manual,
        status: OpportunityStatus::Discovered,
        fit_score,
        fit_components: serde_json::json!({}),
        discovered_at,
        last_evaluated_at: None,
        created_at: discovered_at,
        updated_at: discovered_at,
        metadata: serde_json::json!({}),
        version: 1,
    }
}

// ---------------------------------------------------------------------------
// The `proposals` decision, asserted against the live schema
// ---------------------------------------------------------------------------

/// `lcc.proposals` does not exist; `lcc.applications` does; and
/// `client_proposal` is one of `lcc.applications.application_type`'s allowed
/// values. This is the database half of the decision documented in
/// `src/domain.rs` and in the service README.
#[tokio::test]
async fn proposals_is_an_application_type_not_a_table() {
    let Some(pool) = pool().await else { return };

    let row = sqlx::query(
        r#"SELECT to_regclass('lcc.proposals')::text AS proposals,
                  to_regclass('lcc.applications')::text AS applications,
                  to_regclass('lcc.opportunity_applications')::text AS opportunity_applications"#,
    )
    .fetch_one(&pool)
    .await
    .expect("probe catalog");

    assert_eq!(
        row.try_get::<Option<String>, _>("proposals").unwrap(),
        None,
        "lcc.proposals must not exist: the contract declares no Proposal entity"
    );
    assert_eq!(
        row.try_get::<Option<String>, _>("opportunity_applications")
            .unwrap(),
        None,
        "lcc.opportunity_applications must not exist either"
    );
    assert_eq!(
        row.try_get::<Option<String>, _>("applications").unwrap(),
        Some("lcc.applications".to_string())
    );

    let defs: Vec<String> = sqlx::query_scalar(
        r#"SELECT pg_get_constraintdef(oid)
           FROM pg_constraint
           WHERE conrelid = 'lcc.applications'::regclass
             AND conname = 'applications_application_type_check'"#,
    )
    .fetch_all(&pool)
    .await
    .expect("read application_type CHECK");
    assert_eq!(defs.len(), 1, "the CHECK constraint must exist");
    assert!(
        defs[0].contains("client_proposal"),
        "client_proposal must be a legal application_type; got {}",
        defs[0]
    );
}

// ---------------------------------------------------------------------------
// `position` is `opportunities.title`, never `opportunities.kind`
// ---------------------------------------------------------------------------

/// The defect named in the brief, asserted end to end: an application reports
/// the position its opportunity advertises, and the opportunity CATEGORY is a
/// different string that never appears in the `position` field.
#[tokio::test]
async fn application_position_is_the_opportunity_title_never_the_kind() {
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let member_id = member(&pool, "position").await;

    const TITLE: &str = "Principal Platform Engineer — Helios Robotics";
    let opp = opportunity(
        member_id,
        OpportunityKind::JobPosting,
        TITLE,
        Some(0.91),
        chrono::Utc::now(),
    );
    assert_ne!(opp.kind.as_str(), TITLE);
    assert_eq!(opp.kind.as_str(), "job_posting");
    repo(&pool).insert(&opp).await.expect("insert opportunity");

    // The database column really does hold the position.
    let stored: String =
        sqlx::query_scalar(r#"SELECT title FROM lcc.opportunities WHERE id = $1::uuid"#)
            .bind(opp.id)
            .fetch_one(&pool)
            .await
            .expect("read title");
    assert_eq!(stored, TITLE);

    let app = svc
        .apply(member_id, opp.id, Some("cover".into()), None)
        .await
        .expect("draft application");

    assert_eq!(
        app.position.as_deref(),
        Some(TITLE),
        "position must be opportunities.title"
    );
    assert_ne!(app.position.as_deref(), Some("job_posting"));

    // And the same value comes back out of the list query, which is where the
    // JOIN (rather than a column) is easiest to get wrong.
    let listed = svc
        .list_applications(member_id, 50)
        .await
        .expect("list applications");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].position.as_deref(), Some(TITLE));

    // The opportunity projection carries it too.
    let fetched = svc.get(member_id, opp.id).await.expect("get opportunity");
    assert_eq!(fetched.position, TITLE);
    assert_eq!(fetched.title, TITLE);
    assert_eq!(fetched.kind, OpportunityKind::JobPosting);
    assert_eq!(fetched.track, OpportunityTrack::Job);
}

/// `kind` really is the category enum in the database, and the track
/// projection is the documented one.
#[tokio::test]
async fn opportunity_kind_is_a_category_enum() {
    let Some(pool) = pool().await else { return };

    let labels: Vec<String> = sqlx::query_scalar(
        r#"SELECT e.enumlabel
           FROM pg_enum e
           JOIN pg_type t ON t.oid = e.enumtypid
           JOIN pg_namespace n ON n.oid = t.typnamespace
           WHERE n.nspname = 'lcc' AND t.typname = 'opportunity_kind'
           ORDER BY e.enumsortorder"#,
    )
    .fetch_all(&pool)
    .await
    .expect("read opportunity_kind labels");

    assert_eq!(
        labels,
        vec![
            "job_posting",
            "consulting",
            "partnership",
            "speaking",
            "mentorship",
            "other"
        ],
        "the category enum is fixed by migration 0009"
    );
    // None of them is a position, which is exactly why `kind` must never be
    // served in a `position` field.
    assert!(labels.iter().all(|l| !l.contains(' ')));

    assert_eq!(
        OpportunityTrack::from_kind(OpportunityKind::JobPosting),
        OpportunityTrack::Job
    );
    for k in [
        OpportunityKind::Consulting,
        OpportunityKind::Partnership,
        OpportunityKind::Speaking,
        OpportunityKind::Mentorship,
        OpportunityKind::Other,
    ] {
        assert_eq!(OpportunityTrack::from_kind(k), OpportunityTrack::Client);
    }
}

// ---------------------------------------------------------------------------
// `last_step_sent_at`
// ---------------------------------------------------------------------------

/// The second correction named in the brief, verified against the live schema.
///
/// `lcc.sequences` has no `last_step_sent_at` column, and `sequences.started_at`
/// is `NOT NULL DEFAULT now()` — the moment the sequence was *created*, which
/// is not the moment a step was last sent. The only place a send time lives is
/// `lcc.sequence_steps.sent_at`, which is nullable (a step that has not been
/// sent has no send time), so `last_step_sent_at` is
/// `MAX(sequence_steps.sent_at)` per sequence and is NULL until the first send.
///
/// This belongs to outreach-svc's `Sequence` projection; opportunity-svc has no
/// sequence read path, so there is nothing here to change. The test pins the
/// derivation against the live schema rather than against prose.
#[tokio::test]
async fn last_step_sent_at_derives_from_sequence_steps_sent_at() {
    let Some(pool) = pool().await else { return };

    let on_sequences: i64 = sqlx::query_scalar(
        r#"SELECT count(*) FROM information_schema.columns
           WHERE table_schema = 'lcc' AND table_name = 'sequences'
             AND column_name = 'last_step_sent_at'"#,
    )
    .fetch_one(&pool)
    .await
    .expect("probe sequences columns");
    assert_eq!(on_sequences, 0, "lcc.sequences has no last_step_sent_at");

    // sequences.started_at exists and is NOT NULL: it is the creation time.
    let started_at_nullable: String = sqlx::query_scalar(
        r#"SELECT is_nullable FROM information_schema.columns
           WHERE table_schema = 'lcc' AND table_name = 'sequences'
             AND column_name = 'started_at'"#,
    )
    .fetch_one(&pool)
    .await
    .expect("probe started_at");
    assert_eq!(started_at_nullable, "NO");

    // The send time lives on the step, and is nullable.
    let sent_at_nullable: String = sqlx::query_scalar(
        r#"SELECT is_nullable FROM information_schema.columns
           WHERE table_schema = 'lcc' AND table_name = 'sequence_steps'
             AND column_name = 'sent_at'"#,
    )
    .fetch_one(&pool)
    .await
    .expect("probe sequence_steps.sent_at");
    assert_eq!(sent_at_nullable, "YES");

    // The derivation itself, run for real, against a purpose-built fixture.
    let member_id = member(&pool, "seqderive").await;
    let contact_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO lcc.contacts (id, member_id, display_name)
           VALUES (gen_random_uuid(), $1::uuid, 'seq contact')
           RETURNING id"#,
    )
    .bind(member_id)
    .fetch_one(&pool)
    .await
    .expect("insert contact");

    let seq_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO lcc.sequences (id, member_id, contact_id)
           VALUES (gen_random_uuid(), $1::uuid, $2::uuid)
           RETURNING id"#,
    )
    .bind(member_id)
    .bind(contact_id)
    .fetch_one(&pool)
    .await
    .expect("insert sequence");
    let started_at: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar(r#"SELECT started_at FROM lcc.sequences WHERE id = $1::uuid"#)
            .bind(seq_id)
            .fetch_one(&pool)
            .await
            .expect("read started_at");

    // Before any step is sent, last_step_sent_at is NULL — which is exactly
    // where a `COALESCE(sent_at, started_at)` shortcut would invent a value.
    let before: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        r#"SELECT MAX(ss.sent_at) FROM lcc.sequence_steps ss
           WHERE ss.sequence_id = $1::uuid"#,
    )
    .bind(seq_id)
    .fetch_one(&pool)
    .await
    .expect("derive before send");
    assert_eq!(before, None, "no step has been sent yet");

    // Send the first step, then a second one, with a deliberate gap.
    let t1 = started_at + chrono::Duration::hours(1);
    let t2 = started_at + chrono::Duration::hours(3);
    let steps: [(i32, chrono::DateTime<chrono::Utc>); 2] = [(0, t1), (1, t2)];
    for (idx, at) in steps {
        sqlx::query(
            r#"INSERT INTO lcc.sequence_steps
                  (id, sequence_id, member_id, step_index, body, status,
                   scheduled_at, sent_at)
               VALUES (gen_random_uuid(), $1::uuid, $2::uuid, $3::int, 'b', 'sent',
                       $4::timestamptz, $4::timestamptz)"#,
        )
        .bind(seq_id)
        .bind(member_id)
        .bind(idx)
        .bind(at)
        .execute(&pool)
        .await
        .expect("insert step");
    }

    let after: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        r#"SELECT MAX(ss.sent_at) FROM lcc.sequence_steps ss
           WHERE ss.sequence_id = $1::uuid"#,
    )
    .bind(seq_id)
    .fetch_one(&pool)
    .await
    .expect("derive after send");
    assert_eq!(after, Some(t2), "MAX(sent_at), not started_at");
    assert_ne!(after, Some(started_at), "started_at is NOT the answer");
}

// ---------------------------------------------------------------------------
// Runtime behaviour the detector cannot see
// ---------------------------------------------------------------------------

/// `lcc.opportunities.kind` is `NOT NULL` with no default. The previous INSERT
/// omitted it, which PREPAREd cleanly and would have failed on every request.
#[tokio::test]
async fn discover_persists_the_required_kind() {
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let member_id = member(&pool, "kind").await;

    // No `kind` supplied: the schema's own catch-all is stored, which is
    // honest — it asserts nothing the caller did not say.
    let opp = svc
        .discover(
            member_id,
            "Fractional CTO advisory",
            None,
            Source::Manual,
            None,
            None,
            None,
            serde_json::Value::Null,
        )
        .await
        .expect("discover must succeed");
    assert_eq!(opp.kind, OpportunityKind::Other);

    let (kind, phi_score): (String, f64) = sqlx::query_as(
        r#"SELECT kind::text, phi_score::double precision
           FROM lcc.opportunities WHERE id = $1::uuid"#,
    )
    .bind(opp.id)
    .fetch_one(&pool)
    .await
    .expect("read back");
    assert_eq!(kind, "other");
    // phi_score is NOT NULL; an unscored opportunity is 0, not NULL.
    assert_eq!(phi_score, 0.0);

    // An explicit category round-trips.
    let consulting = svc
        .discover(
            member_id,
            "Architecture review",
            Some(OpportunityKind::Consulting),
            Source::Referral,
            None,
            None,
            None,
            serde_json::json!({"note": "from a warm intro"}),
        )
        .await
        .expect("discover consulting");
    let (kind, title): (String, String) =
        sqlx::query_as(r#"SELECT kind::text, title FROM lcc.opportunities WHERE id = $1::uuid"#)
            .bind(consulting.id)
            .fetch_one(&pool)
            .await
            .expect("read back");
    assert_eq!(kind, "consulting");
    assert_eq!(title, "Architecture review");
}

/// A blank title is rejected before it can become a NOT NULL violation.
#[tokio::test]
async fn discover_rejects_a_blank_title() {
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let member_id = member(&pool, "blank").await;
    let err = svc
        .discover(
            member_id,
            "   ",
            None,
            Source::Manual,
            None,
            None,
            None,
            serde_json::Value::Null,
        )
        .await;
    assert!(matches!(err, Err(Error::Validation(_))), "got {err:?}");
}

/// `lcc.applications.status` has a CHECK constraint that the contract's
/// `Application.status` enum matches exactly. The old code wrote
/// `OpportunityStatus::Applied` (`"applied"`) into it, which no amount of
/// `PREPARE` would have caught.
#[tokio::test]
async fn application_status_check_rejects_opportunity_status_values() {
    let Some(pool) = pool().await else { return };
    let member_id = member(&pool, "statuscheck").await;

    let err = sqlx::query(
        r#"INSERT INTO lcc.applications
              (member_id, opportunity_id, application_type, status)
           VALUES ($1::uuid, gen_random_uuid(), 'job_application', 'applied')"#,
    )
    .bind(member_id)
    .execute(&pool)
    .await
    .expect_err("'applied' must violate the applications status CHECK");
    let msg = err.to_string();
    assert!(
        msg.contains("applications_status_check"),
        "expected the status CHECK to fire, got: {msg}"
    );
}

/// A second job application for the same opportunity is a conflict, not a 500:
/// `lcc.applications` is `UNIQUE (member_id, opportunity_id, application_type)`.
#[tokio::test]
async fn duplicate_application_is_reported_as_a_conflict() {
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let member_id = member(&pool, "conflict").await;
    let opp = opportunity(
        member_id,
        OpportunityKind::JobPosting,
        "Staff Engineer",
        Some(0.5),
        chrono::Utc::now(),
    );
    repo(&pool).insert(&opp).await.unwrap();

    svc.apply(member_id, opp.id, None, None).await.unwrap();
    let err = svc
        .apply(member_id, opp.id, None, None)
        .await
        .expect_err("second application must be rejected");
    assert!(
        matches!(err, Error::Conflict(_)),
        "expected Conflict, got {err:?}"
    );

    // A client proposal for the same opportunity is a DIFFERENT application_type
    // and must still be allowed — that is the whole point of the discriminator.
    svc.propose(member_id, opp.id, "Proposal", "Body", None, None, vec![])
        .await
        .expect("a client_proposal alongside a job_application is allowed");
}

/// Applying to an opportunity that does not exist, or belongs to somebody else,
/// is a 404 — never an orphan row with a dangling `opportunity_id`.
#[tokio::test]
async fn apply_requires_an_existing_owned_opportunity() {
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let alice = member(&pool, "owner").await;
    let bob = member(&pool, "intruder").await;

    let opp = opportunity(
        alice,
        OpportunityKind::Partnership,
        "Channel partnership",
        None,
        chrono::Utc::now(),
    );
    repo(&pool).insert(&opp).await.unwrap();

    let missing = svc.apply(bob, Uuid::new_v4(), None, None).await;
    assert!(
        matches!(missing, Err(Error::NotFound(_))),
        "got {missing:?}"
    );

    let foreign = svc.apply(bob, opp.id, None, None).await;
    assert!(
        matches!(foreign, Err(Error::NotFound(_))),
        "another member's opportunity must be indistinguishable from a missing one; got {foreign:?}"
    );
}

// ---------------------------------------------------------------------------
// Tenant isolation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn member_predicate_isolates_every_read() {
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let alice = member(&pool, "alice").await;
    let bob = member(&pool, "bob").await;

    let opp = opportunity(
        alice,
        OpportunityKind::JobPosting,
        "Senior SRE",
        Some(0.8),
        chrono::Utc::now(),
    );
    repo(&pool).insert(&opp).await.unwrap();
    svc.apply(alice, opp.id, None, None).await.unwrap();
    svc.propose(alice, opp.id, "P", "B", None, None, vec![])
        .await
        .unwrap();

    // Bob sees none of it, through any of the read paths.
    assert!(svc.get(bob, opp.id).await.is_err(), "get leaked");
    assert!(svc
        .list(bob, None, 100, None)
        .await
        .unwrap()
        .items
        .is_empty());
    assert!(svc
        .list(bob, Some(OpportunityStatus::Discovered), 100, None)
        .await
        .unwrap()
        .items
        .is_empty());
    assert!(svc.list_applications(bob, 100).await.unwrap().is_empty());
    assert!(svc.list_proposals(bob, 100).await.unwrap().is_empty());

    // Alice sees exactly her own rows.
    assert_eq!(
        svc.list(alice, None, 100, None).await.unwrap().items.len(),
        1
    );
    assert_eq!(svc.list_applications(alice, 100).await.unwrap().len(), 2);
    assert_eq!(svc.list_proposals(alice, 100).await.unwrap().len(), 1);
}

/// A company or contact id from another member is rejected on write, so a
/// member's opportunity can never carry a cross-tenant pointer.
#[tokio::test]
async fn cross_tenant_references_are_rejected_on_write() {
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let alice = member(&pool, "ref-alice").await;
    let bob = member(&pool, "ref-bob").await;

    let bob_contact: Uuid = sqlx::query_scalar(
        r#"INSERT INTO lcc.contacts (id, member_id, display_name)
           VALUES (gen_random_uuid(), $1::uuid, 'bob contact') RETURNING id"#,
    )
    .bind(bob)
    .fetch_one(&pool)
    .await
    .expect("insert bob contact");

    let err = svc
        .discover(
            alice,
            "Cross-tenant contact",
            None,
            Source::Manual,
            None,
            Some(bob_contact),
            None,
            serde_json::Value::Null,
        )
        .await;
    assert!(
        matches!(err, Err(Error::Validation(_))),
        "a foreign contact_id must be refused; got {err:?}"
    );

    // Nothing was written.
    let n: i64 = sqlx::query_scalar(
        r#"SELECT count(*) FROM lcc.opportunities
           WHERE member_id = $1::uuid AND title = 'Cross-tenant contact'"#,
    )
    .bind(alice)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 0);

    // A missing id is refused identically, so the error is not a probe for
    // whether somebody else's contact exists.
    let missing = svc
        .discover(
            alice,
            "Missing contact",
            None,
            Source::Manual,
            None,
            Some(Uuid::new_v4()),
            None,
            serde_json::Value::Null,
        )
        .await;
    assert!(
        matches!(missing, Err(Error::Validation(_))),
        "got {missing:?}"
    );
}

// ---------------------------------------------------------------------------
// Ordering, nullability, pagination
// ---------------------------------------------------------------------------

/// Unscored opportunities sort last, scored ones by descending φ, ties broken
/// by recency then id, and the keyset cursor walks the whole set exactly once.
#[tokio::test]
async fn list_orders_by_fit_then_recency_and_pages_without_gaps() {
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let member_id = member(&pool, "paging").await;
    let base = chrono::Utc::now();

    // Two at φ=0.5 (a tie), one at φ=0.9, and two unscored.
    for (i, score) in [Some(0.5_f64), Some(0.5), Some(0.9), None, None]
        .iter()
        .enumerate()
    {
        repo(&pool)
            .insert(&opportunity(
                member_id,
                OpportunityKind::JobPosting,
                &format!("Role {i}"),
                *score,
                base + chrono::Duration::seconds(i as i64),
            ))
            .await
            .unwrap();
    }

    // Whole set in one page, to establish the true order.
    let all = svc.list(member_id, None, 100, None).await.unwrap();
    assert_eq!(all.items.len(), 5);
    assert!(!all.has_more);
    assert!(all.next_cursor.is_none());

    // φ desc, NULLS LAST: the 0.9 first, the two 0.5 next, the unscored last.
    assert_eq!(all.items[0].fit_score, Some(0.9));
    assert_eq!(all.items[1].fit_score, Some(0.5));
    assert_eq!(all.items[2].fit_score, Some(0.5));
    assert!(all.items[3].fit_score.is_none(), "unscored must sort last");
    assert!(all.items[4].fit_score.is_none());
    // Within the 0.5 tie, newer first.
    assert!(all.items[1].discovered_at >= all.items[2].discovered_at);

    let mut walked: Vec<Uuid> = Vec::new();
    let mut cursor = None;
    for _ in 0..10 {
        let page: Page<_> = svc.list(member_id, None, 1, cursor).await.unwrap();
        assert!(page.items.len() <= 1);
        if page.items.is_empty() {
            assert!(!page.has_more);
            break;
        }
        walked.push(page.items[0].id);
        assert_eq!(page.has_more, page.next_cursor.is_some());
        match page.next_cursor {
            Some(raw) => cursor = OpportunityCursor::decode(&raw),
            None => break,
        }
    }
    // Every row exactly once, in the same order as the unpaged read — including
    // the NULL-fit_score boundary, which is where a naive keyset comparison
    // silently drops or repeats rows.
    assert_eq!(
        walked,
        all.items.iter().map(|o| o.id).collect::<Vec<_>>(),
        "keyset pagination must reproduce the unpaged order"
    );
}

/// A status filter pages just as correctly, with the same keyset boundary.
#[tokio::test]
async fn status_filter_pages_without_gaps() {
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let member_id = member(&pool, "statuspaging").await;
    let base = chrono::Utc::now();

    for (i, score) in [Some(0.4_f64), None, Some(0.8), None, Some(0.6)]
        .iter()
        .enumerate()
    {
        let mut opp = opportunity(
            member_id,
            OpportunityKind::JobPosting,
            &format!("Filtered {i}"),
            *score,
            base + chrono::Duration::seconds(i as i64),
        );
        if score.is_some() {
            opp.status = OpportunityStatus::Qualified;
        }
        repo(&pool).insert(&opp).await.unwrap();
    }

    let all = svc
        .list(member_id, Some(OpportunityStatus::Qualified), 100, None)
        .await
        .unwrap();
    assert_eq!(all.items.len(), 3);
    assert!(all
        .items
        .iter()
        .all(|o| o.status == OpportunityStatus::Qualified));

    let mut walked = Vec::new();
    let mut cursor = None;
    loop {
        let page = svc
            .list(member_id, Some(OpportunityStatus::Qualified), 1, cursor)
            .await
            .unwrap();
        if page.items.is_empty() {
            break;
        }
        walked.push(page.items[0].id);
        match page.next_cursor {
            Some(raw) => cursor = OpportunityCursor::decode(&raw),
            None => break,
        }
    }
    assert_eq!(walked, all.items.iter().map(|o| o.id).collect::<Vec<_>>());
}

/// `has_more` is a fact, not a guess: a page of exactly `limit` rows with more
/// available reports `has_more`, and the last page does not.
#[tokio::test]
async fn has_more_is_exact_at_the_page_boundary() {
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let member_id = member(&pool, "boundary").await;
    let now = chrono::Utc::now();

    for i in 0..3 {
        repo(&pool)
            .insert(&opportunity(
                member_id,
                OpportunityKind::JobPosting,
                &format!("Exact {i}"),
                Some(0.1 * (i as f64 + 1.0)),
                now + chrono::Duration::seconds(i),
            ))
            .await
            .unwrap();
    }

    let p1 = svc.list(member_id, None, 3, None).await.unwrap();
    assert_eq!(p1.items.len(), 3);
    assert!(!p1.has_more, "3 of 3 rows is not 'more'");
    assert!(p1.next_cursor.is_none());

    let p2 = svc.list(member_id, None, 2, None).await.unwrap();
    assert_eq!(p2.items.len(), 2);
    assert!(p2.has_more, "2 of 3 rows means more exist");
    assert!(p2.next_cursor.is_some());
}

/// Cursor round-trip, including the unscored (null fit_score) case and
/// rejection of junk.
#[test]
fn cursor_round_trips_and_rejects_junk() {
    // Truncate to microseconds: that is PostgreSQL's `timestamptz` resolution,
    // and the cursor is compared against a real column. A nanosecond-precision
    // cursor would encode fine and then never match a stored row, silently
    // skipping or repeating results at the keyset boundary.
    let now = chrono::DateTime::from_timestamp_micros(chrono::Utc::now().timestamp_micros())
        .expect("valid truncation");
    let id = Uuid::new_v4();

    for fit in [Some(0.875), Some(0.0), None] {
        let c = OpportunityCursor {
            fit,
            discovered_at: now,
            id,
        };
        let encoded = c.encode();
        // No `+` anywhere: chrono renders Utc with `Z`, and a literal `+` in a
        // query string would decode as a space and break the round trip.
        assert!(!encoded.contains('+'), "cursor must be URL-safe as-is");
        assert_eq!(OpportunityCursor::decode(&encoded), Some(c));
    }

    for junk in [
        "",
        "abc",
        "0.5|not-a-date|id",
        "0.5|date|not-a-uuid",
        "|||",
        "1|2|3|4",
        "nan||00000000-0000-0000-0000-000000000000",
    ] {
        assert_eq!(OpportunityCursor::decode(junk), None, "accepted {junk:?}");
    }
}

// ---------------------------------------------------------------------------
// Status vocabulary
// ---------------------------------------------------------------------------

/// `OpportunityStatus` is the canonical contract's enum verbatim, and every
/// value it accepts is writable to the column.
#[tokio::test]
async fn opportunity_status_vocabulary_matches_the_contract() {
    let expected = [
        "discovered",
        "qualified",
        "contacted",
        "conversation",
        "applied",
        "proposal_sent",
        "negotiation",
        "won",
        "closed_lost",
        "discarded",
        "dormant",
    ];
    let parsed: Vec<&str> = expected
        .iter()
        .map(|s| OpportunityStatus::parse(s).expect(s).as_str())
        .collect();
    assert_eq!(parsed, expected);
    // The pre-remediation values are gone; they appeared in no contract, no
    // migration and no design document.
    for stale in [
        "interviewing",
        "offer",
        "rejected",
        "closed",
        "withdrawn",
        "drafting",
        "parked",
    ] {
        assert!(
            OpportunityStatus::parse(stale).is_none(),
            "{stale} is not a contract OpportunityStatus"
        );
    }

    // Every value is accepted by the database column (TEXT, no CHECK).
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let member_id = member(&pool, "statuses").await;
    for (i, s) in expected.iter().enumerate() {
        let status = OpportunityStatus::parse(s).expect(s);
        let opp = opportunity(
            member_id,
            OpportunityKind::JobPosting,
            &format!("Status {i}"),
            Some(0.5),
            chrono::Utc::now(),
        );
        repo(&pool).insert(&opp).await.unwrap();
        let v = svc
            .qualify(
                member_id,
                opp.id,
                1,
                0.5,
                status,
                serde_json::json!({"skill": 0.5}),
            )
            .await
            .unwrap_or_else(|e| panic!("qualify to {s}: {e}"));
        // `qualify` returns the updated Opportunity, not a bare version.
        assert_eq!(v.version, 2, "version increments on qualify");

        let (stored, components, phi, last_eval): (
            String,
            serde_json::Value,
            f64,
            Option<chrono::DateTime<chrono::Utc>>,
        ) = sqlx::query_as(
            r#"SELECT status::text, phi_components, phi_score::double precision,
                      last_evaluated_at
               FROM lcc.opportunities WHERE id = $1::uuid"#,
        )
        .bind(opp.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(stored, *s);
        assert_eq!(components["skill"], 0.5, "φ breakdown is persisted");
        assert_eq!(phi, 0.5, "phi_score tracks fit_score");
        assert!(last_eval.is_some(), "qualify stamps last_evaluated_at");
    }

    // fit_score is validated to the contract's [0, 1] range.
    let opp = opportunity(
        member_id,
        OpportunityKind::JobPosting,
        "Out of range",
        None,
        chrono::Utc::now(),
    );
    repo(&pool).insert(&opp).await.unwrap();
    let err = svc
        .qualify(
            member_id,
            opp.id,
            1,
            1.5,
            OpportunityStatus::Qualified,
            serde_json::json!({}),
        )
        .await;
    assert!(matches!(err, Err(Error::Validation(_))), "got {err:?}");
}

/// `ApplicationStatus` is the `lcc.applications` CHECK constraint verbatim.
#[test]
fn application_status_matches_the_check_constraint() {
    for s in [
        "draft",
        "queued",
        "sent",
        "replied",
        "rejected",
        "withdrawn",
    ] {
        assert!(lcc_opportunity_svc::domain::ApplicationStatus::parse(s).is_some());
    }
    for s in ["applied", "interviewing", "offer", "accepted"] {
        assert!(
            lcc_opportunity_svc::domain::ApplicationStatus::parse(s).is_none(),
            "{s} is not a legal lcc.applications.status"
        );
    }
}

/// The legacy `opportunity_funnel` enum is documented and still parses, but is
/// deliberately not written by this service: the two enums do not map onto each
/// other, so deriving one from the other would invent a mapping.
#[test]
fn funnel_vocabulary_is_parsed_but_never_written() {
    for s in [
        "discovered",
        "qualified",
        "in_conversation",
        "proposed",
        "negotiating",
        "won",
        "lost",
    ] {
        assert!(OpportunityFunnel::parse(s).is_some(), "{s} must parse");
    }
    assert!(OpportunityFunnel::parse("contacted").is_none());
    assert!(OpportunityFunnel::parse("proposal_sent").is_none());
}

// ---------------------------------------------------------------------------
// Optimistic concurrency
// ---------------------------------------------------------------------------

#[tokio::test]
async fn qualify_enforces_optimistic_concurrency() {
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let member_id = member(&pool, "oc").await;
    let opp = opportunity(
        member_id,
        OpportunityKind::JobPosting,
        "Concurrent",
        None,
        chrono::Utc::now(),
    );
    repo(&pool).insert(&opp).await.unwrap();

    let v1 = svc
        .qualify(
            member_id,
            opp.id,
            1,
            0.4,
            OpportunityStatus::Qualified,
            serde_json::json!({}),
        )
        .await
        .unwrap();
    assert_eq!(v1.version, 2);

    // Replaying the same expected_version must not silently win.
    let stale = svc
        .qualify(
            member_id,
            opp.id,
            1,
            0.9,
            OpportunityStatus::Won,
            serde_json::json!({}),
        )
        .await;
    assert!(matches!(stale, Err(Error::Conflict(_))), "got {stale:?}");
}

// ---------------------------------------------------------------------------
// Proposal payload
// ---------------------------------------------------------------------------

#[tokio::test]
async fn proposal_is_persisted_as_a_client_proposal_application() {
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let member_id = member(&pool, "proposal").await;
    let opp = opportunity(
        member_id,
        OpportunityKind::Consulting,
        "Platform architecture review",
        Some(0.7),
        chrono::Utc::now(),
    );
    repo(&pool).insert(&opp).await.unwrap();

    let kb = Uuid::new_v4();
    let written = svc
        .propose(
            member_id,
            opp.id,
            "Review",
            "Full body",
            Some(250_000),
            Some("USD".into()),
            vec![kb],
        )
        .await
        .expect("draft proposal");

    assert_eq!(
        written.application_type,
        lcc_opportunity_svc::domain::ApplicationType::ClientProposal
    );
    assert_eq!(
        written.status,
        lcc_opportunity_svc::domain::ApplicationStatus::Draft
    );
    assert_eq!(written.submitted_at, None, "a draft has not been submitted");

    // The row lives in lcc.applications — there is no lcc.proposals to look in.
    let atype: String =
        sqlx::query_scalar(r#"SELECT application_type FROM lcc.applications WHERE id = $1::uuid"#)
            .bind(written.id)
            .fetch_one(&pool)
            .await
            .expect("row must be in lcc.applications");
    assert_eq!(atype, "client_proposal");

    // The drafted content is preserved verbatim in payload.
    let payload: serde_json::Value =
        sqlx::query_scalar(r#"SELECT payload FROM lcc.applications WHERE id = $1::uuid"#)
            .bind(written.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(payload["title"], "Review");
    assert_eq!(payload["body"], "Full body");
    assert_eq!(payload["price_cents"], 250_000);
    assert_eq!(payload["currency"], "USD");
    assert_eq!(payload["kb_refs"][0], kb.to_string());

    // And the payload helper round-trips the same shape.
    let built = ProposalPayload {
        title: "Review".into(),
        body: "Full body".into(),
        kb_refs: vec![kb],
        price_cents: Some(250_000),
        currency: Some("USD".into()),
    }
    .to_json();
    assert_eq!(built, payload);

    // An empty title or body is a validation error, not a stored empty draft.
    assert!(matches!(
        svc.propose(member_id, opp.id, "", "b", None, None, vec![])
            .await,
        Err(Error::Validation(_))
    ));
    assert!(matches!(
        svc.propose(member_id, opp.id, "t", "  ", None, None, vec![])
            .await,
        Err(Error::Validation(_))
    ));
}

/// The job-application payload carries the position from the opportunity's
/// title, not its category — the same rule the read path uses.
#[tokio::test]
async fn job_application_payload_records_the_position() {
    let Some(pool) = pool().await else { return };
    let svc = service(&pool);
    let member_id = member(&pool, "applypayload").await;
    let resume = Uuid::new_v4();
    let opp = opportunity(
        member_id,
        OpportunityKind::JobPosting,
        "Head of Data Platform",
        Some(0.6),
        chrono::Utc::now(),
    );
    repo(&pool).insert(&opp).await.unwrap();

    let app = svc
        .apply(member_id, opp.id, Some("Dear team".into()), Some(resume))
        .await
        .expect("draft application");

    let payload: serde_json::Value =
        sqlx::query_scalar(r#"SELECT payload FROM lcc.applications WHERE id = $1::uuid"#)
            .bind(app.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(payload["position"], "Head of Data Platform");
    assert_eq!(payload["cover_letter"], "Dear team");
    assert_eq!(payload["resume_doc_id"], resume.to_string());
}

// ---------------------------------------------------------------------------
// Schema contract this service depends on
// ---------------------------------------------------------------------------

/// The columns this service reads must keep existing with the types it decodes
/// them into. `lcc.applications.version` in particular is `BIGINT`, not `INT`,
/// and decoding it as `i32` fails at runtime rather than at compile time.
#[tokio::test]
async fn column_types_match_what_the_repository_decodes() {
    let Some(pool) = pool().await else { return };

    // A standalone async fn rather than a closure: a closure returning an
    // async block that borrows its own `&str` arguments cannot satisfy the
    // borrow checker without HRTB gymnastics.
    async fn expect(pool: &PgPool, table: &str, column: &str, ty: &str) {
        let got: String = sqlx::query_scalar(
            r#"SELECT data_type FROM information_schema.columns
               WHERE table_schema = 'lcc' AND table_name = $1 AND column_name = $2"#,
        )
        .bind(table)
        .bind(column)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|e| panic!("{table}.{column}: {e}"));
        assert_eq!(got, ty, "{table}.{column} type changed");
    }
    expect(&pool, "opportunities", "version", "integer").await;
    expect(&pool, "opportunities", "fit_score", "double precision").await;
    expect(&pool, "opportunities", "status", "text").await;
    expect(&pool, "opportunities", "source", "text").await;
    expect(&pool, "opportunities", "title", "text").await;
    expect(
        &pool,
        "opportunities",
        "discovered_at",
        "timestamp with time zone",
    )
    .await;
    expect(&pool, "applications", "version", "bigint").await;
    expect(&pool, "applications", "payload", "jsonb").await;
    expect(&pool, "applications", "application_type", "text").await;
    expect(&pool, "applications", "status", "text").await;
}
