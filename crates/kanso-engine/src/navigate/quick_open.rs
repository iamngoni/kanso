//! Open Note (⌘P): the notes whose titles or words fit what was typed, and
//! the headings that do, as the desktop's palette finds them in Notes mode.
//!
//! With nothing typed it lists the notes edited most recently. With words
//! typed it lists the notes whose titles have them, then the notes that have
//! them anywhere, then the headings that have them. A search with
//! qualifiers (`tag:work`) searches notes only. The notes in the trash are
//! never listed, and a locked note only once the lock session is open.

use kanso_core::outline::headings;
use kanso_core::query::find_ci;
use kanso_core::{Clause, Field};
use kanso_store::{HeadingHit, NoteSummary, SearchParams, Sort, Store, StoreError};

use super::model::{Found, LockedNotes, NoteHit};
use crate::fuzzy;
use crate::list::{Filter, search_params};
use crate::palette::model::{Entry, Section, Target};
use crate::palette::{order_sections, rank_section};

/// How many notes a search lists.
pub const NOTE_LIMIT: usize = 20;
/// How many headings Open Note lists under the notes.
pub const HEADING_LIMIT: usize = 8;

/// The search for notes whose titles have the query's words, or `None` when
/// it has none: its free text, asked of the title instead.
pub fn title_search(params: &SearchParams) -> Option<SearchParams> {
    let mut titled = params.clone();
    let mut any = false;
    for clause in &mut titled.query.clauses {
        if clause.field.is_none() && !clause.negated {
            *clause = Clause::field(Field::Title, std::mem::take(&mut clause.value));
            any = true;
        }
    }
    any.then_some(titled)
}

/// `first`, then the notes of `then` not already in it, at most `limit` in
/// all.
pub fn merge_by_id(
    first: Vec<NoteSummary>,
    then: Vec<NoteSummary>,
    limit: usize,
) -> Vec<NoteSummary> {
    let mut merged: Vec<NoteSummary> = Vec::with_capacity(limit);
    for note in first.into_iter().chain(then) {
        if merged.len() == limit {
            break;
        }
        if !merged.iter().any(|m| m.id == note.id) {
            merged.push(note);
        }
    }
    merged
}

/// The words Open Note looks for in headings: the query, when it is plain
/// words rather than a search with qualifiers. Pure.
pub fn heading_words(query: &str) -> Option<String> {
    let words = query.trim();
    (!words.is_empty() && !words.contains(':')).then(|| words.to_owned())
}

/// Searches the notes for `query`, and, with `headings`, their headings too
/// (Open Note does; the command palette lists notes only).
///
/// Locked notes are left out unless `locks` says the session is open; then
/// they are found by title like any note, and by heading from their opened
/// text.
pub fn search(
    store: &Store,
    query: &str,
    with_headings: bool,
    locks: &LockedNotes,
) -> Result<Found, StoreError> {
    let query = query.trim();
    let base = search_params(&Filter::All, query, Sort::UpdatedAt, true);
    let by_title = title_search(&base);
    // A limit counts what is kept, so when locked notes are dropped the
    // search is asked for more until enough are left, or the store has no more.
    let mut limit = NOTE_LIMIT;
    let notes = loop {
        let mut params = base.clone();
        params.limit = Some(limit);
        let titled = match &by_title {
            Some(titled) => store.search_summaries(&SearchParams {
                limit: Some(limit),
                ..titled.clone()
            })?,
            None => Vec::new(),
        };
        let anywhere = store.search_summaries(&params)?;
        let exhausted = titled.len() < limit && anywhere.len() < limit;
        let keep = |notes: Vec<NoteSummary>| -> Vec<NoteSummary> {
            notes
                .into_iter()
                .filter(|n| !n.locked || locks.can_read(&n.id))
                .collect()
        };
        let merged = merge_by_id(keep(titled), keep(anywhere), NOTE_LIMIT);
        if merged.len() == NOTE_LIMIT || exhausted {
            break merged;
        }
        limit *= 4;
    };
    let notes = notes
        .into_iter()
        .map(|n| NoteHit {
            id: n.id,
            title: n.title,
            book_id: n.book_id,
            locked: n.locked,
        })
        .collect();
    let headings = match heading_words(query).filter(|_| with_headings) {
        Some(words) => heading_hits(store, &words, locks)?,
        None => Vec::new(),
    };
    Ok(Found { notes, headings })
}

/// The headings that have `words`: the store's, minus those of locked notes
/// the session hasn't opened, plus those of the opened locked notes.
fn heading_hits(
    store: &Store,
    words: &str,
    locks: &LockedNotes,
) -> Result<Vec<HeadingHit>, StoreError> {
    let mut limit = HEADING_LIMIT;
    let mut hits = loop {
        let found = store.search_headings(words, limit)?;
        let asked = found.len();
        let mut kept = Vec::with_capacity(asked);
        for hit in found {
            if !store.note_is_locked(&hit.note_id)? {
                kept.push(hit);
            }
        }
        if kept.len() >= HEADING_LIMIT || asked < limit {
            break kept;
        }
        limit *= 4;
    };
    let mut opened: Vec<_> = locks.opened.iter().collect();
    opened.sort_by(|a, b| {
        b.updated_at
            .cmp(&a.updated_at)
            .then_with(|| a.id.cmp(&b.id))
    });
    for note in opened {
        for heading in headings(&note.body) {
            if find_ci(&heading.text, words.trim()).is_some() {
                hits.push(HeadingHit {
                    note_id: note.id.clone(),
                    note_title: note.title.clone(),
                    level: heading.level,
                    text: heading.text,
                    line: heading.line,
                });
            }
        }
    }
    hits.truncate(HEADING_LIMIT);
    Ok(hits)
}

/// A note's title as listed: "Untitled" for an empty one.
pub fn note_title(title: &str) -> String {
    if title.trim().is_empty() {
        "Untitled".to_owned()
    } else {
        title.to_owned()
    }
}

/// The entries for `notes`, each scored by how well `query` fits its title.
/// `open` is the note open now, checked; `book_names` give the hint beside
/// each.
pub fn note_entries(
    query: &str,
    notes: &[NoteHit],
    open: Option<&str>,
    book_names: &[(String, String)],
) -> Vec<(Entry, Option<i64>)> {
    notes
        .iter()
        .map(|note| {
            let title = note_title(&note.title);
            let score = fuzzy::score(query, &title);
            let entry = Entry {
                target: Target::Note {
                    id: note.id.clone(),
                },
                label: title,
                icon: if note.locked {
                    "lock-open"
                } else {
                    "file-text"
                }
                .to_owned(),
                description: None,
                hint: book_names
                    .iter()
                    .find(|(id, _)| *id == note.book_id)
                    .map(|(_, name)| name.clone()),
                checked: open == Some(note.id.as_str()),
                depth: 0,
                status: None,
            };
            (entry, score)
        })
        .collect()
}

/// The entries for `headings`, each scored by how well `query` fits its words.
fn heading_entries(query: &str, found: &[HeadingHit]) -> Vec<(Entry, Option<i64>)> {
    found
        .iter()
        .map(|hit| {
            let entry = Entry {
                target: Target::Heading {
                    note_id: hit.note_id.clone(),
                    line: hit.line,
                },
                label: hit.text.clone(),
                icon: "heading".to_owned(),
                description: None,
                hint: Some(note_title(&hit.note_title)),
                checked: false,
                depth: 0,
                status: None,
            };
            (entry, fuzzy::score(query, &hit.text))
        })
        .collect()
}

/// What Open Note lists for `query`: the notes (the most recently edited
/// ones with nothing typed), then the headings, the sections ordered by
/// their best entry once something is typed.
pub fn sections(
    query: &str,
    found: &Found,
    open: Option<&str>,
    book_names: &[(String, String)],
) -> Vec<Section> {
    let query = query.trim();
    let heading = if query.is_empty() {
        "Recently edited"
    } else {
        "Notes"
    };
    let mut sections = Vec::new();
    sections.extend(rank_section(
        query,
        heading,
        note_entries(query, &found.notes, open, book_names),
        true,
    ));
    sections.extend(rank_section(
        query,
        "Headings",
        heading_entries(query, &found.headings),
        true,
    ));
    order_sections(query, &mut sections);
    sections.into_iter().map(|(section, _)| section).collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use kanso_core::DocType;
    use kanso_store::{Origin, Put};
    use proptest::prelude::*;
    use serde_json::json;

    use super::*;
    use crate::navigate::model::OpenedNote;

    fn store() -> (tempfile::TempDir, Store, String) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        // A note needs a book that exists (the store checks the reference).
        let inbox = store
            .put(
                DocType::Book,
                json!({"name": "Inbox"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        (dir, store, inbox)
    }

    fn put(store: &Store, title: &str, body: &str, book: &str) -> String {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn words() -> impl Strategy<Value = String> {
        prop::sample::select(vec!["alpha", "beta", "gamma", "delta", "plan", "list"])
            .prop_map(str::to_owned)
    }

    /// A note: its title words, a heading word, whether it is trashed.
    fn notes() -> impl Strategy<Value = Vec<(Vec<String>, String, bool)>> {
        proptest::collection::vec(
            (
                proptest::collection::vec(words(), 1..3),
                words(),
                any::<bool>(),
            ),
            0..30,
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]

        /// Whatever the library and the query: at most the limits, no note
        /// twice, no note from the trash, and every note is one the query
        /// fits by title or by text.
        #[test]
        fn prop_quick_open_respects_limits_and_leaves_out_the_trash(
            library in notes(),
            query in prop_oneof![Just(String::new()), words(), "[a-z]{1,3}"],
        ) {
            let (_dir, store, inbox) = store();
            let mut trashed = HashSet::new();
            for (title, heading, in_trash) in &library {
                let book = if *in_trash { "trash" } else { inbox.as_str() };
                let id = put(&store, &title.join(" "), &format!("# {heading}\ntext"), book);
                if *in_trash {
                    trashed.insert(id);
                }
            }
            let found = search(&store, &query, true, &LockedNotes::closed()).unwrap();
            prop_assert!(found.notes.len() <= NOTE_LIMIT);
            prop_assert!(found.headings.len() <= HEADING_LIMIT);
            let ids: HashSet<&str> = found.notes.iter().map(|n| n.id.as_str()).collect();
            prop_assert_eq!(ids.len(), found.notes.len(), "a note twice");
            for note in &found.notes {
                prop_assert!(!trashed.contains(&note.id), "trash: {:?}", note);
                prop_assert_ne!(note.book_id.as_str(), "trash");
            }
            for hit in &found.headings {
                prop_assert!(!trashed.contains(&hit.note_id));
                prop_assert!(hit.text.to_lowercase().contains(&query.to_lowercase()));
            }
            if query.is_empty() {
                prop_assert!(found.headings.is_empty());
                // Everything outside the trash, up to the limit.
                let live = library.iter().filter(|(_, _, t)| !t).count();
                prop_assert_eq!(found.notes.len(), live.min(NOTE_LIMIT));
            }
        }

        /// Notes whose titles have the words come before the rest.
        #[test]
        fn prop_title_matches_come_first(library in notes(), word in words()) {
            let (_dir, store, inbox) = store();
            for (title, heading, _) in &library {
                put(&store, &title.join(" "), &format!("# {heading}\nthe {word} in text"), &inbox);
            }
            let found = search(&store, &word, false, &LockedNotes::closed()).unwrap();
            let titled = |n: &NoteHit| n.title.to_lowercase().contains(&word);
            let first_untitled = found.notes.iter().position(|n| !titled(n));
            if let Some(at) = first_untitled {
                prop_assert!(found.notes[at..].iter().all(|n| !titled(n)), "{:?}", found.notes);
            }
        }
    }

    // no-prop-test: the empty query is the recent notes, newest first, and
    // the sections say so; one fixed library.
    #[test]
    fn nothing_typed_lists_recent_notes_and_a_query_lists_notes_then_headings() {
        let (_dir, store, inbox) = store();
        let first = put(&store, "First", "# Plan\nx", &inbox);
        let second = put(&store, "Second", "# Plans\nx", &inbox);
        let found = search(&store, "", true, &LockedNotes::closed()).unwrap();
        assert_eq!(found.notes.len(), 2);
        let listed = sections("", &found, Some(&second), &[]);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].label, "Recently edited");
        let checked: Vec<bool> = listed[0].entries.iter().map(|e| e.checked).collect();
        assert_eq!(checked.iter().filter(|c| **c).count(), 1);

        let found = search(&store, "plan", true, &LockedNotes::closed()).unwrap();
        assert_eq!(
            found
                .notes
                .iter()
                .map(|n| n.id.as_str())
                .collect::<Vec<_>>(),
            [second.as_str(), first.as_str()]
        );
        assert_eq!(found.headings.len(), 2);
        let listed = sections("plan", &found, None, &[]);
        assert!(listed.iter().any(|s| s.label == "Headings"));
        // A search with a qualifier looks in notes only.
        let found = search(&store, "tag:work", true, &LockedNotes::closed()).unwrap();
        assert!(found.headings.is_empty());
    }

    // no-prop-test: one opened note's headings found after a fixed query.
    #[test]
    fn an_opened_notes_headings_are_found() {
        let (_dir, store, _inbox) = store();
        let locks = LockedNotes {
            opened: vec![OpenedNote {
                id: "note:locked".into(),
                title: "Vault".into(),
                body: "intro\n## Secret plan\n".into(),
                book_id: "book:inbox".into(),
                updated_at: 5,
            }],
        };
        let found = search(&store, "secret", true, &locks).unwrap();
        assert_eq!(found.headings.len(), 1);
        assert_eq!(found.headings[0].note_id, "note:locked");
        assert_eq!(found.headings[0].line, 1);
        assert!(
            search(&store, "secret", true, &LockedNotes::closed())
                .unwrap()
                .headings
                .is_empty()
        );
    }
}
