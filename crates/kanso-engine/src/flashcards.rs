//! Flashcards on the model's side, shared by the desktop and the phones:
//! suggested cards for a note, Quiz Me, and the daily reminder's words.
//!
//! The cards themselves live in note bodies (`kanso_core::cards`) and their
//! schedule in the store (`kanso_store::cards`); nothing here is needed to
//! review. What is here needs AI, and reads only the notes it is given
//! through the one eligibility rule: a locked note, a note in a locked
//! notebook, a trashed note or an `#ai-exclude` note never reaches a prompt.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kanso_ai::draft::{self, QUIZ_QUESTIONS};
use kanso_ai::draft_model::{Graded, QuizQuestion, Suggestion};
use kanso_ai::features::{MAX_CHUNKS, SUMMARY_CHUNK, chunks};
use kanso_ai::{ChatProvider, Event, Excerpt, Prompt};
use kanso_core::cards;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::ask;
use crate::assist::KEPT_OUT;

/// The most excerpts a quiz round draws its questions from.
pub const QUIZ_EXCERPTS: usize = 8;
/// Suggested cards asked for at once.
pub const SUGGEST_CARDS: usize = 8;

/// Asks `prompt` and returns the whole reply.
fn ask_once(
    provider: &dyn ChatProvider,
    prompt: &Prompt,
    cancel: &Arc<AtomicBool>,
) -> Result<String, String> {
    let mut text = String::new();
    let mut failed = None;
    kanso_ai::features::relay(provider.stream(prompt), cancel, &mut |event| match event {
        Event::Text(t) => text.push_str(&t),
        Event::Error(message) => failed = Some(message),
        Event::Status(_) | Event::Done => {}
    });
    match failed {
        Some(message) => Err(message),
        None => Ok(text),
    }
}

/// Cards the model suggests for note `id`, with the note's own lines
/// checked against each card's quote. Only that note is sent. Refuses with
/// [`KEPT_OUT`] when the note may not be sent.
pub fn suggest(
    store: &Store,
    provider: &dyn ChatProvider,
    id: &str,
    cancel: &Arc<AtomicBool>,
) -> Result<Vec<Suggestion>, String> {
    let note = ask::eligible_note(store, id)
        .ok()
        .flatten()
        .ok_or_else(|| KEPT_OUT.to_owned())?;
    if note.body.trim().is_empty() {
        return Err("This note is empty.".into());
    }
    let parts = chunks(&note.body, SUMMARY_CHUNK);
    let mut all: Vec<Suggestion> = Vec::new();
    for part in parts.iter().take(MAX_CHUNKS) {
        if cancel.load(Ordering::Relaxed) {
            return Ok(Vec::new());
        }
        let reply = ask_once(
            provider,
            &draft::cards_prompt(&note.title, part, SUGGEST_CARDS),
            cancel,
        )?;
        // The quote is looked for in the whole note, not just this part.
        all.extend(draft::parse_suggestions(&reply, &note.body, SUGGEST_CARDS));
    }
    let mut seen = std::collections::HashSet::new();
    all.retain(|s| seen.insert(cards::normalize(&s.question)));
    // Not those the note already has.
    let have: std::collections::HashSet<String> = cards::card_lines(&note.body)
        .iter()
        .map(|c| cards::normalize(&c.question))
        .collect();
    all.retain(|s| !have.contains(&cards::normalize(&s.question)));
    Ok(all)
}

/// Why cards cannot be added to a note, or `None` when they can.
pub fn cannot_add(store: &Store, id: &str) -> Option<&'static str> {
    if store.note_is_locked(id).unwrap_or(true) {
        return Some("This note is locked.");
    }
    match store.note(id) {
        Ok(Some(note)) if !note.is_trashed() => None,
        _ => Some("This note isn't there."),
    }
}

/// The note's body with `new` added under `## Flashcards`, and the old
/// body, for the review sheet's diff. Nothing is written.
pub fn preview_add(
    store: &Store,
    id: &str,
    new: &[(String, String)],
) -> Result<(String, String), String> {
    if let Some(why) = cannot_add(store, id) {
        return Err(why.into());
    }
    let note = store
        .note(id)
        .map_err(|e| e.to_string())?
        .ok_or("This note isn't there.")?;
    Ok((note.body.clone(), cards::append_cards(&note.body, new)))
}

/// Writes `new` cards into note `id` under `## Flashcards`: one edit of
/// the note, so one undo step. Cards the note already has are skipped.
/// Returns how many were added.
pub fn add_cards(store: &Store, id: &str, new: &[(String, String)]) -> Result<usize, String> {
    let (old, _) = preview_add(store, id, new)?;
    let have: std::collections::HashSet<String> = cards::card_lines(&old)
        .iter()
        .map(|c| cards::normalize(&c.question))
        .collect();
    let fresh: Vec<(String, String)> = new
        .iter()
        .filter(|(q, _)| !have.contains(&cards::normalize(q)))
        .cloned()
        .collect();
    if fresh.is_empty() {
        return Ok(0);
    }
    let body = cards::append_cards(&old, &fresh);
    store
        .put(
            kanso_core::DocType::Note,
            json!({"_id": id, "body": body}),
            Put::Merge,
            Origin::Ui,
        )
        .map_err(|e| e.to_string())?;
    Ok(fresh.len())
}

/// The excerpts a quiz draws from: the notes of `ids` that may be sent,
/// the first [`QUIZ_EXCERPTS`] with some text.
pub fn quiz_excerpts(store: &Store, ids: &[String]) -> Vec<Excerpt> {
    let mut out = Vec::new();
    for id in ids {
        if let Ok(Some(c)) = ask::eligible_note(store, id)
            && !c.body.trim().is_empty()
        {
            out.push(Excerpt {
                id: c.id,
                title: c.title,
                text: c.body,
            });
        }
        if out.len() >= QUIZ_EXCERPTS {
            break;
        }
    }
    out
}

/// Five questions from `excerpts`.
pub fn quiz(
    provider: &dyn ChatProvider,
    excerpts: &[Excerpt],
    cancel: &Arc<AtomicBool>,
) -> Result<Vec<QuizQuestion>, String> {
    if excerpts.is_empty() {
        return Err(KEPT_OUT.into());
    }
    let reply = ask_once(
        provider,
        &draft::quiz_prompt(excerpts, QUIZ_QUESTIONS, ask::BUDGET),
        cancel,
    )?;
    let asked = draft::parse_quiz(&reply, excerpts.len(), QUIZ_QUESTIONS);
    if asked.is_empty() {
        return Err("The model didn't ask anything. Try again.".into());
    }
    Ok(asked)
}

/// Grades `answer` against the one excerpt the question came from. Nothing
/// is stored and the schedule is not touched.
pub fn grade(
    provider: &dyn ChatProvider,
    question: &str,
    excerpt: &Excerpt,
    answer: &str,
    cancel: &Arc<AtomicBool>,
) -> Result<Graded, String> {
    let reply = ask_once(
        provider,
        &draft::grade_prompt(question, excerpt, answer),
        cancel,
    )?;
    draft::parse_grade(&reply)
        .ok_or_else(|| "The model's grade couldn't be read. Try again.".into())
}

/// The daily reminder's words. A count, never a question or an answer, so a
/// locked screen shows nothing from the notes. `None` when nothing is due.
pub fn reminder_text(due: usize) -> Option<String> {
    match due {
        0 => None,
        1 => Some("1 flashcard is due.".into()),
        n => Some(format!("{n} flashcards are due.")),
    }
}

/// The start of the person's day (ms) for a clock reading `now_ms` and a
/// zone `offset_minutes` east of UTC.
pub fn day_start_ms(now_ms: i64, offset_minutes: i32) -> i64 {
    let offset = i64::from(offset_minutes) * 60_000;
    let day = 86_400_000;
    (now_ms + offset).div_euclid(day) * day - offset
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use kanso_ai::AiError;
    use kanso_core::DocType;
    use proptest::prelude::*;

    use super::*;

    struct Says {
        reply: String,
        prompts: Mutex<Vec<Prompt>>,
    }

    impl ChatProvider for Says {
        fn stream(
            &self,
            prompt: &Prompt,
        ) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
            self.prompts.lock().unwrap().push(prompt.clone());
            Ok(Box::new(
                vec![Event::Text(self.reply.clone()), Event::Done].into_iter(),
            ))
        }
    }

    fn says(reply: &str) -> Says {
        Says {
            reply: reply.into(),
            prompts: Mutex::new(Vec::new()),
        }
    }

    fn library() -> (tempfile::TempDir, Store, String) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Study"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        (dir, store, book)
    }

    fn note(store: &Store, book: &str, title: &str, body: &str) -> String {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn lock(store: &Store, id: &str) {
        let mut sealed = store.get_api(id, None, false).unwrap();
        sealed["body"] = "".into();
        sealed["kansoLock"] =
            json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
        store.lock_note(sealed).unwrap();
    }

    #[test]
    fn suggestions_come_from_the_one_note_and_quotes_are_checked() {
        let (_d, store, book) = library();
        let id = note(
            &store,
            &book,
            "FSRS",
            "# Week 3\n\nFSRS is a scheduler.\nIt predicts recall.\nRomans :: Roma\n",
        );
        let other = note(&store, &book, "Other", "CANARY other note text");
        let model = says(
            "{\"q\":\"What is FSRS?\",\"a\":\"A scheduler\",\"source\":\"FSRS is a scheduler\"}\n{\"q\":\"Romans\",\"a\":\"dup\"}\n{\"q\":\"Ghost?\",\"a\":\"x\",\"source\":\"nowhere\"}",
        );
        let cancel = Arc::new(AtomicBool::new(false));
        let got = suggest(&store, &model, &id, &cancel).unwrap();
        assert_eq!(
            got.len(),
            2,
            "a question the note already has is not offered again"
        );
        assert!(!got[0].source_missing && got[0].line == Some(2));
        assert!(got[1].source_missing);
        let sent = model.prompts.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert!(sent[0].user.contains("FSRS is a scheduler"));
        assert!(!sent[0].user.contains("CANARY") && !sent[0].system.contains("CANARY"));
        let _ = other;
    }

    #[test]
    fn a_locked_or_excluded_note_is_never_in_a_prompt() {
        let (_d, store, book) = library();
        let secret = note(
            &store,
            &book,
            "Secret",
            "CANARY locked text :: CANARY answer",
        );
        lock(&store, &secret);
        let tag = store
            .put(
                DocType::Tag,
                json!({"name": "ai-exclude"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let hidden = store
            .put(
                DocType::Note,
                json!({"title": "Hidden", "body": "CANARY excluded text", "bookId": book, "tags": [tag]}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let model = says("{\"q\":\"x\",\"a\":\"y\"}");
        let cancel = Arc::new(AtomicBool::new(false));
        assert_eq!(
            suggest(&store, &model, &secret, &cancel).unwrap_err(),
            KEPT_OUT
        );
        assert_eq!(
            suggest(&store, &model, &hidden, &cancel).unwrap_err(),
            KEPT_OUT
        );
        assert!(quiz_excerpts(&store, &[secret.clone(), hidden.clone()]).is_empty());
        assert!(quiz(&model, &[], &cancel).is_err());
        assert!(model.prompts.lock().unwrap().is_empty(), "nothing was sent");
        // The explicit card of the excluded note still reviews: it never leaves.
        assert_eq!(store.note_cards(&hidden).unwrap().len(), 0);
    }

    #[test]
    fn adding_cards_is_one_edit_under_the_flashcards_heading() {
        let (_d, store, book) = library();
        let id = note(&store, &book, "N", "text\n\nOld :: card\n");
        let new = vec![
            ("New one".to_owned(), "A".to_owned()),
            ("Old".to_owned(), "again".to_owned()),
            ("New two".to_owned(), "B :: odd".to_owned()),
        ];
        let (before, after) = preview_add(&store, &id, &new).unwrap();
        assert_eq!(before, "text\n\nOld :: card\n");
        assert!(after.contains("## Flashcards"));
        let revs_before = store.revisions(&id).unwrap().len();
        assert_eq!(add_cards(&store, &id, &new).unwrap(), 2);
        let body = store.note(&id).unwrap().unwrap().body;
        let found: Vec<String> = cards::card_lines(&body)
            .into_iter()
            .map(|c| c.question)
            .collect();
        assert_eq!(found, vec!["Old", "New one", "New two"]);
        assert!(
            store.revisions(&id).unwrap().len() <= revs_before + 1,
            "one step"
        );
        assert_eq!(store.note_cards(&id).unwrap().len(), 3);
        assert_eq!(add_cards(&store, &id, &new).unwrap(), 0, "nothing twice");
        // A locked note takes no cards.
        lock(&store, &id);
        assert!(add_cards(&store, &id, &new).is_err());
    }

    #[test]
    fn quiz_grading_sees_only_the_source_excerpt() {
        let model = says("Partly: the note says \"x\".");
        let cancel = Arc::new(AtomicBool::new(false));
        let one = Excerpt {
            id: "note:a".into(),
            title: "A".into(),
            text: "alpha fact".into(),
        };
        let g = grade(&model, "Why?", &one, "because", &cancel).unwrap();
        assert_eq!(g.verdict, kanso_ai::draft_model::Verdict::Partly);
        let sent = model.prompts.lock().unwrap();
        assert!(sent[0].user.contains("alpha fact") && !sent[0].user.contains("beta"));
    }

    // no-prop-test: two fixed sentences.
    #[test]
    fn the_reminder_says_a_count_and_nothing_else() {
        assert_eq!(reminder_text(0), None);
        assert_eq!(reminder_text(1).unwrap(), "1 flashcard is due.");
        assert_eq!(reminder_text(8).unwrap(), "8 flashcards are due.");
    }

    proptest! {
        /// The reminder never holds a card's words: only the count.
        #[test]
        fn notification_text_has_no_card_words(n in 1usize..100_000) {
            let text = reminder_text(n).unwrap();
            let allowed = ["flashcard", "flashcards", "is", "are", "due."];
            for word in text.split_whitespace() {
                prop_assert!(word.parse::<usize>().is_ok() || allowed.contains(&word), "{word}");
            }
        }

        /// The day starts at local midnight for any zone and time.
        #[test]
        fn prop_day_start_is_midnight(now in 0i64..4_000_000_000_000, zone in -720i32..840) {
            let start = day_start_ms(now, zone);
            prop_assert!(start <= now && now - start < 86_400_000);
            prop_assert_eq!((start + i64::from(zone) * 60_000).rem_euclid(86_400_000), 0);
        }
    }
}
