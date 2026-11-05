//! Kanso's agent visibility and permission boundary.
//!
//! A request signed in with an API key is [`Caller::Key`]: what its access
//! allows, through the same [`Visibility`] as an agent. One with an agent's
//! token is [`Caller::Agent`]: `/mcp` only, the read tools its [`AgentScope`] names,
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
use std::time::Duration;

use kanso_core::model::{LOCK_FIELD, PUBLISH_FIELD, SHARE_FIELD, TRASH_BOOK_ID};
use kanso_core::{DocType, Note};
use kanso_store::{KeyAccess, Store};
use serde_json::Value;

use crate::changeset::TaskId;
use crate::key_model::KeyId;
use crate::tokens::TokenId;

/// Read tools offered to an agent. Attachment bytes remain unavailable;
/// PDF tools expose only the extracted, page-scoped text and annotations
/// after the owning note has passed the same visibility checks.
pub const READ_TOOLS: [&str; 27] = [
    "search-notes",
    "read-note",
    "list-notes",
    "list-notebooks",
    "list-templates",
    "kanso-get-outline",
    "kanso-list-tasks",
    "kanso-list-open-loops",
    "kanso-search-by-meaning",
    "kanso-related-notes",
    "kanso-resolve-wikilink",
    "kanso-list-tags",
    "kanso-get-properties",
    "kanso-list-note-links",
    "kanso-list-backlinks",
    "kanso-list-mentions",
    "kanso-run-query-view",
    "kanso-list-versions",
    "kanso-read-version",
    "kanso-ask-search",
    "kanso-ask-read-lines",
    "kanso-ask-when",
    "kanso-list-canvases",
    "kanso-read-canvas",
    "kanso-search-pdf",
    "kanso-read-pdf-page",
    "kanso-list-pdf-highlights",
];

/// The tools Ask your notes gives an agent: the same five a model with an
/// API key gets. The last three number what they show (`L12| text`) so an
/// answer can cite lines and have them checked.
pub const ASK_TOOLS: [&str; 5] = [
    "search-notes",
    "read-note",
    "kanso-ask-search",
    "kanso-ask-read-lines",
    "kanso-ask-when",
];

/// The tools that stage an edit for review instead of writing, given only to
/// a scope the person allowed to suggest edits. The first write release is
/// note text and title only (issue #46, the review queue, decision 8):
/// `update-tag` and `update-book` stay denied, and so does everything that
/// touches attachments, notebooks' locks, or deletes.
pub const SUGGEST_TOOLS: [&str; 3] = ["patch-note", "update-note", "create-note"];

/// The tools only an agent task's token gets: they stage into the task's
/// [`ChangeSet`](crate::changeset::ChangeSet) and write nothing.
pub const TASK_TOOLS: [&str; 4] = [
    "kanso-propose-move",
    "kanso-propose-merge",
    "kanso-list-proposed",
    "kanso-task-done",
];

/// What an agent task may propose, set by the person on the New agent task
/// sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskOps {
    pub create: bool,
    pub edit: bool,
    pub moves: bool,
    pub merge: bool,
}

impl Default for TaskOps {
    fn default() -> Self {
        TaskOps {
            create: true,
            edit: true,
            moves: true,
            merge: true,
        }
    }
}

impl TaskOps {
    /// A merge needs new notes or edits, and moves (its sources go to the
    /// trash).
    pub fn merge_allowed(&self) -> bool {
        self.merge && (self.create || self.edit) && self.moves
    }
}

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
    /// The agent task this token belongs to: its writes stage into the
    /// task's change set instead of the review queue.
    pub task: Option<TaskId>,
    /// Only these notes (by ID), when the person chose notes and not a
    /// notebook; `None` is no such limit.
    pub notes: Option<HashSet<String>>,
    /// What a task may propose.
    pub task_ops: TaskOps,
    /// A task token's own lifetime and tool-call budget.
    pub task_limits: Option<TaskLimits>,
    /// The only notebooks a task may write into (new notes, moves, merge
    /// targets); `None` is any it can see. Reading a note never widens it:
    /// text in a note can't send a write to another notebook.
    pub write_books: Option<HashSet<String>>,
    /// Where a task's new notes go.
    pub new_note_book: Option<String>,
}

/// How long a task token lives and how many tool calls it may make.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskLimits {
    pub lifetime: Duration,
    pub calls: usize,
    /// The most notes one task may be shown.
    pub max_notes: usize,
}

/// The notes a task with `scope` may read now, in no particular order:
/// the chosen notes, or the notes of the chosen notebooks, that
/// [`Visibility`] lets through. Empty when the store can't say.
pub fn readable_notes(store: &Store, scope: &AgentScope) -> Vec<String> {
    let Some(vis) = Visibility::new(store, scope) else {
        return Vec::new();
    };
    let ids: Vec<String> = match (&scope.notes, &vis.books) {
        (Some(notes), _) => notes.iter().cloned().collect(),
        (None, Some(books)) => {
            let books: Vec<String> = books.iter().cloned().collect();
            store
                .notes_in_books(&books)
                .map(|notes| notes.into_iter().map(|n| n.id).collect())
                .unwrap_or_default()
        }
        (None, None) => {
            let books: Vec<String> = store
                .books()
                .map(|all| all.into_iter().map(|b| b.id).collect())
                .unwrap_or_default();
            store
                .notes_in_books(&books)
                .map(|notes| notes.into_iter().map(|n| n.id).collect())
                .unwrap_or_default()
        }
    };
    ids.into_iter().filter(|id| vis.allows(id)).collect()
}

/// Whether a task with `scope` may read any note at all. Stops at the first
/// one; meant for a worker thread, not a keystroke.
pub fn has_readable_notes(store: &Store, scope: &AgentScope) -> bool {
    if store
        .counts()
        .is_ok_and(|counts| counts.get(&DocType::Note).copied().unwrap_or(0) == 0)
    {
        return false;
    }
    !readable_notes(store, scope).is_empty()
}

impl AgentScope {
    /// The scope of an agent task named `agent`: reads like Ask, and its
    /// proposals (new notes, edits, moves, merges) stage into the change set
    /// of `task`. `notebooks` or `notes` is what the person chose to share.
    pub fn task(
        notebooks: Option<HashSet<String>>,
        notes: Option<HashSet<String>>,
        agent: &str,
        task: TaskId,
        ops: TaskOps,
        limits: TaskLimits,
    ) -> AgentScope {
        let mut scope = AgentScope::with_tools(
            notebooks,
            &[
                "search-notes",
                "read-note",
                "list-notes",
                "list-notebooks",
                "kanso-get-outline",
                "kanso-search-by-meaning",
                "kanso-related-notes",
                "kanso-resolve-wikilink",
                "kanso-list-note-links",
                "kanso-list-backlinks",
                "kanso-list-tags",
                "kanso-get-properties",
                "kanso-search-pdf",
                "kanso-read-pdf-page",
                "kanso-list-pdf-highlights",
            ],
        );
        let mut write: Vec<&str> = vec!["kanso-list-proposed", "kanso-task-done"];
        if ops.create {
            write.push("create-note");
        }
        if ops.edit {
            write.push("update-note");
            write.push("patch-note");
        }
        if ops.moves {
            write.push("kanso-propose-move");
        }
        if ops.merge_allowed() {
            write.push("kanso-propose-merge");
        }
        scope.tools.extend(write.into_iter().map(str::to_owned));
        scope.suggests_as = Some(agent.trim().to_owned());
        scope.task = Some(task);
        scope.notes = notes;
        scope.task_ops = ops;
        scope.task_limits = Some(limits);
        scope
    }
}

impl AgentScope {
    /// This task scope, limited to writing into `write_books` (when given),
    /// with new notes going to `new_note_book` (when not empty).
    pub fn with_writes(
        mut self,
        write_books: Option<HashSet<String>>,
        new_note_book: &str,
    ) -> AgentScope {
        self.write_books = write_books;
        self.new_note_book = Some(new_note_book.trim().to_owned()).filter(|b| !b.is_empty());
        self
    }

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
            task: None,
            notes: None,
            task_ops: TaskOps::default(),
            task_limits: None,
            write_books: None,
            new_note_book: None,
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
                || (SUGGEST_TOOLS.contains(&name) && self.suggests_as.is_some())
                || (TASK_TOOLS.contains(&name) && self.task.is_some()))
    }
}

/// Who sent a request.
#[derive(Clone, Debug)]
pub enum Caller {
    /// An agent's token.
    Agent {
        token: TokenId,
        scope: Arc<AgentScope>,
    },
    /// An API key (issue #243): what it may do is its `access`, and what it
    /// may see its `scope`.
    Key {
        id: KeyId,
        access: KeyAccess,
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
    /// The notes the person chose, when a task is limited to notes.
    notes: Option<HashSet<String>>,
    /// Where a task may write; `None` is wherever it can see.
    write_books: Option<HashSet<String>>,
    /// Where a task's new notes go.
    new_note_book: Option<String>,
    locked_books: HashSet<String>,
    /// Tag names by ID, to find `#ai-exclude`.
    tag_names: HashMap<String, String>,
    /// The notes shown so far, in order.
    shown: std::cell::RefCell<Vec<String>>,
    /// A task's read budget: the notes it has read, and the most it may.
    budget: Option<(HashSet<String>, usize)>,
    /// The notes a budgeted request has let through, once each.
    spent: std::cell::RefCell<HashSet<String>>,
    /// Where the lines shown are numbered, when a token is asking.
    numbering: Option<(crate::tokens::TokenStore, TokenId)>,
    /// Whether notebooks and tags carry a note count, of the notes this
    /// caller sees (API keys). Agents get none.
    keep_counts: bool,
}

/// `2026-10-14` for a time in milliseconds since the epoch.
pub(crate) fn day_of(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|at| at.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
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
            notes: scope.notes.clone(),
            write_books: scope.write_books.clone(),
            new_note_book: scope.new_note_book.clone(),
            locked_books,
            tag_names,
            shown: Default::default(),
            budget: None,
            spent: Default::default(),
            numbering: None,
            keep_counts: false,
        })
    }

    /// Gives notebooks and tags a count of the notes this caller sees,
    /// never the library's own count, which includes locked, trashed,
    /// excluded and out-of-scope notes.
    pub(crate) fn with_counts(mut self) -> Visibility<'a> {
        self.keep_counts = true;
        self
    }

    pub(crate) fn keeps_counts(&self) -> bool {
        self.keep_counts
    }

    /// Whether notebook `id` is one the agent may see.
    pub fn book_visible(&self, id: &str) -> bool {
        DocType::of_id(id) == Some(DocType::Book)
            && id != TRASH_BOOK_ID
            && !self.locked_books.contains(id)
            && self.books.as_ref().is_none_or(|books| books.contains(id))
    }

    /// The notebook a task's new notes go in, when the person chose one.
    pub fn new_note_book(&self) -> Option<&str> {
        self.new_note_book.as_deref()
    }

    /// Whether a task may write into notebook `id` (a new note, a move, a
    /// merge target). Narrower than [`Visibility::book_visible`] for a task
    /// with a write scope.
    pub fn book_writable(&self, id: &str) -> bool {
        match &self.write_books {
            None => self.book_visible(id),
            Some(allowed) => {
                allowed.contains(id)
                    && DocType::of_id(id) == Some(DocType::Book)
                    && id != TRASH_BOOK_ID
                    && !self.locked_books.contains(id)
            }
        }
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
        if self.notes.as_ref().is_some_and(|notes| !notes.contains(id)) {
            return None;
        }
        let note = self.store.note(id).ok()??;
        if note.is_trashed() || !self.book_visible(&note.book_id) {
            return None;
        }
        (!self.excluded(&note)).then_some(note)
    }

    /// Whether `note` is tagged `#ai-exclude`.
    pub(crate) fn excluded(&self, note: &Note) -> bool {
        note.tags
            .iter()
            .any(|tag| kanso_core::tags::is_ai_exclude(tag) || self.tag_is_ai_exclude(tag))
    }

    /// Whether tag `id` is the `#ai-exclude` tag, which no caller sees or
    /// changes: renaming or deleting it would show the notes it hides.
    pub(crate) fn tag_is_ai_exclude(&self, id: &str) -> bool {
        self.tag_names
            .get(id)
            .is_some_and(|name| kanso_core::tags::is_ai_exclude(name))
    }

    /// The store this visibility reads.
    pub(crate) fn store(&self) -> &'a Store {
        self.store
    }

    /// Limits this request to the notes left of a read budget: `read` are
    /// the notes already shown, `max` the most one task may be shown.
    pub fn limit_reads(&mut self, read: Vec<String>, max: usize) {
        self.budget = Some((read.into_iter().collect(), max));
    }

    /// Whether the agent may see note `id`. With a read budget, a note not
    /// read yet is let through only while the budget has room, so a list or
    /// a search can't show more than the task may read.
    pub fn allows(&self, id: &str) -> bool {
        if self.note(id).is_none() {
            return false;
        }
        let Some((read, max)) = &self.budget else {
            return true;
        };
        let mut spent = self.spent.borrow_mut();
        if read.contains(id) || spent.contains(id) {
            return true;
        }
        if read.len() + spent.len() >= *max {
            return false;
        }
        spent.insert(id.to_owned());
        true
    }

    /// Lets this request number the lines it shows against token `id` of
    /// `tokens`.
    pub(crate) fn attach(&mut self, tokens: crate::tokens::TokenStore, id: TokenId) {
        self.numbering = Some((tokens, id));
    }

    /// Records that `lines` of `note` were shown, and the number they carry.
    /// 0 when no token is attached (the lines are not numbered).
    pub(crate) fn show_lines(&self, note: &Note, lines: Vec<(u32, String)>) -> usize {
        let Some((tokens, id)) = &self.numbering else {
            return 0;
        };
        self.show(&note.id);
        tokens.show_lines(
            *id,
            &note.id,
            &note.title,
            Some(day_of(note.updated_at)),
            lines,
        )
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
