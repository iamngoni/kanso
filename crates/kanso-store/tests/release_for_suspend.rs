//! Releasing the store for a suspend closes every connection (iOS kills an
//! app with `0xdead10cc` that sleeps holding a lock on a file in a shared
//! container) and loses nothing: the next read or write opens again.

use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

fn note(store: &Store, text: &str) -> String {
    let book = store
        .put(
            DocType::Book,
            json!({"name": "Notes"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    store
        .put(
            DocType::Note,
            json!({"title": text, "body": text, "bookId": book}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id
}

#[test]
fn release_closes_the_connections_and_leaves_no_log_behind() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kanso.db");
    let store = Store::open(&path).unwrap();
    let id = note(&store, "before");
    store.meta("anything").unwrap();
    store.release_for_suspend();
    // The last connection closing checkpoints and removes the log files.
    assert!(!dir.path().join("kanso.db-wal").exists());
    assert!(!dir.path().join("kanso.db-shm").exists());
    // Reading opens again, and the write made before is there.
    assert!(store.get(&id).unwrap().is_some());
}

#[test]
fn a_write_after_a_release_reopens_the_writer() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    let first = note(&store, "one");
    store.release_for_suspend();
    store.release_for_suspend();
    let second = note(&store, "two");
    assert!(store.get(&first).unwrap().is_some());
    assert!(store.get(&second).unwrap().is_some());
}
