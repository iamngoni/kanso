//! What a paste puts in the note: a web address pasted over selected words
//! links them, formatted text from a browser or a document comes in as
//! Markdown, and an image becomes an attachment. All of it is decided in
//! `kanso_engine`, which the phones' editor and share sheet use as well.

// The text rules: links over selected words, HTML as Markdown, blocks of
// their own, `<img>` tags and their widths.
pub use kanso_engine::edit::paste::*;

// Attaching images and PDFs, and the address test.
pub use kanso_engine::attach::{attach_image, image_markdown, stored_as, too_large};
