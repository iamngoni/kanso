//! Inbox triage (issue #195): the Inbox as a place you empty. Each note
//! there gets a card of suggestions (a notebook, tags, a title, links to
//! existing notes); a person accepts or rejects each chip, and filing
//! applies only what was accepted.
//!
//! The suggestions come from the library first: embeddings (the
//! notebooks and tags of the nearest notes), titles and existing links.
//! They need no AI and no network. With the person's own AI on, a model
//! ranks the library's candidates ([`ai`]); it never invents a notebook,
//! tag or link. A locked note, a note in a locked notebook, a trashed note
//! and a note tagged `#ai-exclude` are never shown, never used as
//! evidence (the store's embeddings already leave them out) and never sent
//! to a model.
//!
//! [`suggest`] reads the store; the rules are pure and live in [`notebook`],
//! [`title`] and [`links`] (the last is shared with the resurfacing of
//! related notes while writing). [`file`] turns accepted chips into one
//! write per note, in one batch, with an undo.

#[cfg(feature = "ai")]
pub mod ai;
pub mod file;
pub mod links;
pub mod model;
pub mod notebook;
pub mod resolve;
pub mod session;
pub mod title;

use std::collections::{HashMap, HashSet};

use kanso_core::Note;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_store::{Result, Store};

pub use model::{
    BECAUSE_MAX, Card, Field, Filed, Filing, Reason, Source, Suggestion, TriageOptions,
};
pub use resolve::resolve;

use notebook::{NEIGHBOURS, Vote};

/// How many tag candidates an AI may choose among.
pub const TAG_CANDIDATES: usize = 10;
/// How many notebook candidates an AI may choose among.
pub const BOOK_CANDIDATES: usize = 8;
/// How many link candidates an AI may choose among.
pub const LINK_CANDIDATES: usize = 8;
/// The score of a title taken from the note's first sentence.
const TITLE_SCORE: f32 = 0.6;

/// Cards for the notes Triage shows, newest first: the Inbox's notes not
/// yet triaged since they were last edited. `None` when there is no Inbox
/// yet.
pub fn cards(store: &Store, opts: &TriageOptions, limit: usize) -> Result<Vec<Card>> {
    let waiting = store.inbox_waiting(&opts.inbox_id)?;
    waiting
        .into_iter()
        .take(limit)
        .map(|n| card(store, &n.id, opts))
        .collect::<Result<Vec<_>>>()
        .map(|cards| cards.into_iter().flatten().collect())
}

/// The card for note `id`, if it can have one: it is live, in the Inbox,
/// outside the trash and not locked.
pub fn card(store: &Store, id: &str, opts: &TriageOptions) -> Result<Option<Card>> {
    let Some(note) = store.note(id)? else {
        return Ok(None);
    };
    if note.is_trashed()
        || note.is_locked()
        || note.book_id != opts.inbox_id
        || store.note_is_locked(id)?
    {
        return Ok(None);
    }
    let found = gather(store, &note, opts)?;
    Ok(Some(Card {
        note_id: note.id.clone(),
        rev: note.rev.clone().unwrap_or_default(),
        suggestions: confident(&found, opts),
        candidates: found.candidates,
        missing: found.missing,
    }))
}

/// The library's suggestions for note `id`. Nothing for a note that is
/// locked, trashed or gone.
pub fn suggest(store: &Store, id: &str, opts: &TriageOptions) -> Result<Vec<Suggestion>> {
    let Some(note) = store.note(id)? else {
        return Ok(Vec::new());
    };
    if note.is_trashed() || note.is_locked() || store.note_is_locked(id)? {
        return Ok(Vec::new());
    }
    Ok(confident(&gather(store, &note, opts)?, opts))
}

/// Remembers a rejection, so `suggest` leaves the suggestion out from now
/// on.
pub fn reject(store: &Store, note_id: &str, s: &Suggestion, now_ms: i64) -> Result<()> {
    store.dismiss_triage(note_id, s.field.as_str(), &s.value, now_ms)
}

/// Takes a rejection back.
pub fn unreject(store: &Store, note_id: &str, s: &Suggestion) -> Result<()> {
    store.restore_triage(note_id, s.field.as_str(), &s.value)
}

/// Everything the library finds for `note`, strongest first within a
/// field, without the rejected ones; and why notebooks and tags are
/// missing, if they are.
fn gather(store: &Store, note: &Note, opts: &TriageOptions) -> Result<Gathered> {
    let dismissed: HashSet<(String, String)> =
        store.triage_dismissed(&note.id)?.into_iter().collect();
    let kept =
        |s: &Suggestion| !dismissed.contains(&(s.field.as_str().to_owned(), s.value.clone()));
    let mut out: Vec<Suggestion> = Vec::new();
    let model = store.embedding_model()?;
    let mut missing = None;
    let mut picked = None;
    let mut near = Vec::new();
    match &model {
        None => missing = Some(Reason::RelatedOff),
        Some(model) => match store.nearest_notes(&note.id, model, NEIGHBOURS)? {
            None => missing = Some(Reason::NotEmbedded),
            Some(found) => near = found,
        },
    }
    // The near notes, read once.
    let mut notes: HashMap<String, Note> = HashMap::new();
    for n in &near {
        if let Some(found) = store.note(&n.id)? {
            notes.insert(n.id.clone(), found);
        }
    }
    if let (Some(model), None) = (&model, missing) {
        // Notebooks.
        let locked = store.locked_books()?;
        let books = store.books()?;
        let votes: Vec<Vote> = near
            .iter()
            .filter_map(|n| {
                notes.get(&n.id).map(|found| Vote {
                    book: found.book_id.clone(),
                    note: n.id.clone(),
                    similarity: n.score,
                })
            })
            .collect();
        let allowed = |book: &str| {
            book != opts.inbox_id
                && book != TRASH_BOOK_ID
                && !locked.contains(book)
                && books.iter().any(|b| b.id == book)
        };
        picked = notebook::pick(&votes, allowed).map(|p| p.book);
        for pick in notebook::rank(&votes, allowed)
            .into_iter()
            .take(BOOK_CANDIDATES)
        {
            let Some(book) = books.iter().find(|b| b.id == pick.book) else {
                continue;
            };
            out.push(Suggestion {
                field: Field::Notebook,
                value: pick.book.clone(),
                label: book.name.clone(),
                score: pick.score,
                because: pick.because,
                source: Source::Library,
                is_new: false,
            });
        }
        // Tags, with the near notes that have each.
        let wanted = TAG_CANDIDATES.max(opts.max_tags);
        let tags = store
            .tag_suggestions(&note.id, model, wanted + dismissed.len())?
            .unwrap_or_default();
        for tag in tags {
            let mut with: Vec<(&String, f32)> = near
                .iter()
                .filter(|n| {
                    notes
                        .get(&n.id)
                        .is_some_and(|found| found.tags.contains(&tag.tag_id))
                })
                .map(|n| (&n.id, n.score))
                .collect();
            with.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
            out.push(Suggestion {
                field: Field::Tag,
                value: tag.tag_id.clone(),
                label: tag.name.clone(),
                score: tag.weight.clamp(0., 1.),
                because: with
                    .into_iter()
                    .take(BECAUSE_MAX)
                    .map(|(id, _)| id.clone())
                    .collect(),
                source: Source::Library,
                is_new: false,
            });
        }
    }
    // The title.
    if opts.suggest_titles
        && let Some(title) = title::suggest(&note.title, &note.body)
    {
        out.push(Suggestion {
            field: Field::Title,
            label: title.clone(),
            value: title,
            score: TITLE_SCORE,
            because: Vec::new(),
            source: Source::Library,
            is_new: false,
        });
    }
    // Links.
    if opts.suggest_links {
        let found = links::candidates(store, &note.id, &note.body, model.as_deref())?;
        for link in found.into_iter().take(LINK_CANDIDATES) {
            out.push(Suggestion {
                field: Field::Link,
                label: link.title,
                because: vec![link.id.clone()],
                value: link.id,
                score: link.score,
                source: Source::Library,
                is_new: false,
            });
        }
    }
    out.retain(kept);
    Ok(Gathered {
        candidates: out,
        missing,
        picked_book: picked,
    })
}

/// What [`gather`] found.
struct Gathered {
    candidates: Vec<Suggestion>,
    missing: Option<Reason>,
    /// The notebook that clearly leads, if one does.
    picked_book: Option<String>,
}

/// The candidates worth showing: the notebook only when it clearly leads,
/// at most `max_tags` tags, the title, and the links.
fn confident(found: &Gathered, opts: &TriageOptions) -> Vec<Suggestion> {
    let of = |field: Field| found.candidates.iter().filter(move |s| s.field == field);
    let mut out: Vec<Suggestion> = of(Field::Notebook)
        .filter(|s| Some(&s.value) == found.picked_book.as_ref())
        .cloned()
        .collect();
    out.extend(of(Field::Tag).take(opts.max_tags).cloned());
    out.extend(of(Field::Title).cloned());
    out.extend(of(Field::Link).cloned());
    out
}

#[cfg(test)]
mod tests;
