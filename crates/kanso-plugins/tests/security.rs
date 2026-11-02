//! The plugin read and write paths against generated libraries and
//! generated calls: no query, ID, cursor or argument a plugin can make
//! returns a locked note, a note in a locked notebook, a trashed note, a
//! note tagged `#ai-exclude` or a note outside its grant, nor an attachment
//! only such a note links; a hidden note reads exactly as a missing one;
//! and no change lands outside the notebooks it was granted.

use std::collections::{BTreeSet, HashSet};

use kanso_core::DocType;
use kanso_plugins::model::{
    EditorPermission, Grant, Installed, Manifest, NotesGrant, NotesRequest, Permissions, Publisher,
    ReadRequest, Source,
};
use kanso_plugins::router::{Editor, NoNetwork, Request, Router, Secrets, Selection};
use kanso_store::{Origin, PluginWriter, Put, Store};
use proptest::prelude::*;
use serde_json::{Value, json};

struct NoSecrets;
impl Secrets for NoSecrets {
    fn get(&self, _: &str, _: &str) -> Option<String> {
        None
    }
}

const CANARY: &str = "zebracanary";

/// What a generated note is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Plain,
    Sealed,
    Trashed,
    Excluded,
    Tagged,
}

#[derive(Clone, Debug)]
struct Library {
    /// Each notebook's parent (an earlier index), and whether it is locked.
    books: Vec<(Option<usize>, bool)>,
    /// Each note's notebook and kind.
    notes: Vec<(usize, Kind)>,
}

fn library() -> impl Strategy<Value = Library> {
    let books = prop::collection::vec(
        (prop::option::of(0..6usize), prop::bool::weighted(0.3)),
        2..6,
    )
    .prop_map(|books| {
        books
            .into_iter()
            .enumerate()
            .map(|(i, (parent, locked))| (parent.filter(|p| *p < i), locked))
            .collect::<Vec<_>>()
    });
    books.prop_flat_map(|books| {
        let n = books.len();
        let kinds = prop::sample::select(vec![
            Kind::Plain,
            Kind::Plain,
            Kind::Sealed,
            Kind::Trashed,
            Kind::Excluded,
            Kind::Tagged,
        ]);
        prop::collection::vec((0..n, kinds), 1..12).prop_map(move |notes| Library {
            books: books.clone(),
            notes,
        })
    })
}

struct Built {
    _dir: tempfile::TempDir,
    store: Store,
    books: Vec<String>,
    /// Note ID, its notebook index, its kind, and the file it links.
    notes: Vec<(String, usize, Kind, String)>,
    granted_tag: String,
}

fn put(store: &Store, doc_type: DocType, body: Value) -> String {
    store
        .put(doc_type, body, Put::Upsert, Origin::Ui)
        .unwrap()
        .id
}

fn sealed(store: &Store, id: &str) -> Value {
    let mut note = store.get_api(id, None, false).unwrap();
    note["body"] = "".into();
    note["kansoLock"] = json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "x"});
    note
}

fn build(lib: &Library) -> Built {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    let mut books: Vec<String> = Vec::new();
    for (i, (parent, _)) in lib.books.iter().enumerate() {
        let mut body = json!({ "name": format!("B{i}") });
        if let Some(parent) = parent {
            body["parentBookId"] = books[*parent].clone().into();
        }
        books.push(put(&store, DocType::Book, body));
    }
    let excluded = put(&store, DocType::Tag, json!({ "name": "AI-Exclude" }));
    let granted_tag = put(&store, DocType::Tag, json!({ "name": "share" }));
    let mut notes = Vec::new();
    for (n, (book, kind)) in lib.notes.iter().enumerate() {
        let file = put(
            &store,
            DocType::File,
            json!({ "name": "a.png", "contentType": "image/png", "_attachments": { "index": { "content_type": "image/png", "data": "aGk=" } } }),
        );
        let bare = file.trim_start_matches("file:");
        let mut body = json!({
            "bookId": books[*book],
            "title": format!("{CANARY} title {n}"),
            "body": format!("{CANARY} body ![a](inkdrop://file:{bare})"),
        });
        match kind {
            Kind::Excluded => body["tags"] = json!([excluded]),
            Kind::Tagged => body["tags"] = json!([granted_tag]),
            _ => {}
        }
        let id = put(&store, DocType::Note, body);
        match kind {
            Kind::Sealed => {
                store.lock_note(sealed(&store, &id)).unwrap();
            }
            Kind::Trashed => {
                store
                    .put(
                        DocType::Note,
                        json!({ "_id": id, "bookId": "trash" }),
                        Put::Merge,
                        Origin::Ui,
                    )
                    .unwrap();
            }
            _ => {}
        }
        notes.push((id, *book, *kind, file));
    }
    // Lock notebooks last, sealing what is in them as the app does.
    for (i, (_, locked)) in lib.books.iter().enumerate() {
        if !*locked || store.locked_books().unwrap().contains(&books[i]) {
            continue;
        }
        let inside = kanso_plugins::scope::with_descendants(
            &[books[i].clone()],
            &store
                .books()
                .unwrap()
                .into_iter()
                .map(|b| (b.id, b.parent_book_id))
                .collect::<Vec<_>>(),
        );
        let mut book = store.get_api(&books[i], None, false).unwrap();
        book.as_object_mut().unwrap().remove("count");
        book["kansoLock"] = json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "b"});
        let sealed_notes: Vec<Value> = store
            .notes_in_books(&inside.into_iter().collect::<Vec<_>>())
            .unwrap()
            .iter()
            .filter(|n| !n.is_locked())
            .map(|n| sealed(&store, &n.id))
            .collect();
        store.lock_book(book, sealed_notes).unwrap();
    }
    Built {
        _dir: dir,
        store,
        books,
        notes,
        granted_tag,
    }
}

fn installed(grant: Grant) -> Installed {
    Installed {
        manifest: Manifest {
            schema: 1,
            api_version: 1,
            id: "com.example.probe".into(),
            name: "Probe".into(),
            version: "1.0.0".into(),
            entry: "main.js".into(),
            publisher: Publisher {
                name: "P".into(),
                key: format!("ed25519:{}", "ab".repeat(32)),
            },
            min_app_version: "0.1.0".into(),
            icon: None,
            description: None,
            permissions: Permissions {
                commands: true,
                editor: [
                    EditorPermission::NoteRead,
                    EditorPermission::SelectionRead,
                    EditorPermission::SelectionReplace,
                ]
                .into(),
                notes: NotesRequest {
                    read: Some(ReadRequest {
                        all: true,
                        ..ReadRequest::default()
                    }),
                    write: vec!["any".into()],
                    create: true,
                },
                ..Permissions::default()
            },
        },
        source: Source::Developer { path: "/x".into() },
        pinned_key: String::new(),
        grant,
        disabled: None,
        registered: Default::default(),
        installed_at: 0,
        last_error: None,
    }
}

/// What the test expects the plugin may read: computed from the library
/// independently of the scope code.
fn expected_readable(
    built: &Built,
    lib: &Library,
    grant: &Grant,
    open: Option<&str>,
) -> HashSet<String> {
    let locked_book = |mut i: usize| loop {
        if lib.books[i].1 {
            return true;
        }
        match lib.books[i].0 {
            Some(parent) => i = parent,
            None => return false,
        }
    };
    let under = |mut i: usize, root: &str| loop {
        if built.books[i] == root {
            return true;
        }
        match lib.books[i].0 {
            Some(parent) => i = parent,
            None => return false,
        }
    };
    built
        .notes
        .iter()
        .filter(|(id, book, kind, _)| {
            let hidden =
                matches!(kind, Kind::Sealed | Kind::Trashed | Kind::Excluded) || locked_book(*book);
            let in_books =
                grant.notes.read_all || grant.notes.read_books.iter().any(|r| under(*book, r));
            let in_tags =
                *kind == Kind::Tagged && grant.notes.read_tags.contains(&built.granted_tag);
            let is_open =
                open == Some(id.as_str()) && grant.editor.contains(&EditorPermission::NoteRead);
            !hidden && (in_books || in_tags || is_open)
        })
        .map(|(id, ..)| id.clone())
        .collect()
}

fn grant_for(
    built: &Built,
    books: &BTreeSet<usize>,
    write: &BTreeSet<usize>,
    all: bool,
    tag: bool,
    editor: bool,
) -> Grant {
    let ids = |set: &BTreeSet<usize>| -> BTreeSet<String> {
        set.iter()
            .filter_map(|i| built.books.get(*i).cloned())
            .collect()
    };
    let read_books = ids(books);
    let write_books: BTreeSet<String> = ids(write)
        .into_iter()
        .filter(|b| all || read_books.contains(b))
        .collect();
    Grant {
        commands: true,
        editor: if editor {
            [
                EditorPermission::NoteRead,
                EditorPermission::SelectionRead,
                EditorPermission::SelectionReplace,
            ]
            .into()
        } else {
            BTreeSet::new()
        },
        notes: NotesGrant {
            read_books,
            read_tags: if tag {
                [built.granted_tag.clone()].into()
            } else {
                BTreeSet::new()
            },
            read_all: all,
            write_books,
            create: true,
        },
        fences: BTreeSet::new(),
        panel: false,
        secrets: BTreeSet::new(),
        network: BTreeSet::new(),
        ask_before_changing: true,
    }
}

fn queries() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        String::new(),
        CANARY.to_owned(),
        format!("\"{CANARY} body\""),
        "book:B0".to_owned(),
        "-book:B1".to_owned(),
        "tag:AI-Exclude".to_owned(),
        "tag:share".to_owned(),
        "status:none".to_owned(),
        "title:zebracanary".to_owned(),
        "inkdrop://file".to_owned(),
        "trash".to_owned(),
        "*".to_owned(),
        "'".to_owned(),
        "\" OR 1=1 --".to_owned(),
    ])
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Invariant: whatever the library, grant and calls, every note a
    /// plugin is shown is one it may read, everything it may read can be
    /// listed, and a hidden note's `get` is word for word a missing note's.
    #[test]
    fn prop_reads_never_leave_the_grant(
        lib in library(),
        books in prop::collection::btree_set(0..6usize, 0..3),
        write in prop::collection::btree_set(0..6usize, 0..3),
        all in prop::bool::weighted(0.2),
        tag in any::<bool>(),
        editor in any::<bool>(),
        open in prop::option::of(0..12usize),
        query in queries(),
        book_arg in prop::option::of(0..6usize),
        limit in prop::option::of(0u64..200),
        cursor in prop::option::of("[0-9-]{0,4}"),
    ) {
        let built = build(&lib);
        let grant = grant_for(&built, &books, &write, all, tag, editor);
        let open_id = open.and_then(|i| built.notes.get(i)).map(|(id, ..)| id.clone());
        let expected = expected_readable(&built, &lib, &grant, open_id.as_deref());
        let plugin = installed(grant.clone());
        let editor_state = Editor {
            note_id: open_id.clone(),
            selection: Some(Selection { text: "x".into(), start: 0, end: 1 }),
        };
        let dir = tempfile::tempdir().unwrap();
        let mut network = NoNetwork;
        let mut router = Router {
            store: &built.store,
            plugin: &plugin,
            plugin_dir: dir.path(),
            editor: &editor_state,
            request: Request::Command,
            secrets: &NoSecrets,
            network: &mut network,
            out: Default::default(),
        };

        // Lists, with crafted queries, notebooks, limits and cursors.
        let mut args = json!({ "query": query });
        if let Some(b) = book_arg.and_then(|b| built.books.get(b)) {
            args["bookId"] = b.clone().into();
        }
        if let Some(limit) = limit {
            args["limit"] = limit.into();
        }
        if let Some(cursor) = &cursor {
            args["cursor"] = cursor.clone().into();
        }
        if let Ok(listed) = router.call("notes.list", args) {
            let text = listed.to_string();
            for note in listed["notes"].as_array().unwrap() {
                let id = note["id"].as_str().unwrap();
                prop_assert!(expected.contains(id), "listed {} outside the grant", id);
            }
            // A hidden note's ID or title never shows, even in passing.
            for (n, (id, ..)) in built.notes.iter().enumerate() {
                if !expected.contains(id) {
                    prop_assert!(!text.contains(id.as_str()));
                    let title = format!("{} title {}\"", CANARY, n);
                    prop_assert!(!text.contains(&title));
                }
            }
        }
        // Every readable note is reachable by paging an empty query.
        if grant.notes.reads() || editor {
            let mut seen = HashSet::new();
            let mut cursor: Option<String> = None;
            for _ in 0..20 {
                let mut args = json!({ "limit": 3 });
                if let Some(c) = &cursor { args["cursor"] = c.clone().into(); }
                let page = router.call("notes.list", args).unwrap();
                for note in page["notes"].as_array().unwrap() {
                    seen.insert(note["id"].as_str().unwrap().to_owned());
                }
                cursor = page["cursor"].as_str().map(str::to_owned);
                if cursor.is_none() { break; }
            }
            prop_assert_eq!(&seen, &expected);
        }

        // Gets: hidden reads exactly as missing.
        let missing = router.call("notes.get", json!({ "id": "note:doesnotexist00000000" }));
        for (id, _, _, file) in &built.notes {
            let got = router.call("notes.get", json!({ "id": id }));
            if expected.contains(id) {
                prop_assert_eq!(&got.unwrap()["id"], &json!(id));
            } else if grant.notes.reads() || editor {
                prop_assert_eq!(&got, &missing);
                let data = router.call("notes.pluginData.get", json!({ "id": id }));
                prop_assert_eq!(&data, &missing);
            } else {
                prop_assert!(got.is_err());
            }
            // An attachment is reachable only through a readable note.
            let attachment = router.call("attachments.get", json!({ "fileId": file }));
            prop_assert_eq!(attachment.is_ok(), expected.contains(id), "file of {}", id);
        }

        // The editor never hands over a hidden open note or its selection.
        if editor {
            let note = router.call("editor.noteId", json!({})).unwrap();
            let selection = router.call("editor.selection", json!({})).unwrap();
            match &open_id {
                Some(id) if expected.contains(id) => {
                    prop_assert_eq!(&note, &json!(id));
                    prop_assert!(!selection.is_null());
                }
                _ => {
                    prop_assert!(note.is_null());
                    prop_assert!(selection.is_null());
                    let replaced = router.call("editor.replaceSelection", json!({ "text": "y" }));
                    prop_assert!(replaced.is_err());
                }
            }
        }
    }

    /// Invariant: of any updates a plugin asks for, only notes in its
    /// writable notebooks are staged, and once the batch is applied no
    /// other note has changed.
    #[test]
    fn prop_writes_never_leave_the_grant(
        lib in library(),
        books in prop::collection::btree_set(0..6usize, 0..4),
        write in prop::collection::btree_set(0..6usize, 0..4),
        all in prop::bool::weighted(0.2),
        targets in prop::collection::vec(0..12usize, 1..8),
        create_in in prop::option::of(0..6usize),
    ) {
        let built = build(&lib);
        let grant = grant_for(&built, &books, &write, all, false, false);
        let plugin = installed(grant.clone());
        let editor_state = Editor::default();
        let dir = tempfile::tempdir().unwrap();
        let mut network = NoNetwork;
        let mut router = Router {
            store: &built.store,
            plugin: &plugin,
            plugin_dir: dir.path(),
            editor: &editor_state,
            request: Request::Command,
            secrets: &NoSecrets,
            network: &mut network,
            out: Default::default(),
        };
        let readable = expected_readable(&built, &lib, &grant, None);
        let writable_books: HashSet<String> = {
            let all_books: Vec<(String, Option<String>)> = built.store.books().unwrap().into_iter().map(|b| (b.id, b.parent_book_id)).collect();
            kanso_plugins::scope::with_descendants(&grant.notes.write_books, &all_books)
        };
        let before: Vec<Value> = built.notes.iter().map(|(id, ..)| built.store.get_api(id, None, false).unwrap()).collect();
        for t in &targets {
            let Some((id, ..)) = built.notes.get(*t) else { continue };
            let rev = built.store.get(id).unwrap().unwrap().rev;
            let _ = router.call("notes.update", json!({ "id": id, "baseRev": rev, "fields": { "body": "changed by plugin" } }));
            let _ = router.call("notes.pluginData.set", json!({ "id": id, "value": { "n": 1 } }));
        }
        if let Some(b) = create_in.and_then(|b| built.books.get(b)) {
            let _ = router.call("notes.create", json!({ "bookId": b, "title": "made", "body": "made" }));
        }
        let batch = router.out.batch.clone();
        for edit in &batch.edits {
            prop_assert!(readable.contains(&edit.id));
            let note = built.store.note(&edit.id).unwrap().unwrap();
            prop_assert!(writable_books.contains(&note.book_id));
        }
        for create in &batch.creates {
            prop_assert!(grant.notes.write_books.contains(&create.book_id) || writable_books.contains(&create.book_id));
            prop_assert!(!built.store.locked_books().unwrap().contains(&create.book_id));
        }
        let writer = PluginWriter { plugin_id: "com.example.probe", plugin_name: "Probe", batch: &batch.id };
        built.store.apply_plugin_batch(writer, &batch.edits, &batch.creates, 5).unwrap();
        let staged: HashSet<&str> = batch.edits.iter().map(|e| e.id.as_str()).collect();
        for ((id, ..), before) in built.notes.iter().zip(&before) {
            let now = built.store.get_api(id, None, false).unwrap();
            if !staged.contains(id.as_str()) {
                prop_assert_eq!(&now, before, "{} changed outside the grant", id);
            }
        }
    }
}

#[test]
fn a_renderer_can_call_nothing() {
    let lib = Library {
        books: vec![(None, false)],
        notes: vec![(0, Kind::Plain)],
    };
    let built = build(&lib);
    let mut grant = grant_for(&built, &[0].into(), &[0].into(), true, false, true);
    grant.network.insert("api.example.com".into());
    let plugin = installed(grant);
    let dir = tempfile::tempdir().unwrap();
    let mut network = NoNetwork;
    let editor = Editor::default();
    let mut router = Router {
        store: &built.store,
        plugin: &plugin,
        plugin_dir: dir.path(),
        editor: &editor,
        request: Request::Render,
        secrets: &NoSecrets,
        network: &mut network,
        out: Default::default(),
    };
    for call in kanso_plugins::protocol::CALLS {
        let refused = router
            .call(call, json!({ "id": built.notes[0].0 }))
            .unwrap_err();
        assert_eq!(refused.code, "denied", "{call}");
    }
    assert_eq!(
        router.out.refused as usize,
        kanso_plugins::protocol::CALLS.len()
    );
}

#[test]
fn calls_without_a_permission_are_refused_and_counted() {
    let lib = Library {
        books: vec![(None, false)],
        notes: vec![(0, Kind::Plain)],
    };
    let built = build(&lib);
    let plugin = installed(Grant {
        commands: true,
        editor: BTreeSet::new(),
        notes: NotesGrant::default(),
        fences: BTreeSet::new(),
        panel: false,
        secrets: BTreeSet::new(),
        network: BTreeSet::new(),
        ask_before_changing: true,
    });
    let dir = tempfile::tempdir().unwrap();
    let mut network = NoNetwork;
    let editor = Editor {
        note_id: Some(built.notes[0].0.clone()),
        selection: None,
    };
    let mut router = Router {
        store: &built.store,
        plugin: &plugin,
        plugin_dir: dir.path(),
        editor: &editor,
        request: Request::Command,
        secrets: &NoSecrets,
        network: &mut network,
        out: Default::default(),
    };
    let id = &built.notes[0].0;
    for (call, args) in [
        ("notes.get", json!({ "id": id })),
        ("notes.list", json!({})),
        (
            "notes.update",
            json!({ "id": id, "baseRev": "1-x", "fields": { "body": "x" } }),
        ),
        ("editor.noteId", json!({})),
        ("fetch", json!({ "url": "https://example.com" })),
        ("secrets.get", json!({ "name": "token" })),
        ("ui.panel", json!({ "tree": { "type": "divider" } })),
        ("notes.delete", json!({ "id": id })),
        ("fs.read", json!({ "path": "/etc/passwd" })),
    ] {
        assert_eq!(
            router.call(call, args).unwrap_err().code,
            "denied",
            "{call}"
        );
    }
    assert_eq!(router.out.refused, 9);
    // Settings are the plugin's own.
    router
        .call("settings.set", json!({ "value": { "a": 1 } }))
        .unwrap();
    assert_eq!(router.call("settings.get", json!({})).unwrap()["a"], 1);
}

#[test]
fn an_update_can_name_only_its_four_fields() {
    let lib = Library {
        books: vec![(None, false)],
        notes: vec![(0, Kind::Plain)],
    };
    let built = build(&lib);
    let plugin = installed(grant_for(
        &built,
        &[0].into(),
        &[0].into(),
        false,
        false,
        false,
    ));
    let dir = tempfile::tempdir().unwrap();
    let mut network = NoNetwork;
    let editor = Editor::default();
    let mut router = Router {
        store: &built.store,
        plugin: &plugin,
        plugin_dir: dir.path(),
        editor: &editor,
        request: Request::Command,
        secrets: &NoSecrets,
        network: &mut network,
        out: Default::default(),
    };
    let id = &built.notes[0].0;
    let rev = built.store.get(id).unwrap().unwrap().rev;
    for field in [
        "bookId",
        "kansoLock",
        "kansoPublish",
        "kansoShare",
        "_rev",
        "pinned",
        "share",
        "reminders",
    ] {
        let refused = router
            .call(
                "notes.update",
                json!({ "id": id, "baseRev": rev, "fields": { field: "x" } }),
            )
            .unwrap_err();
        assert_eq!(refused.code, "invalid", "{field}");
    }
    let stale = router
        .call(
            "notes.update",
            json!({ "id": id, "baseRev": "1-stale", "fields": { "body": "x" } }),
        )
        .unwrap_err();
    assert_eq!(stale.code, "conflict");
    assert!(router.out.batch.is_empty());
}
