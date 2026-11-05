//! The words in PDF attachments, for the app's search (issue #43, step 7).
//!
//! A PDF's text is read on every device, never synced: the PDF viewer's
//! worker hands the pages over ([`Store::set_pdf_text`]), each page is kept
//! in `pdf_page_text`, and `pdf_state` says which bytes (MD5) they were
//! read from, so new bytes are read again. The joined pages are mirrored
//! into `file_text` (see `ocr.rs`, which owns the mirror), so the app's
//! search finds a note by the words in the PDFs it links without any change
//! to the query.
//!
//! Locked notes: a PDF is read only while a note that may be read links it
//! (not locked, not in a locked notebook, not in the trash, not tagged
//! `#ai-exclude`: the rule the images follow). The rows of a PDF no such
//! note links any more are deleted by [`Store::forget_unlinked_pdfs`],
//! which locking a note or a notebook runs in the same transaction. Search
//! reads a note's body as empty when it is locked, so a locked note is
//! never returned in the meantime.

use std::collections::{BTreeMap, HashSet};

use kanso_core::model::{FILE_LOCK_FIELD, PDF_OCR_FIELD};
use kanso_core::now_ms;
use kanso_pdf::PdfOcr;
use rusqlite::{Connection, params};
use serde_json::{Map, Value};

use crate::pdf_text_model::{MAX_PAGE_CHARS, MAX_PDF_CHARS, PdfToIndex};
use crate::{Result, Store, schema};

/// The tables, for emptying them with the library.
pub(crate) const TABLES: [&str; 2] = ["pdf_page_text", "pdf_state"];

/// Whether a file document is a PDF whose text may be kept: it says it is
/// one and isn't sealed.
pub(crate) fn is_plain_pdf(fields: &Map<String, Value>) -> bool {
    !fields.contains_key(FILE_LOCK_FIELD)
        && fields
            .get("contentType")
            .and_then(Value::as_str)
            .is_some_and(|t| t.eq_ignore_ascii_case("application/pdf"))
}

fn forget(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM pdf_page_text WHERE file_id = ?", [id])?;
    conn.execute("DELETE FROM pdf_state WHERE file_id = ?", [id])?;
    // Scans read so far were read from the bytes these pages were.
    conn.execute("DELETE FROM pdf_ocr_pending WHERE file_id = ?", [id])?;
    Ok(())
}

/// Brings the extracted pages of file `id` in line with its document
/// (called on every write of a file, from `ocr::write_file`): they go when
/// the file is deleted, sealed, no longer a PDF or has new bytes. Returns
/// the joined pages and the MD5 they were read from, when it still has
/// some.
pub(crate) fn sync_file(
    conn: &Connection,
    id: &str,
    fields: Option<&Map<String, Value>>,
) -> Result<Option<(String, String)>> {
    let Some(fields) = fields.filter(|f| is_plain_pdf(f)) else {
        forget(conn, id)?;
        return Ok(None);
    };
    let now = fields.get("md5digest").and_then(Value::as_str);
    // The words of scanned pages, read here or by another device, for the
    // bytes the file has now.
    let ocr = fields
        .get(PDF_OCR_FIELD)
        .and_then(PdfOcr::from_value)
        .filter(|ocr| Some(ocr.md5.as_str()) == now);
    let read: Option<String> = conn
        .query_row("SELECT md5 FROM pdf_state WHERE file_id = ?", [id], |row| {
            row.get(0)
        })
        .ok();
    let read = read.filter(|read| Some(read.as_str()) == now);
    if read.is_none() {
        forget(conn, id)?;
    }
    let mut pages: BTreeMap<i64, String> = BTreeMap::new();
    if read.is_some() {
        let mut stmt =
            conn.prepare_cached("SELECT page, text FROM pdf_page_text WHERE file_id = ?")?;
        for row in stmt.query_map([id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })? {
            let (page, text) = row?;
            pages.insert(page, text);
        }
    }
    // A page PDFium found no text on takes the words read from its picture.
    if let Some(ocr) = &ocr {
        for page in &ocr.pages {
            let slot = pages.entry(i64::from(page.page)).or_default();
            if slot.trim().is_empty() {
                *slot = page.text();
            }
        }
    }
    let joined = pages.into_values().collect::<Vec<_>>().join("\n");
    let md5 = read.or_else(|| ocr.map(|ocr| ocr.md5));
    Ok(md5
        .filter(|_| !joined.trim().is_empty())
        .map(|md5| (joined, md5)))
}

/// Of `wanted` file ids, the ones a note that may be read links. Reads the
/// `note_files` index (the files each note's body links, kept with every
/// write), so it costs a lookup per file, not a pass over every note.
pub(crate) fn shown_among(conn: &Connection, wanted: HashSet<String>) -> Result<HashSet<String>> {
    if wanted.is_empty() {
        return Ok(HashSet::new());
    }
    let wanted = serde_json::to_string(&wanted.into_iter().collect::<Vec<_>>())
        .expect("a list of strings serialises");
    let mut stmt = conn.prepare_cached(concat!(
        "SELECT DISTINCT f.file_id FROM note_files f JOIN docs d ON d.id = f.note_id
         WHERE f.file_id IN (SELECT value FROM json_each(?1)) AND ",
        schema::discoverable!("d.")
    ))?;
    let shown = stmt
        .query_map([wanted], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(shown)
}

/// Deletes the rows of every PDF that no note that may be read links.
pub(crate) fn forget_unlinked(conn: &Connection) -> Result<usize> {
    let ids: HashSet<String> = conn
        .prepare("SELECT file_id FROM pdf_state")?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let shown = shown_among(conn, ids.clone())?;
    let mut gone = 0;
    for id in ids.difference(&shown) {
        forget(conn, id)?;
        conn.execute("DELETE FROM file_text WHERE file_id = ?", [id])?;
        gone += 1;
    }
    Ok(gone)
}

/// `text` cut to `chars` characters, on a character boundary.
fn truncated(text: &str, chars: usize) -> &str {
    match text.char_indices().nth(chars) {
        Some((end, _)) => &text[..end],
        None => text,
    }
}

impl Store {
    /// Live PDF attachments whose pages haven't been read from their current
    /// bytes, newest first, at most `limit`. Only PDFs that at least one
    /// note that may be read links: not those only a locked note, a note in
    /// the trash or a note tagged `#ai-exclude` links. Sealed files are
    /// never offered.
    pub fn pdfs_to_index(&self, limit: usize) -> Result<Vec<PdfToIndex>> {
        self.read(|conn| {
            if limit == 0 {
                return Ok(Vec::new());
            }
            let mut stmt = conn.prepare(
                "SELECT d.id, json_extract(d.json, '$.md5digest') FROM docs d
                 LEFT JOIN pdf_state s ON s.file_id = d.id
                 WHERE d.type = 'file' AND d.deleted = 0
                   AND lower(json_extract(d.json, '$.contentType')) = 'application/pdf'
                   AND json_extract(d.json, '$.md5digest') IS NOT NULL
                   AND json_extract(d.json, '$.kansoFileLock') IS NULL
                   AND (s.md5 IS NULL OR s.md5 != json_extract(d.json, '$.md5digest'))
                 ORDER BY json_extract(d.json, '$.createdAt') DESC, d.id",
            )?;
            let candidates = stmt
                .query_map([], |row| {
                    Ok(PdfToIndex {
                        id: row.get(0)?,
                        md5: row.get(1)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            let shown = shown_among(conn, candidates.iter().map(|c| c.id.clone()).collect())?;
            Ok(candidates
                .into_iter()
                .filter(|c| shown.contains(&c.id))
                .take(limit)
                .collect())
        })
    }

    /// Keeps `pages` (one string per page, the first being page 1) as the
    /// text of PDF `id`, read from the bytes with `md5` by `engine`, and
    /// mirrors them into search. Replaces what was kept before, so keeping
    /// the same pages again changes nothing. A page is cut to
    /// [`MAX_PAGE_CHARS`] and the whole to [`MAX_PDF_CHARS`] bytes; pages
    /// past that are kept empty. No pages means the PDF has no text layer
    /// (or couldn't be read): it is kept as read, so it isn't tried again
    /// until its bytes change.
    ///
    /// Nothing is kept, and `false` returned, when the file is gone, sealed,
    /// not a PDF, has other bytes now, or no note that may be read links it
    /// (it was locked or unlinked while it was being read).
    pub fn set_pdf_text(
        &self,
        id: &str,
        md5: &str,
        pages: &[String],
        engine: &str,
    ) -> Result<bool> {
        self.write(|tx, _| {
            let json: Option<String> = tx
                .query_row(
                    "SELECT json FROM docs WHERE id = ? AND type = 'file' AND deleted = 0",
                    [id],
                    |row| row.get(0),
                )
                .ok();
            let Some(Value::Object(fields)) = json.and_then(|j| serde_json::from_str(&j).ok())
            else {
                return Ok(false);
            };
            if !is_plain_pdf(&fields)
                || fields.get("md5digest").and_then(Value::as_str) != Some(md5)
                || shown_among(tx, HashSet::from([id.to_owned()]))?.is_empty()
            {
                return Ok(false);
            }
            forget(tx, id)?;
            let mut budget = MAX_PDF_CHARS;
            let mut insert =
                tx.prepare("INSERT INTO pdf_page_text (file_id, page, text) VALUES (?1, ?2, ?3)")?;
            for (ix, page) in pages.iter().enumerate() {
                let mut text = truncated(page, MAX_PAGE_CHARS);
                if text.len() > budget {
                    let mut end = budget;
                    while !text.is_char_boundary(end) {
                        end -= 1;
                    }
                    text = &text[..end];
                }
                budget -= text.len();
                insert.execute(params![id, ix as i64 + 1, text])?;
            }
            drop(insert);
            tx.execute(
                "INSERT INTO pdf_state (file_id, md5, pages, engine, at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, md5, pages.len() as i64, engine, now_ms()],
            )?;
            crate::ocr::write_file(tx, id, Some(&fields))?;
            Ok(true)
        })
    }

    /// The text of each page of PDF `id`, page 1 first; empty when none was
    /// kept.
    pub fn pdf_pages(&self, id: &str) -> Result<Vec<String>> {
        self.read(|conn| {
            let mut stmt = conn
                .prepare_cached("SELECT text FROM pdf_page_text WHERE file_id = ? ORDER BY page")?;
            Ok(stmt
                .query_map([id], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Whether the pages of PDF `id` have been read from its current bytes
    /// (even if it has no text).
    pub fn pdf_indexed(&self, id: &str) -> Result<bool> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    "SELECT 1 FROM pdf_state s JOIN docs d ON d.id = s.file_id
                     WHERE s.file_id = ? AND s.md5 = json_extract(d.json, '$.md5digest')",
                    [id],
                    |_| Ok(()),
                )
                .is_ok())
        })
    }

    /// Deletes the text of every PDF that no note that may be read links
    /// any more, and its search rows. Returns how many PDFs it forgot. The
    /// background pass runs it before choosing the next PDF; locking a note
    /// or a notebook runs it in the same transaction.
    pub fn forget_unlinked_pdfs(&self) -> Result<usize> {
        self.write(|tx, _| forget_unlinked(tx))
    }

    /// Deletes the text of every PDF (Settings turned PDF search off).
    pub fn forget_all_pdf_text(&self) -> Result<()> {
        self.write(|tx, _| {
            let ids: Vec<String> = tx
                .prepare("SELECT file_id FROM pdf_state")?
                .query_map([], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            for id in ids {
                forget(tx, &id)?;
                // The mirror keeps only what the file itself says.
                let json: Option<String> = tx
                    .query_row("SELECT json FROM docs WHERE id = ?", [&id], |r| r.get(0))
                    .ok();
                let fields = json.and_then(|j| serde_json::from_str::<Value>(&j).ok());
                crate::ocr::write_file(tx, &id, fields.as_ref().and_then(Value::as_object))?;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Origin, Put, SearchParams};
    use base64::Engine as _;
    use kanso_core::DocType;
    use proptest::prelude::*;
    use serde_json::json;

    fn open() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    fn pdf(store: &Store, tag: &str) -> String {
        let bytes = format!("%PDF-1.7 {tag}");
        store
            .put(
                DocType::File,
                json!({
                    "name": "doc.pdf",
                    "contentType": "application/pdf",
                    "_attachments": {"index": {
                        "content_type": "application/pdf",
                        "data": base64::engine::general_purpose::STANDARD.encode(bytes),
                    }},
                }),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn book(store: &Store) -> String {
        store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id
    }

    fn note(store: &Store, book: &str, title: &str, body: &str) -> String {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn link(file: &str) -> String {
        format!("[doc.pdf](inkdrop://{file})")
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

    fn lock(store: &Store, note: &str) {
        let mut sealed = store.get_api(note, None, false).unwrap();
        sealed["body"] = "".into();
        sealed["kansoLock"] =
            json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
        store.lock_note(sealed).unwrap();
    }

    fn rows(store: &Store, id: &str) -> (i64, i64, i64) {
        store
            .read(|conn| {
                let count = |sql: &str| -> rusqlite::Result<i64> {
                    conn.query_row(sql, [id], |row| row.get(0))
                };
                Ok((
                    count("SELECT COUNT(*) FROM pdf_page_text WHERE file_id = ?")?,
                    count("SELECT COUNT(*) FROM pdf_state WHERE file_id = ?")?,
                    count("SELECT COUNT(*) FROM file_text WHERE file_id = ?")?,
                ))
            })
            .unwrap()
    }

    fn pages(words: &[Vec<String>]) -> Vec<String> {
        words.iter().map(|p| p.join(" ")).collect()
    }

    #[test]
    fn a_pdfs_words_find_the_note_that_links_it() {
        let (_dir, store) = open();
        let b = book(&store);
        let file = pdf(&store, "a");
        let shows = note(&store, &b, "Report", &link(&file));
        note(&store, &b, "Other", "nothing here");
        let waiting = store.pdfs_to_index(10).unwrap();
        assert_eq!(waiting.len(), 1);
        assert!(!store.pdf_indexed(&file).unwrap());
        assert!(
            store
                .set_pdf_text(
                    &file,
                    &waiting[0].md5,
                    &["Quarterly revenue".into(), "".into(), "Appendix".into()],
                    "test"
                )
                .unwrap()
        );
        assert!(store.pdf_indexed(&file).unwrap());
        assert!(store.pdfs_to_index(10).unwrap().is_empty());
        assert_eq!(find(&store, "REVENUE"), vec![shows.clone()]);
        assert_eq!(find(&store, "appendix report"), vec![shows]);
        assert_eq!(store.pdf_pages(&file).unwrap().len(), 3);
        // Other bytes, a gone file: refused.
        assert!(!store.set_pdf_text(&file, "stale", &[], "test").unwrap());
        // The local API's search keeps to titles and bodies.
        let api = store
            .search_summaries(&SearchParams {
                query: kanso_core::Query::parse("revenue"),
                ..Default::default()
            })
            .unwrap();
        assert!(api.is_empty());
    }

    #[test]
    fn new_bytes_drop_the_old_pages_and_are_read_again() {
        let (_dir, store) = open();
        let b = book(&store);
        let file = pdf(&store, "one");
        note(&store, &b, "Report", &link(&file));
        let md5 = md5_of(&store, &file);
        store
            .set_pdf_text(&file, &md5, &["old words".into()], "test")
            .unwrap();
        let rev = store.get(&file).unwrap().unwrap().rev;
        store
            .put(
                DocType::File,
                json!({"_id": file, "_rev": rev, "_attachments": {"index": {
                    "content_type": "application/pdf",
                    "data": base64::engine::general_purpose::STANDARD.encode(b"%PDF-1.7 two"),
                }}}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        assert_eq!(rows(&store, &file), (0, 0, 0));
        assert!(find(&store, "old").is_empty());
        assert_eq!(store.pdfs_to_index(10).unwrap().len(), 1);
    }

    #[test]
    fn the_mirror_survives_a_rebuild_and_a_page_over_the_cap_is_cut() {
        let (dir, store) = open();
        let b = book(&store);
        let file = pdf(&store, "cap");
        let shows = note(&store, &b, "Report", &link(&file));
        let md5 = md5_of(&store, &file);
        let long = "é".repeat(MAX_PAGE_CHARS + 10);
        store
            .set_pdf_text(&file, &md5, &[long, "tail".into()], "test")
            .unwrap();
        assert_eq!(
            store.pdf_pages(&file).unwrap()[0].chars().count(),
            MAX_PAGE_CHARS
        );
        store
            .read(|conn| {
                conn.execute("DELETE FROM meta WHERE key = 'index.file_text_version'", [])?;
                conn.execute("DELETE FROM file_text", [])?;
                Ok(())
            })
            .unwrap();
        drop(store);
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        assert_eq!(find(&store, "tail"), vec![shows]);
    }

    #[test]
    fn locking_the_only_note_forgets_the_pdf_and_nothing_is_read_after() {
        let (_dir, store) = open();
        let b = book(&store);
        let file = pdf(&store, "lock");
        let shows = note(&store, &b, "Secret", &link(&file));
        let md5 = md5_of(&store, &file);
        store
            .set_pdf_text(&file, &md5, &["classified words".into()], "test")
            .unwrap();
        assert_eq!(find(&store, "classified"), vec![shows.clone()]);
        lock(&store, &shows);
        assert_eq!(rows(&store, &file), (0, 0, 0));
        assert!(find(&store, "classified").is_empty());
        assert!(store.pdfs_to_index(10).unwrap().is_empty());
        assert!(!store.set_pdf_text(&file, &md5, &["x".into()], "t").unwrap());
    }

    fn word() -> impl Strategy<Value = String> {
        "[a-z]{6,10}"
    }

    fn doc_pages() -> impl Strategy<Value = Vec<Vec<String>>> {
        prop::collection::vec(prop::collection::vec(word(), 1..5), 1..5)
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]

        /// Search finds the note that links a PDF by every word on every
        /// indexed page.
        #[test]
        fn prop_search_finds_every_indexed_pages_words(words in doc_pages()) {
            let (_dir, store) = open();
            let b = book(&store);
            let file = pdf(&store, "p");
            let shows = note(&store, &b, "Report", &link(&file));
            let md5 = md5_of(&store, &file);
            prop_assert!(store.set_pdf_text(&file, &md5, &pages(&words), "test").unwrap());
            for page in &words {
                for w in page {
                    prop_assert_eq!(find(&store, w), vec![shows.clone()], "{}", w);
                }
            }
        }

        /// Indexing the same pages again changes nothing; indexing other
        /// pages leaves only those.
        #[test]
        fn prop_reindexing_is_idempotent(first in doc_pages(), second in doc_pages()) {
            let (_dir, store) = open();
            let b = book(&store);
            let file = pdf(&store, "p");
            note(&store, &b, "Report", &link(&file));
            let md5 = md5_of(&store, &file);
            store.set_pdf_text(&file, &md5, &pages(&first), "test").unwrap();
            let once = (store.pdf_pages(&file).unwrap(), rows(&store, &file));
            store.set_pdf_text(&file, &md5, &pages(&first), "test").unwrap();
            prop_assert_eq!((store.pdf_pages(&file).unwrap(), rows(&store, &file)), once);
            store.set_pdf_text(&file, &md5, &pages(&second), "test").unwrap();
            prop_assert_eq!(store.pdf_pages(&file).unwrap(), pages(&second));
            // Words only the first had are gone from search.
            for w in first.iter().flatten().filter(|w| !pages(&second).join(" ").contains(w.as_str())) {
                prop_assert!(find(&store, w).is_empty(), "{}", w);
            }
        }

        /// Deleting the PDF, or the last link to it, deletes its rows.
        #[test]
        fn prop_removing_the_pdf_or_its_link_removes_its_rows(
            words in doc_pages(),
            delete_file in any::<bool>(),
        ) {
            let (_dir, store) = open();
            let b = book(&store);
            let file = pdf(&store, "p");
            let shows = note(&store, &b, "Report", &link(&file));
            let md5 = md5_of(&store, &file);
            store.set_pdf_text(&file, &md5, &pages(&words), "test").unwrap();
            prop_assert_eq!(rows(&store, &file).1, 1);
            if delete_file {
                store.delete(&file, Origin::Ui).unwrap();
            } else {
                let rev = store.get(&shows).unwrap().unwrap().rev;
                store.put(
                    DocType::Note,
                    json!({"_id": shows, "_rev": rev, "body": "no link now"}),
                    Put::Merge,
                    Origin::Ui,
                ).unwrap();
                store.forget_unlinked_pdfs().unwrap();
            }
            prop_assert_eq!(rows(&store, &file), (0, 0, 0));
            for w in words.iter().flatten() {
                prop_assert!(find(&store, w).is_empty(), "{}", w);
            }
        }

        /// A PDF only a locked note links is never offered, read, kept,
        /// searched or returned, whether the note was locked before or after
        /// its pages were read; a plain note linking the same PDF still
        /// finds it.
        #[test]
        fn prop_locked_notes_never_leak(
            words in doc_pages(),
            read_first in any::<bool>(),
            also_plain in any::<bool>(),
        ) {
            let (_dir, store) = open();
            let b = book(&store);
            let file = pdf(&store, "p");
            let secret = note(&store, &b, "Secret", &link(&file));
            let plain = also_plain.then(|| note(&store, &b, "Plain", &link(&file)));
            let md5 = md5_of(&store, &file);
            if read_first {
                prop_assert!(store.set_pdf_text(&file, &md5, &pages(&words), "test").unwrap());
            }
            lock(&store, &secret);
            let expected: Vec<String> = plain.iter().cloned().collect();
            if also_plain {
                if !read_first {
                    prop_assert!(store.set_pdf_text(&file, &md5, &pages(&words), "test").unwrap());
                }
            } else {
                prop_assert_eq!(rows(&store, &file), (0, 0, 0));
                prop_assert!(store.pdfs_to_index(10).unwrap().is_empty());
                prop_assert!(!store.set_pdf_text(&file, &md5, &pages(&words), "test").unwrap());
                prop_assert_eq!(rows(&store, &file), (0, 0, 0));
            }
            for w in words.iter().flatten() {
                let found = find(&store, w);
                prop_assert!(!found.contains(&secret), "{}", w);
                if also_plain {
                    prop_assert_eq!(found, expected.clone());
                } else {
                    prop_assert!(found.is_empty());
                }
            }
        }
    }
}
