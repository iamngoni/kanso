//! The tables derived from note bodies: links, headings and tasks. CI pins
//! the seed with `PROPTEST_RNG_SEED`.

use std::collections::BTreeSet;

use kanso_core::DocType;
use kanso_core::links::{LinkTarget, normalize_title, note_links};
use kanso_core::outline::headings;
use kanso_core::tasks::task_items;
use kanso_store::{DERIVED_VERSION, DERIVED_VERSION_KEY, Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

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

fn book(store: &Store) -> String {
    store
        .put(
            DocType::Book,
            json!({"_id": "book:b", "name": "B"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn note(store: &Store, id: &str, title: &str, body: &str, book: &str) {
    let current = store.get(id).unwrap().map(|d| d.rev);
    let mut doc = json!({"_id": id, "title": title, "body": body, "bookId": book});
    if let Some(rev) = current {
        doc["_rev"] = json!(rev);
    }
    store
        .put(DocType::Note, doc, Put::Upsert, Origin::Ui)
        .unwrap();
}

#[test]
fn backlinks_come_from_id_links_and_wiki_links() {
    let (_dir, store) = temp_store();
    let b = book(&store);
    note(&store, "note:target", "Project Plan", "# Plan\n", &b);
    note(
        &store,
        "note:by-id",
        "By ID",
        "intro\nsee [plan](kanso://note/target) now",
        &b,
    );
    note(
        &store,
        "note:by-title",
        "By title",
        "- [ ] read [[project  plan#Plan|the plan]]",
        &b,
    );
    note(
        &store,
        "note:mention",
        "Mention",
        "The project plan is late.",
        &b,
    );
    note(
        &store,
        "note:trashed",
        "Trashed",
        "[[Project Plan]]",
        "trash",
    );
    let found = store.backlinks("note:target").unwrap();
    let ids: BTreeSet<&str> = found.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(ids, BTreeSet::from(["note:by-id", "note:by-title"]));
    let by_id = found.iter().find(|b| b.id == "note:by-id").unwrap();
    assert_eq!((by_id.line, by_id.context.as_str()), (1, "see plan now"));
    let by_title = found.iter().find(|b| b.id == "note:by-title").unwrap();
    assert_eq!(by_title.context, "read the plan");

    let mentions = store.unlinked_mentions("note:target").unwrap();
    assert_eq!(mentions.len(), 1);
    assert_eq!(mentions[0].id, "note:mention");
    assert_eq!(mentions[0].context, "The project plan is late.");

    assert_eq!(
        store.resolve_title(" PROJECT plan ").unwrap(),
        Some(("note:target".into(), "Project Plan".into()))
    );
    assert_eq!(store.resolve_title("Nope").unwrap(), None);
    assert_eq!(store.outline("note:target").unwrap()[0].text, "Plan");
    let hits = store.search_headings("pla", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].note_title, "Project Plan");
}

#[test]
fn a_title_shared_by_two_notes_names_the_first_created() {
    let (_dir, store) = temp_store();
    let b = book(&store);
    note(&store, "note:first", "Same", "", &b);
    std::thread::sleep(std::time::Duration::from_millis(5));
    note(&store, "note:second", "same", "", &b);
    note(&store, "note:linker", "L", "[[SAME]]", &b);
    assert_eq!(store.backlinks("note:first").unwrap().len(), 1);
    assert!(store.backlinks("note:second").unwrap().is_empty());
}

#[test]
fn retitling_rewrites_links_in_one_go() {
    let (_dir, store) = temp_store();
    let b = book(&store);
    note(&store, "note:t", "Old", "", &b);
    note(&store, "note:a", "A", "[[Old]] and [[old#Sec|x]]", &b);
    note(&store, "note:c", "C", "[[Other]]", &b);
    assert_eq!(store.wiki_link_count("Old", "note:t").unwrap(), (2, 1));
    let changed = store
        .retitle_links("Old", "New", "note:t", Origin::Ui)
        .unwrap();
    assert_eq!(changed, 2);
    assert_eq!(
        store.note("note:a").unwrap().unwrap().body,
        "[[New]] and [[New#Sec|x]]"
    );
    assert_eq!(store.note("note:c").unwrap().unwrap().body, "[[Other]]");
    assert_eq!(store.wiki_link_count("Old", "note:t").unwrap(), (0, 0));
}

#[test]
fn derived_rows_are_rebuilt_when_their_version_changes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kanso.db");
    {
        let store = Store::open(&path).unwrap();
        let b = book(&store);
        note(&store, "note:x", "X", "# H\n- [ ] t @2026-10-25\n[[Y]]", &b);
        assert_eq!(store.derived_rows("note:x").unwrap(), (1, 1, 1));
        assert_eq!(
            store.meta(DERIVED_VERSION_KEY).unwrap().as_deref(),
            Some(DERIVED_VERSION)
        );
        // An older build's tables: stale rows and an older version.
        store.set_meta(DERIVED_VERSION_KEY, "0").unwrap();
    }
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute("DELETE FROM note_headings", []).unwrap();
        conn.execute(
            "INSERT INTO note_links (note_id, ord, target_id, line) VALUES ('note:x', 9, 'note:zz', 0)",
            [],
        )
        .unwrap();
    }
    let store = Store::open(&path).unwrap();
    assert_eq!(store.derived_rows("note:x").unwrap(), (1, 1, 1));
    assert_eq!(
        store.meta(DERIVED_VERSION_KEY).unwrap().as_deref(),
        Some(DERIVED_VERSION)
    );
}

#[test]
fn replacing_the_library_empties_the_derived_tables() {
    let (_dir, store) = temp_store();
    let b = book(&store);
    note(&store, "note:x", "X", "# H\n- [ ] t\n[[Y]]", &b);
    store.replace_library(&[]).unwrap();
    assert_eq!(store.derived_rows("note:x").unwrap(), (0, 0, 0));
}

const TITLES: [&str; 4] = ["Alpha", "beta", "Gamma Ray", "alpha"];

#[derive(Clone, Debug)]
enum Op {
    Put {
        note: usize,
        title: usize,
        body: Vec<Piece>,
        trashed: bool,
    },
    Delete(usize),
}

#[derive(Clone, Debug)]
enum Piece {
    Text(String),
    IdLink(usize),
    WikiLink(usize),
    Heading(u8, String),
    Task(bool),
    Fence,
}

fn piece() -> impl Strategy<Value = Piece> {
    prop_oneof![
        "[a-z ]{0,8}".prop_map(Piece::Text),
        (0usize..4).prop_map(Piece::IdLink),
        (0usize..4).prop_map(Piece::WikiLink),
        (1u8..=3, "[a-z]{1,6}").prop_map(|(l, t)| Piece::Heading(l, t)),
        any::<bool>().prop_map(Piece::Task),
        Just(Piece::Fence),
    ]
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        4 => (0usize..4, 0usize..4, prop::collection::vec(piece(), 0..6), prop::bool::weighted(0.2))
            .prop_map(|(note, title, body, trashed)| Op::Put { note, title, body, trashed }),
        1 => (0usize..4).prop_map(Op::Delete),
    ]
}

fn render(body: &[Piece]) -> String {
    body.iter()
        .map(|p| match p {
            Piece::Text(t) => format!("{t}\n"),
            Piece::IdLink(n) => format!("[x](kanso://note/n{n})\n"),
            Piece::WikiLink(t) => format!("[[{}]]\n", TITLES[*t]),
            Piece::Heading(l, t) => format!("{} {t}\n", "#".repeat(*l as usize)),
            Piece::Task(done) => format!("- [{}] job\n", if *done { "x" } else { " " }),
            Piece::Fence => "```\n".to_owned(),
        })
        .collect()
}

proptest! {
    #![proptest_config(config(48))]

    /// Model-based: after any run of writes and deletions, each note's
    /// derived rows are what its body gives, deleted notes have none, and
    /// backlinks are exactly the other live notes outside the trash whose
    /// links point at the note: by ID, or by a title that names it.
    #[test]
    fn prop_derived_tables_follow_the_bodies(ops in prop::collection::vec(op(), 1..14)) {
        let (_dir, store) = temp_store();
        let b = book(&store);
        // id -> (title, body, trashed); None once deleted.
        let mut model: Vec<Option<(String, String, bool)>> = vec![None; 4];
        let mut created: Vec<Option<u32>> = vec![None; 4];
        let mut clock = 0;
        for op in &ops {
            match op {
                Op::Put { note: n, title, body, trashed } => {
                    let id = format!("note:n{n}");
                    let text = render(body);
                    let book = if *trashed { "trash".to_owned() } else { b.clone() };
                    note(&store, &id, TITLES[*title], &text, &book);
                    if model[*n].is_none() {
                        clock += 1;
                        created[*n] = Some(clock);
                        // Creation times must differ for the first-created rule.
                        std::thread::sleep(std::time::Duration::from_millis(2));
                    }
                    model[*n] = Some((TITLES[*title].to_owned(), text, *trashed));
                }
                Op::Delete(n) => {
                    let id = format!("note:n{n}");
                    if model[*n].is_some() {
                        store.delete(&id, Origin::Ui).unwrap();
                        model[*n] = None;
                        created[*n] = None;
                    }
                }
            }
        }
        // Which note a title names: the first created live one outside the trash.
        let resolve = |title: &str| -> Option<usize> {
            (0..4)
                .filter(|&n| matches!(&model[n], Some((t, _, false)) if normalize_title(t) == normalize_title(title)))
                .min_by_key(|&n| created[n])
        };
        for n in 0..4 {
            let id = format!("note:n{n}");
            let rows = store.derived_rows(&id).unwrap();
            match &model[n] {
                None => prop_assert_eq!(rows, (0, 0, 0)),
                Some((_, body, _)) => {
                    prop_assert_eq!(rows, (note_links(body).len(), headings(body).len(), task_items(body).len()));
                    prop_assert_eq!(store.outline(&id).unwrap(), headings(body));
                }
            }
            let mut expected: Vec<String> = (0..4)
                .filter(|&m| m != n)
                .filter(|&m| match &model[m] {
                    Some((_, body, false)) => note_links(body).iter().any(|link| match &link.target {
                        LinkTarget::Note(target) => *target == id,
                        LinkTarget::Title(title) => model[n].is_some() && resolve(title) == Some(n),
                    }),
                    _ => false,
                })
                .map(|m| format!("note:n{m}"))
                .collect();
            expected.sort();
            let mut found: Vec<String> = if model[n].is_some() {
                store.backlinks(&id).unwrap().into_iter().map(|b| b.id).collect()
            } else {
                Vec::new()
            };
            found.sort();
            // ID links count even to a note that doesn't exist; the store
            // only answers for live ones.
            if model[n].is_some() {
                prop_assert_eq!(found, expected);
            }
        }
    }
}
