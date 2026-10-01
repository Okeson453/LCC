//! Canonical import path for the realtime service's application state.
//!
//! `AppState` is defined in [`crate::db`] alongside the pools it owns, but the
//! transport, event and HTTP modules all refer to it as `crate::state::AppState`
//! (mirroring the other services, where `state` is the canonical home). This
//! module is that home: it re-exports the single definition rather than
//! duplicating it, so there is still exactly one `AppState` type.

pub use crate::db::AppState;
