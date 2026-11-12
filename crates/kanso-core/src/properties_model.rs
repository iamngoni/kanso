//! A front-matter entry as source text, for notes that keep their
//! properties one key at a time (see [`crate::properties::entry_sources`]).

/// One top-level front-matter entry: its key and everything after the
/// key's colon, as written (the value, an inline comment, and the lines of a
/// list or block that belong to it), ending with a line break.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontEntry {
    pub key: String,
    pub rest: String,
}
