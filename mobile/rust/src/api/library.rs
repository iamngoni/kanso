//! A library of notes: lists and counts, reading, writing, changing several
//! notes, notebooks, search, properties and query views, and the stream of
//! changes.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{
    AttachmentFile, CaptureOutcome, Changed, DropsSaved, Hit, ImportEvent, KansoError, Library,
    LibraryOverview, ListQuery, LocalTime, NewNote, NoteChange, NoteList, NoteView, NotebookRow,
    OpenedDocuments, OpenedFile, Property, Saved, SearchScope, Share, Table, TagRow, TaskBoard,
    TaskListQuery, Ticked,
};
// A note's history, Replace in all notes and the backups.
use kanso_mobile::{
    BackupFile, DiffHunk, NoteHistory, PlannedNote, ReplaceOptions, ReplaceOutcome, ReplacePreview,
    ReplaceUndone,
};

use super::account::KansoAccount;
use super::account_settings::KansoAccountSettings;
use super::approvals::KansoApprovals;
use super::discovery::KansoDiscovery;
use super::editor::KansoEditor;
use super::gallery::KansoGallery;
use super::linking::KansoLinking;
use super::locks::KansoLocks;
use super::renderer::KansoRenderer;
use super::shared::KansoShared;
use super::sync::KansoSync;

/// An open library. Cheap to clone on the Rust side; Dart holds one.
#[frb(opaque)]
pub struct KansoLibrary(Library);

impl KansoLibrary {
    /// Opens (creating if needed) the library whose database file is `path`.
    pub fn open(path: String) -> Result<KansoLibrary, KansoError> {
        Library::open(std::path::Path::new(&path)).map(KansoLibrary)
    }

    pub fn list(&self, query: ListQuery) -> Result<NoteList, KansoError> {
        self.0.list(query)
    }

    pub fn overview(&self) -> Result<LibraryOverview, KansoError> {
        self.0.overview()
    }

    /// The Tasks place: open tasks across the notes, in groups; locked
    /// notes are left out.
    pub fn tasks(&self, query: TaskListQuery) -> Result<TaskBoard, KansoError> {
        self.0.tasks(query)
    }

    /// How many open tasks are overdue (the Tasks place's badge).
    pub fn overdue_tasks(&self) -> Result<u32, KansoError> {
        self.0.overdue_tasks()
    }

    pub fn change(&self, ids: Vec<String>, change: NoteChange) -> Result<Changed, KansoError> {
        self.0.change(ids, change)
    }

    pub fn undo(&self, ticket: u64) -> Result<u32, KansoError> {
        self.0.undo(ticket)
    }

    pub fn set_task(
        &self,
        note_id: String,
        line: u32,
        text: String,
        checked: bool,
    ) -> Result<Ticked, KansoError> {
        self.0.set_task(&note_id, line, &text, checked)
    }

    pub fn note(&self, id: String) -> Result<NoteView, KansoError> {
        self.0.note(&id)
    }

    pub fn save(&self, id: String, base_rev: String, body: String) -> Result<Saved, KansoError> {
        self.0.save(&id, &base_rev, body)
    }

    pub fn create(&self, note: NewNote) -> Result<Saved, KansoError> {
        self.0.create(note)
    }

    /// Opens note `id` for editing. Locked while it is locked and closed.
    pub fn edit(&self, id: String) -> Result<KansoEditor, KansoError> {
        self.0.edit(&id).map(KansoEditor::new)
    }

    /// Ticks (or clears) the `index`-th task box of a note drawn at
    /// `base_rev`.
    pub fn toggle_task_at(
        &self,
        note_id: String,
        index: u32,
        base_rev: String,
    ) -> Result<Ticked, KansoError> {
        self.0.toggle_task_at(&note_id, index, &base_rev)
    }

    pub fn create_linked_note(
        &self,
        title: String,
        from_note: String,
    ) -> Result<Saved, KansoError> {
        self.0.create_linked_note(&title, &from_note)
    }

    pub fn attachment(&self, id: String) -> Result<AttachmentFile, KansoError> {
        self.0.attachment(&id)
    }

    pub fn trash(&self, id: String) -> Result<(), KansoError> {
        self.0.trash(&id)
    }

    pub fn notebooks(&self) -> Result<Vec<NotebookRow>, KansoError> {
        self.0.notebooks()
    }

    pub fn create_notebook(
        &self,
        name: String,
        parent_id: Option<String>,
    ) -> Result<NotebookRow, KansoError> {
        self.0.create_notebook(name, parent_id)
    }

    pub fn tags(&self) -> Result<Vec<TagRow>, KansoError> {
        self.0.tags()
    }

    pub fn search(
        &self,
        text: String,
        scope: SearchScope,
        limit: u32,
    ) -> Result<Vec<Hit>, KansoError> {
        self.0.search(&text, scope, limit)
    }

    pub fn properties(&self, id: String) -> Result<Vec<Property>, KansoError> {
        self.0.properties(&id)
    }

    pub fn query_view(&self, source: String, self_id: Option<String>) -> Result<Table, KansoError> {
        self.0.query_view(&source, self_id)
    }

    /// Saves every finished shared item waiting in the `inbox` folder as a
    /// note in the Inbox and removes the folders saved (see
    /// `mobile/docs/capture.md`).
    pub fn save_drops(&self, inbox: String) -> Result<DropsSaved, KansoError> {
        self.0.save_drops(std::path::Path::new(&inbox))
    }

    /// Saves something shared from inside the app as a note in the Inbox.
    pub fn capture(&self, share: Share) -> Result<CaptureOutcome, KansoError> {
        self.0.capture(share)
    }

    /// The ID of today's daily note at the phone's local time `now`, made
    /// if it isn't there.
    pub fn today_note(&self, now: LocalTime) -> Result<String, KansoError> {
        self.0.today_note(now)
    }

    /// The IDs of what changed, in batches, until Dart stops listening.
    pub fn changes(&self, sink: StreamSink<Vec<String>>) {
        self.0.changes(move |ids| sink.add(ids).is_ok());
    }

    /// What importing `path` as a `kind` would do (one of `import_sources`):
    /// progress, then one summary or one failure. Writes nothing. Returns at
    /// once; the work runs on a thread of its own.
    pub fn import_plan(&self, kind: String, path: String, sink: StreamSink<ImportEvent>) {
        self.0
            .import_plan(&kind, &path, move |event| sink.add(event).is_ok());
    }

    /// Imports `path` as a `kind`: progress, then one summary or one
    /// failure. Returns at once; the notes written reach `changes`.
    pub fn import(&self, kind: String, path: String, sink: StreamSink<ImportEvent>) {
        self.0
            .import(&kind, &path, move |event| sink.add(event).is_ok());
    }

    /// Opens the Markdown or text files the system handed over, each into
    /// the Inbox; the last note opened and what went wrong with the others.
    /// Copies of a document the platform names (`identity`) are staged under
    /// `staging` so opening it again updates the same note.
    pub fn open_documents(
        &self,
        files: Vec<OpenedFile>,
        staging: String,
    ) -> Result<OpenedDocuments, KansoError> {
        self.0.open_documents(files, &staging)
    }

    /// The note's saved versions, newest first, beside the note now.
    pub fn history(&self, id: String) -> Result<NoteHistory, KansoError> {
        self.0.history(&id)
    }

    /// What changed from version `rev` to the note now, section by section.
    pub fn history_diff(&self, id: String, rev: String) -> Result<Vec<DiffHunk>, KansoError> {
        self.0.history_diff(&id, &rev)
    }

    /// Sets the note's title and text to those of version `rev`.
    pub fn restore_version(
        &self,
        id: String,
        rev: String,
        base_rev: String,
    ) -> Result<Saved, KansoError> {
        self.0.restore_version(&id, &rev, &base_rev)
    }

    pub fn replace_preview(&self, options: ReplaceOptions) -> Result<ReplacePreview, KansoError> {
        self.0.replace_preview(options)
    }

    pub fn replace_apply(
        &self,
        options: ReplaceOptions,
        replacement: String,
        planned: Vec<PlannedNote>,
    ) -> Result<ReplaceOutcome, KansoError> {
        self.0.replace_apply(options, replacement, planned)
    }

    /// Undoes a replace by its batch; says what was put back.
    pub fn replace_undo(&self, batch: String) -> Result<ReplaceUndone, KansoError> {
        self.0.replace_undo(&batch)
    }

    /// Makes the daily backup if one is due.
    pub fn backup_if_due(&self) -> Result<Option<BackupFile>, KansoError> {
        self.0.backup_if_due()
    }

    pub fn back_up_now(&self) -> Result<BackupFile, KansoError> {
        self.0.back_up_now()
    }

    /// The backups, newest first.
    pub fn backups(&self) -> Result<Vec<BackupFile>, KansoError> {
        self.0.backups()
    }

    /// Deletes a backup made with `back_up_now`; refuses any other.
    pub fn delete_backup(&self, path: String) -> Result<(), KansoError> {
        self.0.delete_backup(&path)
    }
    #[frb(sync)]
    pub fn locks(&self) -> KansoLocks {
        KansoLocks::new(self.0.locks())
    }

    #[frb(sync)]
    pub fn discovery(&self) -> KansoDiscovery {
        KansoDiscovery::new(self.0.discovery())
    }

    #[frb(sync)]
    pub fn linking(&self) -> KansoLinking {
        KansoLinking::new(self.0.linking())
    }

    #[frb(sync)]
    pub fn syncing(&self) -> KansoSync {
        KansoSync::new(self.0.sync())
    }

    #[frb(sync)]
    pub fn renderer(&self) -> KansoRenderer {
        KansoRenderer::new(self.0.renderer())
    }

    #[frb(sync)]
    pub fn account(&self) -> KansoAccount {
        KansoAccount::new(self.0.account())
    }

    #[frb(sync)]
    pub fn account_settings(&self) -> KansoAccountSettings {
        KansoAccountSettings::new(self.0.account_settings())
    }

    #[frb(sync)]
    pub fn approvals(&self) -> KansoApprovals {
        KansoApprovals::new(self.0.approvals())
    }

    #[frb(sync)]
    pub fn shared(&self) -> KansoShared {
        KansoShared::new(self.0.shared())
    }

    #[frb(sync)]
    pub fn gallery(&self) -> KansoGallery {
        KansoGallery::new(self.0.gallery())
    }
}

/// What the Replace button says, or nothing when there is nothing to do.
#[frb(sync)]
pub fn replace_apply_label(matches: u32, notes: u32) -> Option<String> {
    kanso_mobile::replace_apply_label(matches, notes)
}

/// What the backups screen says of the daily backup and Back up now.
#[frb(sync)]
pub fn backups_summary() -> String {
    kanso_mobile::BACKUPS_SUMMARY.to_owned()
}
