//! Apple Notes exported as files: the Files route on a phone.

use kanso_import::library::{SourceKind, import_source, kinds_for, preview};
use kanso_store::Store;

fn export() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("Work")).unwrap();
    std::fs::write(root.join("Groceries.txt"), "Groceries\n☐ milk\n☑ eggs").unwrap();
    std::fs::write(
        root.join("Work/Plan.html"),
        "<html><body><h1>Plan</h1><ul><li><input type=\"checkbox\" checked> ship</li>\
         <li><input type=\"checkbox\"> test</li></ul></body></html>",
    )
    .unwrap();
    std::fs::write(
        root.join("Work/Meeting.rtf"),
        br"{\rtf1\ansi Meeting\par Agenda\par \bullet\tab one\par}",
    )
    .unwrap();
    std::fs::write(root.join("Work/Scan 1.pdf"), b"%PDF-1.4 fake").unwrap();
    std::fs::write(root.join(".DS_Store"), b"junk").unwrap();
    dir
}

/// Every note's title and body, sorted.
fn notes(store: &Store) -> Vec<(String, String)> {
    let books: Vec<String> = store.books().unwrap().into_iter().map(|b| b.id).collect();
    let mut found: Vec<(String, String)> = store
        .notes_in_books(&books)
        .unwrap()
        .into_iter()
        .map(|n| (n.title, n.body))
        .collect();
    found.sort();
    found
}

fn books(store: &Store) -> Vec<(String, Option<String>)> {
    let all = store.books().unwrap();
    let mut named: Vec<(String, Option<String>)> = all
        .iter()
        .map(|b| {
            let parent = b
                .parent_book_id
                .as_ref()
                .and_then(|p| all.iter().find(|x| &x.id == p))
                .map(|p| p.name.clone());
            (b.name.clone(), parent)
        })
        .collect();
    named.sort();
    named
}

#[test]
fn an_export_lands_under_an_apple_notes_notebook() {
    let input = export();
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("db")).unwrap();
    let first = import_source(SourceKind::AppleNotesFiles, input.path(), &store, |_| {}).unwrap();
    assert_eq!(first.source, "Apple Notes");
    assert_eq!(first.found.notes, 4);
    // Apple Notes, and Work inside it.
    assert_eq!(first.found.notebooks, 2);
    assert_eq!(first.attachments.created, 1, "the PDF is attached: {first}");

    let books = books(&store);
    assert!(books.contains(&("Apple Notes".into(), None)), "{books:?}");
    assert!(
        books.contains(&("Work".into(), Some("Apple Notes".into()))),
        "{books:?}"
    );

    let notes = notes(&store);
    let body = |title: &str| {
        notes
            .iter()
            .find(|(t, _)| t == title)
            .map(|(_, b)| b.clone())
            .unwrap_or_else(|| panic!("no note {title}: {notes:?}"))
    };
    assert_eq!(body("Groceries"), "- [ ] milk\n- [x] eggs");
    assert_eq!(body("Plan"), "- [x] ship\n- [ ] test");
    assert_eq!(body("Meeting"), "Agenda\n\n- one");
    assert!(body("Scan 1").contains("Scan 1.pdf"));
}

#[test]
fn importing_again_changes_nothing() {
    let input = export();
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("db")).unwrap();
    import_source(SourceKind::AppleNotesFiles, input.path(), &store, |_| {}).unwrap();
    let before = notes(&store);
    let again = import_source(SourceKind::AppleNotesFiles, input.path(), &store, |_| {}).unwrap();
    assert_eq!(again.notes.unchanged, 4);
    assert_eq!(again.notes.created, 0);
    assert_eq!(again.notebooks.created, 0);
    assert_eq!(notes(&store), before);
}

#[test]
fn a_preview_writes_nothing() {
    let input = export();
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("db")).unwrap();
    let report = preview(SourceKind::AppleNotesFiles, input.path(), &store, |_| {}).unwrap();
    assert!(report.dry_run);
    assert_eq!(report.found.notes, 4);
    assert!(notes(&store).is_empty());
}

#[test]
fn an_image_only_html_note_says_drawing() {
    let input = tempfile::tempdir().unwrap();
    std::fs::write(input.path().join("Sketch.html"), "<img src=\"sketch.png\">").unwrap();
    std::fs::write(input.path().join("sketch.png"), [1, 2, 3]).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("db")).unwrap();
    import_source(SourceKind::AppleNotesFiles, input.path(), &store, |_| {}).unwrap();
    let notes = notes(&store);
    assert_eq!(notes.len(), 1);
    assert!(notes[0].1.starts_with("(drawing)"), "{notes:?}");
}

#[test]
fn an_rtf_or_pdf_folder_is_offered_as_apple_notes() {
    let input = export();
    assert!(kinds_for(input.path()).contains(&SourceKind::AppleNotesFiles));
    assert_eq!(
        kinds_for(&input.path().join("Work/Meeting.rtf")),
        [SourceKind::AppleNotesFiles]
    );
}

/// A zip whose entries climb out (`../`, absolute paths) only ever names
/// notes inside the import, and an entry that inflates past the note limit
/// is skipped unread.
#[test]
fn a_hostile_zip_stays_inside_and_skips_a_bomb() {
    use std::io::Write;

    let dir = tempfile::tempdir().unwrap();
    let zip_path = dir.path().join("export.zip");
    {
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&zip_path).unwrap());
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, body) in [
            ("../escape.txt", "climbs"),
            ("/abs/rooted.txt", "rooted"),
            ("Work/Fine.txt", "fine"),
        ] {
            zip.start_file(name, options).unwrap();
            zip.write_all(body.as_bytes()).unwrap();
        }
        zip.start_file("Bomb.txt", options).unwrap();
        let zeros = vec![b'a'; 1024 * 1024];
        for _ in 0..9 {
            zip.write_all(&zeros).unwrap();
        }
        zip.finish().unwrap();
    }
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("db")).unwrap();
    let report = import_source(SourceKind::AppleNotesFiles, &zip_path, &store, |_| {}).unwrap();
    assert!(
        report.skipped.iter().any(|s| s.path == "Bomb.txt"),
        "{report}"
    );
    assert!(!dir.path().parent().unwrap().join("escape.txt").exists());
    let notes = notes(&store);
    assert!(notes.iter().any(|(t, _)| t == "Fine"), "{notes:?}");
    assert!(notes.iter().all(|(t, _)| t != "Bomb"), "{notes:?}");
    // Every notebook sits under Apple Notes, none named after `..` or `/`.
    for (name, parent) in books(&store) {
        assert!(
            !name.is_empty() && name != ".." && !name.contains('/'),
            "{name}"
        );
        assert!(name == "Apple Notes" || parent.is_some(), "{name}");
    }
}

/// Importing again after the Apple Notes notebook was locked writes no
/// plain text into it: changed and new notes are refused and reported.
#[test]
fn a_locked_apple_notes_notebook_gets_no_plain_text() {
    use kanso_core::DocType;
    use kanso_store::{Origin, Put};

    let input = export();
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("db")).unwrap();
    import_source(SourceKind::AppleNotesFiles, input.path(), &store, |_| {}).unwrap();
    let book = store
        .books()
        .unwrap()
        .into_iter()
        .find(|b| b.name == "Apple Notes")
        .unwrap();
    let mut doc = store.get_api(&book.id, None, false).unwrap();
    doc.as_object_mut().unwrap().remove("count");
    doc["kansoLock"] =
        serde_json::json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
    store
        .put(DocType::Book, doc, Put::Upsert, Origin::Sync)
        .unwrap();

    std::fs::write(
        input.path().join("Groceries.txt"),
        "Groceries\nsecret change",
    )
    .unwrap();
    std::fs::write(input.path().join("New.txt"), "brand new secret").unwrap();
    let again = import_source(SourceKind::AppleNotesFiles, input.path(), &store, |_| {}).unwrap();
    assert_eq!(again.notes.created, 0, "{again}");
    assert_eq!(again.notes.updated, 0, "{again}");
    assert!(!again.rejected.is_empty(), "{again}");
    let books: Vec<String> = store.books().unwrap().into_iter().map(|b| b.id).collect();
    for note in store.notes_in_books(&books).unwrap() {
        assert!(!note.body.contains("secret"), "{note:?}");
    }
}
