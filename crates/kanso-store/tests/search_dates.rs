//! `SearchParams` date bounds: inclusive, by the right column, and never a
//! way around the eligibility of a note.

use kanso_core::DocType;
use kanso_store::{Origin, Put, SearchParams, Store};
use proptest::prelude::*;
use serde_json::json;

fn store_with(times: &[(i64, i64)]) -> (tempfile::TempDir, Store, Vec<String>) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    let book = store
        .put(
            DocType::Book,
            json!({"name": "Work"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let ids = times
        .iter()
        .enumerate()
        .map(|(ix, (created, updated))| {
            store
                .put(
                    DocType::Note,
                    json!({"title": format!("Note {ix}"), "body": "pricing", "bookId": book,
                           "createdAt": created, "updatedAt": updated}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        })
        .collect();
    (dir, store, ids)
}

fn found(store: &Store, params: &SearchParams) -> Vec<String> {
    let mut ids: Vec<String> = store
        .search(params)
        .unwrap()
        .into_iter()
        .map(|doc| doc.id)
        .collect();
    ids.sort();
    ids
}

#[test]
fn the_bounds_are_inclusive_and_use_their_own_column() {
    let (_dir, store, ids) = store_with(&[(1_000, 5_000), (2_000, 3_000), (9_000, 9_500)]);
    let mut sorted = ids.clone();
    sorted.sort();
    let by_updated = SearchParams {
        updated_after: Some(3_000),
        updated_before: Some(5_000),
        ..SearchParams::default()
    };
    let mut want = vec![ids[0].clone(), ids[1].clone()];
    want.sort();
    assert_eq!(found(&store, &by_updated), want);
    let by_created = SearchParams {
        created_after: Some(2_000),
        created_before: Some(8_999),
        ..SearchParams::default()
    };
    assert_eq!(found(&store, &by_created), vec![ids[1].clone()]);
    let open = SearchParams {
        updated_after: Some(9_501),
        ..SearchParams::default()
    };
    assert!(found(&store, &open).is_empty());
    assert_eq!(found(&store, &SearchParams::default()), sorted);
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]

    #[test]
    fn prop_a_bounded_search_returns_exactly_the_notes_inside(
        times in proptest::collection::vec((1_000i64..100_000, 1_000i64..100_000), 1..8),
        lo in 1_000i64..100_000,
        span in 0i64..50_000,
    ) {
        let (_dir, store, ids) = store_with(&times);
        let hi = lo + span;
        let params = SearchParams {
            updated_after: Some(lo),
            updated_before: Some(hi),
            ..SearchParams::default()
        };
        let mut want: Vec<String> = ids
            .iter()
            .zip(&times)
            .filter(|(_, (_, updated))| *updated >= lo && *updated <= hi)
            .map(|(id, _)| id.clone())
            .collect();
        want.sort();
        prop_assert_eq!(found(&store, &params), want);
    }
}
