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
//! [`measure`] says what the model costs on a device.
//!
//! The store keeps the vectors and decides who may be read (`kanso-store`'s
//! `embed.rs`): a note that is locked or tagged `#ai-exclude` is never
//! embedded, and an image only such a note shows is never read.

pub mod graph;
pub mod indexer;
pub mod meaning;
pub mod measure;
pub mod model_fetch;
pub mod ocr;
pub mod related;
