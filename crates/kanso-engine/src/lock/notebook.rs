//! Locked notebooks: what locking one seals, and what taking its lock off
//! leaves. The store reads every note in a locked notebook as locked, sealed
//! or not, so its text stays out of search, the local API and related notes
//! (`kanso_store`'s `locked_books`).

use std::collections::{HashMap, HashSet};

use kanso_core::{Book, DocType, Note};
use kanso_store::{Origin, Put, Store};
use kanso_sync::lock::{Envelope, LockKey};

use super::files::{self, Job, KeyBook, Progress};

/// Notebook `id` and every notebook inside it, at any depth: what locking
/// it seals. Should the notebooks form a cycle, each is taken once.
pub fn book_subtree(id: &str, books: &[Book]) -> Vec<String> {
    let mut found = vec![id.to_owned()];
    let mut i = 0;
    while i < found.len() {
        for book in books {
            if book.parent_book_id.as_deref() == Some(found[i].as_str())
                && !found.contains(&book.id)
            {
                found.push(book.id.clone());
            }
        }
        i += 1;
    }
    found
}

/// Locks notebook `book_id` with `key`: seals every note in it, and in the
/// notebooks inside it, that isn't sealed yet, and sets the notebook's lock,
/// in one write. Returns how many notes were sealed. A note locked on its
/// own keeps its seal. The notes' pictures and files are sealed with them
/// ([`seal_notebook_with`] keeps the keys and reports progress).
pub fn seal_notebook(store: &Store, book_id: &str, key: &LockKey) -> Result<usize, String> {
    seal_notebook_with(store, book_id, key, &KeyBook::default(), &mut |_| Ok(()))
}

/// [`seal_notebook`], with the file keys it makes put in `keys` and `watch`
/// told how the sealing goes: it can stop the lock between files by
/// returning an error, which leaves the notes plain with a journal each
/// (`files::interrupted`).
pub fn seal_notebook_with(
    store: &Store,
    book_id: &str,
    key: &LockKey,
    keys: &KeyBook,
    watch: &mut dyn FnMut(&Progress) -> Result<(), String>,
) -> Result<usize, String> {
    let books = store.books().map_err(|err| err.to_string())?;
    let mut book: Book = books
        .iter()
        .find(|book| book.id == book_id)
        .cloned()
        .ok_or_else(|| "The notebook was deleted.".to_owned())?;
    let inside = book_subtree(book_id, &books);
    let notes = store
        .notes_in_books(&inside)
        .map_err(|err| err.to_string())?;
    // Every note in the subtree is sealed in this pass, so a file they share
    // stays one sealed original rather than becoming a copy each.
    let locking: HashSet<String> = notes.iter().map(|note| note.id.clone()).collect();
    let mut copies = HashMap::new();
    let plain: Vec<Note> = notes.into_iter().filter(|note| !note.is_locked()).collect();
    let mut job = Job::new(store, keys, &locking, &mut copies, watch);
    job.expect(files::summarize(store, &plain));
    let mut sealed = Vec::new();
    for mut note in plain {
        let text = note.body.clone();
        files::seal_note_files(&mut job, &mut note, &text, key).map_err(|err| err.to_string())?;
        note.body.clear();
        sealed.push(serde_json::to_value(&note).map_err(|err| err.to_string())?);
    }
    // The notebook's own lock seals nothing: it tells the password and its
    // salt to a device that opens the notebook before any of its notes.
    Envelope::store(Some(&key.seal(book_id, "")), &mut book.extra);
    let count = sealed.len();
    let book = serde_json::to_value(&book).map_err(|err| err.to_string())?;
    store
        .lock_book(book, sealed)
        .map_err(|err| err.to_string())?;
    Ok(count)
}

/// What locking notebook `book_id` would seal: the plain pictures and files
/// of the notes in it and in the notebooks inside it that aren't locked.
pub fn notebook_summary(store: &Store, book_id: &str) -> files::Summary {
    let Ok(books) = store.books() else {
        return files::Summary::default();
    };
    let inside = book_subtree(book_id, &books);
    let plain: Vec<Note> = store
        .notes_in_books(&inside)
        .unwrap_or_default()
        .into_iter()
        .filter(|note| !note.is_locked())
        .collect();
    files::summarize(store, &plain)
}

/// Writes notebook `book_id` without its lock. Its notes keep theirs.
pub fn take_lock_off(store: &Store, book_id: &str) -> Result<(), String> {
    let books = store.books().map_err(|err| err.to_string())?;
    let mut book = books
        .into_iter()
        .find(|book| book.id == book_id)
        .ok_or_else(|| "The notebook was deleted.".to_owned())?;
    Envelope::store(None, &mut book.extra);
    let value = serde_json::to_value(&book).map_err(|err| err.to_string())?;
    store
        .put(DocType::Book, value, Put::Upsert, Origin::Ui)
        .map(drop)
        .map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use kanso_sync::KdfParams;
    use kanso_sync::lock::LockPassword;
    use proptest::prelude::*;
    use serde_json::json;

    use super::*;

    fn book(ix: usize, parent: Option<usize>) -> Book {
        serde_json::from_value(json!({
            "_id": format!("b{ix}"),
            "name": format!("Book {ix}"),
            "parentBookId": parent.map(|p| format!("b{p}")),
        }))
        .unwrap()
    }

    fn key(text: &str) -> LockKey {
        LockKey::create(&LockPassword::new(text).unwrap(), KdfParams::TEST).unwrap()
    }

    proptest! {
        /// Invariant: a notebook's subtree starts with it, holds each
        /// notebook once, is closed under "inside", and holds only
        /// notebooks that are inside it; a parent cycle ends the walk.
        #[test]
        fn prop_a_subtree_is_the_notebook_and_all_inside_it(
            parents in prop::collection::vec(prop::option::of(0usize..8), 1..8),
            root in any::<prop::sample::Index>(),
        ) {
            let books: Vec<Book> = parents
                .iter()
                .enumerate()
                .map(|(ix, parent)| book(ix, *parent))
                .collect();
            let root = &books[root.index(books.len())].id;
            let inside = book_subtree(root, &books);
            prop_assert_eq!(inside.first(), Some(root));
            let unique: HashSet<&String> = inside.iter().collect();
            prop_assert_eq!(unique.len(), inside.len());
            for b in &books {
                let parent_inside = b
                    .parent_book_id
                    .as_ref()
                    .is_some_and(|parent| inside.contains(parent));
                // Closed under children.
                if parent_inside {
                    prop_assert!(inside.contains(&b.id), "{} is inside", b.id);
                }
                // Nothing else joins, but the notebook itself.
                if inside.contains(&b.id) && &b.id != root {
                    prop_assert!(parent_inside, "{} hangs off the subtree", b.id);
                }
            }
        }
    }

    #[derive(Debug)]
    struct Spec {
        parents: Vec<Option<usize>>,
        /// Per note: its notebook, its text and whether another key locked it.
        notes: Vec<(usize, String, bool)>,
        root: usize,
    }

    fn specs() -> impl Strategy<Value = Spec> {
        (
            prop::collection::vec(prop::option::of(0usize..6), 1..6),
            prop::collection::vec((0usize..6, "\\PC{0,40}", any::<bool>()), 0..8),
            0usize..6,
        )
            .prop_map(|(parents, notes, root)| {
                let count = parents.len();
                // The store refuses a notebook inside itself, so a notebook's
                // parent is one made before it. (A cycle only arrives through
                // sync; `book_subtree` is tested with cycles above.)
                let parents = parents
                    .into_iter()
                    .enumerate()
                    .map(|(ix, parent)| parent.filter(|_| ix > 0).map(|parent| parent % ix))
                    .collect();
                Spec {
                    notes: notes
                        .into_iter()
                        .map(|(book, text, other)| (book % count, text, other))
                        .collect(),
                    root: root % count,
                    parents,
                }
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        /// Invariant: locking a notebook seals exactly the notes in it and
        /// the notebooks inside it. Each opens to its own text, with the new
        /// key, or with the other key when it was locked on its own; the
        /// notes outside, and those locked before, are untouched.
        #[test]
        fn prop_sealing_a_notebook_seals_exactly_its_subtree(spec in specs()) {
            let dir = tempfile::tempdir().unwrap();
            let store = Store::open(dir.path().join("k.db")).unwrap();
            let (mine, other) = (key("this notebook's"), key("another one"));
            let made: Vec<_> = spec
                .parents
                .iter()
                .enumerate()
                .map(|(ix, _)| {
                    store
                        .put(DocType::Book, json!({"name": format!("Book {ix}")}), Put::Upsert, Origin::Ui)
                        .unwrap()
                })
                .collect();
            let ids: Vec<String> = made.iter().map(|made| made.id.clone()).collect();
            // Parents are set once every notebook exists, so a cycle can form.
            for (ix, parent) in spec.parents.iter().enumerate() {
                store
                    .put(
                        DocType::Book,
                        json!({
                            "_id": ids[ix],
                            "_rev": made[ix].rev,
                            "name": format!("Book {ix}"),
                            "parentBookId": parent.map(|p| ids[p].clone()),
                        }),
                        Put::Upsert,
                        Origin::Ui,
                    )
                    .unwrap();
            }
            let mut notes = Vec::new();
            for (book, text, locked_apart) in &spec.notes {
                let id = store
                    .put(
                        DocType::Note,
                        json!({"bookId": ids[*book], "title": "T", "body": text}),
                        Put::Upsert,
                        Origin::Ui,
                    )
                    .unwrap()
                    .id;
                if *locked_apart {
                    let mut note = store.note(&id).unwrap().unwrap();
                    Envelope::store(Some(&other.seal(&id, text)), &mut note.extra);
                    note.body.clear();
                    store.lock_note(serde_json::to_value(&note).unwrap()).unwrap();
                }
                notes.push(id);
            }
            let books = store.books().unwrap();
            let inside = book_subtree(&ids[spec.root], &books);
            let before: Vec<serde_json::Value> = notes
                .iter()
                .map(|id| serde_json::to_value(store.note(id).unwrap().unwrap()).unwrap())
                .collect();

            let count = seal_notebook(&store, &ids[spec.root], &mine).unwrap();

            let mut fresh = 0;
            for (n, id) in notes.iter().enumerate() {
                let (book, text, locked_apart) = &spec.notes[n];
                let note = store.note(id).unwrap().unwrap();
                if inside.contains(&ids[*book]) {
                    prop_assert!(note.is_locked());
                    prop_assert_eq!(note.body.as_str(), "");
                    let envelope = Envelope::from_extra(&note.extra).unwrap().unwrap();
                    let opener = if *locked_apart { &other } else { &mine };
                    let opened = opener.open(id, &envelope).unwrap();
                    prop_assert_eq!(opened.as_str(), text.as_str());
                    if !locked_apart {
                        fresh += 1;
                    }
                }
                if !inside.contains(&ids[*book]) || *locked_apart {
                    // Outside, or locked before: not a byte different.
                    prop_assert_eq!(serde_json::to_value(&note).unwrap(), before[n].clone());
                }
            }
            prop_assert_eq!(count, fresh);
            let book = store.books().unwrap().into_iter().find(|b| b.id == ids[spec.root]).unwrap();
            let envelope = Envelope::from_extra(&book.extra).unwrap().unwrap();
            let opened = mine.open(&book.id, &envelope).unwrap();
            prop_assert_eq!(opened.as_str(), "");
        }
    }

    #[test]
    fn taking_a_notebooks_lock_off_leaves_its_notes_locked() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let mine = key("this notebook's");
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let note = store
            .put(
                DocType::Note,
                json!({"bookId": book, "title": "T", "body": "secret"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        seal_notebook(&store, &book, &mine).unwrap();
        take_lock_off(&store, &book).unwrap();
        assert!(!store.books().unwrap()[0].is_locked());
        assert!(store.note(&note).unwrap().unwrap().is_locked());
        assert_eq!(
            take_lock_off(&store, "gone").unwrap_err(),
            "The notebook was deleted."
        );
    }
}
