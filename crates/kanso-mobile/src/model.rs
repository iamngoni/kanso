//! The plain data the facade returns and takes: rows, views, queries and
//! statuses. No logic lives here; the modules that build them say what each
//! field holds.
//!
//! Every type is `Clone + Debug + PartialEq` with public fields, so the
//! bridge can mirror them field by field. Counts are `u32`, times are
//! milliseconds since the Unix epoch (`i64`), colors are `0xRRGGBB` (`u32`),
//! and text offsets are UTF-16 code units, the unit Flutter counts in.

use crate::sync_model::{QuotaLimit, SignOutReasonKind, SyncAccount};

/// A note's own word-goal setting: use the library default, turn the goal
/// off, or set a note-specific word count.
pub use kanso_engine::word_goal::GoalSetting;

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
    /// Notes with this status (`active`, `onHold`, ...), as the desktop's
    /// status places list them. A locked note is never listed here: its
    /// status is its content.
    Status {
        status: String,
    },
    /// Notes with a tag at or under this level of the nested tag tree
    /// (`work` lists `work` and `work/acme`). A locked note is never listed
    /// here, as for [`ListPlace::Tag`].
    TagPath {
        path: String,
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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LibraryOverview {
    /// Notes outside the trash.
    pub all: u32,
    pub pinned: u32,
    /// Notes changed in the last seven days.
    pub recent: u32,
    pub trash: u32,
    /// Notes with each status, for Active, On hold, Completed and Dropped
    /// in that order (a note with no status is in none of them).
    pub by_status: Vec<StatusCount>,
}

/// How many notes have a status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusCount {
    /// `active`, `onHold`, `completed` or `dropped`.
    pub status: String,
    pub count: u32,
}

/// A status a note can be set to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusChoice {
    /// The value `NoteChange::SetStatus` and `ListPlace::Status` take:
    /// `none`, `active`, `onHold`, `completed` or `dropped`.
    pub status: String,
    /// What the desktop calls it: "None", "Active", "On hold", ...
    pub label: String,
}

/// A tag color the desktop offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorChoice {
    /// The name `Library::set_tag_color` takes (`red`, `default`, ...).
    pub name: String,
    /// `0xRRGGBB`.
    pub rgb: u32,
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
    /// Sets the status (`none`, `active`, `onHold`, `completed`, `dropped`).
    /// A locked note is left out: its status is its content.
    SetStatus { status: String },
    /// Adds the tag (its ID) to each note that hasn't it. A locked note is
    /// left out.
    AddTag { tag_id: String },
    /// Takes the tag (its ID) off each note that has it. A locked note is
    /// left out.
    RemoveTag { tag_id: String },
    /// Moves notes that are in the trash into the first notebook, as the
    /// desktop's Restore does. Notes outside the trash are left alone.
    Restore,
    /// Deletes notes in the trash permanently. It can't be undone.
    DeleteForever,
}

/// A link to copy: the text for the clipboard and what to tell the person.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopiedLink {
    /// A wiki-link (`[[Title]]`) or Kanso's own link by ID.
    pub text: String,
    /// "Link copied." or why a Kanso link was copied instead.
    pub message: String,
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

/// What a change to several notes did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Changed {
    /// How many notes were written (none when they were already as asked).
    pub count: u32,
    /// The ticket that takes the change back (`Library::undo`); none when
    /// nothing was written.
    pub undo: Option<u64>,
    /// What the notice says, like the desktop's ("Moved 3 notes to the
    /// trash."); empty when nothing was written.
    pub message: String,
}

/// What ticking (or clearing) a task did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ticked {
    /// The note was written with revision `rev`; `undo` is the ticket that
    /// takes the tick back (`Library::undo`); `message` is what the notice
    /// says, like the desktop's ("Ticked off “Call Sam”").
    Changed {
        rev: String,
        undo: u64,
        message: String,
    },
    /// The task already was that way.
    Unchanged,
    /// The task isn't in the note any more: its text changed since it was
    /// read.
    Missing,
}

/// What the Tasks place lists.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskListQuery {
    /// Only tasks in this notebook's notes; `None` is every notebook.
    pub notebook_id: Option<String>,
    /// Done tasks too, in Completed.
    pub show_done: bool,
    /// Keys ([`TaskItem::key`]) of tasks ticked here since the place opened:
    /// they stay in the group they were in, struck through, rather than
    /// vanish under the finger, until the place is opened again.
    pub kept: Vec<String>,
}

/// Where a task belongs, relative to now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskGroupKind {
    /// Not done, and its day (or its time today) has passed.
    Overdue,
    /// Not done and due today, at or after now.
    Today,
    /// Not done and due after today.
    Upcoming,
    /// Not done and without a date.
    NoDate,
    /// Done.
    Done,
}

/// How a due chip is drawn: the theme's color is the shell's choice by tone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DueTone {
    /// Upcoming: quiet.
    Plain,
    /// Due today.
    Today,
    /// Past, and the task isn't done: a warm red.
    Overdue,
    /// The task is done: muted.
    Done,
}

/// A task of the Tasks place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskItem {
    /// Identifies the task for [`TaskListQuery::kept`] and for lists.
    pub key: String,
    pub note_id: String,
    pub note_title: String,
    pub notebook_id: String,
    /// The notebook's name; empty for a note in none.
    pub notebook_name: String,
    /// The line (0-based) it is on in the note's text.
    pub line: u32,
    /// Its words as a list shows them: the due date taken out.
    pub text: String,
    /// Its words as written, due date included: what `Library::set_task`
    /// is handed.
    pub source_text: String,
    pub checked: bool,
    /// The due chip's words ("Tomorrow", "Overdue · Yesterday",
    /// "Fri 25 Oct 09:00"); `None` for a task without a date.
    pub due_label: Option<String>,
    /// The chip's tone; `None` for a task without a date.
    pub due_tone: Option<DueTone>,
}

/// A group of the Tasks place, with its tasks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskSection {
    pub group: TaskGroupKind,
    /// "Overdue", "Today", "Upcoming", "No date", "Completed".
    pub label: String,
    pub items: Vec<TaskItem>,
}

/// The Tasks place: the groups that have tasks, in the desktop's order
/// (Overdue, Today, Upcoming, No date, Completed).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskBoard {
    pub groups: Vec<TaskSection>,
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
    /// Another notebook is inside it (the next row is deeper).
    pub has_children: bool,
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
    /// The app went to the background.
    Background,
}

/// Where the lock session stands, for the screens that offer the lock.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LockStatus {
    /// Some note or notebook is locked (here or on another device), so a lock
    /// password exists and a note can be locked with it.
    pub has_lock_password: bool,
    /// The lock password has opened the locked notes.
    pub unlocked: bool,
    /// Biometrics can open the locked notes on this phone.
    pub biometrics_on: bool,
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
    /// The title of the first note the last sync's conflicts were in, when
    /// that note is still here.
    pub conflicted_title: Option<String>,
    /// When a pause ends (`Paused`, `ServerBusy`, `BackOff`).
    pub resume_at_ms: Option<i64>,
    /// How many items were skipped (`SomeItemsSkipped`).
    pub skipped_count: u32,
    /// What is full (`AccountFull`).
    pub quota: Option<QuotaLimit>,
    /// Why the device was signed out: the problem's, or the session's that
    /// ended.
    pub signed_out_reason: Option<SignOutReasonKind>,
    /// Where the account stands: signed out, signed in, waiting to be
    /// unlocked or signed out by the server.
    pub account: SyncAccount,
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

/// The push service that issued this phone's token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PushPlatformKind {
    /// Apple's push service (iOS).
    Apns,
    /// Firebase Cloud Messaging (Android).
    Fcm,
}

/// Which environment issued an APNs token: a build signed for development
/// gets development tokens. FCM tokens are always `Production`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PushEnvironmentKind {
    Production,
    Development,
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

/// An icon the phone draws, in an icon theme.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemedIcon {
    /// The icon's name in the Lucide set, as the app names it
    /// (`circle-question-mark`).
    pub name: String,
    /// The theme's SVG for it; `None` where the theme leaves the icon out
    /// and Kanso's own Lucide icon is drawn.
    pub svg: Option<String>,
}

/// A built-in icon theme: who made it and a few of its icons.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconThemeRow {
    /// The theme's folder name: `lucide`, `phosphor`.
    pub id: String,
    pub name: String,
    /// Empty when the theme names none.
    pub author: String,
    /// An SPDX licence identifier; empty when the theme names none.
    pub license: String,
    pub source: Option<String>,
    pub description: Option<String>,
    /// The icons that show what the theme looks like, in a fixed order.
    pub preview: Vec<ThemedIcon>,
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

// MARK: Import

/// One place notes can be imported from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportSource {
    /// The name the engine takes (`Library::import`'s `kind`).
    pub kind: String,
    pub label: String,
    /// What to pick, in a sentence.
    pub hint: String,
    /// Whether the source can be a file, or a folder.
    pub picks_files: bool,
    pub picks_folders: bool,
    /// The file extensions a file picker offers, lowercase, no dot.
    pub extensions: Vec<String>,
}

/// The stage an import is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportPhase {
    /// Reading every note once to learn titles and paths.
    Reading,
    Notebooks,
    Notes,
}

/// Where an import is.
#[derive(Clone, Debug, PartialEq)]
pub struct ImportProgress {
    /// `None` before the import has said (it is starting).
    pub phase: Option<ImportPhase>,
    /// What the desktop's bar says, e.g. "Notes and attachments… 3 of 12".
    pub label: String,
    pub done: u64,
    /// 0 while not known yet.
    pub total: u64,
    /// 0 to 1; 0 while the total is not known.
    pub fraction: f64,
}

/// What a source holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImportFound {
    pub notes: u64,
    pub notebooks: u64,
    pub tags: u64,
    pub attachments: u64,
}

/// What a plan or an import came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportSummary {
    /// The source kind it ran for.
    pub kind: String,
    /// True for a plan: nothing was written.
    pub dry_run: bool,
    pub found: ImportFound,
    /// Nothing to import: the source holds no notes.
    pub empty: bool,
    /// The desktop's summary lines.
    pub lines: Vec<String>,
    /// What was left out and why, a line per reason.
    pub skipped: Vec<String>,
    /// Empty when everything was imported, else a header and the first few.
    pub rejected: Vec<String>,
    /// The first note imported, to open it.
    pub first_note: Option<String>,
}

/// What an import tells the shell: progress, then one of the other two.
#[derive(Clone, Debug, PartialEq)]
pub enum ImportEvent {
    Progress(ImportProgress),
    Done(ImportSummary),
    /// The import didn't run; a sentence for a person.
    Failed(String),
}

/// A file the system handed over to open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenedFile {
    /// Where the file can be read now.
    pub path: String,
    /// Its name as the person knows it.
    pub name: String,
    /// The platform's own name for the document (an Android content URI)
    /// when `path` is a temporary copy; `None` when `path` is the real,
    /// stable file.
    pub identity: Option<String>,
}

/// What opening files came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenedDocuments {
    /// The last note opened, to show.
    pub note_id: Option<String>,
    /// What went wrong with the others, a sentence each.
    pub problems: Vec<String>,
}

/// What saving the shared items waiting in the inbox folder did
/// ([`Library::save_drops`](crate::Library::save_drops)).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DropsSaved {
    /// The IDs of the notes made, oldest item first. A note already made by
    /// an earlier try is here too.
    pub notes: Vec<String>,
    /// What was left out or refused, a sentence each: a file that couldn't
    /// be attached, an item discarded because the Inbox is locked.
    pub problems: Vec<String>,
    /// Items kept to try again, because saving them failed for a reason that
    /// may pass.
    pub kept: u32,
}

/// A local date and time of day, as the phone's clock shows it, to the
/// second. The library never reads a clock for a calendar day: the app
/// passes the day it means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalTime {
    pub year: i32,
    /// 1 to 12.
    pub month: u32,
    /// 1 to 31.
    pub day: u32,
    /// 0 to 23.
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

/// A calendar day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DayDate {
    pub year: i32,
    /// 1 to 12.
    pub month: u32,
    /// 1 to 31.
    pub day: u32,
}

/// A note found or made, for the app to open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MadeNote {
    pub id: String,
    /// Where a template put `{{cursor}}`, as a UTF-16 offset into the text.
    pub cursor16: Option<u32>,
    /// Whether the keyboard goes to the title first: the note has none yet.
    pub title_first: bool,
}

/// A day in the Journal's month calendar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarDay {
    pub date: DayDate,
    /// In the month itself, not a neighbour's.
    pub in_month: bool,
    /// The day has a daily note outside the trash.
    pub has_note: bool,
    pub is_today: bool,
}

/// A month in whole weeks, from the week's first day.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonthGrid {
    pub year: i32,
    /// 1 to 12.
    pub month: u32,
    /// "October 2026".
    pub title: String,
    /// The weekdays' short names, in the weeks' order: "Mon" first, or "Sun".
    pub weekday_labels: Vec<String>,
    /// Four to six weeks of seven days.
    pub weeks: Vec<Vec<CalendarDay>>,
}

/// A note in the Templates notebook.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateRow {
    pub id: String,
    pub title: String,
}

/// What a board lays out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardKind {
    /// Notes, in a column per status.
    Notes,
    /// Tasks, in To do and Done.
    Tasks,
}

/// A board's column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoardLane {
    /// A note status's column: `none`, `active`, `onHold`, `completed` or
    /// `dropped`, as [`NoteRow::status`] says it.
    Status {
        status: String,
    },
    Todo,
    Done,
}

/// Which card: a note, or a task in its note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardKey {
    Note {
        id: String,
    },
    /// The task as the note's text has it: its line and its words, so a note
    /// that changed since the board was read isn't written to blindly.
    Task {
        note_id: String,
        line: u32,
        text: String,
    },
}

/// A card and what it shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardCard {
    pub key: CardKey,
    /// A note's title, or a task's words as a list shows them.
    pub title: String,
    /// The note it opens.
    pub note_title: String,
    /// A few lines of a note's text; empty for a task and for a locked note.
    pub snippet: String,
    /// A task's due date, `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`.
    pub due: Option<String>,
    /// The chip its due date reads as; `None` for a card without a date.
    pub due_chip: Option<DueChipRow>,
    /// Whether a task is ticked.
    pub checked: bool,
    /// A locked note: its title and its lock, nothing of its text.
    pub locked: bool,
}

/// A board's column with its cards.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardColumn {
    pub lane: BoardLane,
    /// "Active", "To do".
    pub label: String,
    pub cards: Vec<BoardCard>,
}

/// A board: its columns, left to right.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardView {
    pub kind: BoardKind,
    pub columns: Vec<BoardColumn>,
}

/// What moving a card did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardMoved {
    Changed,
    Unchanged,
    Missing,
}

/// A due date's chip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DueChipRow {
    /// "Tomorrow", "Fri 25 Oct 09:00", "Overdue · Yesterday".
    pub label: String,
    pub tone: DueTone,
    /// "Due Sunday, 25 October 2026 at 09:00".
    pub tooltip: String,
}

/// A reminder to show: what the desktop's notification says, and which task
/// it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReminderNote {
    /// The task's reminder key; what an answer names.
    pub key: String,
    pub note_id: String,
    /// The task's line (0-based) in its note.
    pub line: u32,
    /// The task's words as a list shows them.
    pub title: String,
    /// When it is due and in which note: "Today 09:00 · Plans".
    pub body: String,
    /// When it falls due, in local time.
    pub fire_at: LocalTime,
}

/// The reminders to hand the system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReminderPlan {
    pub show_now: Vec<ReminderNote>,
    pub schedule: Vec<ReminderNote>,
}

/// What the person did with a reminder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReminderAnswer {
    Open,
    MarkDone,
    Dismiss,
}

/// A task to open: its note and its line (0-based).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskPlace {
    pub note_id: String,
    pub line: u32,
}

/// Something shared from inside the app (photos, the camera, a PDF), to be
/// saved as a note in the Inbox ([`Library::capture`](crate::Library::capture)).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Share {
    /// A title for the note, if there is one.
    pub subject: Option<String>,
    pub text: Option<String>,
    /// A web address.
    pub url: Option<String>,
    pub files: Vec<ShareFile>,
}

/// A file in a [`Share`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShareFile {
    /// The file's name, as the person knows it.
    pub name: String,
    /// What the app that made the file says it is; only a hint, the file's
    /// extension and first bytes decide.
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

/// What saving a [`Share`] made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureOutcome {
    /// The note's ID.
    pub note_id: String,
    /// What was left out and why, a sentence each.
    pub problems: Vec<String>,
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
    /// A heading of level 1 to 6. The accessory bar has no key for these;
    /// the hardware keyboard's shortcuts do.
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    CodeBlock,
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
    /// The selection in the visible text, in UTF-16 units: where the caret
    /// is now, after a merge moved the text around it.
    pub selection: Range16,
    pub history: HistoryState,
    pub save: SaveState,
    /// The note's own goal setting. `Default` means the library setting
    /// applies.
    pub word_goal: GoalSetting,
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

/// How the preview page is set, beyond the palette.
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewOptions {
    /// The body face, when it isn't the system's.
    pub font: Option<String>,
    /// The monospaced face, when it isn't the system's.
    pub mono: Option<String>,
    /// The body text size, in points (16 is the preview's).
    pub size: f32,
    /// How far local time is ahead of UTC, in seconds: due chips say
    /// "Today" and "Overdue" by the phone's day.
    pub utc_offset_secs: i32,
    /// How code blocks are colored: the theme's syntax styles (the
    /// `syntax` of [`ThemeTokens`]). Empty leaves code uncolored.
    pub code: Vec<SyntaxStyleRow>,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        PreviewOptions {
            font: None,
            mono: None,
            size: 16.,
            utc_offset_secs: 0,
            code: Vec::new(),
        }
    }
}

/// A note as a page to print or keep, and the name a file of it gets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportedPage {
    /// The whole page: one self-contained HTML file.
    pub html: String,
    /// The file name it is written as, from the note's title.
    pub file_name: String,
}

/// What a tapped address in the preview is. Anything the page doesn't emit,
/// and anything unsafe, is [`PreviewLink::Nothing`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreviewLink {
    /// A note that exists and isn't in the trash (a locked one is a valid
    /// target: it opens to its lock). `heading` is the heading named after
    /// `#` in a wiki-link; `anchor` is that heading's `id` in the note's
    /// page, when the note can be read now and has it.
    Note {
        id: String,
        heading: Option<String>,
        anchor: Option<String>,
    },
    /// A wiki-link to a note that doesn't exist yet: its title.
    NewNote { title: String },
    /// An inline `#tag`: its name (`work/clients`).
    Tag { name: String },
    /// A place in the same page: the heading's `id`.
    Heading { id: String },
    /// A web or mail address to hand to the system (`http`, `https`,
    /// `mailto` only).
    Web { url: String },
    /// An attachment, to open or share.
    Attachment {
        id: String,
        name: String,
        content_type: String,
        size: u64,
    },
    /// A task box: its place in the note's task list.
    Task { index: u32 },
    /// A code block's copy button: its place among the note's code blocks.
    CopyCode { index: u32 },
    /// A due-date chip.
    Due,
    /// Nothing to do.
    Nothing,
}

/// An attachment's name, type and bytes, for a viewer or the share sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentFile {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

/// What draws Mermaid diagrams (Settings › Appearance › Diagrams on the
/// desktop).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DiagramRenderer {
    /// Pele, Obsidian's renderer: the default.
    #[default]
    Pele,
    /// Kanso's own.
    Kanso,
}

/// A notebook icon the picker offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconChoice {
    /// What the notebook keeps: the Lucide icon's name.
    pub name: String,
    /// What the picker's tooltip says.
    pub label: String,
}

/// Which notebooks the sidebar shows open, in what the phone keeps between
/// runs (it mirrors the engine's `BookFolds`). Start from the default.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FoldState {
    /// Notebooks opened with their arrows, by ID.
    pub open: Vec<String>,
    /// The notebook that was showing when notebooks around it were folded.
    pub folded_at: Option<String>,
    /// Those folded notebooks; they open around another notebook again.
    pub folded: Vec<String>,
}

/// A notebook row that shows in the sidebar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibleNotebook {
    pub row: NotebookRow,
    /// Notebooks inside it show (always false when it is not a `parent`).
    pub open: bool,
    /// Another notebook is inside it.
    pub parent: bool,
}

/// One row of the nested tag tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagTreeRow {
    /// The full path, as `ListPlace::TagPath` takes it.
    pub path: String,
    /// The last level, which the row shows.
    pub label: String,
    /// The tag with exactly this name, if there is one; a leaf lists as
    /// `ListPlace::Tag { id }`, a level as `ListPlace::TagPath { path }`.
    pub tag_id: Option<String>,
    /// Notes with this tag or any tag inside it, each counted once.
    pub count: u32,
    pub depth: u32,
    /// Other levels are inside it.
    pub parent: bool,
    /// Its levels show (always false when it is not a `parent`).
    pub open: bool,
}

/// The sidebar's tags: chips for tags that aren't nested, and a tree for
/// those that are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagPlaces {
    /// The chips shown, in name order: all of them, or the most used few.
    pub chips: Vec<TagRow>,
    /// There are more chips than the few shown by default ("…").
    pub more: bool,
    /// The tree's rows that show, top to bottom.
    pub tree: Vec<TagTreeRow>,
}

/// Where a keyboard shortcut acts: the desktop's key contexts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutPlace {
    Global,
    NoteList,
    Editor,
    FocusMode,
    NoteWindow,
    Palette,
    TasksView,
    BoardView,
    CalendarView,
    GraphView,
    Settings,
    SettingsNav,
    Capture,
}

/// The menu a keyboard shortcut is listed under in the overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutSection {
    App,
    File,
    Edit,
    Format,
    View,
    Go,
    Note,
    Window,
    Tabs,
    List,
    Palette,
    Calendar,
    Tasks,
    Board,
    Graph,
    Settings,
    Capture,
}

/// One key of the desktop's shortcut catalog, for a phone or tablet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutEntry {
    /// The desktop action's name in snake case. An action with two keys, or
    /// in two places, has an entry for each.
    pub id: String,
    /// The keys on macOS and iPadOS, in GPUI's keystroke syntax
    /// (`secondary-shift-t`).
    pub mac: String,
    /// The keys on Windows, Linux and Android; empty when none are bound.
    pub other: String,
    pub place: ShortcutPlace,
    /// What the desktop's menu calls the action.
    pub label: String,
    pub section: ShortcutSection,
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
