//! What Dart sees of the library: thin wrappers over `kanso-mobile`, one
//! file per area. The bridge generator reads this folder
//! (`mobile/flutter_rust_bridge.yaml`) and writes the Dart under
//! `mobile/lib/core/bridge`.
//!
//! No decision is made here. A wrapper calls the facade and, for a stream,
//! hands the facade's callback a stream sink. `types` mirrors the facade's
//! data types field by field, so the compiler checks that they still match.

pub mod diagnostics;
pub mod editor;
pub mod library;
pub mod locks;
pub mod renderer;
pub mod sync;
pub mod types;
