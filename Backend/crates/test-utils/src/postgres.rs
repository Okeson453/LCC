//! Postgres test helpers — open a PgPool against an ephemeral Postgres,
//! applying all migrations from `schemas/migrations/`.

use lcc_config::yaml::YamlConfigError;
use sqlx::{Executor, PgPool};
use std::time::Duration;

/// Connect to a test Postgres. If `LCC_TEST_DATABASE_URL` is set, use it;
/// otherwise spin up a testcontainer (not enabled by default — requires the
/// `testcontainers` feature).
pub async fn connect_test_postgres() -> Result<PgPool, sqlx::Error> {
    let url = std::env::var("LCC_TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/lcc_test".to_string());

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&url)
        .await?;

    Ok(pool)
}

/// Apply all migrations from the given directory in order.
pub async fn apply_migrations(pool: &PgPool, migrations_dir: &str) -> Result<(), YamlConfigError> {
    let dir = std::fs::read_dir(migrations_dir).map_err(YamlConfigError::Io)?;
    let mut paths: Vec<_> = dir
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("sql"))
        .collect();
    paths.sort();

    for path in paths {
        let sql = std::fs::read_to_string(&path).map_err(YamlConfigError::Io)?;
        pool.execute(sql.as_str()).await.map_err(|e| {
            YamlConfigError::Validation(format!("migration {} failed: {e}", path.display()))
        })?;
    }

    Ok(())
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_smoke() {
        // This test only runs if LCC_TEST_DATABASE_URL is set.
        if std::env::var("LCC_TEST_DATABASE_URL").is_err() {
            return;
        }
        let pool = connect_test_postgres().await.unwrap();
        let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, 1);
    }
}
