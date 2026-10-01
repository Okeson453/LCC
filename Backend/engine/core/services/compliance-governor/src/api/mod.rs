//! HTTP + gRPC API for the Compliance Governor.

pub mod admin;
pub mod evaluate;

pub use admin::{
    activate_config, list_config_versions, propose_config, review_config,
    ActivateConfigVersionRequest, ActivateConfigVersionResponse, ListConfigVersionsResponse,
    ProposeConfigVersionRequest, ProposeConfigVersionResponse, ReviewConfigVersionRequest,
    ReviewConfigVersionResponse,
};
pub use evaluate::{evaluate_action_http, GrpcServer};
