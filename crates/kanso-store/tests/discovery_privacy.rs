//! What the discovery features never touch: a note that is locked (sealed,
//! or in a locked notebook) or tagged `#ai-exclude` is never embedded, so it
//! is never a related note, a source of suggested tags, a twin or a hit of a
//! search by meaning; the images only such notes show are never read; and
//! locked notes are left out of the graph while the lock is closed.

use std::collections::HashSet;
use std::sync::Arc;

use base64::Engine as _;
use kanso_core::links::normalize_title;
use kanso_core::{DocType, TRASH_BOOK_ID};
use kanso_store::{Embedder, MeaningSearch, Origin, Put, Store};
use proptest::prelude::*;
use serde_json::{Value, json};

const MODEL: &str = "letters";

/// Counts letters: texts with the same letters point the same way.
struct Letters;

impl Embedder for Letters {
    fn model(&self) -> &str {
        MODEL
    }
    fn dim(&self) -> usize {
        26
    }
    fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
        texts
            .iter()
            .map(|t| {
                let mut v = vec![0f32; 26];
                for c in t.to_ascii_lowercase().bytes() {
                    if c.is_ascii_lowercase() {
                        v[(c - b'a') as usize] += 1.;
                    }
                }
                v
            })
            .collect()
    }
}

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    store.set_embedding_model(Some(MODEL)).unwrap();
    store.lend_embedder(Some(Arc::new(Letters)));
    (dir, store)
}

fn put(store: &Store, doc_type: DocType, body: Value) -> String {
    store
        .put(doc_type, body, Put::Upsert, Origin::Ui)
        .unwrap()
        .id
}

fn lock_envelope() -> Value {
    json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"})
}

/// Embeds whatever is waiting, as the app's indexer does.
fn index(store: &Store) {
    let queue = store.embedding_queue(MODEL).unwrap();
    let rows: Vec<_> = queue
        .pending
        .into_iter()
        .map(|p| {
            let vector = Letters.embed(&[p.text]).remove(0);
            (p.id, p.hash, vector)
        })
        .collect();
    store.put_embeddings(MODEL, &rows).unwrap();
}

/// What is done to a note after it was embedded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Plain,
    /// Sealed itself.
    Locked,
    /// Tagged `#ai-exclude`.
    Excluded,
    /// In a notebook that was locked afterwards.
    InLockedBook,
    Trashed,
}

fn kind() -> impl Strategy<Value = Kind> {
    prop_oneof![
        Just(Kind::Plain),
        Just(Kind::Locked),
        Just(Kind::Excluded),
        Just(Kind::InLockedBook),
        Just(Kind::Trashed),
    ]
}

struct Library {
    _dir: tempfile::TempDir,
    store: Store,
    /// Each note, what became of it, and the image it shows.
    notes: Vec<(String, Kind, String)>,
}

fn image(store: &Store, seed: usize) -> String {
    let bytes = format!("image {seed}");
    put(
        store,
        DocType::File,
        json!({
            "name": "shot.png",
            "contentType": "image/png",
            "_attachments": {"index": {
                "content_type": "image/png",
                "data": base64::engine::general_purpose::STANDARD.encode(bytes),
            }},
        }),
    )
}

/// A library of notes that all say the same, each tagged `topic` and
/// showing an image of its own, all embedded; then `kinds` happen to them.
fn library(kinds: &[Kind]) -> Library {
    let (dir, store) = open();
    let open_book = put(&store, DocType::Book, json!({"name": "Open"}));
    let private = put(&store, DocType::Book, json!({"name": "Private"}));
    let topic = put(&store, DocType::Tag, json!({"name": "topic"}));
    let secret = put(&store, DocType::Tag, json!({"name": "secret"}));
    let exclude = put(&store, DocType::Tag, json!({"name": "ai-exclude"}));
    let mut notes = Vec::new();
    for (n, kind) in kinds.iter().enumerate() {
        let file = image(&store, n);
        let book = if *kind == Kind::InLockedBook {
            &private
        } else {
            &open_book
        };
        let id = put(
            &store,
            DocType::Note,
            json!({
                "title": format!("Note {n}"),
                "body": format!("orchard fruit trees and pruning ![](inkdrop://{file})"),
                "bookId": book,
                "tags": [topic],
            }),
        );
        notes.push((id, *kind, file));
    }
    index(&store);
    assert_eq!(
        store.embedded_count(MODEL).unwrap(),
        kinds.len(),
        "every note starts embedded"
    );
    for (id, kind, _) in &notes {
        match kind {
            Kind::Plain | Kind::InLockedBook => {}
            Kind::Locked => {
                let mut sealed = store.get_api(id, None, false).unwrap();
                sealed["body"] = "".into();
                sealed["kansoLock"] = lock_envelope();
                sealed["tags"] = json!([topic, secret]);
                store.lock_note(sealed).unwrap();
            }
            Kind::Excluded => {
                store
                    .put(
                        DocType::Note,
                        json!({"_id": id, "tags": [topic, secret, exclude]}),
                        Put::Merge,
                        Origin::Ui,
                    )
                    .unwrap();
            }
            Kind::Trashed => {
                store
                    .put(
                        DocType::Note,
                        json!({"_id": id, "bookId": TRASH_BOOK_ID, "tags": [topic, secret]}),
                        Put::Merge,
                        Origin::Ui,
                    )
                    .unwrap();
            }
        }
        if *kind == Kind::InLockedBook {
            store
                .put(
                    DocType::Note,
                    json!({"_id": id, "tags": [topic, secret]}),
                    Put::Merge,
                    Origin::Ui,
                )
                .unwrap();
        }
    }
    if notes.iter().any(|(_, kind, _)| *kind == Kind::InLockedBook) {
        let mut book = store.get_api(&private, None, false).unwrap();
        book.as_object_mut().unwrap().remove("count");
        book["kansoLock"] = lock_envelope();
        store
            .put(DocType::Book, book, Put::Upsert, Origin::Sync)
            .unwrap();
    }
    Library {
        _dir: dir,
        store,
        notes,
    }
}

impl Library {
    fn plain(&self) -> Vec<&str> {
        self.notes
            .iter()
            .filter(|(_, kind, _)| *kind == Kind::Plain)
            .map(|(id, _, _)| id.as_str())
            .collect()
    }

    fn others(&self) -> Vec<&str> {
        self.notes
            .iter()
            .filter(|(_, kind, _)| *kind != Kind::Plain)
            .map(|(id, _, _)| id.as_str())
            .collect()
    }

    /// Nothing reads a vector of a note that may not be embedded.
    fn check_reads(&self) -> Result<(), TestCaseError> {
        let store = &self.store;
        let plain: HashSet<&str> = self.plain().into_iter().collect();
        for id in &plain {
            let related = store.related_notes(id, MODEL, 20).unwrap().unwrap();
            for note in &related {
                prop_assert!(plain.contains(note.id.as_str()), "related: {}", note.id);
            }
            let suggested = store.tag_suggestions(id, MODEL, 10).unwrap().unwrap();
            prop_assert!(
                suggested.iter().all(|s| s.name != "secret"),
                "a tag came from a note that may not be read"
            );
            if let Some(twin) = store.near_duplicate(id, MODEL).unwrap() {
                prop_assert!(plain.contains(twin.id.as_str()), "twin: {}", twin.id);
            }
        }
        for id in self.others() {
            prop_assert!(store.related_notes(id, MODEL, 20).unwrap().is_none());
            prop_assert!(store.tag_suggestions(id, MODEL, 10).unwrap().is_none());
            prop_assert!(store.near_duplicate(id, MODEL).unwrap().is_none());
        }
        let MeaningSearch::Found(found) = store.search_by_meaning("orchard fruit", 50).unwrap()
        else {
            return Err(TestCaseError::fail("the model is lent"));
        };
        for note in &found {
            prop_assert!(plain.contains(note.id.as_str()), "found: {}", note.id);
        }
        prop_assert_eq!(found.len(), plain.len());
        prop_assert_eq!(store.embedded_count(MODEL).unwrap(), plain.len());
        Ok(())
    }

    /// What the indexer is given, and what it may keep.
    fn check_index(&self) -> Result<(), TestCaseError> {
        let store = &self.store;
        let plain = self.plain().len();
        let queue = store.embedding_queue(MODEL).unwrap();
        prop_assert_eq!(queue.total, plain);
        prop_assert!(
            queue.pending.is_empty(),
            "everything embeddable is embedded"
        );
        // Vectors made earlier for the others are refused, whatever hash.
        let rows: Vec<_> = self
            .others()
            .into_iter()
            .map(|id| (id.to_owned(), "any".to_owned(), vec![1.0f32; 26]))
            .collect();
        store.put_embeddings(MODEL, &rows).unwrap();
        prop_assert_eq!(store.embedded_count(MODEL).unwrap(), plain);
        for id in self.others() {
            prop_assert!(store.related_notes(id, MODEL, 5).unwrap().is_none());
        }
        // Only the images plain notes show are waiting to be read.
        let waiting: HashSet<String> = store
            .unread_images(1000)
            .unwrap()
            .into_iter()
            .map(|image| image.id)
            .collect();
        let shown: HashSet<String> = self
            .notes
            .iter()
            .filter(|(_, kind, _)| *kind == Kind::Plain)
            .map(|(_, _, file)| file.clone())
            .collect();
        prop_assert_eq!(waiting, shown);
        Ok(())
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, ..ProptestConfig::default() })]

    /// However plain, locked, locked-notebook, `#ai-exclude` and trashed
    /// notes mix: the others are never related, suggested from, twins or
    /// found by meaning, never queued, never kept, and their images are
    /// never waiting to be read.
    #[test]
    fn prop_notes_that_may_not_be_read_are_never_discovered(
        kinds in prop::collection::vec(kind(), 1..8),
    ) {
        let library = library(&kinds);
        // Before the queue is read, the reads alone keep them out; after,
        // their vectors are gone too.
        library.check_reads()?;
        library.check_index()?;
        library.check_reads()?;
        index(&library.store);
        library.check_index()?;
    }

    /// While the lock is closed the graph has no locked note and no link to
    /// one; once open it has them all. `#ai-exclude` notes stay.
    #[test]
    fn prop_locked_notes_are_off_the_graph_while_locked(
        kinds in prop::collection::vec(kind(), 1..8),
    ) {
        let library = library(&kinds);
        // Every note links to the next one by title, and the first to the
        // last by ID.
        let store = &library.store;
        let visible: HashSet<&str> = library
            .notes
            .iter()
            .filter(|(_, kind, _)| matches!(kind, Kind::Plain | Kind::Excluded))
            .map(|(id, _, _)| id.as_str())
            .collect();
        let hidden: HashSet<&str> = library
            .notes
            .iter()
            .filter(|(_, kind, _)| matches!(kind, Kind::Locked | Kind::InLockedBook))
            .map(|(id, _, _)| id.as_str())
            .collect();
        let (closed_notes, closed_links) = store.graph_rows(false).unwrap();
        let ids: HashSet<&str> = closed_notes.iter().map(|n| n.id.as_str()).collect();
        prop_assert_eq!(&ids, &visible);
        let (open_notes, _) = store.graph_rows(true).unwrap();
        let open: HashSet<&str> = open_notes.iter().map(|n| n.id.as_str()).collect();
        let expected: HashSet<&str> = visible.union(&hidden).copied().collect();
        prop_assert_eq!(open, expected);
        let hidden_titles: HashSet<String> = library
            .notes
            .iter()
            .enumerate()
            .filter(|(_, (_, kind, _))| matches!(kind, Kind::Locked | Kind::InLockedBook))
            .map(|(n, _)| normalize_title(&format!("Note {n}")))
            .collect();
        for link in &closed_links {
            prop_assert!(!hidden.contains(link.from.as_str()));
            prop_assert!(link.target_id.as_deref().is_none_or(|id| !hidden.contains(id)));
            prop_assert!(link.target_title.as_ref().is_none_or(|t| !hidden_titles.contains(t)));
        }
    }
}

/// A plain note that links to another by its title and by its ID.
fn linking(store: &Store, book: &str, title: &str, to_title: &str, to_id: &str) -> String {
    put(
        store,
        DocType::Note,
        json!({
            "title": title,
            "body": format!("See [[{to_title}]] and [x](inkdrop://note/{}).", to_id.trim_start_matches("note:")),
            "bookId": book,
        }),
    )
}

#[test]
fn links_to_locked_notes_leave_the_graph_with_them() {
    let (_dir, store) = open();
    let book = put(&store, DocType::Book, json!({"name": "Open"}));
    let plain = put(
        &store,
        DocType::Note,
        json!({"title": "Open one", "body": "words", "bookId": book}),
    );
    let secret = put(
        &store,
        DocType::Note,
        json!({"title": "Secret", "body": "words", "bookId": book}),
    );
    let linker = linking(&store, &book, "Linker", "Secret", &plain);
    let both = linking(&store, &book, "Both", "Open one", &secret);
    let mut sealed = store.get_api(&secret, None, false).unwrap();
    sealed["body"] = "".into();
    sealed["kansoLock"] = lock_envelope();
    store.lock_note(sealed).unwrap();

    let (notes, links) = store.graph_rows(false).unwrap();
    let ids: HashSet<&str> = notes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(
        ids,
        HashSet::from([plain.as_str(), linker.as_str(), both.as_str()])
    );
    assert!(
        links
            .iter()
            .all(|l| l.target_id.as_deref() != Some(secret.as_str()))
    );
    assert!(
        links
            .iter()
            .all(|l| l.target_title.as_deref() != Some("secret"))
    );
    // The link from "Both" to "Open one" stays.
    assert!(
        links
            .iter()
            .any(|l| l.from == both && l.target_title.as_deref() == Some("open one"))
    );

    let (notes, links) = store.graph_rows(true).unwrap();
    assert_eq!(notes.len(), 4);
    assert!(
        links
            .iter()
            .any(|l| l.from == linker && l.target_title.as_deref() == Some("secret"))
    );
}

#[test]
fn a_title_a_locked_note_owns_does_not_move_to_another_note() {
    let (_dir, store) = open();
    let book = put(&store, DocType::Book, json!({"name": "Open"}));
    let first = put(
        &store,
        DocType::Note,
        json!({"title": "Same", "body": "words", "bookId": book}),
    );
    std::thread::sleep(std::time::Duration::from_millis(5));
    put(
        &store,
        DocType::Note,
        json!({"title": "Same", "body": "other words", "bookId": book}),
    );
    let linker = put(
        &store,
        DocType::Note,
        json!({"title": "Linker", "body": "[[Same]]", "bookId": book}),
    );
    let mut sealed = store.get_api(&first, None, false).unwrap();
    sealed["body"] = "".into();
    sealed["kansoLock"] = lock_envelope();
    store.lock_note(sealed).unwrap();
    let (_, links) = store.graph_rows(false).unwrap();
    assert!(
        !links.iter().any(|l| l.from == linker),
        "the link to the locked note's title isn't kept for the other note"
    );
    let (_, links) = store.graph_rows(true).unwrap();
    assert!(links.iter().any(|l| l.from == linker));
}

#[test]
fn an_image_shown_by_a_locked_note_and_a_plain_one_is_read() {
    let (_dir, store) = open();
    let book = put(&store, DocType::Book, json!({"name": "Open"}));
    let file = image(&store, 1);
    let lonely = image(&store, 2);
    let shows = |title: &str, file: &str| {
        put(
            &store,
            DocType::Note,
            json!({"title": title, "body": format!("![](inkdrop://{file})"), "bookId": book}),
        )
    };
    let secret = shows("Secret", &file);
    let secret_lonely = shows("Secret two", &lonely);
    shows("Plain", &file);
    for id in [&secret, &secret_lonely] {
        let mut sealed = store.get_api(id, None, false).unwrap();
        sealed["body"] = "".into();
        sealed["kansoLock"] = lock_envelope();
        store.lock_note(sealed).unwrap();
    }
    let waiting: Vec<String> = store
        .unread_images(10)
        .unwrap()
        .into_iter()
        .map(|i| i.id)
        .collect();
    assert_eq!(waiting, vec![file.clone()]);
}

#[test]
fn an_excluded_notes_image_is_not_read_until_the_tag_goes() {
    let (_dir, store) = open();
    let book = put(&store, DocType::Book, json!({"name": "Open"}));
    let exclude = put(&store, DocType::Tag, json!({"name": "#AI-Exclude"}));
    let file = image(&store, 3);
    let note = put(
        &store,
        DocType::Note,
        json!({"title": "N", "body": format!("![](inkdrop://{file})"), "bookId": book, "tags": [exclude]}),
    );
    assert!(store.unread_images(10).unwrap().is_empty());
    store
        .put(
            DocType::Note,
            json!({"_id": note, "tags": []}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(store.unread_images(10).unwrap().len(), 1);
}
