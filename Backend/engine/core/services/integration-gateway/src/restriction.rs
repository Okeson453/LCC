//! Restriction-signal detection.
//!
//! Scans response bodies for phrases that indicate LinkedIn is signaling an
//! account restriction: "Your account has been restricted", CAPTCHA
//! challenge, verification challenge, etc.
//!
//! On detection → account-wide pause, never same-session retry (axiom 6).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RestrictionSignal {
    None,
    AccountRestricted,
    CaptchaChallenge,
    VerificationRequired,
    RateLimit429,
    AuthRevoked,
    QuotaExhausted,
}

impl RestrictionSignal {
    pub fn as_str(self) -> &'static str {
        match self {
            RestrictionSignal::None => "none",
            RestrictionSignal::AccountRestricted => "account_restricted",
            RestrictionSignal::CaptchaChallenge => "captcha_challenge",
            RestrictionSignal::VerificationRequired => "verification_required",
            RestrictionSignal::RateLimit429 => "rate_limit_429",
            RestrictionSignal::AuthRevoked => "auth_revoked",
            RestrictionSignal::QuotaExhausted => "quota_exhausted",
        }
    }
}

/// Phrases (case-insensitive) that indicate a restriction signal.
const PHRASES: &[(&str, RestrictionSignal)] = &[
    (
        "your account has been restricted",
        RestrictionSignal::AccountRestricted,
    ),
    ("account restricted", RestrictionSignal::AccountRestricted),
    (
        "we've restricted your account",
        RestrictionSignal::AccountRestricted,
    ),
    ("captcha", RestrictionSignal::CaptchaChallenge),
    (
        "verification challenge",
        RestrictionSignal::VerificationRequired,
    ),
    (
        "please verify your identity",
        RestrictionSignal::VerificationRequired,
    ),
    (
        "verification required",
        RestrictionSignal::VerificationRequired,
    ),
    ("rate limit exceeded", RestrictionSignal::RateLimit429),
    ("too many requests", RestrictionSignal::RateLimit429),
    ("quota exceeded", RestrictionSignal::QuotaExhausted),
    ("quota exhausted", RestrictionSignal::QuotaExhausted),
];

/// Inspect the response body for restriction signals. Returns the matched
/// signal and the matched phrase (for logging).
pub fn detect(body: &str) -> (RestrictionSignal, Option<String>) {
    let body_lower = body.to_lowercase();
    for (phrase, signal) in PHRASES {
        if body_lower.contains(phrase) {
            return (*signal, Some((*phrase).to_string()));
        }
    }
    (RestrictionSignal::None, None)
}

/// Inspect a response status + body for restriction signals. Status 429 alone
/// is a rate-limit signal regardless of body content.
pub fn detect_status_and_body(status: u16, body: &str) -> (RestrictionSignal, Option<String>) {
    if status == 429 {
        return (
            RestrictionSignal::RateLimit429,
            Some("HTTP 429".to_string()),
        );
    }
    if status == 401 || status == 403 {
        // Auth failure could be token revoked or account restricted.
        let (sig, matched) = detect(body);
        if sig != RestrictionSignal::None {
            return (sig, matched);
        }
        // Fallback: assume auth revoked.
        return (
            RestrictionSignal::AuthRevoked,
            Some(format!("HTTP {status}")),
        );
    }
    detect(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_account_restricted() {
        let (sig, _) = detect("Your account has been restricted. Please verify.");
        assert_eq!(sig, RestrictionSignal::AccountRestricted);
    }

    #[test]
    fn detects_captcha() {
        let (sig, _) = detect("Please complete the CAPTCHA challenge to continue.");
        assert_eq!(sig, RestrictionSignal::CaptchaChallenge);
    }

    #[test]
    fn detects_verification() {
        let (sig, _) = detect("Verification required before continuing.");
        assert_eq!(sig, RestrictionSignal::VerificationRequired);
    }

    #[test]
    fn no_signal_in_clean_response() {
        let (sig, _) = detect("{\"id\": \"abc\"}");
        assert_eq!(sig, RestrictionSignal::None);
    }

    #[test]
    fn status_429_alone_is_rate_limit() {
        let (sig, matched) = detect_status_and_body(429, "");
        assert_eq!(sig, RestrictionSignal::RateLimit429);
        assert!(matched.is_some());
    }

    #[test]
    fn status_401_is_auth_revoked() {
        let (sig, _) = detect_status_and_body(401, "");
        assert_eq!(sig, RestrictionSignal::AuthRevoked);
    }

    #[test]
    fn case_insensitive() {
        let (sig, _) = detect("YOUR ACCOUNT HAS BEEN RESTRICTED");
        assert_eq!(sig, RestrictionSignal::AccountRestricted);
    }
}
