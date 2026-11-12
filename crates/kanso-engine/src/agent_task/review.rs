//! The cards of the Review agent changes sheet (desktop) and page (phone),
//! built from a task's change set. Pure: it reads the store, never writes.
//!
//! One card per op, new notes first, then edits, merges and moves. A card
//! carries what to draw (the note's text, a line diff, "Trip / Lisbon to
//! Archive / 2026") and whether it starts ticked: unticked when the note
//! changed since the agent read it, when the op touches a note outside the
//! ones the person chose, and never tickable when an agent may no longer
//! change the note.

use std::collections::HashMap;

use kanso_server::{
    AppliedSet, ChangeSet, Counts, MergeTarget, OpKind, OpState, ProposedOp, Selection,
};
use kanso_store::Store;

/// The most changed lines one card shows before "Show all".
pub const MAX_CARD_LINES: usize = 200;

/// Whether a line was added or removed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Added,
    Removed,
}

/// The changed lines between two texts and how many were added and removed,
/// at most `limit` of them. Pure.
pub fn diff_lines(
    before: &str,
    after: &str,
    limit: usize,
) -> (Vec<(LineKind, String)>, usize, usize) {
    let diff = similar::TextDiff::from_lines(before, after);
    let (mut lines, mut added, mut removed) = (Vec::new(), 0, 0);
    for change in diff.iter_all_changes() {
        let kind = match change.tag() {
            similar::ChangeTag::Insert => {
                added += 1;
                LineKind::Added
            }
            similar::ChangeTag::Delete => {
                removed += 1;
                LineKind::Removed
            }
            similar::ChangeTag::Equal => continue,
        };
        if lines.len() < limit {
            lines.push((kind, change.value().trim_end_matches('\n').to_owned()));
        }
    }
    (lines, added, removed)
}

/// A card's state against the notes now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardState {
    Fresh,
    /// A note changed since the agent read it: both versions are shown.
    Changed {
        current_title: String,
        current_body: String,
    },
    /// "This note can no longer be changed by an agent."
    Blocked,
}

/// What a card draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardDetail {
    New {
        body: String,
        /// Titles the text cites as `[[Title]]`.
        cites: Vec<String>,
    },
    Edit {
        title: Option<(String, String)>,
        lines: Vec<(LineKind, String)>,
        added: usize,
        removed: usize,
    },
    Move {
        from: String,
        to: String,
    },
    Merge {
        /// "Combines 6 notes into Lisbon plan": the sources' titles.
        sources: Vec<String>,
        /// The merged note's title.
        title: String,
        /// Whether the target is a new note (else it replaces an existing
        /// one and the lines are a diff against it).
        target_is_new: bool,
        lines: Vec<(LineKind, String)>,
        added: usize,
        removed: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewCard {
    /// The op's index in the change set.
    pub index: usize,
    pub kind: OpKind,
    pub title: String,
    /// The notebook, as "Trip / Lisbon".
    pub book: String,
    pub ticked: bool,
    pub state: CardState,
    /// "Not one of the notes you chose".
    pub outside: bool,
    /// "Published": a published or shared note.
    pub published: bool,
    /// The round of the session the op was proposed in.
    pub round: u32,
    pub detail: CardDetail,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewSheet {
    pub goal: String,
    pub agent: String,
    pub counts: Counts,
    /// The agent's last message, "From the agent".
    pub from_agent: String,
    pub warnings: Vec<String>,
    pub cards: Vec<ReviewCard>,
    /// Ops from more than one round are waiting, so each card says which.
    pub multi_round: bool,
}

impl ReviewSheet {
    /// "Proposed 3 new notes, 5 edits, 6 moves".
    pub fn header(&self) -> String {
        format!("Proposed {}", self.counts.describe())
    }

    /// What the cards start as: the ticked ones.
    pub fn default_selection(&self) -> Selection {
        Selection {
            ticked: self
                .cards
                .iter()
                .filter(|card| card.ticked)
                .map(|card| card.index)
                .collect(),
            apply_anyway: Default::default(),
            keep_sources: Default::default(),
        }
    }

    /// "Apply Selected (12)".
    pub fn apply_label(selection: &Selection) -> String {
        format!("Apply Selected ({})", selection.ticked.len())
    }
}

/// `[[Title]]` citations in `text`, once each, in order. Pure.
pub fn cites(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("[[") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else {
            break;
        };
        let title = after[..end].split('|').next().unwrap_or("").trim();
        if !title.is_empty() && !title.contains('\n') && !found.iter().any(|t| t == title) {
            found.push(title.to_owned());
        }
        rest = &after[end + 2..];
    }
    found
}

/// Notebook names by ID, as "Parent / Child" paths.
fn book_paths(store: &Store) -> HashMap<String, String> {
    let books = store.books().unwrap_or_default();
    let by_id: HashMap<&str, &kanso_core::Book> =
        books.iter().map(|b| (b.id.as_str(), b)).collect();
    let mut paths = HashMap::new();
    for book in &books {
        let mut names = vec![book.name.clone()];
        let mut at = book.parent_book_id.as_deref();
        let mut depth = 0;
        while let Some(parent) = at.and_then(|id| by_id.get(id)) {
            names.push(parent.name.clone());
            at = parent.parent_book_id.as_deref();
            depth += 1;
            if depth > 16 {
                break;
            }
        }
        names.reverse();
        paths.insert(book.id.clone(), names.join(" / "));
    }
    paths.insert(
        kanso_core::model::TRASH_BOOK_ID.to_owned(),
        "Trash".to_owned(),
    );
    paths
}

fn title_of(store: &Store, id: &str) -> String {
    store
        .note(id)
        .ok()
        .flatten()
        .map(|n| n.title)
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "Untitled".to_owned())
}

/// The review sheet for `set`, from the notes as they are now. `from_agent`
/// is the agent's last message when `set.summary` is empty.
pub fn build(store: &Store, set: &ChangeSet, from_agent: &str) -> ReviewSheet {
    let paths = book_paths(store);
    let path = |id: &str| {
        paths
            .get(id)
            .cloned()
            .unwrap_or_else(|| "Unknown notebook".into())
    };
    let mut cards = Vec::new();
    for (index, op) in set.ops.iter().enumerate() {
        let state = match set.state_of(store, op) {
            OpState::Fresh => CardState::Fresh,
            OpState::Blocked => CardState::Blocked,
            OpState::Changed { note_id } => {
                let now = store.note(&note_id).ok().flatten();
                CardState::Changed {
                    current_title: now.as_ref().map(|n| n.title.clone()).unwrap_or_default(),
                    current_body: now.map(|n| n.body).unwrap_or_default(),
                }
            }
        };
        let outside = set.is_outside_chosen(op);
        let ticked = state == CardState::Fresh && set.starts_ticked(op);
        let published = set.is_published(store, op);
        let (title, book, detail) = match op {
            ProposedOp::Create {
                book_id,
                title,
                body,
                ..
            } => (
                title.clone(),
                path(book_id),
                CardDetail::New {
                    body: body.clone(),
                    cites: cites(body),
                },
            ),
            ProposedOp::Edit {
                note_id,
                old_title,
                old_body,
                new_title,
                new_body,
                ..
            } => {
                let (lines, added, removed) = diff_lines(old_body, new_body, MAX_CARD_LINES);
                let book = store
                    .note(note_id)
                    .ok()
                    .flatten()
                    .map(|n| path(&n.book_id))
                    .unwrap_or_default();
                (
                    new_title.clone(),
                    book,
                    CardDetail::Edit {
                        title: (old_title != new_title)
                            .then(|| (old_title.clone(), new_title.clone())),
                        lines,
                        added,
                        removed,
                    },
                )
            }
            ProposedOp::Move {
                note_id,
                from_book,
                to_book,
                ..
            } => (
                title_of(store, note_id),
                path(to_book),
                CardDetail::Move {
                    from: path(from_book),
                    to: path(to_book),
                },
            ),
            ProposedOp::Merge {
                into,
                sources,
                title,
                body,
            } => {
                let (target_is_new, before, book) = match into {
                    MergeTarget::New { book_id, .. } => (true, String::new(), path(book_id)),
                    MergeTarget::Existing { note_id, .. } => {
                        let note = store.note(note_id).ok().flatten();
                        (
                            false,
                            note.as_ref().map(|n| n.body.clone()).unwrap_or_default(),
                            note.map(|n| path(&n.book_id)).unwrap_or_default(),
                        )
                    }
                };
                let (lines, added, removed) = diff_lines(&before, body, MAX_CARD_LINES);
                (
                    title.clone(),
                    book,
                    CardDetail::Merge {
                        sources: sources.iter().map(|(id, _)| title_of(store, id)).collect(),
                        title: title.clone(),
                        target_is_new,
                        lines,
                        added,
                        removed,
                    },
                )
            }
        };
        cards.push(ReviewCard {
            index,
            kind: op.kind(),
            title,
            book,
            ticked,
            state,
            outside,
            published,
            round: set.round_of(op),
            detail,
        });
    }
    // New notes, edits, merges, moves; each in the order proposed.
    cards.sort_by_key(|card| (card.kind, card.index));
    ReviewSheet {
        goal: set.goal.clone(),
        agent: set.agent.clone(),
        counts: set.counts(),
        from_agent: if set.summary.trim().is_empty() {
            from_agent.trim().to_owned()
        } else {
            set.summary.clone()
        },
        warnings: set.warnings(),
        cards,
        multi_round: set.rounds().len() > 1,
    }
}

/// The full diff of an edit or merge card, for "Show all".
pub fn all_lines(set: &ChangeSet, store: &Store, index: usize) -> Vec<(LineKind, String)> {
    match set.ops.get(index) {
        Some(ProposedOp::Edit {
            old_body, new_body, ..
        }) => diff_lines(old_body, new_body, usize::MAX).0,
        Some(ProposedOp::Merge { into, body, .. }) => {
            let before = match into {
                MergeTarget::Existing { note_id, .. } => store
                    .note(note_id)
                    .ok()
                    .flatten()
                    .map(|n| n.body)
                    .unwrap_or_default(),
                MergeTarget::New { .. } => String::new(),
            };
            diff_lines(&before, body, usize::MAX).0
        }
        _ => Vec::new(),
    }
}

/// "Round 2" on a card, when the sheet holds more than one round.
pub fn round_label(round: u32) -> String {
    format!("Round {round}")
}

/// "Applied 12 changes from Claude Agent."
pub fn applied_message(agent: &str, applied: &AppliedSet, selected: usize) -> String {
    let n = selected.saturating_sub(applied.dropped.len());
    format!(
        "Applied {n} change{} from {agent}.",
        if n == 1 { "" } else { "s" }
    )
}

/// What the notice says after an Undo that left some notes alone.
pub fn undo_message(skipped: usize) -> String {
    match skipped {
        0 => "Undone.".to_owned(),
        1 => "1 note changed since; it was left as it is.".to_owned(),
        n => format!("{n} notes changed since; they were left as they are."),
    }
}

/// How the history labels the version an agent task replaced.
pub fn history_label(agent: &str) -> String {
    format!("Before {agent}'s change (agent task)")
}
