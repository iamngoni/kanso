//! The recording summary data types, declared again for the generator, as
//! `types.rs` does for the rest. What the fields mean is said where the
//! facade declares them
//! (`crates/kanso-mobile/src/recording_summary_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{SummaryApplied, SummaryDraftView, SummaryItemView, SummaryOffer};

#[frb(mirror(SummaryOffer))]
pub struct _SummaryOffer {
    pub available: bool,
    pub words: u32,
    pub send_line: String,
    pub replaces: bool,
}

#[frb(mirror(SummaryItemView))]
pub struct _SummaryItemView {
    pub text: String,
    pub who: Option<String>,
    pub when: Option<String>,
    pub moment: u32,
}

#[frb(mirror(SummaryDraftView))]
pub struct _SummaryDraftView {
    pub file_id: String,
    pub overview: Vec<String>,
    pub decisions: Vec<SummaryItemView>,
    pub actions: Vec<SummaryItemView>,
    pub dropped: u32,
    pub cut: bool,
}

#[frb(mirror(SummaryApplied))]
pub struct _SummaryApplied {
    pub replaced: bool,
    pub undo: Option<u64>,
}
