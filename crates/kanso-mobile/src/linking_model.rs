//! What the linking and navigation calls hand back and take: the outline, a
//! note's links, and the command palette. Plain data; `linking.rs` fills it.

/// A heading of a note, as the outline lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutlineHeading {
    /// 1 to 6.
    pub level: u8,
    /// The heading's words, without Markdown markers.
    pub text: String,
    /// The line (0-based) it is on.
    pub line: u32,
}

/// A note that links to, or mentions, another, with the line it does it on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkRef {
    pub id: String,
    pub title: String,
    /// The line (0-based) of the first link or mention.
    pub line: u32,
    /// That line as plain words, shortened.
    pub context: String,
}

/// A second list a command opens to choose from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PalettePicker {
    /// The notebook to move the open note into.
    Notebook,
    Tag,
    Status,
    /// A notebook for the note list to show.
    GoToNotebook,
    /// A tag whose notes the list shows.
    GoToTag,
    WordGoal,
    Template,
    ImageSize,
}

/// A place the note list can show.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PalettePlace {
    AllNotes,
    Pinned,
    Recent,
    Tasks,
    Notebook {
        id: String,
    },
    /// A status, by its name (`active`, `onHold`, `completed`, `dropped`).
    Status {
        name: String,
    },
    Shared,
    Trash,
}

/// What choosing an entry does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PaletteTarget {
    /// Run the command, opening its list first when it has one.
    Command {
        id: String,
        picker: Option<PalettePicker>,
    },
    /// Show this place in the note list.
    Place {
        place: PalettePlace,
    },
    Note {
        id: String,
    },
    /// Open a note at one of its headings (a 0-based line).
    Heading {
        note_id: String,
        line: u32,
    },
}

/// One row of the palette or of Open Note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteEntry {
    pub target: PaletteTarget,
    pub label: String,
    /// A Lucide icon, by file name without the extension.
    pub icon: String,
    /// What a command does, for the detail view.
    pub description: Option<String>,
    /// Muted text at the end of the row: a count, or a note's notebook.
    pub hint: Option<String>,
    /// The current choice: the place showing, the open note.
    pub checked: bool,
    /// Indent, for notebooks inside other notebooks.
    pub depth: u32,
    /// A status place's status, whose glyph stands in for the icon.
    pub status: Option<String>,
}

/// A heading and its rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteSection {
    pub label: String,
    pub entries: Vec<PaletteEntry>,
}

/// What the shell knows that decides which commands the palette lists. The
/// library fills in the rest: the open note's state, the places and their
/// counts, and the notes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaletteContext {
    /// The note open now, if one is.
    pub open_note_id: Option<String>,
    pub signed_in: bool,
    /// Signed out, but the locked notes' session is still held: signing in
    /// isn't offered then.
    pub session_held: bool,
    pub ai_on: bool,
    /// Settings are open in place of the notes.
    pub settings_open: bool,
    /// The caret is on a line that is only an image.
    pub image_line: bool,
    /// The caret is on a line that links to an audio attachment.
    pub audio_line: bool,
    /// The open note is shared with others.
    pub shared_note: bool,
    /// The IDs of the commands the app has a handler for. Only those are
    /// listed.
    pub handlers: Vec<String>,
    /// The place the note list shows now.
    pub showing: Option<PalettePlace>,
    /// Overdue tasks, the count beside Tasks.
    pub overdue_tasks: u32,
}
