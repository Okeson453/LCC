//! AppState for profile-svc.

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
    pub async fn new(db: sqlx::PgPool, redis: deadpool_redis::Pool) -> Self {
        let repo = PgRepository::new(db.clone());
        let svc = Arc::new(Service::new(repo, redis.clone()));
        Self(Arc::new(Inner {
            db_pool: db,
            redis,
            service: svc,
        }))
    }
    pub fn for_test(db: sqlx::PgPool, redis: deadpool_redis::Pool) -> Self {
        let repo = PgRepository::new(db.clone());
        let svc = Arc::new(Service::new(repo, redis.clone()));
        Self(Arc::new(Inner {
            db_pool: db,
            redis,
            service: svc,
        }))
    }
    pub fn service(&self) -> &Service {
        &self.0.service
    }
    pub fn db(&self) -> &sqlx::PgPool {
        &self.0.db_pool
    }
}
