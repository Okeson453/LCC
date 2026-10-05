//! Orchestrator AppState.

use std::sync::Arc;

use crate::repository::PgRepository;
use crate::service::Service;

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

struct Inner {
    pub db_pool: sqlx::PgPool,
    pub redis: deadpool_redis::Pool,
    pub service: Arc<Service>,
}

impl AppState {
    pub async fn new(db_pool: sqlx::PgPool, redis: deadpool_redis::Pool) -> Self {
        let repo = PgRepository::new(db_pool.clone());
        let service = Arc::new(Service::new(repo, redis.clone()));
        Self(Arc::new(Inner {
            db_pool,
            redis,
            service,
        }))
    }

    pub fn for_test(db_pool: sqlx::PgPool, redis: deadpool_redis::Pool) -> Self {
        let repo = PgRepository::new(db_pool.clone());
        let service = Arc::new(Service::new(repo, redis.clone()));
        Self(Arc::new(Inner {
            db_pool,
            redis,
            service,
        }))
    }

    pub fn service(&self) -> &Service {
        &self.0.service
    }
    pub fn db(&self) -> &sqlx::PgPool {
        &self.0.db_pool
    }
    pub fn redis(&self) -> &deadpool_redis::Pool {
        &self.0.redis
    }
}
