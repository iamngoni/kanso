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
//! sync and out with Empty Trash like any other attachment, and a lock seals
//! it with the note's other files (#44). The desktop's reader and the phone's
//! list both write it through here, so the two ends agree.
//!
//! A new attachment's id comes from the note and the PDF
//! ([`companion_id`]), so two devices that move the same PDF's ink out of
//! the note at once write one attachment, whose versions sync unites
//! stroke by stroke, rather than two of which the note keeps one.
//!
//! How the bytes are read and written is an [`InkFiles`]: [`Plain`] as the
//! store holds them; an open locked note's reader passes one that opens and
//! writes them sealed with the note's file keys.

use base64::Engine as _;
use kanso_core::DocType;
use kanso_pdf::ink::{INK_FILE_TYPE, InkFile};
use kanso_pdf::{Annotations, PdfError};

use crate::{Origin, Put, Store, StoreError};

/// The attachment's name.
pub const INK_FILE_NAME: &str = "pencil-marks.json";

/// The attachment's content type.
pub const INK_FILE_CONTENT_TYPE: &str = INK_FILE_TYPE;

/// What is said when the ink is sealed with a lock whose key isn't held.
pub const INK_SEALED: &str =
    "These pencil marks are sealed with a locked note. Unlock it to read or change them.";

/// What the store holds of an ink attachment.
#[derive(Debug, PartialEq)]
pub enum Held {
    /// There is no such attachment (yet): it may still be arriving.
    Missing,
    /// It is sealed with a lock whose key the reader doesn't hold.
    Sealed,
    /// It is there but cannot be read.
    Unreadable(String),
    /// Its strokes, and the revision of its document.
    Found(Box<InkFile>, Option<String>),
}

/// An ink attachment's bytes, as a reader gets them.
#[derive(Debug, PartialEq)]
pub enum Fetched {
    Bytes(Vec<u8>),
    /// Sealed, and the reader has no key that opens it.
    Sealed,
    /// No such attachment, or its bytes haven't arrived.
    Missing,
    Failed(String),
}

/// How the ink attachment's bytes are read and written.
pub trait InkFiles {
    /// The bytes of attachment `id`.
    fn fetch(&self, store: &Store, id: &str) -> Fetched;
    /// Stores `bytes` as the ink attachment: over `existing` when there is
    /// one, else as `fresh`. The id written.
    fn write(
        &self,
        store: &Store,
        existing: Option<&str>,
        fresh: &str,
        bytes: &[u8],
    ) -> Result<String, String>;
}

/// The attachment as the store holds it: a sealed one is never opened, and
/// never written over.
pub struct Plain;

impl InkFiles for Plain {
    fn fetch(&self, store: &Store, id: &str) -> Fetched {
        match store.attachment_state(id) {
            Ok(crate::AttachmentState::Sealed) => Fetched::Sealed,
            Ok(crate::AttachmentState::Missing) => Fetched::Missing,
            Ok(crate::AttachmentState::Plain) => match store.attachment(id) {
                Ok(Some((_, attachment))) => Fetched::Bytes(attachment.data),
                Ok(None) => Fetched::Missing,
                Err(error) => Fetched::Failed(error.to_string()),
            },
            Err(error) => Fetched::Failed(error.to_string()),
        }
    }

    fn write(
        &self,
        store: &Store,
        existing: Option<&str>,
        fresh: &str,
        bytes: &[u8],
    ) -> Result<String, String> {
        write(store, existing.unwrap_or(fresh), bytes)
    }
}

/// The id a new ink attachment of note `note_id` for PDF `pdf` (`file:…`)
/// gets: the same on every device, so two that move the ink out of the note
/// at once write one attachment.
pub fn companion_id(note_id: &str, pdf: &str) -> String {
    let digest = crate::Attachment::new(format!("{note_id}\n{pdf}").into_bytes()).md5_hex();
    format!("file:ink-{digest}")
}

/// Reads the ink attachment `id` (`file:<id>`) as the store holds it.
pub fn read_ink_file(store: &Store, id: &str) -> Held {
    read_ink_with(store, id, &Plain)
}

/// Reads the ink attachment `id` (`file:<id>`) through `files`.
pub fn read_ink_with(store: &Store, id: &str, files: &dyn InkFiles) -> Held {
    let rev = store.file_meta(id).ok().flatten().and_then(|m| m.rev);
    match files.fetch(store, id) {
        Fetched::Bytes(bytes) => match InkFile::from_bytes(&bytes) {
            Ok(file) => Held::Found(Box::new(file), rev),
            Err(error) => Held::Unreadable(error.to_string()),
        },
        Fetched::Sealed => Held::Sealed,
        Fetched::Missing => Held::Missing,
        Fetched::Failed(why) => Held::Unreadable(why),
    }
}

/// Reads the attachment the block names, if it names one, into
/// `annotations`. Whether it was read.
pub fn load_ink(store: &Store, annotations: &mut Annotations) -> Result<bool, String> {
    load_ink_with(store, annotations, &Plain)
}

/// [`load_ink`], reading through `files`.
pub fn load_ink_with(
    store: &Store,
    annotations: &mut Annotations,
    files: &dyn InkFiles,
) -> Result<bool, String> {
    let Some(id) = annotations.ink_file_id() else {
        return Ok(false);
    };
    match read_ink_with(store, &id, files) {
        Held::Found(file, _) => annotations
            .absorb_ink(&file)
            .map(|()| true)
            .map_err(|e: PdfError| e.to_string()),
        Held::Missing => Ok(false),
        Held::Sealed => Err(INK_SEALED.to_owned()),
        Held::Unreadable(why) => Err(format!("The pencil marks could not be read: {why}")),
    }
}

/// Writes the ink to its attachment when it has to live there, and has
/// `annotations` name it. Does nothing for ink that fits the block. The
/// attachment's strokes and `annotations`' are united first; the united ink
/// is what is written and what `annotations` keep. `body` is the note's text
/// now: an attachment its block already names is the one written, so a
/// second is never made beside it. `note_id` is the note's; a new attachment
/// takes its id from it ([`companion_id`]).
pub fn publish_ink(
    store: &Store,
    annotations: &mut Annotations,
    note_id: &str,
    body: &str,
) -> Result<(), String> {
    publish_ink_with(store, annotations, note_id, body, &Plain)
}

/// [`publish_ink`], reading and writing through `files`.
pub fn publish_ink_with(
    store: &Store,
    annotations: &mut Annotations,
    note_id: &str,
    body: &str,
    files: &dyn InkFiles,
) -> Result<(), String> {
    annotations.adopt_ink_file(body);
    if !annotations.needs_companion() {
        return Ok(());
    }
    let existing = annotations.ink_file_id();
    if let Some(id) = &existing {
        match read_ink_with(store, id, files) {
            Held::Found(file, _) => {
                annotations
                    .absorb_ink(&file)
                    .map_err(|e: PdfError| e.to_string())?;
            }
            Held::Missing if annotations.ink_pending => {
                return Err(kanso_pdf::highlights::INK_ARRIVING.to_owned());
            }
            Held::Missing => {}
            // Never written over: its strokes would be lost.
            Held::Sealed => return Err(INK_SEALED.to_owned()),
            Held::Unreadable(why) => {
                return Err(format!("The pencil marks could not be read: {why}"));
            }
        }
    }
    let file = annotations
        .companion()
        .ok_or_else(|| "There is no ink to save.".to_owned())?;
    let bytes = file.to_bytes().map_err(|e| e.to_string())?;
    let fresh = companion_id(note_id, annotations.file());
    let id = files.write(store, existing.as_deref(), &fresh, &bytes)?;
    annotations.set_ink_file(&id);
    Ok(())
}

/// Stores the bytes as ink attachment `id`, over what it holds (retrying
/// once if it moved on meanwhile). A new one that another device made
/// meanwhile is written over the same way, after its strokes were united
/// with these by the caller's read.
fn write(store: &Store, id: &str, bytes: &[u8]) -> Result<String, String> {
    let mut doc = serde_json::json!({
        "_id": id,
        "name": INK_FILE_NAME,
        "contentType": INK_FILE_TYPE,
        "publicIn": [],
        "_attachments": {"index": {
            "content_type": INK_FILE_TYPE,
            "data": base64::engine::general_purpose::STANDARD.encode(bytes),
        }},
    });
    for _ in 0..2 {
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
        match store.put(DocType::File, doc.clone(), Put::Upsert, Origin::Ui) {
            Ok(written) => return Ok(written.id),
            Err(StoreError::Conflict) => continue,
            Err(error) => return Err(format!("Couldn't save the pencil marks: {error}")),
        }
    }
    Err("Couldn't save the pencil marks: they changed while being saved.".to_owned())
}
