//! What the typed-date hint and the `@` date list give the phone's editor.

/// A date and time found at the end of an open task's words, and what
/// tapping the hint writes for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhenHint {
    /// Where the phrase starts, in UTF-16 units of the line.
    pub start: u32,
    /// Where it ends, in UTF-16 units of the line.
    pub end: u32,
    /// How it reads to a person: `Fri 14 Nov, 15:00 · every week`.
    pub label: String,
    /// What replaces the phrase when the hint is tapped:
    /// `@2026-11-14T15:00 every week`.
    pub canonical: String,
    /// Whether the phrase repeats.
    pub repeats: bool,
}

/// One row of the `@` list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateChoice {
    /// `Tomorrow · Wed 21 Oct`, or `Every week`.
    pub label: String,
    /// The date a day row stands for, as it is written (`2026-10-21`);
    /// `None` for a repeat.
    pub detail: Option<String>,
    /// What the row writes in place of what was typed.
    pub insert: String,
}

/// The rows of the `@` list for the caret's place in a task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateChoices {
    /// Where the rows' text goes, in UTF-16 units of the text before the
    /// caret; it ends at the caret.
    pub start: u32,
    pub choices: Vec<DateChoice>,
}
