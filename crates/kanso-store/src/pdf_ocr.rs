//! The words on the scanned pages of PDFs (issue #43, step 8).
//!
//! A scanned page has no text layer, so a reader that can recognise text
//! (a Mac) reads its picture. This is the store's side of that:
//!
//! * [`Store::pdfs_to_ocr`] names the PDFs that have pages with no text and
//!   no words read for the bytes they have now;
//! * [`Store::keep_pdf_ocr_page`] keeps each page's words as it is read, in a
//!   local table, so reading can be paused or stopped and go on later;
//! * [`Store::finish_pdf_ocr`] writes the whole set once into the file's
//!   `kansoPdfOcr` field (`kanso_core::model::PDF_OCR_FIELD`), which syncs
//!   sealed with the file, so a Windows or Linux device finds, selects and
//!   highlights the words a Mac read. The words also join the PDF's text in
//!   search (`pdf_text::sync_file` does that on every write of the file).
//!
//! The same rules as the PDFs' own text hold: nothing is offered or kept for
//! a sealed file, a PDF only a locked note links, or bytes that changed
//! since.
//!
//! Locked notes: the words of a scan are the PDF's text, in plain text in
//! the file's document, which syncs. So when a note is locked (here, or on
//! another device, arriving by sync), the PDFs it linked are remembered on
//! this device (`pdf_locked_files`, file ids only), and the words of any of
//! them that no note outside a lock links any more are dropped in the same
//! transaction: the field goes with a new version of the file, which syncs,
//! its search rows and kept pages go, and so do past versions of the file
//! that held the words. Such a file never takes the words again while that
//! holds, whoever writes them (a reader that finished meanwhile, sync, the
//! API). Unlocking the note links it from a readable note again, and the
//! scan is read again ([`Store::pdfs_to_ocr`]). Sealing the words with the
//! note's key instead is step 12 (#44).

use kanso_core::model::PDF_OCR_FIELD;
use kanso_core::{DocType, now_ms};
use kanso_pdf::{OcrPage, PdfOcr};
use rusqlite::params;
use serde_json::{Value, json};
use std::collections::HashSet;

use crate::pdf_text::{is_plain_pdf, shown_among};
use crate::write::{Synced, load_row, next_rev, store_row};
use crate::{Change, Origin, Put, Result, Store, StoreError, schema};

/// The tables, for emptying them with the library.
pub(crate) const TABLES: [&str; 2] = ["pdf_ocr_pending", "pdf_locked_files"];

/// SQL for "this page has no text": empty once white space is trimmed.
const BLANK: &str = "trim(p.text, ' ' || char(9, 10, 13)) = ''";

/// A PDF with scanned pages waiting to be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PdfToOcr {
    pub id: String,
    pub md5: String,
}

/// How far reading a PDF's scanned pages has got.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OcrProgress {
    /// Pages with no text layer.
    pub scanned: u32,
    /// Of them, pages whose words are kept (here, or from another device).
    pub read: u32,
}

impl Store {
    /// Live PDFs that have pages with no text, whose scanned pages haven't
    /// been read for their current bytes, newest first, at most `limit`.
    /// Only PDFs whose text was extracted (so which pages are blank is
    /// known) and that a note that may be read links.
    pub fn pdfs_to_ocr(&self, limit: usize) -> Result<Vec<PdfToOcr>> {
        self.read(|conn| {
            if limit == 0 {
                return Ok(Vec::new());
            }
            let mut stmt = conn.prepare(&format!(
                "SELECT d.id, json_extract(d.json, '$.md5digest') FROM docs d
                 JOIN pdf_state s ON s.file_id = d.id
                 WHERE d.type = 'file' AND d.deleted = 0
                   AND lower(json_extract(d.json, '$.contentType')) = 'application/pdf'
                   AND json_extract(d.json, '$.kansoFileLock') IS NULL
                   AND s.md5 = json_extract(d.json, '$.md5digest')
                   AND json_extract(d.json, '$.{PDF_OCR_FIELD}.md5')
                       IS NOT json_extract(d.json, '$.md5digest')
                   AND EXISTS (SELECT 1 FROM pdf_page_text p
                               WHERE p.file_id = d.id AND {BLANK})
                 ORDER BY json_extract(d.json, '$.createdAt') DESC, d.id"
            ))?;
            let candidates = stmt
                .query_map([], |row| {
                    Ok(PdfToOcr {
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

    /// The pages (1-based) of PDF `id` that have no text and whose words
    /// aren't kept yet for the bytes with `md5`, in order.
    pub fn pdf_pages_to_ocr(&self, id: &str, md5: &str) -> Result<Vec<u32>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT p.page FROM pdf_page_text p
                 WHERE p.file_id = ?1 AND {BLANK}
                   AND p.page NOT IN (SELECT page FROM pdf_ocr_pending
                                      WHERE file_id = ?1 AND md5 = ?2)
                 ORDER BY p.page"
            ))?;
            let pages = stmt
                .query_map(params![id, md5], |row| row.get::<_, u32>(0))?
                .collect::<rusqlite::Result<_>>()?;
            Ok(pages)
        })
    }

    /// How far reading the scanned pages of PDF `id` has got.
    pub fn pdf_ocr_progress(&self, id: &str) -> Result<OcrProgress> {
        self.read(|conn| {
            let scanned: u32 = conn.query_row(
                &format!("SELECT COUNT(*) FROM pdf_page_text p WHERE p.file_id = ? AND {BLANK}"),
                [id],
                |row| row.get(0),
            )?;
            let read: u32 = conn.query_row(
                "SELECT COUNT(*) FROM pdf_ocr_pending WHERE file_id = ?",
                [id],
                |row| row.get(0),
            )?;
            Ok(OcrProgress { scanned, read })
        })
    }

    /// Keeps the words of one scanned page of PDF `id`, read from the bytes
    /// with `md5`. Reading the same page again replaces it. Nothing is kept,
    /// and `false` returned, when the file is gone, sealed, not a PDF, has
    /// other bytes now, or no note that may be read links it.
    pub fn keep_pdf_ocr_page(&self, id: &str, md5: &str, page: &OcrPage) -> Result<bool> {
        self.write(|tx, _| {
            if !current_pdf(tx, id, md5)? {
                return Ok(false);
            }
            tx.execute(
                "INSERT INTO pdf_ocr_pending (file_id, md5, page, json) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT (file_id, page) DO UPDATE SET md5 = excluded.md5, json = excluded.json",
                params![
                    id,
                    md5,
                    page.page,
                    serde_json::to_string(page).expect("a page serialises")
                ],
            )?;
            Ok(true)
        })
    }

    /// Writes every page kept for PDF `id` into its `kansoPdfOcr` as one
    /// change, read by `engine`, and clears the kept pages. A PDF whose
    /// scanned pages had no words is written too (with pages that have
    /// none), so it isn't read again until its bytes change. Whether it
    /// was written: not when the file is gone, changed or no longer
    /// linked; then the kept pages are dropped.
    pub fn finish_pdf_ocr(&self, id: &str, md5: &str, engine: &str) -> Result<bool> {
        let pages: Vec<OcrPage> = self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT page, json FROM pdf_ocr_pending WHERE file_id = ? AND md5 = ? ORDER BY page",
            )?;
            let rows = stmt
                .query_map(params![id, md5], |row| {
                    Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            // A kept page that no longer reads is a page with no words: it
            // was read, and is never read again in a loop.
            Ok(rows
                .iter()
                .map(|(page, json)| {
                    serde_json::from_str(json)
                        .unwrap_or_else(|_| OcrPage::new(*page, 1.0, 1.0, []))
                })
                .collect())
        })?;
        if pages.is_empty() {
            // Nothing was read since: nothing to write.
            return Ok(false);
        }
        let clear = || {
            self.write(|tx, _| {
                tx.execute("DELETE FROM pdf_ocr_pending WHERE file_id = ?", [id])?;
                Ok(())
            })
        };
        let Some(doc) = self.get(id)? else {
            clear()?;
            return Ok(false);
        };
        let fresh = doc.doc_type == DocType::File
            && is_plain_pdf(&doc.fields)
            && doc.fields.get("md5digest").and_then(Value::as_str) == Some(md5)
            && self
                .read(|conn| Ok(!shown_among(conn, HashSet::from([id.to_owned()]))?.is_empty()))?;
        if !fresh {
            clear()?;
            return Ok(false);
        }
        // Through the reader of the field, so the caps hold for what is
        // written as they do for what is read.
        let ocr = PdfOcr {
            md5: md5.to_owned(),
            engine: engine.to_owned(),
            at: now_ms(),
            pages,
        };
        let capped = PdfOcr::from_value(&ocr.to_value()).unwrap_or(ocr);
        let body = json!({"_id": id, "_rev": doc.rev, PDF_OCR_FIELD: capped.to_value()});
        match self.put(DocType::File, body, Put::Merge, Origin::Ui) {
            Ok(_) => {
                clear()?;
                Ok(true)
            }
            // Changed or deleted meanwhile: the next pass reads it again.
            Err(StoreError::Conflict | StoreError::NotFound) => Ok(false),
            Err(err) => Err(err),
        }
    }

    /// The words read from PDF `id`'s scanned pages, if the file has them for
    /// the bytes it has now. They come from this device or another.
    pub fn pdf_ocr(&self, id: &str) -> Result<Option<PdfOcr>> {
        let Some(doc) = self.get(id)? else {
            return Ok(None);
        };
        if doc.doc_type != DocType::File || !is_plain_pdf(&doc.fields) {
            return Ok(None);
        }
        let now = doc.fields.get("md5digest").and_then(Value::as_str);
        Ok(doc
            .fields
            .get(PDF_OCR_FIELD)
            .and_then(PdfOcr::from_value)
            .filter(|ocr| Some(ocr.md5.as_str()) == now))
    }
}

/// The files note `id` links until this write, for [`after_note_write`]:
/// all of them when the note is `newly_locked`, else those of them a lock
/// has been seen on (and only when there are such files). `None` when there
/// is nothing to look at.
pub(crate) fn files_before(
    conn: &rusqlite::Connection,
    id: &str,
    newly_locked: bool,
) -> Result<Option<Vec<String>>> {
    let sql = if newly_locked {
        "SELECT file_id FROM note_files WHERE note_id = ?"
    } else {
        if !conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM pdf_locked_files)",
            [],
            |row| row.get::<_, bool>(0),
        )? {
            return Ok(None);
        }
        "SELECT f.file_id FROM note_files f JOIN pdf_locked_files l ON l.file_id = f.file_id
         WHERE f.note_id = ?"
    };
    let files: Vec<String> = conn
        .prepare_cached(sql)?
        .query_map([id], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok((!files.is_empty()).then_some(files))
}

/// After a note's write: `files` are what it linked before
/// ([`files_before`]). When the note was just locked, they are remembered as
/// linked by a locked note; then the words of any of them that no note
/// outside a lock links any more are dropped.
pub(crate) fn after_note_write(
    tx: &rusqlite::Transaction,
    files: &[String],
    newly_locked: bool,
    changes: &mut Vec<Change>,
) -> Result<()> {
    for file in files {
        if newly_locked {
            let pdf: bool = tx.query_row(
                "SELECT EXISTS (SELECT 1 FROM docs WHERE id = ? AND type = 'file'
                 AND lower(json_extract(json, '$.contentType')) = 'application/pdf')",
                [file],
                |row| row.get(0),
            )?;
            if !pdf {
                continue;
            }
            tx.execute(
                "INSERT OR IGNORE INTO pdf_locked_files (file_id) VALUES (?)",
                [file],
            )?;
        }
        drop_if_locked_only(tx, file, changes)?;
    }
    Ok(())
}

/// After any write of file `id` (`fields` is `None` once it is deleted): a
/// PDF a locked note links, that no other note links, never keeps words
/// read from its scans, whoever wrote them.
pub(crate) fn guard_file(
    tx: &rusqlite::Transaction,
    id: &str,
    fields: Option<&serde_json::Map<String, Value>>,
    changes: &mut Vec<Change>,
) -> Result<()> {
    let Some(fields) = fields else {
        tx.execute("DELETE FROM pdf_locked_files WHERE file_id = ?", [id])?;
        return Ok(());
    };
    if !fields.contains_key(PDF_OCR_FIELD) {
        return Ok(());
    }
    let marked: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM pdf_locked_files WHERE file_id = ?)",
        [id],
        |row| row.get(0),
    )?;
    if marked {
        drop_if_locked_only(tx, id, changes)?;
    }
    Ok(())
}

/// Whether a live note that isn't locked (itself or by its notebook) links
/// file `id`. A note in the trash counts: its text is plain too.
fn linked_unlocked(conn: &rusqlite::Connection, id: &str) -> Result<bool> {
    Ok(conn.query_row(
        concat!(
            "SELECT EXISTS (SELECT 1 FROM note_files f JOIN docs d ON d.id = f.note_id
             WHERE f.file_id = ? AND d.type = 'note' AND d.deleted = 0 AND NOT ",
            schema::note_locked!("d."),
            ")"
        ),
        [id],
        |row| row.get(0),
    )?)
}

/// Drops the words read from PDF `id`'s scans when no note outside a lock
/// links it: its `kansoPdfOcr` (with a new version of the file, so the
/// drop syncs), its kept pages and extracted text, its search row, and the
/// past versions of the file that held the words.
fn drop_if_locked_only(
    tx: &rusqlite::Transaction,
    id: &str,
    changes: &mut Vec<Change>,
) -> Result<()> {
    let Some(prev) = load_row(tx, id)? else {
        return Ok(());
    };
    if prev.doc_type != DocType::File
        || prev.deleted
        || !is_plain_pdf(&prev.fields)
        || linked_unlocked(tx, id)?
    {
        return Ok(());
    }
    crate::pdf_text::forget(tx, id)?;
    if prev.fields.contains_key(PDF_OCR_FIELD) {
        let mut fields = prev.fields.clone();
        fields.remove(PDF_OCR_FIELD);
        let rev = next_rev(Some(&prev), &Value::Object(fields.clone()))?;
        // Mirrors the file's search row again, without the words.
        store_row(
            tx,
            id,
            DocType::File,
            &rev,
            false,
            &fields,
            Some(&prev),
            Synced::Keep,
            Origin::Ui,
            changes,
        )?;
    } else {
        crate::ocr::write_file(tx, id, Some(&prev.fields))?;
    }
    tx.execute(
        &format!(
            "DELETE FROM revisions WHERE doc_id = ?
             AND json_extract(json, '$.{PDF_OCR_FIELD}') IS NOT NULL"
        ),
        [id],
    )?;
    crate::write::set_forgot_text();
    Ok(())
}

/// Whether file `id` is a live, plain PDF with the bytes of `md5` that a
/// note that may be read links.
fn current_pdf(conn: &rusqlite::Connection, id: &str, md5: &str) -> Result<bool> {
    let json: Option<String> = conn
        .query_row(
            "SELECT json FROM docs WHERE id = ? AND type = 'file' AND deleted = 0",
            [id],
            |row| row.get(0),
        )
        .ok();
    let Some(Value::Object(fields)) = json.and_then(|j| serde_json::from_str(&j).ok()) else {
        return Ok(false);
    };
    Ok(is_plain_pdf(&fields)
        && fields.get("md5digest").and_then(Value::as_str) == Some(md5)
        && !shown_among(conn, HashSet::from([id.to_owned()]))?.is_empty())
}

#[cfg(test)]
mod tests;
