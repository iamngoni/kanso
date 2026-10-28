//! The linking and navigation data types, declared again for the generator,
//! as `types.rs` does for the rest. What the fields mean is said where the
//! facade declares them (`crates/kanso-mobile/src/linking_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    LinkRef, OutlineHeading, PaletteContext, PaletteEntry, PalettePicker, PalettePlace,
    PaletteSection, PaletteTarget,
};

#[frb(mirror(OutlineHeading))]
pub struct _OutlineHeading {
    pub level: u8,
    pub text: String,
    pub line: u32,
}

#[frb(mirror(LinkRef))]
pub struct _LinkRef {
    pub id: String,
    pub title: String,
    pub line: u32,
    pub context: String,
}

#[frb(mirror(PalettePicker))]
pub enum _PalettePicker {
    Notebook,
    Tag,
    Status,
    GoToNotebook,
    GoToTag,
    WordGoal,
    Template,
    ImageSize,
}

#[frb(mirror(PalettePlace))]
pub enum _PalettePlace {
    AllNotes,
    Pinned,
    Recent,
    Tasks,
    Notebook { id: String },
    Status { name: String },
    Shared,
    Trash,
}

#[frb(mirror(PaletteTarget))]
pub enum _PaletteTarget {
    Command {
        id: String,
        picker: Option<PalettePicker>,
    },
    Place {
        place: PalettePlace,
    },
    Note {
        id: String,
    },
    Heading {
        note_id: String,
        line: u32,
    },
}

#[frb(mirror(PaletteEntry))]
pub struct _PaletteEntry {
    pub target: PaletteTarget,
    pub label: String,
    pub icon: String,
    pub description: Option<String>,
    pub hint: Option<String>,
    pub checked: bool,
    pub depth: u32,
    pub status: Option<String>,
}

#[frb(mirror(PaletteSection))]
pub struct _PaletteSection {
    pub label: String,
    pub entries: Vec<PaletteEntry>,
}

#[frb(mirror(PaletteContext))]
pub struct _PaletteContext {
    pub open_note_id: Option<String>,
    pub signed_in: bool,
    pub session_held: bool,
    pub ai_on: bool,
    pub settings_open: bool,
    pub image_line: bool,
    pub audio_line: bool,
    pub shared_note: bool,
    pub handlers: Vec<String>,
    pub showing: Option<PalettePlace>,
    pub overdue_tasks: u32,
}
