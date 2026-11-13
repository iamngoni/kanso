//! Boards over the notes a list shows, and the chip a due date gets.

use flutter_rust_bridge::frb;
use kanso_mobile::{
    BoardKind, BoardLane, BoardView, CardKey, CardMoved, DueChipRow, KansoError, LaneChange,
    LaneEdited, LaneRemoval, Library, ListQuery, LocalTime,
};

use super::library::KansoLibrary;

impl KansoLibrary {
    pub fn board(&self, kind: BoardKind, query: ListQuery) -> Result<BoardView, KansoError> {
        self.0.board(kind, query)
    }

    pub fn move_card(
        &self,
        card: CardKey,
        to: BoardLane,
        now_ms: i64,
    ) -> Result<CardMoved, KansoError> {
        self.0.move_card(card, to, now_ms)
    }

    /// Groups notebook `book_id`'s board by a select property, or by status
    /// when `by` is `None`.
    pub fn set_board_group(&self, book_id: String, by: Option<String>) -> Result<(), KansoError> {
        self.0.set_board_group(&book_id, by)
    }

    /// Shows or hides the lane of notes with no value while it is empty.
    pub fn set_board_show_unset(&self, book_id: String, show: bool) -> Result<(), KansoError> {
        self.0.set_board_show_unset(&book_id, show)
    }

    /// What removing `lane` asks first: how many notes are in it, the words,
    /// and where they go by default.
    pub fn board_lane_removal(
        &self,
        book_id: String,
        lane: String,
    ) -> Result<LaneRemoval, KansoError> {
        self.0.board_lane_removal(&book_id, &lane)
    }

    /// Renames, reorders, adds or removes a lane of the notebook's board.
    pub fn edit_board_lanes(
        &self,
        book_id: String,
        change: LaneChange,
        now_ms: i64,
    ) -> Result<LaneEdited, KansoError> {
        self.0.edit_board_lanes(&book_id, change, now_ms)
    }

    #[frb(sync)]
    pub fn due_chip(
        due: String,
        checked: bool,
        now: LocalTime,
    ) -> Result<Option<DueChipRow>, KansoError> {
        Library::due_chip(&due, checked, now)
    }
}
