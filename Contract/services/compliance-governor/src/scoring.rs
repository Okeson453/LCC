//! Scoring client — gRPC client to scoring-intel for H_c refresh.

use lcc_compliance::h_c::H_cInputs;
use serde::{Deserialize, Serialize};

use crate::state::GovernorDeps;
use crate::AccountState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringClient {
    endpoint: String,
}

impl ScoringClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    pub fn placeholder() -> Self {
        Self {
            endpoint: "http://localhost:0".into(),
        }
    }

    /// Compute H_c via scoring-intel. Returns the fresh scalar value.
    pub async fn compute_h_c(
        &self,
        account: &AccountState,
        _deps: &GovernorDeps,
    ) -> Result<f64, String> {
        // Real implementation: tonic client to scoring-intel.ComputeH_c.
        // For now, return the cached value.
        Ok(account.h_c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn placeholder_returns_cached_h_c() {
        let client = ScoringClient::placeholder();
        let deps = GovernorDeps::placeholder();
        let account = AccountState {
            h_c: 0.65,
            h_c_computed_at: Utc::now(),
            is_restricted: false,
            restricted_since: None,
            restricted_reason: None,
            warmup_days_remaining: 0,
            days_active: 30,
            active_compliance_config_version: "test".into(),
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let h_c = rt.block_on(client.compute_h_c(&account, &deps)).unwrap();
        assert!((h_c - 0.65).abs() < 1e-9);
    }

    #[test]
    fn placeholder_constructs() {
        let _ = ScoringClient::placeholder();
        let _ = ScoringClient::new("http://localhost:50051");
    }
}
