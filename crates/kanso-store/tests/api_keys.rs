//! The API keys table (issue #243): saving, listing, revoking, touching.

use kanso_store::{KeyAccess, NewApiKey, Store, StoreError};
use proptest::prelude::*;

fn temp_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn key(id: &str, hash: u8, access: KeyAccess) -> NewApiKey {
    NewApiKey {
        id: id.into(),
        name: format!("key {id}"),
        client: "codex".into(),
        token_hash: [hash; 32],
        access,
        books: None,
        created_at: 10,
    }
}

#[test]
fn a_key_is_saved_listed_touched_and_revoked() {
    let (_d, store) = temp_store();
    let mut new = key("a", 1, KeyAccess::Write);
    new.books = Some(vec!["book:x".into(), "book:y".into()]);
    store.api_key_insert(&new).unwrap();
    store.api_key_insert(&key("b", 2, KeyAccess::Read)).unwrap();

    let live = store.api_keys_live().unwrap();
    assert_eq!(live.len(), 2);
    assert_eq!(live[0].token_hash, [1; 32]);
    assert_eq!(live[0].row.access, KeyAccess::Write);
    assert_eq!(live[0].row.books.as_deref(), new.books.as_deref());
    assert_eq!(live[1].row.books, None);

    store.api_key_touch("a", 99).unwrap();
    assert!(store.api_key_revoke("a", 100).unwrap());
    assert!(!store.api_key_revoke("a", 200).unwrap());
    assert!(!store.api_key_revoke("nope", 200).unwrap());
    store.api_key_touch("a", 300).unwrap();

    let all = store.api_keys_all().unwrap();
    let a = all.iter().find(|r| r.id == "a").unwrap();
    assert_eq!((a.last_used_at, a.revoked_at), (Some(99), Some(100)));
    assert_eq!(store.api_keys_live().unwrap().len(), 1);
}

#[test]
fn the_same_hash_or_id_is_refused() {
    let (_d, store) = temp_store();
    store.api_key_insert(&key("a", 1, KeyAccess::Read)).unwrap();
    for dup in [key("b", 1, KeyAccess::Read), key("a", 2, KeyAccess::Read)] {
        assert!(matches!(
            store.api_key_insert(&dup),
            Err(StoreError::Invalid(_))
        ));
    }
}

#[test]
fn the_debug_output_hides_the_hash() {
    let (_d, store) = temp_store();
    let new = key("a", 7, KeyAccess::Read);
    store.api_key_insert(&new).unwrap();
    assert!(!format!("{new:?}").contains("7, 7"));
    assert!(!format!("{:?}", store.api_keys_live().unwrap()).contains("7, 7"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Whatever is saved and revoked, a live key is exactly one not revoked.
    #[test]
    fn prop_live_is_saved_minus_revoked(
        revoke in prop::collection::vec(0usize..6, 0..10),
        count in 1usize..6,
    ) {
        let (_d, store) = temp_store();
        for i in 0..count {
            store.api_key_insert(&key(&format!("k{i}"), i as u8, KeyAccess::Read)).unwrap();
        }
        let mut gone = std::collections::HashSet::new();
        for r in revoke {
            let revoked = store.api_key_revoke(&format!("k{r}"), 5).unwrap();
            prop_assert_eq!(revoked, r < count && gone.insert(r));
        }
        let live: Vec<String> = store.api_keys_live().unwrap().into_iter().map(|k| k.row.id).collect();
        prop_assert_eq!(live.len(), count - gone.len());
        for r in gone {
            let id = format!("k{}", r);
            prop_assert!(!live.contains(&id));
        }
    }
}
