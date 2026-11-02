//! What a person has done in Triage so far, apart from drawing it: which
//! chip is accepted, rejected or still untouched on each card, which card
//! is skipped, and what "File" and "File all accepted" would apply. Nothing
//! is written here; filing is [`super::file`].

use std::collections::HashSet;

use super::model::{Card, Field, Filing, Reason, Suggestion};

/// Where a chip stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mark {
    /// Outlined: neither accepted nor rejected.
    #[default]
    Untouched,
    /// Filled: filing applies it.
    Accepted,
    /// Struck through and faded: remembered, so it doesn't come back.
    Rejected,
}

/// One chip on a card.
#[derive(Clone, Debug, PartialEq)]
pub struct Chip {
    pub suggestion: Suggestion,
    pub mark: Mark,
    /// Whether the person changed the value.
    pub edited: bool,
}

/// A card with its chips.
#[derive(Clone, Debug, PartialEq)]
pub struct CardState {
    pub note_id: String,
    pub rev: String,
    pub chips: Vec<Chip>,
    pub missing: Option<Reason>,
    /// The note changed elsewhere: the card is recomputed and File waits.
    pub changed: bool,
    /// Hidden for this session.
    pub skipped: bool,
    /// Why the AI's suggestions are missing, if they are.
    pub ai_error: Option<String>,
    /// The library's candidates, for refining and for editing.
    pub candidates: Vec<Suggestion>,
}

impl CardState {
    pub fn new(card: Card) -> CardState {
        CardState {
            note_id: card.note_id,
            rev: card.rev,
            chips: card
                .suggestions
                .into_iter()
                .map(|suggestion| Chip {
                    suggestion,
                    mark: Mark::Untouched,
                    edited: false,
                })
                .collect(),
            missing: card.missing,
            changed: false,
            skipped: false,
            ai_error: None,
            candidates: card.candidates,
        }
    }

    /// Accepts chip `ix`, or lifts the acceptance if it already is. A
    /// notebook accepted un-accepts another notebook.
    pub fn toggle_accept(&mut self, ix: usize) {
        let Some(field) = self.chips.get(ix).map(|c| c.suggestion.field) else {
            return;
        };
        if self.chips[ix].mark == Mark::Accepted {
            self.chips[ix].mark = Mark::Untouched;
            return;
        }
        self.chips[ix].mark = Mark::Accepted;
        if matches!(field, Field::Notebook | Field::Title) {
            for (other, chip) in self.chips.iter_mut().enumerate() {
                if other != ix && chip.suggestion.field == field && chip.mark == Mark::Accepted {
                    chip.mark = Mark::Untouched;
                }
            }
        }
    }

    /// Rejects chip `ix`, or lifts the rejection if it already is.
    /// Returns the suggestion newly rejected (to be remembered), or the one
    /// whose rejection was lifted, with whether it is now rejected.
    pub fn toggle_reject(&mut self, ix: usize) -> Option<(Suggestion, bool)> {
        let chip = self.chips.get_mut(ix)?;
        chip.mark = if chip.mark == Mark::Rejected {
            Mark::Untouched
        } else {
            Mark::Rejected
        };
        Some((chip.suggestion.clone(), chip.mark == Mark::Rejected))
    }

    /// Accepts every chip that isn't rejected: one notebook and one title
    /// at most (the first of each).
    pub fn accept_all(&mut self) {
        let mut book = false;
        let mut title = false;
        for chip in &mut self.chips {
            if chip.mark == Mark::Rejected {
                continue;
            }
            let seen = match chip.suggestion.field {
                Field::Notebook => &mut book,
                Field::Title => &mut title,
                _ => {
                    chip.mark = Mark::Accepted;
                    continue;
                }
            };
            chip.mark = if *seen {
                Mark::Untouched
            } else {
                Mark::Accepted
            };
            *seen = true;
        }
    }

    /// Accepts every chip of `field` that isn't rejected (the first, for a
    /// notebook or title).
    pub fn accept_field(&mut self, field: Field) {
        let mut done = false;
        for chip in &mut self.chips {
            if chip.suggestion.field != field || chip.mark == Mark::Rejected {
                continue;
            }
            if matches!(field, Field::Notebook | Field::Title) {
                chip.mark = if done {
                    Mark::Untouched
                } else {
                    Mark::Accepted
                };
                done = true;
            } else {
                chip.mark = Mark::Accepted;
            }
        }
    }

    /// Changes chip `ix` to `suggestion` (what the person chose or typed)
    /// and accepts it. Another chip of the same value is dropped.
    pub fn edit(&mut self, ix: usize, suggestion: Suggestion) {
        if ix >= self.chips.len() {
            return;
        }
        let (field, value) = (suggestion.field, suggestion.value.clone());
        self.chips[ix] = Chip {
            suggestion,
            mark: Mark::Accepted,
            edited: true,
        };
        let mut at = 0;
        self.chips.retain(|chip| {
            let keep =
                at == ix || !(chip.suggestion.field == field && chip.suggestion.value == value);
            at += 1;
            keep
        });
    }

    /// Adds a chip the person made (a tag they typed, a note they picked),
    /// accepted. Nothing is added when the field's value is already there.
    pub fn add(&mut self, suggestion: Suggestion) {
        if let Some(chip) = self.chips.iter_mut().find(|c| {
            c.suggestion.field == suggestion.field && c.suggestion.value == suggestion.value
        }) {
            chip.mark = Mark::Accepted;
            return;
        }
        self.chips.push(Chip {
            suggestion,
            mark: Mark::Accepted,
            edited: true,
        });
    }

    /// Replaces the chips with `suggestions` (an AI refinement or a
    /// recomputed card), keeping the marks of those that are still there.
    pub fn replace_suggestions(&mut self, suggestions: Vec<Suggestion>) {
        let old = std::mem::take(&mut self.chips);
        self.chips = suggestions
            .into_iter()
            .map(|suggestion| {
                let mark = old
                    .iter()
                    .find(|c| {
                        c.suggestion.field == suggestion.field
                            && c.suggestion.value == suggestion.value
                    })
                    .map_or(Mark::Untouched, |c| c.mark);
                Chip {
                    suggestion,
                    mark,
                    edited: false,
                }
            })
            .collect();
    }

    /// The accepted suggestions, in the card's order.
    pub fn accepted(&self) -> Vec<Suggestion> {
        self.chips
            .iter()
            .filter(|c| c.mark == Mark::Accepted)
            .map(|c| c.suggestion.clone())
            .collect()
    }

    /// Whether anything is accepted.
    pub fn has_accepted(&self) -> bool {
        self.chips.iter().any(|c| c.mark == Mark::Accepted)
    }

    /// What "File" applies for this card. `None` while the card says the
    /// note changed.
    pub fn filing(&self) -> Option<Filing> {
        (!self.changed).then(|| Filing {
            note_id: self.note_id.clone(),
            rev: self.rev.clone(),
            accepted: self.accepted(),
        })
    }
}

/// The cards of a Triage session, and which one has the keys.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Session {
    pub cards: Vec<CardState>,
    pub focus: usize,
    /// Notes filed or put away in this session, so a reload doesn't bring
    /// them back.
    pub done: HashSet<String>,
}

impl Session {
    pub fn new(cards: Vec<Card>) -> Session {
        Session {
            cards: cards.into_iter().map(CardState::new).collect(),
            focus: 0,
            done: HashSet::new(),
        }
    }

    /// The cards still showing: not skipped.
    pub fn shown(&self) -> Vec<usize> {
        self.cards
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.skipped)
            .map(|(i, _)| i)
            .collect()
    }

    /// The card with the keys, if any shows.
    pub fn focused(&self) -> Option<usize> {
        let shown = self.shown();
        shown
            .contains(&self.focus)
            .then_some(self.focus)
            .or_else(|| shown.first().copied())
    }

    /// Moves the keys `by` cards among those shown, stopping at the ends.
    pub fn step(&mut self, by: isize) {
        let shown = self.shown();
        let Some(at) = self
            .focused()
            .and_then(|f| shown.iter().position(|s| *s == f))
        else {
            return;
        };
        let to = (at as isize + by).clamp(0, shown.len() as isize - 1) as usize;
        self.focus = shown[to];
    }

    /// Hides the focused card for this session and moves on.
    pub fn skip_focused(&mut self) {
        if let Some(ix) = self.focused() {
            self.cards[ix].skipped = true;
            let shown = self.shown();
            self.focus = shown
                .iter()
                .find(|s| **s > ix)
                .or_else(|| shown.last())
                .copied()
                .unwrap_or(0);
        }
    }

    /// Ticks every notebook (or tag) chip on every card shown.
    pub fn accept_everywhere(&mut self, field: Field) {
        for card in self.cards.iter_mut().filter(|c| !c.skipped) {
            card.accept_field(field);
        }
    }

    /// What "File all accepted" applies: the cards shown with something
    /// accepted.
    pub fn filings(&self) -> Vec<Filing> {
        self.cards
            .iter()
            .filter(|c| !c.skipped && c.has_accepted())
            .filter_map(CardState::filing)
            .collect()
    }

    /// Takes filed notes off the list.
    pub fn remove(&mut self, ids: &[String]) {
        self.cards.retain(|c| !ids.contains(&c.note_id));
        self.done.extend(ids.iter().cloned());
        self.focus = self.focus.min(self.cards.len().saturating_sub(1));
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::Source;
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn s(field: Field, value: &str) -> Suggestion {
        Suggestion {
            field,
            value: value.into(),
            label: value.into(),
            score: 0.5,
            because: vec![],
            source: Source::Library,
            is_new: false,
        }
    }

    fn card(id: &str, chips: &[(Field, &str)]) -> Card {
        let suggestions: Vec<Suggestion> = chips.iter().map(|(f, v)| s(*f, v)).collect();
        Card {
            note_id: id.into(),
            rev: "1-a".into(),
            candidates: suggestions.clone(),
            suggestions,
            missing: None,
        }
    }

    #[test]
    fn accept_all_leaves_one_notebook_and_skips_rejected() {
        let mut c = CardState::new(card(
            "n",
            &[
                (Field::Notebook, "b1"),
                (Field::Notebook, "b2"),
                (Field::Tag, "t1"),
                (Field::Tag, "t2"),
                (Field::Link, "l"),
            ],
        ));
        c.toggle_reject(3);
        c.accept_all();
        let got: Vec<_> = c.accepted().into_iter().map(|s| s.value).collect();
        assert_eq!(got, ["b1", "t1", "l"]);
        // Accepting the other notebook swaps.
        c.toggle_accept(1);
        let got: Vec<_> = c.accepted().into_iter().map(|s| s.value).collect();
        assert_eq!(got, ["b2", "t1", "l"]);
        assert_eq!(c.chips[0].mark, Mark::Untouched);
    }

    #[test]
    fn rejecting_reports_what_to_remember() {
        let mut c = CardState::new(card("n", &[(Field::Tag, "t1")]));
        let (got, now) = c.toggle_reject(0).unwrap();
        assert_eq!((got.value.as_str(), now), ("t1", true));
        let (_, now) = c.toggle_reject(0).unwrap();
        assert!(!now);
        assert!(c.toggle_reject(9).is_none());
    }

    #[test]
    fn a_changed_card_cannot_be_filed() {
        let mut c = CardState::new(card("n", &[(Field::Tag, "t1")]));
        c.accept_all();
        assert!(c.filing().is_some());
        c.changed = true;
        assert!(c.filing().is_none());
    }

    #[test]
    fn the_session_steps_skips_and_files_what_is_accepted() {
        let mut session = Session::new(vec![
            card("a", &[(Field::Tag, "t")]),
            card("b", &[(Field::Tag, "t")]),
            card("c", &[(Field::Tag, "t")]),
        ]);
        session.cards[0].accept_all();
        session.cards[2].accept_all();
        session.step(1);
        assert_eq!(session.focused(), Some(1));
        session.skip_focused();
        assert_eq!(session.focused(), Some(2));
        session.step(5);
        assert_eq!(session.focused(), Some(2));
        let ids: Vec<_> = session.filings().into_iter().map(|f| f.note_id).collect();
        assert_eq!(ids, ["a", "c"]);
        session.remove(&["a".to_owned(), "c".to_owned()]);
        assert_eq!(session.cards.len(), 1);
        assert!(session.done.contains("a"));
        session.accept_everywhere(Field::Tag);
        assert!(session.filings().is_empty(), "b is skipped");
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Whatever is tapped, a rejected chip is never in what is filed,
        /// and at most one notebook and one title are.
        #[test]
        fn filing_never_includes_a_rejected_chip(
            fields in prop::collection::vec(0usize..4, 1..8),
            taps in prop::collection::vec((0usize..4, 0usize..9), 0..20),
        ) {
            let chips: Vec<(Field, String)> = fields
                .iter()
                .enumerate()
                .map(|(i, f)| (Field::ALL[*f], format!("v{i}")))
                .collect();
            let refs: Vec<(Field, &str)> = chips.iter().map(|(f, v)| (*f, v.as_str())).collect();
            let mut c = CardState::new(card("n", &refs));
            for (kind, ix) in taps {
                match kind {
                    0 => c.toggle_accept(ix),
                    1 => { c.toggle_reject(ix); }
                    2 => c.accept_all(),
                    _ => c.accept_field(Field::ALL[ix % 4]),
                }
            }
            let accepted = c.accepted();
            for chip in &c.chips {
                if chip.mark == Mark::Rejected {
                    prop_assert!(!accepted.iter().any(|a| a.value == chip.suggestion.value));
                }
            }
            for field in [Field::Notebook, Field::Title] {
                prop_assert!(accepted.iter().filter(|a| a.field == field).count() <= 1);
            }
        }
    }
}
