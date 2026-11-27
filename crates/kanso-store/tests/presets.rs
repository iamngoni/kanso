//! A notebook preset with a tree: icons, notebooks inside, notes, tags and
//! keyed templates, made whole in one transaction.

use kanso_core::roles::Role;
use kanso_core::{DocType, NoteStatus, TagColor};
use kanso_store::{
    NotebookPreset, Origin, PresetNote, PresetTag, PresetTemplate, Put, Store, StoreError,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Value, json};

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn config() -> ProptestConfig {
    // The seed comes from PROPTEST_RNG_SEED.
    ProptestConfig {
        cases: 24,
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..ProptestConfig::default()
    }
}

fn template(title: &str, key: Option<&str>) -> PresetTemplate {
    PresetTemplate {
        title: title.into(),
        body: format!("## {title}\n- What?\n"),
        key: key.map(str::to_owned),
    }
}

fn note(title: &str) -> PresetNote {
    PresetNote {
        title: title.into(),
        body: format!("# {title}\n"),
        status: NoteStatus::Active,
    }
}

fn tag(name: &str, color: TagColor) -> PresetTag {
    PresetTag {
        name: name.into(),
        color,
    }
}

/// A notebook with two children, each with a keyed template and a note, and
/// two tags.
fn tree(name: &str, parent: Option<String>) -> NotebookPreset {
    let mut child = |name: &str, icon: &str, key: &str| {
        let mut book = NotebookPreset::named(name, None);
        book.icon = Some(icon.into());
        book.templates = vec![template(&format!("{name} template"), Some(key))];
        book.notes = vec![note(&format!("{name} note"))];
        book
    };
    let mut root = NotebookPreset::named(name, parent);
    root.icon = Some("rocket".into());
    root.children = vec![
        child("Features", "lightbulb", "test/feature"),
        child("Bugs", "bug", "test/bug"),
    ];
    root.tags = vec![
        tag("web", TagColor::Blue),
        tag("important", TagColor::Orange),
    ];
    root
}

fn doc(store: &Store, id: &str) -> Value {
    store.get_api(id, None, false).unwrap()
}

fn books(store: &Store) -> Vec<Value> {
    store.list_api(DocType::Book, None, 0).unwrap()
}

fn lock(store: &Store, id: &str) {
    let mut book = doc(store, id);
    book.as_object_mut().unwrap().remove("count");
    book["kansoLock"] =
        json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"});
    store.lock_book(book, Vec::new()).unwrap();
}

#[test]
fn a_tree_makes_every_notebook_note_tag_and_template_in_order() {
    let (_dir, store) = store();
    let made = store
        .create_notebook_preset(&tree("  Product ", None), Origin::Ui)
        .unwrap();

    let root = doc(&store, &made.book_id);
    assert_eq!(root["name"], "Product");
    assert_eq!(root["kansoIcon"], "rocket");
    assert_eq!(made.child_ids.len(), 2);
    for (id, (name, icon)) in made
        .child_ids
        .iter()
        .zip([("Features", "lightbulb"), ("Bugs", "bug")])
    {
        let book = doc(&store, id);
        assert_eq!(book["name"], name);
        assert_eq!(book["kansoIcon"], icon);
        assert_eq!(book["parentBookId"], made.book_id.as_str());
    }

    // Each child names its own template, and the template is in Templates.
    let templates_book = store.role_book(Role::Templates, Origin::Ui).unwrap();
    assert_eq!(made.template_ids.len(), 2);
    for (child, template_id) in made.child_ids.iter().zip(&made.template_ids) {
        assert_eq!(doc(&store, child)["kansoTemplates"], json!([template_id]));
        let template = doc(&store, template_id);
        assert_eq!(template["bookId"], templates_book.as_str());
    }
    assert_eq!(
        doc(&store, &made.template_ids[0])["kansoStarterTemplate"],
        "test/feature"
    );

    // One note in each child, Active.
    assert_eq!(made.note_ids.len(), 2);
    for (child, id) in made.child_ids.iter().zip(&made.note_ids) {
        let made_note = doc(&store, id);
        assert_eq!(made_note["bookId"], child.as_str());
        assert_eq!(made_note["status"], "active");
    }

    // The tags, with their colors.
    assert_eq!(made.tag_ids.len(), 2);
    assert_eq!(doc(&store, &made.tag_ids[0])["name"], "web");
    assert_eq!(doc(&store, &made.tag_ids[0])["color"], "blue");
    assert_eq!(doc(&store, &made.tag_ids[1])["color"], "orange");
}

#[test]
fn a_template_with_a_key_is_reused_by_the_next_preset() {
    let (_dir, store) = store();
    let first = store
        .create_notebook_preset(&tree("One", None), Origin::Ui)
        .unwrap();
    let second = store
        .create_notebook_preset(&tree("Two", None), Origin::Ui)
        .unwrap();
    assert_eq!(first.template_ids, second.template_ids);
    assert_ne!(first.book_id, second.book_id);
    let templates = store.templates_for(None).unwrap();
    assert_eq!(templates.len(), 2, "no copies in Templates");
    // The second notebook still names them.
    assert_eq!(
        doc(&store, &second.child_ids[0])["kansoTemplates"],
        json!([first.template_ids[0]])
    );
}

#[test]
fn a_trashed_keyed_template_is_not_reused() {
    let (_dir, store) = store();
    let first = store
        .create_notebook_preset(&tree("One", None), Origin::Ui)
        .unwrap();
    let mut trashed = doc(&store, &first.template_ids[0]);
    trashed["bookId"] = kanso_core::TRASH_BOOK_ID.into();
    store
        .put(DocType::Note, trashed, Put::Upsert, Origin::Ui)
        .unwrap();
    let second = store
        .create_notebook_preset(&tree("Two", None), Origin::Ui)
        .unwrap();
    assert_ne!(first.template_ids[0], second.template_ids[0]);
    assert_eq!(first.template_ids[1], second.template_ids[1]);
}

#[test]
fn templates_without_a_key_are_always_made_new() {
    let (_dir, store) = store();
    let mut preset = NotebookPreset::named("Studio", None);
    preset.templates = vec![template("Brief", None)];
    let first = store.create_notebook_preset(&preset, Origin::Ui).unwrap();
    let second = store.create_notebook_preset(&preset, Origin::Ui).unwrap();
    assert_ne!(first.template_ids, second.template_ids);
    assert!(first.child_ids.is_empty() && first.tag_ids.is_empty() && first.note_ids.is_empty());
    assert!(doc(&store, &first.book_id).get("kansoIcon").is_none());
}

#[test]
fn an_existing_tag_is_reused_whatever_its_case_and_keeps_its_color() {
    let (_dir, store) = store();
    let mine = store
        .put(
            DocType::Tag,
            json!({"name": "WEB", "color": "red"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let made = store
        .create_notebook_preset(&tree("Product", None), Origin::Ui)
        .unwrap();
    assert_eq!(made.tag_ids[0], mine);
    assert_eq!(doc(&store, &mine)["color"], "red");
    assert_eq!(store.list_api(DocType::Tag, None, 0).unwrap().len(), 2);
}

#[test]
fn tags_that_differ_only_in_case_inside_a_preset_make_one() {
    let (_dir, store) = store();
    let mut preset = NotebookPreset::named("Product", None);
    preset.tags = vec![tag("Web", TagColor::Blue), tag("web", TagColor::Green)];
    let made = store.create_notebook_preset(&preset, Origin::Ui).unwrap();
    assert_eq!(made.tag_ids[0], made.tag_ids[1]);
    assert_eq!(store.list_api(DocType::Tag, None, 0).unwrap().len(), 1);
}

#[test]
fn a_starter_that_makes_notes_is_refused_inside_a_locked_notebook() {
    let (_dir, store) = store();
    let outer = store
        .put(
            DocType::Book,
            json!({"name": "Secret"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    lock(&store, &outer);
    let before = store.list_api(DocType::Note, None, 0).unwrap().len();
    let err = store
        .create_notebook_preset(&tree("Product", Some(outer.clone())), Origin::Ui)
        .unwrap_err();
    assert!(matches!(
        &err,
        StoreError::Invalid(message)
            if message == "Starters can't be made inside a locked notebook. Make it at the top level, then move it in."
    ));
    // Nothing was made: no notebook inside, no tags, no new notes.
    assert_eq!(books(&store).len(), 1);
    assert!(store.list_api(DocType::Tag, None, 0).unwrap().is_empty());
    assert_eq!(
        store.list_api(DocType::Note, None, 0).unwrap().len(),
        before
    );

    // A preset with no notes (Video studio) still goes in.
    let mut studio = NotebookPreset::named("Studio", Some(outer));
    studio.templates = vec![template("Brief", None)];
    store.create_notebook_preset(&studio, Origin::Ui).unwrap();
}

#[test]
fn a_failure_part_way_leaves_nothing_behind() {
    let (_dir, store) = store();
    let mut preset = tree("Product", None);
    // Too long a name for the second child, found after earlier work.
    preset.children[1].name = "x".repeat(200);
    assert!(store.create_notebook_preset(&preset, Origin::Ui).is_err());
    assert!(books(&store).is_empty());
    assert!(store.list_api(DocType::Tag, None, 0).unwrap().is_empty());
    assert!(store.list_api(DocType::Note, None, 0).unwrap().is_empty());
}

#[test]
fn a_bad_tree_is_refused() {
    let (_dir, store) = store();
    let refused = |preset: NotebookPreset| {
        assert!(matches!(
            store.create_notebook_preset(&preset, Origin::Ui),
            Err(StoreError::Invalid(_))
        ));
    };
    // Grandchildren.
    let mut deep = tree("Product", None);
    deep.children[0].children = vec![NotebookPreset::named("Deeper", None)];
    refused(deep);
    // A child with tags.
    let mut tagged = tree("Product", None);
    tagged.children[0].tags = vec![tag("x", TagColor::Blue)];
    refused(tagged);
    // Too many children, tags and notes.
    let mut many = NotebookPreset::named("Product", None);
    many.children = (0..13)
        .map(|i| NotebookPreset::named(format!("B{i}"), None))
        .collect();
    refused(many);
    let mut tags = NotebookPreset::named("Product", None);
    tags.tags = (0..21)
        .map(|i| tag(&format!("t{i}"), TagColor::Blue))
        .collect();
    refused(tags);
    let mut notes = NotebookPreset::named("Product", None);
    notes.notes = (0..13).map(|i| note(&format!("n{i}"))).collect();
    refused(notes);
    // A blank name, a blank child name, a bad icon and a bad key.
    refused(NotebookPreset::named("  ", None));
    let mut blank = tree("Product", None);
    blank.children[0].name = " ".into();
    refused(blank);
    let mut icon = tree("Product", None);
    icon.icon = Some("x".repeat(41));
    refused(icon);
    let mut key = tree("Product", None);
    key.children[0].templates[0].key = Some("Bad Key".into());
    refused(key);
    assert!(books(&store).is_empty());
}

#[test]
fn a_starter_key_with_a_bad_shape_is_refused_on_a_note() {
    let (_dir, store) = store();
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let put = |key: Value| {
        store.put(
            DocType::Note,
            json!({"bookId": book, "title": "t", "body": "", "kansoStarterTemplate": key}),
            Put::Upsert,
            Origin::Ui,
        )
    };
    assert!(put(json!("product/bug-report")).is_ok());
    assert!(put(json!("Upper")).is_err());
    assert!(put(json!("has space")).is_err());
    assert!(put(json!("a".repeat(65))).is_err());
    assert!(put(json!(5)).is_err());
}

proptest! {
    #![proptest_config(config())]

    /// However the names are cased and repeated, one tag is made per name
    /// that differs, and every position gets the ID of its first spelling.
    #[test]
    fn tags_dedupe_whatever_the_case(names in proptest::collection::vec("[a-cA-C]{1,3}", 1..8)) {
        let (_dir, store) = store();
        let mut preset = NotebookPreset::named("Product", None);
        preset.tags = names.iter().map(|n| tag(n, TagColor::Blue)).collect();
        let made = store.create_notebook_preset(&preset, Origin::Ui).unwrap();
        let mut distinct: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
        distinct.sort();
        distinct.dedup();
        prop_assert_eq!(store.list_api(DocType::Tag, None, 0).unwrap().len(), distinct.len());
        for (i, name) in names.iter().enumerate() {
            let first = names.iter().position(|n| n.eq_ignore_ascii_case(name)).unwrap();
            prop_assert_eq!(&made.tag_ids[i], &made.tag_ids[first]);
        }
    }
}
