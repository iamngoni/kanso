//! The notebook homepage data types, declared again for the generator, as
//! `types.rs` does for the rest. What the fields mean is said where the
//! facade declares them (`crates/kanso-mobile/src/notebook_home_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    HomeCandidate, HomeCard, HomeHeaderInfo, HomeMap, HomeMapEdge, HomeMapNode, HomePanel,
    HomeSectionSetting, HomeSettings, HomeWords,
};

use super::people_types::{PageHidden, PageSection};

#[frb(mirror(HomeSettings))]
pub struct _HomeSettings {
    pub show_homepages: bool,
    pub summaries_with_ai: bool,
    pub refresh: String,
}

#[frb(mirror(HomeWords))]
pub struct _HomeWords {
    pub ai_off: String,
    pub locked: String,
    pub first_visit_button: String,
    pub no_open_tasks: String,
    pub home_row: String,
    pub ask_button: String,
    pub open_home: String,
    pub add_home_note: String,
    pub home_moved: String,
    pub section_keys: Vec<String>,
}

#[frb(mirror(HomeHeaderInfo))]
pub struct _HomeHeaderInfo {
    pub name: String,
    pub notes: u32,
    pub notebooks_inside: u32,
    pub line: String,
    pub has_home_note: bool,
    pub home_note_id: String,
}

#[frb(mirror(HomePanel))]
pub struct _HomePanel {
    pub header: HomeHeaderInfo,
    pub sections: Vec<PageSection>,
    pub hidden: Vec<PageHidden>,
    pub ai_on: bool,
    pub due: Vec<String>,
    pub footer: String,
    pub locked: bool,
    pub locked_open: bool,
}

#[frb(mirror(HomeCard))]
pub struct _HomeCard {
    pub show: bool,
    pub name: String,
    pub about_line: String,
    pub has_home_note: bool,
    pub opens_directly: bool,
}

#[frb(mirror(HomeMapNode))]
pub struct _HomeMapNode {
    pub id: String,
    pub title: String,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub degree: u32,
    pub stubs: u32,
}

#[frb(mirror(HomeMapEdge))]
pub struct _HomeMapEdge {
    pub a: u32,
    pub b: u32,
}

#[frb(mirror(HomeMap))]
pub struct _HomeMap {
    pub width: f32,
    pub height: f32,
    pub nodes: Vec<HomeMapNode>,
    pub edges: Vec<HomeMapEdge>,
}

#[frb(mirror(HomeSectionSetting))]
pub struct _HomeSectionSetting {
    pub key: String,
    pub title: String,
    pub hidden: bool,
}

#[frb(mirror(HomeCandidate))]
pub struct _HomeCandidate {
    pub id: String,
    pub title: String,
    pub current: bool,
}
