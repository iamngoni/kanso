//! Save as note: the one write Ask makes, and only on the person's press.
//!
//! The note is titled "Q: <question>", holds the answer, and ends with the
//! Sources as ordinary note links with line anchors
//! (`[Pricing](inkdrop://note/<id>#L12-14)`), so each opens the cited lines.
//! It goes in the notebook the question was scoped to, else the Inbox.

use kanso_core::DocType;
use kanso_core::links::{NOTE_LINK_PREFIX, line_anchor_text};
use kanso_core::roles::Role;
use kanso_store::{Origin, Put, Store};

use super::model::{AskScope, NO_SOURCE_PREFIX, NOT_FOUND_LABEL, UNVERIFIED_PREFIX};
use super::turn::Finished;

/// The most characters of the question in the title.
const TITLE_MAX: usize = 120;

/// A note made from an answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedAnswer {
    pub title: String,
    pub body: String,
}

/// The note an answer is saved as: the title and the Markdown body.
pub fn saved_note(question: &str, finished: &Finished) -> SavedAnswer {
    let question = question.split_whitespace().collect::<Vec<_>>().join(" ");
    let title = format!("Q: {}", kanso_ai::prompt::truncate(&question, TITLE_MAX));
    let mut text = finished.markdown.clone();
    // Each receipt's number links the lines it points at.
    for receipt in &finished.receipts {
        let from = format!("](kanso-cite:{})", receipt.n);
        let to = format!(
            "]({NOTE_LINK_PREFIX}{}#{})",
            receipt.note_id.trim_start_matches("note:"),
            line_anchor_text(receipt.start, receipt.end)
        );
        text = text.replace(&from, &to);
    }
    // A sentence that was not found keeps its words and its label, without
    // the marker that drew it.
    for prefix in [UNVERIFIED_PREFIX, NO_SOURCE_PREFIX] {
        while let Some(at) = text.find(&format!("]({prefix}")) {
            let Some(open) = text[..at].rfind('[') else {
                break;
            };
            let Some(close) = text[at..].find(')') else {
                break;
            };
            let sentence = text[open + 1..at].to_owned();
            text.replace_range(open..at + close + 1, &sentence);
        }
    }
    let mut body = text.trim().to_owned();
    if !finished.receipts.is_empty() {
        body.push_str("\n\n## Sources\n\n");
        for receipt in &finished.receipts {
            let title = if receipt.title.trim().is_empty() {
                "Untitled"
            } else {
                receipt.title.trim()
            };
            let rest = receipt.source_line();
            let rest = rest
                .strip_prefix(title)
                .map(str::to_owned)
                .unwrap_or_default();
            body.push_str(&format!(
                "{}. [{title}]({NOTE_LINK_PREFIX}{}#{}){rest}\n",
                receipt.n,
                receipt.note_id.trim_start_matches("note:"),
                line_anchor_text(receipt.start, receipt.end),
            ));
        }
    }
    if finished.unverified > 0 && !body.contains(NOT_FOUND_LABEL) {
        body.push_str(&format!("\n\n*{NOT_FOUND_LABEL}*\n"));
    }
    body.push('\n');
    SavedAnswer { title, body }
}

/// Writes the answer as a new note and returns its ID. This is the only
/// write Ask makes, and the panel calls it only when the person presses
/// Save as note.
pub fn save_answer(
    store: &Store,
    scope: &AskScope,
    question: &str,
    finished: &Finished,
) -> kanso_store::Result<String> {
    let saved = saved_note(question, finished);
    let locked = store.locked_books().unwrap_or_default();
    // The scoped notebook, unless it is locked (a scope inside a locked
    // notebook can't be active, so this never writes there) or gone.
    let scoped = match scope {
        AskScope::Book(id) if !locked.contains(id) => store
            .books()?
            .into_iter()
            .find(|book| book.id == *id)
            .map(|book| book.id),
        _ => None,
    };
    let book = match scoped {
        Some(book) => book,
        None => match store.find_role_book(Role::Inbox)? {
            Some(inbox) if !locked.contains(&inbox) => inbox,
            _ => store.role_book(Role::Inbox, Origin::Ui)?,
        },
    };
    store
        .put(
            DocType::Note,
            serde_json::json!({"title": saved.title, "body": saved.body, "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .map(|made| made.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ask::{Shown, ShownExcerpt, finalize, receipts::check};
    use kanso_core::links::note_links;
    use serde_json::json;

    fn finished(raw: &str) -> Finished {
        let shown = Shown::new();
        shown.register(ShownExcerpt {
            number: 1,
            id: "note:pricing".into(),
            title: "Pricing".into(),
            updated: Some("2026-10-14".into()),
            lines: vec![
                (11, "We chose the yearly plan.".into()),
                (12, "40 dollars a year.".into()),
            ],
        });
        let _ = check;
        finalize(raw, &shown, true, &Default::default())
    }

    #[test]
    fn a_saved_answer_links_the_lines_it_cites() {
        let done = finished(
            "We chose the yearly plan at 40 dollars [1:L12-13]. Customers love it [1:L99].",
        );
        let saved = saved_note("What  did we choose?", &done);
        assert_eq!(saved.title, "Q: What did we choose?");
        assert!(
            saved.body.contains("[1](inkdrop://note/pricing#L12-13)"),
            "{}",
            saved.body
        );
        assert!(saved.body.contains("## Sources"));
        assert!(saved.body.contains(
            "1. [Pricing](inkdrop://note/pricing#L12-13) \u{b7} lines 12\u{2013}13 \u{b7} 14 Oct"
        ));
        assert!(saved.body.contains("*Not found in your notes*"));
        assert!(!saved.body.contains("kanso-"), "{}", saved.body);
        // The links are ordinary note links with the line form.
        let links = note_links(&saved.body);
        assert!(links.iter().any(|l| l.lines == Some((11, 12))), "{links:?}");
    }

    #[test]
    fn saving_writes_one_note_in_the_scoped_notebook_or_the_inbox() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let work = store
            .put(
                DocType::Book,
                json!({"name": "Work"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let done = finished("We chose the yearly plan [1:L12].");
        let before = store.search_summaries(&Default::default()).unwrap().len();
        let id = save_answer(&store, &AskScope::Book(work.clone()), "Plan?", &done).unwrap();
        let note = store.note(&id).unwrap().unwrap();
        assert_eq!(note.book_id, work);
        let other = save_answer(&store, &AskScope::All, "Plan?", &done).unwrap();
        let inbox = store.note(&other).unwrap().unwrap().book_id;
        assert_eq!(store.find_role_book(Role::Inbox).unwrap(), Some(inbox));
        assert_eq!(
            store.search_summaries(&Default::default()).unwrap().len(),
            before + 2
        );
    }

    #[test]
    fn a_locked_scope_is_never_written_to() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let vault = store
            .put(
                DocType::Book,
                json!({"name": "Vault"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        store
            .put(
                DocType::Book,
                json!({"_id": vault, "kansoLock": {"salt": "abc"}}),
                Put::Merge,
                Origin::Sync,
            )
            .unwrap();
        let done = finished("We chose the yearly plan [1:L12].");
        let id = save_answer(&store, &AskScope::Book(vault.clone()), "Plan?", &done).unwrap();
        assert_ne!(store.note(&id).unwrap().unwrap().book_id, vault);
    }
}
