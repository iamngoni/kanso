//! What Overview reads from the store (issue #240): where the person last
//! stopped writing, what the loop rules learn from dismissals, the people
//! someone is waiting on, and revisit dates. Locked notes never take part.

use kanso_core::DocType;
use kanso_core::loop_score::Family;
use kanso_core::loops::{LoopKind, loop_items, loop_key};
use kanso_core::revisit::Revisit;
use kanso_core::writing::{GAP_MS, MIN_CHARS};
use kanso_store::{LoopClose, Origin, Put, Store};
use proptest::prelude::*;
use serde_json::{Value, json};

const DAY: i64 = 86_400_000;
const NOW: i64 = 4_000_000_000_000;

fn temp_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn book(store: &Store, id: &str) {
    store
        .put(
            DocType::Book,
            json!({"_id": id, "name": id}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
}

fn note_at(store: &Store, id: &str, body: &str, book: &str, updated: i64, origin: Origin) {
    let current = store.get(id).unwrap().map(|d| d.rev);
    let mut doc = json!({"_id": id, "title": id, "body": body, "bookId": book,
        "createdAt": 1000, "updatedAt": updated});
    if let Some(rev) = current {
        doc["_rev"] = json!(rev);
    }
    store.put(DocType::Note, doc, Put::Upsert, origin).unwrap();
}

fn note(store: &Store, id: &str, body: &str) {
    note_at(store, id, body, "book:a", 1000, Origin::Ui);
}

fn lock() -> Value {
    json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"})
}

fn ids(rows: &[kanso_store::WritingPlace]) -> Vec<&str> {
    rows.iter().map(|r| r.note_id.as_str()).collect()
}

#[test]
fn a_writing_session_needs_forty_characters_or_two_minutes() {
    let (_dir, store) = temp_store();
    book(&store, "book:a");
    note(&store, "note:short", "Hello there");
    note(&store, "note:long", "Hello there");
    note(&store, "note:slow", "Hello there");
    store
        .note_writing("note:short", 0, 5, 12, "h", NOW)
        .unwrap();
    store
        .note_writing("note:long", 0, 5, MIN_CHARS, "h", NOW)
        .unwrap();
    store
        .note_writing("note:slow", 0, 5, 3, "h", NOW - 130_000)
        .unwrap();
    store.note_writing("note:slow", 0, 6, 3, "h", NOW).unwrap();
    let rows = store.pick_up_places(NOW, &[], 5).unwrap();
    let got = ids(&rows);
    assert!(got.contains(&"note:long"));
    assert!(got.contains(&"note:slow"));
    assert!(!got.contains(&"note:short"));
}

#[test]
fn typing_after_a_gap_starts_a_new_session() {
    let (_dir, store) = temp_store();
    book(&store, "book:a");
    note(&store, "note:a", "Hello there");
    store.note_writing("note:a", 0, 5, 30, "h", NOW).unwrap();
    // Within the gap: the session grows past the bar.
    store
        .note_writing("note:a", 0, 9, 30, "h", NOW + GAP_MS)
        .unwrap();
    assert_eq!(store.pick_up_places(NOW + GAP_MS, &[], 5).unwrap().len(), 1);
    // After the gap: a new session of 5 characters does not qualify.
    store
        .note_writing("note:a", 0, 9, 5, "h", NOW + 3 * GAP_MS)
        .unwrap();
    assert!(
        store
            .pick_up_places(NOW + 3 * GAP_MS, &[], 5)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn pick_up_leaves_out_daily_done_stale_hidden_and_excluded_notes() {
    let (_dir, store) = temp_store();
    book(&store, "book:a");
    for id in [
        "note:daily-2026-11-04",
        "note:done",
        "note:stale",
        "note:hidden",
        "note:open",
        "note:skip",
    ] {
        note(&store, id, "Some words to carry on with");
        store.note_writing(id, 0, 5, 80, "h", NOW).unwrap();
    }
    let doc = store.get("note:done").unwrap().unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": "note:done", "_rev": doc.rev, "title": "note:done",
                "body": "Some words to carry on with", "bookId": "book:a", "status": "completed",
                "createdAt": 1000, "updatedAt": 1000}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    store
        .note_writing("note:stale", 0, 5, 80, "h", NOW - 15 * DAY)
        .unwrap();
    store.hide_writing_place("note:hidden", NOW + 1).unwrap();
    let rows = store
        .pick_up_places(NOW + 5, &["note:skip".into()], 5)
        .unwrap();
    assert_eq!(ids(&rows), ["note:open"]);
    // Writing again brings a hidden one back.
    store
        .note_writing("note:hidden", 0, 6, 80, "h", NOW + 10)
        .unwrap();
    let again = store
        .pick_up_places(NOW + 20, &["note:skip".into()], 5)
        .unwrap();
    assert!(ids(&again).contains(&"note:hidden"));
}

#[test]
fn a_locked_note_has_no_place_and_forgets_the_one_it_had() {
    let (_dir, store) = temp_store();
    book(&store, "book:a");
    note(&store, "note:a", "A secret paragraph we typed");
    store.note_writing("note:a", 0, 5, 80, "h", NOW).unwrap();
    assert_eq!(store.pick_up_places(NOW, &[], 5).unwrap().len(), 1);
    store.set_meta("overview.snapshot", "{}").unwrap();
    let doc = store.get("note:a").unwrap().unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": "note:a", "_rev": doc.rev, "title": "note:a", "body": "",
                "bookId": "book:a", "kansoLock": lock(), "createdAt": 1000, "updatedAt": 2000}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    assert!(store.pick_up_places(NOW, &[], 5).unwrap().is_empty());
    assert_eq!(store.meta("overview.snapshot").unwrap(), None);
    store.note_writing("note:a", 0, 5, 80, "h", NOW).unwrap();
    assert!(store.pick_up_places(NOW, &[], 5).unwrap().is_empty());
}

#[test]
fn tagging_a_note_ai_exclude_drops_the_snapshot_and_the_ai_paragraph() {
    let (_dir, store) = temp_store();
    book(&store, "book:a");
    note(&store, "note:a", "Words a model summarised this morning");
    store
        .put(
            DocType::Tag,
            json!({"_id": "tag:x", "name": "ai-exclude"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    store
        .put(
            DocType::Tag,
            json!({"_id": "tag:y", "name": "work"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    let retag = |tag: &str| {
        let doc = store.get("note:a").unwrap().unwrap();
        let mut fields = doc.fields.clone();
        fields.insert("_id".into(), json!("note:a"));
        fields.insert("_rev".into(), json!(doc.rev));
        fields.insert("tags".into(), json!([tag]));
        store
            .put(
                DocType::Note,
                Value::Object(fields),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    };
    store.set_meta("overview.snapshot", "{}").unwrap();
    store
        .set_meta("overview.ai.today.2026-11-04", "{}")
        .unwrap();
    // Another tag keeps them.
    retag("tag:y");
    assert!(store.meta("overview.snapshot").unwrap().is_some());
    retag("tag:x");
    assert_eq!(store.meta("overview.snapshot").unwrap(), None);
    assert_eq!(store.meta("overview.ai.today.2026-11-04").unwrap(), None);
}

#[test]
fn a_session_on_another_device_is_placed_by_the_synced_change() {
    let (_dir, store) = temp_store();
    book(&store, "book:a");
    let now = kanso_core::now_ms();
    note_at(
        &store,
        "note:a",
        "First paragraph.\n\nSecond paragraph.",
        "book:a",
        now - 5000,
        Origin::Ui,
    );
    note_at(
        &store,
        "note:a",
        "First paragraph.\n\nSecond paragraph that the phone kept writing for a good while longer.",
        "book:a",
        now - 1000,
        Origin::Sync,
    );
    let rows = store.pick_up_places(now, &[], 5).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source, "sync");
    assert_eq!(rows[0].line, 2);
}

#[test]
fn two_dismissals_of_one_family_suppress_it() {
    let (_dir, store) = temp_store();
    book(&store, "book:a");
    let body = "I'll send the deck to Farai.\n\nI'll call Rudo about the quote.\n\nI'll book the venue for May.";
    note(&store, "note:a", body);
    let now = kanso_core::now_ms();
    assert!(store.suppressed_loop_families(now).unwrap().is_empty());
    for item in loop_items(body).into_iter().take(2) {
        store
            .close_loop(
                "note:a",
                &loop_key("note:a", &item.text),
                item.line,
                LoopClose::Dismissed,
                now,
                Origin::Ui,
            )
            .unwrap();
    }
    assert_eq!(store.suppressed_loop_families(now).unwrap(), [Family::Will]);
    store.clear_loop_feedback(Family::Will).unwrap();
    assert!(store.suppressed_loop_families(now).unwrap().is_empty());
    // A tick takes one back.
    let third = loop_items(body).into_iter().nth(2).unwrap();
    store
        .close_loop(
            "note:a",
            &loop_key("note:a", &third.text),
            third.line,
            LoopClose::Done {
                mark_in_note: false,
            },
            now,
            Origin::Ui,
        )
        .unwrap();
    assert!(store.suppressed_loop_families(now).unwrap().is_empty());
}

fn person(store: &Store, id: &str, name: &str) {
    let body = format!("---\nkanso: person\n---\n{name} page.\n");
    note_at(store, id, &body, "book:a", 1000, Origin::Ui);
    // The title is the name the mentions are found by.
    let doc = store.get(id).unwrap().unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": id, "_rev": doc.rev, "title": name, "body": body, "bookId": "book:a",
                "createdAt": 1000, "updatedAt": 1000}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
}

#[test]
fn waiting_on_a_person_with_a_page_shows_once_quiet_and_not_when_snoozed() {
    let (_dir, store) = temp_store();
    book(&store, "book:a");
    person(&store, "note:rudo", "Rudo");
    note(
        &store,
        "note:work",
        "Waiting on Rudo for the sign-off.\n\n- [ ] Get the keys from @Rudo\n\nWaiting on Nobody for the thing.",
    );
    // Fresh: not quiet yet.
    assert!(
        store
            .waiting_items(1000 + 2 * DAY, 5, 5)
            .unwrap()
            .is_empty()
    );
    let rows = store.waiting_items(1000 + 9 * DAY, 5, 5).unwrap();
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert!(rows.iter().all(|r| r.person == "Rudo"));
    let kinds: Vec<&str> = rows.iter().map(|r| r.kind.as_str()).collect();
    assert!(kinds.contains(&"loop") && kinds.contains(&"task"));
    store.snooze_waiting(&rows[0].key, 1000 + 12 * DAY).unwrap();
    assert_eq!(store.waiting_items(1000 + 9 * DAY, 5, 5).unwrap().len(), 1);
    assert_eq!(store.waiting_items(1000 + 13 * DAY, 5, 5).unwrap().len(), 2);
}

#[test]
fn a_locked_note_is_never_waited_on() {
    let (_dir, store) = temp_store();
    book(&store, "book:a");
    person(&store, "note:rudo", "Rudo");
    note(&store, "note:work", "Waiting on Rudo for the sign-off.");
    let doc = store.get("note:work").unwrap().unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": "note:work", "_rev": doc.rev, "title": "work", "body": "",
                "bookId": "book:a", "kansoLock": lock(), "createdAt": 1000, "updatedAt": 1000}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    assert!(
        store
            .waiting_items(1000 + 20 * DAY, 5, 5)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_revisit_is_not_an_edit_and_shows_when_due() {
    let (_dir, store) = temp_store();
    book(&store, "book:a");
    note(&store, "note:a", "Check the protocol");
    let before = store.get("note:a").unwrap().unwrap();
    let revisions_before = store.revisions("note:a").unwrap().len();
    let set = Revisit {
        on: "2026-11-18".into(),
        set_at: 5,
        why: "see if it survived".into(),
    };
    assert!(store.set_revisit("note:a", Some(&set)).unwrap());
    let after = store.get("note:a").unwrap().unwrap();
    assert_eq!(after.fields["updatedAt"], before.fields["updatedAt"]);
    assert_eq!(after.fields["body"], before.fields["body"]);
    assert_eq!(store.revisions("note:a").unwrap().len(), revisions_before);
    assert_eq!(store.revisit_of("note:a").unwrap(), Some(set.clone()));
    assert!(store.revisits_due("2026-11-17", 5).unwrap().is_empty());
    let due = store.revisits_due("2026-11-19", 5).unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].why, "see if it survived");
    assert_eq!(store.revisit_due_count("2026-11-19").unwrap(), 1);
    // The same value again changes nothing; clearing removes it.
    assert!(!store.set_revisit("note:a", Some(&set)).unwrap());
    assert!(store.set_revisit("note:a", None).unwrap());
    assert!(store.revisits_due("2026-12-01", 5).unwrap().is_empty());
}

#[test]
fn a_locked_note_cannot_be_revisited_and_never_shows() {
    let (_dir, store) = temp_store();
    book(&store, "book:a");
    note(&store, "note:a", "Secret");
    let set = Revisit {
        on: "2026-11-01".into(),
        set_at: 5,
        why: String::new(),
    };
    store.set_revisit("note:a", Some(&set)).unwrap();
    let doc = store.get("note:a").unwrap().unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": "note:a", "_rev": doc.rev, "title": "a", "body": "",
                "bookId": "book:a", "kansoLock": lock(), "kansoRevisit": {"on": "2026-11-01", "setAt": 5},
                "createdAt": 1000, "updatedAt": 3000}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    assert!(store.revisits_due("2027-01-01", 5).unwrap().is_empty());
    assert!(!store.set_revisit("note:a", None).unwrap());
}

proptest! {
    #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f), cases: 24, ..ProptestConfig::default() })]

    /// Whatever sessions are recorded, a locked or trashed note never comes
    /// back from Pick up.
    #[test]
    fn pick_up_never_returns_a_hidden_note(plan in prop::collection::vec((0usize..3, 0u32..200), 1..8)) {
        let (_dir, store) = temp_store();
        book(&store, "book:a");
        note(&store, "note:ok", "Open words here");
        note(&store, "note:trash", "Trashed words");
        note(&store, "note:locked", "Locked words");
        for (which, chars) in &plan {
            let id = ["note:ok", "note:trash", "note:locked"][*which];
            store.note_writing(id, 0, 1, *chars, "h", NOW).unwrap();
        }
        let doc = store.get("note:trash").unwrap().unwrap();
        store.put(DocType::Note, json!({"_id": "note:trash", "_rev": doc.rev, "title": "t", "body": "x",
            "bookId": "trash", "createdAt": 1000, "updatedAt": 1000}), Put::Upsert, Origin::Ui).unwrap();
        let doc = store.get("note:locked").unwrap().unwrap();
        store.put(DocType::Note, json!({"_id": "note:locked", "_rev": doc.rev, "title": "l", "body": "",
            "bookId": "book:a", "kansoLock": lock(), "createdAt": 1000, "updatedAt": 1000}), Put::Upsert, Origin::Ui).unwrap();
        for row in store.pick_up_places(NOW, &[], 10).unwrap() {
            prop_assert_eq!(row.note_id, "note:ok");
            prop_assert!(!row.body.is_empty());
        }
        let _ = LoopKind::Mine;
    }
}
