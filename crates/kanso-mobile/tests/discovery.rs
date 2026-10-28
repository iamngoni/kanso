//! Discovery on a phone: related notes on and off, what they find, the
//! tags and twins, search by meaning, the graph and the words in images,
//! and that a locked note or one tagged `#ai-exclude` is never part of it.
//!
//! The model is never downloaded: the tests start an indexer with a tiny
//! model of their own and an embedder that counts letters.

mod common;

use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Once};
use std::time::Duration;

use base64::Engine as _;
use common::*;
use kanso_core::DocType;
use kanso_engine::discovery::indexer::Indexer;
use kanso_engine::discovery::model_fetch::{Fetch, Model, ModelError, ModelFile, Opened};
use kanso_engine::discovery::related::MODEL_ID;
use kanso_mobile::{
    Discovery, GraphFilter, GraphRequest, KansoError, Library, MeaningResult, NoteChange,
    RelatedState, RelatedStatus,
};
use kanso_store::{Embedder, Origin, Put, Store};
use proptest::prelude::*;
use serde_json::json;

/// Counts letters: texts with the same letters point the same way. It
/// answers to the real model's name, as the app asks for that one.
struct Letters;

impl Embedder for Letters {
    fn model(&self) -> &str {
        MODEL_ID
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

/// Serves the tiny model's one file from memory.
struct Served(Vec<u8>);

impl Fetch for Served {
    fn open(&self, _url: &str, from: u64) -> Result<Opened, ModelError> {
        Ok(Opened {
            body: Box::new(std::io::Cursor::new(self.0[from as usize..].to_vec())),
            from,
        })
    }
}

/// SHA-256 of "abc".
const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const FILES: [ModelFile; 1] = [ModelFile {
    name: "weights.bin",
    size: 3,
    sha256: ABC,
}];
const TINY: Model = Model {
    repository: "o/m",
    revision: "r",
    folder: "models/potion-base-8M",
    files: &FILES,
};

/// Every library opened by these tests starts the tiny indexer.
fn install() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        Discovery::use_indexer_for_test(Some(Arc::new(|store, data_dir, on_status| {
            Indexer::start_with(
                store,
                TINY,
                TINY.dir(&data_dir),
                Arc::new(Served(b"abc".to_vec())),
                Arc::new(|_: &Path| Ok(Arc::new(Letters) as Arc<dyn Embedder>)),
                on_status,
            )
        })));
    });
}

fn status_stream(library: &Library) -> Receiver<RelatedState> {
    let (tx, rx) = mpsc::channel();
    library
        .discovery()
        .related_stream(move |state| tx.send(state).is_ok());
    rx
}

/// Waits until related notes are ready with all of `notes` indexed.
fn ready(states: &Receiver<RelatedState>, notes: u32) {
    loop {
        let state = states
            .recv_timeout(Duration::from_secs(30))
            .expect("related notes get ready");
        match state.status {
            RelatedStatus::Ready { notes: n } if n == notes => return,
            RelatedStatus::Failed { message } => panic!("failed: {message}"),
            _ => {}
        }
    }
}

const ORCHARD: &str = "orchard fruit trees and pruning in winter";

/// A library whose related notes are on and ready with `notes` indexed.
fn indexed(notes: u32, seed: impl FnOnce(&Phone)) -> (Phone, Receiver<RelatedState>) {
    install();
    let phone = lone_phone();
    seed(&phone);
    let states = status_stream(&phone.library);
    phone.library.discovery().set_related_notes(true).unwrap();
    ready(&states, notes);
    (phone, states)
}

fn store(phone: &Phone) -> Store {
    Store::open(phone.path()).unwrap()
}

#[test]
fn related_notes_turn_on_find_notes_and_turn_off() {
    install();
    let phone = lone_phone();
    let library = &phone.library;
    let discovery = library.discovery();
    let a = note(library, "Orchard", ORCHARD);
    let b = note(
        library,
        "Pruning",
        "fruit trees pruned in winter, orchard care",
    );
    note(library, "Taxes", "invoices and receipts for the accountant");
    assert!(!discovery.related_enabled().unwrap());
    assert_eq!(
        discovery.related_state().unwrap().status,
        RelatedStatus::Off
    );
    // Nothing is found while it is off.
    assert!(discovery.related_notes(&a, 5).unwrap().is_empty());
    assert_eq!(
        discovery.search_by_meaning("orchard fruit", 5).unwrap(),
        MeaningResult::Off
    );

    let states = status_stream(library);
    discovery.set_related_notes(true).unwrap();
    assert!(discovery.related_enabled().unwrap());
    ready(&states, 3);
    let state = discovery.related_state().unwrap();
    assert_eq!(state.title.as_deref(), Some("Ready"));
    assert!(state.detail.unwrap().starts_with("3 notes indexed"));
    assert!(TINY.dir(phone.dir.path()).join("weights.bin").exists());

    let related = discovery.related_notes(&a, 5).unwrap();
    assert_eq!(related[0].id, b);
    assert!(related[0].score > 0.5);
    assert!(!related[0].label.is_empty());
    assert!(related.len() <= 2);
    let MeaningResult::Found { rows } = discovery.search_by_meaning("orchard fruit", 5).unwrap()
    else {
        panic!("found nothing");
    };
    assert!(rows.iter().any(|row| row.id == a));
    // A search that isn't plain words goes by words alone.
    assert_eq!(
        discovery.search_by_meaning("tag:work", 5).unwrap(),
        MeaningResult::Found { rows: Vec::new() }
    );

    // Off forgets the embeddings and keeps the model.
    discovery.set_related_notes(false).unwrap();
    assert!(!discovery.related_enabled().unwrap());
    assert_eq!(
        discovery.related_state().unwrap().status,
        RelatedStatus::Off
    );
    assert!(discovery.related_notes(&a, 5).unwrap().is_empty());
    assert_eq!(
        discovery.search_by_meaning("orchard fruit", 5).unwrap(),
        MeaningResult::Off
    );
    assert!(TINY.dir(phone.dir.path()).join("weights.bin").exists());
    assert_eq!(store(&phone).embedded_count(MODEL_ID).unwrap(), 0);
}

#[test]
fn related_notes_left_on_start_again_when_the_library_opens() {
    install();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.db");
    {
        let store = Store::open(&path).unwrap();
        let book = book(&store, "Notes");
        for title in ["One", "Two"] {
            raw_note(
                &store,
                json!({"title": title, "body": ORCHARD, "bookId": book}),
            );
        }
        store.set_embedding_model(Some(MODEL_ID)).unwrap();
    }
    let library = Library::open_for_test(
        &path,
        Arc::new(kanso_mobile::MemorySecrets::new()),
        Arc::new(kanso_sync::MemoryServer::new()),
    )
    .unwrap();
    let states = status_stream(&library);
    ready(&states, 2);
    assert!(library.discovery().related_enabled().unwrap());
}

#[test]
fn the_models_folder_is_inside_the_folder_kept_out_of_backups() {
    install();
    let phone = lone_phone();
    let discovery = phone.library.discovery();
    // The library keeps the folder's real path (a temporary folder on macOS
    // is under `/var`, which is a link to `/private/var`).
    let folder = phone.dir.path().canonicalize().unwrap();
    assert!(discovery.model_dir().starts_with(&folder));
    let expected = cfg!(target_vendor = "apple").then_some(true);
    assert_eq!(kanso_mobile::diagnostics::backup_excluded(&folder), expected);
}

#[test]
fn a_locked_or_excluded_note_is_never_related_suggested_twinned_or_found() {
    let mut ids = HashMap::new();
    let (phone, states) = indexed(5, |phone| {
        let store = store(phone);
        let topic = tag(&store, "topic");
        let secret = tag(&store, "secret");
        let book = book(&store, "Notes");
        // The same words in every one, so each would be a twin. The words
        // are said twice because the tests' letter-count embedder needs
        // enough of them shared to reach the duplicate threshold (0.95):
        // titles alone would pull two otherwise identical notes apart.
        for (name, tags) in [
            ("plain", vec![&topic]),
            ("friend", vec![&topic]),
            ("locked", vec![&topic, &secret]),
            ("excluded", vec![&topic, &secret]),
            ("vaulted", vec![&topic, &secret]),
        ] {
            let id = raw_note(
                &store,
                json!({"title": name, "body": format!("{ORCHARD}. {ORCHARD}. {name}"), "bookId": book, "tags": tags}),
            );
            ids.insert(name, id);
        }
    });
    let library = &phone.library;
    let discovery = library.discovery();
    // One is locked with the lock password, one is tagged, and one is in a
    // notebook that was locked: all after they were embedded.
    library
        .locks()
        .lock_note(&ids["locked"], LOCK_PASSWORD.into())
        .unwrap();
    {
        let store = store(&phone);
        let exclude = tag(&store, "ai-exclude");
        let secret = tag(&store, "secret");
        let topic = tag(&store, "topic");
        store
            .put(
                DocType::Note,
                json!({"_id": ids["excluded"], "tags": [topic, secret, exclude]}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        let vault = book(&store, "Vault");
        store
            .put(
                DocType::Note,
                json!({"_id": ids["vaulted"], "bookId": vault}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        let mut notebook = store.get_api(&vault, None, false).unwrap();
        notebook.as_object_mut().unwrap().remove("count");
        notebook["kansoLock"] =
            json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "nb"});
        store
            .put(DocType::Book, notebook, Put::Upsert, Origin::Sync)
            .unwrap();
    }
    // Unlocked or not, none of them is discovered, before the indexer has
    // looked again.
    for session_open in [true, false] {
        if !session_open {
            library.locks().lock().unwrap();
        }
        for hidden in ["locked", "excluded", "vaulted"] {
            assert!(
                discovery
                    .related_notes(&ids[hidden], 10)
                    .unwrap()
                    .is_empty()
            );
            let meaning = discovery.note_meaning(&ids[hidden]).unwrap();
            assert!(meaning.tags.is_empty() && meaning.twin.is_none());
        }
        let related = discovery.related_notes(&ids["plain"], 10).unwrap();
        assert_eq!(
            related.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
            [ids["friend"].as_str()]
        );
        let meaning = discovery.note_meaning(&ids["plain"]).unwrap();
        assert!(meaning.tags.iter().all(|t| t.name != "secret"));
        assert_eq!(
            meaning.twin.as_ref().map(|t| t.id.as_str()),
            Some(ids["friend"].as_str())
        );
        let MeaningResult::Found { rows } =
            discovery.search_by_meaning("orchard fruit", 10).unwrap()
        else {
            panic!("found nothing");
        };
        let mut found: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
        found.sort();
        let mut expected = [ids["plain"].as_str(), ids["friend"].as_str()];
        expected.sort();
        assert_eq!(found, expected);
    }
    // The indexer forgets their vectors when it looks again: a change wakes it.
    library
        .change(vec![ids["plain"].clone()], NoteChange::TogglePin)
        .unwrap();
    ready(&states, 2);
    assert_eq!(store(&phone).embedded_count(MODEL_ID).unwrap(), 2);
}

#[test]
fn a_twin_dismissed_is_not_mentioned_again_from_either_note() {
    let mut ids = Vec::new();
    let (phone, _) = indexed(3, |phone| {
        for title in ["First", "Second", "Third"] {
            let body = if title == "Third" {
                "invoices receipts accountant"
            } else {
                ORCHARD
            };
            ids.push(note(&phone.library, title, body));
        }
    });
    let discovery = phone.library.discovery();
    let first = discovery.note_meaning(&ids[0]).unwrap();
    let twin = first.twin.expect("a twin");
    assert_eq!(twin.id, ids[1]);
    assert!(twin.percent >= 90);
    assert!(discovery.note_meaning(&ids[1]).unwrap().twin.is_some());
    discovery.dismiss_twin(&ids[1], &ids[0]).unwrap();
    assert!(discovery.note_meaning(&ids[0]).unwrap().twin.is_none());
    assert!(discovery.note_meaning(&ids[1]).unwrap().twin.is_none());
}

#[test]
fn tags_are_suggested_from_close_notes_without_the_notes_own() {
    let mut ids = Vec::new();
    let (phone, _) = indexed(3, |phone| {
        let store = store(phone);
        let fruit = tag(&store, "fruit");
        let garden = tag(&store, "garden");
        let book = book(&store, "Notes");
        for (title, tags) in [
            ("Mine", vec![&garden]),
            ("Close", vec![&fruit, &garden]),
            ("Also close", vec![&fruit]),
        ] {
            ids.push(raw_note(
                &store,
                json!({"title": title, "body": ORCHARD, "bookId": book, "tags": tags}),
            ));
        }
    });
    let meaning = phone.library.discovery().note_meaning(&ids[0]).unwrap();
    let names: Vec<&str> = meaning.tags.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["fruit"]);
    assert!(meaning.tags[0].weight > 1.);
}

#[test]
fn two_notes_merge_unless_one_is_locked() {
    let mut ids = Vec::new();
    let (phone, _) = indexed(3, |phone| {
        for (title, body) in [
            ("Keep", "kept words"),
            ("Other", "other words"),
            ("Sealed", "sealed words"),
        ] {
            ids.push(note(&phone.library, title, body));
        }
    });
    let library = &phone.library;
    let discovery = library.discovery();
    library
        .locks()
        .lock_note(&ids[2], LOCK_PASSWORD.into())
        .unwrap();
    assert_eq!(
        discovery.merge_notes(&ids[0], &ids[2]),
        Err(KansoError::Locked)
    );
    assert_eq!(
        discovery.merge_notes(&ids[2], &ids[0]),
        Err(KansoError::Locked)
    );
    assert_eq!(
        discovery.merge_notes(&ids[0], &ids[0]),
        Err(KansoError::Invalid {
            message: "A note can't be merged into itself.".into()
        })
    );
    let merged = discovery.merge_notes(&ids[0], &ids[1]).unwrap();
    let body = merged.body.unwrap();
    assert!(body.contains("kept words") && body.contains("other words"));
    // The other is in the trash now.
    assert_eq!(library.note(&ids[1]).unwrap().notebook_id, "trash");
    assert_eq!(
        discovery.merge_notes("note:missing", &ids[0]),
        Err(KansoError::NotFound)
    );
}

fn link(title: &str) -> String {
    format!("[[{title}]]")
}

#[test]
fn the_graph_leaves_out_locked_notes_while_the_lock_is_closed() {
    install();
    let phone = lone_phone();
    let library = &phone.library;
    let discovery = library.discovery();
    let hub = note(
        library,
        "Hub",
        &format!("{} {}", link("Secret"), link("Open")),
    );
    let open = note(library, "Open", &link("Hub"));
    let secret = note(library, "Secret", &link("Hub"));
    let tagged = {
        let store = store(&phone);
        let exclude = tag(&store, "ai-exclude");
        let book = store.books().unwrap()[0].id.clone();
        raw_note(
            &store,
            json!({"title": "Excluded", "body": link("Hub"), "bookId": book, "tags": [exclude]}),
        )
    };
    library
        .locks()
        .lock_note(&secret, LOCK_PASSWORD.into())
        .unwrap();
    let all = GraphRequest::Library {
        filter: GraphFilter::All,
    };

    // Open: every note is there.
    let graph = discovery.graph(&all).unwrap();
    let ids: Vec<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(graph.nodes.len(), 4);
    for id in [&hub, &open, &secret, &tagged] {
        assert!(ids.contains(&id.as_str()));
    }
    assert!(graph.link_count >= 3);

    // Closed: the locked note and its links are gone; the excluded stays.
    library.locks().lock().unwrap();
    let graph = discovery.graph(&all).unwrap();
    let ids: Vec<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(graph.nodes.len(), 3);
    assert!(!ids.contains(&secret.as_str()));
    assert!(ids.contains(&tagged.as_str()));
    for edge in &graph.edges {
        assert!((edge.from as usize) < graph.nodes.len() && (edge.to as usize) < graph.nodes.len());
    }
    let local = discovery
        .graph(&GraphRequest::Local {
            center: hub.clone(),
            hops: 9,
        })
        .unwrap();
    assert_eq!(local.nodes.iter().filter(|n| n.is_center).count(), 1);
    assert!(local.nodes.iter().all(|n| n.id != secret));
    // A hidden note has no graph of its own.
    let hidden = discovery
        .graph(&GraphRequest::Local {
            center: secret.clone(),
            hops: 1,
        })
        .unwrap();
    assert!(hidden.nodes.is_empty());

    // Opened again: it is back.
    assert_eq!(library.locks().unlock(LOCK_PASSWORD.into()), Ok(true));
    assert_eq!(discovery.graph(&all).unwrap().nodes.len(), 4);
}

#[test]
fn the_graph_is_laid_out_the_same_way_with_colors_for_both_themes() {
    install();
    let phone = lone_phone();
    let library = &phone.library;
    let discovery = library.discovery();
    let first = note(library, "First", "[[Second]]");
    note(library, "Second", "[[Third]]");
    note(library, "Third", "nothing");
    let all = GraphRequest::Library {
        filter: GraphFilter::All,
    };
    let a = discovery.graph(&all).unwrap();
    assert_eq!(a, discovery.graph(&all).unwrap());
    assert_eq!(a.link_count, 2);
    for node in &a.nodes {
        assert_eq!(node.color_light >> 24, 0xFF);
        assert_eq!(node.color_dark >> 24, 0xFF);
        assert_ne!(node.color_light, node.color_dark);
        assert!(node.radius > 0. && node.x.is_finite() && node.y.is_finite());
        assert!(!node.is_center);
    }
    // A notebook narrows it; one without notes has none.
    let notebook = a.nodes[0].notebook_id.clone();
    let narrowed = discovery
        .graph(&GraphRequest::Library {
            filter: GraphFilter::Book { id: notebook },
        })
        .unwrap();
    assert_eq!(narrowed.nodes.len(), 3);
    let empty = discovery
        .graph(&GraphRequest::Library {
            filter: GraphFilter::Tag {
                id: "tag:none".into(),
            },
        })
        .unwrap();
    assert!(empty.nodes.is_empty());
    // One hop reaches the next note; two reach the one after.
    let reach = |hops| {
        discovery
            .graph(&GraphRequest::Local {
                center: first.clone(),
                hops,
            })
            .unwrap()
            .nodes
            .len()
    };
    assert_eq!((reach(1), reach(2), reach(3)), (2, 3, 3));
    assert_eq!(reach(0), 2);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    /// However notes are linked, every edge of the graph joins two nodes
    /// it has, every node is placed, and the link count is the edges that
    /// join two different notes.
    #[test]
    fn prop_the_graph_is_whole(
        links in prop::collection::vec((0usize..6, 0usize..6), 0..10),
    ) {
        install();
        let phone = lone_phone();
        let library = &phone.library;
        let mut bodies = vec![String::new(); 6];
        for (from, to) in &links {
            bodies[*from].push_str(&format!("[[Note {to}]] "));
        }
        for (n, body) in bodies.iter().enumerate() {
            note(library, &format!("Note {n}"), body);
        }
        let graph = library
            .discovery()
            .graph(&GraphRequest::Library { filter: GraphFilter::All })
            .unwrap();
        prop_assert_eq!(graph.nodes.len(), 6);
        for edge in &graph.edges {
            prop_assert!(edge.from <= edge.to && (edge.to as usize) < 6);
        }
        let joins = graph.edges.iter().filter(|e| e.from != e.to).count();
        prop_assert_eq!(joins as u32, graph.link_count);
    }
}

fn png(store: &Store, bytes: &[u8]) -> String {
    store
        .put(
            DocType::File,
            json!({
                "name": "shot.png",
                "contentType": "image/png",
                "_attachments": {"index": {
                    "content_type": "image/png",
                    "data": base64::engine::general_purpose::STANDARD.encode(bytes),
                }},
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

#[test]
fn images_are_read_through_the_platform_and_only_those_a_readable_note_shows() {
    install();
    let phone = lone_phone();
    let library = &phone.library;
    let discovery = library.discovery();
    let (shown, hidden, orphan) = {
        let store = store(&phone);
        (
            png(&store, b"shown"),
            png(&store, b"hidden"),
            png(&store, b"orphan"),
        )
    };
    note(library, "Plain", &format!("![](inkdrop://{shown})"));
    let secret = note(library, "Secret", &format!("![](inkdrop://{hidden})"));
    library
        .locks()
        .lock_note(&secret, LOCK_PASSWORD.into())
        .unwrap();
    assert_eq!(discovery.ocr_waiting().unwrap(), 1);
    let image = discovery.ocr_next().unwrap().expect("an image");
    assert_eq!(
        (image.id.as_str(), image.bytes.as_slice()),
        (shown.as_str(), &b"shown"[..])
    );
    assert_ne!(image.id, orphan);
    // Words from other bytes are refused; the right ones are kept.
    assert!(
        !discovery
            .ocr_keep(&image.id, "wrong", "words", "vision")
            .unwrap()
    );
    assert!(
        discovery
            .ocr_keep(&image.id, &image.md5, " words in it ", "mlkit")
            .unwrap()
    );
    assert_eq!(
        store(&phone).image_text(&shown).unwrap().as_deref(),
        Some("words in it")
    );
    assert!(discovery.ocr_next().unwrap().is_none());
    assert_eq!(discovery.ocr_waiting().unwrap(), 0);
    // Unlocked or not, the locked note's image is never offered.
    library.locks().lock().unwrap();
    assert!(discovery.ocr_next().unwrap().is_none());
}

#[test]
fn the_query_rule_and_a_dismissal_are_the_engines() {
    assert_eq!(
        Discovery::meaning_query(" fruit trees "),
        Some("fruit trees".into())
    );
    assert_eq!(Discovery::meaning_query("ab"), None);
    assert_eq!(Discovery::MIN_QUERY_CHARS, 3);
}
