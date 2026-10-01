//! identity-svc application state.

use std::sync::Arc;

use crate::config::Config;
use crate::repository::PgRepository;
use crate::service::Service;

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

struct Inner {
    pub config: Arc<Config>,
    pub db_pool: sqlx::PgPool,
    pub service: Arc<Service>,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self, sqlx::Error> {
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(&config.database_url)
            .await?;

        let cfg = Arc::new(config);
        let repo = PgRepository::new(db_pool.clone());
        let service = Arc::new(Service::new(repo, cfg.clone()));

        Ok(Self(Arc::new(Inner {
            config: cfg,
            db_pool,
            service,
        })))
    }

    pub fn config(&self) -> &Config {
        &self.0.config
    }
    pub fn db(&self) -> &sqlx::PgPool {
        &self.0.db_pool
    }
    pub fn service(&self) -> &Service {
        &self.0.service
    }

    /// Construct an AppState from an already-built Service. Intended for
    /// integration tests that need to exercise the router without a live
    /// database.
    pub fn for_test(service: Arc<Service>) -> Self {
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_millis(50))
            .connect_lazy(service.cfg().database_url.as_str())
            .expect("lazy pool");
        Self(Arc::new(Inner {
            config: std::sync::Arc::new(service.cfg().clone()),
            db_pool,
            service,
        }))
    }
}
