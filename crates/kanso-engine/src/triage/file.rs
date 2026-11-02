//! Filing: what a person accepted becomes one write per note, in one
//! batch, with the write that takes each back. Nothing here runs without a
//! tap: [`plan`] only says what would change (for "File 5 notes? ..."),
//! [`file`] writes it.
//!
//! A note changed since its card was read (its revision moved), locked,
//! trashed or gone is dropped from the batch and reported as stale, so
//! filing never overwrites a newer edit. A locked notebook is never a
//! target: a note moved into one would sit unsealed until it is sealed.

use kanso_core::Note;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_store::{INBOX_TRIAGED_FIELD, Origin, Result, Store};
use serde_json::{Map, Value, json};

use super::links::with_related_line;
use super::model::{Field, Filed, Filing, Suggestion};
use crate::batch::{self, Edit};

/// The version of the `kansoTriaged` marker.
const MARKER_VERSION: i64 = 1;

/// What one note's accepted suggestions do.
#[derive(Clone, Debug, PartialEq)]
pub struct Planned {
    pub edit: Edit,
    pub moved: bool,
    pub tags: usize,
    pub titled: bool,
    pub links: usize,
}

/// The write for `note` given what was `accepted` for it. A notebook is
/// applied only when `book_ok` says it can be moved to; tag suggestions
/// must carry a tag ID. The note is always marked triaged, so it leaves
/// Triage even when nothing was accepted. Pure.
pub fn plan_file(
    note: &Note,
    accepted: &[Suggestion],
    book_ok: impl Fn(&str) -> bool,
    now_ms: i64,
) -> Planned {
    let mut change = Map::new();
    let mut undo = Map::new();
    let mut set = |key: &str, after: Value, before: Value| {
        change.insert(key.to_owned(), after);
        undo.insert(key.to_owned(), before);
    };
    let book = accepted
        .iter()
        .find(|s| s.field == Field::Notebook)
        .map(|s| s.value.as_str())
        .filter(|book| *book != note.book_id && book_ok(book));
    if let Some(book) = book {
        set("bookId", json!(book), json!(note.book_id));
    }
    let mut tags = note.tags.clone();
    let mut added = 0;
    for s in accepted.iter().filter(|s| s.field == Field::Tag) {
        if !tags.contains(&s.value) {
            tags.push(s.value.clone());
            added += 1;
        }
    }
    if added > 0 {
        set("tags", json!(tags), json!(note.tags));
    }
    let title = accepted
        .iter()
        .find(|s| s.field == Field::Title)
        .map(|s| s.value.trim())
        .filter(|t| !t.is_empty() && *t != note.title);
    if let Some(title) = title {
        set("title", json!(title), json!(note.title));
    }
    let mut labels: Vec<&str> = Vec::new();
    for s in accepted.iter().filter(|s| s.field == Field::Link) {
        if !labels.contains(&s.label.as_str()) {
            labels.push(&s.label);
        }
    }
    let body = with_related_line(&note.body, &labels);
    let links = if body == note.body { 0 } else { labels.len() };
    if links > 0 {
        set("body", json!(body), json!(note.body));
    }
    set(
        INBOX_TRIAGED_FIELD,
        json!({"v": MARKER_VERSION, "at": now_ms}),
        note.extra
            .get(INBOX_TRIAGED_FIELD)
            .cloned()
            .unwrap_or(Value::Null),
    );
    set("updatedAt", json!(now_ms), json!(note.updated_at));
    change.insert("_id".into(), json!(note.id));
    undo.insert("_id".into(), json!(note.id));
    Planned {
        edit: Edit {
            id: note.id.clone(),
            change: Value::Object(change),
            undo: Value::Object(undo),
        },
        moved: book.is_some(),
        tags: added,
        titled: title.is_some(),
        links,
    }
}

/// Whether notebook `book` can be filed into: it exists, isn't the trash
/// or locked. Reads the store.
fn book_ok(store: &Store, book: &str) -> bool {
    book != TRASH_BOOK_ID
        && store
            .get(book)
            .ok()
            .flatten()
            .is_some_and(|doc| doc.doc_type == kanso_core::DocType::Book)
        && !store
            .locked_books()
            .is_ok_and(|locked| locked.contains(book))
}

/// What filing `filings` would do (or, with `write`, does the planning
/// for it: new tags are made). Notes that changed since their card are
/// reported in `stale`.
fn build(store: &Store, filings: &[Filing], now_ms: i64, write: bool) -> Result<Filed> {
    let locked = store.locked_books()?;
    let mut out = Filed::default();
    for filing in filings {
        let note = store
            .note(&filing.note_id)?
            .filter(|n| !n.is_trashed() && !locked.contains(&n.book_id));
        let Some(note) =
            note.filter(|n| n.rev.as_deref() == Some(filing.rev.as_str()) && !n.is_locked())
        else {
            out.stale.push(filing.note_id.clone());
            continue;
        };
        let mut accepted = Vec::with_capacity(filing.accepted.len());
        for s in &filing.accepted {
            if s.field == Field::Tag && s.is_new {
                let name = s.value.trim();
                if name.is_empty() {
                    continue;
                }
                let id = if write {
                    store.tag_named(name, Origin::Ui)?
                } else {
                    format!("new:{}", name.to_lowercase())
                };
                accepted.push(Suggestion {
                    value: id,
                    is_new: false,
                    ..s.clone()
                });
            } else {
                accepted.push(s.clone());
            }
        }
        let planned = plan_file(
            &note,
            &accepted,
            |book| book_ok(store, book) && !locked.contains(book),
            now_ms,
        );
        out.moved += usize::from(planned.moved);
        out.tags += planned.tags;
        out.titles += usize::from(planned.titled);
        out.links += planned.links;
        out.edits.push(planned.edit);
    }
    Ok(out)
}

/// What filing would do, without writing anything (for the question "File
/// 5 notes?").
pub fn plan(store: &Store, filings: &[Filing], now_ms: i64) -> Result<Filed> {
    build(store, filings, now_ms, false)
}

/// Files `filings` in one write: every note changes, or none does. Notes
/// changed since their card was read are skipped (`stale`). The returned
/// edits are what [`undo`] takes back.
pub fn file(store: &Store, filings: &[Filing], now_ms: i64) -> Result<Filed> {
    let filed = build(store, filings, now_ms, true)?;
    let changes: Vec<Value> = filed.edits.iter().map(|e| e.change.clone()).collect();
    if !changes.is_empty() {
        store.merge_many(kanso_core::DocType::Note, changes, Origin::Ui)?;
    }
    Ok(filed)
}

/// Takes back a batch from [`file`] in one write: each note gets back what
/// it had, except a field changed again since. Returns how many notes
/// changed.
pub fn undo(store: &Store, edits: &[Edit]) -> Result<usize> {
    let back: Vec<Value> = edits
        .iter()
        .filter_map(|edit| {
            let note = store.note(&edit.id).ok().flatten()?;
            let current = serde_json::to_value(note).ok()?;
            batch::undo_now(&edit.change, &edit.undo, &current)
        })
        .collect();
    let count = back.len();
    if count > 0 {
        store.merge_many(kanso_core::DocType::Note, back, Origin::Ui)?;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::super::model::Source;
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn note() -> Note {
        serde_json::from_value(json!({
            "_id": "note:1", "_rev": "1-a", "title": "Untitled", "body": "Call Sam.",
            "bookId": "book:inbox", "tags": ["tag:old"], "updatedAt": 50
        }))
        .unwrap()
    }

    fn s(field: Field, value: &str, label: &str) -> Suggestion {
        Suggestion {
            field,
            value: value.into(),
            label: label.into(),
            score: 0.5,
            because: vec![],
            source: Source::Library,
            is_new: false,
        }
    }

    /// A merge as the store does it: the change's fields over the note's.
    fn merged(note: &Note, change: &Value) -> Value {
        let mut base = serde_json::to_value(note).unwrap();
        base.as_object_mut()
            .unwrap()
            .extend(change.as_object().unwrap().clone());
        base
    }

    #[test]
    fn accepted_chips_become_one_write() {
        let accepted = [
            s(Field::Notebook, "book:home", "Home"),
            s(Field::Tag, "tag:house", "house"),
            s(Field::Tag, "tag:old", "old"),
            s(Field::Title, "Call Sam", "Call Sam"),
            s(Field::Link, "note:2", "Flat repairs"),
        ];
        let p = plan_file(&note(), &accepted, |_| true, 99);
        assert!(p.moved && p.titled);
        assert_eq!((p.tags, p.links), (1, 1));
        let c = &p.edit.change;
        assert_eq!(c["bookId"], "book:home");
        assert_eq!(c["tags"], json!(["tag:old", "tag:house"]));
        assert_eq!(c["title"], "Call Sam");
        assert_eq!(c["body"], "Call Sam.\n\nRelated: [[Flat repairs]]");
        assert_eq!(c["kansoTriaged"], json!({"v": 1, "at": 99}));
        assert_eq!(p.edit.undo["bookId"], "book:inbox");
        assert_eq!(p.edit.undo["kansoTriaged"], Value::Null);
    }

    #[test]
    fn nothing_accepted_still_marks_the_note() {
        let p = plan_file(&note(), &[], |_| true, 99);
        assert!(!p.moved && !p.titled && p.tags == 0 && p.links == 0);
        assert!(p.edit.change.get("bookId").is_none());
        assert_eq!(p.edit.change["kansoTriaged"]["at"], 99);
    }

    #[test]
    fn a_refused_notebook_is_not_applied() {
        let accepted = [s(Field::Notebook, "book:locked", "Locked")];
        let p = plan_file(&note(), &accepted, |b| b != "book:locked", 99);
        assert!(!p.moved);
        assert!(p.edit.change.get("bookId").is_none());
    }

    fn suggestions() -> impl Strategy<Value = Vec<Suggestion>> {
        prop::collection::vec(
            (0usize..4, "[a-z]{1,6}").prop_map(|(f, v)| match f {
                0 => s(Field::Notebook, &format!("book:{v}"), &v),
                1 => s(Field::Tag, &format!("tag:{v}"), &v),
                2 => s(Field::Title, &v, &v),
                _ => s(Field::Link, &format!("note:{v}"), &v),
            }),
            0..8,
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Applying a filing and then its undo gives back every field the
        /// note had, exactly.
        #[test]
        fn undo_restores_every_field(accepted in suggestions(), body in "[a-zA-Z .\n]{0,40}") {
            let mut original = note();
            original.body = body;
            let p = plan_file(&original, &accepted, |_| true, 99);
            let after = merged(&original, &p.edit.change);
            let current = after.clone();
            let back = batch::undo_now(&p.edit.change, &p.edit.undo, &current).unwrap();
            let mut restored = after;
            restored.as_object_mut().unwrap().extend(back.as_object().unwrap().clone());
            let before = serde_json::to_value(&original).unwrap();
            for key in ["bookId", "tags", "title", "body", "updatedAt"] {
                prop_assert_eq!(&restored[key], &before[key], "{}", key);
            }
            prop_assert!(
                restored.get(INBOX_TRIAGED_FIELD).is_none_or(Value::is_null)
            );
        }

        /// A field edited again after filing keeps its newer value.
        #[test]
        fn undo_keeps_a_newer_edit(accepted in suggestions()) {
            let original = note();
            let p = plan_file(&original, &accepted, |_| true, 99);
            let mut current = merged(&original, &p.edit.change);
            current["title"] = json!("Typed since");
            if let Some(back) = batch::undo_now(&p.edit.change, &p.edit.undo, &current) {
                prop_assert!(back.get("title").is_none());
            }
        }
    }
}
