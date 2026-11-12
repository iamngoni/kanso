//! Ticking a repeating task writes the next one (#390, slice 4): through
//! the store, in one write, taken back in one write. The seed is pinned.

use kanso_core::DocType;
use kanso_core::dates::Date;
use kanso_store::{CheckOff, Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};
use serde_json::json;

const TODAY: Date = Date {
    year: 2026,
    month: 11,
    day: 14,
};

fn config() -> Config {
    Config {
        cases: 64,
        rng_seed: RngSeed::Fixed(0x0074_6963_6b73),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    }
}

fn temp_store(body: &str) -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    store
        .put(
            DocType::Book,
            json!({"_id": "book:a", "name": "a"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": "note:r", "title": "r", "body": body, "bookId": "book:a", "updatedAt": 1000}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    (dir, store)
}

fn body(store: &Store) -> String {
    store.note("note:r").unwrap().unwrap().body
}

const WEEKLY: &str = "# Shoots\n- [ ] film Pixel review @2026-11-14T15:00 every week\n- [ ] edit\n";
const WEEKLY_TEXT: &str = "film Pixel review @2026-11-14T15:00 every week";

#[test]
fn ticking_writes_the_next_one_and_undo_restores_the_text() {
    let (_dir, store) = temp_store(WEEKLY);
    let tick = store
        .tick_task("note:r", 1, WEEKLY_TEXT, true, TODAY, 5000, Origin::Ui)
        .unwrap();
    assert!(matches!(tick.off, CheckOff::Changed { .. }));
    assert_eq!(
        body(&store),
        "# Shoots\n- [ ] film Pixel review @2026-11-21T15:00 every week\n\
         - [x] film Pixel review @2026-11-14T15:00\n- [ ] edit\n"
    );
    // The new line is in the derived task table too.
    let rows = store
        .tasks(&kanso_store::TaskQuery {
            include_done: true,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(rows.len(), 3);

    let undo = tick.undo.expect("a repeat tick can be taken back");
    assert!(matches!(
        store.undo_tick("note:r", &undo, 6000, Origin::Ui).unwrap(),
        CheckOff::Changed { .. }
    ));
    assert_eq!(body(&store), WEEKLY);
}

#[test]
fn undo_leaves_a_note_edited_since_alone() {
    let (_dir, store) = temp_store(WEEKLY);
    let tick = store
        .tick_task("note:r", 1, WEEKLY_TEXT, true, TODAY, 5000, Origin::Ui)
        .unwrap();
    let edited = format!("{}more\n", body(&store));
    let rev = store.get("note:r").unwrap().unwrap().rev;
    store
        .put(
            DocType::Note,
            json!({"_id": "note:r", "_rev": rev, "title": "r", "body": edited, "bookId": "book:a", "updatedAt": 5500}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(
        store
            .undo_tick("note:r", &tick.undo.unwrap(), 6000, Origin::Ui)
            .unwrap(),
        CheckOff::Unchanged
    );
    assert_eq!(body(&store), edited);
}

#[test]
fn unticking_the_log_line_keeps_the_next_one() {
    let (_dir, store) = temp_store(WEEKLY);
    store
        .tick_task("note:r", 1, WEEKLY_TEXT, true, TODAY, 5000, Origin::Ui)
        .unwrap();
    let off = store
        .tick_task(
            "note:r",
            2,
            "film Pixel review @2026-11-14T15:00",
            false,
            TODAY,
            6000,
            Origin::Ui,
        )
        .unwrap();
    assert!(matches!(off.off, CheckOff::Changed { .. }));
    assert!(off.undo.is_none());
    assert_eq!(
        body(&store),
        "# Shoots\n- [ ] film Pixel review @2026-11-21T15:00 every week\n\
         - [ ] film Pixel review @2026-11-14T15:00\n- [ ] edit\n"
    );
}

#[test]
fn a_replayed_tick_writes_no_second_next_line() {
    let (_dir, store) = temp_store(WEEKLY);
    store
        .tick_task("note:r", 1, WEEKLY_TEXT, true, TODAY, 5000, Origin::Ui)
        .unwrap();
    let once = body(&store);
    let again = store
        .tick_task("note:r", 1, WEEKLY_TEXT, true, TODAY, 5001, Origin::Ui)
        .unwrap();
    assert_eq!(again.off, CheckOff::Missing);
    assert_eq!(body(&store), once);
}

#[test]
fn a_plain_task_ticks_as_before_and_has_no_undo_ticket() {
    let (_dir, store) = temp_store("- [ ] pay @2026-11-14\n");
    let tick = store
        .tick_task(
            "note:r",
            0,
            "pay @2026-11-14",
            true,
            TODAY,
            5000,
            Origin::Ui,
        )
        .unwrap();
    assert!(matches!(tick.off, CheckOff::Changed { .. }));
    assert!(tick.undo.is_none());
    assert_eq!(body(&store), "- [x] pay @2026-11-14\n");
}

#[test]
fn set_task_checked_is_the_same_path() {
    let (_dir, store) = temp_store(WEEKLY);
    // 2026-11-14 noon UTC: the local day is the 14th in any zone.
    let noon = 1_794_657_600_000;
    let off = store
        .set_task_checked("note:r", 1, WEEKLY_TEXT, true, noon, Origin::Api)
        .unwrap();
    assert!(matches!(off, CheckOff::Changed { .. }));
    assert!(body(&store).contains("- [ ] film Pixel review @2026-11-21T15:00 every week\n- [x] film Pixel review @2026-11-14T15:00\n"));
}

proptest! {
    #![proptest_config(config())]

    /// Ticking then undoing is the identity on the note's text, the next
    /// line is strictly after both days, and the same tick twice writes one.
    #[test]
    fn prop_tick_then_undo_is_identity(
        rule in prop::sample::select(vec![
            "every day", "every week", "every 2 weeks on mon and thu", "every month",
            "every month on the 31st", "every year", "every 3 days after done", "every weekday",
        ]),
        days in 0i64..40_000,
        today_days in 0i64..40_000,
        time in prop::option::of((0u32..24, 0u32..60)),
        crlf in any::<bool>(),
    ) {
        let due = kanso_core::tasks::Due { date: Date::from_days(days), time };
        let text = format!("task @{} {rule}", due.format());
        let eol = if crlf { "\r\n" } else { "\n" };
        let original = format!("intro{eol}- [ ] {text}{eol}tail{eol}");
        let today = Date::from_days(today_days);
        let (_dir, store) = temp_store(&original);

        let tick = store.tick_task("note:r", 1, &text, true, today, 5000, Origin::Ui).unwrap();
        let changed = matches!(tick.off, CheckOff::Changed { .. });
        prop_assert!(changed);
        let ticked = body(&store);
        let items = kanso_core::tasks::task_items(&ticked);
        prop_assert_eq!(items.len(), 2);
        let next = kanso_core::tasks::Due::parse(items[0].due.as_deref().unwrap()).unwrap();
        prop_assert!(next.date > due.date && next.date > today);

        let replay = store.tick_task("note:r", 1, &text, true, today, 5001, Origin::Ui).unwrap();
        let replayed = matches!(replay.off, CheckOff::Changed { .. });
        prop_assert!(!replayed);
        prop_assert_eq!(body(&store), ticked);

        store.undo_tick("note:r", &tick.undo.unwrap(), 6000, Origin::Ui).unwrap();
        prop_assert_eq!(body(&store), original);
    }

    /// A task with no repeat ticks and clears as it always did.
    #[test]
    fn prop_a_plain_task_is_unchanged_from_today(days in 0i64..40_000, words in "[a-z]{1,8}") {
        let due = kanso_core::tasks::Due { date: Date::from_days(days), time: None };
        let text = format!("{words} @{}", due.format());
        let original = format!("- [ ] {text}\n");
        let (_dir, store) = temp_store(&original);
        store.tick_task("note:r", 0, &text, true, TODAY, 5000, Origin::Ui).unwrap();
        prop_assert_eq!(body(&store), format!("- [x] {text}\n"));
        store.tick_task("note:r", 0, &text, false, TODAY, 5001, Origin::Ui).unwrap();
        prop_assert_eq!(body(&store), original);
    }
}
