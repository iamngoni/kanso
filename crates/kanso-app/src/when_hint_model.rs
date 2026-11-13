//! What the editor's typed-date hint holds: the phrase it found on the
//! caret's task line and what it would become.

use std::ops::Range;

/// A date phrase at the end of the caret's task line (`fri 3pm every
/// week`), and the stored form Tab writes in its place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhenHint {
    /// The phrase, as byte offsets in the note's text.
    pub phrase: Range<usize>,
    /// The caret's line, as byte offsets in the note's text (without its
    /// line break). The hint shows while the caret is on it.
    pub line: Range<usize>,
    /// The stored form: `@2026-11-13T15:00 every week`.
    pub canonical: String,
    /// How it reads to a person: `Fri 13 Nov 2026, 15:00 · every week`.
    pub label: String,
}
