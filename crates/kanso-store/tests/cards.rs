//! Flashcards in the store: the derived `note_cards` table, sessions and
//! limits, answers in `review` documents, their union merge across two
//! devices, and what never reaches a card (locked, trashed, excluded from
//! the API).

use kanso_core::srs::Grade;
use kanso_core::{DocType, TRASH_BOOK_ID, cards};
use kanso_store::{ApplyOutcome, CardScope, CardSettings, Origin, Put, Store, SyncDoc};
use proptest::prelude::*;
use serde_json::{Value, json};

const DAY: i64 = 86_400_000;
const T0: i64 = 1_793_000_000_000;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn put(store: &Store, doc_type: DocType, body: Value) -> String {
    store
        .put(doc_type, body, Put::Upsert, Origin::Ui)
        .unwrap()
        .id
}

fn book(store: &Store) -> String {
    put(store, DocType::Book, json!({"name": "Study"}))
}

fn note(store: &Store, book: &str, body: &str) -> String {
    put(
        store,
        DocType::Note,
        json!({"title": "Deck", "body": body, "bookId": book}),
    )
}

fn lock_envelope() -> Value {
    json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"})
}

fn keys(store: &Store, note_id: &str) -> Vec<String> {
    store
        .note_cards(note_id)
        .unwrap()
        .into_iter()
        .map(|c| c.card_key)
        .collect()
}

#[test]
fn cards_are_derived_from_the_body_and_follow_edits() {
    let (_d, store) = open();
    let b = book(&store);
    let n = note(
        &store,
        &b,
        "Q one :: A one\n\nQ two :: A two\nQ one :: again\n",
    );
    let found = store.note_cards(&n).unwrap();
    assert_eq!(found.len(), 2, "a repeated question is one card");
    assert_eq!(found[0].question, "Q one");
    assert_eq!(found[0].line, 0);
    assert_eq!(found[0].card_key, cards::card_key(&n, "Q one"));
    // Editing only an answer keeps the key.
    put_body(&store, &n, "Q one :: changed\nQ two :: A two\n");
    assert_eq!(
        keys(&store, &n),
        vec![cards::card_key(&n, "Q one"), cards::card_key(&n, "Q two")]
    );
    // Editing a question makes a new card.
    put_body(&store, &n, "Q uno :: A\nQ two :: A two\n");
    assert_ne!(keys(&store, &n)[0], cards::card_key(&n, "Q one"));
    // Deleting the line removes the card.
    put_body(&store, &n, "Q two :: A two\n");
    assert_eq!(store.note_cards(&n).unwrap().len(), 1);
}

fn put_body(store: &Store, id: &str, body: &str) {
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": body}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
}

#[test]
fn a_locked_note_has_no_cards() {
    let (_d, store) = open();
    let b = book(&store);
    let n = note(&store, &b, "CANARY question :: CANARY answer\n");
    assert_eq!(store.note_cards(&n).unwrap().len(), 1);
    let mut sealed = store.get_api(&n, None, false).unwrap();
    sealed["body"] = "".into();
    sealed["kansoLock"] = lock_envelope();
    store.lock_note(sealed).unwrap();
    assert!(store.note_cards(&n).unwrap().is_empty());
    assert!(!store.has_cards().unwrap());
    let all = store.due_cards(&CardScope::All, T0, T0).unwrap();
    assert!(all.is_empty());
}

#[test]
fn a_note_in_a_locked_notebook_has_no_cards() {
    let (_d, store) = open();
    let b = book(&store);
    let n = note(&store, &b, "Secret q :: Secret a\n");
    assert_eq!(store.note_cards(&n).unwrap().len(), 1);
    let mut locked = store.get_api(&b, None, false).unwrap();
    locked.as_object_mut().unwrap().remove("count");
    locked["kansoLock"] = lock_envelope();
    store
        .put(DocType::Book, locked, Put::Upsert, Origin::Sync)
        .unwrap();
    // The derived rows may still be there until the note is next
    // written; nothing may show or take an answer from them.
    assert!(store.note_cards(&n).unwrap().is_empty());
    assert!(store.due_cards(&CardScope::All, T0, T0).unwrap().is_empty());
    let key = cards::card_key(&n, "Secret q");
    assert!(
        store
            .record_review(&n, &key, Grade::Good, "d1", T0)
            .is_err()
    );
}

#[test]
fn trashed_notes_cards_are_not_due() {
    let (_d, store) = open();
    let b = book(&store);
    let n = note(&store, &b, "Q :: A\n");
    assert_eq!(store.due_cards(&CardScope::All, T0, T0).unwrap().len(), 1);
    store
        .put(
            DocType::Note,
            json!({"_id": n, "bookId": TRASH_BOOK_ID}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert!(store.due_cards(&CardScope::All, T0, T0).unwrap().is_empty());
    assert_eq!(store.card_counts(&CardScope::All, T0).unwrap().total, 0);
}

#[test]
fn answers_schedule_cards_and_limits_hold() {
    let (_d, store) = open();
    let b = book(&store);
    let body: String = (0..5).map(|i| format!("Q{i} :: A{i}\n")).collect();
    let n = note(&store, &b, &body);
    store
        .set_card_settings(&CardSettings {
            new_per_day: 2,
            reviews_per_day: 3,
            ..CardSettings::default()
        })
        .unwrap();
    let day = T0 - T0 % DAY;
    let first = store.due_cards(&CardScope::All, T0, day).unwrap();
    assert_eq!(first.len(), 2, "two new cards a day");
    assert!(first.iter().all(|c| c.is_new));
    for c in &first {
        let state = store
            .record_review(&n, &c.card_key, Grade::Easy, "dev", T0)
            .unwrap();
        assert!(state.due_ms > T0 + 10 * DAY);
    }
    // Their limit is used up today; they come back when due.
    assert!(
        store
            .due_cards(&CardScope::All, T0, day)
            .unwrap()
            .is_empty()
    );
    let later = day + 20 * DAY;
    let again = store.due_cards(&CardScope::All, later, later).unwrap();
    assert_eq!(again.len(), 2 + 2, "two reviews and two more new cards");
    assert!(again[0].state.reps > 0 && !again[0].is_new);
    assert_eq!(
        store.cards_waiting(&CardScope::All, later, later).unwrap(),
        4
    );
    let counts = store.card_counts(&CardScope::All, later).unwrap();
    assert_eq!((counts.total, counts.due, counts.new), (5, 2, 3));
    // Intervals preview per grade, never decreasing.
    let iv = store.card_intervals(&first[0].card_key, later).unwrap();
    assert!(iv[0] <= iv[1] && iv[1] <= iv[2] && iv[2] <= iv[3]);
}

#[test]
fn daily_limits_hold_across_devices() {
    let (_d, a) = open();
    let (_e, b) = open();
    let book_a = book(&a);
    let body: String = (0..4).map(|i| format!("Q{i} :: A{i}\n")).collect();
    let n = note(&a, &book_a, &body);
    // The second device has the same note and receives device A's answers.
    let doc = a.get(&n).unwrap().unwrap();
    let _ = doc;
    let settings = CardSettings {
        new_per_day: 2,
        ..CardSettings::default()
    };
    a.set_card_settings(&settings).unwrap();
    let day = T0 - T0 % DAY;
    let shown = a.due_cards(&CardScope::All, T0, day).unwrap();
    assert_eq!(shown.len(), 2);
    for c in &shown {
        a.record_review(&n, &c.card_key, Grade::Good, "A", T0)
            .unwrap();
    }
    // Send A's review documents to B, which holds the same note.
    copy_all(&a, &b);
    b.set_card_settings(&settings).unwrap();
    assert!(
        b.due_cards(&CardScope::All, T0 + 1, day)
            .unwrap()
            .iter()
            .all(|c| !c.is_new)
    );
}

/// Gives `to` every document of `from`, as the sync engine would.
fn copy_all(from: &Store, to: &Store) {
    for doc in from.sync_docs().unwrap() {
        to.apply_remote(&doc, None, None).unwrap();
    }
}

#[test]
fn two_devices_reviewing_offline_merge_without_conflict() {
    let (_d, a) = open();
    let (_e, b) = open();
    let bk = book(&a);
    let n = note(&a, &bk, "Q :: A\nR :: B\n");
    copy_all(&a, &b);
    let q = cards::card_key(&n, "Q");
    let r = cards::card_key(&n, "R");
    // Each reviews a different card offline, the same month.
    a.record_review(&n, &q, Grade::Good, "A", T0).unwrap();
    b.record_review(&n, &r, Grade::Hard, "B", T0 + 5).unwrap();
    let a_docs: Vec<SyncDoc> = a
        .sync_docs()
        .unwrap()
        .into_iter()
        .filter(|d| d.id.starts_with("review:"))
        .collect();
    let b_docs: Vec<SyncDoc> = b
        .sync_docs()
        .unwrap()
        .into_iter()
        .filter(|d| d.id.starts_with("review:"))
        .collect();
    assert_eq!((a_docs.len(), b_docs.len()), (1, 1));
    assert_eq!(
        a_docs[0].id, b_docs[0].id,
        "the same month, the same document"
    );
    // Each side receives the other's version.
    let on_a = a.apply_remote(&b_docs[0], None, None).unwrap();
    let on_b = b.apply_remote(&a_docs[0], None, None).unwrap();
    assert!(matches!(on_a, ApplyOutcome::LocalWon { .. }), "{on_a:?}");
    assert!(matches!(on_b, ApplyOutcome::LocalWon { .. }), "{on_b:?}");
    for store in [&a, &b] {
        let counts = store.card_counts(&CardScope::All, T0 + DAY * 365).unwrap();
        assert_eq!(counts.new, 0, "both answers are in the log of both");
        assert_eq!(counts.total, 2);
    }
    // And the merged documents are equal.
    let fields = |s: &Store| {
        s.sync_docs()
            .unwrap()
            .into_iter()
            .find(|d| d.id.starts_with("review:"))
            .unwrap()
            .fields
    };
    assert_eq!(fields(&a), fields(&b));
}

#[test]
fn suspending_hides_a_card_until_resumed() {
    let (_d, store) = open();
    let bk = book(&store);
    let n = note(&store, &bk, "Q :: A\n");
    let key = cards::card_key(&n, "Q");
    store.suspend_card(&n, &key, true, T0).unwrap();
    assert!(store.due_cards(&CardScope::All, T0, T0).unwrap().is_empty());
    assert_eq!(store.card_counts(&CardScope::All, T0).unwrap().suspended, 1);
    store.suspend_card(&n, &key, false, T0 + 1).unwrap();
    assert_eq!(
        store.due_cards(&CardScope::All, T0 + 2, T0).unwrap().len(),
        1
    );
}

#[test]
fn deleting_a_note_and_resetting_remove_the_log() {
    let (_d, store) = open();
    let bk = book(&store);
    let n = note(&store, &bk, "Q :: A\n");
    let m = note(&store, &bk, "R :: B\n");
    store
        .record_review(&n, &cards::card_key(&n, "Q"), Grade::Good, "A", T0)
        .unwrap();
    store
        .record_review(&m, &cards::card_key(&m, "R"), Grade::Good, "A", T0)
        .unwrap();
    let live = |s: &Store| {
        s.sync_docs()
            .unwrap()
            .into_iter()
            .filter(|d| d.id.starts_with("review:") && !d.deleted)
            .count()
    };
    assert_eq!(live(&store), 2);
    store.delete(&n, Origin::Ui).unwrap();
    assert_eq!(live(&store), 1);
    assert_eq!(store.reset_review_history().unwrap(), 1);
    assert_eq!(live(&store), 0);
    assert_eq!(store.card_counts(&CardScope::All, T0).unwrap().new, 1);
}

#[test]
fn review_documents_are_not_served_by_the_api() {
    let (_d, store) = open();
    let bk = book(&store);
    let n = note(&store, &bk, "Q :: A\n");
    store
        .record_review(&n, &cards::card_key(&n, "Q"), Grade::Good, "dev-1", T0)
        .unwrap();
    let id = store
        .sync_docs()
        .unwrap()
        .into_iter()
        .find(|d| d.id.starts_with("review:"))
        .unwrap()
        .id;
    assert!(store.get_api(&id, None, false).is_err());
    assert!(store.list_api(DocType::Review, None, 0).unwrap().is_empty());
    assert_eq!(DocType::from_collection("reviews"), None);
    // A review log holds card keys and times, never a question or answer.
    let doc = store.get(&id).unwrap().unwrap();
    let text = serde_json::to_string(&doc.fields).unwrap();
    assert!(!text.contains("\"Q\"") && !text.contains("\"A\"") && !text.contains("B\""));
}

#[test]
fn stats_count_the_log() {
    let (_d, store) = open();
    let bk = book(&store);
    let n = note(&store, &bk, "Q :: A\nR :: B\n");
    let day = T0 - T0 % DAY;
    let q = cards::card_key(&n, "Q");
    store.record_review(&n, &q, Grade::Again, "A", T0).unwrap();
    store
        .record_review(&n, &q, Grade::Good, "A", T0 + 1000)
        .unwrap();
    store
        .record_review(&n, &cards::card_key(&n, "R"), Grade::Good, "A", T0 + 2000)
        .unwrap();
    let stats = store.card_stats(&CardScope::All, T0 + 5000, day).unwrap();
    assert_eq!(stats.per_day.len(), 30);
    assert_eq!(stats.per_day[29], 3);
    let retention = stats.retention.unwrap();
    assert!((retention - 2.0 / 3.0).abs() < 1e-9);
    assert_eq!(stats.hardest.len(), 1);
    assert_eq!(stats.hardest[0].agains, 1);
    assert_eq!(stats.forecast.len(), 7);
    assert_eq!(stats.counts.total, 2);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// The derived table always equals what `card_lines` finds in the body
    /// (deduplicated by key), for any body.
    #[test]
    fn prop_note_cards_match_card_lines(
        lines in prop::collection::vec("[A-Za-z0-9 ?:\\-`|>.]{0,30}", 0..12),
    ) {
        let (_d, store) = open();
        let bk = book(&store);
        let body = lines.join("\n");
        let n = note(&store, &bk, &body);
        let mut seen = std::collections::HashSet::new();
        let want: Vec<(String, String)> = cards::card_lines(&body)
            .into_iter()
            .filter(|c| seen.insert(cards::card_key(&n, &c.question)))
            .map(|c| (c.question, c.answer))
            .collect();
        let got: Vec<(String, String)> = store
            .note_cards(&n)
            .unwrap()
            .into_iter()
            .map(|c| (c.question, c.answer))
            .collect();
        prop_assert_eq!(got, want);
    }
}
