//! Search by the words inside a note's PDFs, end to end (issue #43, steps 5
//! and 7): a PDF is attached, the engine's queue reads it through the real
//! worker, and the store finds the note that links it. Locked notes' PDFs
//! are never read.
//!
//! Needs the PDFium library: `scripts/fetch-pdfium.sh`, then
//! `cargo test -p kanso-pdf --features render`.

#![cfg(all(feature = "render", any(target_os = "macos", target_os = "linux")))]

use kanso_core::{DocType, Query};
use kanso_engine::attach::{attach_pdf, file_markdown};
use kanso_engine::discovery::pdf_text::{PdfTextSource, index_next, waiting};
use kanso_engine::discovery::pdf_worker::WorkerText;
use kanso_pdf::fixtures::{encrypted_pdf, pages_pdf, report_pdf};
use kanso_store::{Origin, Put, SearchParams, Store};
use serde_json::json;

mod common;

use common::paths;

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("k.db")).unwrap();
    (dir, store)
}

fn source() -> WorkerText {
    WorkerText::new(paths())
}

fn book(store: &Store) -> String {
    store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id
}

/// Attaches `bytes` as a PDF and writes a note that links it.
fn note_with_pdf(store: &Store, book: &str, title: &str, bytes: &[u8]) -> (String, String) {
    let file = attach_pdf(store, bytes, "document.pdf", None).unwrap();
    let note = store
        .put(
            DocType::Note,
            json!({
                "title": title,
                "body": format!("Reading this.\n\n{}", file_markdown(&file, "document.pdf")),
                "bookId": book,
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    (file, note)
}

/// A PDF whose one page says `words`.
fn pdf_saying(words: &str) -> Vec<u8> {
    pages_pdf(&[format!("BT /F1 18 Tf 72 700 Td ({words}) Tj ET")])
}

fn find(store: &Store, text: &str) -> Vec<String> {
    store
        .search_summaries(&SearchParams {
            query: Query::parse(text),
            ocr: true,
            ..Default::default()
        })
        .unwrap()
        .into_iter()
        .map(|n| n.id)
        .collect()
}

fn lock(store: &Store, note: &str) {
    let mut sealed = store.get_api(note, None, false).unwrap();
    sealed["body"] = "".into();
    sealed["kansoLock"] =
        json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
    store.lock_note(sealed).unwrap();
}

/// Reads every PDF waiting, one pass at a time like the app's loop.
fn index_all(store: &Store, source: &dyn PdfTextSource) -> usize {
    let mut read = 0;
    while index_next(store, source).unwrap() > 0 {
        read += 1;
    }
    read
}

#[test]
fn a_note_is_found_by_a_word_inside_its_linked_pdf() {
    let (_dir, store) = store();
    let book = book(&store);
    let (file, report) = note_with_pdf(&store, &book, "Planning", &report_pdf());
    let (_, other) = note_with_pdf(
        &store,
        &book,
        "Other",
        &pdf_saying("Nothing about money here"),
    );
    let (_, plain) = note_with_pdf(&store, &book, "Plain", &pdf_saying("Sunrise over the lake"));

    // Attached but not read yet: the words aren't searchable.
    assert_eq!(waiting(&store).unwrap(), 3);
    assert!(find(&store, "renewal").is_empty());

    let source = source();
    assert_eq!(source.engine(), "pdfium");
    assert_eq!(index_all(&store, &source), 3);
    assert_eq!(waiting(&store).unwrap(), 0);

    // A word in the report's first page, and its heading, find its note
    // and no other; so do words on later pages.
    assert_eq!(find(&store, "renewal"), vec![report.clone()]);
    assert_eq!(find(&store, "quarterly report"), vec![report.clone()]);
    assert_eq!(find(&store, "sunrise"), vec![plain]);
    assert_eq!(find(&store, "money"), vec![other]);
    assert!(find(&store, "zanzibar").is_empty());

    let pages = store.pdf_pages(&file).unwrap();
    assert_eq!(pages.len(), 3);
    assert!(
        pages[0].contains("Revenue grew 12 percent"),
        "{:?}",
        pages[0]
    );
    assert_eq!(pages[1].trim(), "Page 2");

    // Read once: another pass finds nothing waiting.
    assert_eq!(index_next(&store, &source).unwrap(), 0);
}

#[test]
fn a_locked_notes_pdf_is_never_read_or_found() {
    let (_dir, store) = store();
    let book = book(&store);
    let (secret_file, secret) =
        note_with_pdf(&store, &book, "Secret", &pdf_saying("zanzibarquartz"));
    let (_, open) = note_with_pdf(&store, &book, "Open", &pdf_saying("marigoldtelescope"));
    lock(&store, &secret);

    let source = source();
    // Only the open note's PDF is offered, read and found.
    assert_eq!(waiting(&store).unwrap(), 1);
    assert_eq!(index_all(&store, &source), 1);
    assert!(find(&store, "zanzibarquartz").is_empty());
    assert_eq!(find(&store, "marigoldtelescope"), vec![open]);
    assert!(!store.pdf_indexed(&secret_file).unwrap());
    assert!(store.pdf_pages(&secret_file).unwrap().is_empty());

    // Not by a later pass either, nor by one that sees it again.
    assert_eq!(index_next(&store, &source).unwrap(), 0);
    assert!(find(&store, "zanzibarquartz").is_empty());
}

#[test]
fn locking_a_note_after_its_pdf_was_read_takes_the_words_out_of_search() {
    let (_dir, store) = store();
    let book = book(&store);
    let (file, note) = note_with_pdf(&store, &book, "Soon secret", &pdf_saying("quinceblossom"));
    let source = source();
    assert_eq!(index_all(&store, &source), 1);
    assert_eq!(find(&store, "quinceblossom"), vec![note.clone()]);

    lock(&store, &note);
    assert!(find(&store, "quinceblossom").is_empty());
    assert!(store.pdf_pages(&file).unwrap().is_empty());
    assert!(!store.pdf_indexed(&file).unwrap());
    assert_eq!(index_next(&store, &source).unwrap(), 0);
    assert!(find(&store, "quinceblossom").is_empty());
}

#[test]
fn a_pdf_the_worker_cannot_read_is_kept_as_having_no_text() {
    let (_dir, store) = store();
    let book = book(&store);
    // Encrypted: the worker is never given a password.
    let (locked_pdf, _) = note_with_pdf(&store, &book, "Encrypted", &encrypted_pdf("hunter2"));
    let (_, fine) = note_with_pdf(&store, &book, "Fine", &pdf_saying("lanternfish"));
    let source = source();
    assert_eq!(index_all(&store, &source), 2);
    assert!(store.pdf_indexed(&locked_pdf).unwrap());
    assert!(store.pdf_pages(&locked_pdf).unwrap().is_empty());
    assert_eq!(find(&store, "lanternfish"), vec![fine]);
    // Not tried again.
    assert_eq!(waiting(&store).unwrap(), 0);
}

/// The Settings switch's off state: `forget_all_pdf_text` takes every PDF's
/// words out of search, and turning it on again reads them again.
#[test]
fn turning_pdf_search_off_forgets_the_words_and_on_reads_them_again() {
    let (_dir, store) = store();
    let book = book(&store);
    let (_, note) = note_with_pdf(&store, &book, "Kept", &pdf_saying("peppermintglacier"));
    let source = source();
    assert_eq!(index_all(&store, &source), 1);
    assert_eq!(find(&store, "peppermintglacier"), vec![note.clone()]);

    store.forget_all_pdf_text().unwrap();
    assert!(find(&store, "peppermintglacier").is_empty());
    assert_eq!(waiting(&store).unwrap(), 1);

    assert_eq!(index_all(&store, &source), 1);
    assert_eq!(find(&store, "peppermintglacier"), vec![note]);
}
