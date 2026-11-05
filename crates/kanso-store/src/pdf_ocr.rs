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

use kanso_core::model::PDF_OCR_FIELD;
use kanso_core::{DocType, now_ms};
use kanso_pdf::{OcrPage, PdfOcr};
use rusqlite::params;
use serde_json::{Value, json};
use std::collections::HashSet;

use crate::pdf_text::{is_plain_pdf, shown_among};
use crate::{Origin, Put, Result, Store, StoreError};

/// The tables, for emptying them with the library.
pub(crate) const TABLES: [&str; 1] = ["pdf_ocr_pending"];

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
                "SELECT json FROM pdf_ocr_pending WHERE file_id = ? AND md5 = ? ORDER BY page",
            )?;
            let rows = stmt
                .query_map(params![id, md5], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(rows
                .iter()
                .filter_map(|json| serde_json::from_str(json).ok())
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
