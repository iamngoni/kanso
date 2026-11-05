//! `CanvasDoc` against replicas that edit a canvas the way the view does (with
//! the canvas's own stamped operations), sync, and exchange updates in any
//! order. Property tests run on the seed `PROPTEST_RNG_SEED` pins.

use std::collections::HashSet;
use std::sync::Arc;

use kanso_core::canvas::{Canvas, Edge, INK, Node, Point, Stroke, StrokeRef};
use proptest::prelude::*;
use yrs::{Any, Doc, Map as _, MapPrelim, ReadTxn, Transact};

use crate::canvas_doc::{CanvasDoc, Erased, MAX_STREAM_POINTS, StrokeHead};
use crate::envelope::MAX_UPDATE;

const PART: usize = 64 * 1024;

// ------------------------------------------------------------------ replicas

/// What a person does to the canvas on screen.
#[derive(Clone, Debug)]
enum Op {
    Card { x: i32, y: i32, text: String },
    Move { pick: usize, x: i32, y: i32 },
    Retype { pick: usize, text: String },
    Delete { pick: usize },
    Draw { points: Vec<(i32, i32)> },
    Erase { pick: usize },
    Cut { x: i32, y: i32, x2: i32, y2: i32 },
    Arrow { a: usize, b: usize },
    Unarrow { pick: usize },
    Title { text: String },
}

/// A person's device: the CRDT, the canvas the view held when it last
/// synced (`base`) and the one it holds now (`view`).
struct Replica {
    doc: CanvasDoc,
    base: Canvas,
    view: Canvas,
    tag: usize,
    n: usize,
    now: i64,
    /// Updates this replica wrote that the others haven't been handed.
    outbox: Vec<Vec<u8>>,
    inbox: Vec<Vec<u8>>,
    /// IDs this person deleted.
    deleted: HashSet<String>,
}

impl Replica {
    fn new(tag: usize, doc: CanvasDoc) -> Replica {
        let base = doc.view();
        Replica {
            view: base.clone(),
            base,
            doc,
            tag,
            n: 0,
            now: 1000,
            outbox: Vec::new(),
            inbox: Vec::new(),
            deleted: HashSet::new(),
        }
    }

    fn id(&mut self, prefix: &str) -> String {
        self.n += 1;
        format!("{prefix}{}-{}", self.tag, self.n)
    }

    fn live_cards(&self) -> Vec<String> {
        self.view
            .nodes
            .iter()
            .filter(|n| !n.deleted && n.kind != INK)
            .map(|n| n.id.clone())
            .collect()
    }

    fn live_strokes(&self) -> Vec<StrokeRef> {
        self.view
            .live_strokes()
            .map(|(node, s)| StrokeRef {
                node: node.id.clone(),
                stroke: s.id.clone(),
            })
            .collect()
    }

    fn op(&mut self, op: &Op) {
        self.now += 1;
        let now = self.now;
        match op {
            Op::Card { x, y, text } => {
                let id = self.id("c");
                self.view.put_card(
                    Node {
                        id,
                        kind: "text".into(),
                        x: f64::from(*x),
                        y: f64::from(*y),
                        width: 120.0,
                        height: 60.0,
                        text: Some(text.clone()),
                        ..Node::default()
                    },
                    now,
                );
            }
            Op::Move { pick, x, y } => {
                let cards = self.live_cards();
                if let Some(id) = cards.get(pick % cards.len().max(1)) {
                    self.view.move_card(id, f64::from(*x), f64::from(*y), now);
                }
            }
            Op::Retype { pick, text } => {
                let cards = self.live_cards();
                if let Some(id) = cards.get(pick % cards.len().max(1))
                    && let Some(mut card) = self.view.node(id).cloned()
                    && card.text.is_some()
                {
                    card.text = Some(text.clone());
                    self.view.put_card(card, now);
                }
            }
            Op::Delete { pick } => {
                let cards = self.live_cards();
                if let Some(id) = cards.get(pick % cards.len().max(1))
                    && self.view.delete_card(id, now).is_some()
                {
                    self.deleted.insert(id.clone());
                }
            }
            Op::Draw { points } => {
                let stroke = Stroke {
                    tool: "pen".into(),
                    color: "#1e1e1e".into(),
                    width: 2.0,
                    points: points
                        .iter()
                        .map(|&(x, y)| Point::new(f64::from(x), f64::from(y), 0.5))
                        .collect(),
                    ..Stroke::default()
                };
                let _ = self.view.draw(stroke, now);
            }
            Op::Erase { pick } => {
                let strokes = self.live_strokes();
                if let Some(r) = strokes.get(pick % strokes.len().max(1))
                    && self
                        .view
                        .delete_strokes(std::slice::from_ref(r), now)
                        .is_some()
                {
                    self.deleted.insert(r.stroke.clone());
                }
            }
            Op::Cut { x, y, x2, y2 } => {
                let before: HashSet<String> = self
                    .view
                    .live_strokes()
                    .map(|(_, s)| s.id.clone())
                    .collect();
                let path = [
                    (f64::from(*x), f64::from(*y)),
                    (f64::from(*x2), f64::from(*y2)),
                ];
                if self.view.erase_along(&path, 40.0, now).is_some() {
                    let after: HashSet<String> = self
                        .view
                        .live_strokes()
                        .map(|(_, s)| s.id.clone())
                        .collect();
                    self.deleted.extend(before.difference(&after).cloned());
                }
            }
            Op::Arrow { a, b } => {
                let cards = self.live_cards();
                if cards.len() >= 2 {
                    let from = cards[a % cards.len()].clone();
                    let to = cards[b % cards.len()].clone();
                    let id = self.id("e");
                    self.view.put_edge(
                        Edge {
                            id,
                            from_node: from,
                            to_node: to,
                            ..Edge::default()
                        },
                        now,
                    );
                }
            }
            Op::Unarrow { pick } => {
                let edges: Vec<String> = self
                    .view
                    .edges
                    .iter()
                    .filter(|e| !e.deleted)
                    .map(|e| e.id.clone())
                    .collect();
                if let Some(id) = edges.get(pick % edges.len().max(1))
                    && self.view.remove_edge(id, now)
                {
                    self.deleted.insert(id.clone());
                }
            }
            Op::Title { text } => {
                if let Some(update) = self.doc.set_title(text) {
                    self.outbox.push(update);
                }
            }
        }
    }

    /// The view's sync: what was done is written to the CRDT, and the view
    /// takes the CRDT as it is, remote edits included.
    fn sync(&mut self) {
        for update in self.doc.edit(&self.base, &self.view, PART) {
            assert!(update.len() <= MAX_UPDATE, "{} bytes", update.len());
            self.outbox.push(update);
        }
        self.base = self.doc.view();
        self.view = self.base.clone();
        self.check();
    }

    /// Takes the first `n` updates handed to this replica, in the order the
    /// log gave them (an author's own updates in order, and what an author
    /// had seen before the author wrote it).
    fn receive(&mut self, n: usize) {
        let n = n.min(self.inbox.len());
        for update in self.inbox.drain(..n) {
            self.doc.apply(&update).unwrap();
        }
        self.check();
    }

    /// The projection is always a canvas the view opens.
    fn check(&self) {
        let canvas = self.doc.to_canvas();
        assert_eq!(canvas.validate(), Ok(()), "{canvas:?}");
        let view = self.doc.view();
        assert_eq!(view.validate(), Ok(()), "{view:?}");
        assert!(canvas.nodes.iter().all(|n| !n.deleted));
        assert!(canvas.edges.iter().all(|e| !e.deleted));
    }

    fn live_ids(&self) -> HashSet<String> {
        let canvas = self.doc.to_canvas();
        canvas
            .nodes
            .iter()
            .map(|n| n.id.clone())
            .chain(canvas.edges.iter().map(|e| e.id.clone()))
            .chain(canvas.live_strokes().map(|(_, s)| s.id.clone()))
            .collect()
    }
}

/// Hands each replica's new updates to the others' inboxes.
fn hand_out(replicas: &mut [Replica]) {
    for from in 0..replicas.len() {
        let out = std::mem::take(&mut replicas[from].outbox);
        for (to, replica) in replicas.iter_mut().enumerate() {
            if to != from {
                replica.inbox.extend(out.iter().cloned());
            }
        }
    }
}

// ---------------------------------------------------------------- strategies

fn op() -> impl Strategy<Value = Op> {
    let coord = -300i32..300;
    prop_oneof![
        3 => (coord.clone(), coord.clone(), "[a-z ]{0,12}")
            .prop_map(|(x, y, text)| Op::Card { x, y, text }),
        2 => (any::<usize>(), coord.clone(), coord.clone())
            .prop_map(|(pick, x, y)| Op::Move { pick, x, y }),
        2 => (any::<usize>(), "[a-z ]{0,16}").prop_map(|(pick, text)| Op::Retype { pick, text }),
        1 => any::<usize>().prop_map(|pick| Op::Delete { pick }),
        4 => prop::collection::vec((coord.clone(), coord.clone()), 1..8)
            .prop_map(|points| Op::Draw { points }),
        1 => any::<usize>().prop_map(|pick| Op::Erase { pick }),
        1 => (coord.clone(), coord.clone(), coord.clone(), coord.clone())
            .prop_map(|(x, y, x2, y2)| Op::Cut { x, y, x2, y2 }),
        1 => (any::<usize>(), any::<usize>()).prop_map(|(a, b)| Op::Arrow { a, b }),
        1 => any::<usize>().prop_map(|pick| Op::Unarrow { pick }),
        1 => "[a-z ]{0,10}".prop_map(|text| Op::Title { text }),
    ]
}

/// One step of a session: a replica does something, syncs its view, or
/// takes what was handed to it, in an order the strategy picked.
#[derive(Clone, Debug)]
enum Step {
    Do(usize, Op),
    Sync(usize),
    Take(usize, usize),
}

fn step(replicas: usize) -> impl Strategy<Value = Step> {
    prop_oneof![
        6 => (0..replicas, op()).prop_map(|(r, op)| Step::Do(r, op)),
        3 => (0..replicas).prop_map(Step::Sync),
        2 => (0..replicas, 0usize..6).prop_map(|(r, n)| Step::Take(r, n)),
    ]
}

fn run(replicas: &mut [Replica], steps: &[Step]) {
    for step in steps {
        match step {
            Step::Do(r, op) => replicas[*r].op(op),
            Step::Sync(r) => {
                replicas[*r].sync();
                hand_out(replicas);
            }
            Step::Take(r, n) => replicas[*r].receive(*n),
        }
    }
}

/// Everyone syncs and takes everything, until nothing is left to send.
fn settle(replicas: &mut [Replica]) {
    for _ in 0..4 {
        for replica in replicas.iter_mut() {
            replica.sync();
        }
        hand_out(replicas);
        for replica in replicas.iter_mut() {
            replica.receive(usize::MAX);
        }
    }
}

fn session(steps: &[Step]) -> Vec<Replica> {
    // One canvas shared from the start, as it is once people have joined.
    let seed = CanvasDoc::from_canvas("", &Canvas::default());
    let state = seed.encode_state();
    let mut replicas: Vec<Replica> = (0..3)
        .map(|tag| Replica::new(tag, CanvasDoc::load(&state).unwrap()))
        .collect();
    run(&mut replicas, steps);
    settle(&mut replicas);
    replicas
}

proptest! {
    #![proptest_config(crate::tests::config(96))]

    /// Concurrent edits, handed over in any order, converge: every replica
    /// ends with the same canvas, with tombstones and without, and the same
    /// title; the projection is valid at every step (`Replica::check`).
    #[test]
    fn concurrent_edits_converge(steps in prop::collection::vec(step(3), 1..60)) {
        let replicas = session(&steps);
        let first = &replicas[0];
        for other in &replicas[1..] {
            prop_assert_eq!(other.doc.to_canvas(), first.doc.to_canvas());
            prop_assert_eq!(other.doc.view(), first.doc.view());
            prop_assert_eq!(other.doc.title(), first.doc.title());
        }
        // The same canvas a replica that took the whole state in one go gets.
        let loaded = CanvasDoc::load(&first.doc.encode_state()).unwrap();
        prop_assert_eq!(loaded.to_canvas(), first.doc.to_canvas());
    }

    /// A deleted item never comes back, whoever edited it meanwhile and
    /// however the updates arrived.
    #[test]
    fn tombstones_never_resurrect(steps in prop::collection::vec(step(3), 1..60)) {
        let replicas = session(&steps);
        let deleted: HashSet<String> = replicas
            .iter()
            .flat_map(|r| r.deleted.iter().cloned())
            .collect();
        let live = replicas[0].live_ids();
        for id in &deleted {
            prop_assert!(!live.contains(id), "{id} came back");
        }
    }

    /// Trimming tombstones changes nothing anyone sees, here or elsewhere,
    /// and the document still converges and reloads the same.
    #[test]
    fn compaction_preserves_the_projection(steps in prop::collection::vec(step(3), 1..60)) {
        let replicas = session(&steps);
        let (a, b) = (&replicas[0], &replicas[1]);
        let (canvas, view) = (a.doc.to_canvas(), a.doc.view());
        let trim = a.doc.trim_tombstones();
        prop_assert_eq!(a.doc.to_canvas(), canvas.clone());
        prop_assert_eq!(a.doc.view(), view.clone());
        if let Some(update) = trim {
            b.doc.apply(&update).unwrap();
        }
        prop_assert_eq!(b.doc.to_canvas(), canvas.clone());
        prop_assert_eq!(b.doc.view(), view.clone());
        prop_assert_eq!(a.doc.trim_tombstones(), None);
        let loaded = CanvasDoc::load(&a.doc.encode_state()).unwrap();
        prop_assert_eq!(loaded.to_canvas(), canvas);
        prop_assert_eq!(loaded.view(), view);
    }

    /// A canvas seeded into a document comes back as it was.
    #[test]
    fn seeding_round_trips(ops in prop::collection::vec(op(), 1..40)) {
        let mut local = Replica::new(0, CanvasDoc::new());
        for op in &ops {
            local.op(op);
        }
        let canvas = local.view.clone();
        let doc = CanvasDoc::from_canvas("Plan", &canvas);
        prop_assert_eq!(doc.title(), "Plan");
        let mut expected = canvas.clone().sanitized();
        expected.refit_ink();
        prop_assert_eq!(doc.view(), expected);
        // Without the tombstones, the canvas the library stores.
        let mut live = canvas;
        live.nodes.retain(|n| !n.deleted);
        for n in &mut live.nodes {
            n.strokes.retain(|s| !s.deleted);
        }
        live.edges.retain(|e| !e.deleted);
        let mut live = live.sanitized();
        live.refit_ink();
        prop_assert_eq!(doc.to_canvas(), live);
        // And an edit that changes nothing writes nothing.
        let view = doc.view();
        prop_assert!(doc.edit(&view, &view, PART).is_empty());
    }

    /// Whatever is written into the document, even by something that isn't
    /// Kanso, the projection is a canvas the view opens.
    #[test]
    fn hostile_writes_leave_a_valid_canvas(
        entries in prop::collection::vec(hostile(), 1..24),
        steps in prop::collection::vec(step(2), 0..20),
    ) {
        let mut replicas = session(&steps);
        replicas.truncate(2);
        let raw = Doc::new();
        for (maps, id, fields) in entries {
            let map = raw.get_or_insert_map(maps);
            let mut txn = raw.transact_mut();
            let item = map.insert(&mut txn, id, MapPrelim::default());
            for (key, value) in fields {
                item.insert(&mut txn, key, value);
            }
        }
        let update = raw
            .transact()
            .encode_state_as_update_v1(&yrs::StateVector::default());
        replicas[0].doc.apply(&update).unwrap();
        replicas[0].check();
        // It is the same canvas on a device that gets it second hand.
        replicas[1].doc.apply(&update).unwrap();
        for replica in &replicas {
            replica.check();
        }
        prop_assert_eq!(replicas[0].doc.to_canvas(), replicas[1].doc.to_canvas());
    }
}

type Hostile = (&'static str, &'static str, Vec<(&'static str, Any)>);

fn hostile() -> impl Strategy<Value = Hostile> {
    let any = prop_oneof![
        Just(Any::Null),
        Just(Any::Bool(true)),
        Just(Any::Bool(false)),
        (-1000.0f64..1000.0).prop_map(Any::from),
        Just(Any::from(f64::NAN)),
        Just(Any::from(f64::INFINITY)),
        Just(Any::from(-1.0f64)),
        "[a-z]{0,6}".prop_map(|s| Any::from(s.as_str())),
        Just(Any::from("pen")),
        Just(Any::from("text")),
        Just(Any::from("ink")),
        Just(Any::from("c0-1")),
        Just(Any::Array(Arc::from(vec![
            Any::from(1.0f64),
            Any::from(2.0f64)
        ]))),
    ];
    let key = prop::sample::select(vec![
        "type",
        "x",
        "y",
        "width",
        "height",
        "deleted",
        "tool",
        "color",
        "ink",
        "done",
        "points",
        "text",
        "fromNode",
        "toNode",
        "fromSide",
        "toEnd",
        "kansoZ",
        "kansoVersion",
        "label",
        "strokes",
        "id",
        "owner",
    ]);
    let maps = prop::sample::select(vec!["nodes", "edges", "strokes"]);
    let id = prop::sample::select(vec!["c0-1", "c1-1", "ink-1", "s1", "s2", "e1", ""]);
    (maps, id, prop::collection::vec((key, any), 0..8))
}

// ------------------------------------------------------------------ examples

fn text_card(id: &str, text: &str) -> Node {
    Node {
        id: id.into(),
        kind: "text".into(),
        width: 100.0,
        height: 50.0,
        text: Some(text.into()),
        ..Node::default()
    }
}

fn shared_pair() -> (Replica, Replica) {
    let seed = CanvasDoc::from_canvas("", &Canvas::default());
    let state = seed.encode_state();
    (
        Replica::new(0, CanvasDoc::load(&state).unwrap()),
        Replica::new(1, CanvasDoc::load(&state).unwrap()),
    )
}

fn exchange(a: &mut Replica, b: &mut Replica) {
    a.sync();
    b.sync();
    b.inbox.append(&mut a.outbox);
    a.inbox.append(&mut b.outbox);
    a.receive(usize::MAX);
    b.receive(usize::MAX);
    a.sync();
    b.sync();
}

#[test]
fn two_people_typing_in_one_card_both_keep_their_words() {
    let (mut a, mut b) = shared_pair();
    a.view.put_card(text_card("c", "the quick fox"), 1);
    exchange(&mut a, &mut b);
    assert_eq!(
        b.doc.to_canvas().node("c").unwrap().text.as_deref(),
        Some("the quick fox")
    );
    // At once: one puts a word at the start, the other at the end.
    let mut card = a.view.node("c").cloned().unwrap();
    card.text = Some("a the quick fox".into());
    a.view.put_card(card, 2);
    let mut card = b.view.node("c").cloned().unwrap();
    card.text = Some("the quick fox jumps".into());
    b.view.put_card(card, 2);
    exchange(&mut a, &mut b);
    exchange(&mut a, &mut b);
    let text = a.doc.to_canvas().node("c").unwrap().text.clone().unwrap();
    assert_eq!(text, "a the quick fox jumps");
    assert_eq!(b.doc.to_canvas(), a.doc.to_canvas());
}

#[test]
fn a_move_and_a_recolour_of_one_card_both_win() {
    let (mut a, mut b) = shared_pair();
    a.view.put_card(text_card("c", "hi"), 1);
    exchange(&mut a, &mut b);
    a.view.move_card("c", 40.0, 50.0, 2);
    let mut card = b.view.node("c").cloned().unwrap();
    card.color = Some("4".into());
    b.view.put_card(card, 2);
    exchange(&mut a, &mut b);
    exchange(&mut a, &mut b);
    let card = a.doc.to_canvas().node("c").cloned().unwrap();
    assert_eq!((card.x, card.y), (40.0, 50.0));
    assert_eq!(card.color.as_deref(), Some("4"));
}

#[test]
fn a_card_deleted_while_another_moves_it_stays_deleted() {
    let (mut a, mut b) = shared_pair();
    a.view.put_card(text_card("c", "hi"), 1);
    exchange(&mut a, &mut b);
    a.view.delete_card("c", 2);
    b.view.move_card("c", 9.0, 9.0, 2);
    exchange(&mut a, &mut b);
    exchange(&mut a, &mut b);
    assert!(a.doc.to_canvas().node("c").is_none());
    assert!(b.doc.to_canvas().node("c").is_none());
    // An arrow to it is hidden with it.
    assert!(a.doc.to_canvas().edges.is_empty());
}

#[test]
fn a_late_device_with_an_old_copy_cannot_bring_an_erased_stroke_back() {
    let (mut a, mut b) = shared_pair();
    let stroke = Stroke {
        tool: "pen".into(),
        width: 2.0,
        points: vec![Point::new(0.0, 0.0, 0.5), Point::new(30.0, 30.0, 0.5)],
        ..Stroke::default()
    };
    let (id, _) = a.view.draw(stroke, 1).unwrap();
    exchange(&mut a, &mut b);
    // B goes offline holding the stroke; A erases it and the document is
    // compacted.
    let old_copy = b.doc.encode_state();
    let node = a
        .view
        .nodes
        .iter()
        .find(|n| n.kind == INK)
        .unwrap()
        .id
        .clone();
    a.view
        .delete_strokes(
            &[StrokeRef {
                node,
                stroke: id.clone(),
            }],
            2,
        )
        .unwrap();
    a.sync();
    if let Some(update) = a.doc.trim_tombstones() {
        a.outbox.push(update);
    }
    let latest = a.doc.encode_state();
    // The old copy arrives, then the new one, or the reverse.
    let late = CanvasDoc::load(&old_copy).unwrap();
    late.apply(&latest).unwrap();
    let early = CanvasDoc::load(&latest).unwrap();
    early.apply(&old_copy).unwrap();
    for doc in [&late, &early] {
        assert!(doc.to_canvas().live_strokes().all(|(_, s)| s.id != id));
        assert_eq!(doc.to_canvas(), a.doc.to_canvas());
    }
}

#[test]
fn a_partial_erase_arrives_as_the_old_stroke_gone_and_its_pieces_there() {
    let (mut a, mut b) = shared_pair();
    let points: Vec<Point> = (0..40)
        .map(|i| Point::new(f64::from(i) * 5.0, 0.0, 0.5))
        .collect();
    let (id, _) = a
        .view
        .draw(
            Stroke {
                tool: "pen".into(),
                width: 2.0,
                points,
                ..Stroke::default()
            },
            1,
        )
        .unwrap();
    exchange(&mut a, &mut b);
    assert_eq!(b.doc.to_canvas().live_strokes().count(), 1);
    // The eraser cuts the middle out, on B.
    b.view
        .erase_along(&[(100.0, 0.0), (110.0, 0.0)], 8.0, 2)
        .unwrap();
    // One edit, one update.
    let updates = b.doc.edit(&b.base, &b.view, PART);
    assert_eq!(updates.len(), 1);
    for update in &updates {
        a.doc.apply(update).unwrap();
    }
    let canvas = a.doc.to_canvas();
    assert!(canvas.live_strokes().all(|(_, s)| s.id != id));
    assert!(canvas.live_strokes().count() >= 2);
    assert_eq!(canvas.validate(), Ok(()));
}

#[test]
fn a_stroke_streams_as_it_is_drawn_and_an_erase_stops_it() {
    let (mut a, mut b) = shared_pair();
    // A has an ink layer.
    a.view.ink_layer(1);
    exchange(&mut a, &mut b);
    let ink = a
        .view
        .nodes
        .iter()
        .find(|n| n.kind == INK)
        .unwrap()
        .id
        .clone();
    let head = StrokeHead {
        id: "s-live".into(),
        ink: ink.clone(),
        owner: "a1".into(),
        tool: "pen".into(),
        color: "#000000".into(),
        width: 2.0,
        z: 5.0,
    };
    let mut points: Vec<Point> = Vec::new();
    for chunk in 0..3 {
        points.extend((0..32).map(|i| Point::new(f64::from(chunk * 32 + i), 0.0, 0.5)));
        let streamed = a.doc.stream(&head, &points).unwrap();
        let update = streamed.update.expect("new points");
        assert!(update.len() < 4096);
        b.doc.apply(&update).unwrap();
        // B sees the ink as it grows, before the stroke ends.
        let seen = b.doc.to_canvas();
        let stroke = seen
            .live_strokes()
            .next()
            .expect("the stroke is there")
            .1
            .clone();
        assert_eq!(stroke.points.len(), points.len());
        assert_eq!(seen.validate(), Ok(()));
    }
    // Nothing new, nothing sent.
    assert_eq!(a.doc.stream(&head, &points).unwrap().update, None);
    // B erases it while A is still drawing.
    b.base = b.doc.view();
    b.view = b.base.clone();
    b.view
        .delete_strokes(
            &[StrokeRef {
                node: ink,
                stroke: "s-live".into(),
            }],
            9,
        )
        .unwrap();
    for update in b.doc.edit(&b.base, &b.view, PART) {
        a.doc.apply(&update).unwrap();
    }
    points.push(Point::new(500.0, 0.0, 0.5));
    assert_eq!(a.doc.stream(&head, &points), Err(Erased));
    assert_eq!(a.doc.to_canvas().live_strokes().count(), 0);
}

#[test]
fn a_streamed_stroke_ends_with_the_points_the_person_drew() {
    let (mut a, mut b) = shared_pair();
    a.view.ink_layer(1);
    exchange(&mut a, &mut b);
    let ink = a
        .view
        .nodes
        .iter()
        .find(|n| n.kind == INK)
        .unwrap()
        .id
        .clone();
    let head = StrokeHead {
        id: "s-live".into(),
        ink,
        owner: "a1".into(),
        tool: "pen".into(),
        color: "#000000".into(),
        width: 2.0,
        z: 5.0,
    };
    let all: Vec<Point> = (0..100)
        .map(|i| Point::new(f64::from(i), 1.0, 0.5))
        .collect();
    // The first part streams; the pen lifts and the view draws the stroke.
    let update = a.doc.stream(&head, &all[..60]).unwrap().update.unwrap();
    b.doc.apply(&update).unwrap();
    a.view
        .draw_as(
            Stroke {
                id: "s-live".into(),
                tool: "pen".into(),
                color: "#000000".into(),
                width: 2.0,
                points: all.clone(),
                ..Stroke::default()
            },
            2,
        )
        .unwrap();
    exchange(&mut a, &mut b);
    exchange(&mut a, &mut b);
    let strokes: Vec<_> = b
        .doc
        .to_canvas()
        .live_strokes()
        .map(|(_, s)| s.clone())
        .collect();
    assert_eq!(strokes.len(), 1);
    assert_eq!(strokes[0].id, "s-live");
    assert_eq!(strokes[0].points.len(), 100);
    assert_eq!(a.doc.to_canvas(), b.doc.to_canvas());
}

#[test]
fn only_the_first_stream_points_go_while_the_pen_is_down() {
    let doc = CanvasDoc::new();
    let head = StrokeHead {
        id: "s".into(),
        ink: "ink".into(),
        owner: "a".into(),
        tool: "pen".into(),
        color: "#000".into(),
        width: 1.0,
        z: 1.0,
    };
    let points: Vec<Point> = (0..MAX_STREAM_POINTS + 50)
        .map(|i| Point::new(i as f64, 0.0, 0.5))
        .collect();
    let streamed = doc.stream(&head, &points).unwrap();
    assert!(streamed.full);
    assert!(streamed.update.unwrap().len() < MAX_UPDATE);
}

#[test]
fn a_long_stroke_and_many_cards_go_as_updates_under_the_cap() {
    let (mut a, b) = shared_pair();
    let points: Vec<Point> = (0..20_000)
        .map(|i| Point::new(f64::from(i % 997), f64::from(i / 7 % 811), 0.5))
        .collect();
    a.view
        .draw(
            Stroke {
                tool: "pen".into(),
                width: 2.0,
                points,
                ..Stroke::default()
            },
            1,
        )
        .ok();
    for i in 0..400 {
        a.view
            .put_card(text_card(&format!("card-{i}"), &"words ".repeat(40)), 2);
    }
    let updates = a.doc.edit(&a.base, &a.view, PART);
    assert!(updates.len() > 3, "{}", updates.len());
    for update in &updates {
        assert!(update.len() <= MAX_UPDATE, "{}", update.len());
        b.doc.apply(update).unwrap();
    }
    assert_eq!(b.doc.to_canvas().nodes.len(), a.view.nodes.len());
    assert_eq!(b.doc.to_canvas().validate(), Ok(()));
}

#[test]
fn trimming_drops_what_a_deleted_item_held() {
    let (mut a, mut b) = shared_pair();
    a.view
        .put_card(text_card("c", &"long words ".repeat(200)), 1);
    exchange(&mut a, &mut b);
    // A peer that doesn't trim on delete: write the tombstone by hand.
    {
        let doc = &a.doc;
        let nodes = doc.doc_for_test().get_or_insert_map("nodes");
        let mut txn = doc.doc_for_test().transact_mut();
        let yrs::Out::YMap(card) = nodes.get(&txn, "c").unwrap() else {
            panic!("not a map");
        };
        card.insert(&mut txn, "deleted", true);
    }
    let before = a.doc.encode_state().len();
    assert_eq!(a.doc.tombstones(), 1);
    let update = a.doc.trim_tombstones().expect("something to trim");
    assert!(a.doc.encode_state().len() < before / 2);
    b.doc.apply(&a.doc.encode_state()).unwrap();
    b.doc.apply(&update).unwrap();
    assert_eq!(a.doc.to_canvas(), b.doc.to_canvas());
    assert_eq!(a.doc.tombstones(), 1, "the tombstone itself stays");
}

#[test]
fn a_person_can_undo_an_erase_of_their_own() {
    let (mut a, mut b) = shared_pair();
    let (id, action) = a
        .view
        .draw(
            Stroke {
                tool: "pen".into(),
                width: 2.0,
                points: vec![Point::new(0.0, 0.0, 0.5), Point::new(10.0, 10.0, 0.5)],
                ..Stroke::default()
            },
            1,
        )
        .unwrap();
    exchange(&mut a, &mut b);
    let node = a
        .view
        .nodes
        .iter()
        .find(|n| n.kind == INK)
        .unwrap()
        .id
        .clone();
    let erase = a
        .view
        .delete_strokes(
            &[StrokeRef {
                node,
                stroke: id.clone(),
            }],
            2,
        )
        .unwrap();
    exchange(&mut a, &mut b);
    // The tombstone was trimmed away by then; undo writes the stroke whole.
    a.doc.trim_tombstones();
    a.view.apply(&erase, false, 3);
    exchange(&mut a, &mut b);
    exchange(&mut a, &mut b);
    let strokes: Vec<_> = b
        .doc
        .to_canvas()
        .live_strokes()
        .map(|(_, s)| s.clone())
        .collect();
    assert_eq!(strokes.len(), 1);
    assert_eq!(strokes[0].id, id);
    assert_eq!(strokes[0].points.len(), 2);
    let _ = action;
}

proptest! {
    #![proptest_config(crate::tests::config(96))]

    /// Histories written apart merge to the same canvas in any interleaving
    /// that keeps each author's own updates in order (the log does), and
    /// when some updates arrive twice.
    #[test]
    fn independent_histories_merge_in_any_interleaving(
        a_ops in prop::collection::vec(op(), 1..24),
        b_ops in prop::collection::vec(op(), 1..24),
        picks in prop::collection::vec(any::<bool>(), 0..60),
        repeats in 0usize..4,
    ) {
        let seed = CanvasDoc::from_canvas("", &Canvas::default()).encode_state();
        let mut written: Vec<Vec<Vec<u8>>> = Vec::new();
        for (tag, ops) in [(0, &a_ops), (1, &b_ops)] {
            let mut replica = Replica::new(tag, CanvasDoc::load(&seed).unwrap());
            for chunk in ops.chunks(4) {
                for op in chunk {
                    replica.op(op);
                }
                replica.sync();
            }
            written.push(std::mem::take(&mut replica.outbox));
        }
        let apply_all = |doc: &CanvasDoc, updates: &[&Vec<u8>]| {
            for update in updates {
                doc.apply(update).unwrap();
            }
        };
        // One after the other.
        let one = CanvasDoc::load(&seed).unwrap();
        apply_all(&one, &written[0].iter().chain(&written[1]).collect::<Vec<_>>());
        // Interleaved as the picks say, keeping each author's order.
        let two = CanvasDoc::load(&seed).unwrap();
        let (mut next_a, mut next_b) = (0, 0);
        let mut order: Vec<&Vec<u8>> = Vec::new();
        for pick in picks {
            if pick && next_a < written[0].len() {
                order.push(&written[0][next_a]);
                next_a += 1;
            } else if next_b < written[1].len() {
                order.push(&written[1][next_b]);
                next_b += 1;
            }
        }
        order.extend(&written[0][next_a..]);
        order.extend(&written[1][next_b..]);
        // Some arrive again.
        let again: Vec<&Vec<u8>> = order.iter().take(repeats).copied().collect();
        apply_all(&two, &order);
        apply_all(&two, &again);
        prop_assert_eq!(one.to_canvas(), two.to_canvas());
        prop_assert_eq!(one.view(), two.view());
        prop_assert_eq!(one.title(), two.title());
        prop_assert_eq!(one.to_canvas().validate(), Ok(()));
    }
}

// ------------------------------------------------- the local API's batches

mod api_batches {
    use kanso_core::canvas::{Batch, BatchContext, apply_batch};
    use serde_json::{Value, json};

    use super::*;

    fn known(id: &str) -> bool {
        id == "note:known"
    }

    fn none(_: &str) -> bool {
        false
    }

    /// What `Engine::edit_canvas` does for the local API: applies the batch
    /// to the canvas as the CRDT has it, and writes the difference as updates.
    fn api_edit(replica: &mut Replica, ops: &Value, now: i64) -> bool {
        let batch = Batch::from_json(&json!({ "ops": ops })).unwrap();
        let base = replica.doc.view();
        let ctx = BatchContext {
            now,
            shared: true,
            note_exists: &known,
            attachment_ok: &none,
        };
        let Ok((next, _)) = apply_batch(&base, &batch, &ctx) else {
            return false;
        };
        for update in replica.doc.edit(&base, &next, PART) {
            replica.outbox.push(update);
        }
        // The replica's own view takes the new state, as a view's next sync does.
        replica.base = replica.doc.view();
        replica.view = replica.base.clone();
        replica.check();
        true
    }

    fn seeded() -> (Replica, Replica) {
        let mut canvas = Canvas::default();
        canvas.put_node(text_card("t1", "one"), 1);
        canvas.put_node(text_card("t2", "two"), 1);
        let ink = canvas.ink_layer(1);
        canvas.add_stroke(
            &ink,
            Stroke {
                id: "s1".into(),
                tool: "pen".into(),
                color: "#000000".into(),
                width: 2.0,
                points: vec![Point::new(1.0, 1.0, 0.5), Point::new(9.0, 9.0, 0.5)],
                ..Stroke::default()
            },
            1,
        );
        let state = CanvasDoc::from_canvas("", &canvas).encode_state();
        (
            Replica::new(0, CanvasDoc::load(&state).unwrap()),
            Replica::new(1, CanvasDoc::load(&state).unwrap()),
        )
    }

    #[test]
    fn an_agents_batch_and_a_persons_edit_both_arrive() {
        let (mut agent, mut person) = seeded();
        assert!(api_edit(
            &mut agent,
            &json!([
                { "op": "addNode", "type": "text", "x": 5, "y": 5, "text": "from the agent", "id": "a1" },
                { "op": "moveNode", "id": "t1", "x": 90, "y": 90 },
            ]),
            10
        ));
        // Meanwhile the person recolours the card the agent moved.
        let mut card = person.view.node("t1").cloned().unwrap();
        card.color = Some("4".into());
        person.view.put_card(card, 11);
        exchange(&mut agent, &mut person);
        exchange(&mut agent, &mut person);
        for replica in [&agent, &person] {
            let canvas = replica.doc.to_canvas();
            assert!(canvas.node("a1").is_some());
            let card = canvas.node("t1").unwrap();
            assert_eq!((card.x, card.y), (90.0, 90.0));
            assert_eq!(card.color.as_deref(), Some("4"));
        }
        assert_eq!(agent.doc.to_canvas(), person.doc.to_canvas());
    }

    #[test]
    fn two_agents_removing_and_moving_one_card_leave_it_removed() {
        let (mut a, mut b) = seeded();
        assert!(api_edit(
            &mut a,
            &json!([{ "op": "removeNode", "id": "t2" }]),
            10
        ));
        assert!(api_edit(
            &mut b,
            &json!([{ "op": "moveNode", "id": "t2", "x": 1, "y": 1 }]),
            11
        ));
        exchange(&mut a, &mut b);
        exchange(&mut a, &mut b);
        assert!(a.doc.to_canvas().node("t2").is_none());
        assert_eq!(a.doc.to_canvas(), b.doc.to_canvas());
    }

    fn api_ops() -> impl Strategy<Value = Value> {
        let id = prop::sample::select(vec!["t1", "t2", "a1", "a2", "ghost"]);
        let n = prop_oneof![0.0f64..800.0, Just(1e13)];
        prop_oneof![
            (id.clone(), n.clone(), n.clone()).prop_map(|(i, x, y)| json!({ "op": "addNode", "type": "text", "x": x, "y": y, "text": "q", "id": i })),
            (id.clone(), n.clone(), n.clone()).prop_map(|(i, x, y)| json!({ "op": "moveNode", "id": i, "x": x, "y": y })),
            id.clone().prop_map(|i| json!({ "op": "removeNode", "id": i })),
            id.clone().prop_map(|i| json!({ "op": "updateNode", "id": i, "text": "t", "color": "3" })),
            (id.clone(), id.clone()).prop_map(|(a, b)| json!({ "op": "addEdge", "fromNode": a, "toNode": b })),
            n.clone().prop_map(|p| json!({ "op": "addStrokes", "node": "ink-1", "strokes": [{ "points": [[p, p, 0.5], [p, 1.0, 0.5]] }] })),
            Just(json!({ "op": "removeStrokes", "node": "ink-1", "strokes": ["s1"] })),
            Just(json!({ "op": "moveStrokes", "node": "ink-1", "strokes": ["s1"], "dx": 3, "dy": 4 })),
        ]
    }

    proptest! {
        #![proptest_config(crate::tests::config(96))]

        /// Batches from two API clients and edits from a person, in any
        /// order, converge, and the projection stays a canvas the view opens
        /// (`Replica::check` at every step).
        #[test]
        fn api_batches_and_people_converge(
            first in prop::collection::vec(prop::collection::vec(api_ops(), 1..5), 0..4),
            second in prop::collection::vec(prop::collection::vec(api_ops(), 1..5), 0..4),
            person in prop::collection::vec(op().prop_map(|op| Step::Do(0, op)), 0..12),
        ) {
            let (mut a, mut b) = seeded();
            let mut now = 10;
            for ops in &first {
                now += 1;
                api_edit(&mut a, &Value::Array(ops.clone()), now);
            }
            for ops in &second {
                now += 1;
                api_edit(&mut b, &Value::Array(ops.clone()), now);
            }
            let mut replicas = vec![a, b];
            run_person(&mut replicas[1], &person);
            settle(&mut replicas);
            prop_assert_eq!(replicas[0].doc.to_canvas(), replicas[1].doc.to_canvas());
            prop_assert_eq!(replicas[0].doc.view(), replicas[1].doc.view());
            prop_assert_eq!(replicas[0].doc.to_canvas().validate(), Ok(()));
        }
    }

    /// A person's edits on one replica. Only edits: syncing is `settle`'s.
    fn run_person(replica: &mut Replica, steps: &[Step]) {
        run(std::slice::from_mut(replica), steps);
    }
}
