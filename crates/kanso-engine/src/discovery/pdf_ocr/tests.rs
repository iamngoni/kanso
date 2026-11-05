use super::*;
use base64::Engine as _;
use kanso_core::DocType;
use kanso_pdf::Quad;
use kanso_store::{Origin, Put, SearchParams};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use serde_json::json;
use std::cell::Cell;
use std::sync::Mutex;

const SEED: u64 = 0x0043_0008;

/// Draws a page as the bytes `page N`, and counts.
struct Scans {
    opened: Mutex<usize>,
    fail: Option<ExtractError>,
}

struct Doc;

impl ScanSource for Scans {
    fn open(&self, _pdf: &[u8]) -> Result<Box<dyn ScanDoc>, ExtractError> {
        *self.opened.lock().unwrap() += 1;
        match &self.fail {
            Some(err) => Err(err.clone()),
            None => Ok(Box::new(Doc)),
        }
    }
}

impl ScanDoc for Doc {
    fn size(&self, _page: u32) -> Option<[f32; 2]> {
        Some([612.0, 792.0])
    }
    fn render_png(&mut self, page: u32, width_px: u32) -> Result<Vec<u8>, ExtractError> {
        assert!((600..=4096).contains(&width_px));
        Ok(format!("page {page}").into_bytes())
    }
}

/// Reads "page N" as the words `scanword{N}` and `second{N}`.
struct Words;

impl WordReader for Words {
    fn engine(&self) -> &'static str {
        "fake"
    }
    fn read_words(&self, png: &[u8]) -> Result<Vec<(String, Quad)>, String> {
        let n: u32 = String::from_utf8_lossy(png)
            .trim_start_matches("page ")
            .parse()
            .map_err(|_| "not a page".to_owned())?;
        Ok(vec![
            (format!("scanword{n}"), Quad::new(0.1, 0.1, 0.4, 0.14)),
            (format!("second{n}"), Quad::new(0.42, 0.1, 0.7, 0.14)),
        ])
    }
}

fn scans() -> Scans {
    Scans {
        opened: Mutex::new(0),
        fail: None,
    }
}

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("k.db")).unwrap();
    (dir, store)
}

/// A PDF of `count` pages with no text, linked by a note; its text is read.
fn scanned_pdf(store: &Store, count: usize) -> (String, String) {
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
    let note = store
        .put(
            DocType::Note,
            json!({"title": "Scans", "body": format!("[s](inkdrop://{id})"), "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let md5 = store.get(&id).unwrap().unwrap().fields["md5digest"]
        .as_str()
        .unwrap()
        .to_owned();
    store
        .set_pdf_text(&id, &md5, &vec![String::new(); count], "test")
        .unwrap();
    (id, note)
}

fn found(store: &Store, word: &str) -> usize {
    store
        .search_summaries(&SearchParams {
            query: kanso_core::Query::parse(word),
            ocr: true,
            ..Default::default()
        })
        .unwrap()
        .len()
}

fn never() -> bool {
    false
}

#[test]
fn nothing_waits_without_scanned_pages() {
    let (_dir, store) = store();
    assert_eq!(
        ocr_next(&store, &scans(), &Words, 4, &never).unwrap(),
        Ocred::Nothing
    );
}

#[test]
fn pages_are_read_in_chunks_with_progress_and_then_written() {
    let (_dir, store) = store();
    let (id, _) = scanned_pdf(&store, 5);
    let source = scans();
    let step = |store: &Store| ocr_next(store, &source, &Words, 2, &never).unwrap();
    assert_eq!(
        step(&store),
        Ocred::Read {
            waiting: 1,
            read: 2,
            scanned: 5,
            finished: false
        }
    );
    assert_eq!(found(&store, "scanword1"), 0, "not searchable until done");
    assert_eq!(
        step(&store),
        Ocred::Read {
            waiting: 1,
            read: 4,
            scanned: 5,
            finished: false
        }
    );
    assert_eq!(
        step(&store),
        Ocred::Read {
            waiting: 1,
            read: 5,
            scanned: 5,
            finished: true
        }
    );
    assert_eq!(step(&store), Ocred::Nothing);
    for n in 1..=5 {
        assert_eq!(found(&store, &format!("scanword{n}")), 1);
    }
    let ocr = store.pdf_ocr(&id).unwrap().unwrap();
    assert_eq!(ocr.engine, "fake");
    assert_eq!(ocr.page(3).unwrap().text(), "scanword3 second3");
}

#[test]
fn a_pause_stops_between_pages_and_reading_goes_on() {
    let (_dir, store) = store();
    let (id, _) = scanned_pdf(&store, 4);
    let source = scans();
    // Stop after one page of the chunk.
    let asked = Cell::new(0);
    let stop = || {
        asked.set(asked.get() + 1);
        asked.get() > 3
    };
    let first = ocr_next(&store, &source, &Words, 4, &stop).unwrap();
    assert!(matches!(
        first,
        Ocred::Read {
            read: 2,
            finished: false,
            ..
        }
    ));
    // Paused from the start: nothing is read.
    assert_eq!(
        ocr_next(&store, &source, &Words, 4, &|| true).unwrap(),
        Ocred::Stopped { waiting: 1 }
    );
    assert_eq!(store.pdf_ocr_progress(&id).unwrap().read, 2);
    let rest = ocr_next(&store, &source, &Words, 4, &never).unwrap();
    assert!(matches!(
        rest,
        Ocred::Read {
            read: 4,
            finished: true,
            ..
        }
    ));
    assert_eq!(found(&store, "second4"), 1);
}

#[test]
fn a_failed_reader_keeps_what_was_read_and_tries_again() {
    let (_dir, store) = store();
    let (id, _) = scanned_pdf(&store, 2);
    let broken = Scans {
        opened: Mutex::new(0),
        fail: Some(ExtractError::Reader("no worker".into())),
    };
    assert_eq!(
        ocr_next(&store, &broken, &Words, 4, &never).unwrap(),
        Ocred::ReaderFailed {
            waiting: 1,
            error: "no worker".into()
        }
    );
    assert!(store.pdf_ocr(&id).unwrap().is_none());
    assert_eq!(store.pdfs_to_ocr(5).unwrap().len(), 1);
    let working = scans();
    assert!(matches!(
        ocr_next(&store, &working, &Words, 4, &never).unwrap(),
        Ocred::Read { finished: true, .. }
    ));
}

#[test]
fn a_pdf_that_cannot_be_opened_is_not_tried_forever() {
    let (_dir, store) = store();
    let (id, _) = scanned_pdf(&store, 3);
    let damaged = Scans {
        opened: Mutex::new(0),
        fail: Some(ExtractError::Document("damaged".into())),
    };
    assert!(matches!(
        ocr_next(&store, &damaged, &Words, 4, &never).unwrap(),
        Ocred::Read { finished: true, .. }
    ));
    assert_eq!(
        ocr_next(&store, &damaged, &Words, 4, &never).unwrap(),
        Ocred::Nothing
    );
    assert!(store.pdf_ocr(&id).unwrap().is_some());
    assert_eq!(*damaged.opened.lock().unwrap(), 1);
}

#[test]
fn a_reader_that_fails_on_a_page_leaves_it_empty() {
    struct Bad;
    impl WordReader for Bad {
        fn engine(&self) -> &'static str {
            "fake"
        }
        fn read_words(&self, _: &[u8]) -> Result<Vec<(String, Quad)>, String> {
            Err("vision failed".into())
        }
    }
    let (_dir, store) = store();
    let (id, _) = scanned_pdf(&store, 2);
    assert!(matches!(
        ocr_next(&store, &scans(), &Bad, 4, &never).unwrap(),
        Ocred::Read { finished: true, .. }
    ));
    let ocr = store.pdf_ocr(&id).unwrap().unwrap();
    assert!(ocr.pages.iter().all(|p| p.words.is_empty()));
    assert!(store.pdfs_to_ocr(5).unwrap().is_empty());
}

proptest! {
    #![proptest_config(ProptestConfig { rng_seed: RngSeed::Fixed(SEED), ..ProptestConfig::with_cases(24) })]

    /// However big the chunks and wherever the pauses fall, every scanned
    /// page ends up read exactly once, the count only goes up, and every
    /// page's words are found.
    #[test]
    fn prop_any_pauses_and_chunks_read_every_page_once(
        pages in 1usize..9,
        chunk in 1usize..5,
        stops in prop::collection::vec(any::<bool>(), 0..20),
    ) {
        let (_dir, store) = store();
        let (id, _) = scanned_pdf(&store, pages);
        let source = scans();
        let calls = Cell::new(0usize);
        let stop = || {
            let i = calls.get();
            calls.set(i + 1);
            stops.get(i).copied().unwrap_or(false)
        };
        let mut last = 0;
        for _ in 0..200 {
            match ocr_next(&store, &source, &Words, chunk, &stop).unwrap() {
                Ocred::Nothing => break,
                Ocred::Read { read, scanned, .. } => {
                    prop_assert_eq!(scanned as usize, pages);
                    prop_assert!(read >= last || read == 0);
                    last = read;
                }
                Ocred::Stopped { .. } => {}
                Ocred::ReaderFailed { .. } => prop_assert!(false),
            }
        }
        let ocr = store.pdf_ocr(&id).unwrap().unwrap();
        prop_assert_eq!(ocr.pages.len(), pages);
        for n in 1..=pages as u32 {
            prop_assert_eq!(ocr.page(n).unwrap().words.len(), 2);
            prop_assert_eq!(found(&store, &format!("scanword{n}")), 1);
        }
        prop_assert!(store.pdfs_to_ocr(5).unwrap().is_empty());
    }
}
