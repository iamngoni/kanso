//! Snapshot states for the Schedule (#390, slice 27): the month and the week
//! around the demo's Thursday 28 May 2026, with repeating tasks (↻), dated
//! notes (▶ publish, ⏸ embargo) and a done task.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_core::DocType;
use kanso_core::schema::{PropertyDef, PropertyType};
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::schedule_model::Mode;
use crate::workspace::Workspace;

fn seed(store: &Store) {
    let book = store
        .put(
            DocType::Book,
            json!({"_id": "book:studio", "name": "Studio"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let def = |key: &str, kind| PropertyDef {
        key: key.into(),
        kind,
        options: Vec::new(),
    };
    store
        .set_notebook_properties(
            &book,
            &[
                def("publish", PropertyType::DateTimeZone),
                def("embargo", PropertyType::DateTime),
            ],
            Origin::Ui,
        )
        .unwrap();
    let note = |id: &str, title: &str, body: &str| {
        store
            .put(
                DocType::Note,
                json!({"_id": id, "title": title, "bookId": book, "body": body, "updatedAt": 1000}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    };
    note(
        "note:sched-pixel",
        "Pixel 11 review",
        "---\npublish: 2026-05-29T09:00 America/New_York\nembargo: 2026-05-26T10:30\n---\n# Pixel 11 review\n",
    );
    note(
        "note:sched-tour",
        "Studio tour",
        "---\npublish: 2026-06-03T15:00 Europe/London\n---\n# Studio tour\n",
    );
    note(
        "note:sched-shoots",
        "Shoots",
        "# Shoots\n- [ ] Film the weekly roundup @2026-05-22T15:00 every week\n\
         - [ ] Edit the Pixel review @2026-05-28T11:00\n\
         - [x] Pick up the tripod @2026-05-27\n\
         - [ ] Send the invoice @2026-05-29\n\
         - [ ] Book the studio @2026-05-29\n\
         - [ ] Thumbnails @2026-05-29\n\
         - [ ] Pay the editor @2026-05-31 every month\n",
    );
}

fn show(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    mode: Mode,
) {
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| {
        workspace.update(cx, |workspace, cx| workspace.show_schedule(window, cx));
    })
    .unwrap();
    cx.update(|cx| {
        let view = workspace.read(cx).schedule.clone().unwrap();
        view.update(cx, |view, cx| {
            view.set_mode(mode, cx);
            view.reload_now(cx);
        });
    });
    cx.run_until_parked();
}

pub(super) fn month(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    seed(store);
    show(cx, window, workspace, Mode::Month);
}

pub(super) fn week(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    seed(store);
    show(cx, window, workspace, Mode::Week);
}
