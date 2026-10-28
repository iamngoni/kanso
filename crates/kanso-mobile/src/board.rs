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
use kanso_core::tasks::Due;
use kanso_engine::board::{self, Board, Card, CardId, Lane, Move, Written};
use kanso_engine::tasks::{TaskId, display_text};
use kanso_store::TaskQuery;

use crate::boundary::guard;
use crate::due::chip_row;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{
    BoardCard, BoardColumn, BoardKind, BoardLane, BoardView, CardKey, CardMoved, ListQuery,
};
use crate::tasks::local_now;

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
            let mut engine = Board::new(engine_kind);
            engine.set_cards(board::cards(engine_kind, notes, tasks));
            let columns = engine
                .columns()
                .into_iter()
                .map(|(lane, indices)| BoardColumn {
                    label: lane.label().to_owned(),
                    lane: lane_of(lane),
                    cards: indices
                        .into_iter()
                        .map(|ix| card_of(&engine.cards[ix], now))
                        .collect(),
                })
                .collect();
            Ok(BoardView { kind, columns })
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
                    let Lane::Status(status) = to else {
                        return Err(KansoError::invalid("A note goes in a status column."));
                    };
                    let note = self.stored(&id)?;
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
                        Lane::Status(_) => {
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
            match board::write_move(store, &Move { id, from, to }, now_ms)? {
                Written::Done => Ok(CardMoved::Changed),
                Written::Moved => Ok(CardMoved::Missing),
            }
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
    }
}

fn engine_lane(lane: &BoardLane) -> Result<Lane, KansoError> {
    Ok(match lane {
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
