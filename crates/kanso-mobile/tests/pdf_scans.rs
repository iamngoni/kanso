//! Scanned PDF pages on a phone (issue #43, step 8): the phone has no PDF
//! reader and reads no scans, but the words a computer read arrive with the
//! PDF. It searches them, and the PDF's highlights list says how many scanned
//! pages are readable; a locked note's scans are not part of either.

mod common;

use base64::Engine as _;
use common::*;
use kanso_core::DocType;
use kanso_mobile::SearchScope;
use kanso_pdf::model::Color;
use kanso_pdf::text::HighlightParts;
use kanso_pdf::{Annotations, OcrPage, Quad};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use serde_json::json;
use std::collections::BTreeMap;

const SEED: u64 = 0x0043_0008;

/// A PDF attachment.
fn scanned_pdf(store: &Store) -> String {
    store
        .put(
            DocType::File,
            json!({
                "name": "scan.pdf",
                "contentType": "application/pdf",
                "_attachments": {"index": {
                    "content_type": "application/pdf",
                    "data": base64::engine::general_purpose::STANDARD.encode("%PDF-scan"),
                }},
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

/// A note linking `file` with one highlight on page 1, in its block.
fn note_body(file: &str) -> String {
    let mut annots = Annotations::new(file, "m");
    let mut quads = BTreeMap::new();
    quads.insert(1, vec![Quad::new(0.1, 0.1, 0.5, 0.2)]);
    annots
        .add(
            HighlightParts {
                quads,
                text: "ledger".into(),
            },
            Color::Yellow,
            1,
        )
        .unwrap();
    let id = file.strip_prefix("file:").unwrap();
    let body = format!("[scan.pdf](inkdrop://file:{id})\n");
    annots.save_edit(&body).unwrap().unwrap().apply(&body)
}

/// Gives the file the words a computer read, as sync would.
fn arrive(store: &Store, file: &str, words: &[String]) {
    let md5 = store.get(file).unwrap().unwrap().fields["md5digest"]
        .as_str()
        .unwrap()
        .to_owned();
    let pages: Vec<OcrPage> = words
        .iter()
        .enumerate()
        .map(|(i, w)| {
            OcrPage::new(
                i as u32 + 1,
                612.0,
                792.0,
                w.split_whitespace()
                    .enumerate()
                    .map(|(j, w)| {
                        let x = 0.05 + j as f64 * 0.2;
                        (w.to_owned(), Quad::new(x, 0.1, x + 0.18, 0.14))
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let rev = store.get(file).unwrap().unwrap().rev;
    let ocr = kanso_pdf::PdfOcr {
        md5,
        engine: "vision".into(),
        at: 1,
        pages,
    };
    store
        .put(
            DocType::File,
            json!({"_id": file, "_rev": rev, "kansoPdfOcr": ocr.to_value()}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
}

fn phone_with_scan(words: &[String]) -> (Phone, String, String) {
    let server = std::sync::Arc::new(kanso_sync::MemoryServer::new());
    let mut ids = (String::new(), String::new());
    let phone = phone_with(&server, |store| {
        let file = scanned_pdf(store);
        arrive(store, &file, words);
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let note = store
            .put(
                DocType::Note,
                json!({"title": "Scans", "body": note_body(&file), "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        ids = (file, note);
    });
    (phone, ids.0, ids.1)
}

#[test]
fn a_phone_searches_words_a_computer_read_and_counts_the_pages() {
    let (phone, _file, note) = phone_with_scan(&[
        "general ledger".into(),
        String::new(),
        "appendix tables".into(),
    ]);
    for word in ["ledger", "APPENDIX"] {
        let hits = phone
            .library
            .search(word, SearchScope::default(), 0)
            .unwrap();
        assert_eq!(hits.len(), 1, "{word}");
        assert_eq!(hits[0].row.id, note);
    }
    let notes = phone.library.pdf_highlights(&note).unwrap();
    // Two of the three pages have words.
    assert_eq!(notes[0].scanned_pages, 2);
    assert_eq!(notes[0].highlights.len(), 1);
}

#[test]
fn a_pdf_nobody_read_has_no_scanned_pages() {
    let (phone, _file, note) = phone_with_scan(&[]);
    assert_eq!(
        phone.library.pdf_highlights(&note).unwrap()[0].scanned_pages,
        0
    );
}

/// A locked note offers nothing: not its scans' words in search, and not
/// its highlights list.
#[test]
fn a_locked_notes_scans_are_not_searchable() {
    let (phone, _file, note) = phone_with_scan(&["secret ledger".into()]);
    phone
        .library
        .locks()
        .lock_note(&note, LOCK_PASSWORD.into())
        .unwrap();
    phone.library.locks().lock().unwrap();
    assert!(
        phone
            .library
            .search("ledger", SearchScope::default(), 0)
            .unwrap()
            .is_empty()
    );
    assert!(phone.library.pdf_highlights(&note).is_err());
}

proptest! {
    #![proptest_config(ProptestConfig { rng_seed: RngSeed::Fixed(SEED), ..ProptestConfig::with_cases(12) })]

    /// However the words fall on the pages, the phone counts the pages that
    /// have some and finds every word.
    #[test]
    fn prop_every_read_page_is_counted_and_found(
        pages in prop::collection::vec(prop::collection::vec("[a-z]{7,10}", 0..3), 1..5),
    ) {
        let words: Vec<String> = pages.iter().map(|p| p.join(" ")).collect();
        let (phone, _file, note) = phone_with_scan(&words);
        let expected = pages.iter().filter(|p| !p.is_empty()).count() as u32;
        prop_assert_eq!(phone.library.pdf_highlights(&note).unwrap()[0].scanned_pages, expected);
        for w in pages.iter().flatten() {
            let hits = phone.library.search(w, SearchScope::default(), 0).unwrap();
            prop_assert_eq!(hits.len(), 1, "{}", w);
        }
    }
}
