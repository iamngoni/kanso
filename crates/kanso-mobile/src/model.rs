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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewNote {
    /// The notebook it goes in; `None` is the first notebook (made if there
    /// is none). A locked notebook takes it sealed, and needs the lock open.
    pub notebook_id: Option<String>,
    pub title: String,
    pub body: String,
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

// MARK: Discovery

/// Where related notes stand (`kanso_engine::discovery::related::Status`).
#[derive(Clone, Debug, Default, PartialEq)]
pub enum RelatedStatus {
    #[default]
    Off,
    /// Fetching the model: bytes so far, of all.
    Downloading { done: u64, total: u64 },
    /// Checking and loading the model.
    Loading,
    /// Embedding notes: how many are done, of all.
    Indexing { done: u32, total: u32 },
    /// Every note is embedded.
    Ready { notes: u32 },
    /// Something went wrong; turning related notes on again starts over.
    Failed { message: String },
}

/// Related notes' status, with the words the desktop's Settings tells it in.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RelatedState {
    pub status: RelatedStatus,
    /// The status row's title and line; `None` while it is off.
    pub title: Option<String>,
    pub detail: Option<String>,
}

/// A note close in meaning to another, or to a search.
#[derive(Clone, Debug, PartialEq)]
pub struct RelatedRow {
    pub id: String,
    pub title: String,
    /// The cosine similarity, from -1 to 1.
    pub score: f32,
    /// How close, in words: "Very close", "Close", "Related" or "Loosely
    /// related".
    pub label: String,
}

/// A tag the note's nearest notes have and it hasn't.
#[derive(Clone, Debug, PartialEq)]
pub struct SuggestedTag {
    pub tag_id: String,
    pub name: String,
    /// The similarities of the near notes that have it, added up.
    pub weight: f32,
}

/// A note that says nearly the same as another.
#[derive(Clone, Debug, PartialEq)]
pub struct TwinRow {
    pub id: String,
    pub title: String,
    /// How alike the two are, from 0 to 100.
    pub percent: u32,
}

/// What the stored embeddings say about a note.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NoteMeaning {
    /// At most five, heaviest first, without the note's own tags.
    pub tags: Vec<SuggestedTag>,
    /// Unless it was dismissed this session.
    pub twin: Option<TwinRow>,
}

/// What a search by meaning found.
#[derive(Clone, Debug, PartialEq)]
pub enum MeaningResult {
    /// Related notes are off.
    Off,
    /// Related notes are on, but the model isn't loaded yet.
    NoModel,
    /// Notes close in meaning to the search, closest first; none when the
    /// search is too short or has qualifiers (`meaning_query`).
    Found { rows: Vec<RelatedRow> },
}

/// Which graph to lay out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphRequest {
    /// Every note, or the notes of one notebook (and the notebooks inside
    /// it) or tag.
    Library { filter: GraphFilter },
    /// A note and the notes up to `hops` links from it (1 to 3).
    Local { center: String, hops: u8 },
}

/// What the library's graph is narrowed to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphFilter {
    All,
    Book { id: String },
    Tag { id: String },
}

/// A note in the graph, placed.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeRow {
    pub id: String,
    pub title: String,
    pub notebook_id: String,
    /// How many other notes it links with.
    pub degree: u32,
    /// Where it goes, in layout units around the origin.
    pub x: f32,
    pub y: f32,
    /// How big, in layout units: bigger the more it links.
    pub radius: f32,
    /// Its notebook's color in the light and the dark theme, `0xAARRGGBB`
    /// and opaque.
    pub color_light: u32,
    pub color_dark: u32,
    /// The local graph's middle.
    pub is_center: bool,
}

/// A link between two notes, as indexes into the graph's nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphEdgeRow {
    pub from: u32,
    pub to: u32,
}

/// A laid-out graph.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphLayout {
    pub nodes: Vec<GraphNodeRow>,
    pub edges: Vec<GraphEdgeRow>,
    /// How many links it draws, a note linking to itself left out.
    pub link_count: u32,
}

/// An image waiting to be read for its words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageToRead {
    pub id: String,
    /// Give it back with the words, so new bytes are read again.
    pub md5: String,
    pub bytes: Vec<u8>,
}

/// What the embedding model costs on this phone.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscoveryMeasure {
    /// The model's files on disk, in bytes.
    pub model_bytes: u64,
    /// Checking and loading the model, in milliseconds.
    pub load_ms: u64,
    /// The app's resident memory before and after loading, in bytes, when
    /// the phone says.
    pub resident_before: Option<u64>,
    pub resident_after: Option<u64>,
    /// Sample notes embedded, and how many a second.
    pub notes: u32,
    pub notes_per_second: f64,
}
