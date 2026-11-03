//! The logic of a review session on the desktop (issue #198), apart from
//! the window: which card is up, whether its answer is showing, what an
//! answer does to the queue, and the keys.
//!
//! The cards and their schedule are the store's (`kanso_store`'s `cards`);
//! this keeps only the order they are shown in during one sitting.

use std::collections::VecDeque;

use kanso_core::srs::interval_label;
use kanso_store::DueCard;

/// The four answers, in the order of their keys.
pub const GRADES: [&str; 4] = ["Again", "Hard", "Good", "Easy"];

/// What a key does in the review. Pure, so the keys can be tested.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Space or Return: show the answer.
    ShowAnswer,
    /// 1 to 4 (or Space for Good once the answer shows).
    Answer(u8),
    /// E: open the source note at the card's line.
    Edit,
    /// S: hide the card until it is resumed.
    Suspend,
    /// Tab: between Review and Stats.
    ToggleStats,
    Close,
}

/// The command for a key, if it is one, given whether the answer shows.
pub fn command_for(key: &str, answer_shown: bool) -> Option<Command> {
    Some(match key {
        "space" | "enter" if !answer_shown => Command::ShowAnswer,
        "space" | "enter" => Command::Answer(3),
        "1" if answer_shown => Command::Answer(1),
        "2" if answer_shown => Command::Answer(2),
        "3" if answer_shown => Command::Answer(3),
        "4" if answer_shown => Command::Answer(4),
        "e" => Command::Edit,
        "s" => Command::Suspend,
        "tab" => Command::ToggleStats,
        "escape" => Command::Close,
        _ => return None,
    })
}

/// One sitting: the cards still to show, and how many were answered.
#[derive(Clone, Debug, Default)]
pub struct Session {
    queue: VecDeque<DueCard>,
    answered: usize,
    total: usize,
    shown: bool,
}

impl Session {
    pub fn new(cards: Vec<DueCard>) -> Session {
        Session {
            total: cards.len(),
            queue: cards.into(),
            answered: 0,
            shown: false,
        }
    }

    pub fn current(&self) -> Option<&DueCard> {
        self.queue.front()
    }

    pub fn is_done(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn answered(&self) -> usize {
        self.answered
    }

    /// Whether the answer of the current card shows.
    pub fn answer_shown(&self) -> bool {
        self.shown
    }

    pub fn show_answer(&mut self) {
        if !self.queue.is_empty() {
            self.shown = true;
        }
    }

    /// "12 of 40 · 3 new".
    pub fn progress(&self) -> String {
        let fresh = self.queue.iter().filter(|c| c.is_new).count();
        let at = (self.answered + 1).min(self.total.max(1));
        if fresh > 0 {
            format!("{at} of {} · {fresh} new", self.total)
        } else {
            format!("{at} of {}", self.total)
        }
    }

    /// The current card was answered with `grade` (1 to 4) and the store
    /// has recorded it. A card answered Again comes back at the end of this
    /// sitting. Returns the card that was answered.
    pub fn answered_with(&mut self, grade: u8) -> Option<DueCard> {
        let card = self.queue.pop_front()?;
        self.answered += 1;
        self.shown = false;
        if grade == 1 {
            self.queue.push_back(card.clone());
            self.total += 1;
        }
        Some(card)
    }

    /// The current card was suspended: it leaves the sitting.
    pub fn suspended(&mut self) -> Option<DueCard> {
        let card = self.queue.pop_front()?;
        self.total = self.total.saturating_sub(1);
        self.shown = false;
        Some(card)
    }
}

/// The interval under each answer button: "Good · 4 days".
pub fn button_labels(spans: &[i64; 4]) -> [String; 4] {
    std::array::from_fn(|i| format!("{} · {}", GRADES[i], interval_label(spans[i])))
}

#[cfg(test)]
mod tests {
    use kanso_core::srs::CardState;
    use proptest::prelude::*;

    use super::*;

    fn card(n: usize, is_new: bool) -> DueCard {
        DueCard {
            card_key: format!("k{n}"),
            note_id: "note:a".into(),
            note_title: "Deck".into(),
            line: n as u32,
            question: format!("Q{n}"),
            answer: format!("A{n}"),
            is_new,
            state: CardState::new(),
        }
    }

    #[test]
    fn keys_map_to_commands() {
        assert_eq!(command_for("space", false), Some(Command::ShowAnswer));
        assert_eq!(command_for("space", true), Some(Command::Answer(3)));
        assert_eq!(command_for("1", true), Some(Command::Answer(1)));
        assert_eq!(command_for("4", true), Some(Command::Answer(4)));
        assert_eq!(command_for("4", false), None, "no answer before the reveal");
        assert_eq!(command_for("e", false), Some(Command::Edit));
        assert_eq!(command_for("escape", true), Some(Command::Close));
        assert_eq!(command_for("q", true), None);
    }

    #[test]
    fn answering_moves_on_and_again_comes_back() {
        let mut s = Session::new(vec![card(0, true), card(1, false)]);
        assert_eq!(s.progress(), "1 of 2 · 1 new");
        s.show_answer();
        assert!(s.answer_shown());
        assert_eq!(s.answered_with(1).unwrap().card_key, "k0");
        assert!(!s.answer_shown(), "the next card starts hidden");
        assert_eq!(s.current().unwrap().card_key, "k1");
        s.answered_with(3);
        assert_eq!(s.current().unwrap().card_key, "k0", "Again returns last");
        s.answered_with(4);
        assert!(s.is_done());
        assert_eq!(s.answered(), 3);
    }

    #[test]
    fn labels_carry_the_intervals() {
        let labels = button_labels(&[600_000, 86_400_000, 4 * 86_400_000, 14 * 86_400_000]);
        assert_eq!(labels[0], "Again · 10 min");
        assert_eq!(labels[2], "Good · 4 days");
    }

    proptest! {
        /// Whatever is answered, the sitting ends, never shows a card after
        /// it is done, and answers count every card at least once.
        #[test]
        fn prop_a_sitting_always_ends(grades in prop::collection::vec(1u8..=4, 1..60), n in 1usize..8) {
            let mut s = Session::new((0..n).map(|i| card(i, i % 2 == 0)).collect());
            for (steps, g) in grades.iter().cycle().take(500).enumerate() {
                if s.is_done() { break; }
                // Only the first few answers may be Again, so it ends.
                let g = if steps > 40 { 3 } else { *g };
                s.answered_with(g);
            }
            prop_assert!(s.is_done());
            prop_assert!(s.answered() >= n);
            prop_assert!(s.current().is_none());
        }
    }
}
