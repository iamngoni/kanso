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

// MARK: Locked notes (the words never stay in plain text for a locked PDF)

/// Reads every page of `file`'s scan, with `words` on page 2, and writes
/// the set into the file.
fn read_all(store: &Store, file: &str, md5: &str, words: &[&str]) {
    assert!(store.keep_pdf_ocr_page(file, md5, &page(2, words)).unwrap());
    assert!(store.keep_pdf_ocr_page(file, md5, &page(3, &[])).unwrap());
    assert!(store.finish_pdf_ocr(file, md5, "vision").unwrap());
}

/// Where the words of `file`'s scan are on this device: in its document,
/// its search row, its past versions, and the pages kept so far.
#[derive(Debug, PartialEq)]
struct Traces {
    field: bool,
    search_row: bool,
    history: i64,
    kept: i64,
}

fn traces(store: &Store, file: &str) -> Traces {
    let field = store
        .get(file)
        .unwrap()
        .is_some_and(|d| d.fields.contains_key(PDF_OCR_FIELD));
    store
        .read(|conn| {
            let count = |sql: &str| -> rusqlite::Result<i64> {
                conn.query_row(sql, [file], |row| row.get(0))
            };
            Ok(Traces {
                field,
                search_row: count("SELECT COUNT(*) FROM file_text WHERE file_id = ?")? > 0,
                history: count(&format!(
                    "SELECT COUNT(*) FROM revisions WHERE doc_id = ?
                     AND json_extract(json, '$.{PDF_OCR_FIELD}') IS NOT NULL"
                ))?,
                kept: count("SELECT COUNT(*) FROM pdf_ocr_pending WHERE file_id = ?")?,
            })
        })
        .unwrap()
}

const NONE: Traces = Traces {
    field: false,
    search_row: false,
    history: 0,
    kept: 0,
};

/// The version of `file` the next sync sends, if it sends one.
fn pushed(store: &Store, file: &str) -> Option<crate::SyncDoc> {
    store
        .pending_push(1_000)
        .unwrap()
        .into_iter()
        .find(|d| d.id == file)
}

/// A version of `id` from another device: its fields changed by `change`,
/// one generation on, newer than anything here.
fn from_elsewhere(
    store: &Store,
    id: &str,
    change: impl FnOnce(&mut serde_json::Map<String, Value>),
) {
    let doc = store.get(id).unwrap().unwrap();
    let mut fields = doc.fields.clone();
    change(&mut fields);
    fields.insert("updatedAt".into(), json!(now_ms() + 3_600_000));
    let rev: kanso_core::Rev = doc.rev.parse().unwrap();
    let remote = crate::SyncDoc {
        id: id.to_owned(),
        rev: format!("{}-{}", rev.generation + 1, "ab".repeat(16)),
        deleted: false,
        fields,
        base_rev: None,
    };
    store.apply_remote(&remote, None, None).unwrap();
}

fn sealed(fields: &mut serde_json::Map<String, Value>) {
    fields.insert("body".into(), "".into());
    fields.insert(
        "kansoLock".into(),
        json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"}),
    );
}

fn unlock(store: &Store, note: &str, file: &str) {
    let mut open = store.get_api(note, None, false).unwrap();
    open.as_object_mut().unwrap().remove("kansoLock");
    open["body"] = format!("[scan.pdf](inkdrop://{file})").into();
    store.unlock_note(open).unwrap();
}

#[test]
fn locking_drops_the_words_everywhere_and_the_drop_syncs() {
    let (_dir, store) = open();
    let (file, shows, md5) = scanned(&store, "lock");
    read_all(&store, &file, &md5, &["confidential", "diagnosis"]);
    // A later change to the file keeps a version with the words in history.
    let rev = store.get(&file).unwrap().unwrap().rev;
    store
        .put(
            DocType::File,
            json!({"_id": file, "_rev": rev, "name": "renamed.pdf"}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
    assert!(traces(&store, &file).field && traces(&store, &file).search_row);
    assert!(traces(&store, &file).history > 0);

    lock(&store, &shows);
    assert_eq!(traces(&store, &file), NONE);
    // The file's next version, without the words, is what sync sends.
    let sent = pushed(&store, &file).expect("the drop syncs");
    assert!(!sent.fields.contains_key(PDF_OCR_FIELD));
    assert_eq!(sent.fields["name"], "renamed.pdf");
    assert!(store.pdfs_to_ocr(10).unwrap().is_empty());
}

#[test]
fn a_lock_from_another_device_drops_the_words_here() {
    let (_dir, store) = open();
    let (file, shows, md5) = scanned(&store, "remote-lock");
    read_all(&store, &file, &md5, &["payslip"]);
    from_elsewhere(&store, &shows, sealed);
    assert!(
        store
            .get(&shows)
            .unwrap()
            .unwrap()
            .fields
            .contains_key("kansoLock")
    );
    assert_eq!(traces(&store, &file), NONE);
    assert!(
        !pushed(&store, &file)
            .unwrap()
            .fields
            .contains_key(PDF_OCR_FIELD)
    );
}

#[test]
fn words_written_after_the_lock_are_never_kept() {
    let (_dir, store) = open();
    let (file, shows, md5) = scanned(&store, "late");
    read_all(&store, &file, &md5, &["first"]);
    let words = store.get(&file).unwrap().unwrap().fields[PDF_OCR_FIELD].clone();
    lock(&store, &shows);
    // A device that hadn't seen the lock sends the words again.
    from_elsewhere(&store, &file, |f| {
        f.insert(PDF_OCR_FIELD.into(), words.clone());
    });
    assert_eq!(traces(&store, &file), NONE);
    assert!(
        !pushed(&store, &file)
            .unwrap()
            .fields
            .contains_key(PDF_OCR_FIELD)
    );
    // Nor does a write here keep them.
    let rev = store.get(&file).unwrap().unwrap().rev;
    store
        .put(
            DocType::File,
            json!({"_id": file, "_rev": rev, PDF_OCR_FIELD: words}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(traces(&store, &file), NONE);
}

#[test]
fn a_shared_pdf_keeps_its_words_until_no_open_note_links_it() {
    let (_dir, store) = open();
    let (file, locked, md5) = scanned(&store, "shared");
    let open = note(&store, &file);
    read_all(&store, &file, &md5, &["minutes"]);
    lock(&store, &locked);
    // The open note still shows it: its words stay, and search finds them
    // there.
    assert!(traces(&store, &file).field);
    assert_eq!(find(&store, "minutes"), vec![open.clone()]);
    // The open note lets go of it.
    let rev = store.get(&open).unwrap().unwrap().rev;
    store
        .put(
            DocType::Note,
            json!({"_id": open, "_rev": rev, "body": "No PDF here now."}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(traces(&store, &file), NONE);
    assert!(find(&store, "minutes").is_empty());
}

#[test]
fn unlocking_reads_the_scan_again() {
    let (_dir, store) = open();
    let (file, shows, md5) = scanned(&store, "unlock");
    read_all(&store, &file, &md5, &["receipt"]);
    lock(&store, &shows);
    assert_eq!(traces(&store, &file), NONE);
    unlock(&store, &shows, &file);
    // Its text is read first, then its scans.
    assert!(
        store
            .pdfs_to_index(10)
            .unwrap()
            .iter()
            .any(|p| p.id == file)
    );
    store
        .set_pdf_text(&file, &md5, &["Typed".into(), "".into(), "".into()], "test")
        .unwrap();
    assert_eq!(store.pdfs_to_ocr(10).unwrap().len(), 1);
    read_all(&store, &file, &md5, &["receipt"]);
    assert_eq!(find(&store, "receipt"), vec![shows]);
}

#[test]
fn an_unlocked_note_never_drops_words() {
    let (_dir, store) = open();
    let (file, shows, md5) = scanned(&store, "plain");
    read_all(&store, &file, &md5, &["agenda"]);
    // Edits, a second note, a note in the trash: no lock, nothing dropped.
    let rev = store.get(&shows).unwrap().unwrap().rev;
    store
        .put(
            DocType::Note,
            json!({"_id": shows, "_rev": rev, "title": "Renamed"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    note(&store, &file);
    assert!(traces(&store, &file).field);
    assert!(store.pdf_ocr(&file).unwrap().is_some());
}

#[test]
fn a_kept_page_that_no_longer_reads_is_finished_not_read_forever() {
    let (_dir, store) = open();
    let (file, _, md5) = scanned(&store, "broken");
    store
        .keep_pdf_ocr_page(&file, &md5, &page(2, &["fine"]))
        .unwrap();
    store
        .write(|tx, _| {
            tx.execute(
                "INSERT INTO pdf_ocr_pending (file_id, md5, page, json) VALUES (?1, ?2, 3, 'garbage')",
                params![file, md5],
            )?;
            Ok(())
        })
        .unwrap();
    assert!(store.pdf_pages_to_ocr(&file, &md5).unwrap().is_empty());
    assert!(store.finish_pdf_ocr(&file, &md5, "vision").unwrap());
    assert!(store.pdfs_to_ocr(10).unwrap().is_empty());
    let ocr = store.pdf_ocr(&file).unwrap().unwrap();
    assert_eq!(ocr.pages.iter().map(|p| p.page).collect::<Vec<_>>(), [2, 3]);
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

    /// A PDF only a locked note links leaks nothing, whenever it was locked
    /// and however the lock and the words arrived: not in the file's
    /// document (so not in sync), not in search, not in its history, not
    /// in the pages kept so far. A second, open note keeps the words until
    /// it lets go of the PDF.
    #[test]
    fn prop_locking_forgets_scans(
        read_first in any::<bool>(),
        part_read in any::<bool>(),
        lock_by_sync in any::<bool>(),
        words_again in any::<bool>(),
        shared in any::<bool>(),
        w in word(),
    ) {
        let (_dir, store) = open();
        let (file, shows, md5) = scanned(&store, "l");
        let open_note = shared.then(|| note(&store, &file));
        if read_first {
            read_all(&store, &file, &md5, &[&w]);
        } else if part_read {
            store.keep_pdf_ocr_page(&file, &md5, &page(2, &[&w])).unwrap();
        }
        let words = store.get(&file).unwrap().unwrap().fields.get(PDF_OCR_FIELD).cloned();
        if lock_by_sync {
            from_elsewhere(&store, &shows, sealed);
        } else {
            lock(&store, &shows);
        }
        if words_again && let Some(words) = words.clone() {
            from_elsewhere(&store, &file, |f| {
                f.insert(PDF_OCR_FIELD.into(), words);
            });
        }
        if let Some(open_note) = open_note {
            prop_assert_eq!(traces(&store, &file).field, words.is_some());
            let rev = store.get(&open_note).unwrap().unwrap().rev;
            store
                .put(
                    DocType::Note,
                    json!({"_id": open_note, "_rev": rev, "body": ""}),
                    Put::Merge,
                    Origin::Ui,
                )
                .unwrap();
        }
        prop_assert_eq!(traces(&store, &file), NONE);
        if words.is_some() {
            prop_assert!(!pushed(&store, &file).unwrap().fields.contains_key(PDF_OCR_FIELD));
        }
        prop_assert!(store.pdfs_to_ocr(5).unwrap().is_empty());
        prop_assert!(find(&store, &w).is_empty());
        prop_assert!(!store.keep_pdf_ocr_page(&file, &md5, &page(2, &[&w])).unwrap());
    }
}

/// Seals `file` as locking its note does (issue #44): here, through
/// `put_sealed_file`, or as a version from another device that still
/// carries `words`, as an older writer might send it.
fn seal_file(store: &Store, file: &str, by_sync: bool, words: Option<Value>) {
    let mut lock = kanso_core::model::FileLock::sealed("AAAAAAAAAAAAAAAAAAAAAA".into());
    lock.meta = true;
    if by_sync {
        from_elsewhere(store, file, |f| {
            kanso_core::model::FileLock::store(Some(&lock), f);
            if let Some(words) = words {
                f.insert(PDF_OCR_FIELD.into(), words);
            }
        });
    } else {
        let mut meta = store.file_meta(file).unwrap().unwrap();
        kanso_core::model::FileLock::store(Some(&lock), &mut meta.extra);
        if let Some(words) = words {
            meta.extra.insert(PDF_OCR_FIELD.into(), words);
        }
        store
            .put_sealed_file(meta, crate::Attachment::new(b"sealed container".to_vec()))
            .unwrap();
    }
}

proptest! {
    #![proptest_config(ProptestConfig { rng_seed: RngSeed::Fixed(SEED), ..ProptestConfig::with_cases(32) })]

    /// A locked note's PDF, sealed with it (issue #44), keeps no words read
    /// from its scans in plain text anywhere: not in the file's document,
    /// its search row, its history or the pages kept so far, not in what
    /// sync sends, and not in what the local API serves (with or without
    /// the bytes). However the seal, the lock and stray words arrive.
    #[test]
    fn prop_a_locked_notes_pdf_keeps_no_scan_text(
        read_first in any::<bool>(),
        part_read in any::<bool>(),
        seal_by_sync in any::<bool>(),
        lock_by_sync in any::<bool>(),
        lock_before_seal in any::<bool>(),
        stale_words_in_seal in any::<bool>(),
        words_again in any::<bool>(),
        w in word(),
    ) {
        let (_dir, store) = open();
        let (file, shows, md5) = scanned(&store, "sealed");
        if read_first {
            read_all(&store, &file, &md5, &[&w]);
        } else if part_read {
            store.keep_pdf_ocr_page(&file, &md5, &page(2, &[&w])).unwrap();
        }
        let words = store.get(&file).unwrap().unwrap().fields.get(PDF_OCR_FIELD).cloned();
        let lock_it = |store: &Store| {
            if lock_by_sync {
                from_elsewhere(store, &shows, sealed);
            } else {
                lock(store, &shows);
            }
        };
        if lock_before_seal {
            lock_it(&store);
        }
        seal_file(&store, &file, seal_by_sync, words.clone().filter(|_| stale_words_in_seal));
        if !lock_before_seal {
            lock_it(&store);
        }
        if words_again && let Some(words) = words.clone() {
            from_elsewhere(&store, &file, |f| {
                f.insert(PDF_OCR_FIELD.into(), words);
            });
        }

        prop_assert_eq!(traces(&store, &file), NONE);
        prop_assert_eq!(store.pdf_ocr(&file).unwrap(), None);
        if let Some(sent) = pushed(&store, &file) {
            prop_assert!(!sent.fields.contains_key(PDF_OCR_FIELD));
        }
        for bytes in [false, true] {
            let served = store.get_api(&file, None, bytes).unwrap().to_string();
            prop_assert!(!served.contains(PDF_OCR_FIELD), "{}", served);
            prop_assert!(!served.contains(w.as_str()), "{}", served);
        }
        let any_text: i64 = store
            .read(|conn| {
                Ok(conn.query_row(
                    "SELECT (SELECT COUNT(*) FROM pdf_page_text WHERE file_id = ?1)
                          + (SELECT COUNT(*) FROM file_text WHERE file_id = ?1)",
                    [&file],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        prop_assert_eq!(any_text, 0);
        prop_assert!(find(&store, &w).is_empty());
        prop_assert!(store.pdfs_to_ocr(5).unwrap().is_empty());
        prop_assert!(!store.keep_pdf_ocr_page(&file, &md5, &page(2, &[&w])).unwrap());
    }
}
