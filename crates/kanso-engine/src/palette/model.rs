//! The command palette's data: the catalog's commands and what a search of
//! it hands back. Plain data; the catalog is `catalog.rs`, the ranking is in
//! the module above.

use crate::navigate::model::NoteHit;

/// The palette's sections of commands, in the order they appear with
/// nothing typed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Group {
    Note,
    View,
    GoTo,
    App,
}

impl Group {
    /// Every group, in the order the palette lists them.
    pub const ALL: [Group; 4] = [Group::Note, Group::View, Group::GoTo, Group::App];

    /// The section's heading.
    pub fn label(self) -> &'static str {
        match self {
            Group::Note => "Note actions",
            Group::View => "View",
            Group::GoTo => "Go to",
            Group::App => "App",
        }
    }
}

/// A second list a command opens to choose from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Picker {
    /// The notebook to move the open note into.
    Notebook,
    Tag,
    Status,
    /// A notebook for the note list to show.
    GoToNotebook,
    /// A tag whose notes the list shows.
    GoToTag,
    /// The open note's word goal.
    WordGoal,
    /// A template to make a note from.
    Template,
    /// A width for the image on the caret's line.
    ImageSize,
}

/// When a command applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Needs {
    Nothing,
    /// A note is open.
    Note,
    /// A note is open and isn't in the trash.
    LiveNote,
    /// A note is open, isn't in the trash and isn't locked.
    UnlockedNote,
    /// A locked note is open.
    LockedNote,
    /// Settings are open, in place of the notes.
    Settings,
    /// The device is signed in to an account.
    SignedIn,
    /// The device isn't signed in (or was signed out by the server).
    SignedOut,
    /// The caret is on a line that is only an image, in the note's text.
    ImageLine,
    /// The caret is on a line that links to an audio attachment.
    AudioLine,
    /// A shared note this account is in is open.
    SharedNote,
    /// AI is on in Settings.
    AiOn,
    /// AI is on and a note is open that isn't in the trash or locked.
    AiNote,
}

/// A command's label and icon while the open note already has what it
/// toggles (for pinning: "Unpin note").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Toggled {
    pub label: &'static str,
    pub icon: &'static str,
}

/// One command of the catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command {
    /// Stable and unique; the desktop's command has the same one.
    pub id: &'static str,
    pub label: &'static str,
    pub toggled: Option<Toggled>,
    /// One or two sentences for the detail view.
    pub description: &'static str,
    /// The command's Lucide icon, by file name without the extension
    /// (`file-plus`).
    pub icon: &'static str,
    /// Other words people might type for it.
    pub keywords: &'static [&'static str],
    pub group: Group,
    /// The list the command opens, when it opens one.
    pub picker: Option<Picker>,
    pub needs: Needs,
}

/// What kind of note is open.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpenNote {
    pub id: String,
    /// Sealed under the lock password, or in a locked notebook.
    pub locked: bool,
    /// In the trash.
    pub trashed: bool,
    pub pinned: bool,
}

/// A place the note list can show.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Place {
    AllNotes,
    Pinned,
    Recent,
    Tasks,
    Notebook(String),
    /// A status, by its name (`active`, `on-hold`, …).
    Status(String),
    Shared,
    Trash,
}

/// A notebook the palette lists, in tree order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BookSource {
    pub id: String,
    pub name: String,
    /// 0 for a notebook at the top.
    pub depth: u32,
    pub count: u32,
}

/// What the "Go to" places are made of.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlaceSources {
    pub all: u32,
    pub pinned: u32,
    pub recent: u32,
    /// Overdue tasks.
    pub overdue: u32,
    pub trash: u32,
    /// Notes shared with this account.
    pub shared: u32,
    /// The statuses' note counts, by status name.
    pub by_status: Vec<(String, u32)>,
    pub books: Vec<BookSource>,
    /// The place the note list shows now, to check.
    pub showing: Option<Place>,
}

/// What a search of the palette needs to know.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Context {
    /// The open note, if one is.
    pub note: Option<OpenNote>,
    pub signed_in: bool,
    /// The account was signed out but its locked session is still held:
    /// signing in isn't offered then.
    pub session_held: bool,
    pub ai_on: bool,
    pub settings_open: bool,
    /// The caret is on a line that is only an image.
    pub image_line: bool,
    /// The caret is on a line that links to an audio attachment.
    pub audio_line: bool,
    /// The open note is shared with others.
    pub shared_note: bool,
    /// The IDs of the commands the app has handlers for: only those are
    /// listed, so a package that lands later adds its commands by
    /// registering a handler.
    pub handlers: Vec<String>,
    pub places: PlaceSources,
    /// The notes the same search as Open Note found for the query, as
    /// `Context::notes`'s caller asked it; listed under "Notes" once a query
    /// is typed.
    pub notes: Vec<NoteHit>,
    /// The notebooks' names by ID, for the hint beside a note.
    pub book_names: Vec<(String, String)>,
}

/// What choosing an entry does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// Run the command, opening its list first when it has one.
    Command {
        id: String,
        picker: Option<Picker>,
    },
    /// Show this place in the note list.
    Place(Place),
    Note {
        id: String,
    },
    /// Open a note at one of its headings (a 0-based line).
    Heading {
        note_id: String,
        line: u32,
    },
}

/// One row of the palette.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub target: Target,
    pub label: String,
    /// A Lucide icon name.
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
pub struct Section {
    pub label: String,
    pub entries: Vec<Entry>,
}
