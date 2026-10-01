//! Analytics service.

use chrono::NaiveDate;
use uuid::Uuid;

use crate::domain::{DashboardSummary, Granularity, TimeSeries};
use crate::error::Error;
use crate::repository::PgRepository;

pub struct Service {
    repo: PgRepository,
}

impl Service {
    pub fn new(repo: PgRepository) -> Self { Self { repo } }

    pub async fn dashboard(
        &self, m: Uuid, start: NaiveDate, end: NaiveDate,
    ) -> Result<DashboardSummary, Error> {
        self.repo.dashboard(m, start, end).await
    }

    pub async fn time_series(
        &self, m: Uuid, metric: &str, granularity: Granularity,
        start: NaiveDate, end: NaiveDate,
    ) -> Result<TimeSeries, Error> {
        self.repo.time_series(m, metric, granularity, start, end).await
    }
}
