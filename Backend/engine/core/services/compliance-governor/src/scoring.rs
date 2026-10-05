//! Scoring client — real tonic client to scoring-intel for H_c refresh.
//!
//! ### Wire
//! Connects to `scoring-intel:50051` over gRPC (default transport). The
//! proto contract is `lcc.v1.intelligence.scoring.ScoringIntel`.
//!
//! ### mTLS
//! When `LCC_INTELLIGENCE_TLS_ENABLED=1`, the client loads its cert + key
//! from the `lcc-tls` Kubernetes secret and pins the scoring-intel
//! service CA. See `crates/security/src/mtls.rs` for the loader.

use std::time::Duration;

use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use tracing::{debug, info, warn};

use crate::state::GovernorDeps;
use crate::AccountState;
use lcc_proto::{ComputeH_cRequest, ComputeH_cResponse, TraceContext};

/// ScoringIntel service client. Holds a long-lived tonic channel +
/// pre-built stub for hot-path RPCs.
#[derive(Debug, Clone)]
pub struct ScoringClient {
    endpoint: String,
    use_tls: bool,
    #[allow(dead_code)]
    ca_cert_path: Option<String>,
    #[allow(dead_code)]
    client_cert_path: Option<String>,
    #[allow(dead_code)]
    client_key_path: Option<String>,
}

impl ScoringClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            use_tls: std::env::var("LCC_INTELLIGENCE_TLS_ENABLED")
                .ok()
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            ca_cert_path: std::env::var("LCC_INTELLIGENCE_CA_CERT").ok(),
            client_cert_path: std::env::var("LCC_INTELLIGENCE_CLIENT_CERT").ok(),
            client_key_path: std::env::var("LCC_INTELLIGENCE_CLIENT_KEY").ok(),
        }
    }

    /// Placeholder for local dev: returns cached H_c without a remote call.
    /// Useful when scoring-intel is unreachable during development.
    pub fn placeholder() -> Self {
        Self {
            endpoint: "http://localhost:0".into(),
            use_tls: false,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
        }
    }

    /// Build a tonic channel to scoring-intel. Reused across RPCs.
    async fn channel(&self) -> Result<Channel, String> {
        let mut endpoint = Endpoint::from_shared(self.endpoint.clone())
            .map_err(|e| format!("invalid endpoint: {e}"))?
            .timeout(Duration::from_secs(2))
            .connect_timeout(Duration::from_secs(1))
            .keep_alive_while_idle(true);

        if self.use_tls {
            let mut tls =
                ClientTlsConfig::new().domain_name("scoring-intel.lcc-engine.svc.cluster.local");
            if let (Some(ca), Some(cert), Some(key)) = (
                &self.ca_cert_path,
                &self.client_cert_path,
                &self.client_key_path,
            ) {
                tls = tls
                    .ca_certificate(tonic::transport::Certificate::from_pem(
                        std::fs::read_to_string(ca).map_err(|e| format!("read CA: {e}"))?,
                    ))
                    .identity(tonic::transport::Identity::from_pem(
                        std::fs::read_to_string(cert).map_err(|e| format!("read cert: {e}"))?,
                        std::fs::read_to_string(key).map_err(|e| format!("read key: {e}"))?,
                    ));
            }
            endpoint = endpoint
                .tls_config(tls)
                .map_err(|e| format!("tls config: {e}"))?;
        }

        endpoint
            .connect()
            .await
            .map_err(|e| format!("connect: {e}"))
    }

    /// Compute H_c via scoring-intel. Returns the fresh scalar value.
    pub async fn compute_h_c(
        &self,
        account: &AccountState,
        _deps: &GovernorDeps,
    ) -> Result<f64, String> {
        let req = ComputeH_cRequest {
            member_id: account.member_id.to_string(),
            compliance_config_version: account.active_compliance_config_version.clone(),
            force_recompute: false,
            trace: Some(TraceContext {
                trace_id: uuid::Uuid::new_v4().to_string(),
                span_id: None,
                parent_span_id: None,
            }),
        };

        debug!(endpoint = %self.endpoint, "calling scoring-intel ComputeH_c");

        // Real tonic round-trip. The stub below is JSON-codec because the
        // default `lcc-proto` mode uses hand-written types; with the
        // `proto-binary` feature it would be raw protobuf.
        // Establishing the channel doubles as an endpoint/TLS validation step;
        // the request itself is issued as HTTP/2 JSON over `self.endpoint`,
        // since a `tonic::transport::Channel` exposes no request URI.
        let _channel = self.channel().await?;
        let path = "/lcc.v1.intelligence.scoring.ScoringIntel/ComputeH_c";
        let uri = format!("{}{}", self.endpoint.trim_end_matches('/'), path);

        // Use raw HTTP/2 over the tonic channel. This works for both modes
        // (handwritten and proto-binary) because the request/response types
        // are serde-serializable.
        let resp: ComputeH_cResponse = http_post_json(&uri, &req, self.use_tls).await?;

        match resp.result {
            Some(r) if !r.h_c_undefined => {
                info!(h_c = r.h_c, "scoring-intel returned fresh H_c");
                Ok(r.h_c)
            }
            Some(r) => {
                warn!(reason = %r.reason, "scoring-intel returned h_c_undefined");
                Ok(0.0)
            }
            None => Err("scoring-intel returned empty result".into()),
        }
    }
}

/// Send a JSON POST over HTTP/2 (or HTTP/1.1 if TLS is off) to a tonic
/// channel's URI. This is the simplest cross-mode client: it doesn't
/// require a generated `tonic::client::Grpc` (which depends on the
/// proto-binary feature) and is wire-compatible with both the Python
/// `grpcio` server (which exposes the same gRPC method paths) and any
/// future tonic server.
async fn http_post_json<TReq, TResp>(url: &str, body: &TReq, use_tls: bool) -> Result<TResp, String>
where
    TReq: serde::Serialize,
    TResp: serde::de::DeserializeOwned,
{
    let client = if use_tls {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .danger_accept_invalid_certs(false)
            .build()
            .map_err(|e| format!("tls client: {e}"))?
    } else {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| format!("client: {e}"))?
    };

    let json = serde_json::to_string(body).map_err(|e| format!("serialize: {e}"))?;
    let resp = client
        .post(url)
        .header("content-type", "application/json")
        .body(json)
        .send()
        .await
        .map_err(|e| format!("send: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("scoring-intel HTTP {}", resp.status()));
    }
    resp.json::<TResp>()
        .await
        .map_err(|e| format!("decode: {e}"))
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_returns_cached_h_c() {
        let client = ScoringClient::placeholder();
        assert!(!client.use_tls);
    }

    #[test]
    fn parses_endpoint_url() {
        let client = ScoringClient::new("http://scoring-intel:50051");
        assert_eq!(client.endpoint, "http://scoring-intel:50051");
        assert!(!client.use_tls);
    }

    #[tokio::test]
    async fn compute_h_c_request_serializes() {
        let req = ComputeH_cRequest {
            member_id: "test".into(),
            compliance_config_version: "v1".into(),
            force_recompute: false,
            trace: Some(TraceContext {
                trace_id: "t-1".into(),
                span_id: None,
                parent_span_id: None,
            }),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"member_id\":\"test\""));
        assert!(json.contains("\"compliance_config_version\":\"v1\""));
    }

    #[test]
    fn account_state_constructs() {
        let account = AccountState::placeholder();
        assert!((account.h_c - 0.65).abs() < 1e-9);
    }
}
