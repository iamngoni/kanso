//! Replace in all notes: a preview of every match, note by note, then one
//! change to the notes picked, which one call undoes.
//!
//! A note that changed since the preview (by sync, say) is left alone and
//! reported, and so is a locked note, whatever the plan says: the store
//! only ever reads a locked note as empty and refuses to write one.

use kanso_core::now_ms;
use kanso_core::replace::{Matcher, PatternError};
use kanso_engine::replace::{self, Options};
use kanso_engine::text::byte_to_utf16;
use kanso_store::Planned;

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model_history::{
    PlannedNote, ReplaceMatch, ReplaceNote, ReplaceOptions, ReplaceOutcome, ReplacePreview,
    ReplaceUndone,
};

/// What the Replace button says, or `None` when there is nothing to do.
pub fn replace_apply_label(matches: u32, notes: u32) -> Option<String> {
    replace::apply_label(matches as usize, notes as usize)
}

impl ReplaceOptions {
    fn matcher(&self) -> Result<Matcher, PatternError> {
        let options = Options {
            match_case: self.match_case,
            whole_word: self.whole_word,
            regex: self.regex,
            skip_code: self.skip_code,
            include_trash: self.include_trash,
        };
        Matcher::new(&options.pattern(&self.find))
    }
}

impl Library {
    /// Every match of `options.find` in the notes, most recently changed
    /// note first; notes in the trash only with `include_trash`. A locked
    /// note never appears. An empty find previews nothing, and a regular
    /// expression that doesn't parse says why in `error`.
    pub fn replace_preview(&self, options: ReplaceOptions) -> Result<ReplacePreview, KansoError> {
        guard(|| {
            let matcher = match options.matcher() {
                Ok(matcher) => matcher,
                Err(PatternError::Empty) => return Ok(preview(None, Vec::new())),
                Err(PatternError::Invalid(why)) => return Ok(preview(Some(why), Vec::new())),
            };
            let found = self
                .inner
                .store
                .replace_preview(&matcher, options.include_trash)?;
            let notes = found
                .into_iter()
                .map(|note| ReplaceNote {
                    id: note.id,
                    title: note.title,
                    rev: note.rev,
                    trashed: note.trashed,
                    count: note.count as u32,
                    matches: note
                        .matches
                        .into_iter()
                        .map(|m| ReplaceMatch {
                            start_utf16: byte_to_utf16(&m.text, m.range.start) as u32,
                            end_utf16: byte_to_utf16(&m.text, m.range.end) as u32,
                            line: m.text,
                            line_number: m.line + 1,
                        })
                        .collect(),
                })
                .collect();
            Ok(preview(None, notes))
        })
    }

    /// Replaces the matches with `replacement` (`$1` and `${name}` stand for
    /// a regular expression's groups) in the `planned` notes, in one write
    /// under one batch. A note no longer at its planned revision, gone, or
    /// locked is left alone and counted in `skipped`.
    pub fn replace_apply(
        &self,
        options: ReplaceOptions,
        replacement: String,
        planned: Vec<PlannedNote>,
    ) -> Result<ReplaceOutcome, KansoError> {
        guard(|| {
            let matcher = options
                .matcher()
                .map_err(|err| KansoError::invalid(err.to_string()))?;
            let planned: Vec<Planned> = planned
                .into_iter()
                .map(|note| Planned {
                    id: note.id,
                    rev: note.rev,
                })
                .collect();
            let batch = replace::new_batch();
            let report = self.inner.store.replace_in_notes(
                &matcher,
                &replacement,
                &planned,
                &batch,
                now_ms(),
            )?;
            Ok(ReplaceOutcome {
                message: replace::report_text(&report),
                replaced: report.replaced as u32,
                notes: report.notes.len() as u32,
                skipped: report.skipped.len() as u32,
                batch,
            })
        })
    }

    /// Puts back the text each note of `batch` had, except those edited
    /// since, locked now or gone. Says what it did. A batch is undone once.
    pub fn replace_undo(&self, batch: &str) -> Result<ReplaceUndone, KansoError> {
        guard(|| {
            let report = self.inner.store.undo_replace(batch, now_ms())?;
            Ok(ReplaceUndone {
                restored: report.restored.len() as u32,
                skipped: report.skipped.len() as u32,
                message: replace::undo_text(&report),
            })
        })
    }
}

fn preview(error: Option<String>, notes: Vec<ReplaceNote>) -> ReplacePreview {
    ReplacePreview {
        error,
        total_matches: notes.iter().map(|n| n.count).sum(),
        notes,
    }
}
