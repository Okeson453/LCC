//! Contract test — ComplianceEvaluate request/response shape.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct EvaluateRequest {
    pub member_id: String,
    pub action_type: String,
    pub target_kind: String,
    pub target_id: Option<String>,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EvaluateResponse {
    pub decision: String,
    pub reason: String,
    pub guards_passed: Vec<String>,
    pub guards_failed: Vec<String>,
    pub permit_token: Option<String>,
    pub evaluated_at: chrono::DateTime<chrono::Utc>,
}

#[test]
fn evaluate_request_round_trip() {
    let req = EvaluateRequest {
        member_id: "00000000-0000-0000-0000-000000000001".into(),
        action_type: "post_publish".into(),
        target_kind: "post".into(),
        target_id: Some("post-1".into()),
        context: Some(serde_json::json!({"kb_refs": ["r1"]})),
    };
    let s = serde_json::to_string(&req).unwrap();
    let parsed: EvaluateRequest = serde_json::from_str(&s).unwrap();
    assert_eq!(parsed.action_type, "post_publish");
}

#[test]
fn evaluate_response_round_trip() {
    let resp = EvaluateResponse {
        decision: "allow".into(),
        reason: "all guards passed".into(),
        guards_passed: vec!["daily_cap".into(), "cooldown".into(), "grounding".into()],
        guards_failed: vec![],
        permit_token: Some("eyJhbGciOiJIUzI1NiJ9.test".into()),
        evaluated_at: chrono::Utc::now(),
    };
    let s = serde_json::to_string(&resp).unwrap();
    let parsed: EvaluateResponse = serde_json::from_str(&s).unwrap();
    assert_eq!(parsed.decision, "allow");
    assert_eq!(parsed.guards_passed.len(), 3);
}
