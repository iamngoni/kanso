//! What the person can do with a notebook's homepage: choose or make its
//! Home note, copy an AI section into it, and customise the order and the
//! hidden sections. Nothing here runs unasked. Kanso only ever marks a
//! note as Home (`kanso: home`) or removes the mark: it never writes the
//! note's own words, except to append a section the person copied.

use kanso_core::DocType;
use kanso_core::home::{SECTION_KEYS, home_text, is_home, mark_home, new_home_title, unmark_home};
use kanso_core::page_cache::Layout;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use super::PageSource;
use super::notebook::NotebookPage;

/// Why an action was refused.
#[derive(Debug, thiserror::Error)]
pub enum HomeError {
    #[error("{0}")]
    Store(#[from] kanso_store::StoreError),
    #[error("That notebook is locked.")]
    Locked,
    #[error("That note isn't in this notebook.")]
    NotInBook,
    #[error("A person's or project's page can't be a Home note.")]
    IsAPage,
    #[error("No such section.")]
    NoSuchSection,
}

fn body_of(store: &Store, id: &str) -> Result<(String, String), HomeError> {
    let doc = store.get(id)?;
    let fields = doc.as_ref().map(|d| &d.fields);
    let body = fields
        .and_then(|f| f.get("body").and_then(|b| b.as_str()))
        .unwrap_or("")
        .to_owned();
    let book = fields
        .and_then(|f| f.get("bookId").and_then(|b| b.as_str()))
        .unwrap_or("")
        .to_owned();
    Ok((body, book))
}

fn write_body(store: &Store, id: &str, body: &str, now: i64) -> Result<(), HomeError> {
    store.put(
        DocType::Note,
        json!({ "_id": id, "body": body, "updatedAt": now }),
        Put::Merge,
        Origin::Ui,
    )?;
    Ok(())
}

fn unmark_others(store: &Store, book: &str, keep: &str, now: i64) -> Result<(), HomeError> {
    for mark in store.home_notes_marked(book)? {
        if mark.id == keep {
            continue;
        }
        let (body, _) = body_of(store, &mark.id)?;
        let next = unmark_home(&body);
        if next != body {
            write_body(store, &mark.id, &next, now)?;
        }
    }
    Ok(())
}

/// Makes note `note_id` of notebook `book` its Home note: it gets
/// `kanso: home`, and any other note of the notebook that had it loses it.
pub fn choose_home(store: &Store, book: &str, note_id: &str, now: i64) -> Result<(), HomeError> {
    if store.book_subtree(book, false)?.is_empty() {
        return Err(HomeError::Locked);
    }
    if store.note_is_locked(note_id)? {
        return Err(HomeError::Locked);
    }
    let (body, in_book) = body_of(store, note_id)?;
    if in_book != book {
        return Err(HomeError::NotInBook);
    }
    if store.entity(note_id)?.is_some() {
        return Err(HomeError::IsAPage);
    }
    unmark_others(store, book, note_id, now)?;
    let next = mark_home(&body);
    if next != body {
        write_body(store, note_id, &next, now + 1)?;
    }
    Ok(())
}

/// Makes the notebook's Home note, "About <Notebook>", and returns its note
/// ID. The notebook's previous Home note, if any, becomes an ordinary note.
pub fn new_home(store: &Store, book: &str, name: &str, now: i64) -> Result<String, HomeError> {
    if store.book_subtree(book, false)?.is_empty() {
        return Err(HomeError::Locked);
    }
    let made = store.put(
        DocType::Note,
        json!({
            "title": new_home_title(name),
            "body": mark_home(""),
            "bookId": book,
            "createdAt": now,
            "updatedAt": now + 1,
        }),
        Put::Upsert,
        Origin::Ui,
    )?;
    unmark_others(store, book, &made.id, now)?;
    Ok(made.id)
}

/// Takes the Home note mark off the notebook's Home note.
pub fn clear_home(store: &Store, book: &str, now: i64) -> Result<bool, HomeError> {
    let mut changed = false;
    for mark in store.home_notes_marked(book)? {
        let (body, _) = body_of(store, &mark.id)?;
        let next = unmark_home(&body);
        if next != body {
            write_body(store, &mark.id, &next, now)?;
            changed = true;
        }
    }
    Ok(changed)
}

/// Appends a section's lines (`markdown`, with source links) to the
/// notebook's Home note under `heading`, making "About <Notebook>" when it
/// has none. Returns the Home note's ID, or `None` when there was nothing
/// to copy.
pub fn copy_into_home(
    store: &Store,
    book: &str,
    name: &str,
    heading: &str,
    markdown: &str,
    now: i64,
) -> Result<Option<String>, HomeError> {
    if markdown.trim().is_empty() {
        return Ok(None);
    }
    let id = match store.home_note(book)? {
        Some(mark) => mark.id,
        None => new_home(store, book, name, now)?,
    };
    let (body, _) = body_of(store, &id)?;
    let mut next = body;
    if !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(&format!(
        "\n## {}\n\n{}",
        heading.trim(),
        markdown.trim_end()
    ));
    next.push('\n');
    write_body(store, &id, &next, now + 2)?;
    Ok(Some(id))
}

/// After note `note_id` moved to another notebook: when it is a Home note
/// and the notebook it moved to already has one, its mark is removed.
/// Returns whether it was (the caller says so in a toast).
pub fn after_move(store: &Store, note_id: &str, now: i64) -> Result<bool, HomeError> {
    let (body, book) = body_of(store, note_id)?;
    if !is_home(&body) || book.is_empty() {
        return Ok(false);
    }
    let marks = store.home_notes_marked(&book)?;
    if marks.iter().any(|m| m.id != note_id) {
        write_body(store, note_id, &unmark_home(&body), now)?;
        return Ok(true);
    }
    Ok(false)
}

/// The Home note's text, for a quick read.
pub fn home_note_text(store: &Store, book: &str) -> Result<Option<String>, HomeError> {
    let Some(mark) = store.home_note(book)? else {
        return Ok(None);
    };
    let (body, _) = body_of(store, &mark.id)?;
    Ok(Some(home_text(&body).trim().to_owned()))
}

/// Sets the order of a homepage's sections and which are hidden. Keys the
/// page doesn't have are refused; the later write wins on sync.
pub fn customise(
    store: &Store,
    page: &NotebookPage,
    order: &[String],
    hidden: &[String],
    now: i64,
) -> Result<(), HomeError> {
    let known = |key: &String| SECTION_KEYS.contains(&key.as_str());
    if !order.iter().all(known) || !hidden.iter().all(known) {
        return Err(HomeError::NoSuchSection);
    }
    let mut cache = store.page_cache(page.doc_id())?;
    let mut order: Vec<String> = order.to_vec();
    order.dedup();
    let next = Layout { order, at: now };
    if cache.layout.as_ref().map(|l| &l.order) != Some(&next.order) {
        cache.layout = Some(next);
    }
    for key in SECTION_KEYS {
        let want = hidden.iter().any(|h| h == key);
        if cache.is_hidden(key) != want {
            cache.hidden.insert(
                key.to_owned(),
                kanso_core::page_cache::HiddenMark {
                    at: now,
                    hidden: want,
                },
            );
        }
    }
    store.set_page_cache(page.doc_id(), &cache)?;
    Ok(())
}

/// Puts the sections back in their default order and shows all of them.
pub fn reset_layout(store: &Store, page: &NotebookPage, now: i64) -> Result<(), HomeError> {
    customise(store, page, &[], &[], now)
}
