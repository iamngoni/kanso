//! The data of a notebook homepage (issue #201): its header, its map and the
//! words both platforms show. No logic beyond small accessors; the page is
//! built in [`super::notebook`].

use super::PageView;

/// What an AI section says in place of lines while AI is off.
pub const AI_OFF_LINE: &str = "Turn on AI in Settings to add a summary and decisions.";
/// What a locked notebook's homepage says at its end.
pub const LOCKED_LINE: &str = "Summaries are off for locked notebooks.";
/// The button an AI section shows on a notebook's first visit when an API
/// key pays for it.
pub const FIRST_VISIT_BUTTON: &str = "Write a summary of this notebook";
/// Open tasks, when the notebook has tasks and they are all done.
pub const NO_OPEN_TASKS: &str = "No open tasks";
/// The note-list row and menu entry for the homepage.
pub const HOME_ROW: &str = "Home";
/// The button that opens Ask scoped to the notebook.
pub const ASK_BUTTON: &str = "Ask about this notebook";
/// The mobile header card's link.
pub const OPEN_HOME: &str = "Open home";
/// The mobile header card's prompt when there is no About line.
pub const ADD_HOME_NOTE: &str = "Add a Home note";
/// The toast when a moved Home note loses its mark.
pub const HOME_MOVED_LINE: &str =
    "That notebook already has a Home note, so this one is an ordinary note now.";

/// What the header under the title says.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HomeHeader {
    pub name: String,
    pub notes: u32,
    pub notebooks_inside: u32,
    /// "today", "yesterday" or "28 Oct"; empty for an empty notebook.
    pub last_edited: String,
    /// "48 notes · 3 notebooks inside · last edited today".
    pub line: String,
    /// The notebook has a Home note chosen.
    pub has_home_note: bool,
}

/// A note in the map.
#[derive(Clone, Debug, PartialEq)]
pub struct MapNode {
    pub id: String,
    pub title: String,
    /// Where it is on the canvas ([`MapView::width`] by [`MapView::height`]).
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    /// Links to and from the other notes of the map.
    pub degree: u32,
    /// Links to notes outside the notebook, drawn as short stubs.
    pub stubs: u32,
}

/// The notebook's link graph, laid out for a card.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapView {
    pub width: f32,
    pub height: f32,
    pub nodes: Vec<MapNode>,
    /// Indices into `nodes`, `a <= b`, each once.
    pub edges: Vec<(u32, u32)>,
}

/// An open homepage.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HomeView {
    pub header: HomeHeader,
    /// The sections in the order the person chose, hidden ones left out.
    pub page: PageView,
    /// One muted line at the end ([`AI_OFF_LINE`], [`LOCKED_LINE`]).
    pub footer: Option<String>,
    /// The notebook is locked and not opened: nothing else is filled in.
    pub locked: bool,
    /// AI sections are written for this notebook (AI on, the notebook not
    /// locked).
    pub ai_on: bool,
}

/// A note of a locked notebook as an open session reads it, for the
/// in-memory homepage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenedNote {
    pub id: String,
    pub title: String,
    pub body: String,
    pub book_id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub pinned: bool,
}
