//! Live-database tests for engagement-svc.
//!
//! These exist because the bugs this service had were invisible to a
//! type-checking build and to the existing smoke test: every one of them was a
//! query that named a table or column the schema does not have, or a response
//! assembled in memory rather than read back. `tools/extract_sql.py` catches the
//! first kind by PREPARE-ing each literal; only running the statement catches
//! the second, and only running it against rows proves the *values* are right.
//!
//! Run with:
//!   LCC_TEST_DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/lcc_test \
//!     cargo test -p lcc-engagement-svc --test db_repository
//!
//! Without that variable the database tests announce that they were skipped,
//! loudly, rather than passing quietly.

use std::time::Duration;

use lcc_engagement_svc::domain::{ActionType, InboundKind, TaskStatus};
use lcc_engagement_svc::error::Error;
use lcc_engagement_svc::repository::PgRepository;
use sqlx::PgPool;
use uuid::Uuid;

const TEST_URL_ENV: &str = "LCC_TEST_DATABASE_URL";

/// Connect, or `None` with a visible message. The unit tests in
/// `src/domain.rs` still run without a database, so the package is never
/// green-on-nothing; this only says which half did not execute.
async fn test_pool(test_name: &str) -> Option<PgPool> {
    let url = match std::env::var(TEST_URL_ENV) {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIPPED {test_name}: {TEST_URL_ENV} is not set, so no live-database \
                 assertions ran for it. Re-run with \
                 {TEST_URL_ENV}=postgres://postgres:postgres@127.0.0.1:5432/lcc_test"
            );
            return None;
        }
    };
    match sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&url)
        .await
    {
        Ok(p) => Some(p),
        Err(e) => panic!("{test_name}: could not connect to {url}: {e}"),
    }
}

/// Two members and one contact, torn down on drop. `member_id` is the tenant
/// key every query in this service is required to carry, so having two is what
/// makes the isolation tests meaningful.
struct Fixture {
    pool: PgPool,
    owner: Uuid,
    other: Uuid,
    contact: Uuid,
    inbox_id: Uuid,
    archived_inbox_id: Uuid,
    read_inbox_id: Uuid,
}

impl Fixture {
    async fn create(pool: PgPool) -> Self {
        let owner = Uuid::new_v4();
        let other = Uuid::new_v4();
        for (id, tag) in [(owner, "owner"), (other, "other")] {
            sqlx::query(
                "INSERT INTO lcc.members (id, linkedin_id, display_name) VALUES ($1, $2, $3)",
            )
            .bind(id)
            .bind(format!("li-{tag}-{}", id.simple()))
            .bind(format!("Member {tag}"))
            .execute(&pool)
            .await
            .expect("insert member");
        }

        let contact: (Uuid,) = sqlx::query_as(
            "INSERT INTO lcc.contacts (member_id, display_name) VALUES ($1, $2) RETURNING id",
        )
        .bind(owner)
        .bind("Ada Lovelace")
        .fetch_one(&pool)
        .await
        .expect("insert contact");

        // An inbound DM (unread, visible), an unread archived one, and a read
        // one, so every inbox filter has a row it must exclude and a row it
        // must include.
        let inbox_id = insert_inbound(
            &pool,
            owner,
            Some(contact.0),
            "dm",
            "Hi there",
            false,
            false,
        )
        .await;
        let archived_inbox_id = insert_inbound(
            &pool,
            owner,
            Some(contact.0),
            "comment",
            "Archived",
            false,
            true,
        )
        .await;
        let read_inbox_id = insert_inbound(
            &pool,
            owner,
            Some(contact.0),
            "mention",
            "Already read",
            true,
            false,
        )
        .await;

        Self {
            pool,
            owner,
            other,
            contact: contact.0,
            inbox_id,
            archived_inbox_id,
            read_inbox_id,
        }
    }

    fn repo(&self) -> PgRepository {
        PgRepository::new(self.pool.clone())
    }

    /// Insert a task with a specific queue position, going through the column
    /// names the service itself uses.
    async fn task(
        &self,
        member_id: Uuid,
        action_type: &str,
        status: &str,
        due_in_secs: Option<i64>,
        priority: Option<f64>,
        draft_body: Option<&str>,
    ) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO lcc.engagement_replies
                  (id, member_id, contact_id, action_type, status, priority_score,
                   due_at, draft_body, version, created_at)
               VALUES ($1,$2,$3,$4,$5,$6,
                       CASE WHEN $7::BIGINT IS NULL THEN NULL
                            ELSE NOW() + ($7 || ' seconds')::INTERVAL END,
                       $8, 1, NOW())"#,
        )
        .bind(id)
        .bind(member_id)
        .bind(self.contact)
        .bind(action_type)
        .bind(status)
        .bind(priority)
        .bind(due_in_secs)
        .bind(draft_body)
        .execute(&self.pool)
        .await
        .expect("insert engagement task");
        id
    }

    async fn cleanup(&self) {
        for m in [self.owner, self.other] {
            let _ = sqlx::query("DELETE FROM lcc.members WHERE id = $1")
                .bind(m)
                .execute(&self.pool)
                .await;
        }
    }
}

async fn insert_inbound(
    pool: &PgPool,
    member_id: Uuid,
    contact_id: Option<Uuid>,
    kind: &str,
    body: &str,
    read: bool,
    archived: bool,
) -> Uuid {
    let id: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO lcc.inbound_messages
              (member_id, contact_id, kind, body, thread_id, read_at, is_archived)
           VALUES ($1, $2, $3::lcc.inbound_kind, $4, 'thread-1',
                   CASE WHEN $5 THEN NOW() ELSE NULL END, $6)
           RETURNING id"#,
    )
    .bind(member_id)
    .bind(contact_id)
    .bind(kind)
    .bind(body)
    .bind(read)
    .bind(archived)
    .fetch_one(pool)
    .await
    .expect("insert inbound message");
    id.0
}

// ---------------------------------------------------------------------------
// The invented table
// ---------------------------------------------------------------------------

/// `lcc.engagement_inbox` does not exist and must not be reintroduced by a
/// migration to silence the old 500. If someone ever adds it, the fix here was
/// the wrong one and this fails.
#[tokio::test]
async fn engagement_inbox_table_does_not_exist() {
    let Some(pool) = test_pool("engagement_inbox_table_does_not_exist").await else {
        return;
    };
    let (exists,): (Option<String>,) =
        sqlx::query_as("SELECT to_regclass('lcc.engagement_inbox')::TEXT")
            .fetch_one(&pool)
            .await
            .expect("to_regclass");
    assert_eq!(
        exists, None,
        "lcc.engagement_inbox exists again. The inbox must read lcc.inbound_messages \
         (migration 0007); a table created to satisfy the old query is the failure \
         mode this test guards against."
    );
}

/// The table the inbox is actually read from, and the enum it projects, are the
/// real ones. A 200 built on any other table is a 200 built on a fiction.
#[tokio::test]
async fn inbox_reads_inbound_messages_and_its_real_kind_enum() {
    let Some(pool) = test_pool("inbox_reads_inbound_messages_and_its_real_kind_enum").await else {
        return;
    };
    let (n,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM information_schema.columns
          WHERE table_schema='lcc' AND table_name='inbound_messages'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(n > 0, "lcc.inbound_messages is missing");

    let labels: Vec<String> = sqlx::query_scalar(
        "SELECT e.enumlabel FROM pg_enum e
           JOIN pg_type t ON t.oid = e.enumtypid
          WHERE t.typname = 'inbound_kind'
          ORDER BY e.enumsortorder",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        labels,
        vec![
            "comment",
            "dm",
            "mention",
            "connection_request_inbound",
            "post_reaction"
        ],
        "lcc.inbound_kind is not the vocabulary the domain model pins"
    );
}

// ---------------------------------------------------------------------------
// Engagement tasks
// ---------------------------------------------------------------------------

/// The full list projection, including the two columns whose types needed an
/// explicit cast (`priority_score` NUMERIC -> f64, `version` BIGINT -> i32).
/// sqlx refuses to decode those pairs, so an uncast query fails at runtime
/// while still passing PREPARE.
#[tokio::test]
async fn list_tasks_reads_every_projected_field() {
    let Some(pool) = test_pool("list_tasks_reads_every_projected_field").await else {
        return;
    };
    let f = Fixture::create(pool).await;
    let id = f
        .task(
            f.owner,
            "congratulate",
            "drafted",
            Some(60),
            Some(0.8750),
            Some("Congrats on the launch!"),
        )
        .await;

    let rows = f
        .repo()
        .list_tasks(f.owner, None, 50)
        .await
        .expect("list_tasks");
    f.cleanup().await;

    let t = rows.iter().find(|t| t.id == id).expect("task present");
    assert_eq!(t.member_id, f.owner);
    assert_eq!(t.contact_id, Some(f.contact));
    assert_eq!(t.action_type, ActionType::Congratulate);
    assert_eq!(t.status, TaskStatus::Drafted);
    // NUMERIC(5,4) -> f64 must survive, not be dropped or read as an int.
    assert_eq!(t.priority_score, Some(0.875));
    assert_eq!(t.draft_body.as_deref(), Some("Congrats on the launch!"));
    assert_eq!(t.version, 1);
    assert!(t.due_at.is_some());
}

/// A NULL `priority_score` must stay NULL. `EngagementTask.priority_score` is
/// documented as "ρ if model fitted, else NULL"; coercing it to 0.0 would make
/// an unfitted task look like the lowest possible priority.
#[tokio::test]
async fn null_priority_score_stays_null() {
    let Some(pool) = test_pool("null_priority_score_stays_null").await else {
        return;
    };
    let f = Fixture::create(pool).await;
    f.task(f.owner, "reply", "queued", None, None, None).await;
    let rows = f.repo().list_tasks(f.owner, None, 50).await.expect("list");
    f.cleanup().await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].priority_score, None);
    assert_eq!(rows[0].draft_body, None);
    assert_eq!(rows[0].due_at, None);
    assert_eq!(rows[0].completed_at, None);
}

/// Queue order: soonest `due_at` first, then highest priority, then newest.
/// The two branches of the previous implementation disagreed with each other,
/// so the same task moved depending on whether a status filter was passed.
#[tokio::test]
async fn queue_order_is_due_at_then_priority_then_created() {
    let Some(pool) = test_pool("queue_order_is_due_at_then_priority_then_created").await else {
        return;
    };
    // `Fixture::create` consumes the pool; keep a handle for the tie-breaking
    // UPDATE below.
    let pool2 = pool.clone();
    let f = Fixture::create(pool).await;
    let late = f
        .task(f.owner, "reply", "queued", Some(600), Some(0.9), None)
        .await;
    let soon_low = f
        .task(f.owner, "reply", "queued", Some(60), Some(0.1), None)
        .await;
    let soon_high = f
        .task(f.owner, "reply", "queued", Some(60), Some(0.9), None)
        .await;
    let undated = f
        .task(f.owner, "reply", "queued", None, Some(0.9), None)
        .await;

    // The two "soon" rows were inserted a few microseconds apart, so their
    // `due_at` values differ and the `due_at` key decides the order before
    // priority is ever consulted. Pin them to one timestamp so the test
    // actually exercises the tie it is written to exercise, then prove the
    // pin landed instead of assuming it did.
    let pinned = chrono::DateTime::from_timestamp_micros(chrono::Utc::now().timestamp_micros())
        .expect("valid timestamp");
    sqlx::query("UPDATE lcc.engagement_replies SET due_at = $1 WHERE id = ANY($2::uuid[])")
        .bind(pinned)
        .bind(vec![soon_low, soon_high])
        .execute(&pool2)
        .await
        .expect("pin due_at for the tied pair");
    let (ties,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM lcc.engagement_replies WHERE id = ANY($1::uuid[]) AND due_at = $2",
    )
    .bind(vec![soon_low, soon_high])
    .bind(pinned)
    .fetch_one(&pool2)
    .await
    .expect("verify pin");
    assert_eq!(ties, 2, "the tied pair must share one due_at");

    let unfiltered = f.repo().list_tasks(f.owner, None, 50).await.expect("list");
    let filtered = f
        .repo()
        .list_tasks(f.owner, Some(TaskStatus::Queued), 50)
        .await
        .expect("list filtered");
    f.cleanup().await;

    let order = |v: &[lcc_engagement_svc::domain::EngagementTask]| -> Vec<Uuid> {
        v.iter().map(|t| t.id).collect()
    };
    let want = vec![soon_high, soon_low, late, undated];
    assert_eq!(order(&unfiltered), want, "unfiltered queue order");
    assert_eq!(
        order(&filtered),
        want,
        "a status filter must not reorder the queue"
    );
}

#[tokio::test]
async fn status_filter_selects_only_that_status() {
    let Some(pool) = test_pool("status_filter_selects_only_that_status").await else {
        return;
    };
    let f = Fixture::create(pool).await;
    let queued = f.task(f.owner, "reply", "queued", None, None, None).await;
    f.task(f.owner, "reply", "dismissed", None, None, None)
        .await;

    let rows = f
        .repo()
        .list_tasks(f.owner, Some(TaskStatus::Queued), 50)
        .await
        .expect("list filtered");
    f.cleanup().await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, queued);
}

#[tokio::test]
async fn limit_is_applied() {
    let Some(pool) = test_pool("limit_is_applied").await else {
        return;
    };
    let f = Fixture::create(pool).await;
    for _ in 0..3 {
        f.task(f.owner, "reply", "queued", None, None, None).await;
    }
    let rows = f.repo().list_tasks(f.owner, None, 2).await.expect("list");
    f.cleanup().await;
    assert_eq!(rows.len(), 2);
}

// ---------------------------------------------------------------------------
// Tenant isolation — the invariant that must not regress
// ---------------------------------------------------------------------------

/// A member only ever sees their own tasks. The `member_id` predicate is in
/// every list, read and update statement; this is the test that would fail if
/// one of them lost it.
#[tokio::test]
async fn tasks_are_tenant_scoped_on_every_read_and_write() {
    let Some(pool) = test_pool("tasks_are_tenant_scoped_on_every_read_and_write").await else {
        return;
    };
    let f = Fixture::create(pool).await;
    let mine = f.task(f.owner, "reply", "queued", None, None, None).await;
    let theirs = f.task(f.other, "reply", "queued", None, None, None).await;

    let repo = f.repo();
    let seen = repo.list_tasks(f.owner, None, 50).await.expect("list");
    assert_eq!(
        seen.iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![mine],
        "list_tasks leaked another member's task"
    );
    assert!(seen.iter().all(|t| t.member_id == f.owner));

    // Point read by id, from the wrong member.
    assert!(
        repo.get_task(f.owner, theirs).await.expect("get").is_none(),
        "get_task returned another member's task"
    );
    assert_eq!(
        repo.get_task(f.owner, mine)
            .await
            .expect("get")
            .map(|t| t.id),
        Some(mine)
    );

    // Every guarded UPDATE, aimed at the other member's row.
    assert!(repo
        .update_task(
            f.owner,
            theirs,
            1,
            Some("hijack"),
            Some(TaskStatus::Drafted)
        )
        .await
        .expect("update")
        .is_none());
    assert!(repo
        .complete_task(f.owner, theirs, 1)
        .await
        .expect("complete")
        .is_none());
    assert!(repo
        .dismiss_task(f.owner, theirs, 1)
        .await
        .expect("dismiss")
        .is_none());

    // And the victim's row is untouched.
    let (status, version, draft): (String, i64, Option<String>) = sqlx::query_as(
        "SELECT status, version, draft_body FROM lcc.engagement_replies WHERE id = $1",
    )
    .bind(theirs)
    .fetch_one(&f.pool)
    .await
    .unwrap();
    assert_eq!(status, "queued");
    assert_eq!(version, 1);
    assert_eq!(draft, None, "another member's draft was overwritten");
    f.cleanup().await;
}

#[tokio::test]
async fn inbox_is_tenant_scoped() {
    let Some(pool) = test_pool("inbox_is_tenant_scoped").await else {
        return;
    };
    let f = Fixture::create(pool).await;
    insert_inbound(&f.pool, f.other, None, "dm", "not yours", false, false).await;

    let mine = f.repo().inbox(f.owner, 50, false).await.expect("inbox");
    f.cleanup().await;
    assert!(mine.iter().all(|m| m.id != f.archived_inbox_id));
    // Exactly the owner's three fixtures minus the archived one.
    assert_eq!(mine.len(), 2);
}

// ---------------------------------------------------------------------------
// Mutations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn insert_then_read_round_trips_every_field() {
    let Some(pool) = test_pool("insert_then_read_round_trips_every_field").await else {
        return;
    };
    let f = Fixture::create(pool).await;
    let due = chrono::Utc::now() + chrono::Duration::hours(3);
    let t = lcc_engagement_svc::domain::EngagementTask {
        id: Uuid::new_v4(),
        member_id: f.owner,
        contact_id: Some(f.contact),
        target_post_id: Some("urn:li:share:42".into()),
        action_type: ActionType::FollowUp,
        status: TaskStatus::Queued,
        priority_score: Some(0.5),
        due_at: Some(due),
        draft_body: None,
        completed_at: None,
        version: 1,
        created_at: chrono::Utc::now(),
    };
    f.repo().insert_task(&t).await.expect("insert");

    let back = f
        .repo()
        .get_task(f.owner, t.id)
        .await
        .expect("get")
        .expect("row present");
    f.cleanup().await;

    // PostgreSQL `timestamptz` stores microseconds; Rust `Utc::now()` carries
    // nanoseconds. Compare the fields the database can actually represent.
    assert_eq!(back.id, t.id);
    assert_eq!(back.member_id, t.member_id);
    assert_eq!(back.contact_id, t.contact_id);
    assert_eq!(back.target_post_id, t.target_post_id);
    assert_eq!(back.action_type, t.action_type);
    assert_eq!(back.status, t.status);
    assert_eq!(back.priority_score, t.priority_score);
    assert_eq!(back.draft_body, t.draft_body);
    assert_eq!(back.completed_at, t.completed_at);
    assert_eq!(back.version, t.version);
    assert_eq!(
        back.created_at.timestamp_micros(),
        t.created_at.timestamp_micros(),
        "created_at must round-trip at database resolution"
    );
    assert_eq!(
        back.due_at.map(|d| d.timestamp_micros()),
        t.due_at.map(|d| d.timestamp_micros()),
        "due_at must round-trip at database resolution"
    );
}

/// The core anti-fabrication test. The previous service answered 200 from an
/// in-memory object with `action_type: Reply`, `contact_id: None` and
/// `created_at: Utc::now()`, whatever the row actually held. Here a
/// `congratulate` task with a contact and an old `created_at` must come back
/// describing itself truthfully.
#[tokio::test]
async fn update_draft_returns_the_stored_row_not_a_synthesised_one() {
    let Some(pool) = test_pool("update_draft_returns_the_stored_row_not_a_synthesised_one").await
    else {
        return;
    };
    let f = Fixture::create(pool).await;
    let id = f
        .task(f.owner, "congratulate", "queued", None, None, None)
        .await;
    let before: (DateTimeStamp,) =
        sqlx::query_as("SELECT created_at FROM lcc.engagement_replies WHERE id = $1")
            .bind(id)
            .fetch_one(&f.pool)
            .await
            .unwrap();

    let svc = test_service(&f).await;
    let updated = svc
        .update_draft(f.owner, id, 1, "Well done!")
        .await
        .expect("update_draft");

    f.cleanup().await;

    assert_eq!(updated.id, id);
    assert_eq!(updated.status, TaskStatus::Drafted);
    assert_eq!(updated.version, 2);
    assert_eq!(updated.draft_body.as_deref(), Some("Well done!"));
    // The three fields the old implementation invented:
    assert_eq!(
        updated.action_type,
        ActionType::Congratulate,
        "the response claimed ActionType::Reply regardless of the row"
    );
    assert_eq!(
        updated.contact_id,
        Some(f.contact),
        "the response claimed contact_id: None regardless of the row"
    );
    assert_eq!(
        updated.created_at, before.0,
        "the response claimed created_at: Utc::now() regardless of the row"
    );
}

/// `complete` writes `sent`, the canonical terminal state. The previous
/// implementation wrote `completed`, which is in neither design §11.12's CHECK
/// list, nor the contract enum, nor the proto enum.
#[tokio::test]
async fn complete_writes_the_canonical_sent_status() {
    let Some(pool) = test_pool("complete_writes_the_canonical_sent_status").await else {
        return;
    };
    let f = Fixture::create(pool).await;
    let id = f
        .task(f.owner, "reply", "approved", None, None, Some("draft"))
        .await;

    let svc = test_service(&f).await;
    let done = svc.complete(f.owner, id, 1).await.expect("complete");
    f.cleanup().await;

    assert_eq!(done.status, TaskStatus::Sent);
    assert_eq!(done.version, 2);
    assert!(done.completed_at.is_some(), "completed_at was not stamped");
    assert_eq!(done.draft_body.as_deref(), Some("draft"), "draft was lost");
}

/// `dismiss` writes `dismissed`, not the previous `skipped`.
#[tokio::test]
async fn dismiss_writes_the_canonical_dismissed_status() {
    let Some(pool) = test_pool("dismiss_writes_the_canonical_dismissed_status").await else {
        return;
    };
    let f = Fixture::create(pool).await;
    let id = f.task(f.owner, "like", "queued", None, None, None).await;

    let svc = test_service(&f).await;
    let gone = svc.dismiss(f.owner, id, 1).await.expect("dismiss");
    f.cleanup().await;

    assert_eq!(gone.status, TaskStatus::Dismissed);
    assert_eq!(gone.version, 2);
    assert_eq!(gone.completed_at, None, "dismissal is not a completion");
}

/// Migration 0021 makes the column itself fail closed. This is the test that
/// says the vocabulary is enforced by the database, not only by the service.
#[tokio::test]
async fn the_column_rejects_statuses_outside_the_canonical_vocabulary() {
    let Some(pool) =
        test_pool("the_column_rejects_statuses_outside_the_canonical_vocabulary").await
    else {
        return;
    };
    let f = Fixture::create(pool).await;
    let id = f.task(f.owner, "reply", "queued", None, None, None).await;

    for bad in ["completed", "skipped", "bogus"] {
        let res = sqlx::query("UPDATE lcc.engagement_replies SET status = $2 WHERE id = $1")
            .bind(id)
            .bind(bad)
            .execute(&f.pool)
            .await;
        assert!(
            res.is_err(),
            "status {bad:?} was accepted by the column; the old service wrote exactly these"
        );
    }
    f.cleanup().await;
}

#[tokio::test]
async fn the_column_rejects_action_types_outside_the_canonical_vocabulary() {
    let Some(pool) =
        test_pool("the_column_rejects_action_types_outside_the_canonical_vocabulary").await
    else {
        return;
    };
    let f = Fixture::create(pool).await;
    let id = f.task(f.owner, "reply", "queued", None, None, None).await;

    for bad in ["connect", "remind", "share", "publish", "custom_note"] {
        let res = sqlx::query("UPDATE lcc.engagement_replies SET action_type = $2 WHERE id = $1")
            .bind(id)
            .bind(bad)
            .execute(&f.pool)
            .await;
        assert!(
            res.is_err(),
            "action_type {bad:?} was accepted by the column"
        );
    }
    f.cleanup().await;
}

// ---------------------------------------------------------------------------
// Optimistic concurrency, and not being an existence oracle
// ---------------------------------------------------------------------------

/// A stale version on a task the caller owns is 409. The old code returned 409
/// for this, another member's task and a non-existent id alike, so a caller
/// could probe for the existence of a task id they did not own.
#[tokio::test]
async fn stale_version_conflicts_but_a_foreign_task_is_not_found() {
    let Some(pool) = test_pool("stale_version_conflicts_but_a_foreign_task_is_not_found").await
    else {
        return;
    };
    let f = Fixture::create(pool).await;
    let mine = f
        .task(f.owner, "reply", "drafted", None, None, Some("v1"))
        .await;
    let theirs = f
        .task(f.other, "reply", "drafted", None, None, Some("v1"))
        .await;
    let missing = Uuid::new_v4();

    let svc = test_service(&f).await;

    let stale = svc.update_draft(f.owner, mine, 99, "x").await.unwrap_err();
    assert!(
        matches!(stale, Error::Conflict(_)),
        "a stale version on an owned task must be 409, got {stale:?}"
    );

    // Same call, same wrong version, but aimed at a row this member does not
    // own. The status must NOT reveal that the row exists.
    let foreign = svc
        .update_draft(f.owner, theirs, 99, "x")
        .await
        .unwrap_err();
    assert!(
        matches!(foreign, Error::NotFound(_)),
        "another member's task must be 404, got {foreign:?}"
    );

    let absent = svc
        .update_draft(f.owner, missing, 99, "x")
        .await
        .unwrap_err();
    assert!(
        matches!(absent, Error::NotFound(_)),
        "an unknown id must be 404, got {absent:?}"
    );
    assert!(
        !foreign.to_string().contains("version"),
        "the 404 body must not leak the stored version"
    );

    let completed = svc.complete(f.owner, theirs, 1).await.unwrap_err();
    assert!(
        matches!(completed, Error::NotFound(_)),
        "completing another member's task must be 404, got {completed:?}"
    );
    assert!(matches!(
        svc.dismiss(f.owner, theirs, 1).await.unwrap_err(),
        Error::NotFound(_)
    ));
    assert!(matches!(
        svc.complete(f.owner, missing, 1).await.unwrap_err(),
        Error::NotFound(_)
    ));

    f.cleanup().await;
}

// ---------------------------------------------------------------------------
// Inbox
// ---------------------------------------------------------------------------

/// `preview` is derived from `body` by truncation, `unread` is `read_at IS
/// NULL`, and `kind` is the stored `lcc.inbound_kind` value verbatim. Each of
/// these was previously a column that does not exist.
#[tokio::test]
async fn inbox_derives_preview_and_unread_from_real_columns() {
    let Some(pool) = test_pool("inbox_derives_preview_and_unread_from_real_columns").await else {
        return;
    };
    let f = Fixture::create(pool).await;
    let long = "x".repeat(500);
    let long_id =
        insert_inbound(&f.pool, f.owner, Some(f.contact), "dm", &long, false, false).await;

    let all = f.repo().inbox(f.owner, 50, false).await.expect("inbox");
    f.cleanup().await;

    let unread_dm = all.iter().find(|m| m.id == f.inbox_id).expect("dm present");
    assert_eq!(unread_dm.kind, InboundKind::Dm);
    assert_eq!(unread_dm.preview, "Hi there");
    assert!(unread_dm.unread);
    assert_eq!(unread_dm.contact_id, Some(f.contact));
    assert_eq!(unread_dm.thread_id.as_deref(), Some("thread-1"));

    let long_msg = all.iter().find(|m| m.id == long_id).expect("long present");
    assert_eq!(
        long_msg.preview.chars().count(),
        280,
        "preview must truncate"
    );
    assert!(long_msg.preview.chars().all(|c| c == 'x'));

    let read_msg = all
        .iter()
        .find(|m| m.id == f.read_inbox_id)
        .expect("read present");
    assert!(!read_msg.unread, "read_at IS NOT NULL must mean read");
    assert_eq!(read_msg.kind, InboundKind::Mention);

    assert!(
        all.iter().all(|m| m.id != f.archived_inbox_id),
        "an archived message must not be listed"
    );
    assert!(
        all.windows(2).all(|w| w[0].received_at >= w[1].received_at),
        "the inbox must be newest-first"
    );
}

#[tokio::test]
async fn unread_only_filters_on_read_at() {
    let Some(pool) = test_pool("unread_only_filters_on_read_at").await else {
        return;
    };
    let f = Fixture::create(pool).await;
    let unread = f.repo().inbox(f.owner, 50, true).await.expect("inbox");
    f.cleanup().await;
    assert!(unread.iter().all(|m| m.unread));
    assert!(unread.iter().all(|m| m.id != f.read_inbox_id));
    assert!(unread.iter().any(|m| m.id == f.inbox_id));
}

/// A nullable `inbound_messages.contact_id` must not break the projection, and
/// must not be turned into a fabricated contact.
#[tokio::test]
async fn inbox_tolerates_a_null_contact_id() {
    let Some(pool) = test_pool("inbox_tolerates_a_null_contact_id").await else {
        return;
    };
    let f = Fixture::create(pool).await;
    let orphan = insert_inbound(&f.pool, f.owner, None, "post_reaction", "❤️", false, false).await;
    let all = f.repo().inbox(f.owner, 50, false).await.expect("inbox");
    f.cleanup().await;
    let m = all.iter().find(|m| m.id == orphan).expect("present");
    assert_eq!(m.contact_id, None);
    assert_eq!(m.kind, InboundKind::PostReaction);
}

#[tokio::test]
async fn mark_read_sets_read_at_and_is_idempotent_and_tenant_scoped() {
    let Some(pool) = test_pool("mark_read_sets_read_at_and_is_idempotent_and_tenant_scoped").await
    else {
        return;
    };
    let f = Fixture::create(pool).await;
    let repo = f.repo();

    assert!(repo
        .mark_inbox_read(f.owner, f.inbox_id)
        .await
        .expect("mark"));
    let stamped: (Option<chrono::DateTime<chrono::Utc>>,) =
        sqlx::query_as("SELECT read_at FROM lcc.inbound_messages WHERE id = $1")
            .bind(f.inbox_id)
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert!(stamped.0.is_some(), "read_at was not stamped");

    // Second call matches no row (read_at IS NOT NULL) and is a no-op, not an
    // error: the endpoint is idempotent.
    assert!(!repo
        .mark_inbox_read(f.owner, f.inbox_id)
        .await
        .expect("mark"));

    // Another member's message changes nothing.
    assert!(!repo
        .mark_inbox_read(f.other, f.inbox_id)
        .await
        .expect("mark"));
    assert!(!repo
        .mark_inbox_read(f.owner, Uuid::new_v4())
        .await
        .expect("mark"));

    let still: (Option<chrono::DateTime<chrono::Utc>>,) =
        sqlx::query_as("SELECT read_at FROM lcc.inbound_messages WHERE id = $1")
            .bind(f.inbox_id)
            .fetch_one(&f.pool)
            .await
            .unwrap();
    f.cleanup().await;
    assert_eq!(
        still.0, stamped.0,
        "a foreign mark-read must not move read_at"
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

type DateTimeStamp = chrono::DateTime<chrono::Utc>;

/// The service needs a redis pool for its event publishing, and the test
/// database has no redis. `deadpool_redis` builds a pool lazily, so a pool
/// pointed at an unreachable address is never dialled on these paths — and
/// `publish_event` swallows the failure, which is the behaviour under test
/// elsewhere. A deadpool pool is constructed directly rather than via config so
/// the test does not depend on `REDIS_URL`.
async fn test_service(f: &Fixture) -> lcc_engagement_svc::service::Service {
    let redis = deadpool_redis::Config::from_url("redis://127.0.0.1:1/")
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("redis pool config");
    lcc_engagement_svc::service::Service::new(f.repo(), redis)
}
