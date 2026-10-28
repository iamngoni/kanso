//! The plain data of a note's history, Replace in all notes and the backups.
//! Like [`crate::model`], no logic lives here: the modules that build them
//! ([`crate::history`], [`crate::replace`], [`crate::backups`]) say what
//! each field holds. Text offsets are UTF-16 code units, the unit Flutter
//! counts in; line numbers count from 1.

use crate::model::Range16;

/// A note's saved versions beside the note as it is now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteHistory {
    pub current_title: String,
    pub current_body: String,
    /// "by user2, user3": who changed a shared note since its last saved
    /// version; `None` when nobody is named.
    pub current_authors: Option<String>,
    /// Newest first.
    pub versions: Vec<NoteVersion>,
}

/// One saved version of a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteVersion {
    pub rev: String,
    /// When it was saved, in milliseconds since the Unix epoch.
    pub saved_at: i64,
    /// What the write that replaced it came from: `ui`, `api`, `sync`,
    /// `import`, `collab`, `agent` or `plugin`.
    pub origin: String,
    /// Why it was kept, in words ("Edited in Kanso").
    pub origin_label: String,
    /// "by user2, user3" for a version of a shared note.
    pub authors: Option<String>,
    /// The agent whose suggested edit, applied by the person, replaced it.
    pub agent: Option<String>,
    pub title: String,
    pub body: String,
}

/// What a line of a changed section is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffKind {
    Same,
    /// In the note now, not in the version.
    Added,
    /// In the version, not in the note now.
    Removed,
}

/// A line of a changed section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffKind,
    /// Without its line break.
    pub text: String,
    /// Its number in the version, from 1; `None` for an added line.
    pub old_line: Option<u32>,
    /// Its number in the note now, from 1; `None` for a removed line.
    pub new_line: Option<u32>,
    /// `[start, end)` in UTF-16 units of `text`: the words that changed in
    /// a changed line. Empty for an unchanged line, and for one changed all
    /// through.
    pub changed: Vec<Range16>,
}

/// A changed section: its lines with a few unchanged ones around.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffHunk {
    pub lines: Vec<DiffLine>,
    /// The section as the version has it: what "Copy section" copies.
    pub old_text: String,
    pub added: u32,
    pub removed: u32,
    /// The first and last line of the section in the version ("Lines 4–9");
    /// `None` when the version has none there.
    pub old_start: Option<u32>,
    pub old_end: Option<u32>,
}

/// The toggles of Replace in all notes and what to find.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReplaceOptions {
    pub find: String,
    pub match_case: bool,
    pub whole_word: bool,
    pub regex: bool,
    /// Leave matches inside fenced code alone.
    pub skip_code: bool,
    pub include_trash: bool,
}

/// What a Replace would change, note by note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplacePreview {
    /// In plain words, when the pattern can't be used (a regular expression
    /// that doesn't parse). `None` for no find text too: that previews
    /// nothing.
    pub error: Option<String>,
    /// Matches in all the notes found.
    pub total_matches: u32,
    /// Most recently changed first.
    pub notes: Vec<ReplaceNote>,
}

/// A note with matches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceNote {
    pub id: String,
    pub title: String,
    /// The revision the matches were found in: pass it back in
    /// [`PlannedNote`].
    pub rev: String,
    pub trashed: bool,
    /// How many matches the note has.
    pub count: u32,
    /// The first few of them, with their lines.
    pub matches: Vec<ReplaceMatch>,
}

/// A match shown on its line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceMatch {
    /// The text of the line the match starts on.
    pub line: String,
    /// `[start, end)` in UTF-16 units of `line`.
    pub start_utf16: u32,
    pub end_utf16: u32,
    /// From 1.
    pub line_number: u32,
}

/// A note to change, at the revision its preview was made from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedNote {
    pub id: String,
    pub rev: String,
}

/// What a Replace did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceOutcome {
    /// Undo it with this.
    pub batch: String,
    pub replaced: u32,
    /// Notes changed.
    pub notes: u32,
    /// Notes left alone: changed since the preview, gone, or locked.
    pub skipped: u32,
    /// What to tell the person ("Replaced 4 matches in 2 notes.").
    pub message: String,
}

/// What undoing a Replace did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceUndone {
    /// Notes whose text came back.
    pub restored: u32,
    /// Notes left as they are: edited since the Replace, locked now or gone.
    pub skipped: u32,
    /// What to tell the person ("Put back 2 notes.").
    pub message: String,
}

/// A backup of the library's database.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackupFile {
    pub path: String,
    /// When it was made, in milliseconds since the Unix epoch.
    pub made_at_ms: i64,
    /// Made with Back up now: kept until it is deleted.
    pub manual: bool,
    pub bytes: u64,
}

/// A range in the units of a line's text.
pub(crate) fn range16(start: u32, end: u32) -> Range16 {
    Range16 { start, end }
}
