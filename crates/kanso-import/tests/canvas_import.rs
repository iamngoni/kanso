//! Property tests for importing canvas files from a vault: whatever a canvas
//! file holds, what is imported is a canvas the store accepts, and a card's
//! path becomes the same ID every time it is written, however it is spelled.
//! CI pins the seed with `PROPTEST_RNG_SEED`.

use kanso_core::DocType;
use kanso_core::canvas::{self, Canvas};
use kanso_import::library::{SourceKind, import_source, stable_id};
use kanso_store::{SearchParams, Store};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Value, json};

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// The vault's name: the folder it is in.
const VAULT: &str = "vault";

/// What is in the vault: notes, then files.
const NOTES: &[&str] = &["a.md", "b/c.md"];
const FILES: &[&str] = &["pic.png", "docs/x.pdf"];
const MISSING: &str = "gone/nothing.md";

/// How a card may spell a path.
fn spelled(path: &str, how: u8) -> String {
    match how {
        0 => path.to_owned(),
        1 => path.strip_suffix(".md").unwrap_or(path).to_owned(),
        2 => path.to_uppercase().replace(".MD", ".md"),
        _ => format!("./{path}"),
    }
}

/// A canvas file as a vault might hold it, with the paths its file cards
/// name, and some junk.
fn arb_canvas_file() -> impl Strategy<Value = (String, Vec<(String, u8)>)> {
    let paths: Vec<&'static str> = NOTES
        .iter()
        .chain(FILES)
        .copied()
        .chain([MISSING])
        .collect();
    (
        prop::collection::vec((prop::sample::select(paths), 0u8..4), 0..10),
        prop::collection::vec(
            prop::sample::select(vec!["text", "link", "group", "ink", "x"]),
            0..4,
        ),
        prop::collection::vec((0usize..14, 0usize..14), 0..8),
        any::<bool>(),
    )
        .prop_map(|(files, others, arrows, junk)| {
            let mut nodes: Vec<Value> = Vec::new();
            let mut spellings = Vec::new();
            for (i, (path, how)) in files.iter().enumerate() {
                let file = spelled(path, *how);
                // Capitalised spellings only match on a case-insensitive
                // name, which the vault's names are (all lower case).
                spellings.push(((*path).to_owned(), *how));
                nodes.push(json!({
                    "id": format!("f{i}"), "type": "file", "file": file,
                    "x": i * 10, "y": 0, "width": 100, "height": 50
                }));
            }
            for (i, kind) in others.iter().enumerate() {
                nodes.push(json!({
                    "id": format!("o{i}"), "type": kind, "text": "t", "url": "https://example.com",
                    "label": "l", "x": 0, "y": i * 10, "width": 40, "height": 40
                }));
            }
            let count = nodes.len().max(1);
            let edges: Vec<Value> = arrows
                .iter()
                .enumerate()
                .map(|(i, (a, b))| {
                    let id = |n: usize| match nodes.get(n % count) {
                        Some(node) => node["id"].clone(),
                        None => json!("nowhere"),
                    };
                    json!({"id": format!("e{i}"), "fromNode": id(*a), "toNode": id(*b)})
                })
                .collect();
            let mut doc = json!({"nodes": nodes, "edges": edges});
            if junk {
                doc["nodes"].as_array_mut().unwrap().push(json!("junk"));
                doc["edges"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"fromNode": 5}));
            }
            (doc.to_string(), spellings)
        })
}

fn write(root: &std::path::Path, path: &str, bytes: &[u8]) {
    let full = root.join(path);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(full, bytes).unwrap();
}

proptest! {
    #![proptest_config(config(24))]

    /// Invariants: a canvas file in a vault always imports as a canvas note
    /// the store accepted (nothing rejected), whatever it holds; every file
    /// card that names something in the vault points at the ID the import
    /// gave it, however the path was spelled, and one that names nothing is
    /// left as written; importing again changes nothing.
    #[test]
    fn prop_a_vault_canvas_imports_and_its_paths_map_consistently(
        (file, spellings) in arb_canvas_file()
    ) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(VAULT);
        for note in NOTES {
            write(&root, note, b"A note.\n");
        }
        write(&root, FILES[0], b"\x89PNG\r\n\x1a\nfixture");
        write(&root, FILES[1], b"%PDF-1.4 fixture");
        write(&root, "Board.canvas", file.as_bytes());
        let store = Store::open(dir.path().join("kanso.db")).unwrap();

        let report = import_source(SourceKind::Obsidian, &root, &store, |_| {}).unwrap();
        prop_assert!(report.rejected.is_empty(), "{report}");

        let boards = store
            .search(&SearchParams::default())
            .unwrap()
            .into_iter()
            .filter(|d| d.str_field("doctype") == Some("canvas"))
            .collect::<Vec<_>>();
        // A file that isn't JSON at all is the only way to import no canvas.
        prop_assert_eq!(boards.len(), 1, "{}", report);
        let canvas = canvas::parse(boards[0].str_field("body").unwrap()).unwrap();

        let file_cards: Vec<_> = canvas
            .nodes
            .iter()
            .filter(|n| n.id.starts_with('f') && n.kind == "file")
            .collect();
        prop_assert_eq!(file_cards.len(), spellings.len());
        for (card, (path, _)) in file_cards.iter().zip(&spellings) {
            let expected = if NOTES.contains(&path.as_str()) {
                Some(stable_id(DocType::Note, SourceKind::Obsidian, VAULT, path))
            } else if FILES.contains(&path.as_str()) {
                Some(stable_id(DocType::File, SourceKind::Obsidian, VAULT, &format!("file:{path}")))
            } else {
                None
            };
            match expected {
                Some(id) => {
                    prop_assert_eq!(card.file.as_deref(), Some(id.as_str()), "{:?}", card);
                    prop_assert!(store.get(&id).unwrap().is_some());
                }
                None => {
                    let file = card.file.as_deref().unwrap();
                    prop_assert!(
                        Canvas::note_id(card).is_none() && Canvas::attachment_id(card).is_none(),
                        "{file}"
                    );
                }
            }
        }

        let again = import_source(SourceKind::Obsidian, &root, &store, |_| {}).unwrap();
        prop_assert_eq!(again.notes.written(), 0, "{}", again);
        prop_assert_eq!(again.attachments.written(), 0);
    }
}
