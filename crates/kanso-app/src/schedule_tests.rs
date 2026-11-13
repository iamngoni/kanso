//! The Schedule in a headless window (#390, slice 27): clicking ticks a
//! task, a projected occurrence ticks the real task, dragging a note moves
//! its date property (time and zone kept), dragging a task moves its due
//! date, and the filters ask the engine what the grid then shows.

use gpui_kit::{Entity, Modifiers, MouseButton, TestAppContext, point, px};
use kanso_core::DocType;
use kanso_core::dates::Date;
use kanso_core::schema::{PropertyDef, PropertyType};
use kanso_engine::schedule::{ScheduleKind, schedule};
use kanso_store::{Origin, Put};
use serde_json::json;

use crate::clock::Clock;
use crate::schedule_model::{Mode, range_of};
use crate::schedule_view::{ScheduleView, ShowSchedule};
use crate::ui_tests::{Fixture, settle, setup};

const FILM: &str = "film Pixel review @2026-11-13T15:00 every week";
const VIDEO: &str = "---\npublish: 2026-11-18T09:00 America/New_York\nembargo: 2026-11-16T10:30\n---\n# Pixel 11 review\n";

fn d(year: i32, month: u32, day: u32) -> Date {
    Date::new(year, month, day).unwrap()
}

fn body(f: &Fixture, id: &str) -> String {
    f.store.note(id).unwrap().unwrap().body
}

fn def(key: &str, kind: PropertyType) -> PropertyDef {
    PropertyDef {
        key: key.into(),
        kind,
        options: Vec::new(),
    }
}

/// Thursday 12 November 2026, with a studio notebook that has a video with
/// a zoned publish date and an embargo, a note of tasks (one repeating, one
/// done) and another notebook with a date property of its own.
fn seed(f: &Fixture, cx: &mut gpui_kit::VisualTestContext) {
    let now = d(2026, 11, 12).to_days() * 86_400_000 + 9 * 3_600_000;
    cx.update(|_, cx| cx.set_global(Clock::fixed(now, 0)));
    cx.update(|_, cx| crate::workspace::bind_keys(cx));
    let book = |id: &str, name: &str| {
        f.store
            .put(
                DocType::Book,
                json!({"_id": id, "name": name}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    };
    book("book:studio", "Studio");
    book("book:other", "Other");
    f.store
        .set_notebook_properties(
            "book:studio",
            &[
                def("publish", PropertyType::DateTimeZone),
                def("embargo", PropertyType::DateTime),
            ],
            Origin::Ui,
        )
        .unwrap();
    f.store
        .set_notebook_properties("book:other", &[def("due", PropertyType::Date)], Origin::Ui)
        .unwrap();
    let note = |id: &str, book: &str, title: &str, text: &str| {
        f.store
            .put(
                DocType::Note,
                json!({"_id": id, "title": title, "bookId": book, "body": text, "updatedAt": 1000}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    };
    note("note:video", "book:studio", "Pixel 11 review", VIDEO);
    note(
        "note:tasks",
        "book:studio",
        "Shoots",
        &format!("# Shoots\n- [ ] {FILM}\n- [ ] edit @2026-11-13\n- [x] old @2026-11-12\n"),
    );
    note(
        "note:other",
        "book:other",
        "Taxes",
        "---\ndue: 2026-11-14\n---\n# Taxes\n- [ ] file @2026-11-20\n",
    );
}

fn open(f: &Fixture, cx: &mut gpui_kit::VisualTestContext) -> Entity<ScheduleView> {
    seed(f, cx);
    cx.dispatch_action(ShowSchedule);
    settle(f, cx);
    let view = cx.read(|cx| f.workspace.read(cx).schedule.clone().unwrap());
    view.update(cx, |view, cx| view.reload_now(cx));
    settle(f, cx);
    view
}

/// `debug_bounds` names its target for the life of the program.
fn leak(name: String) -> &'static str {
    Box::leak(name.into_boxed_str())
}

fn find(
    view: &Entity<ScheduleView>,
    cx: &mut gpui_kit::VisualTestContext,
    pick: impl Fn(&kanso_engine::schedule::ScheduleItem) -> bool,
) -> usize {
    view.read_with(cx, |view, _| view.items.iter().position(pick))
        .expect("the item is on the Schedule")
}

fn click_item(f: &Fixture, cx: &mut gpui_kit::VisualTestContext, ix: usize) {
    let bounds = cx
        .debug_bounds(leak(format!("schedule-item-{ix}")))
        .expect("the item is drawn");
    cx.simulate_click(bounds.center(), Modifiers::none());
    settle(f, cx);
}

fn drag_item_to_day(f: &Fixture, cx: &mut gpui_kit::VisualTestContext, ix: usize, day: Date) {
    let from = cx
        .debug_bounds(leak(format!("schedule-item-{ix}")))
        .expect("the item is drawn")
        .center();
    let to = cx
        .debug_bounds(leak(format!("schedule-day-{}", day.iso())))
        .expect("the day is drawn")
        .center();
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::none());
    for step in 1..=10 {
        let t = step as f32 / 10.;
        let at = point(from.x + (to.x - from.x) * t, from.y + (to.y - from.y) * t);
        cx.simulate_mouse_move(at, MouseButton::Left, Modifiers::none());
        f.draw_visual(cx);
    }
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
    settle(f, cx);
}

impl Fixture {
    fn draw_visual(&self, vcx: &mut gpui_kit::VisualTestContext) {
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            _ = window.draw(cx);
            window.refresh();
        });
        vcx.run_until_parked();
    }
}

// no-prop-test: drives one window with the pointer; `schedule_model` and
// `kanso_core::reschedule` carry the properties.
#[gpui_kit::test]
fn the_palette_command_and_the_sidebar_open_the_schedule_with_its_days(cx: &mut TestAppContext) {
    let f = setup(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let view = open(&f, &mut vcx);
    assert!(vcx.debug_bounds("schedule-overlay").is_some());
    assert!(vcx.debug_bounds("schedule-day-2026-11-12").is_some());
    assert!(vcx.debug_bounds("schedule-day-2026-11-30").is_some());
    // The repeating task is marked on every Friday it comes round, the
    // note on its publish and embargo dates.
    let items = view.read_with(&vcx, |view, _| view.items.clone());
    let film: Vec<_> = items.iter().filter(|i| i.text == FILM).collect();
    assert_eq!(
        film.iter().map(|i| i.date).collect::<Vec<_>>(),
        vec![
            d(2026, 11, 13),
            d(2026, 11, 20),
            d(2026, 11, 27),
            d(2026, 12, 4)
        ]
    );
    assert!(film.iter().all(|i| i.repeat.is_some()));
    assert!(film[1].projected && !film[0].projected);
    assert!(
        items
            .iter()
            .any(|i| { i.kind == ScheduleKind::Note && i.property.as_deref() == Some("publish") })
    );
}

// no-prop-test: drives one window with the pointer.
#[gpui_kit::test]
fn clicking_a_task_ticks_it_and_clicking_a_done_one_opens_it_again(cx: &mut TestAppContext) {
    let f = setup(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let view = open(&f, &mut vcx);
    let edit = find(&view, &mut vcx, |i| i.text == "edit @2026-11-13");
    click_item(&f, &mut vcx, edit);
    assert!(body(&f, "note:tasks").contains("- [x] edit @2026-11-13\n"));
    let edit = find(&view, &mut vcx, |i| i.text == "edit @2026-11-13");
    assert!(view.read_with(&vcx, |view, _| view.items[edit].done));
    click_item(&f, &mut vcx, edit);
    assert!(body(&f, "note:tasks").contains("- [ ] edit @2026-11-13\n"));
}

// no-prop-test: drives one window with the pointer.
#[gpui_kit::test]
fn ticking_a_projected_occurrence_ticks_the_real_task_and_writes_the_next(cx: &mut TestAppContext) {
    let f = setup(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let view = open(&f, &mut vcx);
    let projected = find(&view, &mut vcx, |i| {
        i.text == FILM && i.projected && i.date == d(2026, 11, 20)
    });
    click_item(&f, &mut vcx, projected);
    let after = body(&f, "note:tasks");
    assert!(
        after.contains("- [x] film Pixel review @2026-11-13T15:00\n"),
        "{after}"
    );
    assert!(
        after.contains("- [ ] film Pixel review @2026-11-20T15:00 every week\n"),
        "{after}"
    );
    // The last change can be taken back, both lines in one write.
    let undone = view.update(&mut vcx, |view, cx| view.undo(cx));
    assert!(undone);
    assert!(body(&f, "note:tasks").contains(&format!("- [ ] {FILM}\n")));
    assert!(!body(&f, "note:tasks").contains("2026-11-20"));
}

// no-prop-test: drives one window with the pointer.
#[gpui_kit::test]
fn dragging_a_note_moves_its_date_property_and_keeps_its_time_and_zone(cx: &mut TestAppContext) {
    let f = setup(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let view = open(&f, &mut vcx);
    let publish = find(&view, &mut vcx, |i| {
        i.kind == ScheduleKind::Note && i.property.as_deref() == Some("publish")
    });
    drag_item_to_day(&f, &mut vcx, publish, d(2026, 11, 25));
    let after = body(&f, "note:video");
    assert!(
        after.contains("publish: 2026-11-25T09:00 America/New_York\n"),
        "{after}"
    );
    // The other date property of the note was not touched.
    assert!(after.contains("embargo: 2026-11-16T10:30\n"), "{after}");
    assert!(after.ends_with("# Pixel 11 review\n"));
    let moved = view.read_with(&vcx, |view, _| {
        view.items
            .iter()
            .find(|i| i.property.as_deref() == Some("publish"))
            .map(|i| (i.date, i.time, i.zone.clone()))
    });
    assert_eq!(
        moved,
        Some((
            d(2026, 11, 25),
            Some((9, 0)),
            Some("America/New_York".to_owned())
        ))
    );
    // And back, with the note put exactly as it was.
    assert!(view.update(&mut vcx, |view, cx| view.undo(cx)));
    assert_eq!(body(&f, "note:video"), VIDEO);
}

// no-prop-test: drives one window with the pointer.
#[gpui_kit::test]
fn dragging_a_date_only_note_changes_the_day(cx: &mut TestAppContext) {
    let f = setup(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let view = open(&f, &mut vcx);
    let due = find(&view, &mut vcx, |i| i.property.as_deref() == Some("due"));
    drag_item_to_day(&f, &mut vcx, due, d(2026, 11, 26));
    assert!(body(&f, "note:other").contains("due: 2026-11-26\n"));
}

// no-prop-test: drives one window with the pointer.
#[gpui_kit::test]
fn dragging_a_task_changes_its_due_date_and_keeps_its_time_and_repeat(cx: &mut TestAppContext) {
    let f = setup(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let view = open(&f, &mut vcx);
    let film = find(&view, &mut vcx, |i| i.text == FILM && !i.projected);
    drag_item_to_day(&f, &mut vcx, film, d(2026, 11, 24));
    let after = body(&f, "note:tasks");
    assert!(
        after.contains("- [ ] film Pixel review @2026-11-24T15:00 every week\n"),
        "{after}"
    );
    let edit = find(&view, &mut vcx, |i| i.text == "edit @2026-11-13");
    drag_item_to_day(&f, &mut vcx, edit, d(2026, 11, 25));
    assert!(body(&f, "note:tasks").contains("- [ ] edit @2026-11-25\n"));
    // A projected occurrence has no date of its own: it doesn't drag.
    let before = body(&f, "note:tasks");
    let projected = find(&view, &mut vcx, |i| {
        i.text.starts_with("film") && i.projected
    });
    drag_item_to_day(&f, &mut vcx, projected, d(2026, 11, 18));
    assert_eq!(body(&f, "note:tasks"), before);
}

// no-prop-test: drives one window with the pointer.
#[gpui_kit::test]
fn the_filters_ask_the_engine_and_the_grid_shows_what_it_returns(cx: &mut TestAppContext) {
    let f = setup(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let view = open(&f, &mut vcx);
    let shown = |vcx: &mut gpui_kit::VisualTestContext| {
        view.read_with(vcx, |view, _| {
            let query = view.filters.query(range_of(view.mode, view.focused, 0));
            let _ = query;
            view.items.clone()
        })
    };
    let engine = |vcx: &mut gpui_kit::VisualTestContext| {
        view.read_with(vcx, |view, cx| {
            let _ = cx;
            let query = view.filters.query(view_range(view));
            schedule(&f.store, &query).unwrap()
        })
    };
    let all = shown(&mut vcx);
    assert!(all.iter().any(|i| i.book_id == "book:other"));
    assert_eq!(all, engine(&mut vcx));

    // A notebook.
    view.update(&mut vcx, |view, cx| view.set_book(Some("book:studio"), cx));
    view.update(&mut vcx, |view, cx| view.reload_now(cx));
    let studio = shown(&mut vcx);
    assert!(!studio.is_empty() && studio.iter().all(|i| i.book_id == "book:studio"));
    assert_eq!(studio, engine(&mut vcx));

    // A date property: only that one places a note; tasks stay.
    view.update(&mut vcx, |view, cx| view.set_key(Some("embargo"), cx));
    view.update(&mut vcx, |view, cx| view.reload_now(cx));
    let embargo = shown(&mut vcx);
    assert!(
        embargo
            .iter()
            .filter(|i| i.kind == ScheduleKind::Note)
            .all(|i| i.property.as_deref() == Some("embargo"))
    );
    assert!(embargo.iter().any(|i| i.kind == ScheduleKind::Task));
    assert_eq!(embargo, engine(&mut vcx));

    // Done tasks hide.
    assert!(embargo.iter().any(|i| i.done));
    view.update(&mut vcx, |view, cx| view.set_hide_done(true, cx));
    view.update(&mut vcx, |view, cx| view.reload_now(cx));
    let open_only = shown(&mut vcx);
    assert!(open_only.iter().all(|i| !i.done));
    assert_eq!(open_only, engine(&mut vcx));

    // Back to everything.
    view.update(&mut vcx, |view, cx| {
        view.set_book(None, cx);
        view.set_key(None, cx);
        view.set_hide_done(false, cx);
    });
    view.update(&mut vcx, |view, cx| view.reload_now(cx));
    assert_eq!(shown(&mut vcx), all);
    // The property list names the date properties of the notebooks.
    assert_eq!(
        view.read_with(&vcx, |view, _| view.date_keys.clone()),
        vec!["due", "embargo", "publish"]
    );
}

fn view_range(view: &ScheduleView) -> std::ops::Range<Date> {
    range_of(
        view.mode,
        view.focused,
        crate::calendar::system_week_start(),
    )
}

// no-prop-test: drives one window with the keyboard.
#[gpui_kit::test]
fn the_keyboard_moves_between_days_ticks_and_switches_to_the_week(cx: &mut TestAppContext) {
    let f = setup(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let view = open(&f, &mut vcx);
    // Thursday 12: nothing open on it; Right is Friday 13, J the first of
    // its items, Space ticks it.
    vcx.simulate_keystrokes("right");
    assert_eq!(
        view.read_with(&vcx, |view, _| view.focused),
        d(2026, 11, 13)
    );
    vcx.simulate_keystrokes("j space");
    settle(&f, &mut vcx);
    assert!(body(&f, "note:tasks").contains("- [x] edit @2026-11-13\n"));
    // Page Down is a month; W is the week.
    vcx.simulate_keystrokes("pagedown");
    assert_eq!(
        view.read_with(&vcx, |view, _| view.focused),
        d(2026, 12, 13)
    );
    vcx.simulate_keystrokes("w");
    assert_eq!(view.read_with(&vcx, |view, _| view.mode), Mode::Week);
    assert!(vcx.debug_bounds("schedule-day-2026-12-13").is_some());
    assert!(vcx.debug_bounds("schedule-day-2026-11-12").is_none());
    vcx.simulate_keystrokes("t");
    assert_eq!(
        view.read_with(&vcx, |view, _| view.focused),
        d(2026, 11, 12)
    );
    // Escape closes it.
    vcx.simulate_keystrokes("escape");
    settle(&f, &mut vcx);
    assert!(vcx.debug_bounds("schedule-overlay").is_none());
    let _ = px(0.);
}
