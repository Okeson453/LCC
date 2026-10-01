//! Tests for the error crate.

use lcc_error::{LccError, ErrorEnvelope};

#[test]
fn error_display() {
    let e = LccError::NotFound("foo".into());
    assert_eq!(e.to_string(), "not found: foo");
}

#[test]
fn error_envelope_serde() {
    let env = ErrorEnvelope::new("bad_request", "title required");
    let s = serde_json::to_string(&env).expect("serialize");
    assert!(s.contains("bad_request"));
    assert!(s.contains("title required"));
}

#[test]
fn error_envelope_from_lcc_error() {
    let e = LccError::BadRequest("bad".into());
    let env: ErrorEnvelope = e.into();
    let s = serde_json::to_string(&env).expect("serialize");
    assert!(s.contains("bad_request"));
}
