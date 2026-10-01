//! Redis test helpers — connect to a test Redis (or local Redis) and provide
//! a per-test isolation namespace via a random key prefix.

use deadpool_redis::{Config, Pool, Runtime};
use uuid::Uuid;

pub fn test_redis_pool() -> Pool {
    let url = std::env::var("LCC_TEST_REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let cfg = Config::from_url(url);
    cfg.create_pool(Some(Runtime::Tokio1)).expect("redis pool")
}

/// Returns a unique key prefix for test isolation.
pub fn unique_prefix() -> String {
    format!("test:{}:", Uuid::now_v7())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_prefix_format() {
        let p = unique_prefix();
        assert!(p.starts_with("test:"));
    }
}
