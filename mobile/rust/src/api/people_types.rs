//! The People & Projects data types, declared again for the generator, as
//! `types.rs` does for the rest. What the fields mean is said where the
//! facade declares them (`crates/kanso-mobile/src/people_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    PageHeaderInfo, PageHidden, PageItem, PagePanel, PageSection, PageSectionState, PageSource,
    PageTask, PeopleKind, PeopleMerged, PeoplePage, PeopleSettings, PeopleStatus, PeopleSuggestion,
};

#[frb(mirror(PeopleKind))]
pub enum _PeopleKind {
    Person,
    Project,
}

#[frb(mirror(PeopleSettings))]
pub struct _PeopleSettings {
    pub suggest: bool,
    pub keep_in: String,
    pub write_with_ai: bool,
    pub refresh: String,
    pub detect_with_ai: bool,
}

#[frb(mirror(PeopleStatus))]
pub struct _PeopleStatus {
    pub pages: u32,
    pub suggestions: u32,
    pub new_label: String,
    pub suggest_on: bool,
    pub ai_on: bool,
}

#[frb(mirror(PeoplePage))]
pub struct _PeoplePage {
    pub id: String,
    pub title: String,
    pub kind: PeopleKind,
    pub notes: u32,
    pub detail: String,
}

#[frb(mirror(PeopleSuggestion))]
pub struct _PeopleSuggestion {
    pub key: String,
    pub name: String,
    pub kind: PeopleKind,
    pub detail: String,
    pub also_line: String,
    pub alias_of_id: String,
    pub alias_of_title: String,
}

#[frb(mirror(PageSource))]
pub struct _PageSource {
    pub note_id: String,
    pub title: String,
    pub updated: String,
    pub start: u32,
    pub end: u32,
    pub quote: String,
    pub label: String,
}

#[frb(mirror(PageTask))]
pub struct _PageTask {
    pub note_id: String,
    pub line: u32,
    pub text: String,
    pub checked: bool,
}

#[frb(mirror(PageItem))]
pub struct _PageItem {
    pub text: String,
    pub who: String,
    pub sources: Vec<PageSource>,
    pub meta: String,
    pub group: String,
    pub task: Option<PageTask>,
    pub target: String,
    pub tag: String,
    pub shared: bool,
}

#[frb(mirror(PageSectionState))]
pub enum _PageSectionState {
    Fresh,
    Stale,
    Pinned,
    Unwritten,
    Off,
}

#[frb(mirror(PageSection))]
pub struct _PageSection {
    pub key: String,
    pub title: String,
    pub ai: bool,
    pub items: Vec<PageItem>,
    pub state: PageSectionState,
    pub changed: u32,
    pub written_ms: i64,
    pub total: u32,
    pub note: String,
    pub empty_line: String,
    pub stale_line: String,
}

#[frb(mirror(PageHidden))]
pub struct _PageHidden {
    pub key: String,
    pub title: String,
}

#[frb(mirror(PageHeaderInfo))]
pub struct _PageHeaderInfo {
    pub title: String,
    pub kind: PeopleKind,
    pub aliases: Vec<String>,
    pub notes: u32,
    pub last_mentioned: String,
    pub line: String,
}

#[frb(mirror(PagePanel))]
pub struct _PagePanel {
    pub header: PageHeaderInfo,
    pub sections: Vec<PageSection>,
    pub hidden: Vec<PageHidden>,
    pub ai_on: bool,
    pub due: Vec<String>,
}

#[frb(mirror(PeopleMerged))]
pub struct _PeopleMerged {
    pub into: String,
    pub links_retitled: u32,
}
