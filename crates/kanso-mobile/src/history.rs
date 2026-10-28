//! A note's saved versions: [`Library::history`], what changed since one
//! ([`Library::history_diff`]) and bringing one back
//! ([`Library::restore_version`]).
//!
//! A locked note's versions are sealed in the store. While the note is
//! closed none of this reads anything ([`KansoError::Locked`]); while it is
//! open each sealed version is opened with the session's keys, and one that
//! no key opens is left out. Nothing sealed is ever returned.

use kanso_core::model::LOCK_FIELD;
use kanso_core::{Note, now_ms};
use kanso_engine::history::diff::{self, Hunk, LineKind};
use kanso_engine::history::{authors_label, origin_label};
use kanso_engine::lock::LockView;
use kanso_engine::text::byte_to_utf16;
use kanso_store::{Origin, Put, Revision};
use serde_json::Value;

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::Saved;
use crate::model_history::{DiffHunk, DiffKind, DiffLine, NoteHistory, NoteVersion, range16};

/// A note read for its history: as it is now, what its lock allows, and
/// every version that can be read.
struct Past {
    current: Note,
    state: LockView,
    versions: Vec<NoteVersion>,
}

impl Library {
    /// The note's saved versions, newest first, beside the note as it is
    /// now. A closed locked note refuses with [`KansoError::Locked`] and
    /// nothing is read; an open one has its sealed versions opened with this
    /// session's keys, and a version no key opens isn't listed.
    pub fn history(&self, id: &str) -> Result<NoteHistory, KansoError> {
        guard(|| {
            let past = self.past(id)?;
            let authors = self.inner.store.shared_authors(id)?;
            Ok(NoteHistory {
                current_title: past.current.title,
                current_body: past.current.body,
                current_authors: authors_label(authors.iter().map(String::as_str)),
                versions: past.versions,
            })
        })
    }

    /// What changed from the version `rev` to the note now, section by
    /// section, with the words changed in each line. Empty when they are
    /// the same. [`KansoError::NotFound`] when that version isn't kept (or
    /// can't be opened); [`KansoError::Locked`] as for [`Library::history`].
    pub fn history_diff(&self, id: &str, rev: &str) -> Result<Vec<DiffHunk>, KansoError> {
        guard(|| {
            let past = self.past(id)?;
            let version = past
                .versions
                .iter()
                .find(|v| v.rev == rev)
                .ok_or(KansoError::NotFound)?;
            Ok(diff::hunks(&version.body, &past.current.body)
                .iter()
                .map(hunk)
                .collect())
        })
    }

    /// Sets the note's title and text to those of the version `rev`, as
    /// [`Library::save`] writes a text: refused with
    /// [`KansoError::Conflict`] when the note is no longer at `base_rev`,
    /// with [`KansoError::Locked`] while it is closed, and sealed when it is
    /// open. Restoring makes a new revision, it never rewinds, and the text
    /// the note had before is kept in its history, even right after another
    /// save.
    pub fn restore_version(
        &self,
        id: &str,
        rev: &str,
        base_rev: &str,
    ) -> Result<Saved, KansoError> {
        guard(|| {
            let mut past = self.past(id)?;
            let doc = self.inner.store.get(id)?.ok_or(KansoError::NotFound)?;
            if doc.rev != base_rev {
                return Err(KansoError::Conflict {
                    current_rev: doc.rev,
                });
            }
            let version = past
                .versions
                .iter()
                .find(|v| v.rev == rev)
                .ok_or(KansoError::NotFound)?;
            // The version about to be replaced goes in the history now, not
            // when the write's own snapshot decides (it holds back a second
            // one within minutes of the last).
            self.inner
                .store
                .record_revision(id, &doc.rev, &doc.fields, Origin::Ui)?;
            let note = &mut past.current;
            note.title = version.title.clone();
            note.body = version.body.clone();
            note.updated_at = now_ms();
            if past.state == LockView::Plain {
                kanso_engine::links::tags::apply_inline_tags(&self.inner.store, note, None);
            }
            if past.state == LockView::Open {
                held(&self.inner.keys)
                    .seal(note, std::time::Instant::now())
                    .map_err(|_| KansoError::Locked)?;
            }
            let value =
                serde_json::to_value(&*note).map_err(|err| KansoError::storage(err.to_string()))?;
            match self
                .inner
                .store
                .put(kanso_core::DocType::Note, value, Put::Upsert, Origin::Ui)
            {
                Ok(written) => Ok(Saved {
                    id: written.id,
                    rev: written.rev,
                }),
                Err(kanso_store::StoreError::Conflict) => Err(KansoError::Conflict {
                    current_rev: self.stored(id).ok().and_then(|n| n.rev).unwrap_or_default(),
                }),
                Err(err) => Err(err.into()),
            }
        })
    }

    /// The note as it reads now and its readable versions. Locked and
    /// closed: [`KansoError::Locked`], before any version is read.
    fn past(&self, id: &str) -> Result<Past, KansoError> {
        let mut current = self.stored(id)?;
        let state = self.reveal(&mut current);
        if state.is_closed() {
            return Err(KansoError::Locked);
        }
        let revisions = self.inner.store.revisions(id)?;
        let versions = revisions
            .into_iter()
            .filter_map(|revision| self.open_version(id, &state, revision))
            .collect();
        Ok(Past {
            current,
            state,
            versions,
        })
    }

    /// A version as the person reads it: its sealed text opened with this
    /// session's keys. `None` for one that can't be opened, and for a plain
    /// one of a note that is locked now (its history is only ever sealed).
    fn open_version(&self, id: &str, state: &LockView, revision: Revision) -> Option<NoteVersion> {
        let sealed = revision.fields.contains_key(LOCK_FIELD);
        if !sealed && *state != LockView::Plain {
            return None;
        }
        let mut fields = revision.fields;
        fields.insert("_id".into(), id.into());
        fields
            .entry("bookId")
            .or_insert_with(|| Value::String(String::new()));
        let mut version: Note = serde_json::from_value(Value::Object(fields)).ok()?;
        if sealed && self.reveal(&mut version).is_closed() {
            return None;
        }
        Some(NoteVersion {
            rev: revision.rev,
            saved_at: revision.saved_at,
            origin_label: origin_label(&revision.origin).to_owned(),
            origin: revision.origin,
            authors: authors_label(revision.author.as_deref()),
            agent: revision.agent,
            title: version.title,
            body: version.body,
        })
    }
}

/// A section as Dart draws it: kinds and numbers as they are, the changed
/// words in UTF-16 units of their line.
fn hunk(hunk: &Hunk) -> DiffHunk {
    let (added, removed) = hunk.counts();
    let span = hunk.old_span();
    DiffHunk {
        lines: hunk
            .lines
            .iter()
            .map(|line| DiffLine {
                kind: match line.kind {
                    LineKind::Same => DiffKind::Same,
                    LineKind::Added => DiffKind::Added,
                    LineKind::Removed => DiffKind::Removed,
                },
                text: line.text.clone(),
                old_line: line.old_line.map(|n| n as u32),
                new_line: line.new_line.map(|n| n as u32),
                changed: line
                    .changed
                    .iter()
                    .map(|r| {
                        range16(
                            byte_to_utf16(&line.text, r.start) as u32,
                            byte_to_utf16(&line.text, r.end) as u32,
                        )
                    })
                    .collect(),
            })
            .collect(),
        old_text: hunk.old_text(),
        added: added as u32,
        removed: removed as u32,
        old_start: span.map(|(first, _)| first as u32),
        old_end: span.map(|(_, last)| last as u32),
    }
}
