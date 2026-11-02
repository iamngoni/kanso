//! A damaged search index ("fts5: missing row N from content table") is
//! rebuilt from the notes when the store opens.

use kanso_core::{DocType, Query};
use kanso_store::{Origin, Put, SearchParams, Store};
use serde_json::json;

fn search(store: &Store, text: &str) -> kanso_store::Result<usize> {
    store
        .search(&SearchParams {
            query: Query::parse(text),
            ..SearchParams::default()
        })
        .map(|hits| hits.len())
}

#[test]
fn a_damaged_search_index_is_rebuilt_on_open() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kanso.db");
    {
        let store = Store::open(&path).unwrap();
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        for title in ["d2d one", "d2d two", "other"] {
            store
                .put(
                    DocType::Note,
                    json!({"bookId": book, "title": title, "body": "text"}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
        assert_eq!(search(&store, "d2d").unwrap(), 2);
    }
    // Damage it: the index keeps entries whose stored row is gone.
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute("DELETE FROM notes_fts_content", []).unwrap();
    drop(conn);
    {
        let store = Store::open(&path).unwrap();
        assert_eq!(search(&store, "d2d").unwrap(), 2);
    }
}
