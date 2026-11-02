//! The margin of related notes through the real editor: it asks only once
//! typing pauses, shows nothing when nothing is close, keeps the items
//! that stay where they are, links as one undo step, and dismisses.

use std::sync::Arc;

use gpui_kit::TestAppContext;
use kanso_core::DocType;
use kanso_engine::discovery::resurface::{Item, SHOWN};
use kanso_store::{EmbeddedNote, EmbeddedPassage, Embedder, Origin, Put, Store};
use serde_json::json;

use super::writing_tests::Open;
use crate::related::SETTLE;
use crate::resurface::Resurface;

/// Hashes each word of three letters or more into one of 64 places.
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
                for w in t
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|w| w.chars().count() >= 3)
                    .map(str::to_lowercase)
                {
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

fn add_note(store: &Store, title: &str, body: &str, created_at: i64) -> String {
    let book = store.books().unwrap()[0].id.clone();
    store
        .put(
            DocType::Note,
            json!({"title": title, "body": body, "bookId": book, "createdAt": created_at}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

/// Related notes on with the fake model, every note embedded.
fn index(store: &Arc<Store>) {
    store.set_embedding_model(Some("words")).unwrap();
    store.lend_embedder(Some(Arc::new(Words)));
    let queue = store.embedding_queue("words").unwrap();
    let rows: Vec<_> = queue
        .pending
        .into_iter()
        .map(|p| EmbeddedNote {
            vector: Words.embed(std::slice::from_ref(&p.text)).remove(0),
            passages: p
                .passages
                .iter()
                .map(|q| EmbeddedPassage {
                    start: q.start,
                    end: q.end,
                    vector: Words.embed(std::slice::from_ref(&q.text)).remove(0),
                })
                .collect(),
            id: p.id,
            hash: p.hash,
        })
        .collect();
    store.put_embedded("words", &rows).unwrap();
}

fn items(open: &Open, cx: &mut TestAppContext) -> Vec<Item> {
    open.with(cx, |editor, _, cx| {
        editor.resurface().read(cx).items_for_test()
    })
}

fn pause(cx: &mut TestAppContext) {
    cx.executor().advance_clock(SETTLE);
    cx.run_until_parked();
}

const BOILER: &str =
    "The engineer said the pressure valve on the boiler needs replacing before winter.";
const WRITING: &str = "Called about the boiler pressure valve and the engineer, winter is coming.";

fn open_with_past_notes(cx: &mut TestAppContext) -> (Open, String) {
    let open = Open::new(cx, "");
    let boiler = add_note(&open.store, "Boiler service", BOILER, 1_000);
    add_note(
        &open.store,
        "Garden",
        "Tomato plants need watering every morning in summer.",
        2_000,
    );
    index(&open.store);
    (open, boiler)
}

/// Nothing is asked while typing; once it pauses, the matching note shows
/// with the passage that matched. Collapses again when the text moves on.
// no-prop-test: drives the editor; the store's and engine's properties
// cover the search and the words.
#[gpui_kit::test]
fn the_margin_asks_only_after_a_pause(cx: &mut TestAppContext) {
    let (open, boiler) = open_with_past_notes(cx);
    open.input(cx, WRITING);
    // Typing: nothing yet, however long the text.
    assert!(items(&open, cx).is_empty());
    cx.executor().advance_clock(SETTLE / 3);
    cx.run_until_parked();
    assert!(items(&open, cx).is_empty());
    pause(cx);
    let shown = items(&open, cx);
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert_eq!(shown[0].id, boiler);
    assert!(shown[0].snippet.contains("pressure valve"));
    assert!(shown[0].place.contains("B"));
    open.with(cx, |editor, _, cx| {
        assert!(editor.resurface().read(cx).has_content(cx));
    });
    // A paragraph about something else: the boiler note goes.
    open.input(
        cx,
        "\n\nZebras graze across the plains near rivers and lakes all year long.",
    );
    pause(cx);
    assert!(items(&open, cx).is_empty());
    open.with(cx, |editor, _, cx| {
        assert!(!editor.resurface().read(cx).has_content(cx));
    });
}

/// A paragraph under six words asks nothing.
// no-prop-test: drives the editor.
#[gpui_kit::test]
fn a_short_paragraph_asks_nothing(cx: &mut TestAppContext) {
    let (open, _) = open_with_past_notes(cx);
    open.input(cx, "boiler pressure valve");
    pause(cx);
    assert!(items(&open, cx).is_empty());
}

/// Link puts `[[Title]]` at the caret, and ⌘Z takes it back whole.
// no-prop-test: drives the editor with keys.
#[gpui_kit::test]
fn link_is_one_undo_step(cx: &mut TestAppContext) {
    let (open, _) = open_with_past_notes(cx);
    open.input(cx, WRITING);
    let before = open.text(cx).0;
    open.with(cx, |editor, window, cx| {
        editor.link_to("Boiler service", window, cx)
    });
    let after = open.text(cx).0;
    assert_eq!(after, format!("{before}[[Boiler service]]"));
    open.keys(cx, "cmd-z");
    assert_eq!(open.text(cx).0, before);
}

/// "Not related" keeps the note from the margin for good.
// no-prop-test: drives the editor; the store's properties cover the rule.
#[gpui_kit::test]
fn dismissing_hides_a_note_for_this_note(cx: &mut TestAppContext) {
    let (open, boiler) = open_with_past_notes(cx);
    open.input(cx, WRITING);
    pause(cx);
    assert_eq!(items(&open, cx).len(), 1);
    let boiler2 = boiler.clone();
    open.with(cx, move |editor, _, cx| {
        editor
            .resurface()
            .update(cx, |r: &mut Resurface, cx| r.dismiss_for_test(&boiler2, cx))
    });
    assert!(items(&open, cx).is_empty());
    assert!(
        open.store
            .dismissed_related(&open.id)
            .unwrap()
            .contains(&boiler)
    );
    // Asked again, it stays away.
    open.input(
        cx,
        " More about the boiler pressure valve and engineer too.",
    );
    pause(cx);
    assert!(items(&open, cx).is_empty());
}

/// No margin and no query for a note the margin must not work for.
// no-prop-test: drives the editor.
#[gpui_kit::test]
fn a_note_it_must_not_work_for_gets_no_margin(cx: &mut TestAppContext) {
    let (open, _) = open_with_past_notes(cx);
    open.with(cx, |editor, _, cx| {
        editor.resurface().update(cx, |r, cx| r.follow(None, cx))
    });
    open.input(cx, WRITING);
    pause(cx);
    assert!(items(&open, cx).is_empty());
}

fn item(id: &str) -> Item {
    Item {
        id: id.into(),
        title: id.into(),
        book: "B".into(),
        place: "B · March 2024".into(),
        when: "March 2024".into(),
        snippet: "…".into(),
        before: None,
    }
}

/// Items that stay relevant stay in place; new ones come in at the end;
/// the gone go; and there are never more than three.
// no-prop-test: drives the view with fixed lists.
#[gpui_kit::test]
fn items_that_stay_do_not_move(cx: &mut TestAppContext) {
    let (open, _) = open_with_past_notes(cx);
    let apply = |cx: &mut TestAppContext, ids: &[&str]| {
        let found: Vec<Item> = ids.iter().map(|id| item(id)).collect();
        open.with(cx, move |editor, _, cx| {
            editor
                .resurface()
                .update(cx, |r, cx| r.apply_for_test(found, cx))
        });
        items(&open, cx)
            .into_iter()
            .map(|i| i.id)
            .collect::<Vec<_>>()
    };
    assert_eq!(apply(cx, &["a", "b"]), ["a", "b"]);
    assert_eq!(apply(cx, &["c", "b", "a"]), ["a", "b", "c"]);
    assert_eq!(apply(cx, &["c", "a"]), ["a", "c"]);
    assert_eq!(apply(cx, &["w", "x", "y", "z"]).len(), SHOWN);
}

/// A daily note lists the notes written on its date in earlier years,
/// its own daily notes first, newest year first.
// no-prop-test: drives the editor; the engine's properties cover the days.
#[gpui_kit::test]
fn a_daily_note_lists_this_day_in_earlier_years(cx: &mut TestAppContext) {
    let open = Open::new(cx, "");
    let store = open.store.clone();
    let book = store.books().unwrap()[0].id.clone();
    let put = |id: Option<&str>, title: &str, created: i64| {
        let mut fields = json!({"title": title, "body": "x", "bookId": book, "createdAt": created});
        if let Some(id) = id {
            fields["_id"] = id.into();
        }
        store
            .put(DocType::Note, fields, Put::Upsert, Origin::Ui)
            .unwrap()
            .id
    };
    // 2024-03-10 12:00 UTC, and the daily notes of 2025 and today.
    let older = put(None, "Walk", 1_710_072_000_000);
    let daily_2025 = put(
        Some("note:daily-2025-03-10"),
        "2025-03-10",
        1_741_608_000_000,
    );
    let today = put(
        Some("note:daily-2026-03-10"),
        "2026-03-10",
        1_773_144_000_000,
    );
    put(None, "Other day", 1_710_200_000_000 + 86_400_000 * 3);
    open.with(cx, move |editor, window, cx| {
        editor.open(Some(&today), window, cx)
    });
    cx.run_until_parked();
    let day = open.with(cx, |editor, _, cx| {
        editor.resurface().read(cx).day_for_test()
    });
    let ids: Vec<_> = day.iter().map(|d| d.id.clone()).collect();
    assert_eq!(ids, [daily_2025, older]);
    assert_eq!(day[0].ago, "1 year ago");
    assert_eq!(day[1].ago, "2 years ago");
}
