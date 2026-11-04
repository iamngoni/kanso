//! Drawing through the state: a stroke from pointer samples, both erasers,
//! the lasso, undo and redo, and that whatever the gestures do the document
//! stays valid.

use std::sync::Arc;

use kanso_core::canvas::{self, Budget, Canvas, Refused};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

use crate::canvas_state::{COLORS, CanvasState, Outcome, PenSample, Source, Tool, WIDTHS};

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

fn state() -> CanvasState {
    CanvasState::new(Canvas::default())
}

fn drag(s: &mut CanvasState, from: (f64, f64), to: (f64, f64), steps: usize, now: i64) -> Outcome {
    s.begin(PenSample::mouse(from.0, from.1), 6.0, now);
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        s.extend(
            PenSample::mouse(from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t),
            6.0,
            now,
        );
    }
    s.end(now)
}

fn live_count(s: &CanvasState) -> usize {
    s.canvas.live_strokes().count()
}

#[test]
fn dragging_with_the_pen_draws_a_stroke() {
    let mut s = state();
    assert_eq!(drag(&mut s, (0., 0.), (100., 30.), 20, 1), Outcome::Changed);
    assert_eq!(live_count(&s), 1);
    let (_, stroke) = s.canvas.live_strokes().next().unwrap();
    assert_eq!(stroke.tool, "pen");
    assert_eq!(stroke.color, COLORS[0]);
    assert!(stroke.points.len() > 10);
    assert!(s.can_undo() && !s.can_redo());
}

#[test]
fn a_click_without_a_drag_draws_a_dot() {
    let mut s = state();
    s.begin(PenSample::mouse(5., 5.), 6.0, 1);
    assert_eq!(s.end(2), Outcome::Changed);
    assert_eq!(live_count(&s), 1);
}

#[test]
fn the_pen_colour_width_and_tool_are_per_stroke() {
    let mut s = state();
    drag(&mut s, (0., 0.), (50., 0.), 10, 1);
    s.tool = Tool::Highlighter;
    s.color = COLORS[3].into();
    s.width = WIDTHS[3];
    drag(&mut s, (0., 20.), (50., 20.), 10, 2);
    let strokes: Vec<_> = s.canvas.live_strokes().map(|(_, s)| s.clone()).collect();
    assert_eq!(strokes.len(), 2);
    assert_ne!(strokes[0].color, strokes[1].color);
    assert_eq!(strokes[1].tool, "highlighter");
    assert_eq!(strokes[1].width, WIDTHS[3]);
    // Changing the pen did not repaint the earlier stroke.
    assert_eq!(strokes[0].tool, "pen");
}

#[test]
fn the_live_stroke_shows_and_predicted_points_are_never_kept() {
    let mut s = state();
    s.begin(PenSample::mouse(0., 0.), 6.0, 1);
    for i in 1..10 {
        s.extend(PenSample::mouse(i as f64 * 5., 0.), 6.0, 1);
    }
    let (live, polygon) = s.live_outline(&[PenSample::mouse(100., 0.)]).unwrap();
    assert!(!polygon.is_empty());
    assert_eq!(live.points.len(), 11);
    s.end(2);
    let (_, stroke) = s.canvas.live_strokes().next().unwrap();
    assert!(stroke.points.iter().all(|p| p.x < 60.));
    assert!(s.live_outline(&[]).is_none());
}

#[test]
fn the_stroke_eraser_takes_whole_strokes_in_one_undo_step() {
    let mut s = state();
    drag(&mut s, (0., 0.), (100., 0.), 20, 1);
    drag(&mut s, (0., 50.), (100., 50.), 20, 2);
    s.tool = Tool::Eraser;
    // A pass over both strokes.
    drag(&mut s, (50., -10.), (50., 60.), 14, 3);
    assert_eq!(live_count(&s), 0);
    assert_eq!(s.undo(4), Outcome::Changed);
    assert_eq!(live_count(&s), 2);
    assert_eq!(s.redo(5), Outcome::Changed);
    assert_eq!(live_count(&s), 0);
}

#[test]
fn the_partial_eraser_cuts_a_stroke_in_two() {
    let mut s = state();
    drag(&mut s, (0., 0.), (100., 0.), 40, 1);
    s.tool = Tool::Partial;
    assert_eq!(drag(&mut s, (50., -8.), (50., 8.), 4, 2), Outcome::Changed);
    assert_eq!(live_count(&s), 2);
    for (_, stroke) in s.canvas.live_strokes() {
        assert!(stroke.points.iter().all(|p| (p.x - 50.).abs() >= 5.9));
    }
    // One undo step puts the stroke back whole.
    s.undo(3);
    assert_eq!(live_count(&s), 1);
}

#[test]
fn the_pens_eraser_end_rubs_out_whatever_tool_is_chosen() {
    let mut s = state();
    drag(&mut s, (0., 0.), (100., 0.), 40, 1);
    assert_eq!(s.tool, Tool::Pen);
    let flip = |x: f64, y: f64| PenSample {
        source: Source::PenEraser,
        ..PenSample::mouse(x, y)
    };
    s.begin(flip(50., -8.), 6.0, 2);
    s.extend(flip(50., 8.), 6.0, 2);
    assert_eq!(s.end(2), Outcome::Changed);
    assert_eq!(live_count(&s), 2);
}

#[test]
fn the_lasso_selects_moves_and_deletes() {
    let mut s = state();
    drag(&mut s, (10., 10.), (60., 10.), 10, 1);
    drag(&mut s, (300., 300.), (350., 300.), 10, 2);
    s.tool = Tool::Lasso;
    // A loop round the first stroke.
    s.begin(PenSample::mouse(0., 0.), 6.0, 3);
    for p in [(80., 0.), (80., 30.), (0., 30.)] {
        s.extend(PenSample::mouse(p.0, p.1), 6.0, 3);
    }
    s.end(3);
    assert_eq!(s.selection.len(), 1);
    assert!(s.selection_bounds().is_some());

    // Dragging inside the selection moves it.
    s.begin(PenSample::mouse(30., 10.), 6.0, 4);
    s.extend(PenSample::mouse(30., 110.), 6.0, 4);
    assert_eq!(s.end(4), Outcome::Changed);
    assert!(s.canvas.live_strokes().any(|(_, st)| st.points[0].y > 100.));
    assert_eq!(s.selection.len(), 1);
    s.undo(5);
    assert!(s.canvas.live_strokes().any(|(_, st)| st.points[0].y == 10.));
    assert!(
        s.canvas
            .live_strokes()
            .all(|(_, st)| st.points[0].y != 110.)
    );

    s.selection = s
        .canvas
        .strokes_in(&[(0., 0.), (80., 0.), (80., 30.), (0., 30.)]);
    assert_eq!(s.delete_selection(6), Outcome::Changed);
    assert_eq!(live_count(&s), 1);
    s.undo(7);
    assert_eq!(live_count(&s), 2);
}

#[test]
fn text_cards_are_added_edited_moved_and_deleted() {
    let mut s = state();
    let id = s.add_text_card(40.3, 50.6, "Launch plan", 1);
    assert!(s.set_card_text(&id, "Launch plan v2", 2));
    assert!(s.move_card(&id, 100., 120., 3));
    let card = s.card_at(150., 150.).unwrap();
    assert_eq!(card.text.as_deref(), Some("Launch plan v2"));
    assert_eq!((card.x, card.y), (100., 120.));
    assert!(s.card_at(0., 0.).is_none());
    assert!(!s.set_card_text("nope", "x", 4));
    assert!(s.delete_card(&id, 5));
    assert!(s.card_at(150., 150.).is_none());
    assert!(s.canvas.validate().is_ok());
}

#[test]
fn a_full_canvas_refuses_new_ink() {
    let mut s = state();
    drag(&mut s, (0., 0.), (10., 0.), 4, 1);
    let layer = s.canvas.ink_layer(1);
    s.canvas
        .node_mut(&layer)
        .unwrap()
        .extra
        .insert("pad".into(), "x".repeat(canvas::MAX_BYTES).into());
    assert_eq!(s.budget(), Budget::Full);
    assert_eq!(
        drag(&mut s, (0., 20.), (10., 20.), 4, 2),
        Outcome::Refused(Refused::Full)
    );
}

#[test]
fn the_body_round_trips_and_a_broken_one_is_refused() {
    let mut s = state();
    drag(&mut s, (0., 0.), (40., 0.), 8, 1);
    let again = CanvasState::from_body(&s.body()).unwrap();
    assert_eq!(again.canvas, s.canvas);
    assert!(CanvasState::from_body("not a canvas").is_err());
}

#[test]
fn fit_shows_everything() {
    let mut s = state();
    drag(&mut s, (0., 0.), (1000., 600.), 30, 1);
    s.fit([800., 600.]);
    let (lo, hi) = s.view.visible([800., 600.], 0.);
    assert!(lo[0] <= 0. && hi[0] >= 1000. && lo[1] <= 0. && hi[1] >= 600.);
}

#[test]
fn outlines_are_kept_per_stroke_version() {
    let mut s = state();
    drag(&mut s, (0., 0.), (40., 0.), 8, 1);
    let stroke = s.canvas.live_strokes().next().unwrap().1.clone();
    let a = s.outline_of(&stroke);
    let b = s.outline_of(&stroke);
    assert!(Arc::ptr_eq(&a, &b));
}

#[derive(Clone, Debug)]
enum Gesture {
    Draw(f64, f64, f64, f64),
    Erase(f64, f64, f64, f64),
    Partial(f64, f64, f64, f64),
    Lasso(f64, f64),
    Card(f64, f64),
    Undo,
    Redo,
    Delete,
}

fn arb_gesture() -> impl Strategy<Value = Gesture> {
    let p = || (0f64..300., 0f64..300., 0f64..300., 0f64..300.);
    prop_oneof![
        3 => p().prop_map(|(a, b, c, d)| Gesture::Draw(a, b, c, d)),
        1 => p().prop_map(|(a, b, c, d)| Gesture::Erase(a, b, c, d)),
        2 => p().prop_map(|(a, b, c, d)| Gesture::Partial(a, b, c, d)),
        1 => (0f64..300., 0f64..300.).prop_map(|(a, b)| Gesture::Lasso(a, b)),
        1 => (0f64..300., 0f64..300.).prop_map(|(a, b)| Gesture::Card(a, b)),
        2 => Just(Gesture::Undo),
        1 => Just(Gesture::Redo),
        1 => Just(Gesture::Delete),
    ]
}

proptest! {
    #![proptest_config(config(48))]

    /// Invariants: whatever gestures follow one another, the document is
    /// always one the store would accept, and undoing every step leaves no
    /// live ink.
    #[test]
    fn prop_any_gestures_leave_a_valid_canvas(gestures in prop::collection::vec(arb_gesture(), 1..25)) {
        let mut s = state();
        let mut now = 1;
        for g in gestures {
            now += 1;
            match g {
                Gesture::Draw(a, b, c, d) => { s.tool = Tool::Pen; drag(&mut s, (a, b), (c, d), 6, now); }
                Gesture::Erase(a, b, c, d) => { s.tool = Tool::Eraser; drag(&mut s, (a, b), (c, d), 6, now); }
                Gesture::Partial(a, b, c, d) => { s.tool = Tool::Partial; drag(&mut s, (a, b), (c, d), 6, now); }
                Gesture::Lasso(a, b) => {
                    s.tool = Tool::Lasso;
                    s.begin(PenSample::mouse(a, b), 6.0, now);
                    for (x, y) in [(a + 120., b), (a + 120., b + 120.), (a, b + 120.)] {
                        s.extend(PenSample::mouse(x, y), 6.0, now);
                    }
                    s.end(now);
                }
                Gesture::Card(a, b) => { s.add_text_card(a, b, "card", now); }
                Gesture::Undo => { s.undo(now); }
                Gesture::Redo => { s.redo(now); }
                Gesture::Delete => { s.delete_selection(now); }
            }
            prop_assert!(s.canvas.validate().is_ok(), "{:?}", s.canvas.validate());
            prop_assert!(s.body().len() < canvas::MAX_BYTES);
        }
        while s.can_undo() {
            now += 1;
            s.undo(now);
        }
        prop_assert_eq!(s.canvas.live_strokes().count(), 0);
        prop_assert_eq!(s.canvas.live_nodes().filter(|n| n.kind != canvas::INK).count(), 0);
        prop_assert!(s.canvas.validate().is_ok());
    }
}

/// The cards as a person sees them: ID, kind, place and text, no stamps.
fn cards(s: &CanvasState) -> Vec<(String, String, i64, i64, Option<String>)> {
    let mut v: Vec<_> = s
        .canvas
        .live_nodes()
        .filter(|n| n.kind != canvas::INK)
        .map(|n| {
            (
                n.id.clone(),
                n.kind.clone(),
                n.x as i64,
                n.y as i64,
                n.text.clone(),
            )
        })
        .collect();
    v.sort();
    v
}

#[test]
fn card_edits_moves_and_deletes_undo_and_redo_as_single_steps() {
    let mut s = state();
    let id = s.add_text_card(40., 50., "Launch plan", 1);
    let added = cards(&s);
    assert!(s.set_card_text(&id, "Launch plan v2", 2));
    let edited = cards(&s);
    // A drag is many moves and one step.
    for i in 1..=10 {
        assert!(s.move_card(&id, 40. + f64::from(i) * 10., 50., 3));
    }
    s.end_card_drag();
    let moved = cards(&s);
    assert_eq!(moved[0].2, 140);
    assert!(s.delete_card(&id, 4));
    assert!(cards(&s).is_empty());

    assert_eq!(s.undo(5), Outcome::Changed);
    assert_eq!(cards(&s), moved);
    assert_eq!(s.undo(6), Outcome::Changed);
    assert_eq!(cards(&s), edited);
    assert_eq!(s.undo(7), Outcome::Changed);
    assert_eq!(cards(&s), added);
    assert_eq!(s.undo(8), Outcome::Changed);
    assert!(cards(&s).is_empty());
    assert_eq!(s.undo(9), Outcome::Nothing);
    for expect in [&added, &edited, &moved] {
        assert_eq!(s.redo(10), Outcome::Changed);
        assert_eq!(&cards(&s), expect);
    }
    assert_eq!(s.redo(11), Outcome::Changed);
    assert!(cards(&s).is_empty());
    assert!(s.canvas.validate().is_ok());
}

#[test]
fn a_note_card_add_and_a_selected_card_delete_undo() {
    let mut s = state();
    let id = s.add_note_card(0., 0., "note:abc", "Abc", 1).unwrap();
    s.select_card(Some(id.clone()));
    assert_eq!(s.delete_selection(2), Outcome::Changed);
    assert!(s.card_at(10., 10.).is_none());
    s.undo(3);
    assert_eq!(s.card_at(10., 10.).unwrap().id, id);
    s.undo(4);
    assert!(s.card_at(10., 10.).is_none());
    assert!(s.add_note_card(0., 0., "other", "x", 5).is_none());
}

#[test]
fn an_undone_card_delete_keeps_its_arrows() {
    let mut s = state();
    let a = s.add_text_card(0., 0., "a", 1);
    let b = s.add_text_card(400., 0., "b", 1);
    s.canvas.put_edge(
        canvas::Edge {
            id: "e1".into(),
            from_node: a.clone(),
            to_node: b,
            ..canvas::Edge::default()
        },
        2,
    );
    assert!(s.delete_card(&a, 3));
    s.undo(4);
    assert!(s.canvas.node(&a).is_some());
    assert!(s.canvas.edges.iter().any(|e| e.id == "e1" && !e.deleted));
}

#[derive(Clone, Debug)]
enum CardStep {
    Add(i32, i32),
    Edit(usize, u8),
    Drag(usize, Vec<(i32, i32)>),
    Delete(usize),
}

fn arb_card_step() -> impl Strategy<Value = CardStep> {
    prop_oneof![
        2 => (0i32..500, 0i32..500).prop_map(|(x, y)| CardStep::Add(x, y)),
        2 => (0usize..8, any::<u8>()).prop_map(|(i, t)| CardStep::Edit(i, t)),
        3 => (0usize..8, prop::collection::vec((0i32..500, 0i32..500), 1..6))
            .prop_map(|(i, p)| CardStep::Drag(i, p)),
        2 => (0usize..8).prop_map(CardStep::Delete),
    ]
}

proptest! {
    #![proptest_config(config(64))]

    /// One user action is one undo step, whatever it was: undoing it gives
    /// back exactly the cards before it, redoing exactly the cards after,
    /// and undoing everything leaves none.
    #[test]
    fn prop_each_card_action_is_one_undo_step(steps in prop::collection::vec(arb_card_step(), 1..20)) {
        let mut s = state();
        let mut now = 1;
        let mut history = vec![cards(&s)];
        for step in steps {
            now += 1;
            let ids: Vec<String> = cards(&s).into_iter().map(|c| c.0).collect();
            let pick = |i: usize| ids.get(i % ids.len().max(1)).cloned();
            match step {
                CardStep::Add(x, y) => { s.add_text_card(f64::from(x), f64::from(y), "c", now); }
                CardStep::Edit(i, t) => {
                    if let Some(id) = pick(i) { s.set_card_text(&id, &format!("t{t}"), now); }
                }
                CardStep::Drag(i, path) => {
                    if let Some(id) = pick(i) {
                        for (x, y) in path { s.move_card(&id, f64::from(x), f64::from(y), now); }
                        s.end_card_drag();
                    }
                }
                CardStep::Delete(i) => {
                    if let Some(id) = pick(i) { s.delete_card(&id, now); }
                }
            }
            let after = cards(&s);
            if after != *history.last().unwrap() {
                history.push(after);
            }
            prop_assert!(s.canvas.validate().is_ok());
        }
        // Back through every state, then forward again.
        for want in history.iter().rev().skip(1) {
            now += 1;
            prop_assert_eq!(s.undo(now), Outcome::Changed);
            prop_assert_eq!(&cards(&s), want);
        }
        prop_assert!(cards(&s).is_empty());
        for want in history.iter().skip(1) {
            now += 1;
            prop_assert_eq!(s.redo(now), Outcome::Changed);
            prop_assert_eq!(&cards(&s), want);
        }
        prop_assert!(s.canvas.validate().is_ok());
    }
}
