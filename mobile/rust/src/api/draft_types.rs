//! The drafting data types, declared again for the generator. What the
//! fields mean is said where the facade declares them
//! (`crates/kanso-mobile/src/draft_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    DraftCite, DraftEvent, DraftLength, DraftPreview, DraftShape, DraftSourceNote, DraftSourceRef,
    DraftSources, DraftSpan, DraftView,
};

#[frb(mirror(DraftShape))]
pub enum _DraftShape {
    Report,
    Essay,
    Update,
    Outline,
    Custom,
}

#[frb(mirror(DraftLength))]
pub enum _DraftLength {
    Short,
    Medium,
    Long,
}

#[frb(mirror(DraftSourceNote))]
pub struct _DraftSourceNote {
    pub id: String,
    pub title: String,
    pub words: u32,
}

#[frb(mirror(DraftSources))]
pub struct _DraftSources {
    pub notes: Vec<DraftSourceNote>,
    pub left_out_locked: u32,
    pub left_out_excluded: u32,
    pub too_many: bool,
    pub summary: String,
}

#[frb(mirror(DraftSourceRef))]
pub struct _DraftSourceRef {
    pub number: u32,
    pub id: String,
    pub title: String,
}

#[frb(mirror(DraftCite))]
pub struct _DraftCite {
    pub number: u32,
    pub heading: Option<String>,
}

#[frb(mirror(DraftSpan))]
pub struct _DraftSpan {
    pub start: u32,
    pub end: u32,
    pub cites: Vec<DraftCite>,
    pub unsourced: bool,
}

#[frb(mirror(DraftView))]
pub struct _DraftView {
    pub text: String,
    pub spans: Vec<DraftSpan>,
    pub removed: u32,
    pub not_covered: bool,
}

#[frb(mirror(DraftEvent))]
pub enum _DraftEvent {
    Status {
        line: String,
    },
    Text {
        draft: DraftView,
    },
    Done {
        draft: DraftView,
        sources: Vec<DraftSourceRef>,
    },
    CutOff {
        message: String,
    },
    Failed {
        message: String,
    },
    Stopped,
}

#[frb(mirror(DraftPreview))]
pub struct _DraftPreview {
    pub title: String,
    pub markdown: String,
    pub default_book_id: String,
}
