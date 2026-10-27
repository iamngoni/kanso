//! Kanso's agent visibility and permission boundary.
//!
//! A request signed in with the local API's credentials is [`Caller::Full`]:
//! everything the API serves. One with an agent's token is
//! [`Caller::Agent`]: `/mcp` only, the read tools its [`AgentScope`] names,
//! and only the notes [`Visibility`] lets through. A note is visible when it
//! is live, not in the trash, not locked (sealed, or in a locked notebook),
//! not tagged `#ai-exclude`, and in the scope's notebooks or the notebooks
//! inside them. Anything else does not exist for the agent: it reads as
//! "not found", never "forbidden", and an error deciding counts as not
//! visible.
//!
//! An agent never writes. A scope the person has let suggest edits also
//! offers [`SUGGEST_TOOLS`], which only stage a change in the
//! [`ReviewQueue`](crate::review::ReviewQueue) for the person to apply.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use kanso_core::model::{LOCK_FIELD, PUBLISH_FIELD, SHARE_FIELD, TRASH_BOOK_ID};
use kanso_core::{DocType, Note};
use kanso_store::Store;
use serde_json::Value;

use crate::tokens::TokenId;

/// The tools an agent may ever be given: reads, and none that reach
/// attachments. Everything else is denied.
pub const READ_TOOLS: [&str; 17] = [
    "search-notes",
    "read-note",
    "list-notes",
    "list-notebooks",
    "list-templates",
    "kanso-get-outline",
    "kanso-list-tasks",
    "kanso-search-by-meaning",
    "kanso-related-notes",
    "kanso-resolve-wikilink",
    "kanso-list-tags",
    "kanso-get-properties",
    "kanso-list-note-links",
    "kanso-list-backlinks",
    "kanso-run-query-view",
    "kanso-list-versions",
    "kanso-read-version",
];

/// The tools Ask your notes gives an agent: the same two a model with an
/// API key gets.
pub const ASK_TOOLS: [&str; 2] = ["search-notes", "read-note"];

/// The tools that stage an edit for review instead of writing, given only to
/// a scope the person allowed to suggest edits. The first write release is
/// note text and title only (issue #46, the review queue, decision 8):
/// `update-tag` and `update-book` stay denied, and so does everything that
/// touches attachments, notebooks' locks, or deletes.
pub const SUGGEST_TOOLS: [&str; 3] = ["patch-note", "update-note", "create-note"];

/// What one agent token may do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentScope {
    /// Always true: no call of an agent's writes the store. Edits it
    /// suggests wait in the review queue until the person applies them.
    pub read_only: bool,
    /// The notebooks it may read, with the ones inside them; `None` is every
    /// unlocked notebook.
    pub notebooks: Option<HashSet<String>>,
    /// Always false: attachments are never served to an agent.
    pub allow_attachments: bool,
    /// The tools it is offered, within [`READ_TOOLS`] and, when it may
    /// suggest edits, [`SUGGEST_TOOLS`].
    pub tools: BTreeSet<String>,
    /// What the person calls the agent, when it may suggest edits: the
    /// name its suggestions carry in the review queue and in the note's
    /// history. `None` is a scope that suggests nothing.
    pub suggests_as: Option<String>,
}

impl AgentScope {
    /// The scope Ask your notes gives an agent, over `notebooks`.
    pub fn ask(notebooks: Option<HashSet<String>>) -> AgentScope {
        AgentScope::with_tools(notebooks, &ASK_TOOLS)
    }

    /// A read-only scope over `notebooks` offering `tools`; any not in
    /// [`READ_TOOLS`] is left out.
    pub fn with_tools(notebooks: Option<HashSet<String>>, tools: &[&str]) -> AgentScope {
        AgentScope {
            read_only: true,
            notebooks,
            allow_attachments: false,
            tools: tools
                .iter()
                .filter(|tool| READ_TOOLS.contains(tool))
                .map(|tool| (*tool).to_owned())
                .collect(),
            suggests_as: None,
        }
    }

    /// [`AgentScope::ask`], and the agent called `agent` may also suggest
    /// edits to the notes it may read.
    pub fn ask_and_suggest(notebooks: Option<HashSet<String>>, agent: &str) -> AgentScope {
        let mut scope = AgentScope::ask(notebooks);
        scope
            .tools
            .extend(SUGGEST_TOOLS.iter().map(|tool| (*tool).to_owned()));
        scope.suggests_as = Some(agent.trim().to_owned());
        scope
    }

    /// Whether this scope offers tool `name`.
    pub fn allows_tool(&self, name: &str) -> bool {
        self.tools.contains(name)
            && (READ_TOOLS.contains(&name)
                || (SUGGEST_TOOLS.contains(&name) && self.suggests_as.is_some()))
    }
}

/// Who sent a request.
#[derive(Clone, Debug)]
pub enum Caller {
    /// The local API's credentials: everything.
    Full,
    /// An agent's token.
    Agent {
        token: TokenId,
        scope: Arc<AgentScope>,
    },
}

/// The fields an agent never sees: a published note's or notebook's link
/// and key, a shared note's mark, and the lock's envelope.
const HIDDEN_FIELDS: [&str; 3] = [PUBLISH_FIELD, SHARE_FIELD, LOCK_FIELD];

/// A document as an agent may see it: without [`HIDDEN_FIELDS`].
pub(crate) fn strip(mut doc: Value) -> Value {
    if let Some(fields) = doc.as_object_mut() {
        for field in HIDDEN_FIELDS {
            fields.remove(field);
        }
    }
    doc
}

/// What one request of an agent may see, read once from the store.
pub(crate) struct Visibility<'a> {
    store: &'a Store,
    /// The notebooks in scope, without locked ones; `None` is every unlocked
    /// notebook.
    books: Option<HashSet<String>>,
    locked_books: HashSet<String>,
    /// Tag names by ID, to find `#ai-exclude`.
    tag_names: HashMap<String, String>,
    /// The notes shown so far, in order.
    shown: std::cell::RefCell<Vec<String>>,
}

impl<'a> Visibility<'a> {
    /// The visibility of `scope` in `store` now, or `None` when the store
    /// can't say (so nothing is visible).
    pub fn new(store: &'a Store, scope: &AgentScope) -> Option<Visibility<'a>> {
        let locked_books = store.locked_books().ok()?;
        let tag_names = store
            .tags()
            .ok()?
            .into_iter()
            .map(|tag| (tag.id, tag.name))
            .collect();
        let books = match &scope.notebooks {
            None => None,
            Some(roots) => {
                let all = store.books().ok()?;
                let mut children: HashMap<&str, Vec<&str>> = HashMap::new();
                for book in &all {
                    if let Some(parent) = book.parent_book_id.as_deref() {
                        children.entry(parent).or_default().push(&book.id);
                    }
                }
                let mut within = HashSet::new();
                let mut queue: Vec<&str> = all
                    .iter()
                    .map(|book| book.id.as_str())
                    .filter(|id| roots.contains(*id))
                    .collect();
                while let Some(id) = queue.pop() {
                    if within.insert(id.to_owned()) {
                        queue.extend(children.get(id).into_iter().flatten().copied());
                    }
                }
                within.retain(|id| !locked_books.contains(id));
                Some(within)
            }
        };
        Some(Visibility {
            store,
            books,
            locked_books,
            tag_names,
            shown: Default::default(),
        })
    }

    /// Whether notebook `id` is one the agent may see.
    pub fn book_visible(&self, id: &str) -> bool {
        DocType::of_id(id) == Some(DocType::Book)
            && id != TRASH_BOOK_ID
            && !self.locked_books.contains(id)
            && self.books.as_ref().is_none_or(|books| books.contains(id))
    }

    /// Note `id`, when the agent may see it.
    pub fn note(&self, id: &str) -> Option<Note> {
        if DocType::of_id(id) != Some(DocType::Note) {
            return None;
        }
        // Fail closed: a lock that can't be checked is a lock.
        if self.store.note_is_locked(id).unwrap_or(true) {
            return None;
        }
        let note = self.store.note(id).ok()??;
        if note.is_trashed() || !self.book_visible(&note.book_id) {
            return None;
        }
        let excluded = note.tags.iter().any(|tag| {
            kanso_core::tags::is_ai_exclude(tag)
                || self
                    .tag_names
                    .get(tag)
                    .is_some_and(|name| kanso_core::tags::is_ai_exclude(name))
        });
        (!excluded).then_some(note)
    }

    /// Whether the agent may see note `id`.
    pub fn allows(&self, id: &str) -> bool {
        self.note(id).is_some()
    }

    /// Notes note `id` as shown to the agent, for the Used notes list.
    pub fn show(&self, id: &str) {
        let mut shown = self.shown.borrow_mut();
        if !shown.iter().any(|seen| seen == id) {
            shown.push(id.to_owned());
        }
    }

    /// Every note shown, in order.
    pub fn shown(&self) -> Vec<String> {
        self.shown.borrow().clone()
    }
}
