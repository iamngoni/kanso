//! The plain data the facade returns and takes: rows, views, queries and
//! statuses. No logic lives here; the modules that build them say what each
//! field holds.
//!
//! Every type is `Clone + Debug + PartialEq` with public fields, so the
//! bridge can mirror them field by field. Counts are `u32`, times are
//! milliseconds since the Unix epoch (`i64`), colors are `0xRRGGBB` (`u32`),
//! and text offsets are UTF-16 code units, the unit Flutter counts in.

/// One note in a list.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteRow {
    pub id: String,
    pub rev: String,
    /// A locked note's title shows as any note's does.
    pub title: String,
    /// A few lines of the text. Empty for a locked note, open or not.
    pub snippet: String,
    pub notebook_id: String,
    pub status: String,
    /// Tag IDs.
    pub tags: Vec<String>,
    pub pinned: bool,
    pub created_ms: i64,
    pub updated_ms: i64,
    /// Task-list items done, of how many. Both 0 for a locked note.
    pub tasks_done: u32,
    pub tasks_total: u32,
    /// Sealed under the lock password, or in a locked notebook.
    pub locked: bool,
}

/// What a note's lock allows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum LockState {
    /// Not locked.
    #[default]
    Plain,
    /// Locked, and open with a key this session holds.
    Open,
    /// Locked and closed. `reason` is a sentence when a password won't help
    /// (the sealed text is damaged, or from a newer Kanso).
    Closed { reason: Option<String> },
}

impl LockState {
    /// Whether the note's text can't be read.
    pub fn is_closed(&self) -> bool {
        matches!(self, LockState::Closed { .. })
    }
}

/// One note, to read or edit.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteView {
    pub id: String,
    pub rev: String,
    pub title: String,
    /// The text; `None` while the note is locked and closed.
    pub body: Option<String>,
    pub lock: LockState,
    pub notebook_id: String,
    pub tags: Vec<String>,
    pub status: String,
    pub pinned: bool,
    pub created_ms: i64,
    pub updated_ms: i64,
}

/// Where a list is: one of the places the library offers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ListPlace {
    /// Every note outside the trash.
    #[default]
    All,
    Pinned,
    /// Notes changed in the last seven days, newest first whatever the sort.
    Recent,
    /// One notebook's own notes.
    Notebook {
        id: String,
    },
    /// Notes with this tag (its ID). A locked note is never listed here: a
    /// tag on it is its content.
    Tag {
        id: String,
    },
    Trash,
}

/// Which notes a list shows, and in what order. The default is every note
/// outside the trash, newest edit first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListQuery {
    pub place: ListPlace,
    /// Only notes with this status (`active`, `onHold`, ...). A locked note
    /// is not listed then: its status is its content.
    pub status_only: Option<String>,
    pub pinned_only: bool,
    pub sort: SortKey,
    pub descending: bool,
}

impl Default for ListQuery {
    fn default() -> Self {
        ListQuery {
            place: ListPlace::All,
            status_only: None,
            pinned_only: false,
            sort: SortKey::Updated,
            descending: true,
        }
    }
}

/// Which date a list is under headers of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateField {
    Updated,
    Created,
}

/// How a list is laid out, as the desktop decides it (`kanso_engine::list`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListShape {
    /// The sort the list is in: Recent is always by edit time.
    pub sort: SortKey,
    /// Whether the notes sit under headers for their days.
    pub by_date: bool,
    /// The date the headers and the rows' times are of.
    pub date: DateField,
    /// Whether pinned notes sit under their own header.
    pub group_pinned: bool,
}

/// A list: the rows, in order, and how to lay them out.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteList {
    pub rows: Vec<NoteRow>,
    pub shape: ListShape,
}

/// The counts the library's places show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LibraryOverview {
    /// Notes outside the trash.
    pub all: u32,
    pub pinned: u32,
    /// Notes changed in the last seven days.
    pub recent: u32,
    pub trash: u32,
}

/// What is done to every note in a change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoteChange {
    /// Pins every note, or unpins them when they are all pinned.
    TogglePin,
    /// Into this notebook.
    Move { notebook_id: String },
    /// Into the trash.
    Trash,
}

/// What a search looks in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SearchScope {
    /// Titles and texts.
    #[default]
    All,
    /// Titles only.
    Titles,
    /// Tag names (a tag, or the tags inside it). A locked note is never
    /// found this way.
    Tags,
    /// One notebook's notes.
    Notebook { id: String },
}

/// A notebook's job, which it keeps however it is renamed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotebookRole {
    Inbox,
    Journal,
    Templates,
}

/// What a list is ordered by.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortKey {
    #[default]
    Updated,
    Created,
    Title,
}

/// A new note.
///
/// Where it goes is decided in `kanso_engine::list::new_note_target`, which
/// the desktop's New Note asks too: `notebook_id` if given, else the notebook
/// `place` is, else `preferred_notebook_id` if that exists and isn't locked,
/// else the first notebook (made if there is none).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewNote {
    /// The notebook it goes in, whatever else is given; it must exist
    /// ([`KansoError::NotFound`]). A locked notebook takes it sealed, and
    /// needs the lock open.
    pub notebook_id: Option<String>,
    pub title: String,
    pub body: String,
    /// The list it is made from: a notebook's list is that notebook, a tag's
    /// list tags the note, Pinned pins it. Trash and the rest leave it to the
    /// preferred notebook.
    pub place: Option<ListPlace>,
    /// The notebook to use when `place` doesn't name one (the Default
    /// notebook setting). Unknown and locked ones are ignored.
    pub preferred_notebook_id: Option<String>,
    /// Tags (their IDs) the note starts with, besides the place's.
    pub tags: Vec<String>,
    /// Pinned from the start (a list narrowed to pinned notes).
    pub pinned: bool,
}

/// A write's result: the document and its new revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Saved {
    pub id: String,
    pub rev: String,
}

/// A notebook in the sidebar. The rows come in tree order: a notebook is
/// followed by the ones inside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotebookRow {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    /// Notes in it.
    pub count: u32,
    /// The notebook has a lock of its own; its notes read as locked.
    pub locked: bool,
    /// The icon's Lucide name and the wash color, as the notebook syncs them.
    pub icon: Option<String>,
    pub color: Option<String>,
    /// Inbox, Journal or Templates, when it is one.
    pub role: Option<NotebookRole>,
    /// How many notebooks it is inside; 0 at the top.
    pub depth: u32,
}

/// A tag in the sidebar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagRow {
    pub id: String,
    pub name: String,
    /// `0xRRGGBB`.
    pub color: u32,
    pub count: u32,
}

/// A search result: the row, and where the words matched.
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub row: NoteRow,
    /// `[start, end)` in UTF-16 units of `row.title`.
    pub title_ranges: Vec<Range16>,
    /// `[start, end)` in UTF-16 units of `row.snippet`.
    pub snippet_ranges: Vec<Range16>,
}

/// A half-open range of UTF-16 code units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range16 {
    pub start: u32,
    pub end: u32,
}

/// What a property's value is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyKind {
    Text,
    Number,
    Bool,
    /// `YYYY-MM-DD`.
    Date,
    List,
    /// Kept as written: YAML the properties panel doesn't understand.
    Raw,
}

/// One property from a note's front matter.
#[derive(Clone, Debug, PartialEq)]
pub struct Property {
    pub key: String,
    pub kind: PropertyKind,
    /// The value as a line of text (a list comma-separated).
    pub text: String,
    /// A list's items, as text; empty for any other kind.
    pub items: Vec<String>,
}

/// A query view's table.
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    /// The column labels; the first is the title.
    pub columns: Vec<String>,
    pub rows: Vec<TableRow>,
}

/// A row of a query view: the note it links to and a cell for each column
/// after the title (`None` where the note has no value).
#[derive(Clone, Debug, PartialEq)]
pub struct TableRow {
    pub id: String,
    pub title: String,
    pub cells: Vec<Option<String>>,
}

/// Why a lock event happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockReason {
    /// The person locked the notes.
    Manual,
    /// The auto-lock time passed.
    Idle,
}

/// The lock session changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockEvent {
    /// A lock password opened the locked notes.
    Unlocked,
    /// The locked notes closed.
    Locked { reason: LockReason },
}

/// Where sync stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SyncPhase {
    /// No account is signed in.
    #[default]
    SignedOut,
    /// Signed in, with no sync run yet or none since.
    Idle,
    Syncing,
    /// The last sync finished (perhaps with a `problem`).
    Synced,
    /// The last sync failed.
    Failed,
}

/// What went wrong with a sync, in the words of the sync problems table
/// (docs/ACCOUNTS.md › Problems).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncProblemKind {
    SignedOut,
    AccountDisabled,
    Paused,
    ServerBusy,
    BackOff,
    AccountFull,
    ServerFull,
    SomeItemsSkipped,
    UpdateKanso,
    KeysDontMatch,
    ServerNeedsUpdate,
    Offline,
    Other,
}

/// The sync status the sidebar shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SyncStatus {
    pub phase: SyncPhase,
    /// The account's name, when signed in.
    pub username: Option<String>,
    pub server: Option<String>,
    /// When the last sync finished.
    pub last_synced_ms: Option<i64>,
    /// Conflicts in the last sync; the losing versions are in history.
    pub conflicts: u32,
    pub problem: Option<SyncProblemKind>,
}

/// What `sync_now` did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SyncReportRow {
    /// Remote changes written here.
    pub pulled: u32,
    /// Local changes the server took.
    pub pushed: u32,
    pub conflicts: u32,
    /// Local changes still not on the server.
    pub pending: u32,
    /// A sync was already running, and this one runs after it: nothing else
    /// here is filled in.
    pub queued: bool,
}

/// What a server said about itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerInfo {
    /// Whether the server takes new accounts.
    pub signups_open: bool,
}

/// A highlighted run of a note's source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpanRow {
    /// `[start, end)` in UTF-16 units of the text.
    pub range: Range16,
    /// The theme `syntax` keys that style it, outermost first.
    pub tokens: Vec<String>,
}

/// The colors a drawing or a preview is made in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaletteDto {
    pub dark: bool,
    pub background: u32,
    pub text: u32,
    pub muted: u32,
    pub accent: u32,
    pub border: u32,
    pub surface: u32,
    pub heading: Option<u32>,
    pub quote: u32,
    pub task_done: u32,
    pub code_text: Option<u32>,
    pub inline_code_background: u32,
    pub inline_code_text: Option<u32>,
    pub table_header_background: Option<u32>,
}

/// A named color.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorRow {
    pub name: String,
    pub rgb: u32,
}

/// How a syntax token is drawn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxStyleRow {
    /// The token's theme key (`heading`, `keyword`, ...).
    pub token: String,
    pub color: Option<u32>,
    pub italic: Option<bool>,
    /// 100 to 900.
    pub weight: Option<u16>,
}

/// A theme fitted for drawing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeTokens {
    pub name: String,
    pub dark: bool,
    /// The interface colors, by name.
    pub ui: Vec<ColorRow>,
    pub caret: u32,
    /// Selected text: `0xRRGGBB` and its alpha out of 255.
    pub selection: u32,
    pub selection_alpha: u8,
    pub syntax: Vec<SyntaxStyleRow>,
    /// The colors the preview and drawings are made in.
    pub palette: PaletteDto,
}

/// A built-in theme: who made it and how it is drawn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuiltinTheme {
    /// The theme file's name without `.json`: `nord`, `github-light`.
    pub id: String,
    pub name: String,
    pub author: String,
    pub license: String,
    pub source: Option<String>,
    pub dark: bool,
    /// The theme fitted as one theme for the interface, the editor and the
    /// preview (the desktop's own panes).
    pub tokens: ThemeTokens,
}

/// A run of a note's source in a theme's colors. Text between runs, and any
/// attribute a run leaves `None`, is drawn in the theme's plain text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyledRun {
    /// `[start, end)` in UTF-16 units of the text.
    pub range: Range16,
    /// `0xRRGGBB`.
    pub color: Option<u32>,
    pub italic: Option<bool>,
    /// 100 to 900.
    pub weight: Option<u16>,
}

/// Which diagram language a source is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagramKind {
    Mermaid,
    Excalidraw,
}

/// A finished drawing.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgRow {
    /// A standalone SVG document, its size set in points.
    pub markup: String,
    pub width: f32,
    pub height: f32,
    /// For inline math: how far below the top the text's baseline falls.
    pub baseline: Option<f32>,
}

// MARK: The editing session

/// One change to the text, as the shell's text field made it. Offsets are
/// UTF-16 units of the visible text (a note's front matter is not in it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextChange {
    /// The version of the text this change was made against. A change made
    /// against an older text is refused.
    pub base_version: u64,
    /// The units replaced, in the text before the change.
    pub start: u32,
    pub end: u32,
    /// What they became.
    pub text: String,
    /// The selection after the change.
    pub selection: Range16,
    /// The text an input method is composing after the change, if any.
    pub composing: Option<Range16>,
}

/// A replacement of units `start..end` of the text before an outcome by
/// `text`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replacement {
    pub start: u32,
    pub end: u32,
    pub text: String,
}

/// Whether undo and redo have a step to take.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HistoryState {
    pub can_undo: bool,
    pub can_redo: bool,
}

/// What an edit, a command, an undo or a redo did to the session's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditOutcome {
    /// The version of the text now; the next change is made against it.
    pub version: u64,
    /// What the shell's text must have done to it to be the session's text:
    /// `None` when it already is (the shell typed the change itself).
    pub replace: Option<Replacement>,
    /// The selection the text has now.
    pub selection: Range16,
    pub history: HistoryState,
}

/// Where the caret is, for the shell's keys and popups. Answered with every
/// selection, so it is cheap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorContext {
    pub in_list: bool,
    pub in_table: bool,
    pub in_code: bool,
    /// The caret is in an open `[[…` or `#tag`: a completion is worth asking
    /// for.
    pub completion_here: bool,
    pub history: HistoryState,
}

/// The text formats of the accessory bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatKind {
    Bold,
    Italic,
    Strikethrough,
    Code,
    Bulleted,
    Numbered,
    Task,
    Quote,
    Link,
    Image,
    Table,
    Rule,
}

/// A change the editor decides on, from a key or a button.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditCommand {
    /// Enter: continues a table or a list. `plain` types the newline as it
    /// is (Shift-Enter).
    Enter {
        plain: bool,
    },
    /// Tab (Shift-Tab, `out`): moves a table cell or a list item, else
    /// indents.
    Tab {
        out: bool,
    },
    Format {
        format: FormatKind,
    },
    /// The heading button: no heading, H1, H2, H3, none.
    CycleHeading,
    /// `[[` and `]]` around the selection.
    WikiLink,
    /// Puts `text` in place of `range` (UTF-16 units): a completion chosen.
    Insert {
        range: Range16,
        text: String,
    },
}

/// Why a save is or isn't done.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveState {
    /// Everything typed is in the library, as revision `rev`, at `at_ms`.
    Saved { rev: String, at_ms: i64 },
    /// Typing waits for its turn to be saved.
    Pending,
    /// The note was changed (or deleted) elsewhere while it had unsaved
    /// typing; nothing is saved until the person chooses.
    Conflict { deleted: bool },
    /// The last save failed; `message` says why.
    Failed { message: String },
}

/// A note open for editing, as of one version of its text.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorSnapshot {
    pub id: String,
    pub title: String,
    /// The visible text: the note without its front matter.
    pub text: String,
    pub rev: String,
    pub lock: LockState,
    pub properties: Vec<Property>,
    pub version: u64,
    pub history: HistoryState,
    pub save: SaveState,
}

/// Something that happened to an open note that the shell didn't ask for.
#[derive(Clone, Debug, PartialEq)]
pub enum EditorEvent {
    /// Typing was saved.
    Saved { rev: String, at_ms: i64 },
    /// The note was changed elsewhere while it had unsaved typing: the
    /// person chooses which version to keep. `deleted` when it was deleted.
    Incoming { deleted: bool },
    /// The note was changed elsewhere and had no unsaved typing: this is
    /// its text now, and the history starts over.
    Reloaded { snapshot: EditorSnapshot },
    /// The session is over: the note was locked, deleted, or discarded.
    /// Nothing of its text is kept.
    Closed,
    /// A save failed.
    Failed { message: String },
}

/// A run of the visible text in a theme's colors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorRun {
    /// `[start, end)` in UTF-16 units of the visible text.
    pub range: Range16,
    pub color: Option<u32>,
    pub italic: Option<bool>,
    pub weight: Option<u16>,
    /// Code, code blocks, addresses, HTML and front matter: where a spell
    /// checker's complaints are not shown.
    pub quiet: bool,
}

/// The runs of a range of the text, as of one version of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunBatch {
    pub version: u64,
    pub runs: Vec<EditorRun>,
}

/// What a completion completes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionKind {
    /// `[[` and a title.
    Wiki,
    /// `#` and a tag.
    Tag,
}

/// One choice of a completion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    /// A note's notebook, as a path (`Projects / Kanso`).
    pub detail: Option<String>,
    /// The part of the label to highlight, in UTF-16 units of the label.
    pub highlight: Option<Range16>,
    /// What replaces [`Completion::replace`] when it is chosen.
    pub insert: String,
    /// The choice that writes a link to a note that doesn't exist yet.
    pub new_note: bool,
}

/// What the caret is completing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    pub kind: CompletionKind,
    /// The units of the text a choice replaces.
    pub replace: Range16,
    pub items: Vec<CompletionItem>,
}

/// Where a link in the text leads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkTarget {
    Note {
        id: String,
    },
    /// A link to a title no note has.
    NewNote {
        title: String,
    },
    Web {
        url: String,
    },
}
