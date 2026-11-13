//! Boards: the notes a list shows laid out by status, or their tasks as To do
//! and Done, and moving a card from one column to another.
//!
//! The cards, columns and the move are `kanso_engine::board`, which the
//! desktop draws too. This reads them as `read_cards` in the desktop's
//! `board_view.rs` does, and keeps a locked note to its title and its lock:
//! it is a card with no text, status or tasks.

use std::collections::HashSet;

use kanso_core::NoteStatus;
use kanso_core::dates::DateTime;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::properties;
use kanso_core::schema::PropertyType;
use kanso_core::tasks::Due;
use kanso_engine::board::{self, Board, Card, CardId, Lane, Move, Written};
use kanso_engine::board_settings::{
    self, LaneEdit, edit_words, grouped_spec, notes_in_lane, removal_prompt,
};
use kanso_engine::tasks::{TaskId, display_text};
use kanso_store::{Origin, TaskQuery};

use crate::boundary::guard;
use crate::due::chip_row;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{
    BoardCard, BoardColumn, BoardKind, BoardLane, BoardSetup, BoardView, CardKey, CardMoved,
    LaneChange, LaneEdited, LaneRemoval, ListPlace, ListQuery,
};
use crate::tasks::local_now;
use crate::undo::Undoable;

impl Library {
    /// The board of the notes `query` lists, the trash left out: a column per
    /// status (a locked note is in "No status", with its title only), or a
    /// To do and a Done column of the tasks in those notes, in the notes'
    /// order and then down each note. A locked note has no tasks here. The
    /// due chips are worded at the phone's local time now.
    pub fn board(&self, kind: BoardKind, query: ListQuery) -> Result<BoardView, KansoError> {
        self.board_at(kind, query, local_now())
    }

    /// [`Library::board`] with the due chips worded at local time `now`.
    pub fn board_at(
        &self,
        kind: BoardKind,
        query: ListQuery,
        now: DateTime,
    ) -> Result<BoardView, KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let (mut notes, _, _) = self.listed(&query)?;
            notes.retain(|note| note.book_id != TRASH_BOOK_ID);
            let locked: HashSet<String> = notes
                .iter()
                .filter(|note| note.locked)
                .map(|note| note.id.clone())
                .collect();
            let (engine_kind, tasks) = match kind {
                BoardKind::Notes => (board::BoardKind::Notes, Vec::new()),
                BoardKind::Tasks => {
                    let mut tasks = store.tasks(&TaskQuery {
                        include_done: true,
                        ..Default::default()
                    })?;
                    // Their text is sealed with a locked note; one whose note
                    // is locked but not sealed yet is no more to be read.
                    tasks.retain(|task| !locked.contains(&task.note_id));
                    (board::BoardKind::Tasks, tasks)
                }
            };
            // A locked note is its title and its lock, as in a list: what
            // the store holds of its status and snippet isn't passed on.
            for note in notes.iter_mut().filter(|note| note.locked) {
                note.status = NoteStatus::None;
                note.snippet.clear();
                note.tags.clear();
            }
            // A notebook with a board of its own lays its notes out by the
            // property it names, as the desktop does.
            let book = match (&query.place, kind) {
                (ListPlace::Notebook { id }, BoardKind::Notes) => Some(id.as_str()),
                _ => None,
            };
            let group = match book {
                Some(book) => board::property_board(store, book)?,
                None => None,
            };
            let engine = match group {
                Some(group) => board::grouped_board(store, group, notes)?,
                None => {
                    let mut engine = Board::new(engine_kind);
                    engine.set_cards(board::cards(engine_kind, notes, tasks));
                    engine
                }
            };
            let columns = engine
                .columns()
                .into_iter()
                .map(|(lane, indices)| BoardColumn {
                    label: engine.label(&lane),
                    lane: lane_of(lane),
                    cards: indices
                        .into_iter()
                        .map(|ix| card_of(&engine.cards[ix], now))
                        .collect(),
                })
                .collect();
            let setup = match book {
                Some(book) => self.board_setup(book)?,
                None => None,
            };
            Ok(BoardView {
                kind,
                columns,
                setup,
            })
        })
    }

    /// What notebook `book_id`'s board is grouped by and can be grouped by;
    /// `None` for a notebook with no select property and no board of its own
    /// (its notes stay on the status lanes, with nothing to set).
    fn board_setup(&self, book_id: &str) -> Result<Option<BoardSetup>, KansoError> {
        let store = &self.inner.store;
        let groupable: Vec<String> = store
            .notebook_properties(book_id)?
            .into_iter()
            .filter(|def| def.kind == PropertyType::Select)
            .map(|def| def.key)
            .collect();
        let group = board::property_board(store, book_id)?;
        if groupable.is_empty() && group.is_none() {
            return Ok(None);
        }
        Ok(Some(BoardSetup {
            book_id: book_id.to_owned(),
            groupable,
            lanes: group
                .as_ref()
                .map(|group| group.values.clone())
                .unwrap_or_default(),
            show_unset: group.as_ref().is_none_or(|group| group.show_unset),
            group: group.map(|group| group.key),
        }))
    }

    /// Groups notebook `book_id`'s board by property `by` (the status lanes
    /// when `None`), saved with the notebook. Its own lanes stay when it
    /// keeps its property. The property must be one of the notebook's
    /// select properties ([`KansoError::Invalid`] otherwise).
    pub fn set_board_group(&self, book_id: &str, by: Option<String>) -> Result<(), KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let spec = match by {
                None => None,
                Some(by) => {
                    let known = store
                        .notebook_properties(book_id)?
                        .into_iter()
                        .any(|def| def.kind == PropertyType::Select && def.key == by);
                    if !known {
                        return Err(KansoError::invalid(format!(
                            "“{by}” isn't a select property of this notebook."
                        )));
                    }
                    Some(grouped_spec(store.notebook_board(book_id)?.as_ref(), &by))
                }
            };
            store.set_notebook_board(book_id, spec.as_ref(), Origin::Ui)?;
            Ok(())
        })
    }

    /// Shows or hides the lane of notes with no value while it is empty.
    /// A notebook with no board of its own has nothing to set
    /// ([`KansoError::Invalid`]).
    pub fn set_board_show_unset(&self, book_id: &str, show: bool) -> Result<(), KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let mut spec = store
                .notebook_board(book_id)?
                .ok_or_else(|| KansoError::invalid("This notebook has no board."))?;
            spec.show_unset = show;
            store.set_notebook_board(book_id, Some(&spec), Origin::Ui)?;
            Ok(())
        })
    }

    /// What removing `lane` from notebook `book_id`'s board asks: how many
    /// notes are in it and where they go unless the person says otherwise.
    pub fn board_lane_removal(&self, book_id: &str, lane: &str) -> Result<LaneRemoval, KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let group = board::property_board(store, book_id)?
                .ok_or_else(|| KansoError::invalid("This notebook has no board."))?;
            let at = group
                .values
                .iter()
                .position(|value| value == lane)
                .ok_or_else(|| KansoError::invalid(format!("There is no lane called {lane}.")))?;
            let notes = notes_in_lane(store, book_id, &group, lane)?.len() as u32;
            // The notes go to the next lane, or the one before the last.
            let target = group
                .values
                .get(at + 1)
                .or_else(|| {
                    at.checked_sub(1)
                        .and_then(|before| group.values.get(before))
                })
                .cloned();
            Ok(LaneRemoval {
                notes,
                prompt: if notes == 0 {
                    String::new()
                } else {
                    removal_prompt(lane, notes as usize)
                },
                target,
                targets: group
                    .values
                    .iter()
                    .filter(|value| *value != lane)
                    .cloned()
                    .collect(),
            })
        })
    }

    /// Renames, reorders, adds or removes a lane of notebook `book_id`'s
    /// board: every note in a renamed or removed lane is rewritten (one
    /// write each), the lanes and the property's options are saved, and a
    /// refused edit ([`KansoError::Invalid`], with the words to show) writes
    /// nothing. The result has what takes it all back.
    pub fn edit_board_lanes(
        &self,
        book_id: &str,
        change: LaneChange,
        now_ms: i64,
    ) -> Result<LaneEdited, KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let key = store
                .notebook_board(book_id)?
                .ok_or_else(|| KansoError::invalid("This notebook has no board."))?
                .by;
            let edit = match change {
                LaneChange::Rename { from, to } => LaneEdit::Rename { from, to },
                LaneChange::Move { from, to } => LaneEdit::Move {
                    from: from as usize,
                    to: to as usize,
                },
                LaneChange::Add { name } => LaneEdit::Add(name),
                LaneChange::Remove { lane, move_to } => LaneEdit::Remove { lane, move_to },
            };
            let done = board_settings::apply_lane_edit(store, book_id, &edit, now_ms)?;
            let message = edit_words(&edit, done.rewritten, &key);
            Ok(LaneEdited {
                lanes: done.lanes,
                rewritten: done.rewritten as u32,
                message,
                undo: self.keep_undo(Undoable::Lanes(done.undo)),
            })
        })
    }

    /// Puts `card` in column `to`: the note's status alone, or the task's box
    /// alone. Nothing is written when it is there already, or when the task
    /// is gone from its note (the board is out of date). A locked note's
    /// status can't be changed here ([`KansoError::Locked`]); a column that
    /// isn't the card's kind's is [`KansoError::Invalid`].
    pub fn move_card(
        &self,
        card: CardKey,
        to: BoardLane,
        now_ms: i64,
    ) -> Result<CardMoved, KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let to = engine_lane(&to)?;
            let (id, from) = match card {
                CardKey::Note { id } => {
                    let note = self.stored(&id)?;
                    if matches!(to, Lane::Value(_) | Lane::Unset | Lane::Unlisted) {
                        return self.move_note_to_value(&id, &note.book_id, to, now_ms);
                    }
                    let Lane::Status(status) = to else {
                        return Err(KansoError::invalid("A note goes in a status column."));
                    };
                    if store.note_is_locked(&id)? || store.locked_books()?.contains(&note.book_id) {
                        return Err(KansoError::Locked);
                    }
                    if note.status == status {
                        return Ok(CardMoved::Unchanged);
                    }
                    (CardId::Note(id), Lane::Status(note.status))
                }
                CardKey::Task {
                    note_id,
                    line,
                    text,
                } => {
                    let done = match to {
                        Lane::Todo => false,
                        Lane::Done => true,
                        Lane::Status(_) | Lane::Value(_) | Lane::Unset | Lane::Unlisted => {
                            return Err(KansoError::invalid("A task goes in To do or Done."));
                        }
                    };
                    let id = TaskId {
                        note_id,
                        line,
                        text,
                    };
                    let tasks = store.tasks(&TaskQuery {
                        include_done: true,
                        ..Default::default()
                    })?;
                    let Some(task) = tasks.iter().find(|task| TaskId::of(task) == id) else {
                        return Ok(CardMoved::Missing);
                    };
                    if task.checked == done {
                        return Ok(CardMoved::Unchanged);
                    }
                    let from = if task.checked { Lane::Done } else { Lane::Todo };
                    (CardId::Task(id), from)
                }
            };
            match board::write_move(
                store,
                &Move {
                    id,
                    from,
                    to,
                    property: None,
                },
                now_ms,
            )? {
                Written::Done => Ok(CardMoved::Changed {
                    message: None,
                    undo: None,
                }),
                Written::Moved => Ok(CardMoved::Missing),
            }
        })
    }

    /// Puts note `id` in lane `to` of its notebook's board: the one write
    /// of the notebook's property, as the desktop's drag does, with what
    /// takes it back. The lane must be one of the board's (the unlisted
    /// lane never is, a note can be there only by a value typed by hand);
    /// a note in a notebook with no board has none
    /// ([`KansoError::Invalid`]). A locked note is [`KansoError::Locked`].
    fn move_note_to_value(
        &self,
        id: &str,
        book_id: &str,
        to: Lane,
        now_ms: i64,
    ) -> Result<CardMoved, KansoError> {
        let store = &self.inner.store;
        let group = board::property_board(store, book_id)?
            .ok_or_else(|| KansoError::invalid("This note's notebook has no board."))?;
        let allowed = match &to {
            Lane::Value(value) => group.values.contains(value),
            Lane::Unset => true,
            _ => false,
        };
        if !allowed {
            return Err(KansoError::invalid(format!(
                "{} isn't a lane to move a note to.",
                group.label(&to)
            )));
        }
        if store.note_is_locked(id)? || store.locked_books()?.contains(book_id) {
            return Err(KansoError::Locked);
        }
        let from = match store.note(id)? {
            Some(note) => group.place(properties::get(&properties::parse(&note.body), &group.key)),
            None => return Ok(CardMoved::Missing),
        };
        if from == to {
            return Ok(CardMoved::Unchanged);
        }
        let label = group.label(&to);
        let (written, undo) = board::write_move_undoable(
            store,
            &Move {
                id: CardId::Note(id.to_owned()),
                from,
                to,
                property: Some(group.key.clone()),
            },
            now_ms,
        )?;
        Ok(match written {
            Written::Moved => CardMoved::Missing,
            Written::Done => CardMoved::Changed {
                message: Some(format!("Moved to {label}.")),
                undo: undo.map(|undo| self.keep_undo(Undoable::Property(undo))),
            },
        })
    }
}

fn lane_of(lane: Lane) -> BoardLane {
    match lane {
        Lane::Status(status) => BoardLane::Status {
            status: status.as_str().to_owned(),
        },
        Lane::Todo => BoardLane::Todo,
        Lane::Done => BoardLane::Done,
        Lane::Value(value) => BoardLane::Value { value },
        Lane::Unset => BoardLane::Unset,
        Lane::Unlisted => BoardLane::Unlisted,
    }
}

fn engine_lane(lane: &BoardLane) -> Result<Lane, KansoError> {
    Ok(match lane {
        BoardLane::Value { value } => Lane::Value(value.clone()),
        BoardLane::Unset => Lane::Unset,
        BoardLane::Unlisted => Lane::Unlisted,
        BoardLane::Status { status } => Lane::Status(
            NoteStatus::parse(status)
                .ok_or_else(|| KansoError::invalid(format!("“{status}” isn't a status.")))?,
        ),
        BoardLane::Todo => Lane::Todo,
        BoardLane::Done => Lane::Done,
    })
}

/// A card as the app draws it, its due chip worded at local time `now`.
fn card_of(card: &Card, now: DateTime) -> BoardCard {
    match card {
        Card::Note(note) => BoardCard {
            key: CardKey::Note {
                id: note.id.clone(),
            },
            title: note.title.clone(),
            note_title: note.title.clone(),
            snippet: note.snippet.clone(),
            due: None,
            due_chip: None,
            checked: false,
            locked: note.locked,
        },
        Card::Task(task) => {
            let due = task.due.as_deref().and_then(Due::parse);
            BoardCard {
                key: CardKey::Task {
                    note_id: task.note_id.clone(),
                    line: task.line,
                    text: task.text.clone(),
                },
                title: display_text(&task.text),
                note_title: task.note_title.clone(),
                snippet: String::new(),
                due: task.due.clone(),
                due_chip: due.map(|due| chip_row(due, task.checked, now)),
                checked: task.checked,
                locked: false,
            }
        }
    }
}
