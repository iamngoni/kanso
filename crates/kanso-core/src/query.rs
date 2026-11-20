//! The note search language of the local-server API.
//!
//! A query is a list of clauses that must all hold. A clause is free text
//! (a bare word or a `"quoted phrase"`) or a qualifier such as `book:Blog`,
//! `title:"Two words"` or `tagId:tag:abc`, optionally negated with a leading
//! `-`. Text matches case-insensitively as a substring of the title or body.
//!
//! [`Query::matches`] is the reference semantics; the store's SQL search must
//! return exactly the notes it accepts.

use std::fmt;

use crate::id::{DocType, with_prefix};
use crate::model::TRASH_BOOK_ID;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Field {
    /// Notebook name, e.g. `book:"Desktop App"`.
    Book,
    /// Notebook ID with or without the `book:` prefix, e.g. `bookId:kGlLniaV`.
    BookId,
    /// Tag name, e.g. `tag:JavaScript`; a nested tag's parent, such as
    /// `tag:work` for `work/acme`, finds it too.
    Tag,
    /// Tag ID with or without the `tag:` prefix, e.g. `tagId:tag:abc`.
    TagId,
    /// Note status, e.g. `status:onHold`.
    Status,
    /// Substring of the title.
    Title,
    /// Substring of the body.
    Body,
}

impl Field {
    pub const ALL: [Field; 7] = [
        Field::Book,
        Field::BookId,
        Field::Tag,
        Field::TagId,
        Field::Status,
        Field::Title,
        Field::Body,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Field::Book => "book",
            Field::BookId => "bookId",
            Field::Tag => "tag",
            Field::TagId => "tagId",
            Field::Status => "status",
            Field::Title => "title",
            Field::Body => "body",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|f| f.key().eq_ignore_ascii_case(key))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clause {
    pub negated: bool,
    /// `None` for free text.
    pub field: Option<Field>,
    /// Never empty.
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Query {
    pub clauses: Vec<Clause>,
}

/// What a query needs to know about a note to decide whether it matches.
#[derive(Clone, Copy, Debug)]
pub struct NoteFacts<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub book_id: &'a str,
    /// Name of the note's notebook, if it exists.
    pub book_name: Option<&'a str>,
    pub status: &'a str,
    pub tag_ids: &'a [String],
    /// Names of the note's tags that exist.
    pub tag_names: &'a [String],
}

/// Case-insensitive substring test shared by the matcher and the store.
pub fn contains_ci(haystack: &str, needle: &str) -> bool {
    if needle.is_ascii() && haystack.is_ascii() {
        let needle = needle.as_bytes();
        return needle.is_empty()
            || haystack
                .as_bytes()
                .windows(needle.len())
                .any(|w| w.eq_ignore_ascii_case(needle));
    }
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// Where `needle` first occurs in `haystack`, ignoring case, as a byte
/// offset into `haystack`: the first character boundary whose rest of the
/// text, lowercased character by character, starts with the lowercased
/// needle. An empty needle is found at 0.
pub fn find_ci(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if needle.is_ascii() && haystack.is_ascii() {
        let needle = needle.as_bytes();
        return haystack
            .as_bytes()
            .windows(needle.len())
            .position(|w| w.eq_ignore_ascii_case(needle));
    }
    let needle: String = needle.chars().flat_map(char::to_lowercase).collect();
    // The text folded, and where each character's folded form starts in it
    // and in the original.
    let mut folded = String::with_capacity(haystack.len());
    let mut starts = Vec::with_capacity(haystack.len());
    for (byte, c) in haystack.char_indices() {
        starts.push((folded.len(), byte));
        folded.extend(c.to_lowercase());
    }
    starts
        .into_iter()
        .find(|&(at, _)| folded[at..].starts_with(&needle))
        .map(|(_, byte)| byte)
}

impl Query {
    pub fn parse(input: &str) -> Query {
        Parser {
            chars: input.chars().collect(),
            pos: 0,
        }
        .parse()
    }

    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    /// Whether the query asks for trashed notes, which searches otherwise skip.
    pub fn includes_trash(&self) -> bool {
        self.clauses.iter().any(|c| {
            !c.negated
                && match c.field {
                    Some(Field::BookId) => c.value == TRASH_BOOK_ID,
                    Some(Field::Book) => c.value.eq_ignore_ascii_case(TRASH_BOOK_ID),
                    _ => false,
                }
        })
    }

    pub fn matches(&self, note: &NoteFacts) -> bool {
        self.clauses.iter().all(|c| c.matches(note) != c.negated)
    }

    /// The words that can match `field` (the title or the body): free text
    /// and that field's own clauses, but not negated ones.
    pub fn terms_for(&self, field: Field) -> impl Iterator<Item = &str> {
        self.clauses
            .iter()
            .filter(move |c| !c.negated && (c.field.is_none() || c.field == Some(field)))
            .map(|c| c.value.as_str())
    }
}

impl Clause {
    pub fn text(value: impl Into<String>) -> Self {
        Clause {
            negated: false,
            field: None,
            value: value.into(),
        }
    }

    pub fn field(field: Field, value: impl Into<String>) -> Self {
        Clause {
            negated: false,
            field: Some(field),
            value: value.into(),
        }
    }

    /// Whether the clause holds for `note`, ignoring negation.
    pub fn matches(&self, note: &NoteFacts) -> bool {
        let v = self.value.as_str();
        match self.field {
            None => contains_ci(note.title, v) || contains_ci(note.body, v),
            Some(Field::Title) => contains_ci(note.title, v),
            Some(Field::Body) => contains_ci(note.body, v),
            Some(Field::Book) => {
                note.book_name
                    .is_some_and(|name| name.eq_ignore_ascii_case(v))
                    || (note.book_id == TRASH_BOOK_ID && v.eq_ignore_ascii_case(TRASH_BOOK_ID))
            }
            Some(Field::BookId) => note.book_id == self.book_id(),
            Some(Field::Tag) => note.tag_names.iter().any(|t| crate::tags::is_within(t, v)),
            Some(Field::TagId) => note
                .tag_ids
                .iter()
                .any(|t| *t == with_prefix(DocType::Tag, v)),
            Some(Field::Status) => note.status.eq_ignore_ascii_case(v),
        }
    }

    /// The full notebook ID a `bookId:` clause names.
    pub fn book_id(&self) -> String {
        if self.value == TRASH_BOOK_ID {
            TRASH_BOOK_ID.to_owned()
        } else {
            with_prefix(DocType::Book, &self.value)
        }
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, clause) in self.clauses.iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            write!(f, "{clause}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.negated {
            f.write_str("-")?;
        }
        let quote = match self.field {
            Some(field) => {
                write!(f, "{}:", field.key())?;
                self.value.starts_with('"') || self.value.contains(char::is_whitespace)
            }
            None => {
                self.value.starts_with(['"', '-'])
                    || self.value.contains(char::is_whitespace)
                    || self.value.contains(':')
            }
        };
        if quote {
            f.write_str("\"")?;
            for c in self.value.chars() {
                if matches!(c, '"' | '\\') {
                    f.write_str("\\")?;
                }
                write!(f, "{c}")?;
            }
            f.write_str("\"")
        } else {
            f.write_str(&self.value)
        }
    }
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn parse(mut self) -> Query {
        let mut clauses = Vec::new();
        loop {
            while self.peek().is_some_and(char::is_whitespace) {
                self.pos += 1;
            }
            let Some(first) = self.peek() else { break };
            let negated = first == '-';
            if negated {
                self.pos += 1;
                if self.peek().is_none_or(char::is_whitespace) {
                    continue;
                }
            }
            let (field, value) = self.clause_body();
            if !value.is_empty() {
                clauses.push(Clause {
                    negated,
                    field,
                    value,
                });
            }
        }
        Query { clauses }
    }

    /// Reads one clause after any `-`.
    fn clause_body(&mut self) -> (Option<Field>, String) {
        if self.peek() == Some('"') {
            return (None, self.quoted());
        }
        let start = self.pos;
        let key_end = (start..self.chars.len())
            .find(|&i| {
                let c = self.chars[i];
                c == ':' || c == '"' || c.is_whitespace()
            })
            .unwrap_or(self.chars.len());
        if self.chars.get(key_end) == Some(&':') {
            let key: String = self.chars[start..key_end].iter().collect();
            let after = self.chars.get(key_end + 1).copied();
            if let Some(field) = Field::from_key(&key)
                && after.is_some_and(|c| !c.is_whitespace())
            {
                self.pos = key_end + 1;
                let value = if self.peek() == Some('"') {
                    self.quoted()
                } else {
                    self.bare()
                };
                return (Some(field), value);
            }
        }
        (None, self.bare())
    }

    fn bare(&mut self) -> String {
        let start = self.pos;
        while self.peek().is_some_and(|c| !c.is_whitespace()) {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    /// Reads a `"…"` string starting at the opening quote. `\"` and `\\` are
    /// escapes; any other backslash is literal. An unterminated quote runs to
    /// the end of the input.
    fn quoted(&mut self) -> String {
        self.pos += 1;
        let mut value = String::new();
        while let Some(c) = self.peek() {
            self.pos += 1;
            match c {
                '"' => break,
                '\\' if matches!(self.peek(), Some('"' | '\\')) => {
                    value.push(self.peek().unwrap_or('\\'));
                    self.pos += 1;
                }
                _ => value.push(c),
            }
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neg(mut c: Clause) -> Clause {
        c.negated = true;
        c
    }

    #[test]
    fn parses_documented_examples() {
        assert_eq!(
            Query::parse(r#"-book:Backend "closure functions""#).clauses,
            vec![
                neg(Clause::field(Field::Book, "Backend")),
                Clause::text("closure functions")
            ]
        );
        assert_eq!(
            Query::parse(r#"-status:dropped title:"Sprint 10.0" debounce"#).clauses,
            vec![
                neg(Clause::field(Field::Status, "dropped")),
                Clause::field(Field::Title, "Sprint 10.0"),
                Clause::text("debounce"),
            ]
        );
        assert_eq!(
            Query::parse("bookId:kGlLniaV tagId:tag:abc  Typescript").clauses,
            vec![
                Clause::field(Field::BookId, "kGlLniaV"),
                Clause::field(Field::TagId, "tag:abc"),
                Clause::text("Typescript"),
            ]
        );
    }

    #[test]
    fn backlink_uris_are_text() {
        assert_eq!(
            Query::parse("kanso://note/t0iLc_TS").clauses,
            vec![Clause::text("kanso://note/t0iLc_TS")]
        );
    }

    #[test]
    fn edge_cases() {
        assert!(Query::parse("").is_empty());
        assert!(Query::parse(r#"  - "" -"#).is_empty());
        assert_eq!(Query::parse("title:").clauses, vec![Clause::text("title:")]);
        assert_eq!(
            Query::parse("TITLE:x").clauses,
            vec![Clause::field(Field::Title, "x")]
        );
        assert_eq!(
            Query::parse(r#""unterminated phrase"#).clauses,
            vec![Clause::text("unterminated phrase")]
        );
        assert_eq!(
            Query::parse(r#""a \"q\" C:\path""#).clauses,
            vec![Clause::text(r#"a "q" C:\path"#)]
        );
    }

    #[test]
    fn matching() {
        let tags = vec!["tag:js".to_owned()];
        let tag_names = vec!["JavaScript".to_owned()];
        let note = NoteFacts {
            title: "Sprint 10.0 notes",
            body: "We should Debounce input. See kanso://note/abc",
            book_id: "book:kGlLniaV",
            book_name: Some("Desktop App"),
            status: "active",
            tag_ids: &tags,
            tag_names: &tag_names,
        };
        let yes = [
            "debounce",
            r#"title:"sprint 10""#,
            r#"book:"desktop app""#,
            "bookId:kGlLniaV",
            "bookId:book:kGlLniaV",
            "tag:javascript",
            "tagId:tag:js",
            "tagId:js",
            "status:ACTIVE",
            "-status:dropped debounce",
            "kanso://note/abc",
            "body:input",
        ];
        for q in yes {
            assert!(Query::parse(q).matches(&note), "{q}");
        }
        let no = [
            "title:debounce",
            "-tag:javascript",
            "book:Blog",
            "status:onHold",
            "missing",
        ];
        for q in no {
            assert!(!Query::parse(q).matches(&note), "{q}");
        }
    }

    #[test]
    fn find_ci_examples() {
        assert_eq!(
            find_ci("Last writer wins by updatedAt", "UPDATED"),
            Some(20)
        );
        assert_eq!(find_ci("abc", ""), Some(0));
        assert_eq!(find_ci("abc", "d"), None);
        // Offsets are into the original text, after a multi-byte character.
        assert_eq!(find_ci("Über Sync", "sync"), Some(6));
        assert_eq!(find_ci("ÉCOLE école", "éco"), Some(0));
        // A Kelvin sign folds to an ASCII k.
        assert_eq!(find_ci("Ok", "\u{212A}"), Some(1));
    }

    #[test]
    fn terms_for_examples() {
        let query = Query::parse("sync -draft title:plan body:\"two words\" tag:x");
        let body: Vec<&str> = query.terms_for(Field::Body).collect();
        assert_eq!(body, ["sync", "two words"]);
        let title: Vec<&str> = query.terms_for(Field::Title).collect();
        assert_eq!(title, ["sync", "plan"]);
    }

    #[test]
    fn trash_is_opt_in() {
        assert!(Query::parse("bookId:trash").includes_trash());
        assert!(Query::parse("book:Trash x").includes_trash());
        assert!(!Query::parse("-bookId:trash").includes_trash());
        assert!(!Query::parse("trash").includes_trash());
    }
}
