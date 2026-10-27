//! A note open for editing: the text, its undo history, the editor's
//! commands, the colors of the text, completion and saving. The text lives
//! in Rust; the Dart field draws it and hands every change over.
//!
//! The calls the field makes on every keystroke are `sync`: they run on the
//! UI isolate, in microseconds, so that a command and the field's text change
//! in the same turn. The rest run on the bridge's threads.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{
    Completion, CursorMark, EditCommand, EditOutcome, EditorContext, EditorEvent, EditorSession,
    EditorSnapshot, KansoError, LinkTarget, Property, Range16, RunBatch, SaveState, ShareFile,
    TextChange,
};

/// A note open for editing. Cheap to clone on the Rust side; Dart holds one.
#[frb(opaque)]
pub struct KansoEditor(EditorSession);

impl KansoEditor {
    pub(super) fn new(session: EditorSession) -> KansoEditor {
        KansoEditor(session)
    }

    /// The note as it is now.
    pub fn snapshot(&self) -> Result<EditorSnapshot, KansoError> {
        self.0.snapshot()
    }

    /// The field's text changed.
    #[frb(sync)]
    pub fn apply(&self, change: TextChange) -> Result<EditOutcome, KansoError> {
        self.0.apply(change)
    }

    /// The field's selection changed; what is under the caret.
    #[frb(sync)]
    pub fn select(&self, selection: Range16, composing: bool) -> Result<EditorContext, KansoError> {
        self.0.select(selection, composing)
    }

    /// A change the editor decides on; none when there is nothing to do.
    #[frb(sync)]
    pub fn command(&self, command: EditCommand) -> Result<Option<EditOutcome>, KansoError> {
        self.0.command(command)
    }

    #[frb(sync)]
    pub fn undo(&self) -> Result<Option<EditOutcome>, KansoError> {
        self.0.undo()
    }

    #[frb(sync)]
    pub fn redo(&self) -> Result<Option<EditOutcome>, KansoError> {
        self.0.redo()
    }

    #[frb(sync)]
    pub fn set_title(&self, title: String) -> Result<(), KansoError> {
        self.0.set_title(title)
    }

    /// The colors of `start..end` of the text in a built-in theme.
    pub fn styled_runs(
        &self,
        start: u32,
        end: u32,
        theme_id: String,
    ) -> Result<RunBatch, KansoError> {
        self.0.styled_runs(start, end, theme_id)
    }

    /// What the caret completes, if anything.
    pub fn completion(&self) -> Result<Option<Completion>, KansoError> {
        self.0.completion()
    }

    /// Where the link at an offset of the text leads.
    pub fn link_at(&self, offset: u32) -> Result<Option<LinkTarget>, KansoError> {
        self.0.link_at(offset)
    }

    pub fn set_property(
        &self,
        index: Option<u32>,
        key: String,
        value: String,
    ) -> Result<Vec<Property>, KansoError> {
        self.0.set_property(index, key, value)
    }

    pub fn remove_property(&self, index: u32) -> Result<Vec<Property>, KansoError> {
        self.0.remove_property(index)
    }

    /// Saves now.
    pub fn flush(&self) -> Result<SaveState, KansoError> {
        self.0.flush()
    }

    pub fn keep_mine(&self) -> Result<SaveState, KansoError> {
        self.0.keep_mine()
    }

    /// Stores an image or a PDF and puts it in the text over the selection.
    pub fn attach(&self, file: ShareFile) -> Result<EditOutcome, KansoError> {
        self.0.attach(file)
    }

    /// Comments on the selected words of a shared note; the new comment's ID.
    pub fn comment_on_selection(&self, body: String) -> Result<String, KansoError> {
        self.0.comment_on_selection(body)
    }

    /// The words the note's open comments are on, to tint.
    pub fn comment_marks(&self) -> Result<Vec<Range16>, KansoError> {
        self.0.comment_marks()
    }

    /// Other people's carets and selections, in units of the text.
    pub fn remote_cursors(&self) -> Result<Vec<CursorMark>, KansoError> {
        self.0.remote_cursors()
    }

    /// Selects the words a comment is on; the selection, if it was found.
    pub fn select_comment(&self, comment_id: String) -> Result<Option<Range16>, KansoError> {
        self.0.select_comment(comment_id)
    }

    pub fn use_theirs(&self) -> Result<EditorSnapshot, KansoError> {
        self.0.use_theirs()
    }

    /// Saves and ends the session.
    pub fn close(&self) -> Result<SaveState, KansoError> {
        self.0.close()
    }

    /// What happens to the note that Dart didn't ask for, until Dart stops
    /// listening.
    pub fn events(&self, sink: StreamSink<EditorEvent>) {
        self.0.events(move |event| sink.add(event).is_ok());
    }
}
