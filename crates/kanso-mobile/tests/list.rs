//! The lists of the mobile app: places, counts, changes to several notes,
//! scoped search, notebooks in tree order, and the tags a note's text gives
//! it.

mod common;

use std::collections::BTreeSet;
use std::sync::Arc;

use common::*;
use kanso_core::model::LOCK_FIELD;
use kanso_core::now_ms;
use kanso_mobile::{
    DateField, KansoError, ListPlace, ListQuery, NewNote, NoteChange, NotebookRole, SearchScope,
    SortKey,
};
use kanso_sync::MemoryServer;
use proptest::prelude::*;
use serde_json::json;

const DAY: i64 = 86_400_000;

fn server() -> Arc<MemoryServer> {
    Arc::new(MemoryServer::new())
}

/// A note the properties seed.
#[derive(Clone, Debug)]
struct Seed {
    book: usize,
    pinned: bool,
    trashed: bool,
    /// Days since it was changed.
    age: i64,
}

fn seeds() -> impl Strategy<Value = Vec<Seed>> {
    proptest::collection::vec(
        (0usize..2, any::<bool>(), any::<bool>(), 0i64..12).prop_map(
            |(book, pinned, trashed, age)| Seed {
                book,
                pinned,
                trashed,
                age,
            },
        ),
        0..10,
    )
}

fn seeded(seeds: &[Seed]) -> (Phone, Vec<String>, Vec<String>) {
    let mut ids = Vec::new();
    let mut books = Vec::new();
    let now = now_ms();
    let phone = phone_with(&server(), |store| {
        books = vec![book(store, "One"), book(store, "Two")];
        for (i, seed) in seeds.iter().enumerate() {
            let at = now - seed.age * DAY - 60_000;
            ids.push(raw_note(
                store,
                json!({
                    "title": format!("Note {i:02}"),
                    "body": format!("body {i}"),
                    "bookId": if seed.trashed { "trash".to_owned() } else { books[seed.book].clone() },
                    "pinned": seed.pinned,
                    "createdAt": at,
                    "updatedAt": at,
                }),
            ));
        }
    });
    (phone, ids, books)
}

fn ids_of(rows: &[kanso_mobile::NoteRow]) -> Vec<String> {
    rows.iter().map(|row| row.id.clone()).collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// The library's counts are the lengths of its places' lists, and each
    /// place holds what it names: Pinned the pinned notes outside the trash,
    /// Recent those changed in the last seven days, newest first.
    #[test]
    fn prop_counts_are_the_lengths_of_the_places(seeds in seeds()) {
        let (phone, ids, _) = seeded(&seeds);
        let library = &phone.library;
        let list = |place: ListPlace| library.list(ListQuery { place, ..ListQuery::default() }).unwrap();
        let overview = library.overview().unwrap();
        let live = |keep: &dyn Fn(&Seed) -> bool| -> BTreeSet<String> {
            seeds.iter().zip(&ids).filter(|(s, _)| !s.trashed && keep(s)).map(|(_, id)| id.clone()).collect()
        };
        let set = |rows: &[kanso_mobile::NoteRow]| ids_of(rows).into_iter().collect::<BTreeSet<_>>();

        let all = list(ListPlace::All);
        prop_assert_eq!(overview.all as usize, all.rows.len());
        prop_assert_eq!(set(&all.rows), live(&|_| true));

        let pinned = list(ListPlace::Pinned);
        prop_assert_eq!(overview.pinned as usize, pinned.rows.len());
        prop_assert_eq!(set(&pinned.rows), live(&|s| s.pinned));

        let recent = list(ListPlace::Recent);
        prop_assert_eq!(overview.recent as usize, recent.rows.len());
        prop_assert_eq!(set(&recent.rows), live(&|s| s.age < 7));
        prop_assert!(recent.rows.windows(2).all(|w| w[0].updated_ms >= w[1].updated_ms));

        let trash = list(ListPlace::Trash);
        prop_assert_eq!(overview.trash as usize, trash.rows.len());
        prop_assert_eq!(trash.rows.len(), seeds.iter().filter(|s| s.trashed).count());
    }

    /// Pinned notes come first and keep the sort within each group; narrowed
    /// to pinned notes, a list holds the pinned ones in the same order; and
    /// Recent is by edit time whatever the sort asked for.
    #[test]
    fn prop_pinned_first_and_recent_ignores_the_sort(seeds in seeds(), descending: bool, sort in 0usize..3) {
        let (phone, _, _) = seeded(&seeds);
        let library = &phone.library;
        let sort = [SortKey::Updated, SortKey::Created, SortKey::Title][sort];
        let query = ListQuery { sort, descending, ..ListQuery::default() };
        let all = library.list(query.clone()).unwrap();
        let pins: Vec<bool> = all.rows.iter().map(|r| r.pinned).collect();
        let first_unpinned = pins.iter().position(|p| !p).unwrap_or(pins.len());
        prop_assert!(pins[first_unpinned..].iter().all(|p| !p), "{:?}", pins);

        let only = library.list(ListQuery { pinned_only: true, ..query.clone() }).unwrap();
        let expected: Vec<String> = all.rows.iter().filter(|r| r.pinned).map(|r| r.id.clone()).collect();
        prop_assert_eq!(ids_of(&only.rows), expected);
        prop_assert!(!only.shape.group_pinned);
        prop_assert!(all.shape.group_pinned);

        let recent = library.list(ListQuery { place: ListPlace::Recent, ..query }).unwrap();
        let plain = library.list(ListQuery { place: ListPlace::Recent, ..ListQuery::default() }).unwrap();
        prop_assert_eq!(ids_of(&recent.rows), ids_of(&plain.rows));
        prop_assert_eq!(recent.shape.sort, SortKey::Updated);
        prop_assert_eq!(recent.shape.date, DateField::Updated);
        prop_assert!(recent.shape.by_date && !recent.shape.group_pinned);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// A change to several notes does what it says to each of them, in one
    /// write, and says how many it wrote: a note already as asked isn't
    /// written. Pinning twice undoes itself for notes that were alike.
    #[test]
    fn prop_a_change_does_what_it_says(seeds in seeds(), pick in proptest::collection::vec(any::<bool>(), 10), kind in 0usize..3) {
        let (phone, ids, books) = seeded(&seeds);
        let library = &phone.library;
        let chosen: Vec<String> = ids.iter().zip(pick).filter(|(_, p)| *p).map(|(id, _)| id.clone()).collect();
        let before: Vec<_> = chosen.iter().map(|id| library.note(id).unwrap()).collect();
        let change = match kind {
            0 => NoteChange::TogglePin,
            1 => NoteChange::Trash,
            _ => NoteChange::Move { notebook_id: books[1].clone() },
        };
        let written = library.change(chosen.clone(), change.clone()).unwrap().count;
        let after: Vec<_> = chosen.iter().map(|id| library.note(id).unwrap()).collect();
        let changed = before.iter().zip(&after).filter(|(b, a)| b.rev != a.rev).count();
        prop_assert_eq!(written as usize, changed);
        match change {
            NoteChange::TogglePin => {
                let all_pinned = before.iter().all(|n| n.pinned);
                prop_assert!(after.iter().all(|n| n.pinned == !all_pinned));
            }
            NoteChange::Trash => prop_assert!(after.iter().all(|n| n.notebook_id == "trash")),
            NoteChange::Move { ref notebook_id } => {
                prop_assert!(after.iter().all(|n| &n.notebook_id == notebook_id));
            }
            // The actions have their own tests (`tests/actions.rs`).
            _ => unreachable!("only three kinds are drawn"),
        }
        // Asked again, a move or a trash writes nothing.
        if kind != 0 {
            prop_assert_eq!(library.change(chosen, change).unwrap().count, 0);
        }
    }
}

// no-prop-test: the refusals, one each; the change's content is the property
// above.
#[test]
fn a_change_that_cannot_be_made_writes_nothing() {
    let (mut inbox, mut sealed_book, mut plain_book) =
        (String::new(), String::new(), String::new());
    let mut note_ids = Vec::new();
    let phone = phone_with(&server(), |store| {
        plain_book = book(store, "Plain");
        inbox = book(store, "Other");
        sealed_book = store
            .put(
                kanso_core::DocType::Book,
                json!({"name": "Sealed", LOCK_FIELD: {}}),
                kanso_store::Put::Upsert,
                kanso_store::Origin::Ui,
            )
            .unwrap()
            .id;
        note_ids.push(raw_note(store, json!({"title": "A", "bookId": plain_book})));
        note_ids.push(raw_note(
            store,
            json!({"title": "B", "bookId": sealed_book}),
        ));
    });
    let library = &phone.library;
    let revs = |ids: &[String]| -> Vec<String> {
        ids.iter().map(|id| library.note(id).unwrap().rev).collect()
    };
    let before = revs(&note_ids);

    // A note that is not one: nothing is written for the others either.
    let err = library
        .change(
            vec![note_ids[0].clone(), "note:nowhere".into()],
            NoteChange::TogglePin,
        )
        .unwrap_err();
    assert_eq!(err, KansoError::NotFound);
    // A notebook that is not one.
    let err = library
        .change(
            vec![note_ids[0].clone()],
            NoteChange::Move {
                notebook_id: "book:nowhere".into(),
            },
        )
        .unwrap_err();
    assert_eq!(err, KansoError::NotFound);
    // An unlocked note into a locked notebook goes one at a time.
    let err = library
        .change(
            vec![note_ids[0].clone()],
            NoteChange::Move {
                notebook_id: sealed_book.clone(),
            },
        )
        .unwrap_err();
    assert_eq!(err, KansoError::Locked);
    // A note leaves a locked notebook one at a time too.
    let err = library
        .change(
            vec![note_ids[1].clone()],
            NoteChange::Move {
                notebook_id: inbox.clone(),
            },
        )
        .unwrap_err();
    assert_eq!(err, KansoError::Locked);
    assert_eq!(revs(&note_ids), before, "nothing was written");

    // What is allowed: pinning and trashing, even in the locked notebook.
    assert_eq!(
        library
            .change(note_ids.clone(), NoteChange::TogglePin)
            .unwrap()
            .count,
        2
    );
    assert_eq!(
        library
            .change(note_ids.clone(), NoteChange::Trash)
            .unwrap()
            .count,
        2
    );
    assert_eq!(library.change(vec![], NoteChange::Trash).unwrap().count, 0);
}

// no-prop-test: the order of a small fixed tree and its roles.
#[test]
fn notebooks_come_in_tree_order_with_their_roles_and_depth() {
    let mut templates = String::new();
    let phone = phone_with(&server(), |store| {
        let put = |name: &str, parent: Option<&String>| {
            store
                .put(
                    kanso_core::DocType::Book,
                    json!({"name": name, "parentBookId": parent}),
                    kanso_store::Put::Upsert,
                    kanso_store::Origin::Ui,
                )
                .unwrap()
                .id
        };
        let work = put("Work", None);
        let acme = put("Acme", Some(&work));
        put("Plans", Some(&acme));
        put("Garden", None);
        templates = store
            .role_book(kanso_core::roles::Role::Templates, kanso_store::Origin::Ui)
            .unwrap();
    });
    let rows = phone.library.notebooks().unwrap();
    let order: Vec<_> = rows.iter().map(|r| (r.name.as_str(), r.depth)).collect();
    assert_eq!(
        order,
        [
            ("Garden", 0),
            ("Templates", 0),
            ("Work", 0),
            ("Acme", 1),
            ("Plans", 2)
        ]
    );
    // A notebook with a job, however it is named.
    let row = rows.iter().find(|r| r.id == templates).unwrap();
    assert_eq!(row.role, Some(NotebookRole::Templates));
    assert!(
        rows.iter()
            .filter(|r| r.id != templates)
            .all(|r| r.role.is_none())
    );
}

// no-prop-test: a notebook made inside another, and the refusals.
#[test]
fn a_notebook_is_made_inside_another() {
    let phone = lone_phone();
    let library = &phone.library;
    let work = library.create_notebook("Work".into(), None).unwrap();
    let acme = library
        .create_notebook("  Acme ".into(), Some(work.id.clone()))
        .unwrap();
    let deep = library
        .create_notebook("Plans".into(), Some(acme.id.clone()))
        .unwrap();
    assert_eq!(acme.name, "Acme", "the name is trimmed");
    assert_eq!(acme.parent_id.as_deref(), Some(work.id.as_str()));
    assert_eq!((work.depth, acme.depth, deep.depth), (0, 1, 2));
    assert_eq!(deep.count, 0);
    let names: Vec<_> = library
        .notebooks()
        .unwrap()
        .into_iter()
        .map(|r| r.name)
        .collect();
    assert_eq!(names, ["Work", "Acme", "Plans"]);

    assert!(matches!(
        library.create_notebook("   ".into(), None),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(
        library.create_notebook("X".into(), Some("book:nowhere".into())),
        Err(KansoError::NotFound)
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// A note's `#tags` are its tags: written, they appear; taken out of the
    /// text, they go; and a tag added by hand stays. The same on the
    /// first write as on every save.
    #[test]
    fn prop_the_text_gives_the_note_its_tags(
        words in proptest::collection::btree_set("[a-z]{3,8}", 0..4),
        kept in proptest::collection::btree_set("[a-z]{3,8}", 0..3),
    ) {
        let phone = lone_phone();
        let library = &phone.library;
        let text = |words: &BTreeSet<String>| {
            let tags: Vec<String> = words.iter().map(|w| format!("#{w}")).collect();
            format!("Notes {}\n", tags.join(" "))
        };
        let saved = library
            .create(NewNote { title: "T".into(), body: text(&words), ..NewNote::default() })
            .unwrap();
        let tag_names = |id: &str| -> BTreeSet<String> {
            let all = library.tags().unwrap();
            library.note(id).unwrap().tags.iter()
                .map(|t| all.iter().find(|a| &a.id == t).unwrap().name.clone())
                .collect()
        };
        prop_assert_eq!(tag_names(&saved.id), words.clone());

        // Taking the words out of the text takes their tags off; new words
        // add theirs.
        let rev = library.note(&saved.id).unwrap().rev;
        let next = library.save(&saved.id, &rev, text(&kept)).unwrap();
        prop_assert_eq!(tag_names(&saved.id), kept.clone());
        prop_assert_ne!(next.rev, rev);
    }
}

// no-prop-test: one library, scopes read side by side.
#[test]
fn a_search_looks_where_its_scope_says() {
    let (mut garden, mut named, mut texted, mut planted) =
        (String::new(), String::new(), String::new(), String::new());
    let phone = phone_with(&server(), |store| {
        let archive = book(store, "Archive");
        garden = book(store, "Garden");
        let veg = tag(store, "veg");
        let errand = tag(store, "errand");
        named = raw_note(
            store,
            json!({"title": "Tomato plan", "body": "water the beans daily", "bookId": archive, "tags": [veg]}),
        );
        texted = raw_note(
            store,
            json!({"title": "Shopping", "body": "buy tomato seeds", "bookId": archive, "tags": [errand]}),
        );
        planted = raw_note(
            store,
            json!({"title": "Beds", "body": "tomato rows", "bookId": garden}),
        );
    });
    let library = &phone.library;
    let (named, texted, planted) = (&named, &texted, &planted);
    let found = |text: &str, scope: SearchScope| -> BTreeSet<String> {
        library
            .search(text, scope, 0)
            .unwrap()
            .into_iter()
            .map(|hit| hit.row.id)
            .collect()
    };
    let set = |ids: &[&String]| ids.iter().map(|id| (*id).clone()).collect::<BTreeSet<_>>();

    assert_eq!(
        found("tomato", SearchScope::All),
        set(&[named, texted, planted])
    );
    assert_eq!(found("tomato", SearchScope::Titles), set(&[named]));
    assert_eq!(
        found("tomato", SearchScope::Notebook { id: garden.clone() }),
        set(&[planted])
    );
    assert_eq!(found("veg", SearchScope::Tags), set(&[named]));
    assert!(found("tomato", SearchScope::Tags).is_empty());
    // The matches follow the scope: titles only light up in the title.
    let hit = &library.search("tomato", SearchScope::Titles, 0).unwrap()[0];
    assert!(!hit.title_ranges.is_empty());
    assert!(hit.snippet_ranges.is_empty());
    // A limit counts what is kept.
    assert_eq!(
        library.search("tomato", SearchScope::All, 2).unwrap().len(),
        2
    );
}
