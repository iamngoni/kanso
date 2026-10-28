//! Boards over the notes a list shows, and the chip a due date gets.

use flutter_rust_bridge::frb;
use kanso_mobile::{
    BoardKind, BoardLane, BoardView, CardKey, CardMoved, DueChipRow, KansoError, Library,
    ListQuery, LocalTime,
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

    #[frb(sync)]
    pub fn due_chip(
        due: String,
        checked: bool,
        now: LocalTime,
    ) -> Result<Option<DueChipRow>, KansoError> {
        Library::due_chip(&due, checked, now)
    }
}
