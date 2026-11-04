use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

use super::*;

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

fn pen(points: &[(f64, f64)]) -> Stroke {
    Stroke {
        tool: "pen".into(),
        color: "#1e1e1e".into(),
        width: 2.0,
        points: points.iter().map(|&(x, y)| Point::new(x, y, 0.5)).collect(),
        ..Stroke::default()
    }
}

fn live(c: &Canvas) -> Vec<(String, Vec<(f64, f64)>)> {
    let mut v: Vec<_> = c
        .live_strokes()
        .map(|(_, s)| (s.id.clone(), s.points.iter().map(|p| (p.x, p.y)).collect()))
        .collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

fn line(y: f64) -> Vec<(f64, f64)> {
    (0..20).map(|i| (i as f64 * 5.0, y)).collect()
}

#[test]
fn drawing_makes_an_ink_layer_and_a_stroke() {
    let mut c = Canvas::default();
    let (id, action) = c.draw(pen(&line(10.0)), 100).unwrap();
    assert_eq!(c.nodes.len(), 1);
    assert_eq!(c.nodes[0].kind, "ink");
    assert_eq!(live(&c).len(), 1);
    assert_eq!(action.added.len(), 1);
    assert_eq!(action.added[0].1.id, id);
    assert!(c.validate().is_ok());
    // A second stroke reuses the layer.
    c.draw(pen(&line(30.0)), 101).unwrap();
    assert_eq!(c.nodes.len(), 1);
    assert_eq!(live(&c).len(), 2);
}

#[test]
fn drawing_refuses_nothing_and_a_full_canvas() {
    let mut c = Canvas::default();
    assert_eq!(c.draw(pen(&[]), 1).unwrap_err(), Refused::Empty);
    assert_eq!(
        c.draw(pen(&[(f64::NAN, 0.0)]), 1).unwrap_err(),
        Refused::Empty
    );
    c.draw(pen(&line(0.0)), 1).unwrap();
    let node = c.nodes[0].id.clone();
    c.node_mut(&node)
        .unwrap()
        .extra
        .insert("pad".into(), "x".repeat(MAX_BYTES).into());
    assert_eq!(c.draw(pen(&line(5.0)), 2).unwrap_err(), Refused::Full);
}

#[test]
fn a_node_origin_is_taken_out_of_new_points() {
    let mut c = Canvas::default();
    let id = c.ink_layer(1);
    c.node_mut(&id).unwrap().x = 100.0;
    c.node_mut(&id).unwrap().y = 50.0;
    c.draw(pen(&[(110.0, 60.0), (130.0, 60.0)]), 2).unwrap();
    let (_, s) = c.live_strokes().next().unwrap();
    assert_eq!((s.points[0].x, s.points[0].y), (10.0, 10.0));
}

#[test]
fn thinning_keeps_ends_and_drops_crowding() {
    let pts: Vec<Point> = (0..100)
        .map(|i| Point::new(i as f64 * 0.1, 0.0, 0.5))
        .collect();
    let out = thin(&pts);
    assert!(out.len() < 40);
    assert_eq!(out.first().unwrap().x, 0.0);
    assert!((out.last().unwrap().x - 9.9).abs() < 1e-9);
}

#[test]
fn stroke_eraser_takes_whole_strokes_and_undo_brings_them_back() {
    let mut c = Canvas::default();
    c.draw(pen(&line(0.0)), 1).unwrap();
    c.draw(pen(&line(100.0)), 2).unwrap();
    let before = live(&c);
    let action = c.erase_strokes_at(40.0, 1.0, 3.0, 3).unwrap();
    assert_eq!(live(&c).len(), 1);
    assert!(c.erase_strokes_at(500.0, 500.0, 3.0, 4).is_none());
    assert!(c.apply(&action, false, 5));
    assert_eq!(live(&c), before);
    assert!(c.validate().is_ok());
    // The undo is a newer version, never an older document.
    let stroke = c.nodes[0]
        .strokes
        .iter()
        .find(|s| s.id == action.removed[0].1.id)
        .unwrap();
    assert!(stroke.kanso_version > action.removed[0].1.kanso_version);
    assert_eq!(stroke.kanso_updated, 5);
    assert!(c.apply(&action, true, 6));
    assert_eq!(live(&c).len(), 1);
}

#[test]
fn partial_eraser_splits_a_stroke_and_undo_joins_it() {
    let mut c = Canvas::default();
    c.draw(pen(&line(0.0)), 1).unwrap();
    let action = c.erase_along(&[(50.0, -5.0), (50.0, 5.0)], 4.0, 2).unwrap();
    assert_eq!(action.removed.len(), 1);
    assert_eq!(action.added.len(), 2);
    let now = live(&c);
    assert_eq!(now.len(), 2);
    assert!(c.validate().is_ok());
    assert!(c.apply(&action, false, 3));
    let back = live(&c);
    assert_eq!(back.len(), 1);
    assert_eq!(back[0].1.len(), 20);
}

#[test]
fn the_eraser_misses_a_distant_stroke() {
    let mut c = Canvas::default();
    c.draw(pen(&line(0.0)), 1).unwrap();
    assert!(
        c.erase_along(&[(0.0, 100.0), (90.0, 100.0)], 5.0, 2)
            .is_none()
    );
    assert_eq!(live(&c).len(), 1);
}

#[test]
fn lasso_selects_and_moves() {
    let mut c = Canvas::default();
    c.draw(pen(&line(0.0)), 1).unwrap();
    c.draw(pen(&line(200.0)), 2).unwrap();
    let hit = c.strokes_in(&[(-10.0, -10.0), (120.0, -10.0), (120.0, 20.0), (-10.0, 20.0)]);
    assert_eq!(hit.len(), 1);
    let action = c.move_strokes(&hit, 0.0, 50.0, 3).unwrap();
    let moved = live(&c);
    assert!(moved.iter().any(|(_, p)| p[0].1 == 50.0));
    assert!(moved.iter().any(|(_, p)| p[0].1 == 200.0));
    assert!(c.apply(&action, false, 4));
    assert!(live(&c).iter().any(|(_, p)| p[0].1 == 0.0));
    assert!(c.validate().is_ok());
}

#[test]
fn a_selection_can_be_deleted() {
    let mut c = Canvas::default();
    c.draw(pen(&line(0.0)), 1).unwrap();
    let hit = c.strokes_in(&[(-10.0, -10.0), (120.0, -10.0), (120.0, 20.0), (-10.0, 20.0)]);
    assert!(c.delete_strokes(&hit, 2).is_some());
    assert!(live(&c).is_empty());
    assert!(c.delete_strokes(&hit, 3).is_none());
}

#[test]
fn polygon_membership() {
    let sq = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
    assert!(in_polygon((5.0, 5.0), &sq));
    assert!(!in_polygon((15.0, 5.0), &sq));
    assert!(!in_polygon((5.0, 5.0), &[]));
}

#[derive(Clone, Debug)]
enum Step {
    Draw(f64, f64),
    Erase(f64, f64),
    Partial(f64, f64, f64, f64),
    Undo,
    Redo,
}

fn arb_step() -> impl Strategy<Value = Step> {
    prop_oneof![
        (0f64..200., 0f64..200.).prop_map(|(x, y)| Step::Draw(x, y)),
        (0f64..200., 0f64..200.).prop_map(|(x, y)| Step::Erase(x, y)),
        (0f64..200., 0f64..200., 0f64..200., 0f64..200.)
            .prop_map(|(a, b, c, d)| Step::Partial(a, b, c, d)),
        Just(Step::Undo),
        Just(Step::Redo),
    ]
}

proptest! {
    #![proptest_config(config(64))]

    /// Invariants: whatever mix of drawing, both erasers, undo and redo, the
    /// canvas stays valid, and undoing every action in turn brings back an
    /// empty canvas.
    #[test]
    fn prop_edits_stay_valid_and_undo_fully(steps in prop::collection::vec(arb_step(), 1..25)) {
        let mut c = Canvas::default();
        let mut done: Vec<Action> = Vec::new();
        let mut undone: Vec<Action> = Vec::new();
        let mut now = 1;
        for step in steps {
            now += 1;
            match step {
                Step::Draw(x, y) => {
                    if let Ok((_, a)) = c.draw(pen(&[(x, y), (x + 30.0, y + 5.0), (x + 60.0, y)]), now) {
                        done.push(a);
                        undone.clear();
                    }
                }
                Step::Erase(x, y) => {
                    if let Some(a) = c.erase_strokes_at(x, y, 6.0, now) {
                        done.push(a);
                        undone.clear();
                    }
                }
                Step::Partial(a, b, x, y) => {
                    if let Some(act) = c.erase_along(&[(a, b), (x, y)], 6.0, now) {
                        done.push(act);
                        undone.clear();
                    }
                }
                Step::Undo => {
                    if let Some(a) = done.pop() {
                        c.apply(&a, false, now);
                        undone.push(a);
                    }
                }
                Step::Redo => {
                    if let Some(a) = undone.pop() {
                        c.apply(&a, true, now);
                        done.push(a);
                    }
                }
            }
            prop_assert!(c.validate().is_ok(), "{:?}", c.validate());
        }
        while let Some(a) = done.pop() {
            now += 1;
            c.apply(&a, false, now);
        }
        prop_assert!(live(&c).is_empty());
        prop_assert!(c.validate().is_ok());
    }

    /// Invariant: an eraser pass never leaves a point inside its disc (the
    /// cut lands on the edge, so allow rounding slack).
    #[test]
    fn prop_partial_eraser_clears_the_disc(cx in 5f64..90., r in 1f64..8.) {
        let mut c = Canvas::default();
        c.draw(pen(&line(0.0)), 1).unwrap();
        c.erase_along(&[(cx, 0.0)], r, 2);
        for (_, s) in c.live_strokes() {
            for p in &s.points {
                prop_assert!((p.x - cx).hypot(p.y) >= r - 0.02);
            }
        }
    }

    /// Invariant: thinning keeps the first point, never grows the stroke, and
    /// leaves no two neighbours closer than the minimum except at the end.
    #[test]
    fn prop_thin_is_a_subsequence(raw in prop::collection::vec((-50f64..50., -50f64..50.), 0..80)) {
        let pts: Vec<Point> = raw.iter().map(|&(x, y)| Point::new(x, y, 0.5)).collect();
        let out = thin(&pts);
        prop_assert!(out.len() <= pts.len());
        if let Some(first) = pts.first() {
            prop_assert_eq!(out.first().map(|p| (p.x, p.y)), Some((first.x, first.y)));
        }
        if out.len() > 2 {
            for w in out[..out.len() - 1].windows(2) {
                prop_assert!((w[1].x - w[0].x).hypot(w[1].y - w[0].y) >= MIN_SAMPLE_DISTANCE - 1e-9);
            }
        }
    }
}

proptest! {
    #![proptest_config(config(64))]

    /// Invariant: a drag of the partial eraser, many small edits joined with
    /// `then`, undoes in one step back to the live strokes it started with.
    #[test]
    fn prop_a_joined_drag_undoes_in_one_step(
        steps in prop::collection::vec((0f64..100., -6f64..6.), 1..12),
    ) {
        let mut c = Canvas::default();
        c.draw(pen(&line(0.0)), 1).unwrap();
        c.draw(pen(&line(4.0)), 2).unwrap();
        let before = live(&c);
        let mut joined = Action::default();
        let mut last = (0.0, 0.0);
        for (i, (x, y)) in steps.into_iter().enumerate() {
            if let Some(a) = c.erase_along(&[last, (x, y)], 3.0, 10 + i as i64) {
                joined = joined.then(a);
            }
            last = (x, y);
        }
        if !joined.is_empty() {
            prop_assert!(c.apply(&joined, false, 100));
        }
        prop_assert_eq!(live(&c), before);
        prop_assert!(c.validate().is_ok());
    }
}
