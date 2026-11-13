//! `Library::move_scheduled`: a note's date property or a task's due date
//! moved to another day, and taken back.

use std::sync::Arc;

use kanso_core::DocType;
use kanso_store::{Origin, Put};
use kanso_sync::{MemorySecrets, MemoryServer};
use proptest::prelude::*;
use serde_json::json;

use super::*;
use crate::model::CardMoved;

fn day(year: i32, month: u32, day: u32) -> DayDate {
    DayDate { year, month, day }
}

fn library_with(body: &str) -> (tempfile::TempDir, Library) {
    let dir = tempfile::tempdir().unwrap();
    let library = Library::open_for_test(
        &dir.path().join("library.db"),
        Arc::new(MemorySecrets::new()),
        Arc::new(MemoryServer::new()),
    )
    .unwrap();
    let store = &library.inner.store;
    store
        .put(
            DocType::Book,
            json!({"_id": "book:a", "name": "A"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": "note:a", "title": "Pixel 11", "bookId": "book:a", "body": body}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    (dir, library)
}

fn body(library: &Library) -> String {
    library.inner.store.note("note:a").unwrap().unwrap().body
}

const NOTE: &str = "---\npublish: 2026-11-18T09:00 America/New_York\n---\nText\n";
const TASK: &str = "- [ ] Film review @2026-11-13T15:00 every week\n";
const TASK_TEXT: &str = "Film review @2026-11-13T15:00 every week";

fn move_note(library: &Library, key: &str, to: DayDate) -> Result<CardMoved, KansoError> {
    library.move_scheduled("note:a", None, "", Some(key.to_owned()), to)
}

#[test]
fn a_note_keeps_its_time_and_zone() {
    let (_dir, library) = library_with(NOTE);
    let moved = move_note(&library, "publish", day(2026, 11, 20)).unwrap();
    assert!(matches!(moved, CardMoved::Changed { undo: Some(_), .. }));
    assert!(body(&library).contains("publish: 2026-11-20T09:00 America/New_York"));
}

#[test]
fn a_task_keeps_its_time_and_repeat() {
    let (_dir, library) = library_with(TASK);
    library
        .move_scheduled("note:a", Some(0), TASK_TEXT, None, day(2026, 11, 14))
        .unwrap();
    assert_eq!(
        body(&library),
        "- [ ] Film review @2026-11-14T15:00 every week\n"
    );
}

#[test]
fn the_same_day_changes_nothing_and_a_gone_target_is_missing() {
    let (_dir, library) = library_with(NOTE);
    assert_eq!(
        move_note(&library, "publish", day(2026, 11, 18)).unwrap(),
        CardMoved::Unchanged
    );
    assert_eq!(
        move_note(&library, "embargo", day(2026, 11, 18)).unwrap(),
        CardMoved::Missing
    );
    let none = library
        .move_scheduled(
            "note:zzz",
            None,
            "",
            Some("publish".into()),
            day(2026, 11, 18),
        )
        .unwrap();
    assert_eq!(none, CardMoved::Missing);
    assert!(move_note(&library, "publish", day(2026, 2, 30)).is_err());
}

#[test]
fn undo_puts_the_note_back() {
    let (_dir, library) = library_with(NOTE);
    let Ok(CardMoved::Changed {
        undo: Some(ticket), ..
    }) = move_note(&library, "publish", day(2026, 12, 1))
    else {
        panic!("not moved");
    };
    assert_eq!(library.undo(ticket).unwrap(), 1);
    assert_eq!(body(&library), NOTE);
}

#[test]
fn notes_on_a_date_property_come_through_the_bridge() {
    use crate::schedule_view_model::{ScheduleRequest, ScheduleRowKind};
    use kanso_core::schema::{PropertyDef, PropertyType};
    let (_dir, library) = library_with(NOTE);
    let def = |key: &str, kind| PropertyDef {
        key: key.to_owned(),
        kind,
        options: Vec::new(),
    };
    library
        .inner
        .store
        .set_notebook_properties(
            "book:a",
            &[def("publish", PropertyType::DateTimeZone)],
            Origin::Ui,
        )
        .unwrap();
    let request = |keys: Option<Vec<String>>| ScheduleRequest {
        from: day(2026, 11, 1),
        until: day(2026, 12, 1),
        books: None,
        date_keys: keys,
        include_done: true,
    };
    let rows = library.schedule(request(None)).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].kind, ScheduleRowKind::Note);
    assert_eq!(rows[0].property.as_deref(), Some("publish"));
    assert_eq!(rows[0].title, "Pixel 11");
    assert_eq!(rows[0].date, day(2026, 11, 18));
    assert_eq!(rows[0].time.as_deref(), Some("09:00"));
    assert_eq!(rows[0].zone.as_deref(), Some("America/New_York"));
    assert!(rows[0].line.is_none() && !rows[0].projected);
    // The property filter keeps or drops it.
    assert_eq!(
        library
            .schedule(request(Some(vec!["publish".into()])))
            .unwrap()
            .len(),
        1
    );
    assert!(
        library
            .schedule(request(Some(vec!["embargo".into()])))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn date_keys_are_the_date_properties_of_the_notebooks() {
    use kanso_core::schema::{PropertyDef, PropertyType};
    let (_dir, library) = library_with(NOTE);
    let def = |key: &str, kind| PropertyDef {
        key: key.to_owned(),
        kind,
        options: Vec::new(),
    };
    library
        .inner
        .store
        .set_notebook_properties(
            "book:a",
            &[
                def("publish", PropertyType::Date),
                def("embargo", PropertyType::DateTimeZone),
                def("sponsor", PropertyType::Text),
                def("shoot", PropertyType::DateTime),
            ],
            Origin::Ui,
        )
        .unwrap();
    // The same key in another notebook is listed once.
    library
        .inner
        .store
        .put(
            DocType::Book,
            json!({"_id": "book:b", "name": "B"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    library
        .inner
        .store
        .set_notebook_properties("book:b", &[def("publish", PropertyType::Date)], Origin::Ui)
        .unwrap();
    assert_eq!(
        library.schedule_date_keys(None).unwrap(),
        vec!["embargo", "publish", "shoot"]
    );
    assert_eq!(
        library
            .schedule_date_keys(Some(vec!["book:zzz".into()]))
            .unwrap(),
        Vec::<String>::new()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Whatever day a note or a task moves to, it lands on that day with its
    /// time, zone and repeat as written, and Undo gives back every character.
    #[test]
    fn a_move_changes_only_the_day_and_undoes(
        year in 2020i32..2035,
        month in 1u32..=12,
        d in 1u32..=28,
        task in any::<bool>(),
    ) {
        let start = if task { TASK } else { NOTE };
        let (_dir, library) = library_with(start);
        let to = day(year, month, d);
        let moved = if task {
            library.move_scheduled("note:a", Some(0), TASK_TEXT, None, to)
        } else {
            move_note(&library, "publish", to)
        }
        .unwrap();
        let iso = format!("{year:04}-{month:02}-{d:02}");
        match moved {
            CardMoved::Changed { undo, .. } => {
                let after = body(&library);
                prop_assert!(after.contains(&iso));
                let kept = if task {
                    after.ends_with("T15:00 every week\n")
                } else {
                    after.contains("T09:00 America/New_York")
                };
                prop_assert!(kept);
                prop_assert_eq!(library.undo(undo.unwrap()).unwrap(), 1);
            }
            CardMoved::Unchanged => prop_assert!(start.contains(&iso)),
            CardMoved::Missing => prop_assert!(false, "target went missing"),
        }
        prop_assert_eq!(body(&library), start);
    }
}
