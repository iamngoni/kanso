//! What links and navigation hand around: the locked notes the caller can
//! read, and the notes and headings a search found.

/// A locked note the lock session has opened, with the text it holds.
///
/// The store keeps such a note's text sealed (its derived rows, its search
/// index and its body all read empty), so the links, mentions and headings
/// of a note like this are found from this text instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenedNote {
    pub id: String,
    pub title: String,
    pub body: String,
    pub book_id: String,
    pub updated_at: i64,
}

/// What the caller can read of the locked notes: the ones the lock session
/// has opened, none while it is closed.
///
/// A locked note (sealed itself, or in a locked notebook) that isn't in
/// `opened` is left out of every result below: by title, by heading and by
/// link. One that is shows like any other note.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LockedNotes {
    /// Every locked note outside the trash that a key this session holds
    /// opens, with its text. Empty while closed.
    pub opened: Vec<OpenedNote>,
}

impl LockedNotes {
    /// No locked note can be read.
    pub fn closed() -> Self {
        LockedNotes::default()
    }

    /// Whether the locked note `id` has been opened.
    pub fn can_read(&self, id: &str) -> bool {
        self.opened.iter().any(|note| note.id == id)
    }
}

/// A note a search found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteHit {
    pub id: String,
    /// As stored; the caller says "Untitled" for an empty one.
    pub title: String,
    pub book_id: String,
    /// Whether the note is locked (and so only found because the lock
    /// session is open).
    pub locked: bool,
}

/// What a search of the notes found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Found {
    /// Notes whose titles have the words first, then the rest, at most
    /// [`super::quick_open::NOTE_LIMIT`].
    pub notes: Vec<NoteHit>,
    /// Headings with the words, at most
    /// [`super::quick_open::HEADING_LIMIT`]; only for Open Note.
    pub headings: Vec<kanso_store::HeadingHit>,
}
