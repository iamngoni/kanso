//! The reads behind a notebook's homepage (issue #201): the subtree stops at
//! locked notebooks, nothing counts or names a locked note, the open tasks
//! of a subtree are those of its notebooks together, the cache of the AI
//! sections lives on the notebook without counting as an edit, and locking
//! the notebook removes it. CI pins the seed with `PROPTEST_RNG_SEED`.

use std::collections::{BTreeSet, HashSet};

use kanso_core::DocType;
use kanso_core::page_cache::{CachedItem, CachedSection, Layout, PAGE_FIELD, PageCache};
use kanso_store::{Origin, Put, Store, TaskQuery};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Value, json};

const CANARY: &str = "okapicanary9913";

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

fn book(store: &Store, id: &str, parent: Option<&str>) -> String {
    store
        .put(
            DocType::Book,
            json!({"_id": id, "name": id, "parentBookId": parent}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn note(store: &Store, id: &str, title: &str, body: &str, book: &str, at: i64) {
    let current = store.get(id).unwrap().map(|d| d.rev);
    let mut doc = json!({"_id": id, "title": title, "body": body, "bookId": book,
        "createdAt": at, "updatedAt": at});
    if let Some(rev) = current {
        doc["_rev"] = json!(rev);
    }
    store
        .put(DocType::Note, doc, Put::Upsert, Origin::Ui)
        .unwrap();
}

fn lock_value() -> Value {
    json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"})
}

/// The notebook with its lock set, as sync brings it.
fn lock_book_by_sync(store: &Store, id: &str) {
    let mut book = store.get_api(id, None, false).unwrap();
    book.as_object_mut().unwrap().remove("count");
    book["kansoLock"] = lock_value();
    store
        .put(DocType::Book, book, Put::Upsert, Origin::Sync)
        .unwrap();
}

fn seal_note(store: &Store, id: &str) {
    let mut note = store.get_api(id, None, false).unwrap();
    note["body"] = "".into();
    note["kansoLock"] = lock_value();
    store.lock_note(note).unwrap();
}

/// A forest: notebook `i` has parent `parents[i]` (an earlier one) or none.
fn forest() -> impl Strategy<Value = (Vec<Option<usize>>, Vec<bool>)> {
    (1usize..7).prop_flat_map(|n| {
        let parents: Vec<BoxedStrategy<Option<usize>>> = (0..n)
            .map(|i| {
                if i == 0 {
                    Just(None).boxed()
                } else {
                    proptest::option::of(0..i).boxed()
                }
            })
            .collect();
        (
            parents,
            proptest::collection::vec(proptest::bool::weighted(0.3), n),
        )
    })
}

fn build_forest(store: &Store, parents: &[Option<usize>], locked: &[bool]) -> Vec<String> {
    let ids: Vec<String> = (0..parents.len()).map(|i| format!("book:b{i}")).collect();
    for (i, parent) in parents.iter().enumerate() {
        book(store, &ids[i], parent.map(|p| ids[p].as_str()));
    }
    for (i, lock) in locked.iter().enumerate() {
        if *lock {
            lock_book_by_sync(store, &ids[i]);
        }
    }
    ids
}

/// The notebooks under `root`, by the test's own walk: stops at a locked
/// notebook, which hides everything inside it.
fn expected_subtree(
    parents: &[Option<usize>],
    locked: &[bool],
    root: usize,
    include_locked: bool,
) -> BTreeSet<usize> {
    let mut out = BTreeSet::new();
    if !include_locked && locked[root] {
        return out;
    }
    let mut stack = vec![root];
    while let Some(at) = stack.pop() {
        out.insert(at);
        for (i, parent) in parents.iter().enumerate() {
            if *parent == Some(at) && (include_locked || !locked[i]) {
                stack.push(i);
            }
        }
    }
    out
}

proptest! {
    #![proptest_config(config(48))]

    #[test]
    fn prop_subtree_stops_at_locked_notebooks(
        (parents, locked) in forest(),
        root in 0usize..7,
    ) {
        let (_dir, store) = temp_store();
        let ids = build_forest(&store, &parents, &locked);
        let root = root % ids.len();
        for include in [false, true] {
            let got: BTreeSet<String> = store
                .book_subtree(&ids[root], include)
                .unwrap()
                .into_iter()
                .collect();
            let want: BTreeSet<String> = expected_subtree(&parents, &locked, root, include)
                .into_iter()
                .map(|i| ids[i].clone())
                .collect();
            // A notebook inside a locked one is locked too, whatever its
            // own flag says.
            let want: BTreeSet<String> = want
                .into_iter()
                .filter(|id| {
                    include || {
                        let mut at = ids.iter().position(|b| b == id).unwrap();
                        loop {
                            if locked[at] { break false; }
                            match parents[at] { Some(p) => at = p, None => break true }
                        }
                    }
                })
                .collect();
            prop_assert_eq!(got, want);
        }
        let walked = store.book_subtree(&ids[root], false).unwrap();
        if let Some(first) = walked.first() {
            prop_assert_eq!(first, &ids[root]);
        }
    }

    #[test]
    fn prop_homepage_never_counts_or_names_a_locked_note(
        (parents, locked) in forest(),
        placed in proptest::collection::vec((0usize..7, proptest::bool::weighted(0.3)), 0..10),
    ) {
        let (_dir, store) = temp_store();
        let ids = build_forest(&store, &parents, &[false; 7][..parents.len()]);
        let mut visible: Vec<(String, usize)> = Vec::new();
        let mut hidden: HashSet<String> = HashSet::new();
        for (n, (at, seal)) in placed.iter().enumerate() {
            let at = at % ids.len();
            let id = format!("note:n{n}");
            let body = format!("{CANARY} word\n- [ ] task {n} @2026-11-0{}\nSee [[Note {}]]\n", (n % 9) + 1, (n + 1) % 10);
            note(&store, &id, &format!("Note {n}"), &body, &ids[at], 1000 + n as i64);
            if *seal {
                seal_note(&store, &id);
                hidden.insert(id);
            } else {
                visible.push((id, at));
            }
        }
        for (i, lock) in locked.iter().enumerate() {
            if *lock {
                lock_book_by_sync(&store, &ids[i]);
            }
        }
        // What the walk reaches, and the notes that are not sealed in it.
        for root in 0..ids.len() {
            let reach: BTreeSet<String> = store
                .book_subtree(&ids[root], false)
                .unwrap()
                .into_iter()
                .collect();
            let want: BTreeSet<&str> = visible
                .iter()
                .filter(|(_, at)| reach.contains(&ids[*at]))
                .map(|(id, _)| id.as_str())
                .collect();
            let stats = store.home_stats(&ids[root]).unwrap();
            prop_assert_eq!(stats.notes as usize, want.len());
            prop_assert_eq!(
                stats.notebooks_inside as usize,
                reach.len().saturating_sub(1)
            );
            let named = |rows: Vec<kanso_store::HomeNoteRow>| -> BTreeSet<String> {
                rows.into_iter().map(|r| r.id).collect()
            };
            for seen in [
                named(store.home_recent(&ids[root], 100).unwrap()),
                named(store.home_all_notes(&ids[root]).unwrap()),
                named(store.home_edited_since(&ids[root], 0, 100).unwrap()),
            ] {
                prop_assert!(seen.iter().all(|id| want.contains(id.as_str())));
            }
            let (notes, links) = store.home_graph_rows(&ids[root]).unwrap();
            prop_assert!(notes.iter().all(|n| want.contains(n.id.as_str())));
            prop_assert!(links.iter().all(|l| want.contains(l.from.as_str())));
            prop_assert!(store
                .home_linked(&ids[root], 100)
                .unwrap()
                .iter()
                .all(|l| want.contains(l.note_id.as_str())));
            for child in store.home_children(&ids[root]).unwrap() {
                if child.locked {
                    prop_assert_eq!((child.notes, child.open_tasks), (0, 0));
                    prop_assert!(child.last_edited.is_none());
                }
            }
            let tasks = store
                .tasks(&TaskQuery {
                    book_ids: Some(reach.iter().cloned().collect()),
                    ..TaskQuery::default()
                })
                .unwrap();
            prop_assert!(tasks.iter().all(|t| !hidden.contains(&t.note_id)));
            prop_assert_eq!(
                store.home_open_task_count(&ids[root]).unwrap() as usize,
                tasks.len()
            );
        }
    }

    #[test]
    fn prop_open_tasks_equals_tasks_of_each_subtree_notebook_combined(
        (parents, _locked) in forest(),
        placed in proptest::collection::vec((0usize..7, 0usize..4, proptest::bool::ANY), 0..12),
        root in 0usize..7,
    ) {
        let (_dir, store) = temp_store();
        let ids = build_forest(&store, &parents, &vec![false; parents.len()]);
        for (n, (at, tasks, done)) in placed.iter().enumerate() {
            let body: String = (0..*tasks)
                .map(|t| format!("- [{}] job {n}-{t}\n", if *done { "x" } else { " " }))
                .collect();
            note(&store, &format!("note:n{n}"), &format!("N{n}"), &body, &ids[at % ids.len()], 10 + n as i64);
        }
        let root = root % ids.len();
        let subtree = store.book_subtree(&ids[root], false).unwrap();
        for include_done in [false, true] {
            let together: BTreeSet<(String, u32)> = store
                .tasks(&TaskQuery {
                    book_ids: Some(subtree.clone()),
                    include_done,
                    ..TaskQuery::default()
                })
                .unwrap()
                .into_iter()
                .map(|t| (t.note_id, t.line))
                .collect();
            let mut each = BTreeSet::new();
            for b in &subtree {
                for t in store
                    .tasks(&TaskQuery {
                        book_id: Some(b.clone()),
                        include_done,
                        ..TaskQuery::default()
                    })
                    .unwrap()
                {
                    each.insert((t.note_id, t.line));
                }
            }
            prop_assert_eq!(together, each);
        }
    }
}

fn cache(text: &str, at: i64) -> PageCache {
    let mut cache = PageCache::default();
    cache.sections.insert(
        "about".into(),
        CachedSection {
            items: vec![CachedItem {
                text: text.into(),
                ..CachedItem::default()
            }],
            written_at: at,
            fingerprint: format!("f-{text}"),
            ..CachedSection::default()
        },
    );
    cache
}

#[test]
fn a_notebooks_cache_is_not_an_edit_of_the_notebook() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a", None);
    let before = store.get(&b).unwrap().unwrap();
    let updated = before.fields["updatedAt"].clone();
    assert!(store.set_page_cache(&b, &cache("About it", 5)).unwrap());
    let after = store.get(&b).unwrap().unwrap();
    assert_eq!(after.fields["updatedAt"], updated);
    assert_eq!(store.page_cache(&b).unwrap(), {
        let mut c = cache("About it", 5);
        c.v = 1;
        c
    });
    // The notebook as the API serves it never carries the cache.
    assert!(
        store
            .get_api(&b, None, false)
            .unwrap()
            .get(PAGE_FIELD)
            .is_none()
    );
    // Writing the same thing again does nothing.
    assert!(!store.set_page_cache(&b, &cache("About it", 5)).unwrap());
}

#[test]
fn locking_a_notebook_removes_its_page_cache_and_a_locked_one_takes_none() {
    let (_dir, store) = temp_store();
    let outer = book(&store, "book:outer", None);
    let inner = book(&store, "book:inner", Some(&outer));
    note(&store, "note:a", "Plans", &format!("{CANARY}\n"), &outer, 5);
    store.set_page_cache(&outer, &cache(CANARY, 5)).unwrap();
    store.set_page_cache(&inner, &cache(CANARY, 5)).unwrap();
    assert!(
        store
            .get(&outer)
            .unwrap()
            .unwrap()
            .fields
            .contains_key(PAGE_FIELD)
    );

    let mut sealed_book = store.get_api(&outer, None, false).unwrap();
    sealed_book.as_object_mut().unwrap().remove("count");
    sealed_book["kansoLock"] = lock_value();
    let mut sealed_note = store.get_api("note:a", None, false).unwrap();
    sealed_note["body"] = "".into();
    sealed_note["kansoLock"] = lock_value();
    store.lock_book(sealed_book, vec![sealed_note]).unwrap();

    assert!(
        !store
            .get(&outer)
            .unwrap()
            .unwrap()
            .fields
            .contains_key(PAGE_FIELD)
    );
    // The notebook inside it is locked now: it can't take a cache.
    assert!(!store.set_page_cache(&inner, &cache(CANARY, 6)).unwrap());
    assert!(!store.set_page_cache(&outer, &cache(CANARY, 6)).unwrap());
    assert!(store.page_cache(&outer).unwrap().is_empty());
    assert!(store.home_stats(&outer).unwrap().notes == 0);
    assert!(store.home_recent(&outer, 10).unwrap().is_empty());
    assert!(store.home_children(&outer).unwrap().is_empty());
}

#[test]
fn the_layout_and_the_cache_of_two_devices_merge_by_time() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a", None);
    let mut mine = cache("mine", 1);
    mine.layout = Some(Layout {
        order: vec!["about".into(), "map".into()],
        at: 5,
    });
    store.set_page_cache(&b, &mine).unwrap();
    let got = store.page_cache(&b).unwrap();
    assert_eq!(got.layout.as_ref().unwrap().order, ["about", "map"]);
}

#[test]
fn a_notebook_with_two_marked_notes_shows_the_newest() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a", None);
    note(
        &store,
        "note:one",
        "One",
        "---\nkanso: home\n---\nfirst\n",
        &b,
        10,
    );
    note(
        &store,
        "note:two",
        "Two",
        "---\nkanso: home\n---\nsecond\n",
        &b,
        20,
    );
    note(
        &store,
        "note:plain",
        "Plain",
        "kanso: home but not front matter\n",
        &b,
        30,
    );
    let marked = store.home_notes_marked(&b).unwrap();
    assert_eq!(
        marked.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
        ["note:two", "note:one"]
    );
    assert_eq!(store.home_note(&b).unwrap().unwrap().id, "note:two");
    // A sealed Home note is not offered.
    seal_note(&store, "note:two");
    assert_eq!(store.home_note(&b).unwrap().unwrap().id, "note:one");
}

#[test]
fn key_note_inputs_count_each_linking_note_once_and_skip_self_links() {
    let (_dir, store) = temp_store();
    let a = book(&store, "book:a", None);
    let inner = book(&store, "book:in", Some(&a));
    let out = book(&store, "book:out", None);
    note(&store, "note:plan", "Plan", "Plan\n[[Plan]]\n", &a, 1);
    note(
        &store,
        "note:x",
        "X",
        "[[Plan]] and [[Plan]] again\n",
        &inner,
        2,
    );
    note(
        &store,
        "note:y",
        "Y",
        &format!("[Plan]({}plan)\n", kanso_core::links::NOTE_LINK_PREFIX),
        &a,
        3,
    );
    note(&store, "note:z", "Z", "[[Plan]]\n", &out, 4);
    let linked = store.home_linked(&a, 10).unwrap();
    assert_eq!(linked.len(), 1);
    assert_eq!(linked[0].note_id, "note:plan");
    assert_eq!(
        linked[0].linked_from, 2,
        "x and y; not itself, not z outside"
    );
}

/// The homepage lists a subtree's open tasks in the order `Store::tasks`
/// does (dated first, soonest first, then newest note, each note's in
/// order), reading the words only of the ones it keeps; it counts children
/// from the same read.
#[test]
fn open_tasks_of_a_scan_match_the_task_list_and_children_count_them() {
    let (_dir, store) = temp_store();
    book(&store, "book:top", None);
    book(&store, "book:kid", Some("book:top"));
    book(&store, "book:grand", Some("book:kid"));
    note(
        &store,
        "note:a",
        "A",
        "- [ ] undated a\n- [ ] due late @2026-12-01\n- [x] done",
        "book:top",
        10,
    );
    note(
        &store,
        "note:b",
        "B",
        "- [ ] due soon @2026-11-05\n- [ ] undated b",
        "book:kid",
        30,
    );
    note(
        &store,
        "note:c",
        "C",
        "- [ ] undated c1\n- [ ] undated c2\n- [ ] due soon too @2026-11-05",
        "book:grand",
        20,
    );
    let subtree = store.book_subtree("book:top", false).unwrap();
    let expected: Vec<(String, u32)> = store
        .tasks(&TaskQuery {
            book_ids: Some(subtree),
            ..TaskQuery::default()
        })
        .unwrap()
        .into_iter()
        .map(|t| (t.note_id, t.line))
        .collect();
    let scan = store.home_scan("book:top").unwrap();
    for limit in 0..=expected.len() + 1 {
        let got: Vec<(String, u32)> = store
            .home_open_tasks_in(&scan, limit)
            .unwrap()
            .into_iter()
            .map(|t| (t.note_id, t.line))
            .collect();
        assert_eq!(got, expected[..limit.min(expected.len())], "limit {limit}");
    }
    let kids = store.home_children("book:top").unwrap();
    assert_eq!(kids.len(), 1);
    assert_eq!((kids[0].notes, kids[0].open_tasks), (2, 5));
    assert_eq!(kids[0].last_edited, Some(30));
    assert_eq!(store.home_open_task_count("book:top").unwrap(), 7);
    assert_eq!(store.home_task_count("book:top").unwrap(), 8);
}
