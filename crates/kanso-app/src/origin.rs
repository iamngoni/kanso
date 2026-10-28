//! Where a theme came from: compiled in, the person's own, or the gallery.
//!
//! The distinction is shown as a badge and decides whether the gallery may
//! update a theme. A person's own files and built-ins are never touched by
//! the gallery; only a theme installed under `<data>/gallery/` is.

pub use kanso_engine::icons::model::Origin;
