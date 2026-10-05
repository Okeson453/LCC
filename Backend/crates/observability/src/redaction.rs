//! Log redaction — fields matching `*token*`, `*password*`, `*secret*`,
//! `linkedin_url` (member's own) are replaced with `[REDACTED]`.
//!
//! Per Source Backend Design Concept §56.4 and Non-Negotiable §10.

use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RedactedField {
    pub field: String,
}

/// The set of field name patterns that must be redacted.
pub fn redact_patterns() -> Vec<&'static str> {
    vec![
        "token",
        "password",
        "secret",
        "linkedin_url",
        "api_key",
        "private_key",
    ]
}

/// Walk a JSON object and redact matching field names (case-insensitive substring).
pub fn redact_log_fields(v: &Value) -> (Value, Vec<RedactedField>) {
    let patterns: Vec<String> = redact_patterns().iter().map(|s| s.to_lowercase()).collect();
    let mut redacted = Vec::new();
    let out = redact_recursive(v, &patterns, &mut redacted);
    (out, redacted)
}

fn redact_recursive(v: &Value, patterns: &[String], found: &mut Vec<RedactedField>) -> Value {
    match v {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, val) in map {
                let k_lower = k.to_lowercase();
                if patterns.iter().any(|p| k_lower.contains(p)) {
                    found.push(RedactedField { field: k.clone() });
                    out.insert(k.clone(), Value::String("[REDACTED]".into()));
                } else {
                    out.insert(k.clone(), redact_recursive(val, patterns, found));
                }
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(
            arr.iter()
                .map(|v| redact_recursive(v, patterns, found))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Returns a sorted list of unique field names redacted.
pub fn unique_redacted(redacted: &[RedactedField]) -> BTreeSet<String> {
    redacted.iter().map(|r| r.field.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redacts_token_field() {
        let v = json!({"access_token": "abc", "name": "alice"});
        let (red, fields) = redact_log_fields(&v);
        assert_eq!(red["access_token"], "[REDACTED]");
        assert_eq!(red["name"], "alice");
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].field, "access_token");
    }

    #[test]
    fn case_insensitive() {
        let v = json!({"RefreshToken": "x", "API_KEY": "y", "linkedin_url": "z"});
        let (red, fields) = redact_log_fields(&v);
        assert_eq!(red["RefreshToken"], "[REDACTED]");
        assert_eq!(red["API_KEY"], "[REDACTED]");
        assert_eq!(red["linkedin_url"], "[REDACTED]");
        assert_eq!(fields.len(), 3);
    }

    #[test]
    fn recurses_into_nested() {
        let v = json!({
            "outer": {
                "token": "secret",
                "inner": {"password": "hunter2", "name": "alice"}
            }
        });
        let (red, _) = redact_log_fields(&v);
        assert_eq!(red["outer"]["token"], "[REDACTED]");
        assert_eq!(red["outer"]["inner"]["password"], "[REDACTED]");
        assert_eq!(red["outer"]["inner"]["name"], "alice");
    }

    #[test]
    fn recurses_into_array() {
        let v = json!({"items": [{"token": "a"}, {"safe": "b"}]});
        let (red, _) = redact_log_fields(&v);
        assert_eq!(red["items"][0]["token"], "[REDACTED]");
        assert_eq!(red["items"][1]["safe"], "b");
    }
}
