//! The text layer (issue #43, step 5, part 1): from the characters a PDF page
//! holds to lines, blocks and a reading order; selection, copy, word and line
//! expansion; highlight quads; and Find.
//!
//! Everything here is pure. The PDFium worker (step 3) fills [`PageText`]; the
//! reader asks a [`TextDoc`] what is under the pointer and what to copy.

mod find;
mod layout;
pub mod model;
mod select;

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;

pub use find::FindIndex;
pub use layout::layout_page;
pub use model::{
    Block, CharPos, Glyph, HighlightParts, Line, PAGE_BREAK, PageLayout, PageText, Point,
    Selection, Sep, TextChar, TextDoc,
};
