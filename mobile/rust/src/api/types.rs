//! The facade's data types, declared again for the generator.
//!
//! Each `_Name` is the shape of `kanso_mobile::Name`, field by field; the
//! generated code builds and takes apart the real type with these fields, so
//! a change to the facade that these don't follow doesn't compile. Nothing
//! else is in this file: what the fields mean is said where the facade
//! declares them (`crates/kanso-mobile/src/model.rs`, `errors.rs`).

use flutter_rust_bridge::frb;
pub use kanso_mobile::WhenHint;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    AccountDetails, AccountState, AccountStep, AiSummary, ApprovalPhase, ApprovalPrompt,
    ApprovalWait, AttachmentFile, BindingAnswer, BindingQuestion, BoardCard, BoardColumn,
    BoardKind, BoardLane, BoardSetup, BoardView, BriefNote, BriefPrefs, BriefReminder, BriefView,
    BuiltinTheme, CalendarDay, CaptureOutcome, CardKey, CardMoved, Changed, ColorChoice, ColorRow,
    CommentPlacement, CommentRow, Completion, CompletionItem, CompletionKind, CopiedLink,
    CostSummary, CursorMark, DateChoice, DateChoices, DateField, DayDate, DeviceRow, DiagramKind,
    DiagramRenderer, DiscoveryMeasure, DropsSaved, DueChipRow, DueTone, EditCommand, EditOutcome,
    EditorContext, EditorEvent, EditorRun, EditorSnapshot, ExportedPage, FilesMigration, FoldState,
    FormatKind, GalleryColors, GalleryItem, GalleryKind, GoalSetting, GraphEdgeRow, GraphFilter,
    GraphLayout, GraphNodeRow, GraphRequest, HistoryState, Hit, IconChoice, IconThemeRow,
    ImageToRead, ImportEvent, ImportFound, ImportPhase, ImportProgress, ImportSource,
    ImportSummary, InterruptedLock, InvitationRow, JoinRow, KansoError, LaneChange, LaneEdited,
    LaneRemoval, LibraryOverview, LinkTarget, LinkedProviderRow, ListPlace, ListQuery, ListShape,
    LiveBatch, LiveDocBlock, LiveDocBlockKind, LiveDocLink, LiveDocLinkKind, LiveDocMarker,
    LiveDocMarkerKind, LiveDocStyle, LiveDocStyleKind, LiveDocWidget, LiveDocument, LiveKind,
    LiveLine, LiveRun, LiveStatus, LocalTime, LockCopyNotice, LockEvent, LockReason, LockState,
    LockStatus, LockSummary, LockedInfo, LoopAction, LoopProposal, LoopScan, MadeNote,
    MeaningResult, MemberRow, MentionSpan, MigrationOutcome, MonthGrid, NewNote, NoteChange,
    NoteList, NoteMeaning, NoteRow, NoteView, NotebookRole, NotebookRow, OnThisDayRow,
    OpenLoopItem, OpenLoopKind, OpenLoopsBoard, OpenedDocuments, OpenedFile, PairingCode,
    PaletteDto, PasskeyFailure, PasskeyRequest, PasskeyRequestKind, PasskeyRow, PdfHighlightChange,
    PdfHighlightRow, PdfInkRow, PdfNotes, PersonLookup, PersonRow, PlannedNotice, PreviewAlign,
    PreviewBlock, PreviewCanvasNotice, PreviewCanvasShown, PreviewCell, PreviewCodeRun,
    PreviewFigureKind, PreviewInline, PreviewLink, PreviewListItem, PreviewOptions,
    PreviewQueryRow, PreviewRow, PreviewSourcedBlock, PreviewSvg, PreviewTaskBox, PromptState,
    Property, PropertyKind, ProviderEvent, ProviderKind, PushEnvironmentKind, PushPlatformKind,
    QuotaLimit, Range16, RecoveryKeyText, RelatedRow, RelatedState, RelatedStatus, ReminderAnswer,
    ReminderNote, ReminderPlan, Replacement, ResurfaceResult, ResurfaceRow, ReviewFinished,
    ReviewSaved, ReviewTheme, ReviewView, ReviewWritten, RunBatch, SaveState, Saved, SealPhase,
    SealProgress, SearchScope, ServerInfo, ServerMethods, Share, ShareFile, ShareInfo,
    ShareLinkRow, ShareRole, SharedEvent, ShortcutEntry, ShortcutPlace, ShortcutSection,
    SignOutReasonKind, SignedUp, SortKey, SpanRow, StatusChoice, StatusCount, StyledRun,
    SuggestedTag, SvgRow, SyncAccount, SyncIntervalOption, SyncPhase, SyncProblemKind,
    SyncReportRow, SyncSettingsRow, SyncStatus, SyncStatusLine, SyncTone, SyntaxStyleRow, Table,
    TableRow, TagPlaces, TagRow, TagTreeRow, TaskBoard, TaskGroupKind, TaskItem, TaskListQuery,
    TaskPlace, TaskSection, TemplateRow, TextChange, ThemeTokens, ThemedIcon, Ticked, TwinRow,
    VisibleNotebook,
};
pub use kanso_mobile::{
    IconThemeImport, InstalledIconThemeRow, ThemeImport, ThemeNotice, ThemeOrigin, ThemeRow,
};

#[frb(mirror(KansoError))]
pub enum _KansoError {
    NotFound,
    Locked,
    Conflict { current_rev: String },
    Invalid { message: String },
    Storage { message: String },
    Network { message: String },
    SignedOut,
    Auth { message: String },
    Internal { message: String },
}

#[frb(mirror(GoalSetting))]
pub enum _GoalSetting {
    Default,
    Off,
    Words(u32),
}

#[frb(mirror(NoteRow))]
pub struct _NoteRow {
    pub id: String,
    pub rev: String,
    pub title: String,
    pub snippet: String,
    pub notebook_id: String,
    pub status: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub created_ms: i64,
    pub updated_ms: i64,
    pub tasks_done: u32,
    pub tasks_total: u32,
    pub locked: bool,
    pub canvas: bool,
}

#[frb(mirror(LockState))]
pub enum _LockState {
    Plain,
    Open,
    Closed { reason: Option<String> },
}

#[frb(mirror(NoteView))]
pub struct _NoteView {
    pub id: String,
    pub rev: String,
    pub title: String,
    pub body: Option<String>,
    pub lock: LockState,
    pub notebook_id: String,
    pub tags: Vec<String>,
    pub status: String,
    pub pinned: bool,
    pub created_ms: i64,
    pub updated_ms: i64,
}

#[frb(mirror(ListPlace))]
pub enum _ListPlace {
    All,
    Pinned,
    Recent,
    Canvases,
    Notebook { id: String },
    Tag { id: String },
    Status { status: String },
    TagPath { path: String },
    Trash,
}

#[frb(mirror(ListQuery))]
pub struct _ListQuery {
    pub place: ListPlace,
    pub status_only: Option<String>,
    pub pinned_only: bool,
    pub sort: SortKey,
    pub descending: bool,
}

#[frb(mirror(DateField))]
pub enum _DateField {
    Updated,
    Created,
}

#[frb(mirror(ListShape))]
pub struct _ListShape {
    pub sort: SortKey,
    pub by_date: bool,
    pub date: DateField,
    pub group_pinned: bool,
}

#[frb(mirror(NoteList))]
pub struct _NoteList {
    pub rows: Vec<NoteRow>,
    pub shape: ListShape,
}

#[frb(mirror(LibraryOverview))]
pub struct _LibraryOverview {
    pub all: u32,
    pub canvases: u32,
    pub pinned: u32,
    pub recent: u32,
    pub trash: u32,
    pub by_status: Vec<StatusCount>,
}

#[frb(mirror(StatusCount))]
pub struct _StatusCount {
    pub status: String,
    pub count: u32,
}

#[frb(mirror(TaskListQuery))]
pub struct _TaskListQuery {
    pub notebook_id: Option<String>,
    pub show_done: bool,
    pub kept: Vec<String>,
}

#[frb(mirror(TaskGroupKind))]
pub enum _TaskGroupKind {
    Overdue,
    Today,
    Upcoming,
    NoDate,
    Done,
}

#[frb(mirror(DueTone))]
pub enum _DueTone {
    Plain,
    Today,
    Overdue,
    Done,
}

#[frb(mirror(TaskItem))]
pub struct _TaskItem {
    pub key: String,
    pub note_id: String,
    pub note_title: String,
    pub notebook_id: String,
    pub notebook_name: String,
    pub line: u32,
    pub text: String,
    pub source_text: String,
    pub checked: bool,
    pub due_label: Option<String>,
    pub due_tone: Option<DueTone>,
}

#[frb(mirror(TaskSection))]
pub struct _TaskSection {
    pub group: TaskGroupKind,
    pub label: String,
    pub items: Vec<TaskItem>,
}

#[frb(mirror(TaskBoard))]
pub struct _TaskBoard {
    pub groups: Vec<TaskSection>,
}

#[frb(mirror(Ticked))]
pub enum _Ticked {
    Changed {
        rev: String,
        undo: u64,
        message: String,
    },
    Unchanged,
    Missing,
}

#[frb(mirror(NoteChange))]
pub enum _NoteChange {
    TogglePin,
    Move { notebook_id: String },
    Trash,
    SetStatus { status: String },
    AddTag { tag_id: String },
    RemoveTag { tag_id: String },
    Restore,
    DeleteForever,
}

#[frb(mirror(Changed))]
pub struct _Changed {
    pub count: u32,
    pub undo: Option<u64>,
    pub message: String,
}

#[frb(mirror(SearchScope))]
pub enum _SearchScope {
    All,
    Titles,
    Tags,
    Notebook { id: String },
}

#[frb(mirror(NotebookRole))]
pub enum _NotebookRole {
    Inbox,
    Journal,
    Templates,
}

#[frb(mirror(SortKey))]
pub enum _SortKey {
    Updated,
    Created,
    Title,
}

#[frb(mirror(NewNote))]
pub struct _NewNote {
    pub notebook_id: Option<String>,
    pub title: String,
    pub body: String,
    pub place: Option<ListPlace>,
    pub preferred_notebook_id: Option<String>,
    #[frb(default = "const []")]
    pub tags: Vec<String>,
    #[frb(default = false)]
    pub pinned: bool,
}

#[frb(mirror(PdfNotes))]
pub struct _PdfNotes {
    pub file: String,
    pub name: String,
    pub url: String,
    pub editable: bool,
    pub why: Option<String>,
    pub highlights: Vec<PdfHighlightRow>,
    pub scanned_pages: u32,
    pub ink: Vec<PdfInkRow>,
}

#[frb(mirror(PdfInkRow))]
pub struct _PdfInkRow {
    pub page: u32,
    pub strokes: u32,
}

#[frb(mirror(PdfHighlightRow))]
pub struct _PdfHighlightRow {
    pub id: String,
    pub page: u32,
    pub color: String,
    pub text: String,
    pub comment: Option<String>,
    pub in_note: bool,
}

#[frb(mirror(PdfHighlightChange))]
pub enum _PdfHighlightChange {
    Comment { comment: String },
    Color { color: String },
    AddQuote,
    Delete,
}

#[frb(mirror(Saved))]
pub struct _Saved {
    pub id: String,
    pub rev: String,
}

#[frb(mirror(NotebookRow))]
pub struct _NotebookRow {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub count: u32,
    pub locked: bool,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub role: Option<NotebookRole>,
    pub depth: u32,
    pub has_children: bool,
}

#[frb(mirror(TagRow))]
pub struct _TagRow {
    pub id: String,
    pub name: String,
    pub color: u32,
    pub count: u32,
}

#[frb(mirror(StatusChoice))]
pub struct _StatusChoice {
    pub status: String,
    pub label: String,
}

#[frb(mirror(ColorChoice))]
pub struct _ColorChoice {
    pub name: String,
    pub rgb: u32,
}

#[frb(mirror(IconChoice))]
pub struct _IconChoice {
    pub name: String,
    pub label: String,
}

#[frb(mirror(FoldState))]
pub struct _FoldState {
    pub open: Vec<String>,
    pub folded_at: Option<String>,
    pub folded: Vec<String>,
}

#[frb(mirror(VisibleNotebook))]
pub struct _VisibleNotebook {
    pub row: NotebookRow,
    pub open: bool,
    pub parent: bool,
}

#[frb(mirror(TagTreeRow))]
pub struct _TagTreeRow {
    pub path: String,
    pub label: String,
    pub tag_id: Option<String>,
    pub count: u32,
    pub depth: u32,
    pub parent: bool,
    pub open: bool,
}

#[frb(mirror(TagPlaces))]
pub struct _TagPlaces {
    pub chips: Vec<TagRow>,
    pub more: bool,
    pub tree: Vec<TagTreeRow>,
}

#[frb(mirror(CopiedLink))]
pub struct _CopiedLink {
    pub text: String,
    pub message: String,
}

#[frb(mirror(Hit))]
pub struct _Hit {
    pub row: NoteRow,
    pub title_ranges: Vec<Range16>,
    pub snippet_ranges: Vec<Range16>,
}

#[frb(mirror(Range16))]
pub struct _Range16 {
    pub start: u32,
    pub end: u32,
}

#[frb(mirror(PropertyKind))]
pub enum _PropertyKind {
    Text,
    Number,
    Bool,
    Date,
    List,
    Raw,
}

#[frb(mirror(Property))]
pub struct _Property {
    pub key: String,
    pub kind: PropertyKind,
    pub text: String,
    pub items: Vec<String>,
}

#[frb(mirror(Table))]
pub struct _Table {
    pub columns: Vec<String>,
    pub rows: Vec<TableRow>,
}

#[frb(mirror(TableRow))]
pub struct _TableRow {
    pub id: String,
    pub title: String,
    pub cells: Vec<Option<String>>,
}

#[frb(mirror(LockReason))]
pub enum _LockReason {
    Manual,
    Idle,
    Background,
}

#[frb(mirror(LockStatus))]
pub struct _LockStatus {
    pub has_lock_password: bool,
    pub unlocked: bool,
    pub biometrics_on: bool,
}

#[frb(mirror(SealProgress))]
pub struct _SealProgress {
    pub phase: SealPhase,
    pub done: u32,
    pub total: u32,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

#[frb(mirror(SealPhase))]
pub enum _SealPhase {
    Preparing,
    Sealing,
    Locking,
}

#[frb(mirror(LockCopyNotice))]
pub struct _LockCopyNotice {
    pub removed: bool,
    pub copied: u32,
}

#[frb(mirror(LockSummary))]
pub struct _LockSummary {
    pub files: u32,
    pub bytes: u64,
    pub shared: u32,
}

#[frb(mirror(FilesMigration))]
pub struct _FilesMigration {
    pub notes: u32,
    pub stopped: u32,
    pub postponed: bool,
    pub counted: bool,
    pub files: u32,
    pub bytes: u64,
    pub shared: u32,
}

#[frb(mirror(MigrationOutcome))]
pub struct _MigrationOutcome {
    pub notes: u32,
    pub sealed: u32,
    pub copied: u32,
    pub unopened: u32,
    pub failed: u32,
    pub stopped: bool,
}

#[frb(mirror(InterruptedLock))]
pub struct _InterruptedLock {
    pub files: u32,
    pub sealed: u32,
}

#[frb(mirror(LockEvent))]
pub enum _LockEvent {
    Unlocked,
    Locked { reason: LockReason },
}

#[frb(mirror(SyncPhase))]
pub enum _SyncPhase {
    SignedOut,
    Idle,
    Syncing,
    Synced,
    Failed,
}

#[frb(mirror(PushPlatformKind))]
pub enum _PushPlatformKind {
    Apns,
    Fcm,
}

#[frb(mirror(PushEnvironmentKind))]
pub enum _PushEnvironmentKind {
    Production,
    Development,
}

#[frb(mirror(SyncProblemKind))]
pub enum _SyncProblemKind {
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
    /// Files too large for the server stay on this device
    /// (`skipped_count` says how many). Last, so earlier values keep
    /// their index.
    FilesTooLarge,
}

#[frb(mirror(SyncStatus))]
pub struct _SyncStatus {
    pub phase: SyncPhase,
    pub username: Option<String>,
    pub server: Option<String>,
    pub last_synced_ms: Option<i64>,
    pub conflicts: u32,
    pub problem: Option<SyncProblemKind>,
    pub conflicted_title: Option<String>,
    pub resume_at_ms: Option<i64>,
    pub skipped_count: u32,
    pub quota: Option<QuotaLimit>,
    pub signed_out_reason: Option<SignOutReasonKind>,
    pub account: SyncAccount,
}

#[frb(mirror(SyncTone))]
pub enum _SyncTone {
    Good,
    Busy,
    Quiet,
    Warning,
}

#[frb(mirror(SyncStatusLine))]
pub struct _SyncStatusLine {
    pub tone: SyncTone,
    pub label: String,
    pub detail: String,
}

#[frb(mirror(SignOutReasonKind))]
pub enum _SignOutReasonKind {
    SignedOut,
    Revoked,
    PasswordChanged,
    Recovered,
    Evicted,
    Expired,
    Rekeyed,
    Unknown,
}

#[frb(mirror(QuotaLimit))]
pub enum _QuotaLimit {
    Docs,
    DocBytes,
    Files,
    FileBytes,
    Server,
}

#[frb(mirror(AccountStep))]
pub enum _AccountStep {
    StoppingSync,
    DerivingKeys,
    Contacting,
    Saving,
}

#[frb(mirror(SyncAccount))]
pub enum _SyncAccount {
    SignedOut,
    SignedIn,
    Locked,
    SessionEnded,
    Busy(AccountStep),
}

#[frb(mirror(SyncSettingsRow))]
pub struct _SyncSettingsRow {
    pub enabled: bool,
    pub interval_secs: u32,
}

#[frb(mirror(SyncIntervalOption))]
pub struct _SyncIntervalOption {
    pub minutes: u32,
    pub label: String,
}

#[frb(mirror(CommentPlacement))]
pub enum _CommentPlacement {
    Anchored { start: u32, end: u32 },
    Orphaned { at: u32 },
}

#[frb(mirror(MentionSpan))]
pub struct _MentionSpan {
    pub start: u32,
    pub end: u32,
}

#[frb(mirror(CommentRow))]
pub struct _CommentRow {
    pub id: String,
    pub author: String,
    pub author_name: String,
    pub body: String,
    pub created_ms: i64,
    pub edited_ms: Option<i64>,
    pub resolved: bool,
    pub mine: bool,
    pub pending: bool,
    pub can_edit: bool,
    pub can_delete: bool,
    pub can_resolve: bool,
    pub quote: String,
    pub placement: CommentPlacement,
    pub mentions: Vec<MentionSpan>,
}

#[frb(mirror(SyncReportRow))]
pub struct _SyncReportRow {
    pub pulled: u32,
    pub pushed: u32,
    pub conflicts: u32,
    pub pending: u32,
    pub queued: bool,
}

#[frb(mirror(ServerInfo))]
pub struct _ServerInfo {
    pub signups_open: bool,
}

#[frb(mirror(SpanRow))]
pub struct _SpanRow {
    pub range: Range16,
    pub tokens: Vec<String>,
}

#[frb(mirror(PaletteDto))]
pub struct _PaletteDto {
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

#[frb(mirror(ColorRow))]
pub struct _ColorRow {
    pub name: String,
    pub rgb: u32,
}

#[frb(mirror(SyntaxStyleRow))]
pub struct _SyntaxStyleRow {
    pub token: String,
    pub color: Option<u32>,
    pub italic: Option<bool>,
    pub weight: Option<u16>,
}

#[frb(mirror(ThemeTokens))]
pub struct _ThemeTokens {
    pub name: String,
    pub dark: bool,
    pub ui: Vec<ColorRow>,
    pub caret: u32,
    pub selection: u32,
    pub selection_alpha: u8,
    pub syntax: Vec<SyntaxStyleRow>,
    pub palette: PaletteDto,
}

#[frb(mirror(ExportedPage))]
pub struct _ExportedPage {
    pub html: String,
    pub file_name: String,
}

#[frb(mirror(BuiltinTheme))]
pub struct _BuiltinTheme {
    pub id: String,
    pub name: String,
    pub author: String,
    pub license: String,
    pub source: Option<String>,
    pub dark: bool,
    pub tokens: ThemeTokens,
}

#[frb(mirror(ThemeOrigin))]
pub enum _ThemeOrigin {
    Builtin,
    Own,
    Gallery,
}

#[frb(mirror(ThemeRow))]
pub struct _ThemeRow {
    pub id: String,
    pub name: String,
    pub author: String,
    pub license: String,
    pub source: Option<String>,
    pub dark: bool,
    pub origin: ThemeOrigin,
    pub tokens: ThemeTokens,
}

#[frb(mirror(ThemeNotice))]
pub struct _ThemeNotice {
    pub file: String,
    pub sentence: String,
}

#[frb(mirror(ThemeImport))]
pub struct _ThemeImport {
    pub ids: Vec<String>,
    pub names: Vec<String>,
}

#[frb(mirror(ThemedIcon))]
pub struct _ThemedIcon {
    pub name: String,
    pub svg: Option<String>,
}

#[frb(mirror(IconThemeRow))]
pub struct _IconThemeRow {
    pub id: String,
    pub name: String,
    pub author: String,
    pub license: String,
    pub source: Option<String>,
    pub description: Option<String>,
    pub icon_count: u32,
    pub preview: Vec<ThemedIcon>,
}

#[frb(mirror(InstalledIconThemeRow))]
pub struct _InstalledIconThemeRow {
    pub id: String,
    pub name: String,
    pub author: String,
    pub license: String,
    pub origin: ThemeOrigin,
    pub icon_count: u32,
}

#[frb(mirror(IconThemeImport))]
pub struct _IconThemeImport {
    pub ids: Vec<String>,
    pub names: Vec<String>,
    pub summary: String,
}

#[frb(mirror(StyledRun))]
pub struct _StyledRun {
    pub range: Range16,
    pub color: Option<u32>,
    pub italic: Option<bool>,
    pub weight: Option<u16>,
}

#[frb(mirror(DiagramKind))]
pub enum _DiagramKind {
    Mermaid,
    Excalidraw,
}

#[frb(mirror(SvgRow))]
pub struct _SvgRow {
    pub markup: String,
    pub width: f32,
    pub height: f32,
    pub baseline: Option<f32>,
}

#[frb(mirror(ImportSource))]
pub struct _ImportSource {
    pub kind: String,
    pub label: String,
    pub hint: String,
    pub picks_files: bool,
    pub picks_folders: bool,
    pub extensions: Vec<String>,
}

#[frb(mirror(ImportPhase))]
pub enum _ImportPhase {
    Reading,
    Notebooks,
    Notes,
}

#[frb(mirror(ImportProgress))]
pub struct _ImportProgress {
    pub phase: Option<ImportPhase>,
    pub label: String,
    pub done: u64,
    pub total: u64,
    pub fraction: f64,
}

#[frb(mirror(ImportFound))]
pub struct _ImportFound {
    pub notes: u64,
    pub notebooks: u64,
    pub tags: u64,
    pub attachments: u64,
}

#[frb(mirror(ImportSummary))]
pub struct _ImportSummary {
    pub kind: String,
    pub dry_run: bool,
    pub found: ImportFound,
    pub empty: bool,
    pub lines: Vec<String>,
    pub skipped: Vec<String>,
    pub rejected: Vec<String>,
    pub first_note: Option<String>,
}

#[frb(mirror(ImportEvent))]
pub enum _ImportEvent {
    Progress(ImportProgress),
    Done(ImportSummary),
    Failed(String),
}

#[frb(mirror(OpenedFile))]
pub struct _OpenedFile {
    pub path: String,
    pub name: String,
    pub identity: Option<String>,
}

#[frb(mirror(OpenedDocuments))]
pub struct _OpenedDocuments {
    pub note_id: Option<String>,
    pub problems: Vec<String>,
}

// A note's history, Replace in all notes and the backups.
pub use kanso_mobile::{
    BackupFile, DiffHunk, DiffKind, DiffLine, NoteHistory, NoteVersion, PlannedNote, ReplaceMatch,
    ReplaceNote, ReplaceOptions, ReplaceOutcome, ReplacePreview, ReplaceUndone,
};

#[frb(mirror(NoteHistory))]
pub struct _NoteHistory {
    pub current_title: String,
    pub current_body: String,
    pub current_authors: Option<String>,
    pub versions: Vec<NoteVersion>,
}

#[frb(mirror(NoteVersion))]
pub struct _NoteVersion {
    pub rev: String,
    pub saved_at: i64,
    pub origin: String,
    pub origin_label: String,
    pub authors: Option<String>,
    pub agent: Option<String>,
    pub title: String,
    pub body: String,
}

#[frb(mirror(DiffKind))]
pub enum _DiffKind {
    Same,
    Added,
    Removed,
}

#[frb(mirror(DiffLine))]
pub struct _DiffLine {
    pub kind: DiffKind,
    pub text: String,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub changed: Vec<Range16>,
}

#[frb(mirror(DiffHunk))]
pub struct _DiffHunk {
    pub lines: Vec<DiffLine>,
    pub old_text: String,
    pub added: u32,
    pub removed: u32,
    pub old_start: Option<u32>,
    pub old_end: Option<u32>,
}

#[frb(mirror(ReplaceOptions))]
pub struct _ReplaceOptions {
    pub find: String,
    pub match_case: bool,
    pub whole_word: bool,
    pub regex: bool,
    pub skip_code: bool,
    pub include_trash: bool,
}

#[frb(mirror(ReplacePreview))]
pub struct _ReplacePreview {
    pub error: Option<String>,
    pub total_matches: u32,
    pub notes: Vec<ReplaceNote>,
}

#[frb(mirror(ReplaceNote))]
pub struct _ReplaceNote {
    pub id: String,
    pub title: String,
    pub rev: String,
    pub trashed: bool,
    pub count: u32,
    pub matches: Vec<ReplaceMatch>,
}

#[frb(mirror(ReplaceMatch))]
pub struct _ReplaceMatch {
    pub line: String,
    pub start_utf16: u32,
    pub end_utf16: u32,
    pub line_number: u32,
}

#[frb(mirror(PlannedNote))]
pub struct _PlannedNote {
    pub id: String,
    pub rev: String,
}

#[frb(mirror(ReplaceOutcome))]
pub struct _ReplaceOutcome {
    pub batch: String,
    pub replaced: u32,
    pub notes: u32,
    pub skipped: u32,
    pub message: String,
}

#[frb(mirror(ReplaceUndone))]
pub struct _ReplaceUndone {
    pub restored: u32,
    pub skipped: u32,
    pub message: String,
}

#[frb(mirror(BackupFile))]
pub struct _BackupFile {
    pub path: String,
    pub made_at_ms: i64,
    pub manual: bool,
    pub bytes: u64,
}

#[frb(mirror(ProviderKind))]
pub enum _ProviderKind {
    Apple,
    Google,
    Github,
}

#[frb(mirror(ServerMethods))]
pub struct _ServerMethods {
    pub signups_open: bool,
    pub password: bool,
    pub passkeys: bool,
    pub providers: Vec<ProviderKind>,
}

#[frb(mirror(AccountState))]
pub enum _AccountState {
    SignedOut,
    SignedIn { username: String, server: String },
    Locked(LockedInfo),
    Question(BindingQuestion),
}

#[frb(mirror(LockedInfo))]
pub struct _LockedInfo {
    pub username: String,
    pub server: String,
    pub has_password: bool,
    pub approval_pending: bool,
}

#[frb(mirror(BindingQuestion))]
pub struct _BindingQuestion {
    pub title: String,
    pub body: String,
    pub answers: Vec<BindingAnswer>,
}

#[frb(mirror(BindingAnswer))]
pub enum _BindingAnswer {
    Add,
    Replace,
}

#[frb(mirror(RecoveryKeyText))]
pub struct _RecoveryKeyText {
    pub key: String,
}

#[frb(mirror(SignedUp))]
pub struct _SignedUp {
    pub state: AccountState,
    pub recovery_key: RecoveryKeyText,
}

#[frb(mirror(ProviderEvent))]
pub enum _ProviderEvent {
    OpenPage {
        url: String,
        callback_scheme: Option<String>,
    },
    ShowCode {
        user_code: String,
        verification_uri: String,
    },
    ClosePage,
    NeedsUsername {
        suggested: String,
    },
    Finished(AccountState),
    Failed {
        message: String,
    },
    Cancelled,
}

#[frb(mirror(PasskeyRequestKind))]
pub enum _PasskeyRequestKind {
    Register,
    Assert,
}

#[frb(mirror(PasskeyRequest))]
pub struct _PasskeyRequest {
    pub ticket: u64,
    pub kind: PasskeyRequestKind,
    pub options_json: String,
}

#[frb(mirror(PasskeyFailure))]
pub enum _PasskeyFailure {
    Cancelled,
    Unavailable,
    Failed,
}

#[frb(mirror(ApprovalWait))]
pub struct _ApprovalWait {
    pub phase: ApprovalPhase,
    pub code: Option<String>,
    pub other_devices: u32,
    pub offer_recovery: bool,
}

#[frb(mirror(ApprovalPhase))]
pub enum _ApprovalPhase {
    Requesting,
    Waiting,
    Compare,
    Unlocking,
    Approved,
    NoDevices,
    Cancelled,
    Expired,
    Failed { message: String },
}

#[frb(mirror(DeviceRow))]
pub struct _DeviceRow {
    pub id: String,
    pub title: String,
    pub this_device: bool,
    pub last_seen_ms: Option<i64>,
}

#[frb(mirror(PairingCode))]
pub struct _PairingCode {
    pub id: String,
    pub code: String,
    pub expires_at_ms: i64,
}

#[frb(mirror(LinkedProviderRow))]
pub struct _LinkedProviderRow {
    pub provider: ProviderKind,
    pub linked: bool,
}

#[frb(mirror(PasskeyRow))]
pub struct _PasskeyRow {
    pub id: String,
    pub label: String,
    pub added_ms: i64,
    pub last_used_ms: Option<i64>,
    pub can_unlock: bool,
}

#[frb(mirror(CostSummary))]
pub struct _CostSummary {
    pub headline: String,
    pub space_line: String,
    pub learn_more_url: String,
}

#[frb(mirror(AccountDetails))]
pub struct _AccountDetails {
    pub username: String,
    pub server: String,
    pub devices: Vec<DeviceRow>,
    pub storage_line: Option<String>,
    pub cost: CostSummary,
    pub has_password: bool,
    pub providers: Vec<LinkedProviderRow>,
    pub passkeys: Vec<PasskeyRow>,
    pub recovery_key_unsaved: bool,
}

#[frb(mirror(PromptState))]
pub enum _PromptState {
    New,
    Challenged,
    Ready,
    Changed,
}

#[frb(mirror(ApprovalPrompt))]
pub struct _ApprovalPrompt {
    pub id: String,
    pub device_name: Option<String>,
    pub asked_at_ms: i64,
    pub expires_at_ms: i64,
    pub code: Option<String>,
    pub state: PromptState,
}

#[frb(mirror(ShareRole))]
pub enum _ShareRole {
    Owner,
    Editor,
    Commenter,
    Viewer,
}

#[frb(mirror(MemberRow))]
pub struct _MemberRow {
    pub account_id: String,
    pub username: String,
    pub role: ShareRole,
    pub me: bool,
    pub fingerprint: String,
    pub key_changed: bool,
    pub invited: bool,
    pub left: bool,
    pub verified: bool,
}

#[frb(mirror(ShareLinkRow))]
pub struct _ShareLinkRow {
    pub id: String,
    pub role: ShareRole,
    pub expires_ms: i64,
}

#[frb(mirror(ShareInfo))]
pub struct _ShareInfo {
    pub share_id: String,
    pub note_id: String,
    pub my_role: ShareRole,
    pub owner: String,
    pub members: Vec<MemberRow>,
    pub links: Vec<ShareLinkRow>,
    pub requests: Vec<MemberRow>,
}

#[frb(mirror(InvitationRow))]
pub struct _InvitationRow {
    pub share_id: String,
    pub title: Option<String>,
    pub from: String,
    pub role: ShareRole,
    pub at_ms: i64,
}

#[frb(mirror(SharedEvent))]
pub struct _SharedEvent {
    pub note_ids: Vec<String>,
    pub invitations: u32,
    pub comment_notes: Vec<String>,
    pub live_notes: Vec<String>,
}

#[frb(mirror(LiveStatus))]
pub struct _LiveStatus {
    pub role: ShareRole,
    pub connected: bool,
    pub ended: bool,
    pub near_limit: bool,
    pub problems: Vec<String>,
}

#[frb(mirror(CursorMark))]
pub struct _CursorMark {
    pub name: String,
    pub caret: u32,
    pub start: u32,
    pub end: u32,
    pub light: u32,
    pub dark: u32,
}

#[frb(mirror(PersonRow))]
pub struct _PersonRow {
    pub account_id: String,
    pub username: String,
    pub fingerprint: String,
    pub verified: bool,
}

#[frb(mirror(PersonLookup))]
pub enum _PersonLookup {
    Found(PersonRow),
    Changed {
        pinned: Option<PersonRow>,
        offered: PersonRow,
    },
}

#[frb(mirror(JoinRow))]
pub struct _JoinRow {
    pub share_id: String,
    pub owner: String,
    pub role: ShareRole,
}

#[frb(mirror(GalleryKind))]
pub enum _GalleryKind {
    ColorTheme,
    IconTheme,
}

#[frb(mirror(GalleryColors))]
pub struct _GalleryColors {
    pub editor: u32,
    pub text: u32,
    pub accent: u32,
    pub sidebar: u32,
    pub list: u32,
}

#[frb(mirror(GalleryItem))]
pub struct _GalleryItem {
    pub id: String,
    pub name: String,
    pub kind: GalleryKind,
    pub author: String,
    pub official: bool,
    pub version: String,
    pub installs: u64,
    pub updated_ms: i64,
    pub source: Option<String>,
    pub dark: bool,
    pub headline: Option<GalleryColors>,
    pub installed: bool,
    pub update_available: bool,
    pub runs_here: bool,
    pub description: String,
    pub license: String,
    pub family: Option<String>,
    pub edited: bool,
    pub disabled: bool,
}

#[frb(mirror(TextChange))]
pub struct _TextChange {
    pub base_version: u64,
    pub start: u32,
    pub end: u32,
    pub text: String,
    pub selection: Range16,
    pub composing: Option<Range16>,
}

#[frb(mirror(Replacement))]
pub struct _Replacement {
    pub start: u32,
    pub end: u32,
    pub text: String,
}

#[frb(mirror(HistoryState))]
pub struct _HistoryState {
    pub can_undo: bool,
    pub can_redo: bool,
}

#[frb(mirror(EditOutcome))]
pub struct _EditOutcome {
    pub version: u64,
    pub replace: Option<Replacement>,
    pub selection: Range16,
    pub history: HistoryState,
}

#[frb(mirror(EditorContext))]
pub struct _EditorContext {
    pub in_list: bool,
    pub in_table: bool,
    pub in_code: bool,
    pub completion_here: bool,
    pub history: HistoryState,
}

#[frb(mirror(FormatKind))]
pub enum _FormatKind {
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
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    CodeBlock,
}

#[frb(mirror(EditCommand))]
pub enum _EditCommand {
    Enter { plain: bool },
    Tab { out: bool },
    Format { format: FormatKind },
    CycleHeading,
    WikiLink,
    Insert { range: Range16, text: String },
}

#[frb(mirror(SaveState))]
pub enum _SaveState {
    Saved { rev: String, at_ms: i64 },
    Pending,
    Conflict { deleted: bool },
    Failed { message: String },
}

#[frb(mirror(EditorSnapshot))]
pub struct _EditorSnapshot {
    pub id: String,
    pub title: String,
    pub text: String,
    pub rev: String,
    pub lock: LockState,
    pub properties: Vec<Property>,
    pub version: u64,
    pub selection: Range16,
    pub history: HistoryState,
    pub save: SaveState,
    pub word_goal: GoalSetting,
}

#[frb(mirror(EditorEvent))]
pub enum _EditorEvent {
    Saved { rev: String, at_ms: i64 },
    Incoming { deleted: bool },
    Reloaded { snapshot: EditorSnapshot },
    Closed,
    Failed { message: String },
}

#[frb(mirror(EditorRun))]
pub struct _EditorRun {
    pub range: Range16,
    pub color: Option<u32>,
    pub italic: Option<bool>,
    pub weight: Option<u16>,
    pub quiet: bool,
}

#[frb(mirror(RunBatch))]
pub struct _RunBatch {
    pub version: u64,
    pub runs: Vec<EditorRun>,
}

#[frb(mirror(LiveKind))]
pub enum _LiveKind {
    Strong,
    Emphasis,
    Strikethrough,
    Code,
    Link,
    Heading,
}

#[frb(mirror(LiveRun))]
pub struct _LiveRun {
    pub range: Range16,
    pub kind: LiveKind,
}

#[frb(mirror(LiveLine))]
pub struct _LiveLine {
    pub range: Range16,
    pub level: u8,
    pub scale: f64,
}

#[frb(mirror(LiveBatch))]
pub struct _LiveBatch {
    pub version: u64,
    pub conceal: Vec<Range16>,
    pub runs: Vec<LiveRun>,
    pub lines: Vec<LiveLine>,
}

#[frb(mirror(LiveDocument))]
pub struct _LiveDocument {
    pub version: u64,
    pub conceal: Vec<Range16>,
    pub blocks: Vec<LiveDocBlock>,
    pub markers: Vec<LiveDocMarker>,
    pub styles: Vec<LiveDocStyle>,
    pub links: Vec<LiveDocLink>,
    pub widgets: Vec<LiveDocWidget>,
}

#[frb(mirror(LiveDocBlockKind))]
pub enum _LiveDocBlockKind {
    Paragraph,
    Heading,
    Rule,
    Code,
    Table,
    Math,
    Html,
    FrontMatter,
    Image,
    Definition,
    Blank,
}

#[frb(mirror(LiveDocBlock))]
pub struct _LiveDocBlock {
    pub kind: LiveDocBlockKind,
    pub level: u8,
    pub language: Option<String>,
    pub first_line: u32,
    pub end_line: u32,
    pub source: Range16,
    pub list_depth: u32,
    pub quote_depth: u32,
    pub item_start: bool,
    pub top: Option<u32>,
    pub revealed: bool,
}

#[frb(mirror(LiveDocMarkerKind))]
pub enum _LiveDocMarkerKind {
    Bullet,
    Number,
    Task,
    Hidden,
    Quote,
    Indent,
    Rule,
    Underline,
}

#[frb(mirror(LiveDocMarker))]
pub struct _LiveDocMarker {
    pub range: Range16,
    pub line: u32,
    pub kind: LiveDocMarkerKind,
    pub depth: u32,
    pub text: String,
    pub checked: bool,
}

#[frb(mirror(LiveDocStyleKind))]
pub enum _LiveDocStyleKind {
    Strong,
    Emphasis,
    Strikethrough,
    Code,
    Link,
    Heading,
    InlineMath,
    Quote,
    CardSeparator,
    TaskDone,
}

#[frb(mirror(LiveDocStyle))]
pub struct _LiveDocStyle {
    pub range: Range16,
    pub kind: LiveDocStyleKind,
    pub level: u8,
}

#[frb(mirror(LiveDocLinkKind))]
pub enum _LiveDocLinkKind {
    Url,
    Reference,
    Wiki,
}

#[frb(mirror(LiveDocLink))]
pub struct _LiveDocLink {
    pub range: Range16,
    pub text: Range16,
    pub kind: LiveDocLinkKind,
    pub target: String,
    pub heading: Option<String>,
    pub missing: bool,
}

#[frb(mirror(LiveDocWidget))]
pub struct _LiveDocWidget {
    pub kind: LiveDocBlockKind,
    pub language: Option<String>,
    pub first_line: u32,
    pub end_line: u32,
    pub lines: Range16,
    pub source: Range16,
}

#[frb(mirror(CompletionKind))]
pub enum _CompletionKind {
    Wiki,
    Tag,
    Date,
}

#[frb(mirror(CompletionItem))]
pub struct _CompletionItem {
    pub label: String,
    pub detail: Option<String>,
    pub highlight: Option<Range16>,
    pub insert: String,
    pub new_note: bool,
}

#[frb(mirror(Completion))]
pub struct _Completion {
    pub kind: CompletionKind,
    pub replace: Range16,
    pub items: Vec<CompletionItem>,
}

#[frb(mirror(LinkTarget))]
pub enum _LinkTarget {
    Note { id: String },
    NewNote { title: String },
    Web { url: String },
}

#[frb(mirror(PreviewOptions))]
pub struct _PreviewOptions {
    pub font: Option<String>,
    pub mono: Option<String>,
    pub size: f32,
    pub utc_offset_secs: i32,
    pub code: Vec<SyntaxStyleRow>,
}

#[frb(mirror(PreviewLink))]
pub enum _PreviewLink {
    Note {
        id: String,
        heading: Option<String>,
        anchor: Option<String>,
    },
    NewNote {
        title: String,
    },
    Tag {
        name: String,
    },
    Heading {
        id: String,
    },
    Web {
        url: String,
    },
    Attachment {
        id: String,
        name: String,
        content_type: String,
        size: u64,
    },
    Pdf {
        id: String,
        name: String,
        page: Option<u32>,
        highlight: Option<String>,
    },
    Task {
        index: u32,
    },
    CopyCode {
        index: u32,
    },
    Due,
    Audio {
        file_id: String,
        seconds: u32,
    },
    Nothing,
}

#[frb(mirror(PreviewBlock))]
pub enum _PreviewBlock {
    Heading {
        level: u32,
        id: String,
        inlines: Vec<PreviewInline>,
    },
    Paragraph {
        inlines: Vec<PreviewInline>,
    },
    List {
        ordered: bool,
        start: u32,
        tight: bool,
        items: Vec<PreviewListItem>,
    },
    Quote {
        blocks: Vec<PreviewBlock>,
    },
    Code {
        language: Option<String>,
        runs: Vec<PreviewCodeRun>,
        copy: u32,
    },
    Rule,
    Table {
        aligns: Vec<PreviewAlign>,
        head: Vec<PreviewCell>,
        rows: Vec<PreviewRow>,
    },
    Figure {
        kind: PreviewFigureKind,
        svg: PreviewSvg,
    },
    Sketch {
        id: String,
        width: u32,
        height: u32,
        empty: bool,
        svg: PreviewSvg,
    },
    EmptyDrawing,
    PdfNotes {
        url: Option<String>,
        summary: String,
    },
    Canvas {
        note: String,
        group: Option<String>,
        shown: PreviewCanvasShown,
    },
    Failed {
        badge: String,
        detail: String,
        language: Option<String>,
        code: String,
        copy: Option<u32>,
    },
    Query {
        columns: Vec<String>,
        rows: Vec<PreviewQueryRow>,
    },
    QueryInvalid {
        code: String,
        message: String,
    },
    Html {
        text: String,
    },
}

#[frb(mirror(PreviewInline))]
pub enum _PreviewInline {
    Text {
        text: String,
        bold: bool,
        italic: bool,
        strike: bool,
        code: bool,
    },
    Link {
        url: String,
        inlines: Vec<PreviewInline>,
    },
    Due {
        url: String,
        label: String,
        title: String,
        tone: DueTone,
    },
    Math {
        svg: PreviewSvg,
    },
    Image {
        content_type: String,
        bytes: Vec<u8>,
        alt: String,
        width: Option<u32>,
    },
    Missing {
        text: String,
    },
    Break,
}

#[frb(mirror(PreviewListItem))]
pub struct _PreviewListItem {
    pub task: Option<PreviewTaskBox>,
    pub blocks: Vec<PreviewBlock>,
}

#[frb(mirror(PreviewSourcedBlock))]
pub struct _PreviewSourcedBlock {
    pub block: PreviewBlock,
    pub first_line: Option<u32>,
    pub end_line: Option<u32>,
    pub lines: Option<Range16>,
}

#[frb(mirror(PreviewTaskBox))]
pub struct _PreviewTaskBox {
    pub checked: bool,
    pub index: Option<u32>,
}

#[frb(mirror(PreviewCodeRun))]
pub struct _PreviewCodeRun {
    pub text: String,
    pub color: Option<u32>,
    pub bold: bool,
    pub italic: bool,
}

#[frb(mirror(PreviewCell))]
pub struct _PreviewCell {
    pub inlines: Vec<PreviewInline>,
}

#[frb(mirror(PreviewRow))]
pub struct _PreviewRow {
    pub cells: Vec<PreviewCell>,
}

#[frb(mirror(PreviewSvg))]
pub struct _PreviewSvg {
    pub markup: String,
    pub width: f32,
    pub height: f32,
    pub baseline: Option<f32>,
}

#[frb(mirror(PreviewCanvasShown))]
pub enum _PreviewCanvasShown {
    Drawing {
        svg: PreviewSvg,
        notices: Vec<PreviewCanvasNotice>,
    },
    Locked,
    Missing,
    NotCanvas,
    Invalid {
        message: String,
    },
}

#[frb(mirror(PreviewCanvasNotice))]
pub enum _PreviewCanvasNotice {
    WholeCanvas,
    GroupMissing,
}

#[frb(mirror(PreviewFigureKind))]
pub enum _PreviewFigureKind {
    Math,
    Diagram,
    Drawing,
    Plugin,
}

#[frb(mirror(PreviewAlign))]
pub enum _PreviewAlign {
    None,
    Left,
    Center,
    Right,
}

#[frb(mirror(PreviewQueryRow))]
pub struct _PreviewQueryRow {
    pub url: String,
    pub title: String,
    pub cells: Vec<String>,
}

#[frb(mirror(AttachmentFile))]
pub struct _AttachmentFile {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[frb(mirror(DiagramRenderer))]
pub enum _DiagramRenderer {
    Pele,
    Kanso,
}

#[frb(mirror(DropsSaved))]
pub struct _DropsSaved {
    pub notes: Vec<String>,
    pub problems: Vec<String>,
    pub kept: u32,
}

#[frb(mirror(LocalTime))]
pub struct _LocalTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

#[frb(mirror(DayDate))]
pub struct _DayDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[frb(mirror(MadeNote))]
pub struct _MadeNote {
    pub id: String,
    pub cursor16: Option<u32>,
    pub title_first: bool,
}

#[frb(mirror(CalendarDay))]
pub struct _CalendarDay {
    pub date: DayDate,
    pub in_month: bool,
    pub has_note: bool,
    pub is_today: bool,
}

#[frb(mirror(MonthGrid))]
pub struct _MonthGrid {
    pub year: i32,
    pub month: u32,
    pub title: String,
    pub weekday_labels: Vec<String>,
    pub weeks: Vec<Vec<CalendarDay>>,
}

#[frb(mirror(TemplateRow))]
pub struct _TemplateRow {
    pub id: String,
    pub title: String,
}

#[frb(mirror(BoardKind))]
pub enum _BoardKind {
    Notes,
    Tasks,
}

#[frb(mirror(BoardLane))]
pub enum _BoardLane {
    Status { status: String },
    Todo,
    Done,
    Value { value: String },
    Unset,
    Unlisted,
}

#[frb(mirror(CardKey))]
pub enum _CardKey {
    Note {
        id: String,
    },
    Task {
        note_id: String,
        line: u32,
        text: String,
    },
}

#[frb(mirror(BoardCard))]
pub struct _BoardCard {
    pub key: CardKey,
    pub title: String,
    pub note_title: String,
    pub snippet: String,
    pub due: Option<String>,
    pub due_chip: Option<DueChipRow>,
    pub checked: bool,
    pub locked: bool,
}

#[frb(mirror(BoardColumn))]
pub struct _BoardColumn {
    pub lane: BoardLane,
    pub label: String,
    pub cards: Vec<BoardCard>,
}

#[frb(mirror(BoardView))]
pub struct _BoardView {
    pub kind: BoardKind,
    pub columns: Vec<BoardColumn>,
    pub setup: Option<BoardSetup>,
}

#[frb(mirror(BoardSetup))]
pub struct _BoardSetup {
    pub book_id: String,
    pub group: Option<String>,
    pub groupable: Vec<String>,
    pub lanes: Vec<String>,
    pub show_unset: bool,
}

#[frb(mirror(CardMoved))]
pub enum _CardMoved {
    Changed {
        message: Option<String>,
        undo: Option<u64>,
    },
    Unchanged,
    Missing,
}

#[frb(mirror(LaneChange))]
pub enum _LaneChange {
    Rename {
        from: String,
        to: String,
    },
    Move {
        from: u32,
        to: u32,
    },
    Add {
        name: String,
    },
    Remove {
        lane: String,
        move_to: Option<String>,
    },
}

#[frb(mirror(LaneEdited))]
pub struct _LaneEdited {
    pub lanes: Vec<String>,
    pub rewritten: u32,
    pub message: String,
    pub undo: u64,
}

#[frb(mirror(LaneRemoval))]
pub struct _LaneRemoval {
    pub notes: u32,
    pub prompt: String,
    pub target: Option<String>,
    pub targets: Vec<String>,
}

#[frb(mirror(DueChipRow))]
pub struct _DueChipRow {
    pub label: String,
    pub tone: DueTone,
    pub tooltip: String,
}

#[frb(mirror(ReminderNote))]
pub struct _ReminderNote {
    pub key: String,
    pub note_id: String,
    pub line: u32,
    pub title: String,
    pub body: String,
    pub fire_at: LocalTime,
    pub actions: bool,
}

#[frb(mirror(ReminderPlan))]
pub struct _ReminderPlan {
    pub show_now: Vec<ReminderNote>,
    pub schedule: Vec<ReminderNote>,
}

#[frb(mirror(ReminderAnswer))]
pub enum _ReminderAnswer {
    Open,
    MarkDone,
    Dismiss,
}

#[frb(mirror(TaskPlace))]
pub struct _TaskPlace {
    pub note_id: String,
    pub line: u32,
}

#[frb(mirror(Share))]
pub struct _Share {
    pub subject: Option<String>,
    pub text: Option<String>,
    pub url: Option<String>,
    pub files: Vec<ShareFile>,
}

#[frb(mirror(ShareFile))]
pub struct _ShareFile {
    pub name: String,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

#[frb(mirror(CaptureOutcome))]
pub struct _CaptureOutcome {
    pub note_id: String,
    pub problems: Vec<String>,
}

#[frb(mirror(ShortcutPlace))]
pub enum _ShortcutPlace {
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

#[frb(mirror(ShortcutSection))]
pub enum _ShortcutSection {
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

#[frb(mirror(ShortcutEntry))]
pub struct _ShortcutEntry {
    pub id: String,
    pub mac: String,
    pub other: String,
    pub place: ShortcutPlace,
    pub label: String,
    pub section: ShortcutSection,
}

#[frb(mirror(RelatedStatus))]
pub enum _RelatedStatus {
    Off,
    Downloading { done: u64, total: u64 },
    Loading,
    Indexing { done: u32, total: u32 },
    Ready { notes: u32 },
    Failed { message: String },
}

#[frb(mirror(RelatedState))]
pub struct _RelatedState {
    pub status: RelatedStatus,
    pub title: Option<String>,
    pub detail: Option<String>,
}

#[frb(mirror(RelatedRow))]
pub struct _RelatedRow {
    pub id: String,
    pub title: String,
    pub score: f32,
    pub label: String,
}

#[frb(mirror(SuggestedTag))]
pub struct _SuggestedTag {
    pub tag_id: String,
    pub name: String,
    pub weight: f32,
}

#[frb(mirror(TwinRow))]
pub struct _TwinRow {
    pub id: String,
    pub title: String,
    pub percent: u32,
}

#[frb(mirror(NoteMeaning))]
pub struct _NoteMeaning {
    pub tags: Vec<SuggestedTag>,
    pub twin: Option<TwinRow>,
}

#[frb(mirror(MeaningResult))]
pub enum _MeaningResult {
    Off,
    NoModel,
    Found { rows: Vec<RelatedRow> },
}

#[frb(mirror(ResurfaceRow))]
pub struct _ResurfaceRow {
    pub id: String,
    pub title: String,
    pub book: String,
    pub when: String,
    pub snippet: String,
    pub before: Option<String>,
}

#[frb(mirror(ResurfaceResult))]
pub enum _ResurfaceResult {
    Off,
    NoModel,
    Found { rows: Vec<ResurfaceRow> },
}

#[frb(mirror(OnThisDayRow))]
pub struct _OnThisDayRow {
    pub id: String,
    pub title: String,
    pub book: String,
    pub ago: String,
}

#[frb(mirror(GraphFilter))]
pub enum _GraphFilter {
    All,
    Book { id: String },
    Tag { id: String },
}

#[frb(mirror(GraphRequest))]
pub enum _GraphRequest {
    Library { filter: GraphFilter },
    Local { center: String, hops: u8 },
}

#[frb(mirror(GraphNodeRow))]
pub struct _GraphNodeRow {
    pub id: String,
    pub title: String,
    pub notebook_id: String,
    pub degree: u32,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color_light: u32,
    pub color_dark: u32,
    pub is_center: bool,
}

#[frb(mirror(GraphEdgeRow))]
pub struct _GraphEdgeRow {
    pub from: u32,
    pub to: u32,
}

#[frb(mirror(GraphLayout))]
pub struct _GraphLayout {
    pub nodes: Vec<GraphNodeRow>,
    pub edges: Vec<GraphEdgeRow>,
    pub link_count: u32,
}

#[frb(mirror(ImageToRead))]
pub struct _ImageToRead {
    pub id: String,
    pub md5: String,
    pub bytes: Vec<u8>,
}

#[frb(mirror(DiscoveryMeasure))]
pub struct _DiscoveryMeasure {
    pub model_bytes: u64,
    pub load_ms: u64,
    pub resident_before: Option<u64>,
    pub resident_after: Option<u64>,
    pub notes: u32,
    pub notes_per_second: f64,
}

#[frb(mirror(OpenLoopKind))]
pub enum _OpenLoopKind {
    Mine,
    Waiting,
    Question,
}

#[frb(mirror(LoopAction))]
pub enum _LoopAction {
    Done,
    NotALoop,
    MakeTask,
}

#[frb(mirror(OpenLoopItem))]
pub struct _OpenLoopItem {
    pub key: String,
    pub note_id: String,
    pub note_title: String,
    pub notebook_id: String,
    pub line: u32,
    pub kind: OpenLoopKind,
    pub text: String,
    pub due_label: Option<String>,
    pub age_label: String,
    pub from_ai: bool,
}

#[frb(mirror(OpenLoopsBoard))]
pub struct _OpenLoopsBoard {
    pub items: Vec<OpenLoopItem>,
    pub total: u32,
    pub enabled: bool,
}

#[frb(mirror(LoopProposal))]
pub struct _LoopProposal {
    pub note_id: String,
    pub note_title: String,
    pub line: u32,
    pub text: String,
    pub kind: OpenLoopKind,
}

#[frb(mirror(LoopScan))]
pub struct _LoopScan {
    pub proposals: Vec<LoopProposal>,
    pub coverage: Option<String>,
    pub dropped_text: Option<String>,
}

#[frb(mirror(AiSummary))]
pub struct _AiSummary {
    pub text: String,
    pub warning: Option<String>,
}

#[frb(mirror(BriefReminder))]
pub struct _BriefReminder {
    pub note_id: String,
    pub note_title: String,
    pub line: u32,
    pub text: String,
    pub time_label: String,
}

#[frb(mirror(BriefNote))]
pub struct _BriefNote {
    pub note_id: String,
    pub title: String,
    pub why: String,
    pub excerpt: String,
}

#[frb(mirror(BriefView))]
pub struct _BriefView {
    pub date_label: String,
    pub due: Vec<TaskItem>,
    pub loops: Vec<OpenLoopItem>,
    pub loops_total: u32,
    pub later_today: Vec<BriefReminder>,
    pub relevant: Vec<BriefNote>,
    pub wrote_yesterday: u32,
    pub finished_yesterday: u32,
    pub is_empty: bool,
}

#[frb(mirror(ReviewWritten))]
pub struct _ReviewWritten {
    pub note_id: String,
    pub title: String,
    pub notebook_name: String,
    pub created: bool,
}

#[frb(mirror(ReviewFinished))]
pub struct _ReviewFinished {
    pub note_id: String,
    pub note_title: String,
    pub text: String,
    pub was_loop: bool,
}

#[frb(mirror(ReviewTheme))]
pub struct _ReviewTheme {
    pub term: String,
    pub notes: u32,
    pub is_tag: bool,
}

#[frb(mirror(ReviewView))]
pub struct _ReviewView {
    pub from: String,
    pub week_label: String,
    pub range_label: String,
    pub counts_text: String,
    pub written: Vec<ReviewWritten>,
    pub finished: Vec<ReviewFinished>,
    pub still_open: Vec<OpenLoopItem>,
    pub overdue: Vec<TaskItem>,
    pub themes: Vec<ReviewTheme>,
    pub next: Vec<TaskItem>,
    pub saved_note_id: Option<String>,
    pub is_this_week: bool,
}

#[frb(mirror(ReviewSaved))]
pub struct _ReviewSaved {
    pub note_id: String,
    pub exists: bool,
}

#[frb(mirror(BriefPrefs))]
pub struct _BriefPrefs {
    pub loops: bool,
    pub brief: bool,
    pub brief_hour: u32,
    pub brief_minute: u32,
    pub brief_weekdays_only: bool,
    pub review: bool,
    pub review_day: u32,
    pub review_hour: u32,
    pub review_minute: u32,
    pub notify_brief: bool,
    pub notify_review: bool,
    pub mark_done_in_note: bool,
    pub open_to_overview: bool,
    pub quiet_days: u32,
}

#[frb(mirror(PlannedNotice))]
pub struct _PlannedNotice {
    pub id: String,
    pub is_review: bool,
    pub title: String,
    pub body: String,
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
}

#[frb(mirror(WhenHint))]
pub struct _WhenHint {
    pub start: u32,
    pub end: u32,
    pub label: String,
    pub canonical: String,
    pub repeats: bool,
}

#[frb(mirror(DateChoice))]
pub struct _DateChoice {
    pub label: String,
    pub detail: Option<String>,
    pub insert: String,
}

#[frb(mirror(DateChoices))]
pub struct _DateChoices {
    pub start: u32,
    pub choices: Vec<DateChoice>,
}
