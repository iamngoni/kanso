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
use kanso_store::{CheckOff, Origin};

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
    /// A task's box, put back to `checked`.
    Task {
        note_id: String,
        line: u32,
        text: String,
        checked: bool,
    },
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
                Written::Changed(rev) => Ticked::Changed {
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
                        if let Some(undo) = batch::undo_now(&edit.change, &edit.undo, &current) {
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
                Undoable::Task {
                    note_id,
                    line,
                    text,
                    checked,
                } => Ok(match self.write_task(&note_id, line, &text, checked)? {
                    Written::Changed(_) => 1,
                    Written::Unchanged | Written::Missing => 0,
                }),
            }
        })
    }

    /// Keeps `change` for [`Library::undo`] and returns its ticket.
    pub(crate) fn keep_undo(&self, change: Undoable) -> u64 {
        held(&self.inner.undo).keep(change)
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
            return Ok(
                match store.set_task_checked(
                    note_id,
                    line,
                    text,
                    checked,
                    kanso_core::now_ms(),
                    Origin::Ui,
                )? {
                    CheckOff::Changed { rev } => Written::Changed(rev),
                    CheckOff::Unchanged => Written::Unchanged,
                    CheckOff::Missing => Written::Missing,
                },
            );
        }
        // A sealed note: its text is only readable with the lock open, and
        // saving seals it again.
        let (note, body) = self.readable_text(note_id)?.ok_or(KansoError::Locked)?;
        match kanso_core::tasks::set_task_checked(&body, line, text, checked) {
            TaskEdit::Changed(body) => {
                let rev = note.rev.unwrap_or_default();
                Ok(Written::Changed(self.save(note_id, &rev, body)?.rev))
            }
            TaskEdit::Unchanged => Ok(Written::Unchanged),
            TaskEdit::Missing => Ok(Written::Missing),
        }
    }
}

/// What a task write did.
enum Written {
    Changed(String),
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
