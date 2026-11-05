//! Pencil ink in the reader, headless and without a worker (issue #43, step
//! 10): the pointer draws, the erasers rub out, undo and redo walk back, and
//! what is drawn is saved with the highlights in the canvas's stroke format.

use gpui_kit::TestAppContext;
use kanso_pdf::highlights::Status;
use kanso_pdf::ink::PAGE_UNITS;
use kanso_pdf::{Annotations, PageInk};
use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngSeed, TestRunner};

use super::super::annotate::tests::{Fixture, click_selector, drag, draw, on_reader, open, shown};
use super::super::annotate::{SAVE_DELAY, Write};
use crate::canvas_state::{CanvasState, Outcome, PenSample, Source, Tool};

fn saves(f: &Fixture) -> Vec<Annotations> {
    f.writes
        .borrow()
        .iter()
        .filter_map(|w| match w {
            Write::Save(a) => Some((**a).clone()),
            _ => None,
        })
        .collect()
}

fn settle(cx: &mut TestAppContext) {
    cx.executor()
        .advance_clock(SAVE_DELAY + std::time::Duration::from_millis(1));
    cx.run_until_parked();
}

/// A horizontal stroke at `y` units from `x0` to `x1` with rising pressure.
fn line(y: f64, x0: f64, x1: f64) -> Vec<(f64, f64, f64)> {
    (0..=12)
        .map(|i| {
            let t = f64::from(i) / 12.0;
            (x0 + (x1 - x0) * t, y, 0.2 + 0.6 * t)
        })
        .collect()
}

fn pick_up(cx: &mut TestAppContext, f: &Fixture) {
    click_selector(cx, f, "pdf-pencil");
    assert!(on_reader(cx, f, |r, _, _| r.test_pencil_on()));
}

/// The pencil button picks it up: the bar shows, the pointer draws, and
/// Done (or Escape) puts it down.
#[gpui_kit::test]
fn the_pencil_button_shows_the_bar_and_draws_with_the_pointer(cx: &mut TestAppContext) {
    let f = open(cx, "", 2);
    assert!(!shown(cx, &f, "pdf-ink-bar"));
    pick_up(cx, &f);
    for part in [
        "pdf-ink-bar",
        "pdf-ink-pen",
        "pdf-ink-pencil",
        "pdf-ink-marker",
        "pdf-ink-highlighter",
        "pdf-ink-eraser",
        "pdf-ink-partial",
        "pdf-ink-undo",
        "pdf-ink-redo",
        "pdf-ink-done",
    ] {
        assert!(shown(cx, &f, part), "{part} is missing from the bar");
    }
    drag(cx, &f, (0.2, 0.5), (0.7, 0.55));
    let ink = on_reader(cx, &f, |r, _, _| r.test_ink(1));
    assert_eq!(ink.live_count(), 1);
    // A mouse writes the pressure a mouse writes, so the outline widens with speed.
    assert!(ink.strokes[0].points.len() >= 2);
    assert!(
        shown(cx, &f, "pdf-ink-0"),
        "the ink is painted over the page"
    );
    // Drawing selected no text.
    assert!(on_reader(cx, &f, |r, _, _| r.notes.selection.is_none()));
    click_selector(cx, &f, "pdf-ink-done");
    assert!(!on_reader(cx, &f, |r, _, _| r.test_pencil_on()));
    assert!(!shown(cx, &f, "pdf-ink-bar"));
    // The ink stays on the page after the pencil is down.
    assert!(shown(cx, &f, "pdf-ink-0"));
}

/// A stroke with pressure is stored in the canvas's stroke format, and is
/// saved with the block after the pause (one save for a burst).
#[gpui_kit::test]
fn a_pencil_stroke_with_pressure_is_stored_and_saved_once(cx: &mut TestAppContext) {
    let f = open(cx, "", 2);
    pick_up(cx, &f);
    on_reader(cx, &f, |r, window, cx| {
        r.test_pencil_tool(Tool::Pencil, cx);
        r.test_stroke(1, &line(300., 100., 800.), window, cx);
        r.test_stroke(1, &line(400., 100., 800.), window, cx);
        r.test_stroke(2, &line(300., 100., 800.), window, cx);
    });
    assert!(saves(&f).is_empty(), "the block waits for the pause");
    settle(cx);
    let saved = saves(&f);
    assert_eq!(saved.len(), 1, "a burst is one save");
    let ink = saved[0].ink();
    assert_eq!(ink.len(), 2);
    let first = &ink[0].strokes[0];
    assert_eq!(first.tool, "pencil");
    let pressures: Vec<f64> = first.points.iter().map(|p| p.pressure).collect();
    assert!(
        pressures.windows(2).all(|w| w[0] <= w[1])
            && pressures[0] < 0.3
            && pressures[pressures.len() - 1] > 0.7,
        "the pressure was kept: {pressures:?}"
    );
    // The body reads back to the same ink.
    let body = saved[0].save_edit("Notes").unwrap().unwrap().apply("Notes");
    let back = Annotations::from_body(&body, &f.id, "");
    assert_eq!(back.ink(), saved[0].ink());
}

/// The partial eraser cuts a stroke in two; the stroke eraser takes it whole;
/// undo and redo walk back and forth.
#[gpui_kit::test]
fn both_erasers_work_and_undo_walks_back(cx: &mut TestAppContext) {
    let f = open(cx, "", 2);
    pick_up(cx, &f);
    on_reader(cx, &f, |r, window, cx| {
        r.test_stroke(1, &line(300., 100., 900.), window, cx);
    });
    // The partial eraser, over the middle.
    on_reader(cx, &f, |r, window, cx| {
        r.test_pencil_tool(Tool::Partial, cx);
        r.test_stroke(1, &[(500., 300., 0.5), (505., 300., 0.5)], window, cx);
    });
    let ink = on_reader(cx, &f, |r, _, _| r.test_ink(1));
    assert!(ink.live_count() >= 2, "the stroke was cut in two");
    assert!(
        ink.live()
            .all(|s| s.points.iter().all(|p| !(490.0..=515.0).contains(&p.x))),
        "nothing is left under the eraser"
    );
    // Undo puts the stroke back as it was.
    assert!(on_reader(cx, &f, |r, window, cx| r.test_undo(window, cx)));
    let ink = on_reader(cx, &f, |r, _, _| r.test_ink(1));
    assert_eq!(ink.live_count(), 1);
    assert!(on_reader(cx, &f, |r, window, cx| r.test_redo(window, cx)));
    assert!(on_reader(cx, &f, |r, _, _| r.test_ink(1)).live_count() >= 2);
    // The stroke eraser takes a piece whole.
    on_reader(cx, &f, |r, window, cx| {
        r.test_pencil_tool(Tool::Eraser, cx);
        r.test_stroke(1, &[(200., 300., 0.5), (202., 300., 0.5)], window, cx);
    });
    let left = on_reader(cx, &f, |r, _, _| r.test_ink(1)).live_count();
    assert!(left >= 1);
    // Undo everything: the page is blank again.
    while on_reader(cx, &f, |r, window, cx| r.test_undo(window, cx)) {}
    assert!(on_reader(cx, &f, |r, _, _| r.test_ink(1)).is_blank());
    // And what was erased stays a tombstone, so a merge cannot bring it back.
    let page = on_reader(cx, &f, |r, _, _| r.test_ink(1));
    assert!(page.strokes.iter().all(|s| s.deleted));
}

/// Clear in the panel rubs a page out and says how many strokes it had.
#[gpui_kit::test]
fn the_panel_lists_pages_with_pencil_and_clears_one(cx: &mut TestAppContext) {
    let f = open(cx, "", 2);
    pick_up(cx, &f);
    on_reader(cx, &f, |r, window, cx| {
        r.test_stroke(1, &line(300., 100., 900.), window, cx);
        r.test_stroke(1, &line(500., 100., 900.), window, cx);
        r.test_show_panel();
    });
    assert!(shown(cx, &f, "pdf-ink-row-1"));
    assert!(shown(cx, &f, "pdf-ink-clear-1"));
    click_selector(cx, &f, "pdf-ink-clear-1");
    assert!(on_reader(cx, &f, |r, _, _| r.test_ink(1)).is_blank());
    assert!(!shown(cx, &f, "pdf-ink-row-1"));
    settle(cx);
    let last = saves(&f).pop().expect("the clearing was saved");
    assert!(last.ink_rows().is_empty());
}

/// Ink another editor or device added shows up; ink the note lost does not
/// leave the pencil drawing it.
#[gpui_kit::test]
fn the_pencil_follows_the_note(cx: &mut TestAppContext) {
    let f = open(cx, "", 2);
    pick_up(cx, &f);
    on_reader(cx, &f, |r, window, cx| {
        r.test_stroke(1, &line(300., 100., 900.), window, cx);
        r.flush(cx);
    });
    let mut other = Annotations::new(&f.id, "");
    other
        .set_page_ink(PageInk {
            page: 1,
            strokes: {
                let mut s = on_reader(cx, &f, |r, _, _| r.test_ink(1)).strokes[0].clone();
                s.id = "elsewhere".into();
                s.points.iter_mut().for_each(|p| p.y += 200.0);
                vec![s]
            },
            extra: Default::default(),
        })
        .unwrap();
    let body = other.save_edit("Notes").unwrap().unwrap().apply("Notes");
    on_reader(cx, &f, |r, _, cx| r.note_changed(body.clone(), cx));
    let ink = on_reader(cx, &f, |r, _, _| r.test_ink(1));
    assert!(ink.live().any(|s| s.id == "elsewhere"));
}

/// A note whose highlights are read-only takes no ink, and says why.
#[gpui_kit::test]
fn a_read_only_note_takes_no_ink(cx: &mut TestAppContext) {
    let f = open(cx, "", 2);
    pick_up(cx, &f);
    on_reader(cx, &f, |r, _, cx| {
        let annots = r.notes.annots.as_mut().unwrap();
        annots.status = Status::Restorable;
        cx.notify();
    });
    drag(cx, &f, (0.2, 0.5), (0.7, 0.55));
    assert!(
        on_reader(cx, &f, |r, _, _| r.test_ink(1))
            .strokes
            .is_empty()
    );
    assert!(on_reader(cx, &f, |r, _, _| r.notes.notice.is_some()));
}

/// Escape puts the pencil down before it closes the reader.
#[gpui_kit::test]
fn escape_puts_the_pencil_down(cx: &mut TestAppContext) {
    let f = open(cx, "", 2);
    pick_up(cx, &f);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.simulate_keystrokes("escape");
    draw(cx, &f);
    assert!(!on_reader(cx, &f, |r, _, _| r.test_pencil_on()));
}

// ---- properties of the pipeline, without a window -----------------------------------

/// The tests here run the same cases on every machine.
fn runner(cases: u32) -> TestRunner {
    TestRunner::new(Config {
        cases,
        rng_seed: RngSeed::Fixed(0x6b61_6e73_6f70_656e),
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..Config::default()
    })
}

#[derive(Clone, Debug)]
enum Step {
    Draw(Vec<(u16, u16, u8)>, usize),
    Cut(u16, u16, bool),
    Undo,
    Redo,
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        (
            proptest::collection::vec((0u16..1000, 0u16..1400, 1u8..100), 2..14),
            0usize..4
        )
            .prop_map(|(p, tool)| Step::Draw(p, tool)),
        (0u16..1000, 0u16..1400, any::<bool>()).prop_map(|(x, y, whole)| Step::Cut(x, y, whole)),
        Just(Step::Undo),
        Just(Step::Redo),
    ]
}

fn sample(x: f64, y: f64, pressure: f64) -> PenSample {
    PenSample {
        x,
        y,
        pressure,
        tilt: None,
        source: Source::Pen,
    }
}

/// Runs steps the way the reader does: the canvas works, and what it holds
/// goes to the annotations after each step.
fn play(steps: &[Step]) -> (CanvasState, Annotations) {
    let mut state = CanvasState::new(PageInk::new(1).to_canvas());
    let mut annots = Annotations::new("file:abc123", "m");
    let tools = [Tool::Pen, Tool::Pencil, Tool::Marker, Tool::Highlighter];
    let mut now = 1_000;
    for step in steps {
        now += 1;
        let outcome = match step {
            Step::Draw(points, tool) => {
                state.tool = tools[*tool];
                let first = points[0];
                let mut out = state.begin(
                    sample(
                        f64::from(first.0),
                        f64::from(first.1),
                        f64::from(first.2) / 100.0,
                    ),
                    10.0,
                    now,
                );
                for p in &points[1..] {
                    out = state.extend(
                        sample(f64::from(p.0), f64::from(p.1), f64::from(p.2) / 100.0),
                        10.0,
                        now,
                    );
                }
                let end = state.end(now);
                if end == Outcome::Nothing { out } else { end }
            }
            Step::Cut(x, y, whole) => {
                state.tool = if *whole { Tool::Eraser } else { Tool::Partial };
                state.begin(sample(f64::from(*x), f64::from(*y), 0.5), 30.0, now);
                state.end(now)
            }
            Step::Undo => state.undo(now),
            Step::Redo => state.redo(now),
        };
        if outcome == Outcome::Changed {
            let next = annots.page_ink(1).from_canvas(&state.canvas);
            annots.set_page_ink(next).unwrap();
        }
    }
    (state, annots)
}

#[test]
fn what_is_drawn_is_what_is_saved_and_read_back() {
    runner(96)
        .run(&proptest::collection::vec(step(), 1..14), |steps| {
            let (state, annots) = play(&steps);
            let drawn = annots.page_ink(1).from_canvas(&state.canvas);
            // The annotations hold what the canvas holds.
            let held = annots.page_ink(1);
            prop_assert_eq!(held.live_count(), drawn.live_count());
            // The note reads back to the same ink, and every point is on the page.
            if let Some(edit) = annots.save_edit("N").unwrap() {
                let body = edit.apply("N");
                let back = Annotations::from_body(&body, "file:abc123", "m");
                prop_assert_eq!(back.ink(), annots.ink());
            }
            for page in annots.ink() {
                for s in page.live() {
                    prop_assert!(s.points.iter().all(|p| (0.0..=PAGE_UNITS).contains(&p.x)));
                }
            }
            Ok(())
        })
        .unwrap();
}

#[test]
fn undoing_every_step_leaves_a_blank_page() {
    runner(96)
        .run(&proptest::collection::vec(step(), 1..12), |steps| {
            let (mut state, mut annots) = play(&steps);
            let mut now = 9_000;
            while state.can_undo() {
                now += 1;
                if state.undo(now) == Outcome::Changed {
                    let next = annots.page_ink(1).from_canvas(&state.canvas);
                    annots.set_page_ink(next).unwrap();
                }
            }
            prop_assert!(annots.page_ink(1).is_blank());
            Ok(())
        })
        .unwrap();
}

#[test]
fn two_devices_drawing_apart_keep_every_stroke() {
    runner(64)
        .run(
            &(
                proptest::collection::vec(step(), 1..8),
                proptest::collection::vec(step(), 1..8),
            ),
            |(a, b)| {
                let (_, mine) = play(&a);
                let (_, theirs) = play(&b);
                let merged = kanso_pdf::ink::merge_pages(mine.ink(), theirs.ink());
                let again = kanso_pdf::ink::merge_pages(theirs.ink(), mine.ink());
                prop_assert_eq!(&merged, &again);
                for side in [mine.ink(), theirs.ink()] {
                    for page in side {
                        let m = merged.iter().find(|p| p.page == page.page);
                        prop_assert!(m.is_some());
                        for s in &page.strokes {
                            prop_assert!(m.unwrap().strokes.iter().any(|x| x.id == s.id));
                        }
                    }
                }
                Ok(())
            },
        )
        .unwrap();
}
