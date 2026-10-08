use std::sync::Arc;

use crate::repository::PgRepository;
use crate::service::Service;

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

struct Inner {
    pub db: sqlx::PgPool,
    pub redis: deadpool_redis::Pool,
    pub svc: Arc<Service>,
}

impl AppState {
    pub async fn new(db: sqlx::PgPool, redis: deadpool_redis::Pool) -> Self {
        let repo = PgRepository::new(db.clone());
        let svc = Arc::new(Service::new(repo, redis.clone()));
        Self(Arc::new(Inner { db, redis, svc }))
    }
    pub fn for_test(db: sqlx::PgPool, redis: deadpool_redis::Pool) -> Self {
        let repo = PgRepository::new(db.clone());
        let svc = Arc::new(Service::new(repo, redis.clone()));
        Self(Arc::new(Inner { db, redis, svc }))
    }
    pub fn service(&self) -> &Service {
        &self.0.svc
    }
    /// Direct pool access for health/readiness probes and for handlers that
    /// need a transaction rather than a repository call.
    pub fn db(&self) -> &sqlx::PgPool {
        &self.0.db
    }
    pub fn redis(&self) -> &deadpool_redis::Pool {
        &self.0.redis
    }
}

/// Builds a deadpool-redis pool for tests without a running Redis.
///
/// `get()` on the result fails, which is what the service expects: nothing in
/// the CRM path uses Redis today, so a pool that cannot connect is the honest
/// fixture.
///
/// `create_pool` itself only fails on an unparseable URL, and the URL here is a
/// literal, so a panic here means the test harness itself is broken — worth
/// failing loudly rather than handing back a pool that misbehaves later. The
/// crate-wide `expect_used` deny does not fit a fixture.
#[allow(clippy::expect_used)]
pub fn placeholder_redis() -> deadpool_redis::Pool {
    deadpool_redis::Config::from_url("redis://127.0.0.1:1")
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("placeholder redis pool")
}
