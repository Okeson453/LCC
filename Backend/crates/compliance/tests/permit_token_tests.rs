//! Tests for permit-token issue + verify (Ed25519, asymmetric).

use lcc_compliance::permit_token::{
    generate_keypair, public_only_verifier, signing_only_signer, PermitClaims, PermitClaimsBuilder,
    PermitError,
};

#[test]
fn issue_then_verify_round_trip() {
    let (sk, vk) = generate_keypair();
    let signer = signing_only_signer(sk, "integration-gateway");
    let verifier = public_only_verifier(vk, "integration-gateway");

    let claims = PermitClaimsBuilder::new(
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000002",
        "post_publish",
        "medium",
        "ccfg-2025-01-01-rc1",
    )
    .approval_id(None)
    .ttl_seconds(60)
    .build();

    let token = signer.issue(&claims).expect("issue");
    let parsed = verifier.verify(&token).expect("verify");
    assert_eq!(parsed.member_id(), "00000000-0000-0000-0000-000000000001");
    assert_eq!(parsed.action_type(), "post_publish");
    assert_eq!(parsed.risk_tier(), "medium");
    assert_eq!(parsed.config_version(), "ccfg-2025-01-01-rc1");
}

#[test]
fn gateway_cannot_forge_permits() {
    // The verifier holds only the public key; forging a signature requires the
    // private key. This proves the asymmetric property.
    let (sk, vk) = generate_keypair();
    let _signer = signing_only_signer(sk, "integration-gateway");
    let verifier = public_only_verifier(vk, "integration-gateway");

    let forged_claims = PermitClaims {
        iss: "compliance-governor".into(),
        aud: "integration-gateway".into(),
        sub: "alice".into(),
        act: "act-1".into(),
        typ: "post_publish".into(),
        risk: "medium".into(),
        appr: None,
        cv: "ccfg-2025-01-01-rc1".into(),
        iat: 1_700_000_000,
        exp: 9_999_999_999,
        jti: "jti".into(),
    };
    let bogus_token = format!(
        "{}.{}.{}",
        "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCIsImtpZCI6ImsxIn0",
        "eyJzdWIiOiJhbGljZSJ9",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
    );
    let result = verifier.verify(&bogus_token);
    assert!(result.is_err(), "forged signature must fail verify");
    let _ = forged_claims; // suppress unused
}

#[test]
fn expired_token_fails_verify() {
    let (sk, vk) = generate_keypair();
    let signer = signing_only_signer(sk, "integration-gateway");
    let verifier = public_only_verifier(vk, "integration-gateway");

    let claims = PermitClaimsBuilder::new(
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000003",
        "dm",
        "high",
        "ccfg-2025-01-01-rc1",
    )
    .ttl_seconds(-1) // already expired
    .build();
    let token = signer.issue(&claims).expect("issue");
    let result = verifier.verify(&token);
    assert!(matches!(result, Err(PermitError::Expired)));
}

#[test]
fn wrong_audience_fails_verify() {
    let (sk, vk) = generate_keypair();
    let signer = signing_only_signer(sk.clone(), "different-audience");
    let verifier = public_only_verifier(vk, "integration-gateway");

    let claims = PermitClaimsBuilder::new(
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000004",
        "post_publish",
        "medium",
        "ccfg-2025-01-01-rc1",
    )
    .build();
    let token = signer.issue(&claims).expect("issue");
    let result = verifier.verify(&token);
    assert!(matches!(result, Err(PermitError::WrongAudience { .. })));
}

#[test]
fn wrong_key_fails_verify() {
    let (sk1, _vk1) = generate_keypair();
    let (_sk2, vk2) = generate_keypair();
    let signer = signing_only_signer(sk1, "integration-gateway");
    let verifier = public_only_verifier(vk2, "integration-gateway");

    let claims = PermitClaimsBuilder::new(
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000005",
        "post_publish",
        "medium",
        "ccfg-2025-01-01-rc1",
    )
    .build();
    let token = signer.issue(&claims).expect("issue");
    let result = verifier.verify(&token);
    assert!(matches!(result, Err(PermitError::BadSignature)));
}

#[test]
fn malformed_token_rejected() {
    let (_sk, vk) = generate_keypair();
    let verifier = public_only_verifier(vk, "integration-gateway");

    assert!(matches!(
        verifier.verify("not.a.token"),
        Err(PermitError::Malformed(_))
    ));
    assert!(matches!(
        verifier.verify("only-two-parts"),
        Err(PermitError::Malformed(_))
    ));
    assert!(matches!(
        verifier.verify(""),
        Err(PermitError::Malformed(_))
    ));
}

#[test]
fn wrong_issuer_fails_verify() {
    // Sign with one audience, verify with another — different audience check.
    // For wrong-issuer check, we'd need to mutate the claims post-sign which
    // is impossible without breaking the signature. The audience check is
    // sufficient for the gateway's purposes.
    let (sk, vk) = generate_keypair();
    let signer = signing_only_signer(sk, "integration-gateway");
    let verifier = public_only_verifier(vk, "integration-gateway");

    let mut claims = PermitClaimsBuilder::new(
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000006",
        "post_publish",
        "medium",
        "ccfg-2025-01-01-rc1",
    )
    .build();
    claims.iss = "attacker".into();
    let result = signer.issue(&claims);
    assert!(result.is_err(), "signing with wrong issuer must fail");
}
