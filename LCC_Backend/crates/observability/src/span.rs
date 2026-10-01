//! Custom tracing span helpers for hot-path operations.

use std::cell::RefCell;
use tracing::{field, Span};
use uuid::Uuid;

thread_local! {
    static CURRENT_TRACE_ID: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Set the trace ID for the current thread (used by the API Gateway's
/// per-request middleware).
pub fn set_trace_id(id: impl Into<String>) {
    CURRENT_TRACE_ID.with(|cell| *cell.borrow_mut() = Some(id.into()));
}

/// Returns the current thread's trace_id, if set.
pub fn current_trace_id() -> Option<String> {
    CURRENT_TRACE_ID.with(|cell| cell.borrow().clone())
}

/// Span for Compliance Governor guard-stack evaluation.
/// Captures: trace_id, member_id, action_type, decision (filled at end).
pub fn span_for_governor_evaluate(
    trace_id: &str,
    member_id: &str,
    action_type: &str,
) -> Span {
    tracing::info_span!(
        "governor.evaluate",
        trace_id = %trace_id,
        member_id = %member_id,
        action_type = %action_type,
        decision = field::Empty,
        failed_guard = field::Empty,
        duration_ms = field::Empty,
    )
}

/// Span for Integration Gateway execute path.
/// Captures: trace_id, action_id, track, outcome (filled at end).
pub fn span_for_integration_execute(
    trace_id: &str,
    action_id: &str,
    action_type: &str,
) -> Span {
    tracing::info_span!(
        "integration.execute",
        trace_id = %trace_id,
        action_id = %action_id,
        action_type = %action_type,
        track = field::Empty,
        outcome = field::Empty,
        duration_ms = field::Empty,
    )
}

/// Generate a fresh trace_id (UUIDv7).
pub fn fresh_trace_id() -> String {
    Uuid::now_v7().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_trace_id_unique() {
        let a = fresh_trace_id();
        let b = fresh_trace_id();
        assert_ne!(a, b);
    }

    #[test]
    fn thread_local_trace_id() {
        set_trace_id("test_123");
        assert_eq!(current_trace_id(), Some("test_123".to_string()));
    }
}
