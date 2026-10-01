//! Profile-svc config.

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub http_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| "DATABASE_URL not set".to_string())?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            http_port: std::env::var("LCC_HTTP_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8082),
        })
    }
}
