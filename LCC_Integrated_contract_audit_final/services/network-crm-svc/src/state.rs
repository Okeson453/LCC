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
}
