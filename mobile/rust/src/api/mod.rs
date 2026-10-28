//! What Dart sees of the library: thin wrappers over `kanso-mobile`, one
//! file per area. The bridge generator reads this folder
//! (`mobile/flutter_rust_bridge.yaml`) and writes the Dart under
//! `mobile/lib/core/bridge`.
//!
//! No decision is made here. A wrapper calls the facade and, for a stream,
//! hands the facade's callback a stream sink. `types` mirrors the facade's
//! data types field by field, so the compiler checks that they still match.

pub mod account;
pub mod account_settings;
pub mod approvals;
pub mod diagnostics;
pub mod discovery;
pub mod editor;
pub mod gallery;
pub mod import;
pub mod library;
pub mod linking;
pub mod linking_types;
pub mod locks;
pub mod renderer;
pub mod shared;
pub mod sync;
pub mod types;
