use std::sync::Arc;
use crate::repository::PgRepository;
use crate::service::Service;

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

struct Inner { pub db: sqlx::PgPool, pub svc: Arc<Service> }

impl AppState {
    pub async fn new(db: sqlx::PgPool) -> Self {
        let repo = PgRepository::new(db.clone());
        let svc = Arc::new(Service::new(repo));
        Self(Arc::new(Inner { db, svc }))
    }
    pub fn service(&self) -> &Service { &self.0.svc }
}
