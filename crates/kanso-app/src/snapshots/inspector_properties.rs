//! Snapshot states for the inspector's property pickers (issue #390,
//! slice 24): one note per property type, with the notebook's property list
//! naming each, shown beside the note.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_core::schema::{PropertyDef, PropertyType};
use kanso_store::{Origin, Store};

use crate::config::InspectorTab;
use crate::workspace::Workspace;

/// The shots: name (`dark-…` or `light-…`), how to set the window up, and
/// the width of a dialog (none here).
pub const SHOTS: &[(&str, super::Prepare, Option<f32>)] = &[
    ("dark-inspector-property-select", select, None),
    ("light-inspector-property-select", select, None),
    ("dark-inspector-property-date", date, None),
    ("light-inspector-property-date", date, None),
    ("dark-inspector-property-datetime", datetime, None),
    ("light-inspector-property-datetime", datetime, None),
    ("dark-inspector-property-zone", zone, None),
    ("light-inspector-property-zone", zone, None),
    ("dark-inspector-property-person", person, None),
    ("light-inspector-property-person", person, None),
];

fn def(key: &str, kind: PropertyType, options: &[&str]) -> PropertyDef {
    PropertyDef {
        key: key.into(),
        kind,
        options: options.iter().map(|o| (*o).to_owned()).collect(),
    }
}

/// A note with `body`, in the notebook whose property list is `defs`, open
/// with the inspector beside it; `members` are the people of a shared note.
fn show(
    cx: &mut HeadlessAppContext,
    workspace: &Entity<Workspace>,
    store: &Store,
    body: &str,
    defs: &[PropertyDef],
    members: &[&str],
) {
    // The notebook the new note lands in: the first listed note's.
    let book = store
        .search_summaries(&Default::default())
        .ok()
        .and_then(|notes| notes.into_iter().next())
        .and_then(|n| store.note(&n.id).ok().flatten())
        .map(|note| note.book_id)
        .unwrap();
    store
        .set_notebook_properties(&book, defs, Origin::Ui)
        .unwrap();
    super::open_new_note(cx, workspace, store, "Pixel 11 review", body);
    cx.run_until_parked();
    let members: Vec<String> = members.iter().map(|m| (*m).to_owned()).collect();
    cx.update(|cx| {
        let inspector = workspace.read(cx).inspector.clone();
        inspector.update(cx, |inspector, _| inspector.set_members_for_test(members));
        workspace.update(cx, |workspace, cx| {
            workspace.show_inspector(Some(InspectorTab::Outline), cx)
        })
    });
}

const BODY: &str = "# Pixel 11 review\n\n## Angle\n\nWhy this phone, why now.\n\n## Shot list\n\n- [ ] A-roll intro\n- [ ] B-roll product hero\n";

fn note(front: &str) -> String {
    format!("---\n{front}---\n{BODY}")
}

/// A stage chosen from the notebook's five.
fn select(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    show(
        cx,
        workspace,
        store,
        &note("stage: Shooting\nsponsor: Acme\n"),
        &[def(
            "stage",
            PropertyType::Select,
            &["Idea", "Scripting", "Shooting", "Editing", "Published"],
        )],
        &[],
    );
}

/// A publish day.
fn date(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    show(
        cx,
        workspace,
        store,
        &note("stage: Editing\nreview: 2026-11-18\n"),
        &[def("review", PropertyType::Date, &[])],
        &[],
    );
}

/// A publish day and time.
fn datetime(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    show(
        cx,
        workspace,
        store,
        &note("stage: Editing\npublish: 2026-11-20T15:00\n"),
        &[def("publish", PropertyType::DateTime, &[])],
        &[],
    );
}

/// An embargo: a time in a zone, and the zone beside it.
fn zone(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    show(
        cx,
        workspace,
        store,
        &note("stage: Editing\nembargo: 2026-11-18T09:00 America/New_York\n"),
        &[def("embargo", PropertyType::DateTimeZone, &[])],
        &[],
    );
}

/// An owner chosen from the people in a shared note.
fn person(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    show(
        cx,
        workspace,
        store,
        &note("stage: Scripting\nowner: amy\n"),
        &[def("owner", PropertyType::Person, &[])],
        &["amy", "bo", "carla"],
    );
}
