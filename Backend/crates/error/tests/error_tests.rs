//! Tests for the error crate.
//!
//! Written against the real `LccError` shape: `NotFound` is a struct variant
//! carrying `resource_type`/`resource_id`, there is no `BadRequest` variant
//! (`Validation` is the 400), and the `LccError -> ErrorEnvelope` conversion
//! now lives in the crate rather than being re-implemented per service.
// Integration tests assert on real return values; `unwrap`/`expect` on a
// failing assertion is the point, so the production deny does not apply.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use lcc_error::{ErrorEnvelope, LccError};

#[test]
fn not_found_displays_its_resource() {
    let e = LccError::NotFound {
        resource_type: "content_item".into(),
        resource_id: "foo".into(),
    };
    assert_eq!(e.to_string(), "not found: content_item foo");
}

#[test]
fn error_envelope_serde() {
    let env = ErrorEnvelope::new("bad_request", "title required");
    let s = serde_json::to_string(&env).expect("serialize");
    assert!(s.contains("bad_request"));
    assert!(s.contains("title required"));
    // The contract's canonical shape is `{ "error": { ... } }`.
    assert!(s.contains("\"error\""));
}

#[test]
fn error_envelope_from_lcc_error() {
    let e = LccError::Validation("title required".into());
    let env: ErrorEnvelope = e.into();
    let s = serde_json::to_string(&env).expect("serialize");
    assert!(s.contains("validation_failed"));
    assert!(s.contains("title required"));
}

#[test]
fn borrowed_errors_also_convert() {
    let e = LccError::NotFound {
        resource_type: "contact".into(),
        resource_id: "c-1".into(),
    };
    let env: ErrorEnvelope = (&e).into();
    assert_eq!(env.error.code, "not_found");
    assert_eq!(env.error.message, "not found: contact c-1");
}

#[test]
fn codes_match_http_statuses() {
    // A client switching on `code` and a client switching on the status must
    // never disagree about whether something was a 404 or a 500.
    let cases: Vec<(LccError, &str, u16)> = vec![
        (LccError::Validation("x".into()), "validation_failed", 400),
        (
            LccError::NotFound {
                resource_type: "x".into(),
                resource_id: "y".into(),
            },
            "not_found",
            404,
        ),
        (LccError::Conflict("x".into()), "conflict", 409),
        (LccError::Forbidden("x".into()), "forbidden", 403),
        (LccError::Unauthorized("x".into()), "unauthorized", 401),
        (
            LccError::RateLimited { retry_after_ms: 1 },
            "rate_limited",
            429,
        ),
        (
            LccError::CircuitOpen {
                service: "s".into(),
            },
            "circuit_open",
            503,
        ),
        (LccError::Timeout("x".into()), "upstream_timeout", 504),
    ];
    for (e, code, status) in cases {
        assert_eq!(e.error_code(), code, "wrong code for {e}");
        assert_eq!(e.http_status().as_u16(), status, "wrong status for {e}");
    }
}

#[test]
fn internal_errors_do_not_leak_detail_in_the_code() {
    // The code is a stable identifier; the message is where detail belongs.
    let e = LccError::Internal("db password is hunter2".into());
    let env = e.to_envelope();
    assert_eq!(env.error.code, "internal_error");
    assert!(env.error.message.contains("hunter2"));
}

#[test]
fn trace_id_is_carried_through() {
    let env = ErrorEnvelope::new("internal_error", "boom").with_trace_id("trace-1");
    let s = serde_json::to_string(&env).expect("serialize");
    assert!(s.contains("trace-1"));
}
