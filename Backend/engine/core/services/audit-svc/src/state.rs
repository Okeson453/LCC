use crate::repository::PgRepository;
use crate::service::Service;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

struct Inner {
    pub svc: Arc<Service>,
}

impl AppState {
    pub async fn new(db: sqlx::PgPool) -> Self {
        let repo = PgRepository::new(db);
        let svc = Arc::new(Service::new(repo));
        // `Service` owns the pool through `PgRepository`; the copy kept here
        // was never read.
        Self(Arc::new(Inner { svc }))
    }
    pub fn service(&self) -> &Service {
        &self.0.svc
    }
}
