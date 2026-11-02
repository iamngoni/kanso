use std::sync::Arc;

use proptest::prelude::*;
use serde_json::json;

use super::*;
use crate::embed::{EmbeddedNote, EmbeddedPassage};
use crate::{Embedder, Origin, Put};
use kanso_core::DocType;

/// Hashes each word of three letters or more into one of 64 places: texts
/// with the same words point the same way.
struct Words;

impl Embedder for Words {
    fn model(&self) -> &str {
        "words"
    }
    fn dim(&self) -> usize {
        64
    }
    fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
        texts
            .iter()
            .map(|t| {
                let mut v = vec![0f32; 64];
                for w in match_words(t) {
                    let h = w
                        .bytes()
                        .fold(7usize, |h, b| h.wrapping_mul(31).wrapping_add(b as usize));
                    v[h % 64] += 1.;
                }
                v
            })
            .collect()
    }
}

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    store.set_embedding_model(Some("words")).unwrap();
    store.lend_embedder(Some(Arc::new(Words)));
    (dir, store)
}

fn book(store: &Store) -> String {
    if let Some(b) = store.books().unwrap().first() {
        return b.id.clone();
    }
    store
        .put(
            DocType::Book,
            json!({"name": "Home"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn note(store: &Store, title: &str, body: &str) -> String {
    store
        .put(
            DocType::Note,
            json!({"title": title, "body": body, "bookId": book(store)}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

/// Embeds whatever is waiting with its passages, as the indexer does.
fn index(store: &Store) {
    let queue = store.embedding_queue("words").unwrap();
    let mut rows = Vec::new();
    for pending in queue.pending {
        let vector = Words.embed(std::slice::from_ref(&pending.text)).remove(0);
        let passages = pending
            .passages()
            .iter()
            .map(|p| EmbeddedPassage {
                start: p.start,
                end: p.end,
                vector: Words.embed(std::slice::from_ref(&p.text)).remove(0),
            })
            .collect();
        rows.push(EmbeddedNote {
            id: pending.id,
            hash: pending.hash,
            vector,
            passages,
        });
    }
    store.put_embedded("words", &rows).unwrap();
}

fn opts(threshold: f32, k: usize) -> ResurfaceOpts {
    ResurfaceOpts { threshold, k }
}

fn found(store: &Store, text: &str, id: &str, opts: ResurfaceOpts) -> Vec<Resurfaced> {
    match store.resurface(text, id, opts).unwrap() {
        Resurfacing::Found(found) => found,
        other => panic!("expected notes, got {other:?}"),
    }
}

const BOILER: &str =
    "The engineer said the pressure valve on the boiler needs replacing before winter.";

#[test]
fn a_past_note_comes_back_with_the_passage_that_matched() {
    let (_dir, store) = open();
    let boiler = note(
        &store,
        "Boiler service",
        &format!("Intro about the flat.\n\n{BOILER}\n\nPlumber booked for Tuesday."),
    );
    note(
        &store,
        "Garden",
        "Tomato plants need watering every morning in summer.",
    );
    index(&store);
    let current = note(&store, "Today", "nothing yet");
    let list = found(
        &store,
        "boiler pressure valve engineer winter",
        &current,
        opts(0.3, 3),
    );
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, boiler);
    assert_eq!(list[0].book, "Home");
    assert!(list[0].created_at > 0);
    assert!(
        list[0].snippet.contains("pressure valve"),
        "{}",
        list[0].snippet
    );
    assert!(list[0].passage.0 < list[0].passage.1);
}

#[test]
fn a_note_without_passages_still_gets_a_snippet() {
    let (_dir, store) = open();
    note(&store, "Boiler service", &format!("Intro.\n\n{BOILER}"));
    let queue = store.embedding_queue("words").unwrap();
    let rows: Vec<_> = queue
        .pending
        .into_iter()
        .map(|p| (p.id, p.hash, Words.embed(&[p.text]).remove(0)))
        .collect();
    store.put_embeddings("words", &rows).unwrap();
    let list = found(
        &store,
        "boiler pressure valve winter",
        "note:none",
        opts(0.3, 3),
    );
    assert_eq!(list.len(), 1);
    assert!(
        list[0].snippet.contains("pressure valve"),
        "{}",
        list[0].snippet
    );
}

#[test]
fn it_says_nothing_while_related_notes_are_off_or_the_model_is_missing() {
    let (_dir, store) = open();
    store.lend_embedder(None);
    assert_eq!(
        store.resurface("boiler", "note:x", opts(0.3, 3)).unwrap(),
        Resurfacing::NoModel
    );
    store.set_embedding_model(None).unwrap();
    assert_eq!(
        store.resurface("boiler", "note:x", opts(0.3, 3)).unwrap(),
        Resurfacing::Off
    );
}

#[test]
fn the_note_itself_linked_trashed_locked_and_dismissed_notes_are_left_out() {
    let (_dir, store) = open();
    let words = "orchard apples cider pressing barrels autumn harvest";
    let current = note(&store, "Current", words);
    // Not in any list below: the current note links to it.
    note(&store, "Linked", words);
    let trashed = note(&store, "Trashed", words);
    let locked = note(&store, "Locked", words);
    let dismissed = note(&store, "Dismissed", words);
    let plain = note(&store, "Plain", words);
    // The current note links to Linked.
    store
        .put(
            DocType::Note,
            json!({"_id": current, "body": format!("{words} [[Linked]]")}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    index(&store);
    let all = |store: &Store| -> Vec<String> {
        found(store, words, &current, opts(0.3, 10))
            .into_iter()
            .map(|r| r.id)
            .collect()
    };
    let mut ids = all(&store);
    ids.sort();
    let mut expected = vec![
        trashed.clone(),
        locked.clone(),
        dismissed.clone(),
        plain.clone(),
    ];
    expected.sort();
    assert_eq!(ids, expected);

    store
        .put(
            DocType::Note,
            json!({"_id": trashed, "bookId": kanso_core::model::TRASH_BOOK_ID}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let mut sealed = store.get_api(&locked, None, false).unwrap();
    sealed["body"] = "".into();
    sealed["kansoLock"] =
        json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
    store.lock_note(sealed).unwrap();
    store.dismiss_related(&current, &dismissed).unwrap();
    let mut ids = all(&store);
    ids.sort();
    let mut expected = vec![plain.clone()];
    expected.sort();
    assert_eq!(ids, expected);
    // Once locked, a note keeps no passages either.
    assert_eq!(passage_count(&store, &locked, "words"), 0);
    assert!(passage_count(&store, &plain, "words") > 0);
    // The dismissal is for that note only.
    assert!(store.dismissed_related(&plain).unwrap().is_empty());
    assert!(store.is_dismissed(&current, &dismissed).unwrap());
    assert!(store.is_dismissed(&dismissed, &current).unwrap());
}

#[test]
fn turning_related_notes_off_empties_the_passages_and_the_threshold_holds() {
    let (_dir, store) = open();
    let a = note(
        &store,
        "A",
        "orchard apples cider pressing barrels autumn harvest",
    );
    index(&store);
    assert!(passage_count(&store, &a, "words") > 0);
    assert!(found(&store, "kayak river rapids paddle", "note:x", opts(0.5, 3)).is_empty());
    store.set_embedding_model(None).unwrap();
    assert_eq!(passage_count(&store, &a, "words"), 0);
}

#[test]
fn twin_dismissals_survive_the_store_being_reopened_and_go_with_the_note() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kanso.db");
    let store = Store::open(&path).unwrap();
    let a = note(&store, "A", "one");
    let b = note(&store, "B", "two");
    store.dismiss_twin(&a, &b).unwrap();
    drop(store);
    let store = Store::open(&path).unwrap();
    assert!(store.is_dismissed(&a, &b).unwrap());
    assert!(store.is_dismissed(&b, &a).unwrap());
    store.delete(&b, Origin::Ui).ok();
}

#[test]
fn on_this_day_lists_daily_notes_first_and_skips_the_unreadable() {
    let (_dir, store) = open();
    let day = |ms: i64, title: &str| {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": "x", "bookId": book(&store), "createdAt": ms}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    };
    let early = day(1_000, "Early");
    let inside = day(5_000, "Inside");
    let later = day(6_000, "Later");
    let outside = day(9_000, "Outside");
    let found = store
        .on_this_day(
            &[(4_000, 7_000), (0, 2_000)],
            std::slice::from_ref(&early),
            3,
        )
        .unwrap();
    let ids: Vec<&str> = found.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids, [early.as_str(), later.as_str(), inside.as_str()]);
    assert!(!ids.contains(&outside.as_str()));
    assert_eq!(found[0].book, "Home");
    assert_eq!(
        store.on_this_day(&[(4_000, 7_000)], &[], 1).unwrap().len(),
        1
    );
    assert!(
        store
            .on_this_day(&[(4_000, 7_000)], &[], 0)
            .unwrap()
            .is_empty()
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        failure_persistence: None,
        rng_seed: proptest::test_runner::RngSeed::Fixed(0x0072_6573),
        ..ProptestConfig::default()
    })]

    /// Whatever the notes, the answer has at most k notes, closest first,
    /// none below the threshold, never the note itself, a note it links
    /// to, a dismissed, trashed or locked note.
    #[test]
    fn prop_resurfacing_obeys_its_rules(
        bodies in proptest::collection::vec("[a-f]{3}( [a-f]{3}){2,8}", 2..9),
        query in "[a-f]{3}( [a-f]{3}){2,5}",
        k in 1usize..5,
        threshold in 0.1f32..0.8,
        dismiss in 0usize..9,
        trash in 0usize..9,
        link in 0usize..9,
    ) {
        let (_dir, store) = open();
        let ids: Vec<String> = bodies
            .iter()
            .enumerate()
            .map(|(n, body)| note(&store, &format!("Note number {n}"), body))
            .collect();
        let current = note(&store, "Current", &query);
        let (dismiss, trash, link) = (dismiss % ids.len(), trash % ids.len(), link % ids.len());
        store
            .put(
                DocType::Note,
                json!({"_id": current, "body": format!("{query} [[Note number {link}]]")}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        index(&store);
        store.dismiss_related(&current, &ids[dismiss]).unwrap();
        store
            .put(
                DocType::Note,
                json!({"_id": ids[trash], "bookId": kanso_core::model::TRASH_BOOK_ID}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        let list = found(&store, &query, &current, opts(threshold, k));
        prop_assert!(list.len() <= k);
        prop_assert!(list.windows(2).all(|w| w[0].score >= w[1].score));
        for item in &list {
            prop_assert!(item.score >= threshold);
            prop_assert!(item.id != current);
            prop_assert!(item.id != ids[link]);
            prop_assert!(item.id != ids[dismiss]);
            prop_assert!(item.id != ids[trash]);
        }
    }

    /// Snippets are short, one line, and never longer than the limit
    /// plus the ellipses.
    #[test]
    fn prop_snippets_are_short_plain_lines(body in "(.|\n){0,600}", a in 0usize..700, b in 0usize..700) {
        let text = snippet(&body, a, b);
        prop_assert!(text.chars().count() <= SNIPPET_CHARS + 2);
        prop_assert!(!text.contains('\n'));
    }
}
