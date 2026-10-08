//! Integration test: per-member RLS isolation.
//!
//! Verifies that the RLS session variable (`app.current_member_id`) gates
//! row visibility — Member A cannot read or write Member B's data even if
//! both members exist in the table.

// F-AUDIT-51: the workspace lint set denies `clippy::unwrap_used`,
// `expect_used` and `panic` because an `unwrap` on a `Result` can take a
// production service down. In a test binary the opposite holds: panicking IS
// the failure signal, and `unwrap()` is the idiomatic way to assert "this
// fixture must be valid, and if it is not the test must fail". These suites
// were never compiled by any crate before the `[[test]]` targets were added
// in `crates/test-utils/Cargo.toml`, so they never faced the gate.
// The exemption is file-scoped so the production lints stay fully intact.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// F-AUDIT-71: `common/harness.rs` is a shared module pulled in by every
// suite below via `#[path]`. Each test binary exercises only part of it, so
// the rest is unreachable *from that binary* and `dead_code` fires on items
// that are genuinely used by their siblings — `MockRlsDb`, `sha256_hex`,
// `RiskTier`, `set_restriction`, and others. The alternative (one harness per
// suite) duplicates the mocks this file exists to share.
#![allow(dead_code)]
// `resource_id.into()` reads as redundant field-name shorthand, but the
// struct field is String and the parameter is &str, so the conversion is
// load-bearing and the shorthand would not compile.
#![allow(clippy::redundant_field_names)]
#[path = "common/harness.rs"]
mod harness;

use harness::MockRlsDb;
use uuid::Uuid;

#[test]
fn no_session_blocks_all_writes() {
    let db = MockRlsDb::new();
    let result = db.insert("content_items", "alice-row");
    assert!(
        result.is_err(),
        "writes without RLS context must be rejected"
    );
    assert_eq!(db.read("content_items").len(), 0);
}

#[test]
fn member_a_cannot_see_member_b_rows() {
    let db = MockRlsDb::new();
    let alice = Uuid::new_v4();
    let bob = Uuid::new_v4();

    db.set_session(alice);
    db.insert("content_items", "alice-1").unwrap();
    db.insert("content_items", "alice-2").unwrap();
    db.clear_session();

    db.set_session(bob);
    db.insert("content_items", "bob-1").unwrap();
    db.clear_session();

    db.set_session(alice);
    let alice_rows = db.read("content_items");
    assert_eq!(alice_rows.len(), 2);
    assert!(alice_rows.iter().all(|r| r.starts_with("alice-")));
    db.clear_session();

    db.set_session(bob);
    let bob_rows = db.read("content_items");
    assert_eq!(bob_rows.len(), 1);
    assert_eq!(bob_rows[0], "bob-1");
    db.clear_session();
}

#[test]
fn switching_member_context_isolates_writes() {
    let db = MockRlsDb::new();
    let alice = Uuid::new_v4();
    let bob = Uuid::new_v4();

    db.set_session(alice);
    db.insert("kb_records", "alice-kb-1").unwrap();
    db.set_session(bob);
    db.insert("kb_records", "bob-kb-1").unwrap();

    // Confirm both writes are isolated.
    db.set_session(alice);
    assert_eq!(db.read("kb_records"), vec!["alice-kb-1".to_string()]);
    db.set_session(bob);
    assert_eq!(db.read("kb_records"), vec!["bob-kb-1".to_string()]);
}

#[test]
fn no_session_returns_empty_on_read() {
    let db = MockRlsDb::new();
    let alice = Uuid::new_v4();
    db.set_session(alice);
    db.insert("content_items", "x").unwrap();
    db.clear_session();
    assert_eq!(db.read("content_items").len(), 0);
}

#[test]
fn concurrent_member_sessions_are_independent() {
    let db = MockRlsDb::new();
    let alice = Uuid::new_v4();
    let bob = Uuid::new_v4();
    let carol = Uuid::new_v4();

    db.set_session(alice);
    db.insert("audience", "alice-list").unwrap();
    db.set_session(bob);
    db.insert("audience", "bob-list").unwrap();
    db.set_session(carol);
    db.insert("audience", "carol-list").unwrap();

    db.set_session(alice);
    assert_eq!(db.read("audience"), vec!["alice-list".to_string()]);
    db.set_session(bob);
    assert_eq!(db.read("audience"), vec!["bob-list".to_string()]);
    db.set_session(carol);
    assert_eq!(db.read("audience"), vec!["carol-list".to_string()]);
}
