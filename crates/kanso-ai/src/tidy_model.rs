//! The shapes of a tidy-up (issue #193): what the model is asked to make of
//! a transcript, and what it gave back once its reply has been read.

/// What kind of talk a transcript is. A meeting or lecture gets the fixed
/// shape (Summary, Decisions, Action items, Notes); a short note gets a
/// title, paragraphs and tasks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TidyKind {
    Note,
    Meeting,
}

/// A decision or other line the model says the transcript supports, with the
/// words it points to. The words are checked against the transcript before
/// the line is kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quoted {
    pub text: String,
    /// Snippets copied from the transcript, as the model says it heard them.
    pub quotes: Vec<String>,
}

/// Something someone said they would do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub text: String,
    /// Who, only when a name was said.
    pub who: Option<String>,
    /// When, only when it was said, in the words said ("Tuesday").
    pub when: Option<String>,
    pub quotes: Vec<String>,
}

/// A stretch of the talk about one thing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub heading: String,
    pub paragraphs: Vec<String>,
}

/// A reply, read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tidy {
    pub title: Option<String>,
    pub summary: Vec<String>,
    pub decisions: Vec<Quoted>,
    pub actions: Vec<Action>,
    pub sections: Vec<Section>,
    /// Only the first part of a transcript longer than the cap was tidied.
    pub cut: bool,
}
