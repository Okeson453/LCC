//! lcc-migrate-runner — applies SQL migrations from `schemas/migrations/`.
//!
//! Uses sqlx::migrate!() to apply the canonical migration set.

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(about = "Apply SQL migrations to the configured database")]
struct Cli {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    #[arg(long, default_value = "schemas/migrations")]
    migrations_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    tracing::info!(url = %cli.database_url, dir = ?cli.migrations_dir, "running migrations");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&cli.database_url)
        .await?;
    let migrator = sqlx::migrate::Migrator::new(cli.migrations_dir).await?;
    migrator.run(&pool).await?;
    tracing::info!("migrations applied");
    Ok(())
}
