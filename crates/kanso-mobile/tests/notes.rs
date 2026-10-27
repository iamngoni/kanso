//! The note calls of `Library`: lists, one note, saving, creating, trashing,
//! notebooks, tags, search, properties and query views.

mod common;

use std::collections::BTreeSet;
use std::sync::Arc;

use common::*;
use kanso_mobile::{
    KansoError, ListPlace, ListQuery, LockState, NewNote, PropertyKind, Range16, SearchScope,
    SortKey,
};
use kanso_sync::MemoryServer;
use proptest::prelude::*;
use serde_json::json;

fn server() -> Arc<MemoryServer> {
    Arc::new(MemoryServer::new())
}

/// A note the property test seeds: where it is and what it carries.
#[derive(Clone, Debug)]
struct Seed {
    book: usize,
    tagged: bool,
    status: &'static str,
    trashed: bool,
}

fn seeds() -> impl Strategy<Value = Vec<Seed>> {
    proptest::collection::vec(
        (
            0usize..2,
            any::<bool>(),
            prop::sample::select(vec!["none", "active", "onHold", "completed", "dropped"]),
            any::<bool>(),
        )
            .prop_map(|(book, tagged, status, trashed)| Seed {
                book,
                tagged,
                status,
                trashed,
            }),
        0..9,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Whatever notes exist, a list holds exactly the notes its filters name
    /// (outside the trash unless the trash is asked for), and sorting by
    /// title orders titles, either way round.
    #[test]
    fn prop_lists_hold_what_their_filters_name(seeds in seeds()) {
        let mut ids = Vec::new();
        let mut books = Vec::new();
        let mut the_tag = String::new();
        let phone = phone_with(&server(), |store| {
            books = vec![book(store, "One"), book(store, "Two")];
            the_tag = tag(store, "work");
            for (i, seed) in seeds.iter().enumerate() {
                ids.push(raw_note(store, json!({
                    "title": format!("Note {i:02}"),
                    "body": format!("body {i}"),
                    "bookId": if seed.trashed { "trash".to_owned() } else { books[seed.book].clone() },
                    "tags": if seed.tagged { vec![the_tag.clone()] } else { vec![] },
                    "status": seed.status,
                })));
            }
        });
        let ids_where = |keep: &dyn Fn(&Seed) -> bool| -> BTreeSet<String> {
            seeds.iter().zip(&ids).filter(|(s, _)| keep(s)).map(|(_, id)| id.clone()).collect()
        };
        let listed = |query: ListQuery| -> BTreeSet<String> {
            phone.library.list(query).unwrap().rows.into_iter().map(|row| row.id).collect()
        };

        prop_assert_eq!(listed(ListQuery::default()), ids_where(&|s| !s.trashed));
        prop_assert_eq!(
            listed(ListQuery { place: ListPlace::Trash, ..ListQuery::default() }),
            ids_where(&|s| s.trashed)
        );
        prop_assert_eq!(
            listed(ListQuery { place: ListPlace::Notebook { id: books[1].clone() }, ..ListQuery::default() }),
            ids_where(&|s| !s.trashed && s.book == 1)
        );
        prop_assert_eq!(
            listed(ListQuery { place: ListPlace::Tag { id: the_tag.clone() }, ..ListQuery::default() }),
            ids_where(&|s| !s.trashed && s.tagged)
        );
        prop_assert_eq!(
            listed(ListQuery { status_only: Some("active".into()), ..ListQuery::default() }),
            ids_where(&|s| !s.trashed && s.status == "active")
        );

        // Titles ascending, then descending: the same notes, reversed.
        let by_title = |descending: bool| {
            phone.library.list(ListQuery {
                sort: SortKey::Title, descending, ..ListQuery::default()
            }).unwrap().rows
        };
        let all = by_title(false);
        let titles: Vec<_> = all.iter().map(|row| row.title.clone()).collect();
        let mut sorted = titles.clone();
        sorted.sort();
        prop_assert_eq!(&titles, &sorted);
        let mut backwards: Vec<_> = by_title(true).into_iter().map(|row| row.title).collect();
        backwards.reverse();
        prop_assert_eq!(backwards, titles);
        prop_assert_eq!(all.len(), ids_where(&|s| !s.trashed).len());
    }
}

#[test]
fn a_status_that_is_not_one_is_refused() {
    let phone = lone_phone();
    let err = phone
        .library
        .list(ListQuery {
            status_only: Some("someday".into()),
            ..ListQuery::default()
        })
        .unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
}

// no-prop-test: the shape of a row; the filters are covered by property above.
#[test]
fn a_row_carries_what_a_list_shows() {
    let mut book_id = String::new();
    let phone = phone_with(&server(), |store| {
        book_id = book(store, "Projects");
        raw_note(
            store,
            json!({"title": "Plan", "body": "- [x] one\n- [ ] two\n", "bookId": book_id, "pinned": true, "status": "active"}),
        );
    });
    let rows = phone.library.list(ListQuery::default()).unwrap().rows;
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.title, "Plan");
    assert!(row.snippet.contains("one"));
    assert_eq!(row.notebook_id, book_id);
    assert_eq!(row.status, "active");
    assert!(row.pinned);
    assert!(!row.locked);
    assert_eq!((row.tasks_done, row.tasks_total), (1, 2));
    assert!(row.updated_ms >= row.created_ms && row.created_ms > 0);
    assert!(row.rev.contains('-'));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Saving with the revision you read works and returns the next one;
    /// saving with an older one writes nothing and says the current one.
    /// What `note` returns is always the last text saved.
    #[test]
    fn prop_saves_follow_revisions(bodies in proptest::collection::vec("[a-zA-Z0-9 \n]{0,60}", 1..6)) {
        let phone = lone_phone();
        let id = note(&phone.library, "Draft", "first");
        let mut last = phone.library.note(&id).unwrap();
        prop_assert_eq!(last.body.as_deref(), Some("first"));
        let mut previous_revs = vec![last.rev.clone()];
        for body in &bodies {
            let saved = phone.library.save(&id, &last.rev, body.clone()).unwrap();
            prop_assert_eq!(&saved.id, &id);
            prop_assert!(!previous_revs.contains(&saved.rev));
            // A stale revision is refused, and nothing changes.
            let stale = previous_revs.last().unwrap().clone();
            let err = phone.library.save(&id, &stale, "lost".into()).unwrap_err();
            prop_assert_eq!(err, KansoError::Conflict { current_rev: saved.rev.clone() });
            last = phone.library.note(&id).unwrap();
            prop_assert_eq!(last.body.as_deref(), Some(body.as_str()));
            prop_assert_eq!(&last.rev, &saved.rev);
            previous_revs.push(saved.rev);
        }
    }
}

// no-prop-test: the fields of a new note and its default notebook.
#[test]
fn creating_makes_a_notebook_when_there_is_none_and_reads_back() {
    let phone = lone_phone();
    assert!(phone.library.notebooks().unwrap().is_empty());
    let saved = phone
        .library
        .create(NewNote {
            notebook_id: None,
            title: "Hello".into(),
            body: "# Hi\n- [ ] task".into(),
            ..NewNote::default()
        })
        .unwrap();
    let books = phone.library.notebooks().unwrap();
    assert_eq!(books.len(), 1);
    assert_eq!(books[0].name, "Notes");
    assert_eq!(books[0].count, 1);
    assert!(!books[0].locked);
    let view = phone.library.note(&saved.id).unwrap();
    assert_eq!(view.rev, saved.rev);
    assert_eq!(view.title, "Hello");
    assert_eq!(view.body.as_deref(), Some("# Hi\n- [ ] task"));
    assert_eq!(view.lock, LockState::Plain);
    assert_eq!(view.notebook_id, books[0].id);
    // A second note goes in the same notebook; a made-up one is refused.
    let second = phone.library.create(NewNote::default()).unwrap();
    assert_eq!(
        phone.library.note(&second.id).unwrap().notebook_id,
        books[0].id
    );
    let err = phone
        .library
        .create(NewNote {
            notebook_id: Some("book:nowhere".into()),
            ..NewNote::default()
        })
        .unwrap_err();
    assert_eq!(err, KansoError::NotFound);
    // The task counts follow the text.
    let rows = phone.library.list(ListQuery::default()).unwrap().rows;
    let row = rows.iter().find(|row| row.id == saved.id).unwrap();
    assert_eq!((row.tasks_done, row.tasks_total), (0, 1));
}

/// Where the property test makes a note from.
#[derive(Clone, Debug)]
enum From {
    Nowhere,
    All,
    Pinned,
    Trash,
    Notebook(usize),
    Tag(usize),
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, ..ProptestConfig::default() })]

    /// A new note lands where the engine says: its own notebook when made
    /// from one, else the preferred notebook when it exists, else the first.
    /// A tag's list tags it, Pinned pins it, and nothing goes to the trash.
    #[test]
    fn prop_a_new_note_goes_where_its_place_and_preference_say(
        from in prop_oneof![
            Just(From::Nowhere),
            Just(From::All),
            Just(From::Pinned),
            Just(From::Trash),
            (0usize..3).prop_map(From::Notebook),
            (0usize..2).prop_map(From::Tag),
        ],
        preferred in prop::option::of(0usize..4),
        extra_pin in any::<bool>(),
        extra_tag in any::<bool>(),
    ) {
        let mut books = Vec::new();
        let mut tags = Vec::new();
        let phone = phone_with(&server(), |store| {
            for name in ["One", "Two", "Three"] {
                books.push(book(store, name));
            }
            for name in ["a", "b"] {
                tags.push(tag(store, name));
            }
        });
        let preferred_id = preferred.map(|ix| books.get(ix).cloned().unwrap_or_else(|| "book:gone".into()));
        let place = match &from {
            From::Nowhere => None,
            From::All => Some(ListPlace::All),
            From::Pinned => Some(ListPlace::Pinned),
            From::Trash => Some(ListPlace::Trash),
            From::Notebook(ix) => Some(ListPlace::Notebook { id: books[*ix].clone() }),
            From::Tag(ix) => Some(ListPlace::Tag { id: tags[*ix].clone() }),
        };
        let saved = phone.library.create(NewNote {
            title: "T".into(),
            place,
            preferred_notebook_id: preferred_id.clone(),
            tags: if extra_tag { vec![tags[1].clone()] } else { Vec::new() },
            pinned: extra_pin,
            ..NewNote::default()
        }).unwrap();
        let view = phone.library.note(&saved.id).unwrap();
        let first = phone.library.notebooks().unwrap()[0].id.clone();
        let expected_book = match (&from, &preferred_id) {
            (From::Notebook(ix), _) => books[*ix].clone(),
            (_, Some(id)) if books.contains(id) => id.clone(),
            _ => first,
        };
        prop_assert_eq!(&view.notebook_id, &expected_book);
        prop_assert_ne!(&view.notebook_id, "trash");
        let mut expected_tags = BTreeSet::new();
        if let From::Tag(ix) = from {
            expected_tags.insert(tags[ix].clone());
        }
        if extra_tag {
            expected_tags.insert(tags[1].clone());
        }
        prop_assert_eq!(view.tags.iter().cloned().collect::<BTreeSet<_>>(), expected_tags);
        prop_assert_eq!(view.pinned, extra_pin || matches!(from, From::Pinned));
    }
}

// no-prop-test: the library with no notebook, and the refusals around one.
#[test]
fn a_new_note_in_an_empty_library_makes_the_notes_notebook_whatever_the_preference() {
    let phone = lone_phone();
    let saved = phone
        .library
        .create(NewNote {
            preferred_notebook_id: Some("book:gone".into()),
            place: Some(ListPlace::Trash),
            ..NewNote::default()
        })
        .unwrap();
    let books = phone.library.notebooks().unwrap();
    assert_eq!(books.len(), 1);
    assert_eq!(books[0].name, "Notes");
    assert_eq!(
        phone.library.note(&saved.id).unwrap().notebook_id,
        books[0].id
    );
    // An explicit notebook still has to exist, preference or not.
    let err = phone
        .library
        .create(NewNote {
            notebook_id: Some("book:nowhere".into()),
            preferred_notebook_id: Some(books[0].id.clone()),
            ..NewNote::default()
        })
        .unwrap_err();
    assert_eq!(err, KansoError::NotFound);
}

// no-prop-test: the errors of a missing note and of a text over the limit.
#[test]
fn a_missing_note_is_not_found_and_too_long_a_title_is_invalid() {
    let phone = lone_phone();
    for id in ["note:missing", "book:x", "not an id", ""] {
        assert_eq!(phone.library.note(id), Err(KansoError::NotFound), "{id}");
        assert_eq!(
            phone.library.save(id, "1-a", "x".into()),
            Err(KansoError::NotFound)
        );
        assert_eq!(phone.library.trash(id), Err(KansoError::NotFound));
        assert_eq!(phone.library.properties(id), Err(KansoError::NotFound));
    }
    let err = phone
        .library
        .create(NewNote {
            title: "t".repeat(500),
            ..NewNote::default()
        })
        .unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
}

// no-prop-test: one move, read three ways.
#[test]
fn trashing_moves_a_note_out_of_the_lists_and_into_the_trash() {
    let phone = lone_phone();
    let keep = note(&phone.library, "Keep", "stays");
    let gone = note(&phone.library, "Gone", "goes");
    phone.library.trash(&gone).unwrap();
    let listed: Vec<_> = phone
        .library
        .list(ListQuery::default())
        .unwrap()
        .rows
        .into_iter()
        .map(|row| row.id)
        .collect();
    assert_eq!(listed, vec![keep]);
    let trash = phone
        .library
        .list(ListQuery {
            place: ListPlace::Trash,
            ..ListQuery::default()
        })
        .unwrap()
        .rows;
    assert_eq!(trash.len(), 1);
    assert_eq!(trash[0].id, gone);
    assert_eq!(phone.library.note(&gone).unwrap().notebook_id, "trash");
}

// no-prop-test: the shape of the sidebar's rows.
#[test]
fn notebooks_and_tags_come_as_the_sidebar_shows_them() {
    let (mut parent, mut child, mut work) = (String::new(), String::new(), String::new());
    let phone = phone_with(&server(), |store| {
        parent = book(store, "Parent");
        child = store
            .put(
                kanso_core::DocType::Book,
                json!({"name": "Child", "parentBookId": parent, "kansoIcon": "rocket", "kansoColor": "#2185d0"}),
                kanso_store::Put::Upsert,
                kanso_store::Origin::Ui,
            )
            .unwrap()
            .id;
        work = tag(store, "work");
        raw_note(
            store,
            json!({"title": "A", "bookId": child, "tags": [work]}),
        );
        raw_note(
            store,
            json!({"title": "B", "bookId": child, "tags": [work]}),
        );
    });
    let books = phone.library.notebooks().unwrap();
    let names: Vec<_> = books.iter().map(|b| b.name.as_str()).collect();
    // Tree order: the parent, then what is inside it.
    assert_eq!(names, vec!["Parent", "Child"]);
    assert_eq!((books[0].depth, books[1].depth), (0, 1));
    assert_eq!(books[0].role, None);
    let child_row = &books[1];
    assert_eq!(child_row.parent_id.as_deref(), Some(parent.as_str()));
    assert_eq!(child_row.count, 2);
    assert_eq!(child_row.icon.as_deref(), Some("rocket"));
    assert_eq!(child_row.color.as_deref(), Some("#2185d0"));
    assert!(!child_row.locked);
    let tags = phone.library.tags().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!((tags[0].name.as_str(), tags[0].count), ("work", 2));
    assert_eq!(tags[0].id, work);
    assert_eq!(tags[0].color, kanso_core::TagColor::Default.rgb());
}

// no-prop-test: the search language is the core's; this checks the facade's
// part, the hit's ranges, which `matching.rs` covers by property.
#[test]
fn search_finds_words_and_says_where_they_matched() {
    let phone = lone_phone();
    let hit = note(
        &phone.library,
        "Café notes",
        "We met at the café on Tuesday. Café again.",
    );
    note(&phone.library, "Other", "nothing here");
    let hits = phone.library.search("café", SearchScope::All, 0).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].row.id, hit);
    // "Café" is four UTF-16 units at the start of the title.
    assert_eq!(hits[0].title_ranges, vec![Range16 { start: 0, end: 4 }]);
    let snippet = &hits[0].row.snippet;
    assert!(!hits[0].snippet_ranges.is_empty());
    let units: Vec<u16> = snippet.encode_utf16().collect();
    for range in &hits[0].snippet_ranges {
        let word = String::from_utf16(&units[range.start as usize..range.end as usize]).unwrap();
        assert_eq!(word.to_lowercase(), "café");
    }
    assert!(
        phone
            .library
            .search("   ", SearchScope::All, 0)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        phone
            .library
            .search("tuesday", SearchScope::All, 1)
            .unwrap()
            .len(),
        1
    );
    assert!(
        phone
            .library
            .search("zebra", SearchScope::All, 0)
            .unwrap()
            .is_empty()
    );
    // A phrase and a field.
    assert_eq!(
        phone
            .library
            .search("title:other", SearchScope::All, 0)
            .unwrap()
            .len(),
        1
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// The properties of a note's front matter come back as written: each
    /// key once, in order, with its text.
    #[test]
    fn prop_front_matter_comes_back_as_written(
        pairs in proptest::collection::btree_map("k[a-z]{1,5}", "v[a-z]{1,6}", 0..5),
    ) {
        let phone = lone_phone();
        let mut body = String::from("---\n");
        for (key, value) in &pairs {
            body.push_str(&format!("{key}: {value}\n"));
        }
        body.push_str("---\ntext\n");
        let id = note(&phone.library, "P", &body);
        let found = phone.library.properties(&id).unwrap();
        let expected: Vec<_> = pairs.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let got: Vec<_> = found.iter().map(|p| (p.key.clone(), p.text.clone())).collect();
        prop_assert_eq!(got, expected);
        prop_assert!(found.iter().all(|p| p.kind == PropertyKind::Text && p.items.is_empty()));
    }
}

// no-prop-test: the kinds of value a front matter holds.
#[test]
fn properties_have_kinds() {
    let phone = lone_phone();
    let id = note(
        &phone.library,
        "P",
        "---\nrating: 4.5\ndone: true\ndue: 2026-10-25\ntags: [a, b]\nowner: Ada\nmap: {x: 1}\n---\nbody",
    );
    let props = phone.library.properties(&id).unwrap();
    let kinds: Vec<_> = props.iter().map(|p| (p.key.as_str(), p.kind)).collect();
    assert_eq!(
        kinds,
        vec![
            ("rating", PropertyKind::Number),
            ("done", PropertyKind::Bool),
            ("due", PropertyKind::Date),
            ("tags", PropertyKind::List),
            ("owner", PropertyKind::Text),
            ("map", PropertyKind::Raw),
        ]
    );
    assert_eq!(props[3].items, vec!["a", "b"]);
    assert_eq!(props[3].text, "a, b");
    assert!(
        phone
            .library
            .properties(&note(&phone.library, "None", "no front matter"))
            .unwrap()
            .is_empty()
    );
}

// no-prop-test: the query language is the core's; this checks the facade's
// table and its refusals.
#[test]
fn a_query_view_lists_matching_notes_as_a_table() {
    let phone = lone_phone();
    let a = note(&phone.library, "Alpha", "---\nstatus: open\nrank: 2\n---\n");
    let b = note(&phone.library, "Beta", "---\nstatus: open\nrank: 1\n---\n");
    note(&phone.library, "Gamma", "---\nstatus: closed\n---\n");
    let table = phone
        .library
        .query_view(
            "from: status:none has:rank\ncolumns: rank, status\nsort: rank asc",
            None,
        )
        .unwrap();
    assert_eq!(table.columns, vec!["Title", "rank", "status"]);
    let rows: Vec<_> = table
        .rows
        .iter()
        .map(|r| (r.id.clone(), r.title.clone()))
        .collect();
    assert_eq!(
        rows,
        vec![(b.clone(), "Beta".into()), (a.clone(), "Alpha".into())]
    );
    assert_eq!(
        table.rows[0].cells,
        vec![Some("1".into()), Some("open".into())]
    );
    // Leaving a note out, and a query that doesn't parse.
    let table = phone
        .library
        .query_view("from: status:none has:rank", Some(a))
        .unwrap();
    assert_eq!(table.rows.len(), 1);
    let err = phone.library.query_view("nonsense", None).unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
}
