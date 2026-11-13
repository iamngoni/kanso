//! The embargo's data type, declared again for the generator, as `types.rs`
//! does for the rest. What the fields mean is said where the facade declares
//! it (`crates/kanso-mobile/src/embargo_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names this type by this module's path.
pub use kanso_mobile::EmbargoView;

#[frb(mirror(EmbargoView))]
pub struct _EmbargoView {
    pub lifts: String,
    pub lifts_in: String,
    pub lifts_at_ms: i64,
    pub zone: Option<String>,
}
