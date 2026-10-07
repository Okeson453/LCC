//! `lcc-token-mint-cli` — issue a session JWT for local development and
//! contract verification.
//!
//! This bin was a `println!("stub")` placeholder, which left the API surface
//! untestable end-to-end: every protected route returns 401 without a token,
//! so "does auth work" and "is the route wired" were indistinguishable, and
//! the request/response schemas on the canonical contract could never be
//! exercised against a running service.
//!
//! It signs with the same `lcc_auth::JwtIssuer` the login flow uses, and takes
//! the same `LCC_AUTH_JWT_SECRET` the services verify with, so a token minted
//! here is accepted by the gateway, the domain services and `realtime-svc`.
//!
//! **Development use only.** This is a bearer-token minter: anyone who can run
//! it against the signing secret can mint an admin token. It must never be
//! built into a production image — see the `[lints]`/release notes in
//! `Backend/docs/` for the deployment policy.

use std::process::ExitCode;

use clap::Parser;
use lcc_auth::JwtIssuer;
use uuid::Uuid;

/// Role names accepted by `lcc_auth::rbac::Role::from_str`. Validated here so a
/// typo fails here rather than producing a token the gateway rejects at the
/// edge with a confusing "unknown role" error.
const ROLES: [&str; 5] = ["owner", "assistant", "reviewer", "admin", "auditor"];

#[derive(Parser, Debug)]
#[command(
    name = "lcc-token-mint-cli",
    about = "Mint an LCC session JWT for local development",
    version
)]
struct Cli {
    /// Member (subject) the token is issued for. Defaults to a fresh v7 UUID.
    #[arg(long)]
    member: Option<Uuid>,

    /// Role claim. Must be one of: owner, assistant, reviewer, admin, auditor.
    #[arg(long, default_value = "owner")]
    role: String,

    /// Token lifetime in minutes.
    #[arg(long, default_value_t = 60)]
    ttl_minutes: i64,

    /// Signing secret. Must match the services' `LCC_AUTH_JWT_SECRET`.
    #[arg(
        long,
        env = "LCC_AUTH_JWT_SECRET",
        default_value = "dev-secret-change-me",
        hide_default_value = true
    )]
    secret: String,

    /// `kid` header value; informational, the verifier selects on `iss`/`aud`.
    #[arg(long, default_value = "lcc-dev")]
    key_id: String,

    /// `iss` claim. Must match the services' configured issuer
    /// (`LCC_AUTH_JWT_ISSUER`). Defaults to the platform default so a freshly
    /// built gateway accepts the token with no extra flags.
    #[arg(long, env = "LCC_AUTH_JWT_ISSUER", default_value = "lcc-identity-svc")]
    issuer: String,

    /// `aud` claim. Must match the services' configured audience.
    #[arg(long, default_value = "lcc-api")]
    audience: String,

    /// Print the claims as JSON alongside the token.
    #[arg(long, default_value_t = false)]
    json: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(out) => {
            println!("{out}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("lcc-token-mint-cli: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let cli = Cli::parse();

    if !ROLES.contains(&cli.role.as_str()) {
        return Err(format!(
            "unknown role {:?}; expected one of {}",
            cli.role,
            ROLES.join(", ")
        ));
    }
    if cli.ttl_minutes <= 0 {
        return Err(format!(
            "--ttl-minutes must be positive, got {}",
            cli.ttl_minutes
        ));
    }
    if cli.secret.is_empty() {
        return Err("--secret / LCC_AUTH_JWT_SECRET must not be empty".to_string());
    }

    let member_id = cli.member.unwrap_or_else(Uuid::now_v7);
    let issuer = JwtIssuer::new(
        cli.secret.as_bytes(),
        &cli.key_id,
        &cli.issuer,
        &cli.audience,
        cli.ttl_minutes,
    );
    let (token, claims) = issuer
        .issue(&member_id, &cli.role)
        .map_err(|e| format!("sign token: {e}"))?;

    if cli.json {
        let payload = serde_json::json!({
            "token": token,
            "member_id": member_id.to_string(),
            "role": cli.role,
            "expires_at": claims.exp,
        });
        Ok(serde_json::to_string_pretty(&payload).unwrap_or_default())
    } else {
        // Bare token on stdout so it composes: TOKEN=$(lcc-token-mint-cli …)
        Ok(token)
    }
}
