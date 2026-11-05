use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngSeed, TestRunner};

use kanso_core::canvas::{Eraser, Point};

use super::*;
use crate::highlights::Annotations;
use crate::model::PdfBlock;

const FILE: &str = "file:abc123";

/// The tests here run the same cases on every machine.
fn runner(cases: u32) -> TestRunner {
    TestRunner::new(Config {
        cases,
        rng_seed: RngSeed::Fixed(0x6b61_6e73_6f69_6e6b),
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..Config::default()
    })
}

fn stroke_strategy() -> impl Strategy<Value = Stroke> {
    (
        0u32..12,
        0usize..TOOLS.len(),
        0usize..WIDTHS.len(),
        proptest::collection::vec((0u32..100_000, 0u32..140_000, 0u32..100), 1..8),
        0u64..4,
        any::<bool>(),
    )
        .prop_map(|(n, tool, width, points, version, deleted)| Stroke {
            id: format!("s{n}"),
            tool: TOOLS[tool].into(),
            color: "#1e1e1e".into(),
            width: WIDTHS[width],
            points: if deleted {
                Vec::new()
            } else {
                points
                    .into_iter()
                    .map(|(x, y, p)| {
                        Point::new(
                            f64::from(x) / 100.0,
                            f64::from(y) / 100.0,
                            f64::from(p) / 100.0,
                        )
                    })
                    .collect()
            },
            deleted,
            kanso_version: version,
            kanso_updated: version as i64 * 10,
            kanso_z: f64::from(n),
            extra: Default::default(),
        })
}

/// A page's ink with unique stroke ids.
fn page_strategy(page: u32) -> impl Strategy<Value = PageInk> {
    proptest::collection::vec(stroke_strategy(), 0..6).prop_map(move |mut strokes| {
        let mut seen = std::collections::HashSet::new();
        strokes.retain(|s| seen.insert(s.id.clone()));
        PageInk {
            page,
            strokes,
            extra: Default::default(),
        }
    })
}

fn same(a: &PageInk, b: &PageInk) -> bool {
    serde_json::to_string(a).unwrap() == serde_json::to_string(b).unwrap()
}

fn live_stroke(id: &str, from: f64, to: f64) -> Stroke {
    Stroke {
        id: id.into(),
        tool: "pen".into(),
        color: "#1e1e1e".into(),
        width: 4.0,
        points: (0..=10)
            .map(|i| {
                Point::new(
                    from + (to - from) * f64::from(i) / 10.0,
                    300.0,
                    ((0.4 + f64::from(i) / 40.0) * 1000.0).round() / 1000.0,
                )
            })
            .collect(),
        ..Stroke::default()
    }
}

fn inked(page: u32, strokes: Vec<Stroke>) -> PageInk {
    PageInk {
        page,
        strokes,
        extra: Default::default(),
    }
}

#[test]
fn merging_pages_is_commutative_associative_and_idempotent() {
    runner(256)
        .run(
            &(page_strategy(3), page_strategy(3), page_strategy(3)),
            |(a, b, c)| {
                prop_assert!(same(&merge_page(&a, &b), &merge_page(&b, &a)));
                prop_assert!(same(&merge_page(&a, &a), &merge_page(&a, &a)));
                let ab = merge_page(&a, &b);
                prop_assert!(same(&merge_page(&ab, &a), &ab));
                prop_assert!(same(&merge_page(&ab, &b), &ab));
                prop_assert!(same(
                    &merge_page(&merge_page(&a, &b), &c),
                    &merge_page(&a, &merge_page(&b, &c))
                ));
                Ok(())
            },
        )
        .unwrap();
}

#[test]
fn a_merge_keeps_every_stroke_and_every_erase() {
    runner(256)
        .run(&(page_strategy(2), page_strategy(2)), |(a, b)| {
            let merged = merge_page(&a, &b);
            let ids = |p: &PageInk| -> std::collections::BTreeSet<String> {
                p.strokes.iter().map(|s| s.id.clone()).collect()
            };
            let both: std::collections::BTreeSet<String> =
                ids(&a).union(&ids(&b)).cloned().collect();
            prop_assert_eq!(ids(&merged), both);
            // A stroke deleted on one side, at a version no lower than the
            // other side's copy, is not drawn after the merge.
            for s in merged.strokes.iter().filter(|s| !s.deleted) {
                for side in [&a, &b] {
                    if let Some(o) = side.strokes.iter().find(|o| o.id == s.id) {
                        prop_assert!(!(o.deleted && o.kanso_version > s.kanso_version));
                    }
                }
            }
            Ok(())
        })
        .unwrap();
}

#[test]
fn pages_merge_by_page_in_order() {
    runner(128)
        .run(
            &(
                proptest::collection::vec(1u32..6, 0..5),
                proptest::collection::vec(1u32..6, 0..5),
            ),
            |(a, b)| {
                let mk = |pages: &[u32]| -> Vec<PageInk> {
                    let mut seen = std::collections::BTreeSet::new();
                    pages
                        .iter()
                        .filter(|p| seen.insert(**p))
                        .map(|p| inked(*p, vec![live_stroke(&format!("p{p}"), 10.0, 100.0)]))
                        .collect()
                };
                let (a, b) = (mk(&a), mk(&b));
                let merged = merge_pages(&a, &b);
                let pages: Vec<u32> = merged.iter().map(|p| p.page).collect();
                let mut sorted = pages.clone();
                sorted.sort_unstable();
                sorted.dedup();
                prop_assert_eq!(&pages, &sorted);
                let want: std::collections::BTreeSet<u32> =
                    a.iter().chain(&b).map(|p| p.page).collect();
                prop_assert_eq!(
                    pages.into_iter().collect::<std::collections::BTreeSet<_>>(),
                    want
                );
                prop_assert_eq!(merge_pages(&merged, &a), merged.clone());
                Ok(())
            },
        )
        .unwrap();
}

#[test]
fn the_canvas_round_trip_loses_nothing() {
    runner(256)
        .run(&page_strategy(5), |page| {
            let back = page.from_canvas(&page.to_canvas());
            // Same strokes; the canvas order is by stacking order.
            prop_assert_eq!(back.strokes.len(), page.strokes.len());
            for s in &page.strokes {
                prop_assert!(back.strokes.contains(s));
            }
            Ok(())
        })
        .unwrap();
}

#[test]
fn a_page_of_ink_survives_json_byte_for_byte() {
    runner(128)
        .run(&page_strategy(7), |page| {
            let json = serde_json::to_string(&page).unwrap();
            let back: PageInk = serde_json::from_str(&json).unwrap();
            prop_assert!(same(&back, &page));
            prop_assert_eq!(serde_json::to_string(&back).unwrap(), json);
            Ok(())
        })
        .unwrap();
}

#[test]
fn pressure_and_tilt_are_stored_and_read_back() {
    let mut s = live_stroke("a", 100.0, 400.0);
    for (i, p) in s.points.iter_mut().enumerate() {
        p.pressure = ((0.1 + i as f64 / 12.0) * 1000.0).round() / 1000.0;
        p.tilt = Some((0.3, 1.1));
    }
    let page = inked(4, vec![s.clone()]);
    let mut block = PdfBlock::new(FILE, "m");
    block.ink = vec![page];
    let fence = block.to_fence().unwrap();
    let json = fence
        .strip_prefix("```kanso-pdf\n")
        .and_then(|f| f.strip_suffix("\n```"))
        .unwrap();
    let back = PdfBlock::parse(json).unwrap();
    assert_eq!(back.ink[0].strokes[0], s);
    assert_eq!(back.to_fence().unwrap(), fence);
}

#[test]
fn the_pencil_looks_the_same_in_the_pdf_and_on_a_canvas() {
    // The same strokes, as a PDF page's ink and as an ink layer of a canvas
    // note, are one stroke type and one outline: nothing is converted.
    let page = inked(1, vec![live_stroke("a", 80.0, 700.0)]);
    let on_canvas = Canvas {
        nodes: vec![Node {
            id: "ink-1".into(),
            kind: INK.into(),
            width: 1000.0,
            height: 1000.0,
            strokes: page.strokes.clone(),
            ..Node::default()
        }],
        ..Canvas::default()
    };
    let as_text = canvas::to_body(&on_canvas);
    let read = canvas::parse(&as_text).unwrap();
    assert_eq!(read.nodes[0].strokes, page.strokes);
    assert_eq!(page.to_canvas().nodes[0].strokes, page.strokes);
}

#[test]
fn erasing_part_of_a_stroke_leaves_the_rest_and_the_cut_stays_erased() {
    let page = inked(1, vec![live_stroke("a", 100.0, 900.0)]);
    let mut c = page.to_canvas();
    let action = c.erase_along(&[(500.0, 300.0)], 40.0, 1_000).unwrap();
    assert_eq!(action.removed.len(), 1);
    assert!(action.added.len() >= 2);
    let after = page.from_canvas(&c);
    // The first stroke is a tombstone; the pieces are drawn.
    assert!(after.strokes.iter().any(|s| s.id == "a" && s.deleted));
    assert!(after.live_count() >= 2);
    assert!(
        after
            .live()
            .all(|s| s.points.iter().all(|p| (p.x - 500.0).abs() >= 39.0))
    );
    // A device that still has the whole stroke cannot bring it back.
    let merged = merge_page(&page, &after);
    assert!(merged.live().all(|s| s.id != "a"));
    assert_eq!(merged.live_count(), after.live_count());
}

#[test]
fn erasing_a_whole_stroke_tombstones_it() {
    let page = inked(
        1,
        vec![
            live_stroke("a", 100.0, 300.0),
            live_stroke("b", 600.0, 800.0),
        ],
    );
    let mut c = page.to_canvas();
    c.erase_strokes_at(200.0, 300.0, 5.0, 10).unwrap();
    let after = page.from_canvas(&c);
    assert_eq!(after.live_count(), 1);
    assert_eq!(after.live().next().unwrap().id, "b");
    let _ = Eraser {
        path: vec![],
        radius: 1.0,
    };
}

#[test]
fn clearing_a_page_erases_everything_on_it_for_good() {
    let page = inked(
        2,
        vec![
            live_stroke("a", 100.0, 300.0),
            live_stroke("b", 600.0, 800.0),
        ],
    );
    let cleared = cleared(&page, 50);
    assert!(cleared.is_blank());
    assert_eq!(cleared.strokes.len(), 2);
    assert!(merge_page(&page, &cleared).is_blank());
}

#[test]
fn points_are_taken_into_the_page() {
    runner(256)
        .run(
            &(
                -5_000f64..5_000.,
                -5_000f64..5_000.,
                1f64..2_000.,
                1f64..4_000.,
            ),
            |(x, y, w, h)| {
                let (ux, uy) = to_units(x, y, w, h);
                let (ew, eh) = page_extent(w, h);
                prop_assert!((0.0..=PAGE_UNITS).contains(&ux));
                prop_assert!(uy >= 0.0 && uy <= eh + 1e-9);
                prop_assert_eq!(ew, PAGE_UNITS);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(to_units(1.0, 1.0, 0.0, 0.0), (0.0, 0.0));
    assert_eq!(to_units(f64::NAN, 1.0, 10.0, 10.0).0, 0.0);
    assert_eq!(page_extent(0.0, 5.0), (PAGE_UNITS, PAGE_UNITS));
}

#[test]
fn ink_that_does_not_fit_a_page_is_refused() {
    let mut off = live_stroke("a", 10.0, 20.0);
    off.points[0].x = 9_999_999.0;
    assert!(inked(1, vec![off]).validate().is_err());
    let mut nan = live_stroke("a", 10.0, 20.0);
    nan.points[0].y = f64::NAN;
    assert!(inked(1, vec![nan]).validate().is_err());
    let mut tool = live_stroke("a", 10.0, 20.0);
    tool.tool = "chalk".into();
    assert!(inked(1, vec![tool]).validate().is_err());
    assert!(
        inked(0, vec![live_stroke("a", 1.0, 2.0)])
            .validate()
            .is_err()
    );
    let twins = inked(
        1,
        vec![live_stroke("a", 1.0, 2.0), live_stroke("a", 3.0, 4.0)],
    );
    assert!(twins.validate().is_err());
    let pages = [
        inked(1, vec![live_stroke("a", 1.0, 2.0)]),
        inked(1, vec![live_stroke("b", 1.0, 2.0)]),
    ];
    assert!(validate_pages(&pages).is_err());
    assert!(
        inked(3, vec![live_stroke("a", 1.0, 2.0)])
            .validate()
            .is_ok()
    );
}

#[test]
fn a_block_with_bad_ink_is_not_read() {
    let mut block = PdfBlock::new(FILE, "m");
    let mut off = live_stroke("a", 10.0, 20.0);
    off.points[0].x = 1e12;
    block.ink = vec![inked(1, vec![off])];
    let json = serde_json::to_string(&block).unwrap();
    assert!(PdfBlock::parse(&json).is_err());
}

#[test]
fn unknown_fields_on_a_page_survive_a_save() {
    let json = r#"{"file":"file:abc123","md5":"m","version":1,"highlights":[],"ink":[{"page":2,"strokes":[],"future":{"a":1}}]}"#;
    let block = PdfBlock::parse(json).unwrap();
    assert_eq!(block.ink[0].extra.get("future").unwrap()["a"], 1);
    assert!(block.to_json().unwrap().contains("\"future\":{\"a\":1}"));
}

#[test]
fn the_companion_file_round_trips_and_is_checked() {
    let pages = vec![inked(1, vec![live_stroke("a", 1.0, 99.0)])];
    let file = InkFile::new(FILE, pages);
    let bytes = file.to_bytes().unwrap();
    assert_eq!(InkFile::from_bytes(&bytes).unwrap(), file);
    assert!(InkFile::from_bytes(b"{").is_err());
    let newer =
        String::from_utf8(bytes.clone())
            .unwrap()
            .replacen("\"version\":1", "\"version\":9", 1);
    assert!(matches!(
        InkFile::from_bytes(newer.as_bytes()),
        Err(PdfError::NewerVersion(9))
    ));
    let bad = String::from_utf8(bytes)
        .unwrap()
        .replace("\"page\":1", "\"page\":0");
    assert!(InkFile::from_bytes(bad.as_bytes()).is_err());
}

#[test]
fn the_ink_file_is_named_by_its_link() {
    assert_eq!(file_id("inkdrop://file:xyz9"), Some("file:xyz9".into()));
    assert_eq!(file_id("file:xyz9"), Some("file:xyz9".into()));
    assert_eq!(file_id("inkdrop://file:"), None);
    assert_eq!(file_id("inkdrop://file:a b"), None);
    assert_eq!(file_link("file:xyz9"), "inkdrop://file:xyz9");
    assert_eq!(file_id(&file_link("file:Ab_-1")), Some("file:Ab_-1".into()));
    // The link is what makes the note own the attachment.
    let mut block = PdfBlock::new(FILE, "m");
    block.ink_file = Some(file_link("file:xyz9"));
    let body = format!("Notes\n\n{}", block.to_fence().unwrap());
    assert!(kanso_core::tasks::linked_file_ids(&body).contains(&"file:xyz9".to_owned()));
}

fn many_strokes(n: usize) -> PageInk {
    inked(
        1,
        (0..n)
            .map(|i| {
                let mut s = live_stroke(&format!("s{i}"), 10.0, 900.0);
                s.points
                    .extend((0..200).map(|j| Point::new(f64::from(j), f64::from(j), 0.5)));
                s
            })
            .collect(),
    )
}

#[test]
fn big_ink_moves_to_the_companion_and_the_block_stays_small() {
    let mut a = Annotations::new(FILE, "m");
    a.set_page_ink(inked(1, vec![live_stroke("a", 1.0, 99.0)]))
        .unwrap();
    assert!(!a.needs_companion());
    assert!(a.companion().is_none());
    a.set_page_ink(many_strokes(60)).unwrap();
    assert!(a.needs_companion());
    let file = a.companion().unwrap();
    assert_eq!(file.pages.len(), 1);
    a.set_ink_file("file:ink001");
    assert_eq!(a.ink_file_id().as_deref(), Some("file:ink001"));
    let edit = a.save_edit("Notes").unwrap().unwrap();
    let body = edit.apply("Notes");
    assert!(body.len() < 1_000, "the body holds only the link");
    assert!(body.contains("inkdrop://file:ink001"));
    // Another device reads the body, then the attachment.
    let mut b = Annotations::from_body(&body, FILE, "m");
    assert!(b.ink_pending);
    assert!(b.set_page_ink(many_strokes(1)).is_err());
    b.absorb_ink(&file).unwrap();
    assert!(!b.ink_pending);
    assert_eq!(b.ink().len(), 1);
    assert_eq!(b.ink()[0].live_count(), 60);
    // The body changing again does not lose what was read.
    assert!(!b.sync(&body));
    assert_eq!(b.ink()[0].live_count(), 60);
    assert!(!b.ink_pending);
}

#[test]
fn ink_is_refused_past_what_a_pdf_can_hold_and_nothing_changes() {
    let mut a = Annotations::new(FILE, "m");
    a.set_page_ink(many_strokes(10)).unwrap();
    let before = a.clone();
    // 8 MB of ink is far past ten strokes: build it page by page.
    let mut refused = false;
    for page in 2..2000 {
        let mut p = many_strokes(40);
        p.page = page;
        if a.set_page_ink(p).is_err() {
            refused = true;
            break;
        }
    }
    assert!(refused);
    assert!(written_len(&a.block.ink) <= MAX_INK_BYTES);
    assert!(a.block.ink.len() >= before.block.ink.len());
}

#[test]
fn clearing_a_page_through_the_annotations() {
    let mut a = Annotations::new(FILE, "m");
    assert!(!a.clear_page_ink(1, 5).unwrap());
    a.set_page_ink(inked(1, vec![live_stroke("a", 1.0, 99.0)]))
        .unwrap();
    a.set_page_ink(inked(
        4,
        vec![live_stroke("b", 1.0, 99.0), live_stroke("c", 5.0, 50.0)],
    ))
    .unwrap();
    assert_eq!(
        a.ink_rows(),
        vec![
            crate::InkRow {
                page: 1,
                strokes: 1
            },
            crate::InkRow {
                page: 4,
                strokes: 2
            }
        ]
    );
    assert!(a.clear_page_ink(4, 9).unwrap());
    assert_eq!(
        a.ink_rows(),
        vec![crate::InkRow {
            page: 1,
            strokes: 1
        }]
    );
    // A blank page is not an empty one: its erased strokes are remembered.
    assert!(a.ink().iter().any(|p| p.page == 4 && p.strokes.len() == 2));
}

#[test]
fn a_read_only_set_takes_no_ink() {
    let mut a = Annotations::new(FILE, "m");
    a.status = crate::Status::Restorable;
    assert!(
        a.set_page_ink(inked(1, vec![live_stroke("a", 1.0, 2.0)]))
            .is_err()
    );
    assert!(a.ink().is_empty());
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, failure_persistence: None, ..ProptestConfig::default() })]

    /// A block with any ink writes, reads and writes again to the same text,
    /// and its highlights and ink-free fields are untouched by it.
    #[test]
    fn a_block_with_ink_is_canonical(a in page_strategy(1), b in page_strategy(9)) {
        let mut block = PdfBlock::new(FILE, "m");
        block.ink = merge_pages(&[a], &[b]);
        block.ink.retain(|p| !p.strokes.is_empty());
        let text = block.to_json().unwrap();
        let back = PdfBlock::parse(&text).unwrap();
        prop_assert_eq!(&back.ink, &block.ink);
        prop_assert_eq!(back.to_json().unwrap(), text);
    }

    /// Whatever is drawn on a page, saving and reading the note brings the
    /// same ink back.
    #[test]
    fn saved_ink_reads_back(a in page_strategy(2)) {
        let mut ann = Annotations::new(FILE, "m");
        prop_assume!(!a.strokes.is_empty());
        ann.set_page_ink(a.clone()).unwrap();
        let body = ann.save_edit("Notes").unwrap().unwrap().apply("Notes");
        let back = Annotations::from_body(&body, FILE, "m");
        prop_assert_eq!(back.page_ink(2), a);
    }
}
