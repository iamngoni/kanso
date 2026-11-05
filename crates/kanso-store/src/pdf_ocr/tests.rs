use super::*;
use crate::SearchParams;
use base64::Engine as _;
use kanso_pdf::Quad;
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

const SEED: u64 = 0x0043_0008;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn pdf(store: &Store, tag: &str) -> String {
    store
        .put(
            DocType::File,
            json!({
                "name": "scan.pdf",
                "contentType": "application/pdf",
                "_attachments": {"index": {
                    "content_type": "application/pdf",
                    "data": base64::engine::general_purpose::STANDARD.encode(format!("%PDF-1.7 {tag}")),
                }},
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn note(store: &Store, file: &str) -> String {
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    store
        .put(
            DocType::Note,
            json!({"title": "Scans", "body": format!("[scan.pdf](inkdrop://{file})"), "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn md5_of(store: &Store, id: &str) -> String {
    store.get(id).unwrap().unwrap().fields["md5digest"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn find(store: &Store, text: &str) -> Vec<String> {
    store
        .search_summaries(&SearchParams {
            query: kanso_core::Query::parse(text),
            ocr: true,
            ..Default::default()
        })
        .unwrap()
        .into_iter()
        .map(|n| n.id)
        .collect()
}

fn page(n: u32, words: &[&str]) -> OcrPage {
    OcrPage::new(
        n,
        612.0,
        792.0,
        words.iter().enumerate().map(|(i, w)| {
            let x = 0.05 + i as f64 * 0.12;
            (w.to_string(), Quad::new(x, 0.1, x + 0.1, 0.14))
        }),
    )
}

fn lock(store: &Store, note: &str) {
    let mut sealed = store.get_api(note, None, false).unwrap();
    sealed["body"] = "".into();
    sealed["kansoLock"] =
        json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
    store.lock_note(sealed).unwrap();
}

/// A scanned PDF: pages 2 and 3 have no text, page 1 does.
fn scanned(store: &Store, tag: &str) -> (String, String, String) {
    let file = pdf(store, tag);
    let shows = note(store, &file);
    let md5 = md5_of(store, &file);
    assert!(
        store
            .set_pdf_text(
                &file,
                &md5,
                &["Typed cover page".into(), "".into(), " \n".into()],
                "test"
            )
            .unwrap()
    );
    (file, shows, md5)
}

#[test]
fn scanned_pages_are_read_kept_synced_and_searchable() {
    let (_dir, store) = open();
    let (file, shows, md5) = scanned(&store, "a");
    assert_eq!(
        store.pdfs_to_ocr(10).unwrap(),
        vec![PdfToOcr {
            id: file.clone(),
            md5: md5.clone()
        }]
    );
    assert_eq!(store.pdf_pages_to_ocr(&file, &md5).unwrap(), vec![2, 3]);
    assert_eq!(
        store.pdf_ocr_progress(&file).unwrap(),
        OcrProgress {
            scanned: 2,
            read: 0
        }
    );
    // Nothing is searchable on the scans yet.
    assert!(find(&store, "ledger").is_empty());

    assert!(
        store
            .keep_pdf_ocr_page(&file, &md5, &page(2, &["general", "ledger"]))
            .unwrap()
    );
    assert_eq!(store.pdf_pages_to_ocr(&file, &md5).unwrap(), vec![3]);
    assert_eq!(store.pdf_ocr_progress(&file).unwrap().read, 1);
    // Kept pages are not in the document, or in search, until the end.
    assert!(store.pdf_ocr(&file).unwrap().is_none());
    assert!(find(&store, "ledger").is_empty());

    assert!(
        store
            .keep_pdf_ocr_page(&file, &md5, &page(3, &["appendix"]))
            .unwrap()
    );
    assert!(store.finish_pdf_ocr(&file, &md5, "vision").unwrap());
    assert!(store.pdfs_to_ocr(10).unwrap().is_empty());
    assert_eq!(find(&store, "LEDGER"), vec![shows.clone()]);
    assert_eq!(find(&store, "appendix typed"), vec![shows]);
    let ocr = store.pdf_ocr(&file).unwrap().unwrap();
    assert_eq!(ocr.engine, "vision");
    assert_eq!(ocr.pages.len(), 2);
    assert_eq!(ocr.page(2).unwrap().text(), "general ledger");
    // It is in the file's document, to sync.
    let doc = store.get(&file).unwrap().unwrap();
    assert_eq!(doc.fields[PDF_OCR_FIELD]["md5"], json!(md5));
    // The typed page keeps its own text.
    assert_eq!(store.pdf_pages(&file).unwrap()[0], "Typed cover page");
    // The pending rows are gone.
    assert_eq!(store.pdf_ocr_progress(&file).unwrap().read, 0);
}

#[test]
fn reading_goes_on_where_it_stopped() {
    let (dir, store) = open();
    let (file, _shows, md5) = scanned(&store, "b");
    store
        .keep_pdf_ocr_page(&file, &md5, &page(2, &["first"]))
        .unwrap();
    drop(store);
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    assert_eq!(store.pdfs_to_ocr(10).unwrap().len(), 1);
    assert_eq!(store.pdf_pages_to_ocr(&file, &md5).unwrap(), vec![3]);
}

#[test]
fn a_scan_with_no_words_is_not_read_again() {
    let (_dir, store) = open();
    let (file, _shows, md5) = scanned(&store, "blank");
    store.keep_pdf_ocr_page(&file, &md5, &page(2, &[])).unwrap();
    store.keep_pdf_ocr_page(&file, &md5, &page(3, &[])).unwrap();
    assert!(store.finish_pdf_ocr(&file, &md5, "vision").unwrap());
    assert!(store.pdfs_to_ocr(10).unwrap().is_empty());
}

#[test]
fn new_bytes_drop_the_words_and_the_pages_kept_so_far() {
    let (_dir, store) = open();
    let (file, shows, md5) = scanned(&store, "c");
    store
        .keep_pdf_ocr_page(&file, &md5, &page(2, &["stale"]))
        .unwrap();
    store
        .keep_pdf_ocr_page(&file, &md5, &page(3, &["words"]))
        .unwrap();
    store.finish_pdf_ocr(&file, &md5, "vision").unwrap();
    assert_eq!(find(&store, "stale"), vec![shows]);
    store
        .keep_pdf_ocr_page(&file, &md5, &page(2, &["half"]))
        .ok();
    let rev = store.get(&file).unwrap().unwrap().rev;
    store
        .put(
            DocType::File,
            json!({"_id": file, "_rev": rev, "_attachments": {"index": {
                "content_type": "application/pdf",
                "data": base64::engine::general_purpose::STANDARD.encode(b"%PDF-1.7 other"),
            }}}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert!(find(&store, "stale").is_empty());
    assert!(store.pdf_ocr(&file).unwrap().is_none());
    assert_eq!(store.pdf_ocr_progress(&file).unwrap().read, 0);
    // Words read from the old bytes are refused.
    assert!(
        !store
            .keep_pdf_ocr_page(&file, &md5, &page(2, &["late"]))
            .unwrap()
    );
    assert!(!store.finish_pdf_ocr(&file, &md5, "vision").unwrap());
}

/// A device that can't read scans gets the words with the file: it finds
/// them, and has them to select, though it never read a page itself.
#[test]
fn words_that_arrive_with_the_file_are_found_without_reading() {
    let (_dir, store) = open();
    let file = pdf(&store, "d");
    let shows = note(&store, &file);
    let md5 = md5_of(&store, &file);
    let ocr = PdfOcr {
        md5: md5.clone(),
        engine: "vision".into(),
        at: 1,
        pages: vec![page(1, &["remote", "words"])],
    };
    let rev = store.get(&file).unwrap().unwrap().rev;
    store
        .put(
            DocType::File,
            json!({"_id": file, "_rev": rev, PDF_OCR_FIELD: ocr.to_value()}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(find(&store, "remote"), vec![shows.clone()]);
    assert_eq!(store.pdf_ocr(&file).unwrap(), Some(ocr));
    // Words for other bytes are ignored.
    let rev = store.get(&file).unwrap().unwrap().rev;
    store
        .put(
            DocType::File,
            json!({"_id": file, "_rev": rev, PDF_OCR_FIELD: {"md5": "other", "pages": [page(1, &["bogus"])]}}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert!(find(&store, "bogus").is_empty());
    assert!(find(&store, "remote").is_empty());
    assert!(store.pdf_ocr(&file).unwrap().is_none());
}

#[test]
fn a_pdf_only_a_locked_note_links_is_not_read_or_kept() {
    let (_dir, store) = open();
    let (file, shows, md5) = scanned(&store, "e");
    store
        .keep_pdf_ocr_page(&file, &md5, &page(2, &["secret"]))
        .unwrap();
    lock(&store, &shows);
    assert!(store.pdfs_to_ocr(10).unwrap().is_empty());
    assert_eq!(store.pdf_ocr_progress(&file).unwrap().read, 0);
    assert!(
        !store
            .keep_pdf_ocr_page(&file, &md5, &page(3, &["secret"]))
            .unwrap()
    );
    assert!(!store.finish_pdf_ocr(&file, &md5, "vision").unwrap());
    assert!(find(&store, "secret").is_empty());
}

fn strs(v: &[String]) -> Vec<&str> {
    v.iter().map(String::as_str).collect()
}

fn word() -> impl Strategy<Value = String> {
    "[a-z]{6,10}"
}

/// For each of up to six pages: its typed text (if any) and the words read
/// from its picture.
fn pages_strategy() -> impl Strategy<Value = Vec<(Option<String>, Vec<String>)>> {
    prop::collection::vec(
        (
            prop::option::of(word()),
            prop::collection::vec(word(), 0..4),
        ),
        1..6,
    )
}

proptest! {
    #![proptest_config(ProptestConfig { rng_seed: RngSeed::Fixed(SEED), ..ProptestConfig::with_cases(24) })]

    /// Every word read from a blank page is found by search once the
    /// document is finished; a page with typed text keeps it, and the words
    /// read from it, which were never asked for, change nothing.
    #[test]
    fn prop_scans_are_searchable_and_typed_pages_win(pages in pages_strategy()) {
        let (_dir, store) = open();
        let file = pdf(&store, "p");
        let shows = note(&store, &file);
        let md5 = md5_of(&store, &file);
        let typed: Vec<String> = pages.iter().map(|(t, _)| t.clone().unwrap_or_default()).collect();
        store.set_pdf_text(&file, &md5, &typed, "test").unwrap();
        let blank: Vec<u32> = pages.iter().enumerate()
            .filter(|(_, (t, _))| t.is_none()).map(|(i, _)| i as u32 + 1).collect();
        prop_assert_eq!(store.pdf_pages_to_ocr(&file, &md5).unwrap(), blank.clone());
        prop_assert_eq!(store.pdfs_to_ocr(5).unwrap().len(), usize::from(!blank.is_empty()));
        for n in &blank {
            let words: Vec<&str> = pages[*n as usize - 1].1.iter().map(String::as_str).collect();
            prop_assert!(store.keep_pdf_ocr_page(&file, &md5, &page(*n, &words)).unwrap());
        }
        if !blank.is_empty() {
            prop_assert!(store.finish_pdf_ocr(&file, &md5, "test").unwrap());
        }
        prop_assert!(store.pdfs_to_ocr(5).unwrap().is_empty());
        for n in &blank {
            for w in &pages[*n as usize - 1].1 {
                prop_assert_eq!(find(&store, w), vec![shows.clone()], "{}", w);
            }
        }
        for (i, (t, _)) in pages.iter().enumerate() {
            if let Some(t) = t {
                prop_assert_eq!(&store.pdf_pages(&file).unwrap()[i], t);
                prop_assert_eq!(find(&store, t), vec![shows.clone()]);
            }
        }
    }

    /// Reading a page twice keeps the later words only, and finishing twice
    /// writes nothing new.
    #[test]
    fn prop_rereading_a_page_replaces_it(first in prop::collection::vec(word(), 1..4), second in prop::collection::vec(word(), 1..4)) {
        let (_dir, store) = open();
        let (file, _shows, md5) = scanned(&store, "r");
        store.keep_pdf_ocr_page(&file, &md5, &page(2, &strs(&first))).unwrap();
        store.keep_pdf_ocr_page(&file, &md5, &page(2, &strs(&second))).unwrap();
        store.keep_pdf_ocr_page(&file, &md5, &page(3, &[])).unwrap();
        prop_assert!(store.finish_pdf_ocr(&file, &md5, "t").unwrap());
        let ocr = store.pdf_ocr(&file).unwrap().unwrap();
        let read: Vec<String> = ocr.page(2).unwrap().words.iter().map(|w| w.text.clone()).collect();
        prop_assert_eq!(read, second);
        let rev = store.get(&file).unwrap().unwrap().rev;
        prop_assert!(!store.finish_pdf_ocr(&file, &md5, "t").unwrap());
        prop_assert_eq!(store.get(&file).unwrap().unwrap().rev, rev);
    }

    /// A PDF only a locked note links leaks nothing, whenever it was locked.
    #[test]
    fn prop_locking_forgets_scans(read_first in any::<bool>(), w in word()) {
        let (_dir, store) = open();
        let (file, shows, md5) = scanned(&store, "l");
        if read_first {
            store.keep_pdf_ocr_page(&file, &md5, &page(2, &[&w])).unwrap();
            store.keep_pdf_ocr_page(&file, &md5, &page(3, &[])).unwrap();
            store.finish_pdf_ocr(&file, &md5, "t").unwrap();
        }
        lock(&store, &shows);
        prop_assert!(store.pdfs_to_ocr(5).unwrap().is_empty());
        prop_assert!(find(&store, &w).is_empty());
        prop_assert!(!store.keep_pdf_ocr_page(&file, &md5, &page(2, &[&w])).unwrap());
    }
}
