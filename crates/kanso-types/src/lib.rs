//! Shared domain types for Kanso.
//!
//! Lives between the on-device engine (`kanso-engine`) and the sync service
//! (`kanso-cloud`) so the sync wire format is defined exactly once.

pub mod auth;
pub mod ids;
pub mod payloads;
pub mod sync;

pub use auth::*;
pub use ids::*;
pub use sync::*;
