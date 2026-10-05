//! `lcc` — admin CLI.
//!
//! Subcommands:
//! - `lcc governor evaluate ...`  → POST to compliance-governor
//! - `lcc config show`            → show effective config
//! - `lcc audit verify`           → verify audit-log chain
//! - `lcc ratelimit test`         → test rate limiter

use clap::{Parser, Subcommand};
use serde_json::json;

#[derive(Parser, Debug)]
#[command(name = "lcc", about = "OKESON-LCC admin CLI", version)]
struct Cli {
    #[arg(
        long,
        env = "LCC_API_GATEWAY_URL",
        default_value = "http://localhost:8080"
    )]
    api_gateway_url: String,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    Governor {
        #[command(subcommand)]
        cmd: GovernorCmd,
    },
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },
    Audit {
        #[command(subcommand)]
        cmd: AuditCmd,
    },
    RateLimitTest,
}

#[derive(Subcommand, Debug)]
enum GovernorCmd {
    Evaluate {
        member_id: String,
        action_type: String,
        target_kind: String,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCmd {
    Show,
}

#[derive(Subcommand, Debug)]
enum AuditCmd {
    Verify,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let client = reqwest::Client::new();
    let base = cli.api_gateway_url.clone();

    match cli.cmd {
        Cmd::Governor { cmd } => match cmd {
            GovernorCmd::Evaluate {
                member_id,
                action_type,
                target_kind,
            } => {
                let body = json!({
                    "member_id": member_id,
                    "action_type": action_type,
                    "target_kind": target_kind,
                });
                let resp = client
                    .post(format!("{base}/v1/admin/governor/evaluate"))
                    .json(&body)
                    .send()
                    .await?;
                println!("{}", resp.text().await?);
            }
        },
        Cmd::Config { cmd } => match cmd {
            ConfigCmd::Show => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({ "api_gateway_url": base }))?
                );
            }
        },
        Cmd::Audit { cmd } => match cmd {
            AuditCmd::Verify => {
                println!("audit verify endpoint not yet implemented");
            }
        },
        Cmd::RateLimitTest => {
            println!("rate-limit test not yet implemented");
        }
    }
    Ok(())
}
