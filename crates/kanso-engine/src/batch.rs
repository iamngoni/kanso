//! One change to several notes at once, planned: the selected notes from the
//! list's menu or the palette, the notes dropped on a notebook or tag in the
//! sidebar, or a swipe on the phone (issue #48). [`plan`] says which notes
//! are written and how, and the write that undoes each. Every note changes in
//! one write, and Undo puts back what each note had, in one write too.

use std::collections::HashSet;

use kanso_core::NoteStatus;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_store::NoteSummary;
use serde_json::{Value, json};

/// What happens to every note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    /// Into this notebook (the trash's ID moves them to the trash).
    Move(String),
    /// Adds this tag to each note that hasn't it.
    AddTag(String),
    /// Takes this tag off each note that has it.
    RemoveTag(String),
    /// Adds this tag to every note, or takes it off every note when they
    /// all have it, as choosing a checked tag does for one note.
    ToggleTag(String),
    /// Pins every note, or unpins them when they are all pinned.
    TogglePin,
    Status(NoteStatus),
}

/// One note's write, and the write that undoes it.
#[derive(Clone, Debug, PartialEq)]
pub struct Edit {
    pub id: String,
    pub change: Value,
    pub undo: Value,
}

/// The writes `change` takes for `notes`: one for each note it changes,
/// none for a note that is already as asked.
pub fn plan(notes: &[NoteSummary], change: &Change) -> Vec<Edit> {
    let all_have = |tag: &str| notes.iter().all(|n| n.tags.iter().any(|t| t == tag));
    let all_pinned = notes.iter().all(|n| n.pinned);
    notes
        .iter()
        .filter_map(|note| {
            let (field, before, after) = match change {
                Change::Move(book) => ("bookId", json!(note.book_id), json!(book)),
                Change::AddTag(tag) | Change::ToggleTag(tag) | Change::RemoveTag(tag) => {
                    let has = note.tags.iter().any(|t| t == tag);
                    let off = matches!(change, Change::RemoveTag(_))
                        || matches!(change, Change::ToggleTag(_)) && all_have(tag);
                    let tags: Vec<&String> = if off {
                        note.tags.iter().filter(|t| *t != tag).collect()
                    } else if has {
                        note.tags.iter().collect()
                    } else {
                        note.tags.iter().chain([tag]).collect()
                    };
                    ("tags", json!(note.tags), json!(tags))
                }
                Change::TogglePin => ("pinned", json!(note.pinned), json!(!all_pinned)),
                Change::Status(status) => (
                    "status",
                    json!(note.status.as_str()),
                    json!(status.as_str()),
                ),
            };
            (before != after).then(|| Edit {
                id: note.id.clone(),
                change: json!({ "_id": note.id, field: after }),
                undo: json!({ "_id": note.id, field: before }),
            })
        })
        .collect()
}

/// What of an undo still applies to a note that is `current` now (as
/// JSON): the fields still holding what the batch wrote (`change`) go
/// back to `undo`'s; a field changed again since keeps its newer value.
/// `None` when nothing is left to put back. Pure.
pub fn undo_now(change: &Value, undo: &Value, current: &Value) -> Option<Value> {
    let mut back = serde_json::Map::new();
    for (key, value) in undo.as_object()? {
        if key != "_id" && current.get(key) == change.get(key) {
            back.insert(key.clone(), value.clone());
        }
    }
    if back.is_empty() {
        return None;
    }
    back.insert("_id".into(), undo.get("_id")?.clone());
    Some(Value::Object(back))
}

/// What the notice says once `count` notes changed. `place` names the
/// notebook, tag or status the change is about; `on` is whether a toggle
/// turned the tag or pin on. Pure.
pub fn summary(change: &Change, count: usize, place: &str, on: bool) -> String {
    let notes = if count == 1 {
        "1 note".to_owned()
    } else {
        format!("{count} notes")
    };
    match change {
        Change::Move(book) if book == TRASH_BOOK_ID => format!("Moved {notes} to the trash."),
        Change::Move(_) => format!("Moved {notes} to {place}."),
        Change::AddTag(_) => format!("Added #{place} to {notes}."),
        Change::RemoveTag(_) => format!("Took #{place} off {notes}."),
        Change::ToggleTag(_) if on => format!("Added #{place} to {notes}."),
        Change::ToggleTag(_) => format!("Took #{place} off {notes}."),
        Change::TogglePin if on => format!("Pinned {notes}."),
        Change::TogglePin => format!("Unpinned {notes}."),
        Change::Status(_) => format!("Set {notes} to {place}."),
    }
}

/// Whether the `edits` of a batch turned the pin or the tag of `change`
/// on (the first edit says: a batch is alike for every note). Pure.
pub fn turned_on(change: &Change, edits: &[Edit]) -> bool {
    let Some(first) = edits.first() else {
        return true;
    };
    match change {
        Change::TogglePin => first.change["pinned"] == json!(true),
        Change::ToggleTag(tag) => first.change["tags"]
            .as_array()
            .is_some_and(|tags| tags.iter().any(|t| t == tag)),
        _ => true,
    }
}

/// Whether the notes must go one at a time: unlocked notes into a locked
/// notebook are sealed as they go, which needs the lock's key and so one
/// note at a time with the editor. `locked` is the locked notebooks, each
/// with the ones inside it (`Store::locked_books`).
pub fn needs_one_by_one(notes: &[NoteSummary], change: &Change, locked: &HashSet<String>) -> bool {
    matches!(change, Change::Move(book) if locked.contains(book)) && notes.iter().any(|n| !n.locked)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use kanso_core::NoteStatus;
    use kanso_core::model::TRASH_BOOK_ID;
    use kanso_store::NoteSummary;
    use serde_json::{Value, json};
    use std::collections::HashSet;

    use super::{Change, needs_one_by_one, plan, summary, turned_on, undo_now};

    fn note(id: usize, book: &str, tags: &[&str], pinned: bool, status: NoteStatus) -> NoteSummary {
        NoteSummary {
            id: format!("note:{id}"),
            rev: "1-a".into(),
            title: format!("n{id}"),
            snippet: String::new(),
            book_id: book.into(),
            status,
            tags: tags.iter().map(|t| (*t).to_owned()).collect(),
            pinned,
            updated_at: 0,
            created_at: 0,
            tasks_done: 0,
            tasks_total: 0,
            locked: false,
        }
    }

    #[test]
    fn notices() {
        assert_eq!(
            summary(&Change::Move(TRASH_BOOK_ID.into()), 3, "", true),
            "Moved 3 notes to the trash."
        );
        assert_eq!(
            summary(&Change::Move("book:a".into()), 1, "Work", true),
            "Moved 1 note to Work."
        );
        assert_eq!(
            summary(&Change::AddTag("t".into()), 2, "ideas", true),
            "Added #ideas to 2 notes."
        );
        assert_eq!(
            summary(&Change::RemoveTag("t".into()), 1, "ideas", false),
            "Took #ideas off 1 note."
        );
        assert_eq!(
            summary(&Change::ToggleTag("t".into()), 2, "ideas", false),
            "Took #ideas off 2 notes."
        );
        assert_eq!(
            summary(&Change::ToggleTag("t".into()), 2, "ideas", true),
            "Added #ideas to 2 notes."
        );
        assert_eq!(summary(&Change::TogglePin, 1, "", true), "Pinned 1 note.");
        assert_eq!(
            summary(&Change::TogglePin, 2, "", false),
            "Unpinned 2 notes."
        );
        assert_eq!(
            summary(&Change::Status(NoteStatus::Completed), 2, "Completed", true),
            "Set 2 notes to Completed."
        );
    }

    /// Writes `edit`'s fields into the matching note, as a merge would.
    fn merge(notes: &mut [NoteSummary], fields: &Value) {
        let note = notes
            .iter_mut()
            .find(|n| fields["_id"] == json!(n.id))
            .expect("a write for a listed note");
        for (key, value) in fields.as_object().unwrap() {
            match key.as_str() {
                "_id" => {}
                "bookId" => note.book_id = value.as_str().unwrap().into(),
                "tags" => note.tags = serde_json::from_value(value.clone()).unwrap(),
                "pinned" => note.pinned = value.as_bool().unwrap(),
                "status" => note.status = NoteStatus::parse(value.as_str().unwrap()).unwrap(),
                other => panic!("unexpected field {other}"),
            }
        }
    }

    fn notes() -> impl Strategy<Value = Vec<NoteSummary>> {
        let one = (
            prop::sample::select(vec!["book:a", "book:b"]),
            prop::collection::btree_set(prop::sample::select(vec!["t1", "t2", "t3"]), 0..3),
            any::<bool>(),
            prop::sample::select(NoteStatus::ALL.to_vec()),
        );
        prop::collection::vec(one, 0..8).prop_map(|notes| {
            notes
                .into_iter()
                .enumerate()
                .map(|(ix, (book, tags, pinned, status))| {
                    let tags: Vec<&str> = tags.into_iter().collect();
                    note(ix, book, &tags, pinned, status)
                })
                .collect()
        })
    }

    fn change() -> impl Strategy<Value = Change> {
        prop_oneof![
            prop::sample::select(vec!["book:a", "book:b", TRASH_BOOK_ID])
                .prop_map(|b| Change::Move(b.into())),
            prop::sample::select(vec!["t1", "t2"]).prop_map(|t| Change::AddTag(t.into())),
            prop::sample::select(vec!["t1", "t2"]).prop_map(|t| Change::RemoveTag(t.into())),
            prop::sample::select(vec!["t1", "t2"]).prop_map(|t| Change::ToggleTag(t.into())),
            Just(Change::TogglePin),
            prop::sample::select(NoteStatus::ALL.to_vec()).prop_map(Change::Status),
        ]
    }

    /// Whether `note` is as `change` asks, given whether a toggle turned on.
    fn is_as_asked(note: &NoteSummary, change: &Change, on: bool) -> bool {
        match change {
            Change::Move(book) => &note.book_id == book,
            Change::AddTag(tag) => note.tags.contains(tag),
            Change::RemoveTag(tag) => !note.tags.contains(tag),
            Change::ToggleTag(tag) => note.tags.contains(tag) == on,
            Change::TogglePin => note.pinned == on,
            Change::Status(status) => note.status == *status,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Invariants: afterwards every note is as asked, a tag never shows
        /// twice on a note, and only notes that weren't already are
        /// written, once each. Round trip: Undo brings back every note
        /// exactly as it was. Idempotence: asking again for a move, tag or
        /// status writes nothing; a toggle asked again flips back.
        #[test]
        fn prop_a_batch_changes_all_and_undoes_exactly(before in notes(), change in change()) {
            check_batch(&before, &change)?;
        }

        /// Invariant: a late Undo puts back exactly the fields that still
        /// hold the batch's value, and never touches one changed since.
        /// With nothing changed since, it is the whole undo.
        #[test]
        fn prop_a_late_undo_keeps_later_changes(
            book_kept in any::<bool>(),
            pinned_kept in any::<bool>(),
        ) {
            let change = json!({"_id": "note:1", "bookId": "b2", "pinned": true});
            let undo = json!({"_id": "note:1", "bookId": "b1", "pinned": false});
            let current = json!({
                "_id": "note:1",
                "bookId": if book_kept { "b2" } else { "b3" },
                "pinned": pinned_kept,
            });
            let back = undo_now(&change, &undo, &current);
            match back {
                None => prop_assert!(!book_kept && !pinned_kept),
                Some(back) => {
                    prop_assert_eq!(&back["_id"], "note:1");
                    prop_assert_eq!(back.get("bookId").is_some(), book_kept);
                    prop_assert_eq!(back.get("pinned").is_some(), pinned_kept);
                    if book_kept && pinned_kept {
                        prop_assert_eq!(&back, &undo);
                    }
                }
            }
        }
    }

    /// The batch property's body, out of the macro, which can't expand
    /// this much.
    fn check_batch(before: &[NoteSummary], change: &Change) -> Result<(), TestCaseError> {
        let edits = plan(before, change);
        let mut after = before.to_vec();
        for edit in &edits {
            merge(&mut after, &edit.change);
        }
        let on = match change {
            Change::ToggleTag(tag) => !before.iter().all(|n| n.tags.contains(tag)),
            Change::TogglePin => !before.iter().all(|n| n.pinned),
            _ => true,
        };
        for (old, new) in before.iter().zip(&after) {
            prop_assert!(is_as_asked(new, change, on), "{new:?} after {change:?}");
            let unique: std::collections::BTreeSet<_> = new.tags.iter().collect();
            prop_assert_eq!(unique.len(), new.tags.len());
            let written = edits.iter().filter(|e| e.id == old.id).count();
            prop_assert_eq!(written, usize::from(old != new));
        }

        let mut undone = after.clone();
        for edit in &edits {
            merge(&mut undone, &edit.undo);
        }
        prop_assert_eq!(&undone[..], before);

        let again = plan(&after, change);
        match change {
            Change::ToggleTag(_) | Change::TogglePin if !after.is_empty() => {
                let mut back = after.clone();
                for edit in &again {
                    merge(&mut back, &edit.change);
                }
                // Back to all off when it turned on; when it turned off,
                // every note gets it.
                for note in &back {
                    prop_assert!(is_as_asked(note, change, !on));
                }
            }
            _ => prop_assert!(again.is_empty()),
        }
        Ok(())
    }

    #[test]
    fn examples() {
        let notes = vec![
            note(1, "book:a", &["t1"], true, NoteStatus::None),
            note(2, "book:b", &[], false, NoteStatus::Active),
        ];
        let moved = plan(&notes, &Change::Move("book:b".into()));
        assert_eq!(moved.len(), 1, "the note already there isn't written");
        assert_eq!(
            moved[0].change,
            json!({"_id": "note:1", "bookId": "book:b"})
        );
        assert_eq!(moved[0].undo, json!({"_id": "note:1", "bookId": "book:a"}));

        let pinned = plan(&notes, &Change::TogglePin);
        assert_eq!(pinned.len(), 1, "one unpinned: all pin");
        assert_eq!(pinned[0].change["pinned"], json!(true));

        let tagged = plan(&notes, &Change::ToggleTag("t1".into()));
        assert_eq!(tagged[0].change["tags"], json!(["t1"]));
    }

    #[test]
    fn undo_after_a_tag_was_typed_leaves_the_tags() {
        let change = json!({"_id": "note:1", "tags": ["a", "b"]});
        let undo = json!({"_id": "note:1", "tags": ["a"]});
        let current = json!({"_id": "note:1", "tags": ["a", "b", "c"]});
        assert_eq!(undo_now(&change, &undo, &current), None);
        let current = json!({"_id": "note:1", "tags": ["a", "b"]});
        assert_eq!(undo_now(&change, &undo, &current), Some(undo));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Invariant: a move writes the notebook and nothing else, and does
        /// not touch a note that is there already.
        #[test]
        fn prop_a_move_sets_the_notebook_only(before in notes(), book in prop::sample::select(vec!["book:a", "book:b", TRASH_BOOK_ID])) {
            let edits = plan(&before, &Change::Move(book.into()));
            for edit in &edits {
                let fields: Vec<&String> = edit.change.as_object().unwrap().keys().filter(|k| *k != "_id").collect();
                prop_assert_eq!(fields, vec!["bookId"]);
                prop_assert_eq!(&edit.change["bookId"], &json!(book));
                let old = before.iter().find(|n| n.id == edit.id).unwrap();
                prop_assert_ne!(old.book_id.as_str(), book);
            }
            let moving = before.iter().filter(|n| n.book_id != book).count();
            prop_assert_eq!(edits.len(), moving);
        }

        /// Round trip: toggling the pin twice leaves every note as it was
        /// when they were all alike, and all pinned or all unpinned
        /// otherwise; asking for nothing new writes nothing.
        #[test]
        fn prop_toggling_the_pin_twice_is_the_identity_for_alike_notes(pinned: bool, count in 1usize..6) {
            let before: Vec<NoteSummary> = (0..count).map(|i| note(i, "book:a", &[], pinned, NoteStatus::None)).collect();
            let mut once = before.clone();
            for edit in plan(&before, &Change::TogglePin) {
                merge(&mut once, &edit.change);
            }
            prop_assert!(once.iter().all(|n| n.pinned != pinned));
            let mut twice = once.clone();
            for edit in plan(&once, &Change::TogglePin) {
                merge(&mut twice, &edit.change);
            }
            prop_assert_eq!(twice, before);
        }

        /// Model: notes must go one at a time exactly when the move is into
        /// a locked notebook and some note is not sealed yet.
        #[test]
        fn prop_one_by_one_is_a_move_into_a_lock_with_unsealed_notes(
            before in notes(),
            locked_flags in prop::collection::vec(any::<bool>(), 8),
            book in prop::sample::select(vec!["book:a", "book:b", TRASH_BOOK_ID]),
            locked_books in prop::collection::btree_set(prop::sample::select(vec!["book:a", "book:b"]), 0..3),
        ) {
            let mut before = before;
            for (note, locked) in before.iter_mut().zip(locked_flags) {
                note.locked = locked;
            }
            let locked_set: HashSet<String> = locked_books.iter().map(|b| (*b).to_owned()).collect();
            let moved = Change::Move(book.into());
            let expected = locked_set.contains(book) && before.iter().any(|n| !n.locked);
            prop_assert_eq!(needs_one_by_one(&before, &moved, &locked_set), expected);
            // Nothing but a move is held up by a lock.
            prop_assert!(!needs_one_by_one(&before, &Change::TogglePin, &locked_set));
        }
    }

    #[test]
    fn a_pin_batch_says_which_way_it_went() {
        let pinned = [note(1, "book:a", &[], false, NoteStatus::Active)];
        let edits = plan(&pinned, &Change::TogglePin);
        assert!(turned_on(&Change::TogglePin, &edits));
        let unpinned = [note(1, "book:a", &[], true, NoteStatus::Active)];
        let edits = plan(&unpinned, &Change::TogglePin);
        assert!(!turned_on(&Change::TogglePin, &edits));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// The notice counts the notes it is about, and a pin or tag toggle
        /// says "Pinned"/"Added" exactly when the batch turned it on.
        #[test]
        fn prop_the_notice_matches_the_batch(before in notes(), change in change()) {
            let edits = plan(&before, &change);
            let on = turned_on(&change, &edits);
            let text = summary(&change, edits.len(), "Place", on);
            if edits.len() == 1 {
                prop_assert!(text.contains("1 note") && !text.contains("1 notes"));
            } else {
                let count = format!("{} notes", edits.len());
                prop_assert!(text.contains(&count));
            }
            if matches!(change, Change::TogglePin) && !edits.is_empty() {
                prop_assert_eq!(text.starts_with("Pinned"), on);
                prop_assert_eq!(text.starts_with("Unpinned"), !on);
            }
            prop_assert!(text.ends_with('.'));
        }
    }
}
