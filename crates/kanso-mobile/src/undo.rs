//! Changes the Undo notice can take back (#48 step 7): a swipe or a
//! multi-select change to notes (`kanso_engine::batch`) and a ticked task.
//!
//! The change is kept here, in Rust, under a ticket; the shell holds only
//! the ticket and hands it back to [`Library::undo`]. The last few changes
//! are kept, so an older notice's Undo still works while it shows, and a
//! ticket is used once.
//!
//! Undo puts back only what still holds the change: a note's field changed
//! again since keeps its newer value (`batch::undo_now`), and a task whose
//! box was changed again, or that moved away, is left alone.

use std::collections::VecDeque;

use kanso_core::DocType;
use kanso_core::tasks::TaskEdit;
use kanso_engine::batch;
use kanso_store::{CheckOff, Origin, TickUndo};

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::Ticked;

/// How many changes are kept for Undo.
const KEPT: usize = 16;

/// A change that can be taken back.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Undoable {
    /// Notes' fields, each with the write that undoes it.
    Fields(Vec<batch::Edit>),
    /// A Triage filing: the notes' fields, and the tags it made, which go
    /// again when nothing else uses them.
    Filing {
        edits: Vec<batch::Edit>,
        new_tags: Vec<String>,
    },
    /// A task's box, put back to `checked`.
    Task {
        note_id: String,
        line: u32,
        text: String,
        checked: bool,
        /// Set when the tick wrote a repeating task's next one: the body to
        /// put back, which takes both lines away in one write.
        restore: Option<TickUndo>,
    },
    /// A card moved to a lane of a notebook's property board: its note's
    /// property, put back in one write.
    Property(kanso_engine::board::MoveUndo),
    /// A lane edit of a property board: the board, the property list and
    /// each note it rewrote.
    Lanes(kanso_engine::board_settings::SettingsUndo),
}

/// The changes kept for Undo, newest last.
#[derive(Debug, Default)]
pub(crate) struct UndoBook {
    next: u64,
    kept: VecDeque<(u64, Undoable)>,
}

impl UndoBook {
    /// Keeps `change` and returns its ticket; the oldest goes when more
    /// than [`KEPT`] are kept.
    pub(crate) fn keep(&mut self, change: Undoable) -> u64 {
        self.next += 1;
        self.kept.push_back((self.next, change));
        while self.kept.len() > KEPT {
            self.kept.pop_front();
        }
        self.next
    }

    /// The change kept under `ticket`, taken out: a ticket is used once.
    pub(crate) fn take(&mut self, ticket: u64) -> Option<Undoable> {
        let at = self.kept.iter().position(|(t, _)| *t == ticket)?;
        self.kept.remove(at).map(|(_, change)| change)
    }
}

impl Library {
    /// Ticks (or clears) the task with `text` on `line` (0-based) of note
    /// `note_id`, in one write that changes only its box, the way the
    /// desktop's Tasks view and preview do; when the line moved, the one
    /// task with that text is changed. A locked note that is open is sealed
    /// again; a closed one refuses with [`KansoError::Locked`].
    pub fn set_task(
        &self,
        note_id: &str,
        line: u32,
        text: &str,
        checked: bool,
    ) -> Result<Ticked, KansoError> {
        guard(|| {
            let ticked = self.write_task(note_id, line, text, checked)?;
            Ok(match ticked {
                Written::Changed(rev, restore) => Ticked::Changed {
                    rev,
                    message: {
                        let name = kanso_engine::tasks::display_text(text);
                        if checked {
                            format!("Ticked off “{name}”")
                        } else {
                            format!("Opened “{name}” again")
                        }
                    },
                    undo: self.keep_undo(Undoable::Task {
                        note_id: note_id.to_owned(),
                        line,
                        text: text.to_owned(),
                        checked: !checked,
                        restore,
                    }),
                },
                Written::Unchanged => Ticked::Unchanged,
                Written::Missing => Ticked::Missing,
            })
        })
    }

    /// Takes back the change kept under `ticket`, as far as it still holds,
    /// and returns how many notes it wrote (none when nothing was left to
    /// put back, or the ticket is used or too old).
    pub fn undo(&self, ticket: u64) -> Result<u32, KansoError> {
        guard(|| {
            let Some(change) = held(&self.inner.undo).take(ticket) else {
                return Ok(0);
            };
            match change {
                Undoable::Fields(edits) => {
                    let mut back = Vec::with_capacity(edits.len());
                    for edit in edits {
                        let Ok(note) = self.stored(&edit.id) else {
                            continue;
                        };
                        let current = serde_json::to_value(&note)
                            .map_err(|err| KansoError::storage(err.to_string()))?;
                        if let Some(undo) = batch::undo_stamped(
                            &edit.change,
                            &edit.undo,
                            &current,
                            kanso_core::now_ms(),
                        ) {
                            back.push(undo);
                        }
                    }
                    let count = back.len() as u32;
                    if !back.is_empty() {
                        self.inner
                            .store
                            .merge_many(DocType::Note, back, Origin::Ui)?;
                    }
                    Ok(count)
                }
                Undoable::Filing { edits, new_tags } => Ok(kanso_engine::triage::file::undo(
                    &self.inner.store,
                    &edits,
                    &new_tags,
                    kanso_core::now_ms(),
                )? as u32),
                Undoable::Property(undo) => Ok(u32::from(kanso_engine::board::undo_move(
                    &self.inner.store,
                    &undo,
                )?)),
                Undoable::Lanes(undo) => {
                    kanso_engine::board_settings::undo_lane_edit(&self.inner.store, &undo)?;
                    Ok(undo.edits.len() as u32)
                }
                Undoable::Task {
                    note_id,
                    line,
                    text,
                    checked,
                    restore,
                } => {
                    if let Some(restore) = restore {
                        return self.restore_tick(&note_id, &restore);
                    }
                    Ok(match self.write_task(&note_id, line, &text, checked)? {
                        Written::Changed(..) => 1,
                        Written::Unchanged | Written::Missing => 0,
                    })
                }
            }
        })
    }

    /// Keeps `change` for [`Library::undo`] and returns its ticket.
    pub(crate) fn keep_undo(&self, change: Undoable) -> u64 {
        held(&self.inner.undo).keep(change)
    }

    /// Puts back the body before a repeating task's tick, as one write,
    /// while the note still is what the tick made; a sealed note has to be
    /// open. Returns how many notes it wrote.
    fn restore_tick(&self, note_id: &str, restore: &TickUndo) -> Result<u32, KansoError> {
        let store = &self.inner.store;
        if !store.note_is_locked(note_id)? {
            let off = store.undo_tick(note_id, restore, kanso_core::now_ms(), Origin::Ui)?;
            return Ok(u32::from(matches!(off, CheckOff::Changed { .. })));
        }
        let (note, body) = self.readable_text(note_id)?.ok_or(KansoError::Locked)?;
        if body != restore.after {
            return Ok(0);
        }
        let rev = note.rev.unwrap_or_default();
        self.save(note_id, &rev, restore.before.clone())?;
        Ok(1)
    }

    /// The write behind [`Library::set_task`], without a ticket.
    fn write_task(
        &self,
        note_id: &str,
        line: u32,
        text: &str,
        checked: bool,
    ) -> Result<Written, KansoError> {
        let store = &self.inner.store;
        if !store.note_is_locked(note_id)? {
            let now = kanso_core::now_ms();
            let tick = store.tick_task(
                note_id,
                line,
                text,
                checked,
                kanso_store::local_today(now),
                now,
                Origin::Ui,
            )?;
            return Ok(match tick.off {
                CheckOff::Changed { rev } => Written::Changed(rev, tick.undo),
                CheckOff::Unchanged => Written::Unchanged,
                CheckOff::Missing => Written::Missing,
            });
        }
        // A sealed note: its text is only readable with the lock open, and
        // saving seals it again.
        let (note, body) = self.readable_text(note_id)?.ok_or(KansoError::Locked)?;
        let today = kanso_store::local_today(kanso_core::now_ms());
        match kanso_core::tasks::set_task_checked(&body, line, text, checked, today) {
            TaskEdit::Changed(after) => {
                let rev = note.rev.unwrap_or_default();
                let restore = (after.lines().count() > body.lines().count()).then(|| TickUndo {
                    before: body.clone(),
                    after: after.clone(),
                });
                Ok(Written::Changed(
                    self.save(note_id, &rev, after)?.rev,
                    restore,
                ))
            }
            TaskEdit::Unchanged => Ok(Written::Unchanged),
            TaskEdit::Missing => Ok(Written::Missing),
        }
    }
}

/// What a task write did.
enum Written {
    /// The new revision, and what takes a repeating task's tick back.
    Changed(String, Option<TickUndo>),
    Unchanged,
    Missing,
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn task(n: u32) -> Undoable {
        Undoable::Task {
            note_id: format!("note:{n}"),
            line: n,
            text: "t".into(),
            checked: false,
            restore: None,
        }
    }

    #[test]
    fn a_ticket_is_used_once() {
        let mut book = UndoBook::default();
        let ticket = book.keep(task(1));
        assert_eq!(book.take(ticket), Some(task(1)));
        assert_eq!(book.take(ticket), None);
    }

    #[test]
    fn unknown_tickets_take_nothing() {
        let mut book = UndoBook::default();
        book.keep(task(1));
        assert_eq!(book.take(0), None);
        assert_eq!(book.take(99), None);
    }

    proptest! {
        /// Tickets are new each time; the last `KEPT` changes can be taken
        /// back, each as it was kept, and older ones can't.
        #[test]
        fn prop_the_last_changes_are_kept(count in 1u32..64) {
            let mut book = UndoBook::default();
            let tickets: Vec<u64> = (0..count).map(|n| book.keep(task(n))).collect();
            let mut seen = std::collections::HashSet::new();
            prop_assert!(tickets.iter().all(|t| seen.insert(*t)));
            let first_kept = count.saturating_sub(KEPT as u32);
            for (n, ticket) in tickets.iter().enumerate() {
                let n = n as u32;
                let taken = book.take(*ticket);
                if n >= first_kept {
                    prop_assert_eq!(taken, Some(task(n)));
                } else {
                    prop_assert_eq!(taken, None);
                }
            }
        }
    }
}
