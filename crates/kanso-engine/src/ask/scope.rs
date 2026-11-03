//! Scope: which notes a question may look at. Resolved once per question and
//! applied to every list before fusion and to every tool call, so a tool
//! cannot leave the scope. A scope never widens what Ask may read: locked
//! notes, the trash and `#ai-exclude` notes are still left out by
//! [`super::eligible`].

use std::collections::{HashMap, HashSet};

use kanso_store::Store;

use super::model::{AskScope, ScopeRefusal};

/// A scope resolved against the library.
#[derive(Clone, Debug, Default)]
pub struct ScopeFilter {
    /// Notebook IDs allowed (the notebook and all inside it), when scoped
    /// to a notebook.
    books: Option<HashSet<String>>,
    /// Tag IDs whose name matches, when scoped to a tag.
    tags: Option<HashSet<String>>,
    /// Note IDs allowed, when scoped to notes.
    notes: Option<HashSet<String>>,
}

impl ScopeFilter {
    /// A filter that allows everything.
    pub fn all() -> ScopeFilter {
        ScopeFilter::default()
    }

    /// Resolves `scope`. A scope that names a locked notebook is refused
    /// before anything runs.
    pub fn resolve(store: &Store, scope: &AskScope) -> Result<ScopeFilter, ScopeRefusal> {
        match scope {
            AskScope::All => Ok(ScopeFilter::all()),
            AskScope::Book(book) => {
                let books = store.books().unwrap_or_default();
                let pairs: Vec<(String, Option<String>)> = books
                    .iter()
                    .map(|b| (b.id.clone(), b.parent_book_id.clone()))
                    .collect();
                if !pairs.iter().any(|(id, _)| id == book) {
                    // A notebook that isn't there has no notes.
                    return Ok(ScopeFilter {
                        books: Some(HashSet::new()),
                        ..ScopeFilter::default()
                    });
                }
                // Fail closed: a lock check that cannot run refuses.
                if store
                    .locked_books()
                    .map_or(true, |locked| locked.contains(book))
                {
                    return Err(ScopeRefusal::Locked);
                }
                let inside: HashSet<String> = pairs
                    .iter()
                    .filter(|(id, _)| in_notebook(id, book, &pairs))
                    .map(|(id, _)| id.clone())
                    .collect();
                Ok(ScopeFilter {
                    books: Some(inside),
                    ..ScopeFilter::default()
                })
            }
            AskScope::Tag(name) => {
                let wanted = name.trim().trim_start_matches('#').to_lowercase();
                let tags: HashSet<String> = store
                    .tags()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|tag| tag.name.to_lowercase() == wanted)
                    .map(|tag| tag.id)
                    .collect();
                Ok(ScopeFilter {
                    tags: Some(tags),
                    ..ScopeFilter::default()
                })
            }
            AskScope::Notes(ids) => Ok(ScopeFilter {
                notes: Some(ids.iter().map(|id| normalize_id(id)).collect()),
                ..ScopeFilter::default()
            }),
        }
    }

    /// Whether a note is inside the scope: its ID, notebook and tag IDs.
    pub fn allows(&self, id: &str, book_id: &str, tags: &[String]) -> bool {
        if let Some(notes) = &self.notes
            && !notes.contains(id)
        {
            return false;
        }
        if let Some(books) = &self.books
            && !books.contains(book_id)
        {
            return false;
        }
        if let Some(wanted) = &self.tags
            && !tags.iter().any(|tag| wanted.contains(tag))
        {
            return false;
        }
        true
    }

    /// Whether the filter lets everything through.
    pub fn is_all(&self) -> bool {
        self.books.is_none() && self.tags.is_none() && self.notes.is_none()
    }
}

/// Whether notebook `id` is `book` or inside it, by `books`' `(id, parent)`
/// pairs: up the parents, at most as deep as there are notebooks.
fn in_notebook(id: &str, book: &str, books: &[(String, Option<String>)]) -> bool {
    let mut current = Some(id);
    for _ in 0..=books.len() {
        let Some(at) = current else { return false };
        if at == book {
            return true;
        }
        current = books
            .iter()
            .find(|(b, _)| b == at)
            .and_then(|(_, parent)| parent.as_deref());
    }
    false
}

fn normalize_id(id: &str) -> String {
    let id = id.trim();
    if id.starts_with("note:") {
        id.to_owned()
    } else {
        format!("note:{id}")
    }
}

/// The notebooks as `id -> path` (`Work/Plans`), for labelling an excerpt.
pub fn book_paths(store: &Store) -> HashMap<String, String> {
    let books = store.books().unwrap_or_default();
    let by_id: HashMap<&str, &kanso_core::model::Book> =
        books.iter().map(|b| (b.id.as_str(), b)).collect();
    let mut paths = HashMap::new();
    for book in &books {
        let mut names = vec![book.name.trim().to_owned()];
        let mut parent = book.parent_book_id.as_deref();
        for _ in 0..books.len() {
            let Some(id) = parent else { break };
            let Some(up) = by_id.get(id) else { break };
            names.push(up.name.trim().to_owned());
            parent = up.parent_book_id.as_deref();
        }
        names.reverse();
        paths.insert(book.id.clone(), names.join("/"));
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_allows_everything_and_a_notes_scope_only_those_notes() {
        let all = ScopeFilter::all();
        assert!(all.allows("note:a", "book:x", &[]));
        let notes = ScopeFilter {
            notes: Some(HashSet::from(["note:a".to_owned()])),
            ..ScopeFilter::default()
        };
        assert!(notes.allows("note:a", "book:x", &[]));
        assert!(!notes.allows("note:b", "book:x", &[]));
        assert_eq!(normalize_id("a"), "note:a");
    }
}
