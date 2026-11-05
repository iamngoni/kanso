//! Discovery: finding notes by how they link and by what they are about.
//!
//! [`graph`] is the note graph: which notes link to which, a layout of it
//! and the view transform it is drawn through. [`related`] is the embedding
//! model related notes use, with the words each of its states is told in;
//! [`model_fetch`] downloads a model once, checked against pinned hashes;
//! [`indexer`] keeps notes embedded, on whatever thread the app gives it.
//! [`meaning`] is what the embeddings say about the open note (tags to
//! suggest, a near twin) and the rule for which searches go by meaning;
//! [`ocr`] is reading the words in pictures through the platform's reader.
//! [`ink_ocr`] does the same for handwriting on canvases and sketches.
//! [`pdf_text`] is the queue that reads the words in PDFs through the PDF
//! worker, behind a trait; [`pdf_worker`] (the `pdf-worker` feature) is that
//! trait over the worker's client.
//! [`pdf_ocr`] reads the scanned pages of PDFs the same way, with the
//! platform's text reader.
//! [`measure`] says what the model costs on a device.
//!
//! The store keeps the vectors and decides who may be read (`kanso-store`'s
//! `embed.rs`): a note that is locked or tagged `#ai-exclude` is never
//! embedded, and an image only such a note shows is never read.

pub mod graph;
pub mod indexer;
pub mod ink_ocr;
pub mod meaning;
pub mod measure;
pub mod model_fetch;
pub mod ocr;
#[cfg(feature = "pdf-worker")]
pub mod pdf_ocr;
pub mod pdf_text;
#[cfg(feature = "pdf-worker")]
pub mod pdf_worker;
pub mod related;
pub mod resurface;
