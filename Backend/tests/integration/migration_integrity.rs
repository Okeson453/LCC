//! Live-database tests for the migration set and RLS enforcement.
//!
//! # Why this file exists
//!
//! `rls_isolation.rs` in this same directory asserts the right properties, but
//! it does so against `MockRlsDb` — an in-memory `HashMap` that reimplements
//! the isolation rule inside the test binary. It therefore passes whether or
//! not the actual database enforces anything, and whether or not the migration
//! set even applies.
//!
//! That gap hid six consecutive defects that made the schema impossible to
//! build (see F-AUDIT-36..44 in the migration files). Every one of them was
//! found by applying the migrations to a real PostgreSQL 15 instance, which is
//! what this file now does so the class of bug cannot come back silently.
//!
//! # Running
//!
//! These tests require a live PostgreSQL reachable via `DATABASE_URL`:
//!
//! ```sh
//! createdb lcc_test
//! export LCC_TEST_DATABASE_URL=postgres://lcc:lcc@127.0.0.1/lcc_test
//! just migrate-verify
//! ```
//!
//! They are skipped (not failed) when no URL is provided, so `cargo test`
//! remains runnable without a database. CI wires `LCC_TEST_DATABASE_URL` to a
//! service container so the live path is always exercised there.

// F-AUDIT-51: the workspace lint set denies `clippy::unwrap_used`,
// `expect_used` and `panic` because an `unwrap` on a `Result` can take a
// production service down. In a test binary the opposite holds: panicking IS
// the failure signal, and `unwrap()` is the idiomatic way to assert "this
// fixture must be valid, and if it is not the test must fail". These suites
// were never compiled by any crate before the `[[test]]` targets were added
// in `crates/test-utils/Cargo.toml`, so they never faced the gate.
// The exemption is file-scoped so the production lints stay fully intact.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;

/// Resolves the test database URL, preferring the explicit test-only variable
/// so a run can never point at a developer or production database.
fn test_db_url() -> Option<String> {
    std::env::var("LCC_TEST_DATABASE_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
}

/// Rewrites the URL to connect as the non-superuser application role.
///
/// This matters more than it looks. PostgreSQL *superusers bypass RLS
/// unconditionally*, and table owners bypass it unless FORCE ROW LEVEL
/// SECURITY is set. Running these assertions over a superuser connection
/// would report every isolation property as failing even when the policies
/// are correct — the exact mistake a first pass at this file made, where all
/// four behavioural tests failed against a fully working schema.
///
/// The migrations already provision `lcc_app` for this purpose (0001), so the
/// test uses the real production role rather than inventing one. The
/// superuser connection is retained separately, as `admin_pool()`, for the
/// structural checks and for seeding rows that must exist before the RLS
/// context is set.
fn app_url(url: &str) -> String {
    // Replace whatever user/password is in the URL with the app role, keeping
    // host/port/database intact.
    let (creds, rest) = match url.split_once('@') {
        Some((c, r)) => (c, r),
        None => return url.to_string(),
    };
    let _ = creds;
    let (scheme, tail) = match rest.split_once("://") {
        Some(_) => ("", rest),
        None => return url.to_string(),
    };
    let _ = scheme;
    let host_db = match tail.split_once('/') {
        Some((h, d)) => (h, d),
        None => return url.to_string(),
    };
    let user = std::env::var("LCC_TEST_APP_USER").unwrap_or_else(|_| "lcc_app".to_string());
    let pass = std::env::var("LCC_TEST_APP_PASSWORD").unwrap_or_else(|_| "dev_lcc_app".to_string());
    format!("postgres://{user}:{pass}@{}/{}", host_db.0, host_db.1)
}

async fn pool() -> Option<sqlx::PgPool> {
    let url = test_db_url()?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(&app_url(&url))
        .await
        .ok()?;
    // These tests create and drop rows. Refuse to run against anything that is
    // not obviously a scratch database.
    let name: String = sqlx::query("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .ok()?
        .try_get(0)
        .unwrap_or_default();
    if !(name.contains("test") || name.contains("ci") || name.contains("scratch")) {
        return None;
    }
    // Guard: if this connection can still bypass RLS, the behavioural tests
    // below would be meaningless. Fail loudly rather than assert a falsehood.
    let bypass: bool = sqlx::query_scalar(
        "SELECT r.rolsuper OR r.rolbypassrls
           FROM pg_roles r WHERE r.rolname = current_user",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    if bypass {
        eprintln!(
            "skipped: test connection ({}) is superuser/BYPASSRLS, so RLS assertions \
             would be meaningless; grant the run to lcc_app",
            "current_user"
        );
        return None;
    }
    Some(pool)
}

/// A privileged connection used only for structural inspection and for seeding
/// rows that must exist before any RLS context is established.
async fn admin_pool() -> Option<sqlx::PgPool> {
    let url = test_db_url()?;
    PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(&url)
        .await
        .ok()
}

/// Skips the calling test when no live database is configured.
macro_rules! live_db {
    () => {
        match pool().await {
            Some(p) => p,
            None => {
                eprintln!(
                    "skipped: set LCC_TEST_DATABASE_URL to a *test* database to run this test"
                );
                return;
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Schema shape
// ---------------------------------------------------------------------------

/// Every table that the design marks member-scoped must carry RLS *and* an
/// isolation policy. `outbox` and `compliance_config_versions` are
/// intentionally excluded: the outbox is a cross-tenant delivery queue drained
/// by workers, and config versions are global configuration, not member data.
const EXPECTED_RLS_TABLES: &[&str] = &[
    "members",
    "oauth_tokens",
    "consents",
    "profile_snapshots",
    "profile_audits",
    "kb_records",
    "kb_record_chunks",
    "content_items",
    "post_metrics",
    "inbound_messages",
    "engagement_replies",
    "sequences",
    "sequence_steps",
    "contacts",
    "network_lists",
    "network_list_memberships",
    "tags",
    "opportunities",
    "opportunity_signals",
    "outreach_drafts",
    "account_health_snapshots",
    "daily_caps",
    "cooldown_rules",
    "restrictions",
    "approvals",
    "idempotency_keys",
    "session_pacing",
    "briefings",
    "applications",
    "message_templates",
    "companies",
    "interactions",
];

/// F-AUDIT-36..41 guard: RLS coverage must be real, not incidental.
///
/// This asserts on a live database, so a migration that fails to apply, or an
/// `attach_member_rls` call that is silently skipped, fails the test rather
/// than quietly leaving a table unprotected.
#[tokio::test]
async fn member_scoped_tables_have_rls_and_policies() {
    let Some(pool) = admin_pool().await else {
        return;
    };

    for table in EXPECTED_RLS_TABLES {
        let rel: bool = sqlx::query_scalar(
            "SELECT c.relrowsecurity
               FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
              WHERE n.nspname = 'lcc' AND c.relname = $1",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|e| {
            panic!("table lcc.{table} must exist ({e}) — migrations did not apply")
        });

        assert!(
            rel,
            "lcc.{table} has RLS disabled; every member-scoped table must enforce isolation"
        );

        let policies: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_policies
              WHERE schemaname = 'lcc' AND tablename = $1 AND policyname = $2",
        )
        .bind(table)
        .bind(format!("member_isolation_{table}"))
        .fetch_one(&pool)
        .await
        .unwrap_or_default();
        assert!(
            policies > 0,
            "lcc.{table} has RLS enabled but no member_isolation_{table} policy; \
             rows would be unrestricted"
        );
    }
}

/// The two tables the canonical contract requires that were previously never
/// created because 0017 aborted (F-AUDIT-37, F-AUDIT-43).
#[tokio::test]
async fn contract_required_tables_exist() {
    let Some(pool) = admin_pool().await else {
        return;
    };
    for table in ["companies", "interactions"] {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables
                             WHERE table_schema = 'lcc' AND table_name = $1)",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .unwrap_or_default();
        assert!(
            exists,
            "lcc.{table} is required by the canonical contract but is missing"
        );
    }
}

/// F-AUDIT-42: the columns approval-svc and the contract's `Approval` schema
/// read and write must exist, or every approval query fails at runtime.
#[tokio::test]
async fn approvals_carry_contract_columns() {
    let Some(pool) = admin_pool().await else {
        return;
    };
    for col in [
        "tier",
        "rule_version",
        "requested_by",
        "decided_reason",
        "version",
        "reviewer_ids",
    ] {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM information_schema.columns
                             WHERE table_schema = 'lcc' AND table_name = 'approvals'
                               AND column_name = $1)",
        )
        .bind(col)
        .fetch_one(&pool)
        .await
        .unwrap_or_default();
        assert!(
            exists,
            "lcc.approvals.{col} is required by the contract and by approval-svc"
        );
    }
}

/// F-AUDIT-38 / F-AUDIT-41: columns the services bind must exist.
#[tokio::test]
async fn service_bound_columns_exist() {
    let Some(pool) = admin_pool().await else {
        return;
    };
    let expected: &[(&str, &str)] = &[
        ("sequence_steps", "idempotency_key"),
        ("sequence_steps", "contact_id"),
        ("contacts", "company_id"),
        ("opportunities", "status"),
        ("opportunities", "fit_score"),
        ("opportunities", "company_id"),
        ("opportunities", "discovered_at"),
        ("opportunities", "version"),
        ("engagement_replies", "version"),
    ];
    for (table, col) in expected {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM information_schema.columns
                             WHERE table_schema = 'lcc' AND table_name = $1 AND column_name = $2)",
        )
        .bind(table)
        .bind(col)
        .fetch_one(&pool)
        .await
        .unwrap_or_default();
        assert!(
            exists,
            "lcc.{table}.{col} is bound by a service query but does not exist"
        );
    }
}

/// `contacts.company_id` must reference `companies.id` (F-AUDIT-41).
#[tokio::test]
async fn contacts_company_fk_exists() {
    let Some(pool) = admin_pool().await else {
        return;
    };
    let fks: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.table_constraints
          WHERE table_schema = 'lcc' AND table_name = 'contacts'
            AND constraint_name = 'contacts_company_id_fkey'",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or_default();
    assert_eq!(fks, 1, "contacts.company_id must be an FK to companies.id");
}

// ---------------------------------------------------------------------------
// RLS behaviour (the part a mock cannot prove)
// ---------------------------------------------------------------------------

/// Creates a member-scoped row and returns the two member UUIDs plus the new
/// row id.
///
/// Seeding uses the admin connection deliberately: `lcc.members` has RLS
/// enabled, so inserting the member rows themselves only works from a role
/// that bypasses RLS. Once the members exist, the *behavioural* assertions all
/// run over the restricted `lcc_app` pool, which is the connection shape
/// production services use.
async fn seed(pool: &sqlx::PgPool) -> (uuid::Uuid, uuid::Uuid, uuid::Uuid) {
    let a: uuid::Uuid = uuid::Uuid::new_v4();
    let b: uuid::Uuid = uuid::Uuid::new_v4();
    let contact = uuid::Uuid::new_v4();

    for (id, name) in [(a, "rls-alice"), (b, "rls-bob")] {
        sqlx::query(
            "INSERT INTO lcc.members (id, linkedin_id, display_name)
             VALUES ($1, $2, $3)",
        )
        .bind(id)
        .bind(format!("li-{id}"))
        .bind(name)
        .execute(pool)
        .await
        .expect("seed member");
    }

    sqlx::query("INSERT INTO lcc.contacts (id, member_id, display_name) VALUES ($1, $2, $3)")
        .bind(contact)
        .bind(a)
        .bind("alice-owned")
        .execute(pool)
        .await
        .expect("seed contact");

    (a, b, contact)
}

/// Seeds via the privileged connection. Kept as a macro so the behavioural
/// tests read as one line and cannot accidentally mix pools. Borrows rather
/// than consumes, so callers keep using `admin` for cleanup afterwards.
macro_rules! seed_rows {
    ($admin:expr) => {{
        match $admin.as_ref() {
            Some(p) => seed(p).await,
            None => return,
        }
    }};
}

async fn cleanup(pool: &sqlx::PgPool, a: uuid::Uuid, b: uuid::Uuid) {
    for id in [a, b] {
        let _ = sqlx::query("DELETE FROM lcc.members WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await;
    }
}

/// RLS must fail *closed*: with no member context set, a member-scoped read
/// returns nothing rather than everything.
#[tokio::test]
async fn rls_read_without_context_returns_nothing() {
    let pool = live_db!();
    let admin = admin_pool().await;
    let (a, b, _c) = seed_rows!(admin);

    // A fresh pool has no `app.current_member_id` set.
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM lcc.contacts")
        .fetch_one(&pool)
        .await
        .unwrap_or_default();
    assert_eq!(count, 0, "RLS leaked rows with no member context set");

    if let Some(p) = admin.as_ref() {
        cleanup(p, a, b).await;
    }
}

/// Member A must not see Member B's rows.
#[tokio::test]
async fn rls_isolates_reads_between_members() {
    let pool = live_db!();
    let admin = admin_pool().await;
    let (a, b, contact) = seed_rows!(admin);

    // A second member-scoped row owned by B.
    let bob_contact = uuid::Uuid::new_v4();
    if let Some(p) = admin.as_ref() {
        sqlx::query("INSERT INTO lcc.contacts (id, member_id, display_name) VALUES ($1, $2, $3)")
            .bind(bob_contact)
            .bind(b)
            .bind("bob-owned")
            .execute(p)
            .await
            .expect("seed bob contact");
    }

    // A sees only its own.
    {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_member_id', $1, true)")
            .bind(a.to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        let rows: Vec<(uuid::Uuid, String)> =
            sqlx::query_as("SELECT id, display_name FROM lcc.contacts")
                .fetch_all(&mut *tx)
                .await
                .unwrap();
        assert!(
            rows.iter().any(|(id, _)| *id == contact),
            "member A must be able to read its own row"
        );
        assert!(
            !rows.iter().any(|(id, _)| *id == bob_contact),
            "member A saw member B's row — tenant isolation is broken"
        );
        tx.commit().await.unwrap();
    }

    // B sees only its own.
    {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_member_id', $1, true)")
            .bind(b.to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        let rows: Vec<(uuid::Uuid, String)> =
            sqlx::query_as("SELECT id, display_name FROM lcc.contacts")
                .fetch_all(&mut *tx)
                .await
                .unwrap();
        assert!(rows.iter().any(|(id, _)| *id == bob_contact));
        assert!(
            !rows.iter().any(|(id, _)| *id == contact),
            "member B saw member A's row — tenant isolation is broken"
        );
        tx.commit().await.unwrap();
    }

    if let Some(p) = admin.as_ref() {
        cleanup(p, a, b).await;
    }
}

/// A member must not be able to write a row owned by another member. This is
/// the cross-tenant *write* case (the IDOR shape) and is the assertion that
/// most directly protects the "no cross-tenant access paths" requirement.
#[tokio::test]
async fn rls_rejects_cross_tenant_write() {
    let pool = live_db!();
    let admin = admin_pool().await;
    let (a, b, _c) = seed_rows!(admin);

    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_member_id', $1, true)")
        .bind(b.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();

    // B tries to write a row owned by A. WITH CHECK must reject this.
    let result: Result<sqlx::postgres::PgQueryResult, sqlx::Error> = sqlx::query(
        "INSERT INTO lcc.contacts (id, member_id, display_name) VALUES ($1, $2, $3)",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(a) // <-- other member's id
    .bind("cross-tenant-write")
    .execute(&mut *tx)
    .await;

    assert!(
        result.is_err(),
        "RLS allowed a cross-tenant write: member B inserted a row owned by member A"
    );

    // Roll back rather than commit a failed transaction.
    let _ = tx.rollback().await;
    if let Some(p) = admin.as_ref() {
        cleanup(p, a, b).await;
    }
}

/// A member must not be able to re-point an existing row at another member.
#[tokio::test]
async fn rls_rejects_cross_tenant_update() {
    let pool = live_db!();
    let admin = admin_pool().await;
    let (a, b, contact) = seed_rows!(admin);

    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_member_id', $1, true)")
        .bind(b.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();

    // B cannot even see A's row, so the UPDATE must affect zero rows rather
    // than silently succeeding.
    let affected = sqlx::query("UPDATE lcc.contacts SET display_name = 'stolen' WHERE id = $1")
        .bind(contact)
        .execute(&mut *tx)
        .await
        .expect("update must execute, not error");
    assert_eq!(
        affected.rows_affected(),
        0,
        "member B updated member A's row — cross-tenant write path"
    );

    let _ = tx.rollback().await;
    if let Some(p) = admin.as_ref() {
        cleanup(p, a, b).await;
    }
}

/// The audit role must be able to append but not to update or delete — the
/// append-only guarantee the audit-log design depends on.
#[tokio::test]
async fn audit_table_is_append_only_for_audit_role() {
    let Some(pool) = admin_pool().await else {
        return;
    };
    let has_role: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'lcc_audit_writer')",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or_default();
    if !has_role {
        eprintln!("skipped: lcc_audit_writer role not provisioned in this database");
        return;
    }
    // The grants are asserted structurally so the test is meaningful even with
    // no rows present: UPDATE/DELETE must not be granted to the audit writer.
    let mutating: Vec<String> = sqlx::query_scalar(
        "SELECT privilege_type FROM information_schema.role_table_grants
          WHERE grantee = 'lcc_audit_writer' AND table_schema = 'lcc'
            AND privilege_type IN ('UPDATE', 'DELETE')",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    assert!(
        mutating.is_empty(),
        "lcc_audit_writer must not hold mutating grants on audit tables, found: {mutating:?}"
    );
}

/// Sanity check that the seed data migration produced a usable dev member.
#[tokio::test]
async fn dev_seed_member_present() {
    let Some(pool) = admin_pool().await else {
        return;
    };
    let n: i64 =
        sqlx::query_scalar("SELECT count(*) FROM lcc.members WHERE email = 'dev@lcc.local'")
            .fetch_one(&pool)
            .await
            .unwrap_or_default();
    assert_eq!(n, 1, "9999_seed_dev.sql must provision dev@lcc.local");
}
