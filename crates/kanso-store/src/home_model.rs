//! The plain rows a notebook homepage is built from (issue #201). No
//! logic: `home.rs` fills them, `kanso_engine::pages::notebook` shapes them.

/// A note of a notebook's subtree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeNoteRow {
    pub id: String,
    pub title: String,
    pub book_id: String,
    pub book_name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub pinned: bool,
    /// How many saved versions it has (an edit count).
    pub revisions: u32,
}

/// What the header line counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HomeStats {
    /// Notes in the notebook and the unlocked notebooks inside it.
    pub notes: u32,
    /// Unlocked notebooks inside it, at any depth.
    pub notebooks_inside: u32,
    pub last_edited: Option<i64>,
}

/// A notebook directly inside another, as "Inside this notebook" lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeChild {
    pub id: String,
    pub name: String,
    /// Locked: only its name is known, and no counts.
    pub locked: bool,
    pub notes: u32,
    pub last_edited: Option<i64>,
    pub open_tasks: u32,
}

/// How many notes of the subtree link to a note of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeLinked {
    pub note_id: String,
    pub linked_from: u32,
}

/// A person or project mentioned in the subtree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeEntity {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub notes: u32,
}

/// A note that is a notebook's Home note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeNoteMark {
    pub id: String,
    pub title: String,
    pub updated_at: i64,
}

/// A notebook of a homepage's subtree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeBook {
    pub id: String,
    pub name: String,
    /// The notebook it is in; `None` for the homepage's own notebook when
    /// it is at the top.
    pub parent: Option<String>,
}

/// One read of a notebook's subtree: the unlocked notebooks in it and
/// every readable note of them, first created first. Every computed
/// section is shaped from this and from the tables kept per note (links,
/// tasks, mentions, revisions), read by note ID, so a homepage walks the
/// notebook's documents once. Empty when the notebook is gone or locked.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HomeScan {
    /// The notebook the scan is of.
    pub id: String,
    /// The notebook itself first, then the unlocked ones inside it.
    pub books: Vec<HomeBook>,
    /// `revisions` is 0 here: it is counted when asked for.
    pub notes: Vec<HomeNoteRow>,
}
