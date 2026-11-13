//! Moving a task's due date through the store (#390, slice 27): one write,
//! the time and repeat kept, taken back in one write.

use kanso_core::DocType;
use kanso_core::dates::Date;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

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

const TEXT: &str = "film @2026-11-14T15:00 every week";

#[test]
fn a_moved_task_keeps_its_time_and_repeat_and_is_taken_back_in_one_write() {
    let original = format!("# Shoots\n- [ ] {TEXT}\n- [ ] edit\n");
    let (_dir, store) = temp_store(&original);
    let day = Date::new(2026, 11, 17).unwrap();
    let edit = store
        .set_task_due("note:r", 1, TEXT, day, 5000, Origin::Ui)
        .unwrap()
        .unwrap();
    assert!(edit.changed);
    assert_eq!(
        body(&store),
        "# Shoots\n- [ ] film @2026-11-17T15:00 every week\n- [ ] edit\n"
    );
    assert!(store.undo_note_property(&edit, Origin::Ui).unwrap());
    assert_eq!(body(&store), original);
}

#[test]
fn the_same_day_writes_nothing_and_a_missing_task_is_none() {
    let original = format!("- [ ] {TEXT}\n");
    let (_dir, store) = temp_store(&original);
    let rev = store.note("note:r").unwrap().unwrap().rev;
    let same = Date::new(2026, 11, 14).unwrap();
    let edit = store
        .set_task_due("note:r", 0, TEXT, same, 5000, Origin::Ui)
        .unwrap()
        .unwrap();
    assert!(!edit.changed);
    assert_eq!(store.note("note:r").unwrap().unwrap().rev, rev);
    assert!(
        store
            .set_task_due("note:r", 0, "gone @2026-11-14", same, 5000, Origin::Ui)
            .unwrap()
            .is_none()
    );
}
