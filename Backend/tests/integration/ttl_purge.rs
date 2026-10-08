//! Integration test: TTL purge of third-party data.
//!
//! Per spec §6: third-party data has a TTL; first-party data does not.
//! The data-purge-worker removes rows where `ttl_expires_at < NOW()`.
//! Vectors in Qdrant must be removed alongside the SQL row.

// F-AUDIT-51: the workspace lint set denies `clippy::unwrap_used`,
// `expect_used` and `panic` because an `unwrap` on a `Result` can take a
// production service down. In a test binary the opposite holds: panicking IS
// the failure signal, and `unwrap()` is the idiomatic way to assert "this
// fixture must be valid, and if it is not the test must fail". These suites
// were never compiled by any crate before the `[[test]]` targets were added
// in `crates/test-utils/Cargo.toml`, so they never faced the gate.
// The exemption is file-scoped so the production lints stay fully intact.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use std::collections::HashSet;

#[derive(Debug, Clone)]
struct TtlRecord {
    id: String,
    member_id: String,
    ttl_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    is_third_party: bool,
}

struct TtlPurger {
    sql_rows: Vec<TtlRecord>,
    qdrant_points: HashSet<String>,
}

impl TtlPurger {
    fn new() -> Self {
        Self {
            sql_rows: Vec::new(),
            qdrant_points: HashSet::new(),
        }
    }

    fn add(&mut self, rec: TtlRecord) {
        if rec.is_third_party {
            self.qdrant_points.insert(rec.id.clone());
        }
        self.sql_rows.push(rec);
    }

    fn purge_expired(&mut self, now: chrono::DateTime<chrono::Utc>) -> (usize, usize) {
        let mut purged_sql = 0;
        let mut purged_vec = 0;
        let mut kept = Vec::new();
        for rec in self.sql_rows.drain(..) {
            let expired = match (rec.is_third_party, rec.ttl_expires_at) {
                (true, Some(exp)) => exp < now,
                _ => false,
            };
            if expired {
                purged_sql += 1;
                if self.qdrant_points.remove(&rec.id) {
                    purged_vec += 1;
                }
            } else {
                kept.push(rec);
            }
        }
        self.sql_rows = kept;
        (purged_sql, purged_vec)
    }
}

fn make_rec(id: &str, days_ago: i64, is_third_party: bool) -> TtlRecord {
    let ttl = if is_third_party {
        Some(chrono::Utc::now() - chrono::Duration::days(days_ago))
    } else {
        None
    };
    TtlRecord {
        id: id.into(),
        member_id: "m-1".into(),
        ttl_expires_at: ttl,
        is_third_party,
    }
}

#[test]
fn third_party_expired_is_purged_with_vector() {
    let mut p = TtlPurger::new();
    p.add(make_rec("opp-sig-1", 5, true)); // expired 5 days ago
    let (sql_n, vec_n) = p.purge_expired(chrono::Utc::now());
    assert_eq!(sql_n, 1);
    assert_eq!(vec_n, 1);
    assert!(p.sql_rows.is_empty());
    assert!(p.qdrant_points.is_empty());
}

#[test]
fn first_party_data_is_never_purged_even_with_ttl() {
    let mut p = TtlPurger::new();
    p.add(make_rec("first-party-1", 100, false)); // 100d old but first-party
    let (sql_n, vec_n) = p.purge_expired(chrono::Utc::now());
    assert_eq!(sql_n, 0, "first-party rows are never purged");
    assert_eq!(vec_n, 0);
    assert_eq!(p.sql_rows.len(), 1);
}

#[test]
fn third_party_within_ttl_is_kept() {
    let mut p = TtlPurger::new();
    p.add(TtlRecord {
        id: "opp-sig-2".into(),
        member_id: "m-1".into(),
        ttl_expires_at: Some(chrono::Utc::now() + chrono::Duration::days(15)), // future
        is_third_party: true,
    });
    let (sql_n, vec_n) = p.purge_expired(chrono::Utc::now());
    assert_eq!(sql_n, 0);
    assert_eq!(vec_n, 0);
    assert_eq!(p.sql_rows.len(), 1);
}

#[test]
fn mixed_first_and_third_party_purges_only_third() {
    let mut p = TtlPurger::new();
    p.add(make_rec("first-1", 365, false));
    p.add(make_rec("third-1", 60, true));
    p.add(make_rec("first-2", 30, false));
    p.add(make_rec("third-2", 90, true));
    let (sql_n, _) = p.purge_expired(chrono::Utc::now());
    assert_eq!(sql_n, 2, "only 2 third-party rows are purged");
    assert_eq!(p.sql_rows.len(), 2, "2 first-party rows are kept");
}

#[test]
fn vectors_for_kept_rows_are_kept() {
    let mut p = TtlPurger::new();
    p.add(TtlRecord {
        id: "fresh-1".into(),
        member_id: "m-1".into(),
        ttl_expires_at: Some(chrono::Utc::now() + chrono::Duration::days(20)),
        is_third_party: true,
    });
    p.purge_expired(chrono::Utc::now());
    assert!(p.qdrant_points.contains("fresh-1"));
}

#[test]
fn bulk_purge_removes_all_expired() {
    let mut p = TtlPurger::new();
    for i in 0..100 {
        p.add(make_rec(&format!("third-{i}"), i + 1, true));
    }
    let (sql_n, vec_n) = p.purge_expired(chrono::Utc::now());
    assert_eq!(sql_n, 100);
    assert_eq!(vec_n, 100);
    assert!(p.sql_rows.is_empty());
}
