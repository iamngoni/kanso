//! The CRDT a shared note lives in: text for a Markdown note ([`CollabDoc`]),
//! items for a canvas ([`CanvasDoc`]). Which one is decided when the note is
//! shared and recorded in its row and in the owner-signed member list, so a
//! body that later fails to parse cannot change what the engine merges.

use kanso_store::SharedKind;

use crate::canvas_doc::CanvasDoc;
use crate::doc::{CollabDoc, DocError};

/// A loaded shared note's document.
#[derive(Debug)]
pub enum SharedDoc {
    Text(CollabDoc),
    Canvas(CanvasDoc),
}

impl SharedDoc {
    /// An empty document of `kind`.
    pub fn new(kind: SharedKind) -> SharedDoc {
        match kind {
            SharedKind::Text => SharedDoc::Text(CollabDoc::new()),
            SharedKind::Canvas => SharedDoc::Canvas(CanvasDoc::new()),
        }
    }

    /// A document of `kind` from its encoded state.
    pub fn load(kind: SharedKind, state: &[u8]) -> Result<SharedDoc, DocError> {
        Ok(match kind {
            SharedKind::Text => SharedDoc::Text(CollabDoc::load(state)?),
            SharedKind::Canvas => SharedDoc::Canvas(CanvasDoc::load(state)?),
        })
    }

    pub fn kind(&self) -> SharedKind {
        match self {
            SharedDoc::Text(_) => SharedKind::Text,
            SharedDoc::Canvas(_) => SharedKind::Canvas,
        }
    }

    pub fn encode_state(&self) -> Vec<u8> {
        match self {
            SharedDoc::Text(doc) => doc.encode_state(),
            SharedDoc::Canvas(doc) => doc.encode_state(),
        }
    }

    /// The text document, for a text note.
    pub fn text(&self) -> Option<&CollabDoc> {
        match self {
            SharedDoc::Text(doc) => Some(doc),
            SharedDoc::Canvas(_) => None,
        }
    }

    /// The canvas document, for a canvas.
    pub fn canvas(&self) -> Option<&CanvasDoc> {
        match self {
            SharedDoc::Canvas(doc) => Some(doc),
            SharedDoc::Text(_) => None,
        }
    }

    /// The note's title, whichever kind it is.
    pub fn title(&self) -> String {
        match self {
            SharedDoc::Text(doc) => doc.text(crate::doc::Field::Title),
            SharedDoc::Canvas(doc) => doc.title(),
        }
    }

    /// Merges an update. Whether it changed anything the note shows.
    pub fn apply(&self, update: &[u8]) -> Result<bool, DocError> {
        match self {
            SharedDoc::Text(doc) => doc.apply(update).map(|merged| !merged.is_empty()),
            SharedDoc::Canvas(doc) => doc.apply(update),
        }
    }
}
