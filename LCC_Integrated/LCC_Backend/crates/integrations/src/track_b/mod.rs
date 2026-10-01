//! Track B — browser-assist extension messaging protocol.
//!
//! Per the spec (Source §37, Backend Design Concept §38), actions the
//! official LinkedIn API does not support (personal-profile posting,
//! connection requests, DMs, most search) are delivered to the user's
//! browser-assist extension via WSS. The extension fills the form fields
//! and waits for human confirmation before submitting.
//!
//! NEVER a fully autonomous bot flow.

pub mod extension_protocol;

pub use extension_protocol::{
    BrowserExtensionMessage, ExtensionMessageKind, TrackBError,
};
