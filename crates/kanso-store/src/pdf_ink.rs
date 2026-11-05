//! The attachment that holds a PDF's pencil ink once it outgrows the note
//! (issue #43, step 10).
//!
//! Small amounts of ink are part of the annotation block. A page of
//! handwriting is tens of thousands of points, so past about 100 KB the ink
//! goes into an attachment of its own (`application/vnd.kanso.ink+json`) that
//! the block names with `inkFile`. The note's text and its history stay
//! small, and the PDF is untouched either way.
//!
//! * [`read_ink_file`] takes the strokes in; a reader asks for it whenever the
//!   note changes.
//! * [`publish_ink`] writes them out before the block is saved. It first unites
//!   what the store already has (another device may have drawn too), so a
//!   save never loses a stroke made elsewhere.
//!
//! The attachment is owned by the note through the link the block holds
//! (`inkdrop://file:<id>`), so it follows the note into the trash, through
//! sync and out with Empty Trash like any other attachment. The desktop's
//! reader and the phone's list both write it through here, so the two ends
//! agree.

use base64::Engine as _;
use kanso_core::DocType;
use kanso_pdf::ink::{INK_FILE_TYPE, InkFile};
use kanso_pdf::{Annotations, PdfError};

use crate::{Origin, Put, Store, StoreError};

/// The attachment's name.
const NAME: &str = "pencil-marks.json";

/// What the store holds of an ink attachment.
#[derive(Debug, PartialEq)]
pub enum Held {
    /// There is no such attachment (yet): it may still be arriving.
    Missing,
    /// It is there but cannot be read.
    Unreadable(String),
    /// Its strokes, and the revision of its document.
    Found(Box<InkFile>, Option<String>),
}

/// Reads the ink attachment `id` (`file:<id>`).
pub fn read_ink_file(store: &Store, id: &str) -> Held {
    let rev = store.file_meta(id).ok().flatten().and_then(|m| m.rev);
    match store.attachment(id) {
        Ok(Some((_, attachment))) => match InkFile::from_bytes(&attachment.data) {
            Ok(file) => Held::Found(Box::new(file), rev),
            Err(error) => Held::Unreadable(error.to_string()),
        },
        Ok(None) => Held::Missing,
        Err(error) => Held::Unreadable(error.to_string()),
    }
}

/// Reads the attachment the block names, if it names one, into
/// `annotations`. Whether it was read.
pub fn load_ink(store: &Store, annotations: &mut Annotations) -> Result<bool, String> {
    let Some(id) = annotations.ink_file_id() else {
        return Ok(false);
    };
    match read_ink_file(store, &id) {
        Held::Found(file, _) => annotations
            .absorb_ink(&file)
            .map(|()| true)
            .map_err(|e: PdfError| e.to_string()),
        Held::Missing => Ok(false),
        Held::Unreadable(why) => Err(format!("The pencil marks could not be read: {why}")),
    }
}

/// Writes the ink to its attachment when it has to live there, and has
/// `annotations` name it. Does nothing for ink that fits the block. The
/// attachment's strokes and `annotations`' are united first; the united ink
/// is what is written and what `annotations` keep. `body` is the note's text
/// now: an attachment its block already names is the one written, so a
/// second is never made beside it.
pub fn publish_ink(store: &Store, annotations: &mut Annotations, body: &str) -> Result<(), String> {
    annotations.adopt_ink_file(body);
    if !annotations.needs_companion() {
        return Ok(());
    }
    let existing = annotations.ink_file_id();
    if let Some(id) = &existing {
        match read_ink_file(store, id) {
            Held::Found(file, _) => {
                annotations
                    .absorb_ink(&file)
                    .map_err(|e: PdfError| e.to_string())?;
            }
            Held::Missing if annotations.ink_pending => {
                return Err(kanso_pdf::highlights::INK_ARRIVING.to_owned());
            }
            Held::Missing => {}
            Held::Unreadable(why) => {
                return Err(format!("The pencil marks could not be read: {why}"));
            }
        }
    }
    let file = annotations
        .companion()
        .ok_or_else(|| "There is no ink to save.".to_owned())?;
    let bytes = file.to_bytes().map_err(|e| e.to_string())?;
    let id = write(store, existing.as_deref(), &bytes)?;
    annotations.set_ink_file(&id);
    Ok(())
}

/// Stores the bytes as the ink attachment: replacing `existing` when there
/// is one (retrying once if it moved on meanwhile), else as a new one.
fn write(store: &Store, existing: Option<&str>, bytes: &[u8]) -> Result<String, String> {
    let mut doc = serde_json::json!({
        "name": NAME,
        "contentType": INK_FILE_TYPE,
        "publicIn": [],
        "_attachments": {"index": {
            "content_type": INK_FILE_TYPE,
            "data": base64::engine::general_purpose::STANDARD.encode(bytes),
        }},
    });
    for _ in 0..2 {
        if let Some(id) = existing {
            doc["_id"] = id.into();
            match store.file_meta(id) {
                Ok(Some(meta)) => {
                    if let Some(rev) = meta.rev {
                        doc["_rev"] = rev.into();
                    }
                }
                Ok(None) => {
                    if let Some(fields) = doc.as_object_mut() {
                        fields.remove("_rev");
                    }
                }
                Err(error) => return Err(error.to_string()),
            }
        }
        match store.put(DocType::File, doc.clone(), Put::Upsert, Origin::Ui) {
            Ok(written) => return Ok(written.id),
            Err(StoreError::Conflict) if existing.is_some() => continue,
            Err(error) => return Err(format!("Couldn't save the pencil marks: {error}")),
        }
    }
    Err("Couldn't save the pencil marks: they changed while being saved.".to_owned())
}
