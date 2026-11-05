//! Properties of inline sketches in a note: the fence round-trips, edits stay
//! within the caps, promotion keeps every stroke, and the index reads a
//! sketch's words and never its JSON.

use proptest::prelude::*;

use super::tests::config;
use super::*;

fn arb_stroke() -> impl Strategy<Value = Stroke> {
    (
        prop::collection::vec((0u32..4800, 0u32..3200, 0u32..=1000), 1..40),
        prop::sample::select(vec!["pen", "pencil", "marker", "highlighter"]),
    )
        .prop_map(|(points, tool)| Stroke {
            tool: tool.into(),
            color: "#1e1e1e".into(),
            width: 3.0,
            points: points
                .into_iter()
                .map(|(x, y, p)| Point {
                    x: f64::from(x) / 10.0,
                    y: f64::from(y) / 10.0,
                    pressure: f64::from(p) / 1000.0,
                    tilt: None,
                })
                .collect(),
            ..Stroke::default()
        })
}

fn arb_size() -> impl Strategy<Value = (u32, u32)> {
    prop::sample::select(sketch::SIZES.to_vec())
}

/// A sketch with `strokes` drawn in it.
fn drawn(id: &str, (w, h): (u32, u32), strokes: Vec<Stroke>) -> Canvas {
    let mut s = sketch::new_sketch(id, w, h);
    for (i, stroke) in strokes.into_iter().enumerate() {
        let _ = s.draw(stroke, 1000 + i as i64);
    }
    s
}

fn arb_prose() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 .,\n#-]{0,200}"
}

proptest! {
    #![proptest_config(config(64))]

    /// A sketch written as a fence and read back is the same sketch, and the
    /// text around it is untouched.
    #[test]
    fn prop_fence_round_trips(
        size in arb_size(),
        strokes in prop::collection::vec(arb_stroke(), 0..6),
        before in arb_prose(),
        after in arb_prose(),
    ) {
        let s = drawn("sk-a", size, strokes);
        prop_assume!(s.validate_sketch().is_ok());
        // Keep the prose from forming a fence of its own.
        prop_assume!(!before.contains("```") && !after.contains("```"));
        let body = format!("{before}\n\n{}\n{after}", sketch::to_fence(&s));
        let found = sketch::find(&body);
        prop_assert_eq!(found.len(), 1);
        let (at, read) = sketch::find_by_id(&body, "sk-a").unwrap();
        prop_assert_eq!(&at, &found[0]);
        prop_assert_eq!(&read, &s);
        prop_assert!(body.starts_with(&before));
        prop_assert!(body.ends_with(&after));
        // Writing the same sketch again changes nothing.
        prop_assert_eq!(sketch::replace(&body, "sk-a", &s).unwrap(), body);
    }

    /// Insert puts one empty sketch on lines of its own, with an ID no other
    /// sketch in the note has, and leaves the text on both sides alone.
    #[test]
    fn prop_insert_adds_one_sketch(
        size in arb_size(),
        before in arb_prose(),
        after in arb_prose(),
        count in 0usize..4,
    ) {
        prop_assume!(!before.contains("```") && !after.contains("```"));
        let mut body = format!("{before}{after}");
        let mut ids = std::collections::HashSet::new();
        for _ in 0..count {
            let (id, next) = sketch::insert(&body, body.len(), size.0, size.1).unwrap();
            prop_assert!(ids.insert(id));
            body = next;
        }
        let (id, next) = sketch::insert(&body, before.len().min(body.len()), size.0, size.1).unwrap();
        prop_assert!(!ids.contains(&id));
        prop_assert_eq!(sketch::sketches(&next).len(), count + 1);
        prop_assert!(sketch::check_budget(&next).is_ok());
        // The text either side is still there, in order.
        let squeeze = |s: &str| s.split_whitespace().collect::<String>();
        let without: String = {
            let mut text = next.clone();
            for (found, _) in sketch::sketches(&next).into_iter().rev() {
                text.replace_range(found.range, "");
            }
            text
        };
        prop_assert_eq!(squeeze(&without), squeeze(&format!("{before}{after}")));
    }

    /// Whatever is drawn or written, a checked replace either refuses or
    /// leaves every sketch within its cap and the note within its own.
    #[test]
    fn prop_edits_stay_within_caps(
        pad in 0usize..(2 * sketch::MAX_SKETCH_BYTES),
        others in 0usize..4,
        strokes in prop::collection::vec(arb_stroke(), 0..4),
    ) {
        let mut body = String::from("Notes\n\n");
        // Other sketches, each padded (JSON allows spaces) toward its cap.
        for i in 0..others {
            let mut s = sketch::new_sketch(&format!("sk-o{i}"), 320, 240);
            s.extra.insert("pad".into(), "x".repeat(pad.min(sketch::MAX_SKETCH_BYTES - 2000)).into());
            body.push_str(&sketch::to_fence(&s));
            body.push('\n');
        }
        let (id, with) = match sketch::insert(&body, body.len(), 480, 320) {
            Ok(inserted) => inserted,
            Err(refusal) => {
                // Refused only because the note's sketches are already full.
                prop_assert_eq!(refusal, sketch::Refusal::Full(sketch::NOTE_FULL.into()));
                prop_assert!(sketch::used_bytes(&body) + 600 > sketch::MAX_NOTE_SKETCH_BYTES);
                return Ok(());
            }
        };
        let mut s = sketch::find_by_id(&with, &id).unwrap().1;
        for (i, stroke) in strokes.into_iter().enumerate() {
            let _ = s.draw(stroke, 2000 + i as i64);
        }
        s.extra.insert("pad".into(), "y".repeat(pad).into());
        match sketch::replace_checked(&with, &id, &s) {
            Ok(out) => {
                prop_assert!(sketch::check_budget(&out).is_ok());
                prop_assert!(sketch::used_bytes(&out) <= sketch::MAX_NOTE_SKETCH_BYTES
                    || sketch::used_bytes(&out) <= sketch::used_bytes(&with));
                for (found, _) in sketch::sketches(&out) {
                    prop_assert!(found.json.len() <= sketch::MAX_SKETCH_BYTES);
                }
            }
            Err(sketch::Refusal::Full(words)) => {
                prop_assert!(words == sketch::SKETCH_FULL || words == sketch::NOTE_FULL);
            }
            Err(sketch::Refusal::Missing) => prop_assert!(false, "the sketch is in the note"),
        }
    }

    /// Promotion keeps every stroke, point for point, the recognized
    /// handwriting, and leaves one embed where the fence was and every other
    /// sketch alone.
    #[test]
    fn prop_promotion_keeps_every_stroke(
        size in arb_size(),
        strokes in prop::collection::vec(arb_stroke(), 0..8),
        which in 0usize..3,
        word in "[a-z]{3,10}",
    ) {
        let mut sketches: Vec<Canvas> = (0..3)
            .map(|i| drawn(&format!("sk-{i}"), size, strokes.clone()))
            .collect();
        sketches[which].kanso_ink_ocr.push(InkOcr {
            node: "i1".into(),
            md5: "m".into(),
            engine: "vision".into(),
            text: word.clone(),
        });
        let body: String = sketches
            .iter()
            .map(|s| format!("Intro\n\n{}\n", sketch::to_fence(s)))
            .collect();
        let (doc, next) = sketch::promote(&body, &format!("sk-{which}"), "NoteId1234").unwrap();
        // Same strokes, same order, same points.
        let before: Vec<&Stroke> = sketches[which].live_strokes().map(|(_, s)| s).collect();
        let after: Vec<&Stroke> = doc.live_strokes().map(|(_, s)| s).collect();
        prop_assert_eq!(before, after);
        prop_assert!(doc.validate().is_ok());
        prop_assert!(doc.kanso_sketch.is_none() && doc.kanso_sketch_id.is_none());
        prop_assert!(doc.search_text().contains(&word));
        // The fence became an embed; nothing else moved.
        let target = format!("sk-{which}");
        prop_assert!(sketch::find_by_id(&next, &target).is_none());
        prop_assert!(next.contains(&sketch::embed("NoteId1234", size.0)));
        prop_assert_eq!(sketch::sketches(&next).len(), 2);
        for i in (0..3).filter(|i| *i != which) {
            let (_, other) = sketch::find_by_id(&next, &format!("sk-{i}")).unwrap();
            prop_assert_eq!(&other, &sketches[i]);
        }
        // Promoting twice finds nothing and promotes nothing.
        prop_assert!(sketch::promote(&next, &target, "Other").is_none());
        // The document survives being written as a canvas note.
        prop_assert_eq!(parse(&to_body(&doc)).unwrap(), doc);
    }

    /// The index reads the words recognized in a sketch and the note's prose,
    /// and never a sketch's JSON, however much ink it holds.
    #[test]
    fn prop_index_has_the_words_and_not_the_json(
        size in arb_size(),
        strokes in prop::collection::vec(arb_stroke(), 0..6),
        word in "[a-z]{4,12}",
        prose in "[a-z]{4,12}",
    ) {
        let mut s = drawn("sk-i", size, strokes);
        s.kanso_ink_ocr.push(InkOcr {
            node: "i1".into(),
            md5: "m".into(),
            engine: "vision".into(),
            text: word.clone(),
        });
        let body = format!("{prose}\n\n{}\nend", sketch::to_fence(&s));
        let text = sketch::index_text(&body);
        prop_assert!(text.contains(&word));
        prop_assert!(text.contains(&prose));
        for json_word in ["kansoSketch", "\"nodes\"", "pressure", "kansoVersion", "\"points\""] {
            prop_assert!(!text.contains(json_word));
        }
        // Words that were never recognized are not invented.
        s.kanso_ink_ocr.clear();
        let blank = sketch::index_text(&format!("{prose}\n\n{}\nend", sketch::to_fence(&s)));
        prop_assert!(!blank.contains(&word) || prose.contains(&word));
    }

    /// Points outside the box are not the sketch's.
    #[test]
    fn prop_box_contains_only_its_own_points(
        (w, h) in arb_size(),
        x in -1000.0f64..2000.0,
        y in -1000.0f64..2000.0,
    ) {
        let b = Sketch { w, h, v: 1 };
        let inside = x >= 0.0 && y >= 0.0 && x <= f64::from(w) && y <= f64::from(h);
        prop_assert_eq!(b.contains(x, y), inside);
        prop_assert!(!b.contains(f64::NAN, y) && !b.contains(x, f64::INFINITY));
    }
}
