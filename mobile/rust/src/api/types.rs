//! The facade's data types, declared again for the generator.
//!
//! Each `_Name` is the shape of `kanso_mobile::Name`, field by field; the
//! generated code builds and takes apart the real type with these fields, so
//! a change to the facade that these don't follow doesn't compile. Nothing
//! else is in this file: what the fields mean is said where the facade
//! declares them (`crates/kanso-mobile/src/model.rs`, `errors.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    AccountDetails, AccountState, AccountStep, ApprovalPhase, ApprovalPrompt, ApprovalWait,
    AttachmentFile, BindingAnswer, BindingQuestion, BuiltinTheme, CaptureOutcome, Changed,
    ColorChoice, ColorRow, CommentPlacement, CommentRow, Completion, CompletionItem,
    CompletionKind, CopiedLink, CursorMark, DateField, DeviceRow, DiagramKind, DiagramRenderer,
    DropsSaved, DueTone, EditCommand, EditOutcome, EditorContext, EditorEvent, EditorRun,
    EditorSnapshot, ExportedPage, FoldState, FormatKind, GalleryColors, GalleryItem, GalleryKind,
    HistoryState, Hit, IconChoice, IconThemeRow, ImportEvent, ImportFound, ImportPhase,
    ImportProgress, ImportSource, ImportSummary, InvitationRow, JoinRow, KansoError,
    LibraryOverview, LinkTarget, LinkedProviderRow, ListPlace, ListQuery, ListShape, LiveStatus,
    LocalTime, LockEvent, LockReason, LockState, LockStatus, LockedInfo, MemberRow, MentionSpan,
    NewNote, NoteChange, NoteList, NoteRow, NoteView, NotebookRole, NotebookRow, OpenedDocuments,
    OpenedFile, PaletteDto, PasskeyFailure, PasskeyRequest, PasskeyRequestKind, PasskeyRow,
    PersonLookup, PersonRow, PreviewLink, PreviewOptions, PromptState, Property, PropertyKind,
    ProviderEvent, ProviderKind, PushEnvironmentKind, PushPlatformKind, QuotaLimit, Range16,
    RecoveryKeyText, Replacement, RunBatch, SaveState, Saved, SearchScope, ServerInfo,
    ServerMethods, Share, ShareFile, ShareInfo, ShareLinkRow, ShareRole, SharedEvent,
    SignOutReasonKind, SignedUp, SortKey, SpanRow, StatusChoice, StatusCount, StyledRun, SvgRow,
    SyncAccount, SyncIntervalOption, SyncPhase, SyncProblemKind, SyncReportRow, SyncSettingsRow,
    SyncStatus, SyncStatusLine, SyncTone, SyntaxStyleRow, Table, TableRow, TagPlaces, TagRow,
    TagTreeRow, TaskBoard, TaskGroupKind, TaskItem, TaskListQuery, TaskSection, TextChange,
    ThemeTokens, ThemedIcon, Ticked, VisibleNotebook,
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

#[frb(mirror(AccountDetails))]
pub struct _AccountDetails {
    pub username: String,
    pub server: String,
    pub devices: Vec<DeviceRow>,
    pub storage_line: Option<String>,
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

#[frb(mirror(CompletionKind))]
pub enum _CompletionKind {
    Wiki,
    Tag,
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
    Task {
        index: u32,
    },
    CopyCode {
        index: u32,
    },
    Due,
    Nothing,
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
