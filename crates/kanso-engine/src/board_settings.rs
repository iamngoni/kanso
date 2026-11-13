//! Board settings: renaming, reordering, adding and removing the lanes of a
//! board grouped by a property. A lane is a value of the property, so
//! renaming or removing one rewrites the property on every note in it, and
//! no note is ever left in a lane the board does not have: a removed lane's
//! notes move to another lane or to "no value".
//!
//! The lanes are kept in the notebook's board (`kansoBoard.lanes`) and, when
//! the property is a select, in its options too, so the two agree and no
//! note warns "Not one of this notebook's stages" after an edit.

use kanso_core::properties::{self, Value};
use kanso_core::schema::{BoardSpec, PropertyType};
use kanso_store::{Origin, PropertyEdit, Store, StoreError};

use crate::board::Lane;
use crate::board_model::PropertyBoard;
pub use crate::board_settings_model::{LaneEdit, LaneEditError, LaneEdited, SettingsUndo};

/// The lanes after `edit`, or why it is refused. Pure.
pub fn lanes_after(lanes: &[String], edit: &LaneEdit) -> Result<Vec<String>, LaneEditError> {
    let has = |name: &str| lanes.iter().any(|lane| lane == name);
    let mut next = lanes.to_vec();
    match edit {
        LaneEdit::Rename { from, to } => {
            let to = clean(to)?;
            let at = lanes
                .iter()
                .position(|lane| lane == from)
                .ok_or_else(|| LaneEditError::Missing(from.clone()))?;
            if to != *from && has(&to) {
                return Err(LaneEditError::Duplicate(to));
            }
            next[at] = to;
        }
        LaneEdit::Move { from, to } => {
            if *from >= lanes.len() || *to >= lanes.len() {
                return Err(LaneEditError::Position);
            }
            let lane = next.remove(*from);
            next.insert(*to, lane);
        }
        LaneEdit::Add(name) => {
            let name = clean(name)?;
            if has(&name) {
                return Err(LaneEditError::Duplicate(name));
            }
            next.push(name);
        }
        LaneEdit::Remove { lane, move_to } => {
            let at = lanes
                .iter()
                .position(|l| l == lane)
                .ok_or_else(|| LaneEditError::Missing(lane.clone()))?;
            if lanes.len() == 1 {
                return Err(LaneEditError::LastLane);
            }
            next.remove(at);
            if let Some(target) = move_to
                && !next.contains(target)
            {
                return Err(LaneEditError::Target(target.clone()));
            }
        }
    }
    Ok(next)
}

fn clean(name: &str) -> Result<String, LaneEditError> {
    let name = name.trim();
    if name.is_empty() {
        Err(LaneEditError::Empty)
    } else {
        Ok(name.to_owned())
    }
}

/// What a note whose property reads `value` is rewritten to by `edit`, or
/// `None` when the edit leaves it. A value that is no lane's is never
/// touched. Pure.
pub fn value_after(edit: &LaneEdit, value: &str) -> Option<String> {
    match edit {
        LaneEdit::Rename { from, to } if value == from => Some(to.trim().to_owned()),
        LaneEdit::Remove { lane, move_to } if value == lane => {
            Some(move_to.clone().unwrap_or_default())
        }
        _ => None,
    }
}

/// A select property's options after `edit`: the same change, so they keep
/// listing the board's lanes. A reorder leaves them; a name already there
/// is not listed twice. Pure.
pub fn options_after(options: &[String], edit: &LaneEdit) -> Vec<String> {
    let mut next: Vec<String> = match edit {
        LaneEdit::Rename { from, to } => options
            .iter()
            .map(|o| {
                if o == from {
                    to.trim().to_owned()
                } else {
                    o.clone()
                }
            })
            .collect(),
        LaneEdit::Add(name) => options
            .iter()
            .cloned()
            .chain(Some(name.trim().to_owned()))
            .collect(),
        LaneEdit::Remove { lane, .. } => options.iter().filter(|o| *o != lane).cloned().collect(),
        LaneEdit::Move { .. } => options.to_vec(),
    };
    let mut seen = std::collections::HashSet::new();
    next.retain(|option| seen.insert(option.clone()));
    next
}

/// The words that ask where a removed lane's notes go:
/// "3 notes are in Shooting. Move them to:". Pure.
pub fn removal_prompt(lane: &str, notes: usize) -> String {
    match notes {
        1 => format!("1 note is in {lane}. Move it to:"),
        n => format!("{n} notes are in {lane}. Move them to:"),
    }
}

/// The live notes of notebook `book_id` in lane `lane` of the board.
pub fn notes_in_lane(
    store: &Store,
    book_id: &str,
    group: &PropertyBoard,
    lane: &str,
) -> Result<Vec<String>, StoreError> {
    let mut ids = Vec::new();
    for note in store.notes_in_books(&[book_id.to_owned()])? {
        if note.is_locked() {
            continue;
        }
        let all = properties::parse(&note.body);
        if group.place(properties::get(&all, &group.key)) == Lane::Value(lane.to_owned()) {
            ids.push(note.id);
        }
    }
    Ok(ids)
}

/// Writes `edit` to the board of notebook `book_id`: the notes first (each
/// in one write), then the board and the property's options. When the
/// board can't be saved the notes are put back. The result has what undoes
/// all of it ([`undo_lane_edit`]).
pub fn apply_lane_edit(
    store: &Store,
    book_id: &str,
    edit: &LaneEdit,
    now_ms: i64,
) -> Result<LaneEdited, StoreError> {
    let spec = store
        .notebook_board(book_id)?
        .ok_or_else(|| StoreError::Invalid("This notebook has no board.".into()))?;
    let defs = store.notebook_properties(book_id)?;
    let group = PropertyBoard::from_spec(&spec, &defs);
    let lanes =
        lanes_after(&group.values, edit).map_err(|err| StoreError::Invalid(err.message()))?;

    let affected = match edit {
        LaneEdit::Rename { from, .. } => notes_in_lane(store, book_id, &group, from)?,
        LaneEdit::Remove { lane, .. } => notes_in_lane(store, book_id, &group, lane)?,
        _ => Vec::new(),
    };
    let put_back = |edits: &[PropertyEdit]| {
        for done in edits.iter().rev() {
            let _ = store.undo_note_property(done, Origin::Ui);
        }
    };
    let mut edits = Vec::new();
    for id in affected {
        let current = store
            .note(&id)?
            .and_then(|note| {
                let all = properties::parse(&note.body);
                properties::get(&all, &group.key).map(Value::text)
            })
            .unwrap_or_default();
        let Some(text) = value_after(edit, &current) else {
            continue;
        };
        match store.set_note_property(&id, &group.key, &Value::Text(text), now_ms, Origin::Ui) {
            Ok(done) if done.changed => edits.push(done),
            Ok(_) => {}
            Err(err) => {
                put_back(&edits);
                return Err(err);
            }
        }
    }

    let mut new_defs = defs.clone();
    if let Some(def) = new_defs
        .iter_mut()
        .find(|def| def.kind == PropertyType::Select && def.key == group.key)
    {
        def.options = options_after(&def.options, edit);
    }
    let new_spec = BoardSpec {
        by: spec.by.clone(),
        lanes: lanes.clone(),
        show_unset: spec.show_unset,
    };
    let saved = store
        .set_notebook_properties(book_id, &new_defs, Origin::Ui)
        .and_then(|_| store.set_notebook_board(book_id, Some(&new_spec), Origin::Ui));
    if let Err(err) = saved {
        let _ = store.set_notebook_properties(book_id, &defs, Origin::Ui);
        put_back(&edits);
        return Err(err);
    }
    Ok(LaneEdited {
        lanes,
        rewritten: edits.len(),
        undo: SettingsUndo {
            book_id: book_id.to_owned(),
            spec,
            defs,
            edits,
        },
    })
}

/// Puts the board back as `undo` found it: its layout, the property list,
/// and each rewritten note. A note edited since keeps its newer words;
/// `false` says there was one.
pub fn undo_lane_edit(store: &Store, undo: &SettingsUndo) -> Result<bool, StoreError> {
    store.set_notebook_properties(&undo.book_id, &undo.defs, Origin::Ui)?;
    store.set_notebook_board(&undo.book_id, Some(&undo.spec), Origin::Ui)?;
    let mut all = true;
    for edit in undo.edits.iter().rev() {
        all &= store.undo_note_property(edit, Origin::Ui)?;
    }
    Ok(all)
}

/// The grouping a board saves when it is grouped by `by`: the notebook's
/// own lanes stay when it keeps its property, and the lane of notes with no
/// value keeps its setting (shown, for a board that had none). Pure.
pub fn grouped_spec(own: Option<&BoardSpec>, by: &str) -> BoardSpec {
    let kept = own.filter(|own| own.by == by);
    BoardSpec {
        by: by.to_owned(),
        lanes: kept.map(|own| own.lanes.clone()).unwrap_or_default(),
        show_unset: own.is_none_or(|own| own.show_unset),
    }
}

/// The notice for a written edit: "Renamed Shooting to Filming. 3 notes
/// updated." `key` is the board's property. Pure.
pub fn edit_words(edit: &LaneEdit, rewritten: usize, key: &str) -> String {
    let notes = |n: usize| match n {
        1 => "1 note".to_owned(),
        n => format!("{n} notes"),
    };
    match edit {
        LaneEdit::Rename { from, to } => {
            let base = format!("Renamed {from} to {}.", to.trim());
            if rewritten == 0 {
                base
            } else {
                format!("{base} {} updated.", notes(rewritten))
            }
        }
        LaneEdit::Move { .. } => "Moved a lane.".to_owned(),
        LaneEdit::Add(name) => format!("Added the lane {}.", name.trim()),
        LaneEdit::Remove { lane, move_to } => {
            let base = format!("Removed {lane}.");
            match (rewritten, move_to) {
                (0, _) => base,
                (n, Some(to)) => format!("{base} {} moved to {to}.", notes(n)),
                (n, None) => format!("{base} {} moved to No {key}.", notes(n)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::DocType;
    use kanso_core::schema::PropertyDef;
    use kanso_store::Put;
    use proptest::prelude::*;
    use proptest::test_runner::{Config, RngAlgorithm, RngSeed};
    use serde_json::json;

    fn names(of: &[&str]) -> Vec<String> {
        of.iter().map(|s| (*s).to_owned()).collect()
    }

    // no-prop-test: examples of each edit's wording; the rules are the
    // property below.
    #[test]
    fn the_edits_change_the_lanes() {
        let lanes = names(&["Idea", "Scripting", "Shooting"]);
        let rename = LaneEdit::Rename {
            from: "Shooting".into(),
            to: " Filming ".into(),
        };
        assert_eq!(
            lanes_after(&lanes, &rename).unwrap(),
            names(&["Idea", "Scripting", "Filming"])
        );
        let reorder = LaneEdit::Move { from: 2, to: 0 };
        assert_eq!(
            lanes_after(&lanes, &reorder).unwrap(),
            names(&["Shooting", "Idea", "Scripting"])
        );
        assert_eq!(
            lanes_after(&lanes, &LaneEdit::Add("Editing".into()))
                .unwrap()
                .len(),
            4
        );
        assert_eq!(
            lanes_after(&lanes, &LaneEdit::Add("Idea".into())),
            Err(LaneEditError::Duplicate("Idea".into()))
        );
        assert_eq!(
            lanes_after(&lanes, &LaneEdit::Add("  ".into())),
            Err(LaneEditError::Empty)
        );
        let remove = LaneEdit::Remove {
            lane: "Shooting".into(),
            move_to: Some("Gone".into()),
        };
        assert_eq!(
            lanes_after(&lanes, &remove),
            Err(LaneEditError::Target("Gone".into()))
        );
        let last = LaneEdit::Remove {
            lane: "Only".into(),
            move_to: None,
        };
        assert_eq!(
            lanes_after(&names(&["Only"]), &last),
            Err(LaneEditError::LastLane)
        );
    }

    // no-prop-test: the exact copy.
    #[test]
    fn the_prompt_counts_the_notes() {
        assert_eq!(
            removal_prompt("Shooting", 3),
            "3 notes are in Shooting. Move them to:"
        );
        assert_eq!(
            removal_prompt("Shooting", 1),
            "1 note is in Shooting. Move it to:"
        );
    }

    fn setup() -> (tempfile::TempDir, Store, String) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        store
            .set_notebook_properties(
                &book,
                &[PropertyDef {
                    key: "stage".into(),
                    kind: PropertyType::Select,
                    options: names(&["Idea", "Scripting", "Shooting"]),
                }],
                Origin::Ui,
            )
            .unwrap();
        store
            .set_notebook_board(
                &book,
                Some(&BoardSpec {
                    by: "stage".into(),
                    lanes: Vec::new(),
                    show_unset: true,
                }),
                Origin::Ui,
            )
            .unwrap();
        (dir, store, book)
    }

    fn note(store: &Store, book: &str, stage: Option<&str>) -> String {
        let body = match stage {
            Some(stage) => format!("---\nstage: {stage}\n---\nbody"),
            None => "body".to_owned(),
        };
        store
            .put(
                DocType::Note,
                json!({"title": "n", "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn stage(store: &Store, id: &str) -> Option<String> {
        let body = store.note(id).unwrap().unwrap().body;
        properties::get(&properties::parse(&body), "stage").map(Value::text)
    }

    // no-prop-test: drives the store through one edit and its undo.
    #[test]
    fn renaming_rewrites_the_notes_and_undo_restores_everything() {
        let (_dir, store, book) = setup();
        let a = note(&store, &book, Some("Shooting"));
        let b = note(&store, &book, Some("Shooting"));
        let c = note(&store, &book, Some("Idea"));
        let edit = LaneEdit::Rename {
            from: "Shooting".into(),
            to: "Filming".into(),
        };
        let done = apply_lane_edit(&store, &book, &edit, 5).unwrap();
        assert_eq!(done.rewritten, 2);
        assert_eq!(stage(&store, &a).as_deref(), Some("Filming"));
        assert_eq!(stage(&store, &b).as_deref(), Some("Filming"));
        assert_eq!(stage(&store, &c).as_deref(), Some("Idea"));
        let spec = store.notebook_board(&book).unwrap().unwrap();
        assert_eq!(spec.lanes, names(&["Idea", "Scripting", "Filming"]));
        let defs = store.notebook_properties(&book).unwrap();
        assert_eq!(defs[0].options, names(&["Idea", "Scripting", "Filming"]));
        assert!(store.note_property_warnings(&a).unwrap().is_empty());

        assert!(undo_lane_edit(&store, &done.undo).unwrap());
        assert_eq!(stage(&store, &a).as_deref(), Some("Shooting"));
        assert!(
            store
                .notebook_board(&book)
                .unwrap()
                .unwrap()
                .lanes
                .is_empty()
        );
        assert_eq!(
            store.notebook_properties(&book).unwrap()[0].options,
            names(&["Idea", "Scripting", "Shooting"])
        );
    }

    // no-prop-test: drives the store through removals and an undo.
    #[test]
    fn removing_moves_the_notes_to_a_lane_or_to_no_value() {
        let (_dir, store, book) = setup();
        let a = note(&store, &book, Some("Shooting"));
        let odd = note(&store, &book, Some("Archived"));
        let to_lane = LaneEdit::Remove {
            lane: "Shooting".into(),
            move_to: Some("Scripting".into()),
        };
        let done = apply_lane_edit(&store, &book, &to_lane, 5).unwrap();
        assert_eq!(stage(&store, &a).as_deref(), Some("Scripting"));
        // A value no lane had is left as it was.
        assert_eq!(stage(&store, &odd).as_deref(), Some("Archived"));
        assert!(undo_lane_edit(&store, &done.undo).unwrap());
        assert_eq!(stage(&store, &a).as_deref(), Some("Shooting"));

        let to_none = LaneEdit::Remove {
            lane: "Shooting".into(),
            move_to: None,
        };
        apply_lane_edit(&store, &book, &to_none, 6).unwrap();
        assert_eq!(stage(&store, &a).as_deref(), Some(""));
        let spec = store.notebook_board(&book).unwrap().unwrap();
        assert_eq!(spec.lanes, names(&["Idea", "Scripting"]));
    }

    // no-prop-test: drives the store through two edits.
    #[test]
    fn adding_and_reordering_change_the_board_only() {
        let (_dir, store, book) = setup();
        let a = note(&store, &book, Some("Idea"));
        apply_lane_edit(&store, &book, &LaneEdit::Add("Editing".into()), 1).unwrap();
        apply_lane_edit(&store, &book, &LaneEdit::Move { from: 3, to: 0 }, 2).unwrap();
        let spec = store.notebook_board(&book).unwrap().unwrap();
        assert_eq!(
            spec.lanes,
            names(&["Editing", "Idea", "Scripting", "Shooting"])
        );
        assert_eq!(stage(&store, &a).as_deref(), Some("Idea"));
    }

    proptest! {
        #![proptest_config(Config {
            cases: 256,
            rng_seed: RngSeed::Fixed(0x006c_616e_6573),
            rng_algorithm: RngAlgorithm::ChaCha,
            failure_persistence: None,
            ..Config::default()
        })]

        /// After any run of settings edits no note is lost and none is in a
        /// lane the board doesn't have: a note that was in a lane is in
        /// one now or has no value, one that was in none is untouched, and
        /// the lanes stay named, different and at least one.
        #[test]
        fn prop_no_note_is_lost_or_left_in_a_missing_lane(
            start in proptest::collection::vec("[a-d]", 1..4),
            values in proptest::collection::vec(proptest::option::of("[a-f]"), 0..12),
            raw in proptest::collection::vec(
                (0u8..4, "[a-f]", 0usize..6, 0usize..6, proptest::option::of("[a-f]")),
                0..12,
            ),
        ) {
            let mut lanes: Vec<String> = Vec::new();
            for name in start {
                if !lanes.contains(&name) {
                    lanes.push(name);
                }
            }
            let mut values: Vec<String> =
                values.into_iter().map(Option::unwrap_or_default).collect();
            for (kind, name, a, b, target) in raw {
                let edit = match kind {
                    0 => LaneEdit::Rename { from: lanes[a % lanes.len()].clone(), to: name },
                    1 => LaneEdit::Move { from: a, to: b },
                    2 => LaneEdit::Add(name),
                    _ => LaneEdit::Remove { lane: lanes[a % lanes.len()].clone(), move_to: target },
                };
                let Ok(next) = lanes_after(&lanes, &edit) else { continue };
                prop_assert!(!next.is_empty());
                let mut unique = next.clone();
                unique.sort();
                unique.dedup();
                prop_assert_eq!(unique.len(), next.len());
                prop_assert!(next.iter().all(|lane| lane == lane.trim() && !lane.is_empty()));
                let before = values.len();
                for value in &mut values {
                    let was = value.clone();
                    if let Some(text) = value_after(&edit, value) {
                        *value = text;
                    }
                    if lanes.contains(&was) {
                        prop_assert!(
                            value.is_empty() || next.contains(value),
                            "{} went to {} outside {:?}", was, value, next
                        );
                    } else {
                        prop_assert_eq!(&was, &*value, "a value in no lane is left alone");
                    }
                }
                prop_assert_eq!(values.len(), before);
                lanes = next;
            }
        }
    }

    proptest! {
        #![proptest_config(Config {
            cases: 64,
            rng_seed: RngSeed::Fixed(0x0067_726f_7570),
            rng_algorithm: RngAlgorithm::ChaCha,
            failure_persistence: None,
            ..Config::default()
        })]

        /// Grouping keeps the lanes exactly when the property stays, and
        /// the empty lane's setting except for a board that had none.
        #[test]
        fn prop_grouping_keeps_the_lanes_of_the_same_property(
            own_by in "[a-c]", by in "[a-c]",
            lanes in prop::collection::vec("[A-Z][a-z]{0,5}", 0..5),
            show in any::<bool>(),
            has_board in any::<bool>(),
        ) {
            let own = BoardSpec { by: own_by.clone(), lanes: lanes.clone(), show_unset: show };
            let own = has_board.then_some(&own);
            let spec = grouped_spec(own, &by);
            prop_assert_eq!(&spec.by, &by);
            prop_assert_eq!(spec.show_unset, own.is_none_or(|own| own.show_unset));
            if has_board && own_by == by {
                prop_assert_eq!(spec.lanes, lanes);
            } else {
                prop_assert!(spec.lanes.is_empty());
            }
        }

        /// Every edit has words that name what it touched and end in a
        /// full stop, and the count agrees with the notes' number.
        #[test]
        fn prop_the_words_name_the_lane_and_count_the_notes(
            from in "[A-Z][a-z]{1,5}", to in "[A-Z][a-z]{1,5}", n in 0usize..5,
        ) {
            let rename = edit_words(&LaneEdit::Rename { from: from.clone(), to: to.clone() }, n, "stage");
            prop_assert!(rename.starts_with(&format!("Renamed {from} to {to}.")), "{}", rename);
            prop_assert_eq!(rename.contains("updated"), n > 0);
            let remove = edit_words(
                &LaneEdit::Remove { lane: from.clone(), move_to: Some(to.clone()) }, n, "stage");
            prop_assert_eq!(remove.contains(&format!("moved to {to}.")), n > 0);
            let none = edit_words(&LaneEdit::Remove { lane: from, move_to: None }, n, "stage");
            prop_assert_eq!(none.contains("moved to No stage."), n > 0);
            prop_assert!(none.ends_with('.'));
        }
    }
}
