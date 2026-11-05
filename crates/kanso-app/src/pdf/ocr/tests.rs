use super::*;
use crate::pdf::session::scanned_text;
use base64::Engine as _;
use kanso_core::DocType;
use kanso_pdf::{OcrPage, PageText, Quad, TextDoc};
use kanso_store::{Origin, Put};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use serde_json::json;

const SEED: u64 = 0x0043_0008;

#[test]
fn nothing_reads_without_a_worker_beside_the_app() {
    // Under test there is no worker beside the test binary.
    assert!(!supported());
}

#[test]
fn settings_say_where_reading_stands() {
    assert_eq!(
        words(&Status::Reading {
            read: 39,
            scanned: 300,
            waiting: 1
        })
        .as_deref(),
        Some("Reading 40 of 300 pages.")
    );
    assert_eq!(
        words(&Status::Reading {
            read: 0,
            scanned: 12,
            waiting: 3
        })
        .as_deref(),
        Some("Reading 1 of 12 pages; 2 PDFs to go.")
    );
    assert_eq!(
        words(&Status::Paused { waiting: 1 }).as_deref(),
        Some("Paused. 1 PDF to go.")
    );
    assert!(words(&Status::Idle).is_none() && words(&Status::Off).is_none());
}

proptest! {
    #![proptest_config(ProptestConfig { rng_seed: RngSeed::Fixed(SEED), ..ProptestConfig::with_cases(64) })]

    /// Whatever the counts, the words name a page that exists (never "41 of
    /// 40") and every state that has work says so.
    #[test]
    fn prop_the_words_never_pass_the_total(read in 0u32..5000, scanned in 0u32..5000, waiting in 1usize..50) {
        let said = words(&Status::Reading { read, scanned, waiting }).unwrap();
        if scanned > 0 {
            let at = said.strip_prefix("Reading ").unwrap();
            let (n, rest) = at.split_once(" of ").unwrap();
            let n: usize = n.replace(',', "").parse().unwrap();
            let total: usize = rest.split(' ').next().unwrap().replace(',', "").parse().unwrap();
            prop_assert!(n >= 1 && n <= total);
        }
        let (paused, failing) = (Status::Paused { waiting }, Status::Waiting { waiting });
        prop_assert!(words(&paused).is_some());
        prop_assert!(words(&failing).is_some());
    }
}

/// A scanned PDF whose pages 1 and 2 have no text, read: page 1 says
/// "alpha beta gamma", page 2 was not read.
fn scanned_pdf_with_words() -> (tempfile::TempDir, Store, String) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("k.db")).unwrap();
    let id = store
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
        .id;
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    store
        .put(
            DocType::Note,
            json!({"title": "Scans", "body": format!("[s](inkdrop://{id})"), "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    let md5 = store.get(&id).unwrap().unwrap().fields["md5digest"]
        .as_str()
        .unwrap()
        .to_owned();
    store
        .set_pdf_text(&id, &md5, &[String::new(), String::new()], "test")
        .unwrap();
    let page = OcrPage::new(
        1,
        612.0,
        792.0,
        [("alpha", 0.10), ("beta", 0.30), ("gamma", 0.50)]
            .map(|(w, x)| (w.to_owned(), Quad::new(x, 0.1, x + 0.18, 0.14))),
    );
    store.keep_pdf_ocr_page(&id, &md5, &page).unwrap();
    store
        .keep_pdf_ocr_page(&id, &md5, &OcrPage::new(2, 612.0, 792.0, []))
        .unwrap();
    assert!(store.finish_pdf_ocr(&id, &md5, "test").unwrap());
    (dir, store, id)
}

fn empty_page(page: u32) -> PageText {
    PageText {
        page,
        width: 595.0,
        height: 842.0,
        rotation: 90,
        chars: Vec::new(),
    }
}

/// The reader's text for a page PDFium found no characters on comes from the
/// words read from its picture, on the page's size, upright.
#[test]
fn a_scanned_page_gets_its_words_as_a_text_layer() {
    let (_dir, store, id) = scanned_pdf_with_words();
    let text = scanned_text(&store, &id, &empty_page(1)).expect("page 1 was read");
    assert_eq!((text.page, text.width, text.height), (1, 595.0, 842.0));
    assert_eq!(text.rotation, 0);
    let doc = TextDoc::new([&text]);
    let all = doc.select_all().unwrap();
    assert_eq!(doc.text(&all), "alpha beta gamma");
    // A double click on the second word selects it for a highlight.
    let beta = kanso_pdf::Point {
        page: 1,
        x: 0.39,
        y: 0.12,
    };
    let word = doc.word_at(doc.hit(beta).unwrap()).unwrap();
    let parts = doc.highlight_parts(&word);
    assert_eq!(parts.text, "beta");
    assert_eq!(parts.quads[&1].len(), 1);
}

#[test]
fn a_page_with_no_words_stays_without_a_text_layer() {
    let (_dir, store, id) = scanned_pdf_with_words();
    // Read, with no words.
    assert!(scanned_text(&store, &id, &empty_page(2)).is_none());
    // Not in the document at all.
    assert!(scanned_text(&store, &id, &empty_page(9)).is_none());
    assert!(scanned_text(&store, "file:nothing", &empty_page(1)).is_none());
}
