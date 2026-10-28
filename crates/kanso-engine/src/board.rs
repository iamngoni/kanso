//! The board's cards and columns without a window: the notes a list shows
//! laid out by status, or their tasks as To do and Done. A card moves to
//! another column by a drag or by the keyboard, and a move changes one thing
//! in its note: the note's status, or the task's box. The desktop draws it
//! in `board_view.rs`, the mobile app in Flutter.

use kanso_core::{DocType, NoteStatus};
use kanso_store::{CheckOff, NoteSummary, Origin, Put, Store, StoreError, TaskRow};
use serde_json::json;

use crate::tasks::TaskId;

/// What a board lays out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BoardKind {
    /// Notes, in a column per status.
    #[default]
    Notes,
    /// Tasks, in To do and Done.
    Tasks,
}

/// A board's column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lane {
    Status(NoteStatus),
    Todo,
    Done,
}

impl Lane {
    pub fn label(self) -> &'static str {
        match self {
            Lane::Status(NoteStatus::None) => "No status",
            Lane::Status(status) => status.label(),
            Lane::Todo => "To do",
            Lane::Done => "Done",
        }
    }
}

impl BoardKind {
    /// The columns, left to right.
    pub fn lanes(self) -> Vec<Lane> {
        match self {
            BoardKind::Notes => NoteStatus::ALL.into_iter().map(Lane::Status).collect(),
            BoardKind::Tasks => vec![Lane::Todo, Lane::Done],
        }
    }
}

/// Which card: a note, or a task in its note.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CardId {
    Note(String),
    Task(TaskId),
}

/// A card and what it shows.
#[derive(Clone, Debug, PartialEq)]
pub enum Card {
    Note(NoteSummary),
    Task(TaskRow),
}

impl Card {
    pub fn id(&self) -> CardId {
        match self {
            Card::Note(note) => CardId::Note(note.id.clone()),
            Card::Task(row) => CardId::Task(TaskId::of(row)),
        }
    }

    /// The column it is in, from its note's status or its task's box.
    pub fn lane(&self) -> Lane {
        match self {
            Card::Note(note) => Lane::Status(note.status),
            Card::Task(row) if row.checked => Lane::Done,
            Card::Task(_) => Lane::Todo,
        }
    }

    /// The note it opens.
    pub fn note_id(&self) -> &str {
        match self {
            Card::Note(note) => &note.id,
            Card::Task(row) => &row.note_id,
        }
    }

    /// Puts the card in `lane`, as the store will once the move is written.
    fn set_lane(&mut self, lane: Lane) {
        match (self, lane) {
            (Card::Note(note), Lane::Status(status)) => note.status = status,
            (Card::Task(row), Lane::Todo) => row.checked = false,
            (Card::Task(row), Lane::Done) => row.checked = true,
            _ => {}
        }
    }
}

/// A card moved from one column to another, to be written to its note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Move {
    pub id: CardId,
    pub from: Lane,
    pub to: Lane,
}

/// The cards, their columns and the selection. Pure.
#[derive(Default)]
pub struct Board {
    pub kind: BoardKind,
    pub cards: Vec<Card>,
    pub loaded: bool,
    pub selected: Option<CardId>,
}

impl Board {
    pub fn new(kind: BoardKind) -> Self {
        Board {
            kind,
            ..Board::default()
        }
    }

    /// Each column with its cards (indices into `cards`), in the order
    /// they came. Every card is in exactly one column: its own.
    pub fn columns(&self) -> Vec<(Lane, Vec<usize>)> {
        self.kind
            .lanes()
            .into_iter()
            .map(|lane| {
                let cards = (0..self.cards.len())
                    .filter(|&ix| self.cards[ix].lane() == lane)
                    .collect();
                (lane, cards)
            })
            .collect()
    }

    /// Where card `id` is: its column and its place in it.
    pub fn position(&self, id: &CardId) -> Option<(usize, usize)> {
        self.columns()
            .iter()
            .enumerate()
            .find_map(|(col, (_, cards))| {
                cards
                    .iter()
                    .position(|&ix| self.cards[ix].id() == *id)
                    .map(|row| (col, row))
            })
    }

    /// Where the selection is.
    pub fn selected_position(&self) -> Option<(usize, usize)> {
        self.position(self.selected.as_ref()?)
    }

    pub fn selected_card(&self) -> Option<&Card> {
        let id = self.selected.as_ref()?;
        self.cards.iter().find(|card| card.id() == *id)
    }

    /// Takes newly read cards. The selection stays on its card, or goes to
    /// the card now in its place in the same column.
    pub fn set_cards(&mut self, cards: Vec<Card>) {
        let before = self.selected_position();
        self.cards = cards;
        self.loaded = true;
        if self.selected_position().is_none() {
            self.selected = before.and_then(|(col, row)| {
                let columns = self.columns();
                let cards = &columns.get(col)?.1;
                let ix = *cards.get(row.min(cards.len().checked_sub(1)?))?;
                Some(self.cards[ix].id())
            });
        }
    }

    /// Moves the selection `delta` cards down (or up) its column; the first
    /// card on the board when none is selected.
    pub fn step_down(&mut self, delta: isize) {
        let columns = self.columns();
        let Some((col, row)) = self.selected_position() else {
            self.select_first(&columns);
            return;
        };
        let cards = &columns[col].1;
        let row = row.saturating_add_signed(delta).min(cards.len() - 1);
        self.selected = Some(self.cards[cards[row]].id());
    }

    /// Moves the selection to the next column `delta` to the right (or
    /// left) that has cards, at the same height or its last card. At the
    /// edge, or with only empty columns beyond, it stays.
    pub fn step_across(&mut self, delta: isize) {
        let columns = self.columns();
        let Some((col, row)) = self.selected_position() else {
            self.select_first(&columns);
            return;
        };
        let mut at = col;
        loop {
            let Some(next) = at.checked_add_signed(delta).filter(|&n| n < columns.len()) else {
                return;
            };
            at = next;
            let cards = &columns[at].1;
            if !cards.is_empty() {
                let ix = cards[row.min(cards.len() - 1)];
                self.selected = Some(self.cards[ix].id());
                return;
            }
        }
    }

    fn select_first(&mut self, columns: &[(Lane, Vec<usize>)]) {
        self.selected = columns
            .iter()
            .find_map(|(_, cards)| cards.first())
            .map(|&ix| self.cards[ix].id());
    }

    /// Moves the selected card one column `delta` to the right (or left),
    /// empty or not. Nothing at the edge.
    pub fn move_selected(&mut self, delta: isize) -> Option<Move> {
        let id = self.selected.clone()?;
        let lanes = self.kind.lanes();
        let (col, _) = self.position(&id)?;
        let to = *lanes.get(col.checked_add_signed(delta)?)?;
        self.move_card(&id, to)
    }

    /// Puts card `id` in column `to`, and says what to write. Nothing when
    /// the card isn't on the board, is in that column already, or the
    /// column isn't this board's. The card stays selected.
    pub fn move_card(&mut self, id: &CardId, to: Lane) -> Option<Move> {
        if !self.kind.lanes().contains(&to) {
            return None;
        }
        let card = self.cards.iter_mut().find(|card| card.id() == *id)?;
        let from = card.lane();
        if from == to {
            return None;
        }
        card.set_lane(to);
        self.selected = Some(card.id());
        Some(Move {
            id: id.clone(),
            from,
            to,
        })
    }
}

/// What writing a move did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Written {
    Done,
    /// The task's note changed since the board was read; nothing was
    /// written.
    Moved,
}

/// Writes `mv` to its note: the note's status alone, or the task's box
/// alone. The open note goes through the editor instead (the workspace
/// sees to that), so its unsaved typing stays.
pub fn write_move(store: &Store, mv: &Move, now_ms: i64) -> Result<Written, StoreError> {
    match (&mv.id, mv.to) {
        (CardId::Note(id), Lane::Status(status)) => {
            store.put(
                DocType::Note,
                json!({"_id": id, "status": status.as_str()}),
                Put::Merge,
                Origin::Ui,
            )?;
            Ok(Written::Done)
        }
        (CardId::Task(task), Lane::Todo | Lane::Done) => {
            let checked = mv.to == Lane::Done;
            match store.set_task_checked(
                &task.note_id,
                task.line,
                &task.text,
                checked,
                now_ms,
                Origin::Ui,
            ) {
                Ok(CheckOff::Changed { .. } | CheckOff::Unchanged) => Ok(Written::Done),
                Ok(CheckOff::Missing) | Err(StoreError::NotFound) => Ok(Written::Moved),
                Err(err) => Err(err),
            }
        }
        // A move is only ever made within its own kind of board.
        _ => Ok(Written::Done),
    }
}

/// The board's cards: the notes, or every task in them, in the notes'
/// order and then down each note.
pub fn cards(kind: BoardKind, notes: Vec<NoteSummary>, tasks: Vec<TaskRow>) -> Vec<Card> {
    match kind {
        BoardKind::Notes => notes.into_iter().map(Card::Note).collect(),
        BoardKind::Tasks => {
            let order: std::collections::HashMap<&str, usize> = notes
                .iter()
                .enumerate()
                .map(|(ix, note)| (note.id.as_str(), ix))
                .collect();
            let mut tasks: Vec<(usize, TaskRow)> = tasks
                .into_iter()
                .filter_map(|row| Some((*order.get(row.note_id.as_str())?, row)))
                .collect();
            tasks.sort_by_key(|(ix, row)| (*ix, row.line));
            tasks.into_iter().map(|(_, row)| Card::Task(row)).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Board, BoardKind, Card, CardId, Lane, Move, Written, cards, write_move};
    use kanso_core::{DocType, NoteStatus};
    use kanso_store::{NoteSummary, Origin, Put, Store, TaskQuery};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 64,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn note(ix: usize, status: NoteStatus) -> NoteSummary {
        NoteSummary {
            id: format!("note:n{ix}"),
            rev: String::new(),
            title: format!("N{ix}"),
            snippet: String::new(),
            book_id: "book:a".into(),
            status,
            tags: Vec::new(),
            pinned: false,
            updated_at: 0,
            created_at: 0,
            tasks_done: 0,
            tasks_total: 0,
            locked: false,
        }
    }

    fn notes_board(statuses: &[usize]) -> Board {
        let mut board = Board::new(BoardKind::Notes);
        board.set_cards(
            statuses
                .iter()
                .enumerate()
                .map(|(ix, s)| Card::Note(note(ix, NoteStatus::ALL[*s])))
                .collect(),
        );
        board
    }

    fn status_of(board: &Board, ix: usize) -> NoteStatus {
        match &board.cards[ix] {
            Card::Note(note) => note.status,
            Card::Task(_) => unreachable!(),
        }
    }

    #[test]
    fn columns_and_keys() {
        // Active: 0, 2; Completed: 1; the rest empty.
        let mut board = notes_board(&[1, 3, 1]);
        let lanes: Vec<usize> = board.columns().iter().map(|(_, c)| c.len()).collect();
        assert_eq!(lanes, [0, 2, 0, 1, 0]);
        // Down from nothing: the first card on the board.
        board.step_down(1);
        assert_eq!(board.selected_position(), Some((1, 0)));
        board.step_down(1);
        assert_eq!(board.selected_position(), Some((1, 1)));
        // Across skips the empty On hold, keeping the height it can.
        board.step_across(1);
        assert_eq!(board.selected_position(), Some((3, 0)));
        board.step_across(1);
        assert_eq!(board.selected_position(), Some((3, 0)), "only empty beyond");
        // Moving goes one column, empty or not, and the card stays selected.
        let moved = board.move_selected(-1).unwrap();
        assert_eq!(
            moved,
            Move {
                id: CardId::Note("note:n1".into()),
                from: Lane::Status(NoteStatus::Completed),
                to: Lane::Status(NoteStatus::OnHold),
            }
        );
        assert_eq!(board.selected_position(), Some((2, 0)));
        assert_eq!(status_of(&board, 1), NoteStatus::OnHold);
        // At the edge there is nowhere to go.
        board.move_selected(-1);
        board.move_selected(-1);
        assert_eq!(board.selected_position(), Some((0, 0)));
        assert_eq!(board.move_selected(-1), None);
        // A card already in the column, or a column of the other board.
        let id = CardId::Note("note:n0".into());
        assert_eq!(board.move_card(&id, Lane::Status(NoteStatus::Active)), None);
        assert_eq!(board.move_card(&id, Lane::Done), None);
    }

    #[test]
    fn the_selection_keeps_its_place_when_its_card_goes() {
        let mut board = notes_board(&[1, 1, 1]);
        board.selected = Some(CardId::Note("note:n1".into()));
        let rest = vec![
            Card::Note(note(0, NoteStatus::Active)),
            Card::Note(note(2, NoteStatus::Active)),
        ];
        board.set_cards(rest);
        assert_eq!(board.selected, Some(CardId::Note("note:n2".into())));
        board.set_cards(Vec::new());
        assert_eq!(board.selected, None);
    }

    #[derive(Clone, Debug)]
    enum Op {
        Down(isize),
        Across(isize),
        MoveSelected(isize),
        Drag(usize, usize),
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            (-2isize..=2).prop_map(Op::Down),
            (-2isize..=2).prop_map(Op::Across),
            prop_oneof![Just(-1isize), Just(1)].prop_map(Op::MoveSelected),
            (0usize..8, 0usize..5).prop_map(|(c, l)| Op::Drag(c, l)),
        ]
    }

    fn store_with_notes(statuses: &[usize]) -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        store
            .put(
                DocType::Book,
                json!({"_id": "book:a", "name": "A"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        for (ix, status) in statuses.iter().enumerate() {
            store
                .put(
                    DocType::Note,
                    json!({"_id": format!("note:n{ix}"), "title": format!("N{ix}"),
                        "body": format!("body {ix}\n- [ ] a\n"), "bookId": "book:a",
                        "tags": [], "status": NoteStatus::ALL[*status].as_str()}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
        (dir, store)
    }

    proptest! {
        #![proptest_config(config())]

        /// Model-based: any keys, moves and drags on a board of notes,
        /// written to the store as they happen. The board always holds the
        /// same notes, each in exactly one column (its status's); a move
        /// changes only the moved note's status, in the board and in the
        /// store, and the store's notes keep their titles, bodies and
        /// notebooks.
        #[test]
        fn prop_moves_change_only_the_status(
            statuses in prop::collection::vec(0usize..5, 1..8),
            ops in prop::collection::vec(op(), 0..30),
        ) {
            let (_dir, store) = store_with_notes(&statuses);
            let mut board = notes_board(&statuses);
            let mut model: Vec<NoteStatus> = statuses.iter().map(|s| NoteStatus::ALL[*s]).collect();
            for op in ops {
                let moved = match op {
                    Op::Down(d) => { board.step_down(d); None }
                    Op::Across(d) => { board.step_across(d); None }
                    Op::MoveSelected(d) => board.move_selected(d),
                    Op::Drag(card, lane) => {
                        let id = CardId::Note(format!("note:n{}", card % statuses.len()));
                        board.move_card(&id, Lane::Status(NoteStatus::ALL[lane]))
                    }
                };
                if let Some(mv) = &moved {
                    let CardId::Note(id) = &mv.id else { unreachable!() };
                    let ix: usize = id["note:n".len()..].parse().unwrap();
                    prop_assert_eq!(mv.from, Lane::Status(model[ix]));
                    prop_assert_ne!(mv.from, mv.to);
                    let Lane::Status(to) = mv.to else { unreachable!() };
                    model[ix] = to;
                    prop_assert_eq!(write_move(&store, mv, 5).unwrap(), Written::Done);
                }
                // The same notes, each once, in its own column.
                let columns = board.columns();
                let mut seen: Vec<usize> = columns.iter().flat_map(|(_, c)| c.clone()).collect();
                seen.sort_unstable();
                prop_assert_eq!(seen, (0..statuses.len()).collect::<Vec<_>>());
                for (lane, cards) in &columns {
                    for &ix in cards {
                        prop_assert_eq!(board.cards[ix].lane(), *lane);
                    }
                }
                for (ix, status) in model.iter().enumerate() {
                    prop_assert_eq!(status_of(&board, ix), *status);
                    let stored = store.note(&format!("note:n{ix}")).unwrap().unwrap();
                    prop_assert_eq!(stored.status, *status);
                    prop_assert_eq!(stored.title, format!("N{ix}"));
                    prop_assert_eq!(stored.body, format!("body {ix}\n- [ ] a\n"));
                    prop_assert_eq!(stored.book_id, "book:a");
                }
                // The selection is a card on the board, or nothing.
                if let Some(id) = &board.selected {
                    prop_assert!(board.position(id).is_some());
                }
            }
        }

        /// Model-based, for tasks: dragging between To do and Done ticks
        /// the task's box in its note and nothing else in the note changes;
        /// read again, the board has the same tasks, each in the column
        /// its box says.
        #[test]
        fn prop_task_moves_tick_only_the_box(
            checked in prop::collection::vec(any::<bool>(), 1..7),
            drags in prop::collection::vec((0usize..7, any::<bool>()), 0..16),
        ) {
            let body: String = checked.iter().enumerate()
                .map(|(i, c)| format!("- [{}] task {i}\n", if *c { "x" } else { " " }))
                .collect();
            let (_dir, store) = store_with_notes(&[0]);
            store.put(DocType::Note, json!({"_id": "note:n0", "body": body}), Put::Merge, Origin::Ui).unwrap();
            let read = |store: &Store| {
                let notes = vec![note(0, NoteStatus::None)];
                let rows = store.tasks(&TaskQuery { include_done: true, ..Default::default() }).unwrap();
                cards(BoardKind::Tasks, notes, rows)
            };
            let mut board = Board::new(BoardKind::Tasks);
            board.set_cards(read(&store));
            let mut model = checked.clone();
            for (task, done) in drags {
                let task = task % checked.len();
                let id = board.cards.iter().map(Card::id)
                    .find(|id| matches!(id, CardId::Task(t) if t.text == format!("task {task}")))
                    .unwrap();
                let to = if done { Lane::Done } else { Lane::Todo };
                if let Some(mv) = board.move_card(&id, to) {
                    prop_assert_eq!(write_move(&store, &mv, 9).unwrap(), Written::Done);
                    model[task] = done;
                }
                board.set_cards(read(&store));
                prop_assert_eq!(board.cards.len(), checked.len());
                let expected: String = model.iter().enumerate()
                    .map(|(i, c)| format!("- [{}] task {i}\n", if *c { "x" } else { " " }))
                    .collect();
                prop_assert_eq!(store.note("note:n0").unwrap().unwrap().body, expected);
                for card in &board.cards {
                    let Card::Task(row) = card else { unreachable!() };
                    let i: usize = row.text["task ".len()..].parse().unwrap();
                    prop_assert_eq!(card.lane(), if model[i] { Lane::Done } else { Lane::Todo });
                }
            }
        }
    }

    /// The task board takes only the tasks of the notes the list shows.
    #[test]
    fn task_cards_follow_the_listed_notes() {
        let (_dir, store) = store_with_notes(&[0, 1, 2]);
        let rows = store
            .tasks(&TaskQuery {
                include_done: true,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(rows.len(), 3);
        let listed = vec![note(0, NoteStatus::None), note(2, NoteStatus::None)];
        let cards = cards(BoardKind::Tasks, listed, rows);
        let notes: Vec<&str> = cards.iter().map(Card::note_id).collect();
        assert_eq!(notes, ["note:n0", "note:n2"]);
        // A task gone from its note is reported, not guessed at.
        let Card::Task(row) = &cards[0] else {
            unreachable!()
        };
        let mut gone = crate::tasks::TaskId::of(row);
        gone.text = "not there".into();
        let mv = Move {
            id: CardId::Task(gone),
            from: Lane::Todo,
            to: Lane::Done,
        };
        assert_eq!(write_move(&store, &mv, 1).unwrap(), Written::Moved);
    }
}
