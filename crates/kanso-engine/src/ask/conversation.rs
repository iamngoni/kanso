//! The rules of an Ask conversation that have nothing to do with a window.
//!
//! A conversation is kept in memory only. It ends when what it was asked
//! under changes (another provider, URL or model), an exchange is dropped
//! when a note it drew on stops being eligible, and what is resent to the
//! model is capped by characters, oldest first. The desktop panel and the
//! phones' [`Conversation`] share the pieces here.

use kanso_ai::Excerpt;
use kanso_store::Store;

use super::note_is_eligible;

/// How many characters of earlier exchanges a follow-up may resend. The
/// oldest turns are dropped first.
pub const HISTORY_BUDGET: usize = 12_000;

/// One completed exchange: what was asked, what was shown, and the notes it
/// drew on, so a later question can drop it if one of them is no longer
/// eligible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskedTurn {
    pub question: String,
    pub answer: String,
    pub notes: Vec<String>,
}

impl AskedTurn {
    pub fn chars(&self) -> usize {
        self.question.chars().count() + self.answer.chars().count()
    }
}

/// Notes the answer being written read: every one goes on this turn's list,
/// even a note an earlier turn read too, so the turn is dropped if any of
/// them stops being eligible. `used` keeps one excerpt per note for links.
pub fn record_turn_notes(turn_notes: &mut Vec<String>, used: &mut Vec<Excerpt>, new: Vec<Excerpt>) {
    for excerpt in new {
        if !turn_notes.contains(&excerpt.id) {
            turn_notes.push(excerpt.id.clone());
        }
        if !used.iter().any(|seen| seen.id == excerpt.id) {
            used.push(excerpt);
        }
    }
}

/// The turns whose notes may all still be sent. An error in the eligibility
/// check drops the turn too (fail closed).
pub fn eligible_turns(turns: Vec<AskedTurn>, store: &Store) -> Vec<AskedTurn> {
    turns
        .into_iter()
        .filter(|turn| turn.notes.iter().all(|id| note_is_eligible(store, id)))
        .collect()
}

/// Keeps the newest turns that fit `budget` characters, dropping the oldest.
/// The newest turn is always kept, even if it alone is over budget, so a
/// follow-up still has the exchange it follows.
pub fn cap_history(mut turns: Vec<AskedTurn>, budget: usize) -> Vec<AskedTurn> {
    let mut kept = Vec::new();
    let mut used = 0;
    while let Some(turn) = turns.pop() {
        let size = turn.chars();
        if !kept.is_empty() && used + size > budget {
            break;
        }
        used += size;
        kept.push(turn);
    }
    kept.reverse();
    kept
}

/// An Ask conversation: the exchanges so far, the notes they drew on, and
/// what it was asked under.
#[derive(Debug, Default)]
pub struct Conversation {
    /// What the conversation was asked under (provider, URL and model), if it
    /// has started. A different one starts it over.
    identity: Option<String>,
    turns: Vec<AskedTurn>,
    /// One excerpt per note any turn read, for linking citations.
    used: Vec<Excerpt>,
}

impl Conversation {
    /// Gets the conversation ready for a question asked under `identity`. A
    /// different identity forgets every exchange; then exchanges whose notes
    /// may no longer be sent are dropped and the history is capped. Returns
    /// whether the conversation was started over.
    pub fn begin(&mut self, identity: &str, store: &Store) -> bool {
        let restarted = self
            .identity
            .as_deref()
            .is_some_and(|last| last != identity);
        if restarted {
            self.clear();
        }
        self.identity = Some(identity.to_owned());
        self.turns = eligible_turns(std::mem::take(&mut self.turns), store);
        self.turns = cap_history(std::mem::take(&mut self.turns), HISTORY_BUDGET);
        self.used
            .retain(|excerpt| note_is_eligible(store, &excerpt.id));
        restarted
    }

    /// The exchanges to resend with the next question.
    pub fn history(&self) -> Vec<kanso_ai::Turn> {
        self.turns
            .iter()
            .map(|turn| kanso_ai::Turn {
                question: turn.question.clone(),
                answer: turn.answer.clone(),
            })
            .collect()
    }

    /// Keeps a finished exchange. An empty answer is not kept.
    pub fn finish(&mut self, question: String, answer: String, notes: Vec<String>) {
        if !answer.trim().is_empty() {
            self.turns.push(AskedTurn {
                question,
                answer,
                notes,
            });
        }
    }

    /// Remembers the excerpts a turn drew on (one per note, for links) and
    /// adds their ids to `turn_notes`.
    pub fn note_used(&mut self, turn_notes: &mut Vec<String>, new: Vec<Excerpt>) {
        record_turn_notes(turn_notes, &mut self.used, new);
    }

    /// Every note any turn drew on, once each.
    pub fn used(&self) -> &[Excerpt] {
        &self.used
    }

    /// The exchanges kept.
    pub fn turns(&self) -> &[AskedTurn] {
        &self.turns
    }

    /// Forgets everything.
    pub fn clear(&mut self) {
        self.identity = None;
        self.turns.clear();
        self.used.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::DocType;
    use kanso_store::{Origin, Put};
    use serde_json::json;

    fn turn(question: &str, answer: &str, notes: Vec<String>) -> AskedTurn {
        AskedTurn {
            question: question.to_owned(),
            answer: answer.to_owned(),
            notes,
        }
    }

    fn pricing() -> (tempfile::TempDir, Store, String) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Notes"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let note = store
            .put(
                DocType::Note,
                json!({"title": "Pricing", "body": "The plan costs 40.", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        (dir, store, note)
    }

    #[test]
    fn history_is_capped_by_dropping_the_oldest_turns() {
        let turns = vec![
            turn("old", &"a".repeat(60), vec![]),
            turn("middle", &"b".repeat(60), vec![]),
            turn("new", &"c".repeat(60), vec![]),
        ];
        let kept = cap_history(turns, 140);
        let questions: Vec<&str> = kept.iter().map(|turn| turn.question.as_str()).collect();
        assert_eq!(questions, ["middle", "new"]);
        assert!(kept.iter().map(AskedTurn::chars).sum::<usize>() <= 140);
        assert!(cap_history(vec![], 10).is_empty());
    }

    #[test]
    fn a_turn_whose_note_is_locked_is_dropped() {
        let (_dir, store, note) = pricing();
        let turns = vec![turn("what?", "the plan", vec![note.clone()])];
        assert_eq!(eligible_turns(turns.clone(), &store).len(), 1);
        store
            .put(
                DocType::Note,
                json!({"_id": note, "kansoLock": {"salt": "abc"}}),
                Put::Merge,
                Origin::Sync,
            )
            .unwrap();
        assert!(eligible_turns(turns, &store).is_empty());
    }

    #[test]
    fn a_note_read_again_is_on_every_turn_that_read_it() {
        let excerpt = |id: &str| Excerpt {
            id: id.to_owned(),
            title: String::new(),
            text: String::new(),
        };
        let mut used = Vec::new();
        let mut first = Vec::new();
        record_turn_notes(
            &mut first,
            &mut used,
            vec![excerpt("note:a"), excerpt("note:b")],
        );
        let mut second = Vec::new();
        record_turn_notes(
            &mut second,
            &mut used,
            vec![excerpt("note:a"), excerpt("note:a")],
        );
        assert_eq!(first, ["note:a", "note:b"]);
        assert_eq!(second, ["note:a"], "the second turn read note:a too");
        assert_eq!(used.len(), 2, "one excerpt per note for links");
    }

    #[test]
    fn an_eligibility_error_drops_the_turn() {
        let (_dir, store, note) = pricing();
        {
            let conn = rusqlite::Connection::open(store.path()).unwrap();
            conn.execute("DROP VIEW locked_books", []).unwrap();
        }
        let turns = vec![turn("what?", "the plan", vec![note])];
        assert!(eligible_turns(turns, &store).is_empty());
    }

    #[test]
    fn the_history_budget_is_documented() {
        assert_eq!(HISTORY_BUDGET, 12_000);
    }

    #[test]
    fn a_different_identity_starts_the_conversation_over() {
        let (_dir, store, note) = pricing();
        let mut conversation = Conversation::default();
        assert!(!conversation.begin("local|http://a|m", &store));
        conversation.finish("q".into(), "a".into(), vec![note]);
        assert!(!conversation.begin("local|http://a|m", &store));
        assert_eq!(conversation.history().len(), 1);
        assert!(conversation.begin("openai|http://a|m", &store));
        assert!(conversation.history().is_empty());
    }

    #[test]
    fn a_locked_note_drops_its_turn_and_its_link() {
        let (_dir, store, note) = pricing();
        let mut conversation = Conversation::default();
        conversation.begin("x", &store);
        let mut notes = Vec::new();
        conversation.note_used(
            &mut notes,
            vec![Excerpt {
                id: note.clone(),
                title: "Pricing".into(),
                text: "t".into(),
            }],
        );
        conversation.finish("q".into(), "a".into(), notes);
        store
            .put(
                DocType::Note,
                json!({"_id": note, "kansoLock": {"salt": "abc"}}),
                Put::Merge,
                Origin::Sync,
            )
            .unwrap();
        conversation.begin("x", &store);
        assert!(conversation.turns().is_empty());
        assert!(conversation.used().is_empty());
    }
}
