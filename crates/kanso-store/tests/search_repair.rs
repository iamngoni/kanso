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

/// Issue #449: the index damaged while the store is open. A rare word
/// misses the damaged entries, so `BuzzFi` and a quoted phrase still worked
/// while `daily audit` failed with "fts5: missing row". A search that hits
/// the damage now rebuilds the index and answers.
#[test]
fn a_search_hitting_damage_while_open_rebuilds_the_index() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kanso.db");
    let store = Store::open(&path).unwrap();
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    for (title, body) in [
        ("BuzzFi daily audit", "ledger"),
        ("Weekly audit", "daily standup"),
        ("Other", "nothing"),
    ] {
        store
            .put(
                DocType::Note,
                json!({"bookId": book, "title": title, "body": body}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    }
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute(
        "DELETE FROM notes_fts_content WHERE c1 = 'Weekly audit'",
        [],
    )
    .unwrap();
    drop(conn);
    assert_eq!(search(&store, "BuzzFi").unwrap(), 1);
    assert_eq!(search(&store, "\"BuzzFi daily audit\"").unwrap(), 1);
    assert_eq!(search(&store, "daily audit").unwrap(), 2);
    assert_eq!(search(&store, "BuzzFi daily audit").unwrap(), 1);
    let summaries = store
        .search_summaries(&SearchParams {
            query: Query::parse("audit daily"),
            ..SearchParams::default()
        })
        .unwrap();
    assert_eq!(summaries.len(), 2);
    assert!(!store.repair_search_index().unwrap());
}

mod any_words {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    const OPERATORS: [&str; 4] = ["AND", "OR", "NOT", "NEAR"];

    fn word() -> impl Strategy<Value = String> {
        prop_oneof![
            "[a-zA-Z]{1,8}",
            "[a-z]{0,4}[\"'*:^()+\\-.,!?]{1,3}[a-z]{0,4}",
            proptest::sample::select(&OPERATORS[..]).prop_map(str::to_owned),
            "(tag|book|title|body|status|-tag|-)?:[a-zA-Z\"]{0,6}",
            "\\PC{1,4}",
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 128,
            rng_seed: RngSeed::Fixed(449),
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Any words, quoted or not, joined by spaces, never make a search
        /// fail, and a note holding every plain word is found by them all.
        #[test]
        fn any_string_of_words_searches(words in proptest::collection::vec(word(), 1..6)) {
            let dir = tempfile::tempdir().unwrap();
            let store = Store::open(dir.path().join("kanso.db")).unwrap();
            let book = store
                .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
                .unwrap()
                .id;
            let plain: Vec<&str> = words
                .iter()
                .map(String::as_str)
                .filter(|w| w.chars().all(|c| c.is_ascii_alphabetic()))
                .collect();
            store
                .put(
                    DocType::Note,
                    json!({"bookId": book, "title": "t", "body": plain.join(" ")}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
            let keyword = words.join(" ");
            let found = search(&store, &keyword);
            prop_assert!(found.is_ok(), "{:?}: {:?}", keyword, found);
            let summaries = store.search_summaries(&SearchParams {
                query: Query::parse(&keyword),
                ..SearchParams::default()
            });
            prop_assert!(summaries.is_ok(), "{:?}: {:?}", keyword, summaries);
            if plain.len() == words.len() {
                prop_assert_eq!(found.unwrap(), 1, "{:?}", keyword);
            }
        }
    }
}
