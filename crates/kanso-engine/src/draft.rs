//! Drafting from notes, as the desktop and the phones share it: which
//! notes a draft may read, in what order, and saving the result.
//!
//! The prompts, the map-reduce and the citation check are in
//! `kanso_ai::draft`. This is the rule about the library: every source
//! passes the one eligibility rule Ask uses (`ask::eligible_note`: not
//! locked, not in a locked notebook, not in the trash, not tagged
//! `#ai-exclude`); the sheet's counts say how many were left out, never
//! which; a draft is held in memory until the person saves it.

use std::collections::HashSet;

use kanso_ai::Excerpt;
use kanso_ai::draft::{MAX_SOURCE_CHARS, MAX_SOURCES, SourceRef};
use kanso_ai::draft_model::Source;
use kanso_core::query::Query;
use kanso_core::{DocType, TRASH_BOOK_ID};
use kanso_store::{Origin, Put, SearchParams, Store};
use serde_json::json;

use crate::ask;

/// A note a draft will read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceNote {
    pub id: String,
    pub title: String,
    pub words: usize,
}

/// What the sheet's Sources part shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceList {
    pub notes: Vec<SourceNote>,
    /// Locked, in a locked notebook, or in the trash: counted, never named.
    pub left_out_locked: usize,
    /// Tagged `#ai-exclude`.
    pub left_out_excluded: usize,
    /// More than a draft can read.
    pub too_many: bool,
}

impl SourceList {
    pub fn words(&self) -> usize {
        self.notes.iter().map(|n| n.words).sum()
    }

    /// "12 notes, about 9,000 words. 2 locked notes left out. 1 note
    /// tagged #ai-exclude left out."
    pub fn summary(&self) -> String {
        let n = self.notes.len();
        let mut line = format!(
            "{n} {}, about {} words.",
            if n == 1 { "note" } else { "notes" },
            group(self.words())
        );
        if self.left_out_locked > 0 {
            line.push_str(&format!(
                " {} locked {} left out.",
                self.left_out_locked,
                if self.left_out_locked == 1 {
                    "note"
                } else {
                    "notes"
                }
            ));
        }
        if self.left_out_excluded > 0 {
            line.push_str(&format!(
                " {} {} tagged #ai-exclude left out.",
                self.left_out_excluded,
                if self.left_out_excluded == 1 {
                    "note"
                } else {
                    "notes"
                }
            ));
        }
        line
    }
}

fn group(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn full_id(id: &str) -> String {
    if id.trim().starts_with("note:") {
        id.trim().to_owned()
    } else {
        format!("note:{}", id.trim())
    }
}

/// The notes of a notebook and the notebooks inside it, newest first, at
/// most [`MAX_SOURCES`]. Locked and trashed ones are still listed here;
/// [`resolve`] counts them out.
pub fn notes_of_book(store: &Store, book_id: &str) -> kanso_store::Result<Vec<String>> {
    let books = store.books()?;
    let mut wanted = vec![book_id.to_owned()];
    let mut i = 0;
    while i < wanted.len() {
        let parent = wanted[i].clone();
        wanted.extend(
            books
                .iter()
                .filter(|b| b.parent_book_id.as_deref() == Some(parent.as_str()))
                .map(|b| b.id.clone()),
        );
        i += 1;
    }
    let mut found = Vec::new();
    for book in &wanted {
        found.extend(search_ids(store, &format!("bookId:{book}"))?);
    }
    Ok(newest(found))
}

/// The notes with a tag, newest first, at most [`MAX_SOURCES`].
pub fn notes_of_tag(store: &Store, tag_id: &str) -> kanso_store::Result<Vec<String>> {
    Ok(newest(search_ids(store, &format!("tagId:{tag_id}"))?))
}

fn search_ids(store: &Store, query: &str) -> kanso_store::Result<Vec<(i64, String)>> {
    let params = SearchParams {
        query: Query::parse(query),
        ..SearchParams::default()
    };
    Ok(store
        .search_summaries(&params)?
        .into_iter()
        .filter(|n| n.book_id != TRASH_BOOK_ID)
        .map(|n| (n.updated_at, n.id))
        .collect())
}

fn newest(mut found: Vec<(i64, String)>) -> Vec<String> {
    found.sort_by(|a, b| b.cmp(a));
    found.truncate(MAX_SOURCES);
    found.into_iter().map(|(_, id)| id).collect()
}

/// Which of `ids` a draft may read, in the order given and without
/// repeats, with the counts of what was left out. A note the check cannot
/// make a decision about counts as locked.
pub fn resolve(store: &Store, ids: &[String]) -> SourceList {
    let mut list = SourceList::default();
    let mut seen = HashSet::new();
    let mut chars = 0usize;
    for id in ids {
        let id = full_id(id);
        if !seen.insert(id.clone()) {
            continue;
        }
        match ask::eligible_note(store, &id) {
            Ok(Some(c)) => {
                chars += c.body.chars().count();
                list.notes.push(SourceNote {
                    id,
                    title: c.title,
                    words: c.body.split_whitespace().count(),
                });
            }
            Ok(None) => {
                if is_excluded(store, &id) {
                    list.left_out_excluded += 1;
                } else if store.note(&id).ok().flatten().is_some() {
                    list.left_out_locked += 1;
                }
            }
            Err(_) => list.left_out_locked += 1,
        }
    }
    if list.notes.len() > MAX_SOURCES || chars > MAX_SOURCE_CHARS {
        list.too_many = true;
    }
    list
}

/// Whether a note that is out of reach is so because of `#ai-exclude`
/// alone (not locked, not trashed).
fn is_excluded(store: &Store, id: &str) -> bool {
    if store.note_is_locked(id).unwrap_or(true) {
        return false;
    }
    let Ok(Some(note)) = store.note(id) else {
        return false;
    };
    if note.is_trashed() {
        return false;
    }
    let names: std::collections::HashMap<String, String> = store
        .tags()
        .map(|t| t.into_iter().map(|t| (t.id, t.name)).collect())
        .unwrap_or_default();
    note.tags
        .iter()
        .any(|t| ask::is_exclude_tag(names.get(t).map_or(t.as_str(), String::as_str)))
}

/// The sources for a draft, numbered from 1, each read through the one
/// eligibility rule again (so a note locked since the sheet was opened is
/// not sent). When there are more than the draft can read one by one, the
/// ones that match the request's words best come first.
pub fn sources(store: &Store, ids: &[String], request: &str) -> Vec<Source> {
    let mut picked: Vec<(usize, Excerpt)> = Vec::new();
    let mut seen = HashSet::new();
    for id in ids {
        let id = full_id(id);
        if !seen.insert(id.clone()) {
            continue;
        }
        if let Ok(Some(c)) = ask::eligible_note(store, &id) {
            let score = relevance(request, &c.title, &c.body);
            picked.push((
                score,
                Excerpt {
                    id: c.id,
                    title: c.title,
                    text: c.body,
                },
            ));
        }
        if picked.len() >= MAX_SOURCES {
            break;
        }
    }
    let total: usize = picked.iter().map(|(_, e)| e.text.chars().count()).sum();
    if total > kanso_ai::draft::DRAFT_BUDGET {
        // Stable: equal scores keep the order the person gave.
        picked.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    }
    picked
        .into_iter()
        .enumerate()
        .map(|(i, (_, excerpt))| Source {
            number: i + 1,
            excerpt,
        })
        .collect()
}

/// How many of the request's words a note's title and text hold (title
/// matches count double).
fn relevance(request: &str, title: &str, body: &str) -> usize {
    let terms = ask::question_terms(request);
    let (title, body) = (title.to_lowercase(), body.to_lowercase());
    terms
        .iter()
        .map(|t| {
            let t = t.to_lowercase();
            2 * usize::from(title.contains(&t)) + usize::from(body.contains(&t))
        })
        .sum()
}

/// The sources as the saved note names them.
pub fn refs(sources: &[Source]) -> Vec<SourceRef> {
    sources
        .iter()
        .map(|s| SourceRef {
            number: s.number,
            id: s.excerpt.id.clone(),
            title: s.excerpt.title.clone(),
        })
        .collect()
}

/// The prompt-free label of a draft's first version in the note's history.
pub fn authored_by(sources: usize) -> String {
    format!(
        "Drafted by AI from {sources} {}",
        if sources == 1 { "note" } else { "notes" }
    )
}

/// Saves a reviewed draft as a new note in `book_id`: one write, with the
/// history's first version named for the draft. The Markdown has been made
/// safe (`ask::sanitize_answer`) by the caller. Undoing it deletes the note.
pub fn save(
    store: &Store,
    title: &str,
    markdown: &str,
    book_id: &str,
    sources: usize,
) -> kanso_store::Result<String> {
    let title = if title.trim().is_empty() {
        "Draft"
    } else {
        title.trim()
    };
    let id = store
        .put(
            DocType::Note,
            json!({"title": title, "body": markdown, "bookId": book_id}),
            Put::Upsert,
            Origin::Ui,
        )?
        .id;
    store.label_first_version(&id, &authored_by(sources))?;
    Ok(id)
}

/// The title of a draft: its first Markdown heading, else its first line.
pub fn title_of(draft: &str) -> String {
    let first = draft
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with("```"))
        .unwrap_or("");
    let title = first.trim_start_matches('#').trim();
    let title: String = title.chars().take(80).collect();
    if title.is_empty() {
        "Draft".into()
    } else {
        title
    }
}

/// A model's draft made safe to show and checked against its sources:
/// remote images dropped, raw HTML neutralised, then citations checked.
pub fn prepare(raw: &str, sources: &[Source]) -> kanso_ai::draft_model::CheckedDraft {
    kanso_ai::draft::check_citations(&ask::sanitize_answer(raw, &[]), sources)
}

/// Whether a draft is the model saying the notes don't hold what was asked.
pub fn not_covered(text: &str) -> bool {
    text.trim_start()
        .to_lowercase()
        .starts_with("your notes don't cover")
}

/// The Markdown that is saved from a checked draft's text: citations as
/// links to their notes, renumbered, and a `## Sources` section.
pub fn final_markdown(checked_text: &str, refs: &[SourceRef]) -> String {
    let checked = kanso_ai::draft_model::CheckedDraft {
        text: checked_text.to_owned(),
        ..Default::default()
    };
    kanso_ai::draft::saved_markdown(&checked, refs)
}

/// The notebook a draft is offered to be saved in: the first source's.
pub fn default_book(store: &Store, refs: &[SourceRef]) -> String {
    refs.iter()
        .find_map(|r| store.note(&r.id).ok().flatten())
        .map(|n| n.book_id)
        .filter(|b| b != TRASH_BOOK_ID)
        .unwrap_or_else(|| "book:inbox".to_owned())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn lib() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        (dir, store)
    }

    fn put(store: &Store, t: DocType, body: serde_json::Value) -> String {
        store.put(t, body, Put::Upsert, Origin::Ui).unwrap().id
    }

    fn lock(store: &Store, id: &str) {
        let mut sealed = store.get_api(id, None, false).unwrap();
        sealed["body"] = "".into();
        sealed["kansoLock"] =
            json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
        store.lock_note(sealed).unwrap();
    }

    #[test]
    fn sources_skip_what_ai_may_not_read_and_count_it() {
        let (_d, store) = lib();
        let book = put(&store, DocType::Book, json!({"name": "B"}));
        let child = put(
            &store,
            DocType::Book,
            json!({"name": "C", "parentBookId": book}),
        );
        let tag = put(&store, DocType::Tag, json!({"name": "ai-exclude"}));
        let plain = put(
            &store,
            DocType::Note,
            json!({"title": "Plain", "body": "alpha budget text", "bookId": book}),
        );
        let deep = put(
            &store,
            DocType::Note,
            json!({"title": "Deep", "body": "beta", "bookId": child}),
        );
        let hidden = put(
            &store,
            DocType::Note,
            json!({"title": "Hid", "body": "CANARY", "bookId": book, "tags": [tag]}),
        );
        let secret = put(
            &store,
            DocType::Note,
            json!({"title": "Sec", "body": "CANARY", "bookId": book}),
        );
        lock(&store, &secret);

        let ids = notes_of_book(&store, &book).unwrap();
        assert_eq!(ids.len(), 4, "the notebook and the one inside it");
        let list = resolve(&store, &ids);
        assert_eq!(list.notes.len(), 2);
        assert_eq!((list.left_out_locked, list.left_out_excluded), (1, 1));
        assert_eq!(
            list.summary(),
            "2 notes, about 4 words. 1 locked note left out. 1 note tagged #ai-exclude left out."
        );
        let numbered = sources(&store, &ids, "budget");
        assert_eq!(
            numbered.iter().map(|s| s.number).collect::<Vec<_>>(),
            vec![1, 2]
        );
        let text: String = numbered.iter().map(|s| s.excerpt.text.clone()).collect();
        assert!(!text.contains("CANARY"));
        assert!(numbered.iter().any(|s| s.excerpt.id == plain));
        assert!(numbered.iter().any(|s| s.excerpt.id == deep));
        assert_eq!(
            resolve(&store, &[hidden, secret, plain.clone(), plain])
                .notes
                .len(),
            1
        );
    }

    #[test]
    fn a_saved_draft_names_its_first_version_and_links_its_sources() {
        let (_d, store) = lib();
        let book = put(&store, DocType::Book, json!({"name": "B"}));
        let a = put(
            &store,
            DocType::Note,
            json!({"title": "Pricing", "body": "a", "bookId": book}),
        );
        let id = save(
            &store,
            "Update",
            "# Update\n\nBody [[Pricing]]\n",
            &book,
            12,
        )
        .unwrap();
        let revs = store.revisions(&id).unwrap();
        assert_eq!(revs.len(), 1);
        assert_eq!(
            revs[0].author.as_deref(),
            Some("Drafted by AI from 12 notes")
        );
        assert_eq!(
            store.backlinks(&a).unwrap().len(),
            1,
            "the draft shows on its source"
        );
        assert_eq!(title_of("\n# The Plan\n\ntext"), "The Plan");
        assert_eq!(title_of(""), "Draft");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(16))]

        /// Source numbers are 1..=n whatever order the request ranks them in.
        #[test]
        fn prop_sources_are_numbered_in_order(
            words in prop::collection::vec("[a-z]{3,8}", 0..6),
            request in "[a-z ]{0,20}",
        ) {
            let (_d, store) = lib();
            let book = put(&store, DocType::Book, json!({"name": "B"}));
            let ids: Vec<String> = words
                .iter()
                .map(|w| put(&store, DocType::Note, json!({"title": w, "body": w, "bookId": book})))
                .collect();
            let got = sources(&store, &ids, &request);
            prop_assert_eq!(got.len(), ids.len());
            for (i, s) in got.iter().enumerate() {
                prop_assert_eq!(s.number, i + 1);
            }
        }
    }
}
