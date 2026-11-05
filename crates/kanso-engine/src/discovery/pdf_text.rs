//! Text in PDFs: the words on each page of a PDF attachment, extracted on
//! this device as soon as the PDF arrives (attached, imported or synced),
//! so search finds a note by its PDFs without opening anything (issue #43,
//! step 7).
//!
//! The extraction belongs to the PDF worker (step 3), behind
//! [`PdfTextSource`]. This is the queue around it: [`next_unindexed`] says
//! which PDF is next, [`index_next`] reads it with a source and keeps its
//! pages in the store (`kanso-store`'s `pdf_text.rs`), which mirrors them
//! into search. PDFs are indexed locally on every device and never synced.
//!
//! Only PDFs a note that may be read links are offered: not those only a
//! locked note, a note in the trash or a note tagged `#ai-exclude` links
//! (the store decides). Each pass first forgets the PDFs no such note links
//! any more.

use kanso_store::Store;

/// Extracts the text of a PDF. Step 3's worker implements it.
pub trait PdfTextSource: Send + Sync {
    /// Names the extractor in the store (for example `"pdfium"`).
    fn engine(&self) -> &'static str;
    /// The text of each page of `pdf`, page 1 first; a page without a text
    /// layer is an empty string. The error says whether the PDF or the
    /// reader is to blame.
    fn extract(&self, pdf: &[u8]) -> Result<Vec<String>, ExtractError>;
}

/// Why a PDF's text wasn't read.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ExtractError {
    /// The PDF itself can't be read: it is encrypted, damaged, too large or
    /// too slow to read. It is kept as having no text, and not tried again
    /// until its bytes change.
    #[error("{0}")]
    Document(String),
    /// The reader failed, not the PDF: the worker wouldn't start, it
    /// crashed, or the read was stopped. Nothing is kept, so the PDF is read
    /// again later; a source that can't tell must say this, not
    /// [`ExtractError::Document`], or one bad moment empties search.
    #[error("{0}")]
    Reader(String),
}

/// What one [`index_next`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Indexed {
    /// No PDF is waiting.
    Nothing,
    /// The next PDF was read, or kept as having no text. `waiting` PDFs
    /// were waiting, this one included.
    Read { waiting: usize },
    /// The reader failed; nothing was kept and `waiting` PDFs still wait.
    /// Try again later (the app backs off).
    ReaderFailed { waiting: usize, error: String },
}

impl Indexed {
    /// How many PDFs were waiting, the one just tried included.
    pub fn waiting(&self) -> usize {
        match self {
            Indexed::Nothing => 0,
            Indexed::Read { waiting } | Indexed::ReaderFailed { waiting, .. } => *waiting,
        }
    }
}

/// Where indexing stands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Off,
    /// Reading: how many PDFs are left.
    Indexing { left: usize },
    /// Every PDF is indexed.
    Idle,
    /// The reader failed; `left` PDFs wait for it to be tried again.
    Waiting { left: usize },
}

/// A PDF waiting to be read: its file, the MD5 of the bytes (kept with the
/// pages, so new bytes are read again) and the bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PdfToRead {
    pub id: String,
    pub md5: String,
    pub bytes: Vec<u8>,
    /// How many PDFs are waiting, this one included.
    pub waiting: usize,
}

/// How many PDFs are asked of the store at once.
const LOOK_AHEAD: usize = 64;

/// The next PDF to index, newest first, with its bytes; `None` when none is
/// waiting. A PDF whose bytes are gone is kept as having no text, so it is
/// passed over.
pub fn next_unindexed(store: &Store) -> kanso_store::Result<Option<PdfToRead>> {
    store.forget_unlinked_pdfs()?;
    let waiting = store.pdfs_to_index(LOOK_AHEAD)?;
    let count = waiting.len();
    for (ix, pdf) in waiting.into_iter().enumerate() {
        match store.attachment(&pdf.id)? {
            Some((_, attachment)) => {
                return Ok(Some(PdfToRead {
                    id: pdf.id,
                    md5: pdf.md5,
                    bytes: attachment.data,
                    waiting: count - ix,
                }));
            }
            None => {
                store.set_pdf_text(&pdf.id, &pdf.md5, &[], "none")?;
            }
        }
    }
    Ok(None)
}

/// Keeps `pages` as the text of PDF `id`, read from the bytes with `md5`.
/// Whether it was kept: not when the file is gone, its bytes changed, or a
/// note that may be read no longer links it.
pub fn keep(
    store: &Store,
    id: &str,
    md5: &str,
    pages: &[String],
    engine: &str,
) -> kanso_store::Result<bool> {
    store.set_pdf_text(id, md5, pages, engine)
}

/// How many PDFs are waiting to be indexed.
pub fn waiting(store: &Store) -> kanso_store::Result<usize> {
    Ok(store.pdfs_to_index(usize::MAX)?.len())
}

/// Indexes the next PDF with `source`, if there is one. A PDF that can't be
/// read ([`ExtractError::Document`]) is kept with no text, so it isn't tried
/// again until its bytes change. When the reader fails
/// ([`ExtractError::Reader`]) nothing is kept: the PDF still waits.
pub fn index_next(store: &Store, source: &dyn PdfTextSource) -> kanso_store::Result<Indexed> {
    let Some(pdf) = next_unindexed(store)? else {
        return Ok(Indexed::Nothing);
    };
    let pages = match source.extract(&pdf.bytes) {
        Ok(pages) => pages,
        Err(ExtractError::Document(err)) => {
            eprintln!("kanso: couldn't read the text in {}: {err}", pdf.id);
            Vec::new()
        }
        Err(ExtractError::Reader(error)) => {
            return Ok(Indexed::ReaderFailed {
                waiting: pdf.waiting,
                error,
            });
        }
    };
    keep(store, &pdf.id, &pdf.md5, &pages, source.engine())?;
    Ok(Indexed::Read {
        waiting: pdf.waiting,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use kanso_core::DocType;
    use kanso_store::{Origin, Put};
    use proptest::prelude::*;
    use serde_json::json;
    use std::sync::Mutex;

    /// Says each PDF's pages as the text after its `%PDF-` header split on
    /// `|`, and counts; or fails with the error given.
    struct Fake(Mutex<usize>, Option<ExtractError>);

    impl PdfTextSource for Fake {
        fn engine(&self) -> &'static str {
            "fake"
        }
        fn extract(&self, pdf: &[u8]) -> Result<Vec<String>, ExtractError> {
            *self.0.lock().unwrap() += 1;
            if let Some(err) = &self.1 {
                return Err(err.clone());
            }
            let text = String::from_utf8_lossy(&pdf[5..]).into_owned();
            Ok(text.split('|').map(str::to_owned).collect())
        }
    }

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        (dir, store)
    }

    /// A file of `%PDF-{body}`, and a note in a notebook that links it.
    fn attach(store: &Store, body: &str, content_type: &str) -> (String, String) {
        let bytes = format!("%PDF-{body}");
        let id = store
            .put(
                DocType::File,
                json!({
                    "name": "a.pdf",
                    "contentType": content_type,
                    "_attachments": {"index": {
                        "content_type": content_type,
                        "data": base64::engine::general_purpose::STANDARD.encode(bytes),
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
                json!({"title": "Shows", "body": format!("[a](inkdrop://{id})"), "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        (id, note)
    }

    #[test]
    fn pdfs_are_read_once_and_only_pdfs() {
        let (_dir, store) = store();
        let (pdf, _) = attach(&store, "alpha|beta", "application/pdf");
        attach(&store, "x", "image/png");
        let source = Fake(Mutex::new(0), None);
        assert_eq!(
            index_next(&store, &source).unwrap(),
            Indexed::Read { waiting: 1 }
        );
        assert_eq!(index_next(&store, &source).unwrap(), Indexed::Nothing);
        assert_eq!(*source.0.lock().unwrap(), 1);
        assert_eq!(store.pdf_pages(&pdf).unwrap(), vec!["alpha", "beta"]);
    }

    #[test]
    fn an_unreadable_pdf_is_not_tried_forever() {
        let (_dir, store) = store();
        let (pdf, _) = attach(&store, "x", "application/pdf");
        let source = Fake(
            Mutex::new(0),
            Some(ExtractError::Document("damaged".into())),
        );
        assert_eq!(
            index_next(&store, &source).unwrap(),
            Indexed::Read { waiting: 1 }
        );
        assert_eq!(index_next(&store, &source).unwrap(), Indexed::Nothing);
        assert!(store.pdf_indexed(&pdf).unwrap());
        assert!(store.pdf_pages(&pdf).unwrap().is_empty());
    }

    /// A reader that fails (no worker, a crash) keeps nothing: the PDF is
    /// still waiting, and is read once the reader works again.
    #[test]
    fn a_failed_reader_keeps_nothing_and_the_pdf_is_read_later() {
        let (_dir, store) = store();
        let (pdf, _) = attach(&store, "later|words", "application/pdf");
        let broken = Fake(
            Mutex::new(0),
            Some(ExtractError::Reader("no worker".into())),
        );
        for _ in 0..3 {
            assert_eq!(
                index_next(&store, &broken).unwrap(),
                Indexed::ReaderFailed {
                    waiting: 1,
                    error: "no worker".into()
                }
            );
            assert!(!store.pdf_indexed(&pdf).unwrap());
            assert_eq!(waiting(&store).unwrap(), 1);
        }
        let working = Fake(Mutex::new(0), None);
        assert_eq!(
            index_next(&store, &working).unwrap(),
            Indexed::Read { waiting: 1 }
        );
        assert_eq!(store.pdf_pages(&pdf).unwrap(), vec!["later", "words"]);
    }

    #[test]
    fn a_worker_can_read_between_the_two_calls() {
        let (_dir, store) = store();
        let (pdf, _) = attach(&store, "one|two", "application/pdf");
        assert_eq!(waiting(&store).unwrap(), 1);
        let next = next_unindexed(&store).unwrap().unwrap();
        assert_eq!(next.id, pdf);
        assert!(!keep(&store, &pdf, "not-the-md5", &["x".into()], "w").unwrap());
        assert!(keep(&store, &pdf, &next.md5, &["x".into()], "w").unwrap());
        assert!(next_unindexed(&store).unwrap().is_none());
    }

    #[test]
    fn a_pdf_unlinked_since_is_forgotten_by_the_next_pass() {
        let (_dir, store) = store();
        let (pdf, note) = attach(&store, "gone", "application/pdf");
        let source = Fake(Mutex::new(0), None);
        index_next(&store, &source).unwrap();
        assert!(!store.pdf_pages(&pdf).unwrap().is_empty());
        let rev = store.get(&note).unwrap().unwrap().rev;
        store
            .put(
                DocType::Note,
                json!({"_id": note, "_rev": rev, "body": "unlinked"}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        assert_eq!(index_next(&store, &source).unwrap(), Indexed::Nothing);
        assert!(store.pdf_pages(&pdf).unwrap().is_empty());
        assert!(!store.pdf_indexed(&pdf).unwrap());
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]

        /// However many PDFs there are, indexing them one by one reads each
        /// once, counting down what is left.
        #[test]
        fn prop_every_pdf_is_indexed_exactly_once(count in 0usize..5) {
            let (_dir, store) = store();
            for n in 0..count {
                attach(&store, &format!("wordnumber{n}"), "application/pdf");
            }
            let source = Fake(Mutex::new(0), None);
            let mut left = count;
            while left > 0 {
                prop_assert_eq!(index_next(&store, &source).unwrap(), Indexed::Read { waiting: left });
                left -= 1;
            }
            prop_assert_eq!(index_next(&store, &source).unwrap(), Indexed::Nothing);
            prop_assert_eq!(*source.0.lock().unwrap(), count);
            prop_assert_eq!(waiting(&store).unwrap(), 0);
        }
    }
}
