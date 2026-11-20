//! What a plugin may read and change, resolved from its grant once per
//! request.
//!
//! ```text
//! book scope = granted notebooks + every notebook inside them, minus locked ones
//! tag scope  = granted tags
//! readable   = live, not in the trash, not locked (sealed, or in a locked
//!              notebook), not tagged #ai-exclude, and in the book scope, the
//!              tag scope, every unlocked notebook (when granted), or the
//!              open note (with `note.read`)
//! writable   = readable, and in the granted writable notebooks (with those
//!              inside them, minus locked ones)
//! ```
//!
//! The lock test is the store's own (`note_is_locked`, over the
//! `locked_books` view), so this path can't drift from the rest of the app,
//! and it fails closed: a lock that can't be checked is a lock. Unlock
//! state is irrelevant. Anything not readable does not exist for the
//! plugin: lists drop it before counting, and `get` says "not found",
//! never "locked", so a locked note can't be told from a missing one.
//!
//! An attachment is readable only while a readable note's text links it
//! (`kanso://file:<id>`). A locked note's text is empty to every reader
//! outside the app, so its attachments never are.

use std::collections::{HashMap, HashSet};

use kanso_core::links::FILE_LINK_PREFIX;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{DocType, Note, Query};
use kanso_store::{SearchParams, Store};
use serde_json::{Value, json};

use crate::model::{EditorPermission, Grant};

/// The most notes one `notes.list` returns.
pub const MAX_LIST: usize = 100;
/// What `notes.list` returns without a `limit`.
pub const DEFAULT_LIST: usize = 50;
/// The largest attachment a plugin may read: it travels as base64 inside a
/// message to the host.
pub const MAX_ATTACHMENT_BYTES: usize = 4 * 1024 * 1024;

/// A plugin's view of the library for one request.
pub struct Scope<'a> {
    store: &'a Store,
    /// `None`: every unlocked notebook.
    read_books: Option<HashSet<String>>,
    read_tags: HashSet<String>,
    write_books: HashSet<String>,
    locked_books: HashSet<String>,
    /// Tag IDs named `ai-exclude`, however written.
    excluded_tags: HashSet<String>,
    tag_names: HashMap<String, String>,
    /// The open note, when the plugin may read it.
    open_note: Option<String>,
}

/// What `notes.list` was asked for.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListQuery {
    pub book_id: Option<String>,
    pub tag: Option<String>,
    pub query: Option<String>,
    pub limit: Option<usize>,
    /// Where the last page stopped.
    pub cursor: Option<String>,
}

impl ListQuery {
    /// From `notes.list`'s argument object; anything else is ignored.
    pub fn from_args(args: &Value) -> ListQuery {
        let text = |key: &str| args.get(key).and_then(Value::as_str).map(str::to_owned);
        ListQuery {
            book_id: text("bookId"),
            tag: text("tag"),
            query: text("query"),
            limit: args
                .get("limit")
                .and_then(Value::as_u64)
                .map(|n| n as usize),
            cursor: text("cursor"),
        }
    }
}

/// Every notebook under `roots` (themselves included), from the
/// `(id, parent)` pairs of all notebooks. Pure.
pub fn with_descendants<'b>(
    roots: impl IntoIterator<Item = &'b String>,
    books: &[(String, Option<String>)],
) -> HashSet<String> {
    let mut children: HashMap<&str, Vec<&str>> = HashMap::new();
    for (id, parent) in books {
        if let Some(parent) = parent.as_deref() {
            children.entry(parent).or_default().push(id);
        }
    }
    let known: HashSet<&str> = books.iter().map(|(id, _)| id.as_str()).collect();
    let mut within = HashSet::new();
    let mut queue: Vec<&str> = roots
        .into_iter()
        .map(String::as_str)
        .filter(|id| known.contains(id))
        .collect();
    while let Some(id) = queue.pop() {
        if within.insert(id.to_owned()) {
            queue.extend(children.get(id).into_iter().flatten().copied());
        }
    }
    within
}

/// The attachment IDs (`file:…`) a note's text links, in order, each once.
/// Pure.
pub fn linked_files(body: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for (at, _) in body.match_indices(FILE_LINK_PREFIX) {
        let rest = &body[at + FILE_LINK_PREFIX.len()..];
        let id: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
            .collect();
        if id.is_empty() {
            continue;
        }
        let id = format!("file:{id}");
        if !found.contains(&id) {
            found.push(id);
        }
    }
    found
}

/// `file:abc` from `file:abc` or `abc`. Pure.
pub fn file_id(given: &str) -> String {
    if given.starts_with("file:") {
        given.to_owned()
    } else {
        format!("file:{given}")
    }
}

impl<'a> Scope<'a> {
    /// `grant`'s view of `store` now, with `open_note` the note open in the
    /// editor. `None` when the store can't say (so nothing is readable).
    pub fn new(store: &'a Store, grant: &Grant, open_note: Option<&str>) -> Option<Scope<'a>> {
        let locked_books = store.locked_books().ok()?;
        let books: Vec<(String, Option<String>)> = store
            .books()
            .ok()?
            .into_iter()
            .map(|book| (book.id, book.parent_book_id))
            .collect();
        let tags = store.tags().ok()?;
        let excluded_tags = tags
            .iter()
            .filter(|tag| kanso_core::tags::is_ai_exclude(&tag.name))
            .map(|tag| tag.id.clone())
            .collect();
        let tag_names = tags.into_iter().map(|tag| (tag.id, tag.name)).collect();
        let notes = &grant.notes;
        let unlocked = |mut set: HashSet<String>| {
            set.retain(|id| !locked_books.contains(id) && id != TRASH_BOOK_ID);
            set
        };
        let read_books = if notes.read_all {
            None
        } else {
            Some(unlocked(with_descendants(&notes.read_books, &books)))
        };
        let write_books = unlocked(with_descendants(&notes.write_books, &books));
        let open_note = open_note
            .filter(|_| grant.editor.contains(&EditorPermission::NoteRead))
            .map(str::to_owned);
        Some(Scope {
            store,
            read_books,
            read_tags: notes.read_tags.iter().cloned().collect(),
            write_books,
            locked_books,
            excluded_tags,
            tag_names,
            open_note,
        })
    }

    fn excluded(&self, note: &Note) -> bool {
        note.tags
            .iter()
            .any(|tag| self.excluded_tags.contains(tag) || kanso_core::tags::is_ai_exclude(tag))
    }

    /// Note `id`, when the plugin may read it.
    pub fn readable(&self, id: &str) -> Option<Note> {
        if DocType::of_id(id) != Some(DocType::Note) {
            return None;
        }
        // Fail closed: a lock that can't be checked is a lock.
        if self.store.note_is_locked(id).unwrap_or(true) {
            return None;
        }
        let note = self.store.note(id).ok()??;
        if note.is_trashed()
            || note.is_locked()
            || self.locked_books.contains(&note.book_id)
            || self.excluded(&note)
        {
            return None;
        }
        let in_books = self
            .read_books
            .as_ref()
            .is_none_or(|books| books.contains(&note.book_id));
        let in_tags = note.tags.iter().any(|tag| self.read_tags.contains(tag));
        let open = self.open_note.as_deref() == Some(id);
        (in_books || in_tags || open).then_some(note)
    }

    /// Whether the open note is one the plugin may see at all: the editor
    /// calls (selection, replace) use it.
    pub fn open_note_visible(&self, id: &str) -> bool {
        self.readable(id).is_some()
    }

    /// Note `id`, when the plugin may change it.
    pub fn writable(&self, id: &str) -> Option<Note> {
        self.readable(id)
            .filter(|note| self.write_books.contains(&note.book_id))
    }

    /// Whether the plugin may make a note in notebook `book_id`.
    pub fn can_create_in(&self, book_id: &str) -> bool {
        self.write_books.contains(book_id)
    }

    /// The tag names of `note`, for what the plugin is shown.
    fn tag_names_of(&self, note: &Note) -> Vec<String> {
        note.tags
            .iter()
            .map(|tag| {
                self.tag_names
                    .get(tag)
                    .cloned()
                    .unwrap_or_else(|| tag.clone())
            })
            .collect()
    }

    /// A readable note as `notes.get` shows it.
    pub fn note_json(&self, note: &Note) -> Value {
        json!({
            "id": note.id,
            "rev": note.rev,
            "title": note.title,
            "body": note.body,
            "bookId": note.book_id,
            "tags": note.tags,
            "tagNames": self.tag_names_of(note),
            "status": note.status.as_str(),
            "createdAt": note.created_at,
            "updatedAt": note.updated_at,
        })
    }

    /// A readable note as `notes.list` shows it: no body.
    pub fn summary_json(&self, note: &Note) -> Value {
        json!({
            "id": note.id,
            "rev": note.rev,
            "title": note.title,
            "bookId": note.book_id,
            "tags": note.tags,
            "tagNames": self.tag_names_of(note),
            "status": note.status.as_str(),
            "createdAt": note.created_at,
            "updatedAt": note.updated_at,
        })
    }

    /// One page of the readable notes `query` names, most recently changed
    /// first, and the cursor for the next page. Every note is held to
    /// [`Scope::readable`] before it is counted, so a hidden note never
    /// shows as a gap.
    pub fn list(&self, query: &ListQuery) -> (Vec<Note>, Option<String>) {
        let limit = query.limit.unwrap_or(DEFAULT_LIST).clamp(1, MAX_LIST);
        let skip: usize = query
            .cursor
            .as_deref()
            .and_then(|c| c.parse().ok())
            .unwrap_or(0);
        let params = SearchParams {
            query: Query::parse(query.query.as_deref().unwrap_or("")),
            ..SearchParams::default()
        };
        let Ok(found) = self.store.search(&params) else {
            return (Vec::new(), None);
        };
        let mut page = Vec::new();
        let mut matched = 0;
        let mut more = false;
        for doc in found {
            if doc.doc_type != DocType::Note {
                continue;
            }
            let Some(note) = self.readable(&doc.id) else {
                continue;
            };
            if query.book_id.as_ref().is_some_and(|b| *b != note.book_id) {
                continue;
            }
            if let Some(tag) = &query.tag
                && !note.tags.contains(tag)
                && !self
                    .tag_names_of(&note)
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(tag))
            {
                continue;
            }
            matched += 1;
            if matched <= skip {
                continue;
            }
            if page.len() == limit {
                more = true;
                break;
            }
            page.push(note);
        }
        let cursor = more.then(|| (skip + limit).to_string());
        (page, cursor)
    }

    /// Attachment `file` (`file:…` or its bare ID), when a readable note
    /// links it: its content type and bytes.
    pub fn attachment(&self, file: &str) -> Option<(String, Vec<u8>)> {
        let id = file_id(file);
        if DocType::of_id(&id) != Some(DocType::File) {
            return None;
        }
        if !self.links_file(&id) {
            return None;
        }
        let (content_type, attachment) = self.store.attachment(&id).ok()??;
        Some((content_type, attachment.data))
    }

    /// Whether a note the plugin may read links attachment `id`.
    pub fn links_file(&self, id: &str) -> bool {
        let bare = id.trim_start_matches("file:");
        let params = SearchParams {
            query: Query::parse(&format!("\"{bare}\"")),
            ..SearchParams::default()
        };
        let candidates: Vec<String> = match self.store.search(&params) {
            Ok(found) => found.into_iter().map(|doc| doc.id).collect(),
            Err(_) => return false,
        };
        let mut checked: HashSet<String> = HashSet::new();
        let mut linked = |note_id: &str| {
            checked.insert(note_id.to_owned())
                && self
                    .readable(note_id)
                    .is_some_and(|note| linked_files(&note.body).iter().any(|f| f == id))
        };
        if candidates.iter().any(|note| linked(note)) {
            return true;
        }
        // The search may tokenize IDs differently: look through the open
        // note and the granted notebooks' notes too.
        if let Some(open) = self.open_note.clone()
            && linked(&open)
        {
            return true;
        }
        let books: Vec<String> = match &self.read_books {
            Some(books) => books.iter().cloned().collect(),
            None => match self.store.books() {
                Ok(all) => all.into_iter().map(|b| b.id).collect(),
                Err(_) => return false,
            },
        };
        let Ok(notes) = self.store.notes_in_books(&books) else {
            return false;
        };
        notes.iter().any(|note| linked(&note.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn linked_files_finds_each_link_once() {
        let body = "![a](kanso://file:abc_1) and [b](kanso://file:abc_1) kanso://file:Z-9.";
        assert_eq!(linked_files(body), vec!["file:abc_1", "file:Z-9"]);
        assert!(linked_files("kanso://file:").is_empty());
    }

    #[test]
    fn descendants_follow_the_tree() {
        let books = vec![
            ("book:a".to_owned(), None),
            ("book:b".to_owned(), Some("book:a".to_owned())),
            ("book:c".to_owned(), Some("book:b".to_owned())),
            ("book:d".to_owned(), None),
        ];
        let within = with_descendants(&["book:a".to_owned()], &books);
        assert_eq!(within.len(), 3);
        assert!(!within.contains("book:d"));
        assert!(with_descendants(&["book:zz".to_owned()], &books).is_empty());
    }

    proptest! {
        /// Invariant: every ID linked_files returns is linked in the text,
        /// as `kanso://file:` and that ID.
        #[test]
        fn prop_linked_files_are_in_the_text(body in "(kanso://file:[a-z0-9_-]{0,6}|[a-z :/()!\\[\\]]{0,6}){0,8}") {
            for id in linked_files(&body) {
                let bare = id.strip_prefix("file:").unwrap();
                let link = format!("{FILE_LINK_PREFIX}{bare}");
                prop_assert!(body.contains(&link));
            }
        }

        /// Invariant: descendants never include a notebook outside the
        /// roots' subtrees, and always include the roots that exist.
        #[test]
        fn prop_descendants_stay_in_their_subtrees(parents in prop::collection::vec(prop::option::of(0..8usize), 8), root in 0..8usize) {
            let books: Vec<(String, Option<String>)> = parents
                .iter()
                .enumerate()
                .map(|(i, p)| (format!("book:{i}"), p.filter(|p| *p < i).map(|p| format!("book:{p}"))))
                .collect();
            let roots = vec![format!("book:{root}")];
            let within = with_descendants(&roots, &books);
            prop_assert!(within.contains(&roots[0]));
            for id in &within {
                // Walk up: the root is an ancestor (or the notebook itself).
                let mut cursor = Some(id.clone());
                let mut reached = false;
                while let Some(c) = cursor {
                    if c == roots[0] { reached = true; break; }
                    cursor = books.iter().find(|(b, _)| *b == c).and_then(|(_, p)| p.clone());
                }
                prop_assert!(reached);
            }
        }
    }
}
