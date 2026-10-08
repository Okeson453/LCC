//! Integration test: publish idempotency.
//!
//! Verifies that the Integration Gateway returns the same dispatch result
//! when the same idempotency_key is replayed (at-least-once delivery + dedup).

// F-AUDIT-51: the workspace lint set denies `clippy::unwrap_used`,
// `expect_used` and `panic` because an `unwrap` on a `Result` can take a
// production service down. In a test binary the opposite holds: panicking IS
// the failure signal, and `unwrap()` is the idiomatic way to assert "this
// fixture must be valid, and if it is not the test must fail". These suites
// were never compiled by any crate before the `[[test]]` targets were added
// in `crates/test-utils/Cargo.toml`, so they never faced the gate.
// The exemption is file-scoped so the production lints stay fully intact.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "common/harness.rs"]
mod harness;

use harness::{ActionType, MockIntegrationGateway};
use uuid::Uuid;

#[test]
fn first_dispatch_succeeds_and_is_recorded() {
    let gw = MockIntegrationGateway::new();
    let member = Uuid::new_v4();
    let r = gw.execute(
        member,
        "permit-1",
        ActionType::PostPublish,
        "idem-1",
        "post-body",
    );
    assert!(r.is_ok());
    let state = gw.state.lock().unwrap();
    assert_eq!(state.dispatched.len(), 1);
    assert_eq!(state.permits_seen.len(), 1);
}

#[test]
fn replay_returns_original_result_without_redispatch() {
    let gw = MockIntegrationGateway::new();
    let member = Uuid::new_v4();
    let r1 = gw.execute(
        member,
        "permit-1",
        ActionType::PostPublish,
        "idem-1",
        "post-body",
    );
    let r2 = gw.execute(
        member,
        "permit-1",
        ActionType::PostPublish,
        "idem-1",
        "post-body",
    );
    assert_eq!(r1.unwrap(), r2.unwrap(), "replay must return same result");
    let state = gw.state.lock().unwrap();
    assert_eq!(
        state.dispatched.len(),
        1,
        "second call must not re-dispatch"
    );
}

#[test]
fn different_idempotency_keys_dispatch_independently() {
    let gw = MockIntegrationGateway::new();
    let member = Uuid::new_v4();
    let _ = gw.execute(
        member,
        "permit-1",
        ActionType::PostPublish,
        "idem-1",
        "post-1",
    );
    let _ = gw.execute(
        member,
        "permit-2",
        ActionType::PostPublish,
        "idem-2",
        "post-2",
    );
    let state = gw.state.lock().unwrap();
    assert_eq!(
        state.dispatched.len(),
        2,
        "two distinct keys dispatch twice"
    );
}

#[test]
fn dispatch_failure_does_not_store_idempotent_success() {
    let gw = MockIntegrationGateway::new();
    let member = Uuid::new_v4();
    *gw.dispatch_fail.lock().unwrap() = true;
    let r1 = gw.execute(
        member,
        "permit-1",
        ActionType::PostPublish,
        "idem-1",
        "post-1",
    );
    assert!(r1.is_err(), "dispatch failure must return error");
    *gw.dispatch_fail.lock().unwrap() = false;
    let r2 = gw.execute(
        member,
        "permit-1",
        ActionType::PostPublish,
        "idem-1",
        "post-1",
    );
    assert!(r2.is_ok(), "retry after failure must succeed");
}

#[test]
fn idempotency_is_per_member() {
    let gw = MockIntegrationGateway::new();
    let alice = Uuid::new_v4();
    let bob = Uuid::new_v4();
    let _ = gw.execute(
        alice,
        "permit-1",
        ActionType::PostPublish,
        "idem-1",
        "alice-post",
    );
    let _ = gw.execute(
        bob,
        "permit-2",
        ActionType::PostPublish,
        "idem-1",
        "bob-post",
    );
    let state = gw.state.lock().unwrap();
    assert_eq!(
        state.dispatched.len(),
        2,
        "same idem key for different members must dispatch twice"
    );
}

#[test]
fn hundred_replays_return_original() {
    let gw = MockIntegrationGateway::new();
    let member = Uuid::new_v4();
    let first = gw
        .execute(member, "permit-1", ActionType::Dm, "idem-1", "msg")
        .unwrap();
    for _ in 0..99 {
        let r = gw
            .execute(member, "permit-1", ActionType::Dm, "idem-1", "msg")
            .unwrap();
        assert_eq!(r, first);
    }
    let state = gw.state.lock().unwrap();
    assert_eq!(state.dispatched.len(), 1, "100 replays → 1 dispatch");
}
