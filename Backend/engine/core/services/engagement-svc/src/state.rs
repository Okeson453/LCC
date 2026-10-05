use std::sync::Arc;

use crate::repository::PgRepository;
use crate::service::Service;

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

struct Inner {
    pub svc: Arc<Service>,
}

impl AppState {
    pub async fn new(db: sqlx::PgPool, redis: deadpool_redis::Pool) -> Self {
        let repo = PgRepository::new(db.clone());
        let svc = Arc::new(Service::new(repo, redis));
        // The pools are not kept here: `Service` already owns them via
        // `PgRepository` and the redis pool, and a second copy here was never
        // read.
        Self(Arc::new(Inner { svc }))
    }
    pub fn service(&self) -> &Service {
        &self.0.svc
    }
}
