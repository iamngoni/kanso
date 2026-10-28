//! Backlinks and unlinked mentions, as the store lists them, with the locked
//! notes handled: a locked note the lock session hasn't opened never shows,
//! and one it has opened shows like any other, found from its opened text
//! (the store holds none of it).

use std::collections::HashMap;

use kanso_core::links::{
    LinkTarget, line_text, mention_lines, normalize_title, note_links, plain_line,
};
use kanso_store::{Backlink, Store, StoreError};

use super::model::LockedNotes;

/// How long a context line gets, in characters: the store's.
const CONTEXT_CHARS: usize = 160;
/// The most unlinked mentions listed: the store's.
const MENTION_LIMIT: usize = 30;

/// Whether note `id` is locked and can't be read now.
pub fn hidden(store: &Store, id: &str, locks: &LockedNotes) -> Result<bool, StoreError> {
    Ok(!locks.can_read(id) && store.note_is_locked(id)?)
}

/// A note's context line, as plain words, shortened, as the store makes it.
pub fn context(body: &str, line: u32) -> String {
    let text = plain_line(line_text(body, line));
    if text.chars().count() <= CONTEXT_CHARS {
        return text;
    }
    let mut short: String = text.chars().take(CONTEXT_CHARS - 1).collect();
    short.push('…');
    short
}

/// The other live notes outside the trash that link to note `id` (by ID link
/// or by a wiki-link to its title), most recently changed first, each with
/// the line of its first link. Empty for a locked note that is closed.
pub fn backlinks(
    store: &Store,
    id: &str,
    locks: &LockedNotes,
) -> Result<Vec<Backlink>, StoreError> {
    if hidden(store, id, locks)? {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    // Notes in locked notebooks can have link rows; their context would be
    // empty too, so every locked note comes from its opened text instead.
    for link in store.backlinks(id)? {
        if !store.note_is_locked(&link.id)? {
            found.push(link);
        }
    }
    if locks.opened.is_empty() {
        return Ok(found);
    }
    let by_title = title_that_names(store, id)?;
    let mut added: Vec<(i64, Backlink)> = Vec::new();
    for note in locks.opened.iter().filter(|n| n.id != id) {
        if let Some(line) = first_link_line(&note.body, id, by_title.as_deref()) {
            added.push((
                note.updated_at,
                Backlink {
                    id: note.id.clone(),
                    title: note.title.clone(),
                    line,
                    context: context(&note.body, line),
                },
            ));
        }
    }
    Ok(merge_by_recency(store, found, added)?)
}

/// Live notes outside the trash that mention note `id`'s title as words of
/// their own without linking to it, most recently changed first. Empty for a
/// locked note that is closed.
pub fn unlinked_mentions(
    store: &Store,
    id: &str,
    locks: &LockedNotes,
) -> Result<Vec<Backlink>, StoreError> {
    if hidden(store, id, locks)? {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    for mention in store.unlinked_mentions(id)? {
        if !store.note_is_locked(&mention.id)? {
            found.push(mention);
        }
    }
    if locks.opened.is_empty() {
        return Ok(found);
    }
    let Some(title) = store
        .note(id)?
        .map(|n| n.title.trim().to_owned())
        .filter(|t| t.chars().count() >= 2)
    else {
        return Ok(found);
    };
    let normalized = normalize_title(&title);
    let mut added: Vec<(i64, Backlink)> = Vec::new();
    for note in locks.opened.iter().filter(|n| n.id != id) {
        if links_to(&note.body, id, &normalized) {
            continue;
        }
        if let Some(&line) = mention_lines(&note.body, &title).first() {
            added.push((
                note.updated_at,
                Backlink {
                    id: note.id.clone(),
                    title: note.title.clone(),
                    line,
                    context: context(&note.body, line),
                },
            ));
        }
    }
    let mut merged = merge_by_recency(store, found, added)?;
    merged.truncate(MENTION_LIMIT);
    Ok(merged)
}

/// The normalized title wiki-links to note `id` go by: its title, when that
/// title names it (and not an earlier note with the same title).
fn title_that_names(store: &Store, id: &str) -> Result<Option<String>, StoreError> {
    let Some(note) = store.note(id)? else {
        return Ok(None);
    };
    Ok(store
        .resolve_title(&note.title)?
        .filter(|(found, _)| found == id)
        .map(|_| normalize_title(&note.title)))
}

/// The line of the first link in `body` to note `id`, by ID or by the
/// wiki-link title `by_title`.
fn first_link_line(body: &str, id: &str, by_title: Option<&str>) -> Option<u32> {
    note_links(body)
        .into_iter()
        .filter(|link| match &link.target {
            LinkTarget::Note(target) => target == id,
            LinkTarget::Title(title) => by_title == Some(title.as_str()),
        })
        .map(|link| link.line)
        .min()
}

/// Whether `body` links to note `id`, by ID or by any wiki-link to the
/// normalized `title`: what keeps a note out of the unlinked mentions.
fn links_to(body: &str, id: &str, title: &str) -> bool {
    note_links(body).iter().any(|link| match &link.target {
        LinkTarget::Note(target) => target == id,
        LinkTarget::Title(found) => found == title,
    })
}

/// `store` rows and `added` rows (with their notes' change times) as one
/// list, the most recently changed first, a tie by ID.
fn merge_by_recency(
    store: &Store,
    stored: Vec<Backlink>,
    added: Vec<(i64, Backlink)>,
) -> Result<Vec<Backlink>, StoreError> {
    if added.is_empty() {
        return Ok(stored);
    }
    let ids: Vec<&str> = stored.iter().map(|b| b.id.as_str()).collect();
    let times: HashMap<String, i64> = store
        .summaries_of(&ids)?
        .into_iter()
        .map(|s| (s.id, s.updated_at))
        .collect();
    let mut all: Vec<(i64, Backlink)> = stored
        .into_iter()
        .map(|b| (times.get(&b.id).copied().unwrap_or(0), b))
        .chain(added)
        .collect();
    all.sort_by(|(at, a), (bt, b)| bt.cmp(at).then_with(|| a.id.cmp(&b.id)));
    Ok(all.into_iter().map(|(_, b)| b).collect())
}
