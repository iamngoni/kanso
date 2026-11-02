//! The change set of an agent task: everything the agent proposed, held in
//! memory for the person to review and apply in one go.
//!
//! The model is plain data. Nothing here writes the store while the agent
//! works: its tools only add to the set ([`ChangeSet::stage_create`] and
//! friends), reading the store through the request's
//! [`Visibility`](crate::scope::Visibility). The one way a set reaches the
//! store is [`ChangeSet::apply`], which the app calls when the person
//! presses Apply: it re-checks what the agent may still change, lowers the
//! ticked ops to one batch and writes it in one transaction as the agent's
//! edit approved by the person ([`kanso_store::Store::apply_batch`]), so one
//! [`undo`] takes all of it back.
//!
//! Sets are never synced and never written to disk: they hold note text the
//! person has not approved.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use kanso_core::model::{PUBLISH_FIELD, SHARE_FIELD, TRASH_BOOK_ID};
use kanso_store::{BatchWriter, PluginCreate, PluginEdit, Store, StoreError, agent_batch_id};

use crate::review::MAX_TEXT_BYTES;
use crate::scope::{AgentScope, Visibility};

/// What a new note's temporary ID starts with. It looks like a note ID so
/// the standard `update-note` and `patch-note` schemas accept it.
pub const TEMP_PREFIX: &str = "note:new-";

/// The most ops one task may propose.
pub const MAX_OPS: usize = 500;
/// The most text, in bytes, one change set may hold.
pub const MAX_SET_BYTES: usize = 8 << 20;
/// The fewest and most notes one merge may combine.
pub const MERGE_SOURCES: std::ops::RangeInclusive<usize> = 2..=50;
/// A change set touching more notes than this gets a warning at review.
pub const WIDE_SET: usize = 50;

/// Which agent task, for as long as the app runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(pub u64);

/// A merge's target note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MergeTarget {
    /// A new note, in `book_id`.
    New { temp: String, book_id: String },
    /// An existing note, replaced by the merged title and text.
    Existing { note_id: String, base_rev: String },
}

/// One thing the agent proposed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProposedOp {
    Create {
        /// The temporary ID later calls in the same set can target.
        temp: String,
        book_id: String,
        title: String,
        body: String,
        /// Note IDs it cites.
        sources: Vec<String>,
    },
    Edit {
        note_id: String,
        base_rev: String,
        old_title: String,
        old_body: String,
        new_title: String,
        new_body: String,
    },
    Move {
        note_id: String,
        base_rev: String,
        from_book: String,
        to_book: String,
    },
    Merge {
        into: MergeTarget,
        /// `(id, base_rev)` of each source.
        sources: Vec<(String, String)>,
        title: String,
        body: String,
    },
}

/// What kind of card an op is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpKind {
    Create,
    Edit,
    Merge,
    Move,
}

impl ProposedOp {
    pub fn kind(&self) -> OpKind {
        match self {
            ProposedOp::Create { .. } => OpKind::Create,
            ProposedOp::Edit { .. } => OpKind::Edit,
            ProposedOp::Merge { .. } => OpKind::Merge,
            ProposedOp::Move { .. } => OpKind::Move,
        }
    }

    /// The existing notes this op reads or changes, with the revision the
    /// agent saw each at.
    pub fn touched(&self) -> Vec<(&str, &str)> {
        match self {
            ProposedOp::Create { .. } => Vec::new(),
            ProposedOp::Edit {
                note_id, base_rev, ..
            }
            | ProposedOp::Move {
                note_id, base_rev, ..
            } => vec![(note_id, base_rev)],
            ProposedOp::Merge { into, sources, .. } => {
                let mut all: Vec<(&str, &str)> = sources
                    .iter()
                    .map(|(id, rev)| (id.as_str(), rev.as_str()))
                    .collect();
                if let MergeTarget::Existing { note_id, base_rev } = into {
                    all.push((note_id, base_rev));
                }
                all
            }
        }
    }

    /// The notebooks this op writes into.
    pub fn target_books(&self) -> Vec<&str> {
        match self {
            ProposedOp::Create { book_id, .. } => vec![book_id],
            ProposedOp::Move { to_book, .. } => vec![to_book],
            ProposedOp::Merge {
                into: MergeTarget::New { book_id, .. },
                ..
            } => vec![book_id],
            _ => Vec::new(),
        }
    }

    fn text_bytes(&self) -> usize {
        match self {
            ProposedOp::Create { title, body, .. } | ProposedOp::Merge { title, body, .. } => {
                title.len() + body.len()
            }
            ProposedOp::Edit {
                new_title,
                new_body,
                old_title,
                old_body,
                ..
            } => new_title.len() + new_body.len() + old_title.len() + old_body.len(),
            ProposedOp::Move { .. } => 0,
        }
    }
}

/// How many ops of each kind a set holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub creates: usize,
    pub edits: usize,
    pub moves: usize,
    pub merges: usize,
}

impl Counts {
    pub fn total(&self) -> usize {
        self.creates + self.edits + self.moves + self.merges
    }

    /// "3 new notes, 5 edits, 6 moves", leaving out what is zero.
    pub fn describe(&self) -> String {
        let plural =
            |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
        let mut parts = Vec::new();
        if self.creates > 0 {
            parts.push(plural(self.creates, "new note", "new notes"));
        }
        if self.edits > 0 {
            parts.push(plural(self.edits, "edit", "edits"));
        }
        if self.merges > 0 {
            parts.push(plural(self.merges, "merge", "merges"));
        }
        if self.moves > 0 {
            parts.push(plural(self.moves, "move", "moves"));
        }
        if parts.is_empty() {
            "no changes".to_owned()
        } else {
            parts.join(", ")
        }
    }
}

/// Why a staging call was refused. The text is what the agent is told.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused(pub String);

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn refused<T>(text: &str) -> Result<T, Refused> {
    Err(Refused(text.to_owned()))
}

/// The change set of one task.
#[derive(Clone, Debug)]
pub struct ChangeSet {
    pub task: TaskId,
    pub goal: String,
    /// What the person calls the agent.
    pub agent: String,
    pub scope: Arc<AgentScope>,
    pub ops: Vec<ProposedOp>,
    /// Text held, for the limit.
    pub bytes: usize,
    /// The agent's closing message (`kanso-task-done`), or its last words.
    pub summary: String,
    /// The agent called `kanso-task-done`.
    pub done: bool,
    /// The notebooks the person named: those they shared and the one new
    /// notes go in. A move or a new note elsewhere gets a warning.
    pub named_books: BTreeSet<String>,
    next_temp: u64,
}

/// The state of an op against the store now, for its card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpState {
    /// As the agent read it.
    Fresh,
    /// A note changed since the agent read it; the first is the note.
    Changed { note_id: String },
    /// A note, or a target notebook, can no longer be changed by an agent
    /// (locked, trashed, `#ai-exclude`, or outside the scope since).
    Blocked,
}

/// Which ops the person ticked.
#[derive(Clone, Debug, Default)]
pub struct Selection {
    /// Indexes into [`ChangeSet::ops`].
    pub ticked: BTreeSet<usize>,
    /// The ticked ops whose conflict the person chose to override.
    pub apply_anyway: BTreeSet<usize>,
    /// The ticked merges whose source notes stay where they are instead of
    /// moving to the Trash ("Keep the original notes").
    pub keep_sources: BTreeSet<usize>,
}

impl Selection {
    pub fn all(count: usize) -> Selection {
        Selection {
            ticked: (0..count).collect(),
            apply_anyway: BTreeSet::new(),
            keep_sources: BTreeSet::new(),
        }
    }
}

/// What [`ChangeSet::apply`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppliedSet {
    /// The batch ID, for [`undo`].
    pub batch: String,
    /// Notes changed, new notes made.
    pub changed: Vec<String>,
    pub created: Vec<String>,
    /// Ticked ops dropped because their notes could no longer be changed.
    pub dropped: Vec<usize>,
}

/// Why Apply wrote nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyFailure {
    /// A note changed since the agent read it and was not marked "apply
    /// anyway". Nothing was written.
    Conflict,
    /// Nothing ticked can still be applied.
    NothingToApply,
    /// The store refused; nothing was written.
    Failed(String),
}

impl ApplyFailure {
    pub fn message(&self) -> String {
        match self {
            ApplyFailure::Conflict => "A note changed since the agent read it.".into(),
            ApplyFailure::NothingToApply => {
                "None of the ticked changes can be applied any more.".into()
            }
            ApplyFailure::Failed(text) => text.clone(),
        }
    }
}

impl ChangeSet {
    pub fn new(
        task: TaskId,
        goal: &str,
        agent: &str,
        scope: Arc<AgentScope>,
        named_books: BTreeSet<String>,
    ) -> ChangeSet {
        ChangeSet {
            task,
            goal: goal.to_owned(),
            agent: agent.to_owned(),
            scope,
            ops: Vec::new(),
            bytes: 0,
            summary: String::new(),
            done: false,
            named_books,
            next_temp: 1,
        }
    }

    pub fn counts(&self) -> Counts {
        let mut counts = Counts::default();
        for op in &self.ops {
            match op.kind() {
                OpKind::Create => counts.creates += 1,
                OpKind::Edit => counts.edits += 1,
                OpKind::Move => counts.moves += 1,
                OpKind::Merge => counts.merges += 1,
            }
        }
        counts
    }

    /// The agent's answer to a staging call: what is proposed so far.
    pub fn ack(&self) -> String {
        format!(
            "Proposed. {} so far. Nothing is written until the person approves.",
            self.counts().describe()
        )
    }

    fn recount(&mut self) {
        self.bytes = self.ops.iter().map(ProposedOp::text_bytes).sum();
    }

    fn room(&self, adding: usize) -> Result<(), Refused> {
        if adding > MAX_TEXT_BYTES {
            return refused("That proposed text is too large.");
        }
        if self.bytes + adding > MAX_SET_BYTES {
            return refused("This task has proposed as much text as it may. Finish up.");
        }
        Ok(())
    }

    fn temp_index(&self, temp: &str) -> Option<usize> {
        self.ops.iter().position(|op| match op {
            ProposedOp::Create { temp: t, .. } => t == temp,
            ProposedOp::Merge {
                into: MergeTarget::New { temp: t, .. },
                ..
            } => t == temp,
            _ => false,
        })
    }

    /// Whether `id` is a temporary ID this set handed out.
    pub fn is_temp(id: &str) -> bool {
        id.starts_with(TEMP_PREFIX)
    }

    fn fresh_temp(&mut self) -> String {
        let temp = format!("{TEMP_PREFIX}{}", self.next_temp);
        self.next_temp += 1;
        temp
    }

    /// Proposes a new note. Returns its temporary ID.
    pub fn stage_create(
        &mut self,
        book_id: &str,
        title: &str,
        body: &str,
        sources: Vec<String>,
    ) -> Result<String, Refused> {
        if !self.scope.task_ops.create {
            return refused("This task may not propose new notes.");
        }
        self.room(title.len() + body.len())?;
        if self.ops.len() >= MAX_OPS {
            return refused("This task has proposed as many changes as it may. Finish up.");
        }
        let temp = self.fresh_temp();
        self.ops.push(ProposedOp::Create {
            temp: temp.clone(),
            book_id: book_id.to_owned(),
            title: title.to_owned(),
            body: body.to_owned(),
            sources,
        });
        self.recount();
        Ok(temp)
    }

    /// Proposes a new title and/or text for `note_id` (a real note, read at
    /// `base_rev` with `old_*`) or for a new note of this set (a temporary
    /// ID). A later edit of the same note replaces the earlier one.
    #[allow(clippy::too_many_arguments)]
    pub fn stage_edit(
        &mut self,
        note_id: &str,
        base_rev: &str,
        old: (&str, &str),
        new_title: Option<&str>,
        new_body: Option<&str>,
    ) -> Result<(), Refused> {
        if Self::is_temp(note_id) {
            let at = self
                .temp_index(note_id)
                .ok_or_else(|| Refused("No such note.".into()))?;
            self.room(new_title.map_or(0, str::len) + new_body.map_or(0, str::len))?;
            match &mut self.ops[at] {
                ProposedOp::Create { title, body, .. } | ProposedOp::Merge { title, body, .. } => {
                    if let Some(t) = new_title {
                        *title = t.to_owned();
                    }
                    if let Some(b) = new_body {
                        *body = b.to_owned();
                    }
                }
                _ => {}
            }
            self.recount();
            return Ok(());
        }
        if !self.scope.task_ops.edit {
            return refused("This task may not propose edits.");
        }
        let existing = self.ops.iter().position(|op| {
            matches!(op, ProposedOp::Edit { note_id: id, base_rev: rev, .. } if id == note_id && rev == base_rev)
        });
        let (title, body) = match existing.map(|at| &self.ops[at]) {
            Some(ProposedOp::Edit {
                new_title,
                new_body,
                ..
            }) => (new_title.clone(), new_body.clone()),
            _ => (old.0.to_owned(), old.1.to_owned()),
        };
        let new_title = new_title.map_or(title, str::to_owned);
        let new_body = new_body.map_or(body, str::to_owned);
        let edit = ProposedOp::Edit {
            note_id: note_id.to_owned(),
            base_rev: base_rev.to_owned(),
            old_title: old.0.to_owned(),
            old_body: old.1.to_owned(),
            new_title,
            new_body,
        };
        match existing {
            Some(at) => {
                self.room(edit.text_bytes().saturating_sub(self.ops[at].text_bytes()))?;
                self.ops[at] = edit;
            }
            None => {
                self.room(edit.text_bytes())?;
                if self.ops.len() >= MAX_OPS {
                    return refused("This task has proposed as many changes as it may. Finish up.");
                }
                self.ops.push(edit);
            }
        }
        self.recount();
        Ok(())
    }

    /// Proposes moving a note to `to_book`. A later move of the same note
    /// replaces the earlier one; moving a note of this set changes where it
    /// will be made.
    pub fn stage_move(
        &mut self,
        note_id: &str,
        base_rev: &str,
        from_book: &str,
        to_book: &str,
    ) -> Result<(), Refused> {
        if Self::is_temp(note_id) {
            let at = self
                .temp_index(note_id)
                .ok_or_else(|| Refused("No such note.".into()))?;
            match &mut self.ops[at] {
                ProposedOp::Create { book_id, .. }
                | ProposedOp::Merge {
                    into: MergeTarget::New { book_id, .. },
                    ..
                } => *book_id = to_book.to_owned(),
                _ => {}
            }
            return Ok(());
        }
        if !self.scope.task_ops.moves {
            return refused("This task may not propose moves between notebooks.");
        }
        if from_book == to_book {
            return refused("The note is already in that notebook.");
        }
        let op = ProposedOp::Move {
            note_id: note_id.to_owned(),
            base_rev: base_rev.to_owned(),
            from_book: from_book.to_owned(),
            to_book: to_book.to_owned(),
        };
        let existing = self
            .ops
            .iter()
            .position(|o| matches!(o, ProposedOp::Move { note_id: id, .. } if id == note_id));
        match existing {
            Some(at) => self.ops[at] = op,
            None => {
                if self.ops.len() >= MAX_OPS {
                    return refused("This task has proposed as many changes as it may. Finish up.");
                }
                self.ops.push(op);
            }
        }
        Ok(())
    }

    /// Proposes merging `sources` into one note. Returns the target's
    /// temporary ID when it is a new note.
    pub fn stage_merge(
        &mut self,
        sources: Vec<(String, String)>,
        into: Option<(String, String)>,
        book_id: Option<String>,
        title: &str,
        body: &str,
    ) -> Result<Option<String>, Refused> {
        if !self.scope.task_ops.merge_allowed() {
            return refused("This task may not propose merges.");
        }
        if !MERGE_SOURCES.contains(&sources.len()) {
            return refused("A merge combines 2 to 50 notes.");
        }
        let unique: BTreeSet<&String> = sources.iter().map(|(id, _)| id).collect();
        if unique.len() != sources.len() {
            return refused("A merge lists each source note once.");
        }
        if let Some((id, _)) = &into
            && unique.contains(id)
        {
            return refused("The merged note can't also be one of the sources.");
        }
        // A note is a source or a target of one merge, not both: the
        // target's merged text would be built on notes that are leaving.
        for op in &self.ops {
            let ProposedOp::Merge {
                into: other_into,
                sources: other_sources,
                ..
            } = op
            else {
                continue;
            };
            // A merge over the same sources is replaced below.
            if other_sources.iter().any(|(id, _)| unique.contains(id)) {
                continue;
            }
            let into_is_other_source = into
                .as_ref()
                .is_some_and(|(id, _)| other_sources.iter().any(|(s, _)| s == id));
            let source_is_other_target = matches!(
                other_into,
                MergeTarget::Existing { note_id, .. } if unique.contains(note_id)
            );
            if into_is_other_source || source_is_other_target {
                return refused(
                    "A note can't be the target of one merge and a source of another. Merge in one step.",
                );
            }
        }
        self.room(title.len() + body.len())?;
        if self.ops.len() >= MAX_OPS {
            return refused("This task has proposed as many changes as it may. Finish up.");
        }
        let (target, temp) = match (into, book_id) {
            (Some((note_id, base_rev)), _) => (MergeTarget::Existing { note_id, base_rev }, None),
            (None, Some(book_id)) => {
                let temp = self.fresh_temp();
                (
                    MergeTarget::New {
                        temp: temp.clone(),
                        book_id,
                    },
                    Some(temp),
                )
            }
            (None, None) => return refused("Give `into` or a `bookId` for the merged note."),
        };
        // A source can be in one merge only.
        self.ops.retain(|op| match op {
            ProposedOp::Merge { sources: s, .. } => !s.iter().any(|(id, _)| unique.contains(id)),
            _ => true,
        });
        self.ops.push(ProposedOp::Merge {
            into: target,
            sources,
            title: title.to_owned(),
            body: body.to_owned(),
        });
        self.recount();
        Ok(temp)
    }

    /// The title and text now proposed for existing note `note_id` read at
    /// `rev`, when an edit of it is waiting.
    pub fn pending_text(&self, note_id: &str, rev: &str) -> Option<(String, String)> {
        self.ops.iter().find_map(|op| match op {
            ProposedOp::Edit {
                note_id: id,
                base_rev,
                new_title,
                new_body,
                ..
            } if id == note_id && base_rev == rev => Some((new_title.clone(), new_body.clone())),
            _ => None,
        })
    }

    /// The agent's closing message.
    pub fn finish(&mut self, summary: &str) {
        self.summary = summary.trim().to_owned();
        self.done = true;
    }

    /// The set as the agent sees it (`kanso-list-proposed`).
    pub fn describe_for_agent(&self) -> serde_json::Value {
        use serde_json::json;
        let ops: Vec<_> = self
            .ops
            .iter()
            .enumerate()
            .map(|(i, op)| match op {
                ProposedOp::Create {
                    temp,
                    book_id,
                    title,
                    ..
                } => json!({ "index": i, "kind": "create", "id": temp, "bookId": book_id, "title": title }),
                ProposedOp::Edit {
                    note_id, new_title, ..
                } => json!({ "index": i, "kind": "edit", "id": note_id, "title": new_title }),
                ProposedOp::Move {
                    note_id, to_book, ..
                } => json!({ "index": i, "kind": "move", "id": note_id, "bookId": to_book }),
                ProposedOp::Merge {
                    into,
                    sources,
                    title,
                    ..
                } => json!({
                    "index": i, "kind": "merge", "title": title,
                    "into": match into {
                        MergeTarget::New { temp, .. } => temp.clone(),
                        MergeTarget::Existing { note_id, .. } => note_id.clone(),
                    },
                    "sources": sources.iter().map(|(id, _)| id).collect::<Vec<_>>(),
                }),
            })
            .collect();
        json!({ "summary": self.counts().describe(), "proposed": ops })
    }

    // --- review ---

    /// Whether op `index` is a note or notebook the person did not choose:
    /// a new note, or an edit/move of a note outside the chosen notes. Such
    /// cards start unticked.
    pub fn is_outside_chosen(&self, op: &ProposedOp) -> bool {
        let Some(chosen) = &self.scope.notes else {
            return false;
        };
        op.touched().iter().any(|(id, _)| !chosen.contains(*id))
    }

    /// Whether the card starts ticked: unticked for ops outside the notes
    /// the person chose.
    pub fn starts_ticked(&self, op: &ProposedOp) -> bool {
        !self.is_outside_chosen(op)
    }

    /// Warnings for the top of the review sheet: a set touching many notes,
    /// or moving notes into a notebook the person did not name.
    pub fn warnings(&self) -> Vec<String> {
        let mut out = Vec::new();
        let touched: BTreeSet<&str> = self
            .ops
            .iter()
            .flat_map(|op| op.touched().into_iter().map(|(id, _)| id))
            .collect();
        if touched.len() > WIDE_SET {
            out.push(format!(
                "This task changes {} notes. Look through it before you apply it.",
                touched.len()
            ));
        }
        let unnamed = self.ops.iter().any(|op| {
            matches!(op, ProposedOp::Move { .. })
                && op
                    .target_books()
                    .iter()
                    .any(|book| !self.named_books.contains(*book))
        });
        if unnamed {
            out.push("Some notes would move into a notebook you didn't name.".to_owned());
        }
        out
    }

    /// The state of op `op` against `store` now.
    pub fn state_of(&self, store: &Store, op: &ProposedOp) -> OpState {
        let Some(vis) = Visibility::new(store, &self.scope) else {
            return OpState::Blocked;
        };
        self.state_with(&vis, op)
    }

    fn state_with(&self, vis: &Visibility, op: &ProposedOp) -> OpState {
        for book in op.target_books() {
            if !vis.book_writable(book) {
                return OpState::Blocked;
            }
        }
        let mut changed = None;
        for (id, rev) in op.touched() {
            match vis.note(id) {
                None => return OpState::Blocked,
                Some(note) => {
                    if note.rev.as_deref() != Some(rev) && changed.is_none() {
                        changed = Some(id.to_owned());
                    }
                }
            }
        }
        match changed {
            Some(note_id) => OpState::Changed { note_id },
            None => OpState::Fresh,
        }
    }

    /// Whether any touched note is published or shared (marked on its card).
    pub fn is_published(&self, store: &Store, op: &ProposedOp) -> bool {
        op.touched().iter().any(|(id, _)| {
            store.get(id).ok().flatten().is_some_and(|doc| {
                doc.fields.contains_key(PUBLISH_FIELD) || doc.fields.contains_key(SHARE_FIELD)
            })
        })
    }

    // --- apply ---

    /// Lowers the ticked ops to one batch's edits and creates, each note
    /// once. `current_rev` gives a note's revision now (used when the person
    /// chose to apply over a change). Pure.
    pub fn lower(
        &self,
        ticked: &BTreeSet<usize>,
        apply_anyway: &BTreeSet<usize>,
        keep_sources: &BTreeSet<usize>,
        current_rev: &dyn Fn(&str) -> Option<String>,
    ) -> (Vec<PluginEdit>, Vec<PluginCreate>) {
        let mut edits: BTreeMap<String, PluginEdit> = BTreeMap::new();
        let mut order: Vec<String> = Vec::new();
        let mut creates = Vec::new();
        let rev_for = |index: usize, id: &str, base: &str| {
            if apply_anyway.contains(&index) {
                current_rev(id).unwrap_or_else(|| base.to_owned())
            } else {
                base.to_owned()
            }
        };
        let mut edit_of = |edits: &mut BTreeMap<String, PluginEdit>, id: &str, rev: String| {
            if !edits.contains_key(id) {
                order.push(id.to_owned());
            }
            edits.entry(id.to_owned()).or_insert_with(|| PluginEdit {
                id: id.to_owned(),
                base_rev: rev,
                ..PluginEdit::default()
            });
        };
        for (index, op) in self.ops.iter().enumerate() {
            if !ticked.contains(&index) {
                continue;
            }
            match op {
                ProposedOp::Create {
                    book_id,
                    title,
                    body,
                    ..
                } => creates.push(PluginCreate {
                    book_id: book_id.clone(),
                    title: title.clone(),
                    body: body.clone(),
                }),
                ProposedOp::Edit {
                    note_id,
                    base_rev,
                    new_title,
                    new_body,
                    ..
                } => {
                    edit_of(&mut edits, note_id, rev_for(index, note_id, base_rev));
                    let edit = edits.get_mut(note_id).expect("just inserted");
                    edit.title = Some(new_title.clone());
                    edit.body = Some(new_body.clone());
                }
                ProposedOp::Move {
                    note_id,
                    base_rev,
                    to_book,
                    ..
                } => {
                    edit_of(&mut edits, note_id, rev_for(index, note_id, base_rev));
                    let edit = edits.get_mut(note_id).expect("just inserted");
                    // A merge's trash wins over a move.
                    if edit.book_id.as_deref() != Some(TRASH_BOOK_ID) {
                        edit.book_id = Some(to_book.clone());
                    }
                }
                ProposedOp::Merge {
                    into,
                    sources,
                    title,
                    body,
                } => {
                    match into {
                        MergeTarget::New { book_id, .. } => creates.push(PluginCreate {
                            book_id: book_id.clone(),
                            title: title.clone(),
                            body: body.clone(),
                        }),
                        MergeTarget::Existing { note_id, base_rev } => {
                            edit_of(&mut edits, note_id, rev_for(index, note_id, base_rev));
                            let edit = edits.get_mut(note_id).expect("just inserted");
                            edit.title = Some(title.clone());
                            edit.body = Some(body.clone());
                        }
                    }
                    for (id, rev) in sources.iter().filter(|_| !keep_sources.contains(&index)) {
                        edit_of(&mut edits, id, rev_for(index, id, rev));
                        edits.get_mut(id).expect("just inserted").book_id =
                            Some(TRASH_BOOK_ID.to_owned());
                    }
                }
            }
        }
        (
            order
                .into_iter()
                .filter_map(|id| edits.remove(&id))
                .collect(),
            creates,
        )
    }

    /// Writes the ticked ops to `store` in one transaction, as the agent's
    /// edit approved by the person. Ops whose notes an agent may no longer
    /// change are dropped first (and reported); an op whose note changed
    /// since the agent read it fails the whole batch unless it is in
    /// `apply_anyway`. Nothing is written on failure.
    pub fn apply(
        &self,
        store: &Store,
        selection: &Selection,
        now_ms: i64,
    ) -> Result<AppliedSet, ApplyFailure> {
        let Some(vis) = Visibility::new(store, &self.scope) else {
            return Err(ApplyFailure::Failed("Couldn't read the notes.".into()));
        };
        let mut ticked = BTreeSet::new();
        let mut dropped = Vec::new();
        for &index in &selection.ticked {
            let Some(op) = self.ops.get(index) else {
                continue;
            };
            if self.state_with(&vis, op) == OpState::Blocked {
                dropped.push(index);
            } else {
                ticked.insert(index);
            }
        }
        let current_rev = |id: &str| store.get(id).ok().flatten().map(|doc| doc.rev);
        let (edits, creates) = self.lower(
            &ticked,
            &selection.apply_anyway,
            &selection.keep_sources,
            &current_rev,
        );
        if edits.is_empty() && creates.is_empty() {
            return Err(ApplyFailure::NothingToApply);
        }
        let batch = agent_batch_id();
        let writer = BatchWriter::Agent {
            agent: &self.agent,
            batch: &batch,
        };
        match store.apply_batch(writer, &edits, &creates, now_ms) {
            Ok(report) => Ok(AppliedSet {
                batch,
                changed: report.changed.into_iter().map(|w| w.id).collect(),
                created: report.created.into_iter().map(|w| w.id).collect(),
                dropped,
            }),
            Err(StoreError::Conflict) => Err(ApplyFailure::Conflict),
            Err(other) => Err(ApplyFailure::Failed(other.to_string())),
        }
    }
}

/// Takes back an applied set: every note it changed gets back what it had,
/// every note it made goes to the Trash (never deleted outright), unless the
/// person changed it since or its notebook was locked (those are left and
/// reported).
pub fn undo(
    store: &Store,
    batch: &str,
    now_ms: i64,
) -> Result<kanso_store::UndoReport, StoreError> {
    store.undo_agent_batch(batch, now_ms)
}

/// The change sets of the running tasks, shared with the server that
/// answers the agents.
#[derive(Clone, Default)]
pub struct ChangeSets {
    inner: Arc<Mutex<Sets>>,
}

#[derive(Default)]
struct Sets {
    sets: HashMap<TaskId, ChangeSet>,
    next: u64,
}

impl fmt::Debug for ChangeSets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChangeSets").finish_non_exhaustive()
    }
}

impl ChangeSets {
    fn lock(&self) -> MutexGuard<'_, Sets> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// A new task ID, to build the task's scope with.
    pub fn next_id(&self) -> TaskId {
        let mut inner = self.lock();
        inner.next += 1;
        TaskId(inner.next)
    }

    /// Opens the (empty) change set of a task.
    pub fn open(&self, set: ChangeSet) {
        self.lock().sets.insert(set.task, set);
    }

    /// Runs `f` on the set of `task`, when it is open.
    pub fn with<T>(&self, task: TaskId, f: impl FnOnce(&mut ChangeSet) -> T) -> Option<T> {
        self.lock().sets.get_mut(&task).map(f)
    }

    /// A copy of the set of `task`.
    pub fn get(&self, task: TaskId) -> Option<ChangeSet> {
        self.lock().sets.get(&task).cloned()
    }

    /// Forgets the set of `task` (discarded, or applied).
    pub fn remove(&self, task: TaskId) -> Option<ChangeSet> {
        self.lock().sets.remove(&task)
    }

    /// The tasks with a set open.
    pub fn tasks(&self) -> Vec<TaskId> {
        let mut tasks: Vec<_> = self.lock().sets.keys().copied().collect();
        tasks.sort();
        tasks
    }
}
