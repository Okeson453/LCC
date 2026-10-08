//! Integration test: member onboarding flow.
//!
//! End-to-end happy path:
//! 1. Member signs up → identity-svc creates member row + audit row.
//! 2. OAuth starts → returns auth_url.
//! 3. LinkedIn token stored encrypted.
//! 4. First content draft → ai-worker → 3 variants emitted.
//! 5. Approval submitted → approval-svc creates pending approval.
//! 6. Compliance Governor evaluates PostPublish → allow + permit.
//! 7. Integration Gateway dispatches → content_item state → published.
//! Each step writes an audit row; the chain remains intact.

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

use harness::{ActionType, MockAuditLog, MockComplianceGovernor, MockIntegrationGateway};
use uuid::Uuid;

#[test]
fn full_happy_path_first_post() {
    let mut audit = MockAuditLog::new();
    let gov = MockComplianceGovernor::new();
    let gw = MockIntegrationGateway::new();
    let member = Uuid::new_v4();

    // 1. Signup.
    audit.insert(
        "system:identity-svc",
        "member.created",
        "member",
        &member.to_string(),
        "ok",
        serde_json::json!({"linkedin_id": "li-1", "email": "a@b"}),
    );

    // 2. OAuth start.
    audit.insert(
        "user:alice",
        "auth.start",
        "oauth_flow",
        &member.to_string(),
        "ok",
        serde_json::json!({"scopes": ["r_liteprofile", "w_member_social"]}),
    );

    // 3. Token stored.
    audit.insert(
        "system:identity-svc",
        "oauth.token_stored",
        "oauth_token",
        &member.to_string(),
        "ok",
        serde_json::json!({"provider": "linkedin"}),
    );

    // 4. AI draft.
    audit.insert(
        "system:ai-worker",
        "llm.draft_content",
        "content_item",
        "draft-1",
        "ok",
        serde_json::json!({"tokens_in": 200, "tokens_out": 400, "model_id": "gpt-4o"}),
    );

    // 5. Approval requested.
    audit.insert(
        "user:alice",
        "approval.requested",
        "approval",
        "approval-1",
        "ok",
        serde_json::json!({"resource_type": "content_item", "decision": "pending"}),
    );

    // 6. Governor evaluates PostPublish.
    let req = harness::GovernorRequest {
        member_id: member,
        action_type: ActionType::PostPublish,
        target_kind: "post".into(),
        target_id: Some("draft-1".into()),
        context: serde_json::json!({"kb_refs": ["rec-1", "rec-2"]}),
    };
    let decision = gov.evaluate(&req);
    assert_eq!(decision.decision, "allow");
    let permit = decision.permit_token.expect("permit must be minted");

    audit.insert(
        "system:compliance-governor",
        "governor.evaluate",
        "action",
        "draft-1",
        "ok",
        serde_json::json!({
            "decision": "allow",
            "guards_passed": decision.guards_passed,
            "guards_failed": decision.guards_failed,
            "permit_token": permit,
        }),
    );

    // 7. Integration gateway dispatches.
    let dispatch = gw.execute(
        member,
        &permit,
        ActionType::PostPublish,
        "draft-1:post",
        "hello world",
    );
    assert!(dispatch.is_ok());
    audit.insert(
        "system:integration-gateway",
        "integration.execute",
        "content_item",
        "draft-1",
        "ok",
        serde_json::json!({"idempotency_key": "draft-1:post", "result": dispatch.unwrap()}),
    );

    // 8. State transitioned to published.
    audit.insert(
        "system:content-svc",
        "content.state_changed",
        "content_item",
        "draft-1",
        "ok",
        serde_json::json!({"from": "scheduled", "to": "published"}),
    );

    // All 8 audit rows must form a valid chain.
    audit.verify_chain().expect("end-to-end chain must verify");
    assert_eq!(audit.rows.len(), 8);
}

#[test]
fn onboarding_with_low_grounding_blocks_publish() {
    let mut audit = MockAuditLog::new();
    let gov = MockComplianceGovernor::new();
    let member = Uuid::new_v4();

    // Signup + draft + try to publish without KB refs.
    audit.insert(
        "system:identity-svc",
        "member.created",
        "member",
        &member.to_string(),
        "ok",
        serde_json::json!({}),
    );
    audit.insert(
        "system:ai-worker",
        "llm.draft_content",
        "content_item",
        "d1",
        "ok",
        serde_json::json!({"kb_refs": []}),
    );

    let req = harness::GovernorRequest {
        member_id: member,
        action_type: ActionType::PostPublish,
        target_kind: "post".into(),
        target_id: Some("d1".into()),
        context: serde_json::json!({"kb_refs": []}),
    };
    let decision = gov.evaluate(&req);
    assert_eq!(decision.decision, "deny");
    assert_eq!(decision.guards_failed, vec!["grounding"]);
    assert!(decision.permit_token.is_none());

    // Audit the deny.
    audit.insert(
        "system:compliance-governor",
        "governor.evaluate",
        "action",
        "d1",
        "denied",
        serde_json::json!({
            "decision": "deny",
            "failed_guard": "grounding",
            "reason": "low_grounding: only 0 KB refs (need 1)",
        }),
    );

    audit
        .verify_chain()
        .expect("chain must verify even on deny path");
    assert_eq!(audit.rows.len(), 3);
}
