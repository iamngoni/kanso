//! The plain data a notebook's homepage hands the shell (issue #201): the
//! header card on the note list, an open homepage, its map and its
//! settings. No logic lives here; `notebook_home.rs` builds them from the
//! engine's (`kanso_engine::pages::notebook`). The sections are the same
//! [`crate::PageSection`]s a person's page shows.

use crate::people_model::{PageHidden, PageSection};

/// Settings › Notebooks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeSettings {
    /// Show notebook homepages. Off restores today's note list and its
    /// empty-state line.
    pub show_homepages: bool,
    /// Write notebook summaries with AI.
    pub summaries_with_ai: bool,
    /// When AI sections are rewritten, shared with people's pages: `auto`,
    /// `open` or `press`.
    pub refresh: String,
}

/// The words both platforms show, from the engine, so they never differ.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeWords {
    pub ai_off: String,
    pub locked: String,
    pub first_visit_button: String,
    pub no_open_tasks: String,
    pub home_row: String,
    pub ask_button: String,
    pub open_home: String,
    pub add_home_note: String,
    pub home_moved: String,
    /// The keys of every section, in the order they are shown until the
    /// person changes it.
    pub section_keys: Vec<String>,
}

/// The header under the notebook's name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeHeaderInfo {
    pub name: String,
    pub notes: u32,
    pub notebooks_inside: u32,
    /// "48 notes · 3 notebooks inside · last edited today".
    pub line: String,
    pub has_home_note: bool,
    /// The Home note's ID, or empty.
    pub home_note_id: String,
}

/// An open homepage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomePanel {
    pub header: HomeHeaderInfo,
    /// In the order the person chose; hidden and empty sections are left out.
    pub sections: Vec<PageSection>,
    pub hidden: Vec<PageHidden>,
    /// AI sections are written for this notebook (AI is on).
    pub ai_on: bool,
    /// Sections that want rewriting now, by the person's refresh setting.
    pub due: Vec<String>,
    /// One muted line at the end, or empty.
    pub footer: String,
    /// The notebook is locked and closed: nothing else is filled in.
    pub locked: bool,
    /// The notebook is locked and open: computed sections, in memory.
    pub locked_open: bool,
}

/// The card above a notebook's first note on the phone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeCard {
    /// Homepages are on and the notebook is readable.
    pub show: bool,
    pub name: String,
    /// The AI summary's text, or empty.
    pub about_line: String,
    pub has_home_note: bool,
    /// The notebook has no notes of its own, only notebooks inside it: its
    /// homepage opens instead of an empty list.
    pub opens_directly: bool,
}

/// A note on the map.
#[derive(Clone, Debug, PartialEq)]
pub struct HomeMapNode {
    pub id: String,
    pub title: String,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub degree: u32,
    pub stubs: u32,
}

/// A link on the map, by index into the nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomeMapEdge {
    pub a: u32,
    pub b: u32,
}

/// The notebook's link graph, laid out on a `width` by `height` canvas.
#[derive(Clone, Debug, PartialEq)]
pub struct HomeMap {
    pub width: f32,
    pub height: f32,
    pub nodes: Vec<HomeMapNode>,
    pub edges: Vec<HomeMapEdge>,
}

/// A section in Customise home.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeSectionSetting {
    pub key: String,
    pub title: String,
    pub hidden: bool,
}

/// A note that can be chosen as the Home note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeCandidate {
    pub id: String,
    pub title: String,
    /// It is the Home note now.
    pub current: bool,
}
