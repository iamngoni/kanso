use std::sync::Arc;

use kanso_core::DocType;
use kanso_core::zoned::from_utc_ms;
use kanso_store::{Origin, Put};
use kanso_sync::{MemorySecrets, MemoryServer};
use proptest::prelude::*;
use serde_json::json;

use super::*;

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
            json!({"_id": "book:a", "name": "Studio"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": "note:a", "title": "Pixel 11 review", "bookId": "book:a", "body": body}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    (dir, library)
}

#[test]
fn a_note_without_one_or_a_missing_note_has_no_embargo() {
    let (_dir, library) = library_with("# Pixel 11 review\n");
    assert_eq!(library.embargo("note:a", 0, 0).unwrap(), None);
    assert_eq!(library.embargo("note:gone", 0, 0).unwrap(), None);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// The phone is told of an embargo exactly while it holds, with the
    /// moment it lifts whatever the phone's offset.
    #[test]
    fn prop_the_phone_sees_the_embargo_until_it_lifts(
        ahead in -3_000_000_000i64..3_000_000_000,
        offset in -14i64 * 3_600_000..14 * 3_600_000,
        zone in prop::sample::select(vec!["America/New_York", "Asia/Tokyo", "Africa/Harare"]),
    ) {
        let now = 1_794_582_000_000i64;
        let written = from_utc_ms(now + ahead - (now + ahead).rem_euclid(60_000), zone).unwrap();
        let moment = written.utc_ms().unwrap();
        let (_dir, library) = library_with(&format!("---\nembargo: {}\n---\nWords.\n", written.text()));
        let seen = library.embargo("note:a", now, offset).unwrap();
        prop_assert_eq!(seen.is_some(), moment > now);
        if let Some(seen) = seen {
            prop_assert_eq!(seen.lifts_at_ms, moment);
            prop_assert_eq!(seen.zone.as_deref(), Some(zone));
            prop_assert_eq!(seen.lifts_in, embargo::in_words(moment - now));
        }
    }
}
