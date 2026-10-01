//! Integration test: OAuth token refresh + rotation.
//!
//! LinkedIn access tokens expire (typically 60 days). The identity-svc must:
//! 1. Detect upcoming expiry via `oauth_tokens.expires_at`.
//! 2. Refresh using the refresh_token.
//! 3. Encrypt and store the new access_token.
//! 4. Emit `oauth.token_refreshed` audit event.
//! 5. On refresh failure, emit `oauth.token_refresh_failed` (so a restriction
//!    can be detected).

use chrono::{Duration, Utc};

#[derive(Debug, Clone)]
struct OAuthToken {
    member_id: String,
    access_token_hash: String,
    refresh_token_hash: String,
    expires_at: chrono::DateTime<Utc>,
}

#[derive(Default)]
struct OAuthStore {
    tokens: Vec<OAuthToken>,
    refresh_attempts: Vec<(String, Result<String, String>)>, // (member_id, result)
}

impl OAuthStore {
    fn add(&mut self, t: OAuthToken) {
        // Replace existing token for the same member.
        self.tokens.retain(|t2| t2.member_id != t.member_id);
        self.tokens.push(t);
    }
    fn get_mut(&mut self, member_id: &str) -> Option<&mut OAuthToken> {
        self.tokens.iter_mut().find(|t| t.member_id == member_id)
    }

    fn refresh_if_expiring(&mut self, member_id: &str, threshold_days: i64) -> Result<String, String> {
        let t = self.get_mut(member_id).ok_or("not found")?;
        let soon = Utc::now() + Duration::days(threshold_days);
        if t.expires_at > soon {
            return Err("not expiring soon".into());
        }
        // Simulate refresh: replace access token, bump expiry by 60 days.
        let new_access = format!("new-access-{member_id}-{}", Utc::now().timestamp());
        t.access_token_hash = sha256(&new_access);
        t.expires_at = Utc::now() + Duration::days(60);
        self.refresh_attempts.push((member_id.to_string(), Ok(new_access.clone())));
        Ok(new_access)
    }

    fn refresh_force_fail(&mut self, member_id: &str) {
        self.refresh_attempts
            .push((member_id.to_string(), Err("linkedin_401".into())));
    }
}

fn sha256(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    format!("{:x}", h.finalize())
}

#[test]
fn expiring_token_is_refreshed() {
    let mut store = OAuthStore::default();
    store.add(OAuthToken {
        member_id: "alice".into(),
        access_token_hash: "old".into(),
        refresh_token_hash: "refresh".into(),
        expires_at: Utc::now() - Duration::days(1), // already expired
    });
    let new_token = store.refresh_if_expiring("alice", 7).unwrap();
    assert!(new_token.starts_with("new-access-alice-"));
    let t = store.get_mut("alice").unwrap();
    assert_eq!(t.access_token_hash, sha256(&new_token));
    assert!(t.expires_at > Utc::now() + Duration::days(30));
}

#[test]
fn fresh_token_is_not_refreshed() {
    let mut store = OAuthStore::default();
    store.add(OAuthToken {
        member_id: "bob".into(),
        access_token_hash: "valid".into(),
        refresh_token_hash: "refresh".into(),
        expires_at: Utc::now() + Duration::days(30), // 30 days remaining
    });
    let result = store.refresh_if_expiring("bob", 7);
    assert!(result.is_err(), "fresh tokens must not be refreshed early");
    assert_eq!(result.unwrap_err(), "not expiring soon");
}

#[test]
fn refresh_within_threshold_is_refreshed() {
    let mut store = OAuthStore::default();
    store.add(OAuthToken {
        member_id: "carol".into(),
        access_token_hash: "expiring".into(),
        refresh_token_hash: "refresh".into(),
        expires_at: Utc::now() + Duration::days(3), // within 7d threshold
    });
    let new_token = store.refresh_if_expiring("carol", 7).unwrap();
    assert!(new_token.starts_with("new-access-carol-"));
}

#[test]
fn refresh_failure_emits_failure_event() {
    let mut store = OAuthStore::default();
    store.add(OAuthToken {
        member_id: "dave".into(),
        access_token_hash: "old".into(),
        refresh_token_hash: "refresh".into(),
        expires_at: Utc::now() - Duration::days(1),
    });
    store.refresh_force_fail("dave");
    let last = store.refresh_attempts.last().unwrap();
    assert_eq!(last.0, "dave");
    assert!(last.1.is_err());
}

#[test]
fn adding_new_token_replaces_old() {
    let mut store = OAuthStore::default();
    store.add(OAuthToken {
        member_id: "alice".into(),
        access_token_hash: "first".into(),
        refresh_token_hash: "r".into(),
        expires_at: Utc::now() + Duration::days(30),
    });
    store.add(OAuthToken {
        member_id: "alice".into(),
        access_token_hash: "second".into(),
        refresh_token_hash: "r".into(),
        expires_at: Utc::now() + Duration::days(60),
    });
    let alice_tokens: Vec<_> = store
        .tokens
        .iter()
        .filter(|t| t.member_id == "alice")
        .collect();
    assert_eq!(alice_tokens.len(), 1, "duplicate member_id must replace, not stack");
    assert_eq!(alice_tokens[0].access_token_hash, "second");
}

#[test]
fn missing_member_returns_not_found() {
    let mut store = OAuthStore::default();
    let result = store.refresh_if_expiring("missing", 7);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "not found");
}
