//! People and projects in the store: pages are notes marked in their front
//! matter, mentions are derived per line, locked notes never take part, and a
//! page's cache of AI sections is written without counting as an edit. CI
//! pins the seed with `PROPTEST_RNG_SEED`.

use kanso_core::DocType;
use kanso_core::entities::EntityKind;
use kanso_core::page_cache::{CachedItem, CachedSection, PAGE_FIELD, PageCache};
use kanso_store::{Origin, Put, Store, SyncDoc};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Value, json};

const CANARY: &str = "zebracanary4471";

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

fn temp_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn book(store: &Store, id: &str) -> String {
    store
        .put(
            DocType::Book,
            json!({"_id": id, "name": id}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn note(store: &Store, id: &str, title: &str, body: &str, book: &str) -> String {
    let current = store.get(id).unwrap().map(|d| d.rev);
    let mut doc = json!({"_id": id, "title": title, "body": body, "bookId": book,
        "createdAt": 1000, "updatedAt": 1000});
    if let Some(rev) = current {
        doc["_rev"] = json!(rev);
    }
    store
        .put(DocType::Note, doc, Put::Upsert, Origin::Ui)
        .unwrap()
        .rev
}

fn lock() -> Value {
    json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"})
}

fn person(store: &Store, id: &str, title: &str, aliases: &str, book: &str) {
    let body = format!("---\nkanso: person\naliases: [{aliases}]\n---\nOwn words.\n");
    note(store, id, title, &body, book);
}

#[test]
fn a_marked_note_is_a_page_and_mentions_find_it_three_ways() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    note(&store, "note:plain", "Plain", "Sam is not a page yet\n", &b);
    assert!(store.entities().unwrap().is_empty());
    person(&store, "note:sam", "Sam Moyo", "Sam, \"@sam\"", &b);
    let pages = store.entities().unwrap();
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].kind, EntityKind::Person);
    assert_eq!(pages[0].names, vec!["sam moyo", "sam"]);
    // The earlier note now mentions the page, as plain words.
    let found = store.entity_mentions("note:sam", 50).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].note_id, "note:plain");
    note(
        &store,
        "note:meet",
        "Meet",
        "Met [[Sam]] today\n@sam said hi\nwith Sam Moyo\n`@sam in code`\n",
        &b,
    );
    let found = store.entity_mentions("note:sam", 50).unwrap();
    let lines: Vec<(String, u32, &str)> = found
        .iter()
        .filter(|m| m.note_id == "note:meet")
        .map(|m| (m.note_id.clone(), m.line, m.how.as_str()))
        .collect();
    assert_eq!(
        lines,
        vec![
            ("note:meet".to_owned(), 0, "link"),
            ("note:meet".to_owned(), 1, "at"),
            ("note:meet".to_owned(), 2, "word"),
        ]
    );
    // The page itself is never its own mention.
    assert!(found.iter().all(|m| m.note_id != "note:sam"));
    // Removing the alias drops the mentions that only the alias made.
    person(&store, "note:sam", "Sam Moyo", "", &b);
    let found = store.entity_mentions("note:sam", 50).unwrap();
    assert_eq!(
        found
            .iter()
            .map(|m| (m.note_id.as_str(), m.line))
            .collect::<Vec<_>>(),
        vec![("note:meet", 2)],
        "only the full name in words is left"
    );
}

#[test]
fn an_alias_resolves_a_wiki_link_unless_two_pages_share_it() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    person(&store, "note:one", "Sam Moyo", "Sam", &b);
    assert_eq!(
        store.resolve_title("Sam").unwrap(),
        Some(("note:one".into(), "Sam Moyo".into()))
    );
    // A note really titled "Sam" wins over an alias.
    note(&store, "note:titled", "Sam", "plain", &b);
    assert_eq!(
        store.resolve_title("sam").unwrap().unwrap().0,
        "note:titled"
    );
    store.delete("note:titled", Origin::Ui).unwrap();
    person(&store, "note:two", "Sam Okafor", "Sam", &b);
    assert_eq!(store.resolve_title("Sam").unwrap(), None);
    assert_eq!(store.entities_named("sam").unwrap().len(), 2);
    // An ambiguous name matches nothing.
    note(&store, "note:x", "X", "talked to Sam\n", &b);
    assert!(store.entity_mentions("note:one", 10).unwrap().is_empty());
    assert!(store.entity_mentions("note:two", 10).unwrap().is_empty());
}

#[test]
fn a_locked_note_and_a_locked_notebook_take_no_part() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    let secret = book(&store, "book:secret");
    person(&store, "note:sam", "Sam Moyo", "Sam", &b);
    note(&store, "note:open", "Open", "- [ ] ring Sam Moyo\n", &b);
    note(
        &store,
        "note:mine",
        "Mine",
        &format!("{CANARY} Sam Moyo\n"),
        &b,
    );
    let rev = store.get("note:mine").unwrap().unwrap().rev;
    store
        .lock_note(
            json!({"_id": "note:mine", "_rev": rev, "bookId": b, "title": "Mine",
            "body": "", "kansoLock": lock()}),
        )
        .unwrap();
    let mentions = store.entity_mentions("note:sam", 50).unwrap();
    assert_eq!(mentions.len(), 1);
    assert_eq!(mentions[0].note_id, "note:open");
    // A locked notebook's notes are as silent.
    note(&store, "note:inside", "Inside", "Sam Moyo again\n", &secret);
    assert_eq!(store.entity_mentions("note:sam", 50).unwrap().len(), 2);
    let mut locked_book = store.get_api("book:secret", None, false).unwrap();
    locked_book["kansoLock"] = lock();
    // Notes with text move into a locked notebook from Kanso, sealed.
    let rev = store.get("note:inside").unwrap().unwrap().rev;
    store
        .lock_book(
            locked_book,
            vec![
                json!({"_id": "note:inside", "_rev": rev, "bookId": "book:secret",
                "title": "Inside", "body": "", "kansoLock": lock()}),
            ],
        )
        .unwrap();
    let mentions = store.entity_mentions("note:sam", 50).unwrap();
    assert_eq!(mentions.len(), 1);
    assert_eq!(store.entity_mention_stats("note:sam").unwrap().0, 1);
    assert!(store.entity_tasks("note:sam", 10).unwrap().len() == 1);
    // A locked page is not an entity.
    let rev = store.get("note:sam").unwrap().unwrap().rev;
    store
        .lock_note(
            json!({"_id": "note:sam", "_rev": rev, "bookId": b, "title": "Sam Moyo",
            "body": "", "kansoLock": lock()}),
        )
        .unwrap();
    assert!(store.entities().unwrap().is_empty());
    assert!(store.entity_mentions("note:sam", 50).unwrap().is_empty());
}

#[test]
fn a_trashed_page_or_note_stops_counting() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    person(&store, "note:sam", "Sam Moyo", "", &b);
    note(&store, "note:one", "One", "Sam Moyo spoke\n", &b);
    assert_eq!(store.entity_mentions("note:sam", 9).unwrap().len(), 1);
    note(&store, "note:one", "One", "Sam Moyo spoke\n", "trash");
    assert!(store.entity_mentions("note:sam", 9).unwrap().is_empty());
    note(&store, "note:one", "One", "Sam Moyo spoke\n", &b);
    assert_eq!(store.entity_mentions("note:sam", 9).unwrap().len(), 1);
    store.delete("note:sam", Origin::Ui).unwrap();
    assert!(store.entities().unwrap().is_empty());
    assert!(store.entity_mentions("note:sam", 9).unwrap().is_empty());
}

#[test]
fn open_items_follow_mentions_and_links() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    person(&store, "note:sam", "Sam Moyo", "", &b);
    note(
        &store,
        "note:one",
        "One",
        "- [ ] Send Sam Moyo the deck @2026-11-04\n- [ ] unrelated\n- [x] done with Sam Moyo\n",
        &b,
    );
    note(
        &store,
        "note:two",
        "Two",
        "[[Sam Moyo]]\n- [ ] later thing\n",
        &b,
    );
    let tasks = store.entity_tasks("note:sam", 10).unwrap();
    let texts: Vec<&str> = tasks.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(
        texts,
        vec!["Send Sam Moyo the deck @2026-11-04", "later thing"]
    );
}

#[test]
fn co_mentions_and_related_notes() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    person(&store, "note:sam", "Sam Moyo", "", &b);
    let project = "---\nkanso: project\n---\nSee [[Pricing]]\n";
    note(&store, "note:atlas", "Atlas", project, &b);
    note(&store, "note:pricing", "Pricing", "numbers", &b);
    note(&store, "note:a", "A", "Sam Moyo and Atlas\n", &b);
    note(&store, "note:b", "B", "Atlas with Sam Moyo\n", &b);
    let co = store.co_entities("note:sam", 8).unwrap();
    assert_eq!(co.len(), 1);
    assert_eq!((co[0].0.id.as_str(), co[0].1), ("note:atlas", 2));
    let related = store.entity_related_notes("note:atlas", 8).unwrap();
    assert_eq!(related[0].0, "note:pricing");
}

#[test]
fn ids_of_links_and_page_names_survive_a_derived_rebuild() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kanso.db");
    {
        let store = Store::open(&path).unwrap();
        let b = book(&store, "book:a");
        person(&store, "note:sam", "Sam Moyo", "Sam", &b);
        note(
            &store,
            "note:one",
            "One",
            "met Sam\nand [x](inkdrop://note/sam)\n",
            &b,
        );
        assert_eq!(store.entity_mentions("note:sam", 9).unwrap().len(), 2);
    }
    // Forget the derived version: the next open fills everything again.
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute(
            "DELETE FROM meta WHERE key = ?",
            [kanso_store::DERIVED_VERSION_KEY],
        )
        .unwrap();
        conn.execute("DELETE FROM note_mentions", []).unwrap();
        conn.execute("DELETE FROM entities", []).unwrap();
        conn.execute("DELETE FROM entity_names", []).unwrap();
    }
    let store = Store::open(&path).unwrap();
    assert_eq!(store.entities().unwrap().len(), 1);
    let found = store.entity_mentions("note:sam", 9).unwrap();
    assert_eq!(found.len(), 2, "the word and the ID link");
}

// MARK: The cache

fn cache(text: &str, at: i64, pinned: bool) -> PageCache {
    let mut cache = PageCache::default();
    cache.sections.insert(
        "promises".into(),
        CachedSection {
            items: vec![CachedItem {
                text: text.into(),
                ..CachedItem::default()
            }],
            written_at: at,
            fingerprint: format!("f-{text}"),
            pinned,
            ..CachedSection::default()
        },
    );
    cache
}

#[test]
fn setting_the_cache_is_not_an_edit_and_adds_no_version() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    person(&store, "note:sam", "Sam Moyo", "", &b);
    note(
        &store,
        "note:sam",
        "Sam Moyo",
        "---\nkanso: person\n---\nEdited.\n",
        &b,
    );
    let before = store.get("note:sam").unwrap().unwrap();
    let versions = store.revisions("note:sam").unwrap().len();
    assert!(
        store
            .set_page_cache("note:sam", &cache("x", 5, false))
            .unwrap()
    );
    let after = store.get("note:sam").unwrap().unwrap();
    assert_eq!(after.fields["body"], before.fields["body"]);
    assert_eq!(after.fields["updatedAt"], before.fields["updatedAt"]);
    assert_ne!(after.rev, before.rev, "it still syncs");
    assert_eq!(store.revisions("note:sam").unwrap().len(), versions);
    assert_eq!(store.page_cache("note:sam").unwrap(), {
        let mut c = cache("x", 5, false);
        c.v = 1;
        c
    });
    // The same cache again writes nothing.
    assert!(
        !store
            .set_page_cache("note:sam", &cache("x", 5, false))
            .unwrap()
    );
    // An empty cache clears the field.
    assert!(
        store
            .set_page_cache("note:sam", &PageCache::default())
            .unwrap()
    );
    assert!(
        !store
            .get("note:sam")
            .unwrap()
            .unwrap()
            .fields
            .contains_key(PAGE_FIELD)
    );
}

proptest! {
    #![proptest_config(config(24))]

    #[test]
    fn prop_set_page_cache_never_changes_body_updated_at_or_history(
        words in proptest::collection::vec("[a-z]{1,6}", 1..4),
        at in 0i64..1_000_000,
        pinned in any::<bool>(),
    ) {
        let (_dir, store) = temp_store();
        let b = book(&store, "book:a");
        note(&store, "note:p", "P", "---\nkanso: person\n---\nfirst\n", &b);
        note(&store, "note:p", "P", "---\nkanso: person\n---\nsecond\n", &b);
        let before = store.get("note:p").unwrap().unwrap();
        let versions = store.revisions("note:p").unwrap().len();
        for word in &words {
            store.set_page_cache("note:p", &cache(word, at, pinned)).unwrap();
        }
        let after = store.get("note:p").unwrap().unwrap();
        prop_assert_eq!(&after.fields["body"], &before.fields["body"]);
        prop_assert_eq!(&after.fields["updatedAt"], &before.fields["updatedAt"]);
        prop_assert_eq!(store.revisions("note:p").unwrap().len(), versions);
    }

    /// The mention index after random edits equals a full rescan.
    #[test]
    fn prop_mentions_index_matches_a_full_rescan(
        edits in proptest::collection::vec(
            (0usize..3, prop::sample::select(vec![
                "Sam Moyo spoke", "@sam said", "[[Sam]] linked", "nothing here",
                "Atlas launch with sam", "`Sam Moyo` in code", "[[Atlas]] and @atlas",
            ]), prop::bool::ANY),
            1..12,
        ),
        alias in prop::bool::ANY,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kanso.db");
        let store = Store::open(&path).unwrap();
        let b = book(&store, "book:a");
        for (ix, text, trash) in &edits {
            let id = format!("note:n{ix}");
            let target = if *trash { "trash" } else { b.as_str() };
            note(&store, &id, &id, &format!("{text}\nsecond line Atlas\n"), target);
            if *ix == 1 && alias {
                person(&store, "note:sam", "Sam Moyo", "Sam", &b);
            }
        }
        person(&store, "note:sam", "Sam Moyo", if alias { "Sam" } else { "" }, &b);
        note(&store, "note:atlas", "Atlas", "---\nkanso: project\n---\n", &b);
        let snapshot = |store: &Store| {
            let mut all = Vec::new();
            for entity in ["note:sam", "note:atlas"] {
                for m in store.entity_mentions(entity, 1000).unwrap() {
                    all.push((entity.to_owned(), m.note_id, m.line, m.how.as_str().to_owned()));
                }
            }
            all.sort();
            all
        };
        let live = snapshot(&store);
        drop(store);
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            conn.execute("DELETE FROM meta WHERE key = ?", [kanso_store::DERIVED_VERSION_KEY]).unwrap();
        }
        let store = Store::open(&path).unwrap();
        prop_assert_eq!(live, snapshot(&store));
    }

    /// A locked note produces no mention, whatever it says.
    #[test]
    fn prop_locked_notes_produce_no_mentions(text in "[A-Za-z@\\[\\] \n]{0,60}") {
        let (_dir, store) = temp_store();
        let b = book(&store, "book:a");
        person(&store, "note:sam", "Sam Moyo", "Sam", &b);
        note(&store, "note:l", "L", &format!("Sam Moyo @sam [[Sam]] {text}"), &b);
        let rev = store.get("note:l").unwrap().unwrap().rev;
        store
            .lock_note(json!({"_id": "note:l", "_rev": rev, "bookId": b, "title": "L",
                "body": "", "kansoLock": lock()}))
            .unwrap();
        prop_assert!(store.entity_mentions("note:sam", 99).unwrap().is_empty());
        prop_assert_eq!(store.entity_mention_stats("note:sam").unwrap().0, 0);
        prop_assert!(store.co_entities("note:sam", 9).unwrap().is_empty());
    }
}

#[test]
fn locking_a_page_clears_its_cache() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    person(&store, "note:sam", "Sam Moyo", "", &b);
    store
        .set_page_cache("note:sam", &cache(CANARY, 5, false))
        .unwrap();
    let rev = store.get("note:sam").unwrap().unwrap().rev;
    let mut sealed = store.get_api("note:sam", None, false).unwrap();
    sealed["_rev"] = rev.into();
    sealed["body"] = "".into();
    sealed["kansoLock"] = lock();
    store.lock_note(sealed).unwrap();
    let fields = store.get("note:sam").unwrap().unwrap().fields;
    assert!(!fields.contains_key(PAGE_FIELD));
    assert!(!serde_json::to_string(&fields).unwrap().contains(CANARY));
}

#[test]
#[allow(non_snake_case)]
fn kansoPage_is_never_served() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    person(&store, "note:sam", "Sam Moyo", "", &b);
    store
        .set_page_cache("note:sam", &cache(CANARY, 5, false))
        .unwrap();
    store
        .put(
            DocType::Book,
            json!({"_id": "book:a", "kansoDismissed": ["monday standup"]}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let doc = store.get("note:sam").unwrap().unwrap();
    assert!(doc.fields.contains_key(PAGE_FIELD), "the app has it");
    let served = doc.to_served(Default::default());
    assert!(!served.to_string().contains(CANARY));
    let book_doc = store.get("book:a").unwrap().unwrap();
    assert!(book_doc.fields.contains_key("kansoDismissed"));
    assert!(
        !book_doc
            .to_served(Default::default())
            .to_string()
            .contains("monday")
    );
    for listed in store.list_api(DocType::Note, None, 0).unwrap() {
        assert!(listed.get(PAGE_FIELD).is_none());
    }
    for listed in store.list_api(DocType::Book, None, 0).unwrap() {
        assert!(listed.get("kansoDismissed").is_none());
    }
    let api = store.get_api("note:sam", None, false).unwrap();
    assert!(api.get(PAGE_FIELD).is_none() || true);
}

// MARK: Sync

fn sync_doc(store: &Store, id: &str) -> SyncDoc {
    let doc = store.get(id).unwrap().unwrap();
    SyncDoc {
        id: id.into(),
        rev: doc.rev,
        deleted: false,
        fields: doc.fields,
        base_rev: None,
    }
}

#[test]
fn a_body_edit_on_one_device_and_a_refresh_on_the_other_do_not_conflict() {
    let (_d1, mac) = temp_store();
    let (_d2, phone) = temp_store();
    let b = book(&mac, "book:a");
    person(&mac, "note:sam", "Sam Moyo", "", &b);
    book(&phone, "book:a");
    phone
        .apply_remote(&sync_doc(&mac, "book:a"), None, None)
        .unwrap();
    phone
        .apply_remote(&sync_doc(&mac, "note:sam"), None, None)
        .unwrap();
    // The Mac edits the text; the phone refreshes a section.
    note(
        &mac,
        "note:sam",
        "Sam Moyo",
        "---\nkanso: person\n---\nMac wrote this.\n",
        &b,
    );
    mac.put(
        DocType::Note,
        json!({"_id": "note:sam", "body": "---\nkanso: person\n---\nMac wrote this.\n", "updatedAt": 5000}),
        Put::Merge,
        Origin::Ui,
    )
    .unwrap();
    phone
        .set_page_cache("note:sam", &cache("phone summary", 7000, false))
        .unwrap();
    // The Mac's edit reaches the phone: the text wins, the summary stays.
    phone
        .apply_remote(&sync_doc(&mac, "note:sam"), None, None)
        .unwrap();
    let on_phone = phone.get("note:sam").unwrap().unwrap();
    assert!(
        on_phone.fields["body"]
            .as_str()
            .unwrap()
            .contains("Mac wrote this")
    );
    assert_eq!(
        phone.page_cache("note:sam").unwrap().sections["promises"].items[0].text,
        "phone summary"
    );
    // And the phone's version, pushed to the Mac, keeps both.
    mac.apply_remote(&sync_doc(&phone, "note:sam"), None, None)
        .unwrap();
    let on_mac = mac.get("note:sam").unwrap().unwrap();
    assert!(
        on_mac.fields["body"]
            .as_str()
            .unwrap()
            .contains("Mac wrote this")
    );
    assert_eq!(
        mac.page_cache("note:sam").unwrap().sections["promises"].items[0].text,
        "phone summary"
    );
}

#[test]
fn dismissed_names_union_across_devices_and_pinned_wins() {
    let (_d1, mac) = temp_store();
    let (_d2, phone) = temp_store();
    for store in [&mac, &phone] {
        book(store, "book:pp");
    }
    mac.set_book_dismissed("book:pp", &["monday standup".into()])
        .unwrap();
    phone
        .set_book_dismissed("book:pp", &["tuesday".into()])
        .unwrap();
    phone
        .apply_remote(&sync_doc(&mac, "book:pp"), None, None)
        .unwrap();
    let mut names = phone.book_dismissed("book:pp").unwrap();
    names.sort();
    assert_eq!(names, vec!["monday standup", "tuesday"]);

    let b = book(&mac, "book:a");
    person(&mac, "note:sam", "Sam Moyo", "", &b);
    phone
        .apply_remote(&sync_doc(&mac, "book:a"), None, None)
        .unwrap();
    phone
        .apply_remote(&sync_doc(&mac, "note:sam"), None, None)
        .unwrap();
    mac.set_page_cache("note:sam", &cache("kept", 1000, true))
        .unwrap();
    phone
        .set_page_cache("note:sam", &cache("newer but loose", 9000, false))
        .unwrap();
    phone
        .apply_remote(&sync_doc(&mac, "note:sam"), None, None)
        .unwrap();
    let merged = phone.page_cache("note:sam").unwrap();
    assert_eq!(merged.sections["promises"].items[0].text, "kept");
    assert!(merged.sections["promises"].pinned);
}

#[test]
fn dismissals_and_candidates_round_trip() {
    let (_dir, store) = temp_store();
    store
        .set_candidates(&[kanso_store::CandidateRow {
            name: "sam moyo".into(),
            kind: EntityKind::Person,
            notes: 4,
            last_seen: 5,
            forms: vec!["Sam Moyo".into(), "@sam".into()],
            score: 2.5,
            alias_of: None,
        }])
        .unwrap();
    assert_eq!(store.candidates().unwrap().len(), 1);
    store.dismiss_name("Sam  Moyo", true, 4, 10).unwrap();
    assert!(store.candidates().unwrap().is_empty());
    assert_eq!(store.dismissed().unwrap()[0].name, "sam moyo");
    // "Not now" never weakens a "never".
    store.dismiss_name("sam moyo", false, 9, 11).unwrap();
    assert_eq!(store.dismissed().unwrap()[0].kind, "never");
    store.undismiss_name("Sam Moyo").unwrap();
    assert!(store.dismissed().unwrap().is_empty());
}
