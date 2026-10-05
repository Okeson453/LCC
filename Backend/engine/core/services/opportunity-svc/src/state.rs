use crate::repository::PgRepository;
use crate::service::Service;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

struct Inner {
    pub svc: Arc<Service>,
}

impl AppState {
    pub async fn new(db: sqlx::PgPool, redis: deadpool_redis::Pool) -> Self {
        let repo = PgRepository::new(db);
        let svc = Arc::new(Service::new(repo, redis));
        // The pools are not held here: `Service` already owns them through
        // `PgRepository`, and the copies kept alongside were never read.
        Self(Arc::new(Inner { svc }))
    }
    pub fn service(&self) -> &Service {
        &self.0.svc
    }
}
