//! HTTP-layer request shapes local to the cloud service.
//!
//! The shared sync wire types (push/pull bodies, events) live in `kanso-types`;
//! these are only the service's own request bindings.

use serde::Deserialize;

/// Query string for `GET /v1/sync/pull`.
#[derive(Debug, Deserialize)]
pub struct PullQuery {
    /// The pulling device's id. Events that originated from this device are
    /// excluded so a device never receives its own changes echoed back.
    pub device_id: String,
    #[serde(default)]
    pub since: i64,
    pub limit: Option<i64>,
}
