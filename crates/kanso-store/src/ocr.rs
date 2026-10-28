//! Text read from image attachments, for the app's search.
//!
//! The text itself belongs to the file document, in `kansoOcr` (see
//! [`OCR_FIELD`]), so it syncs sealed with the file and a device that
//! can't read images still finds them by their words. `file_text` mirrors
//! it locally, kept current by every write of a file (from `store_row`),
//! filled again on open when it was built another way, and emptied with
//! the library.

use std::collections::HashSet;

use kanso_core::model::OCR_FIELD;
use kanso_core::tasks::linked_file_ids;
use kanso_core::{DocType, now_ms};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Map, Value, json};

use crate::{Origin, Put, Result, Store, StoreError, schema};

/// The table, for emptying it with the library.
pub(crate) const TABLES: [&str; 1] = ["file_text"];

/// The `meta` key holding the [`FILE_TEXT_VERSION`] `file_text` was built
/// with.
const FILE_TEXT_VERSION_KEY: &str = "index.file_text_version";
const FILE_TEXT_VERSION: &str = "1";

/// The most files one search term matches by their text: enough for any
/// real search, and a bound on the query it makes.
const MATCH_LIMIT: usize = 200;

/// Image types text is read from: the bitmaps, not SVG (whose words are
/// already text) or animated GIF frames.
pub const OCR_TYPES: [&str; 5] = [
    "image/png",
    "image/jpeg",
    "image/jpg",
    "image/heic",
    "image/heif",
];

/// An image whose text hasn't been read, or was read from other bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnreadImage {
    pub id: String,
    pub md5: String,
}

/// The text and the MD5 it was read from in a file's `kansoOcr`.
fn ocr_of(fields: &Map<String, Value>) -> Option<(&str, &str)> {
    let ocr = fields.get(OCR_FIELD)?.as_object()?;
    Some((ocr.get("text")?.as_str()?, ocr.get("md5")?.as_str()?))
}

/// Mirrors file `id`'s `kansoOcr` into `file_text`: its text, or none for
/// a deleted file or one without it.
pub(crate) fn write_file(
    conn: &Connection,
    id: &str,
    fields: Option<&Map<String, Value>>,
) -> Result<()> {
    match fields.and_then(ocr_of) {
        Some((text, md5)) => {
            conn.execute(
                "INSERT INTO file_text (file_id, md5, text) VALUES (?1, ?2, ?3)
                 ON CONFLICT (file_id) DO UPDATE SET md5 = excluded.md5, text = excluded.text",
                params![id, md5, text],
            )?;
        }
        None => {
            conn.execute("DELETE FROM file_text WHERE file_id = ?", [id])?;
        }
    }
    Ok(())
}

/// Fills `file_text` again from every live file when it was built another
/// way, or never.
pub(crate) fn rebuild_if_stale(conn: &mut Connection) -> Result<bool> {
    if schema::get_meta(conn, FILE_TEXT_VERSION_KEY)?.as_deref() == Some(FILE_TEXT_VERSION) {
        return Ok(false);
    }
    let tx = rusqlite::Transaction::new(conn, rusqlite::TransactionBehavior::Immediate)?;
    tx.execute("DELETE FROM file_text", [])?;
    let files: Vec<(String, String)> = tx
        .prepare("SELECT id, json FROM docs WHERE type = 'file' AND deleted = 0")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    for (id, json) in files {
        if let Ok(Value::Object(fields)) = serde_json::from_str(&json) {
            write_file(&tx, &id, Some(&fields))?;
        }
    }
    schema::set_meta(&tx, FILE_TEXT_VERSION_KEY, FILE_TEXT_VERSION)?;
    tx.commit()?;
    Ok(true)
}

/// The files whose read text contains `needle` (case-insensitive), at
/// most [`MATCH_LIMIT`].
pub(crate) fn files_matching(conn: &Connection, needle: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare_cached(
        "SELECT file_id FROM file_text WHERE kanso_contains(text, ?1) ORDER BY file_id LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![needle, MATCH_LIMIT as i64], |row| row.get(0))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

impl Store {
    /// Live image attachments of [`OCR_TYPES`] whose text hasn't been read
    /// from their current bytes, newest first, at most `limit`. Only images
    /// that at least one live note outside the trash shows, and that note
    /// isn't locked or tagged `#ai-exclude` (so its body is readable and
    /// allowed): an image only a locked note shows is never read.
    pub fn unread_images(&self, limit: usize) -> Result<Vec<UnreadImage>> {
        self.read(|conn| {
            let types = OCR_TYPES.map(|t| format!("'{t}'")).join(", ");
            let mut stmt = conn.prepare(&format!(
                "SELECT d.id, json_extract(d.json, '$.md5digest') FROM docs d
                 LEFT JOIN file_text t ON t.file_id = d.id
                 WHERE d.type = 'file' AND d.deleted = 0
                   AND lower(json_extract(d.json, '$.contentType')) IN ({types})
                   AND json_extract(d.json, '$.md5digest') IS NOT NULL
                   AND json_extract(d.json, '$.kansoFileLock') IS NULL
                   AND (t.md5 IS NULL OR t.md5 != json_extract(d.json, '$.md5digest'))
                 ORDER BY json_extract(d.json, '$.createdAt') DESC, d.id"
            ))?;
            let candidates = stmt
                .query_map([], |row| {
                    Ok(UnreadImage {
                        id: row.get(0)?,
                        md5: row.get(1)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            if candidates.is_empty() || limit == 0 {
                return Ok(Vec::new());
            }
            let mut wanted: HashSet<&str> = candidates.iter().map(|c| c.id.as_str()).collect();
            let mut shown: HashSet<String> = HashSet::new();
            // The bodies of the notes that may be read, once: each file
            // they embed that is waiting to be read is shown.
            let mut notes = conn.prepare(concat!(
                "SELECT ",
                schema::readable_body!("d."),
                " FROM docs d WHERE ",
                schema::discoverable!("d.")
            ))?;
            let mut rows = notes.query([])?;
            while let Some(row) = rows.next()? {
                let body: Option<String> = row.get(0)?;
                for id in linked_file_ids(body.as_deref().unwrap_or("")) {
                    if wanted.remove(id.as_str()) {
                        shown.insert(id);
                    }
                }
                if wanted.is_empty() {
                    break;
                }
            }
            Ok(candidates
                .into_iter()
                .filter(|c| shown.contains(&c.id))
                .take(limit)
                .collect())
        })
    }

    /// Keeps `text` as what image `id` says, read from the bytes with
    /// `md5`, in its `kansoOcr`. Nothing is written when the file is gone
    /// or its bytes changed since: it is read again. Returns whether it
    /// was kept.
    pub fn set_image_text(&self, id: &str, md5: &str, text: &str, engine: &str) -> Result<bool> {
        let Some(doc) = self.get(id)? else {
            return Ok(false);
        };
        if doc.doc_type != DocType::File
            || doc.fields.contains_key(kanso_core::model::FILE_LOCK_FIELD)
            || doc.fields.get("md5digest").and_then(Value::as_str) != Some(md5)
        {
            return Ok(false);
        }
        let body = json!({
            "_id": id,
            "_rev": doc.rev,
            OCR_FIELD: {"text": text, "md5": md5, "engine": engine, "at": now_ms()},
        });
        match self.put(DocType::File, body, Put::Merge, Origin::Ui) {
            Ok(_) => Ok(true),
            // Changed or deleted meanwhile: the next pass reads it again.
            Err(StoreError::Conflict | StoreError::NotFound) => Ok(false),
            Err(err) => Err(err),
        }
    }

    /// The text read from image `id`, if any.
    pub fn image_text(&self, id: &str) -> Result<Option<String>> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    "SELECT text FROM file_text WHERE file_id = ?",
                    [id],
                    |row| row.get(0),
                )
                .optional()?)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::Attachment;
    use base64::Engine as _;
    use kanso_core::model::FileLock;

    fn open() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    fn image(store: &Store, bytes: &[u8]) -> String {
        store
            .put(
                DocType::File,
                json!({
                    "name": "shot.png",
                    "contentType": "image/png",
                    "_attachments": {"index": {
                        "content_type": "image/png",
                        "data": base64::engine::general_purpose::STANDARD.encode(bytes),
                    }},
                }),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    /// A note in a notebook of its own that shows file `id`.
    fn show(store: &Store, id: &str, tags: &[&str]) -> String {
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        store
            .put(
                DocType::Note,
                json!({"title": "Shows", "body": format!("![](inkdrop://{id})"), "bookId": book, "tags": tags}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    #[test]
    fn read_text_is_kept_in_the_file_and_mirrored() {
        let (_dir, store) = open();
        let id = image(&store, b"one");
        show(&store, &id, &[]);
        let unread = store.unread_images(10).unwrap();
        assert_eq!(unread.len(), 1);
        let md5 = unread[0].md5.clone();
        // From other bytes: refused.
        assert!(!store.set_image_text(&id, "nope", "x", "test").unwrap());
        assert!(
            store
                .set_image_text(&id, &md5, "Quarterly Plan", "test")
                .unwrap()
        );
        assert!(store.unread_images(10).unwrap().is_empty());
        assert_eq!(
            store.image_text(&id).unwrap().as_deref(),
            Some("Quarterly Plan")
        );
        let doc = store.get(&id).unwrap().unwrap();
        assert_eq!(doc.fields[OCR_FIELD]["text"], "Quarterly Plan");
        // Rebuilt from the documents.
        store
            .read(|conn| {
                conn.execute("DELETE FROM meta WHERE key = ?", [FILE_TEXT_VERSION_KEY])?;
                conn.execute("DELETE FROM file_text", [])?;
                Ok(())
            })
            .unwrap();
        drop(store);
        let store = Store::open(_dir.path().join("kanso.db")).unwrap();
        assert_eq!(
            store.image_text(&id).unwrap().as_deref(),
            Some("Quarterly Plan")
        );
        store.delete(&id, Origin::Ui).unwrap();
        assert_eq!(store.image_text(&id).unwrap(), None);
    }

    #[test]
    fn a_sealed_png_is_not_sent_to_ocr() {
        let (_dir, store) = open();
        let id = image(&store, b"png bytes");
        show(&store, &id, &[]);
        let mut meta = store.file_meta(&id).unwrap().unwrap();
        let mut lock = FileLock::sealed("AAAAAAAAAAAAAAAAAAAAAA".into());
        lock.meta = true;
        FileLock::store(Some(&lock), &mut meta.extra);
        store
            .put_sealed_file(meta, Attachment::new(b"sealed container".to_vec()))
            .unwrap();

        assert!(store.unread_images(10).unwrap().is_empty());
    }

    #[test]
    fn the_apps_search_finds_notes_by_their_images_words() {
        let (_dir, store) = open();
        let id = image(&store, b"two");
        let shows = show(&store, &id, &[]);
        let book = store.get(&shows).unwrap().unwrap().fields["bookId"]
            .as_str()
            .unwrap()
            .to_owned();
        let md5 = store.unread_images(1).unwrap()[0].md5.clone();
        store
            .set_image_text(&id, &md5, "Whiteboard: launch checklist", "test")
            .unwrap();
        store
            .put(
                DocType::Note,
                json!({"_id": shows, "title": "Meeting"}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        store
            .put(
                DocType::Note,
                json!({"title": "Other", "body": "nothing here", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let find = |text: &str, ocr: bool| -> Vec<String> {
            store
                .search_summaries(&crate::SearchParams {
                    query: kanso_core::Query::parse(text),
                    ocr,
                    ..Default::default()
                })
                .unwrap()
                .into_iter()
                .map(|n| n.id)
                .collect()
        };
        assert_eq!(find("checklist", true), vec![shows.clone()]);
        assert_eq!(find("CHECKLIST meeting", true), vec![shows.clone()]);
        // The local API's search keeps to titles and bodies.
        assert!(find("checklist", false).is_empty());
        // Negated terms and fields don't look at images.
        assert_eq!(find("-checklist", true).len(), 2);
        assert!(find("title:checklist", true).is_empty());
    }
}
