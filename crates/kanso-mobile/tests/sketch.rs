//! Inline sketches on the phone (#45 step 10): the same session as a canvas,
//! over one sketch's fence in a Markdown note. Ink stays in its box and
//! within the sketch's cap, saves go into that fence only, a shared note's
//! sketch is read only, and "Open as canvas" keeps every stroke.

mod common;

use common::*;
use kanso_core::canvas::sketch;
use kanso_mobile::{InkStyle, KansoError, PenSample, sketch_insert};
use proptest::prelude::*;
use serde_json::json;

fn pen() -> InkStyle {
    InkStyle {
        tool: "pen".into(),
        color: "#e03131".into(),
        width: 3.0,
    }
}

fn line(y: f64) -> Vec<PenSample> {
    (0..20)
        .map(|i| PenSample {
            x: 10.0 + f64::from(i) * 5.0,
            y,
            pressure: 0.6,
            ..PenSample::default()
        })
        .collect()
}

/// A note holding text, a sketch with the ID "sk-a", and a second one, "sk-b".
fn note_with_sketches(phone: &Phone) -> String {
    let a = sketch::to_fence(&sketch::new_sketch("sk-a", 320, 240));
    let b = sketch::to_fence(&sketch::new_sketch("sk-b", 480, 320));
    note(&phone.library, "Notes", &format!("Top\n\n{a}\nmiddle\n\n{b}\nEnd"))
}

fn body(phone: &Phone, id: &str) -> String {
    phone.library.note(id).unwrap().body.unwrap()
}

#[test]
fn drawing_saves_into_that_sketch_only() {
    let phone = lone_phone();
    let id = note_with_sketches(&phone);
    let before = body(&phone, &id);
    let session = phone.library.sketch(&id, "sk-b").unwrap();
    let info = session.sketch_info().unwrap();
    assert_eq!((info.id.as_str(), info.width, info.height), ("sk-b", 480, 320));
    assert!(!info.read_only);
    let view = session.draw(pen(), line(50.0), vec![]).unwrap();
    assert_eq!(view.strokes.len(), 1);
    // Nothing is written until the shell saves.
    assert_eq!(body(&phone, &id), before);
    session.save().unwrap();
    let after = body(&phone, &id);
    assert_ne!(after, before);
    let a = sketch::find_by_id(&after, "sk-a").unwrap().1;
    assert_eq!(a, sketch::new_sketch("sk-a", 320, 240));
    let b = sketch::find_by_id(&after, "sk-b").unwrap().1;
    assert_eq!(b.live_strokes().count(), 1);
    assert!(after.starts_with("Top\n\n") && after.ends_with("End"));
    // A save of an unchanged session writes nothing.
    session.save().unwrap();
    assert_eq!(body(&phone, &id), after);
}

#[test]
fn ink_outside_the_box_is_ignored() {
    let phone = lone_phone();
    let id = note_with_sketches(&phone);
    let session = phone.library.sketch(&id, "sk-a").unwrap();
    // The box is 320 by 240: every sample of this line is below it.
    let view = session.draw(pen(), line(900.0), vec![]).unwrap();
    assert!(view.strokes.is_empty());
    // Part in, part out: only the inside is the sketch's.
    let mixed: Vec<PenSample> = (0..30)
        .map(|i| PenSample {
            x: f64::from(i) * 20.0,
            y: 100.0,
            pressure: 0.5,
            ..PenSample::default()
        })
        .collect();
    let view = session.draw(pen(), mixed, vec![]).unwrap();
    assert_eq!(view.strokes.len(), 1);
    let right = view.strokes[0].bounds.x + view.strokes[0].bounds.width;
    // The points end at the box's edge; the outline adds the pen's width.
    assert!(right <= 320.0 + 6.0);
}

#[test]
fn a_full_sketch_refuses_new_ink() {
    let phone = lone_phone();
    let mut full = sketch::new_sketch("sk-f", 320, 240);
    full.extra
        .insert("pad".into(), "x".repeat(sketch::MAX_SKETCH_BYTES - 700).into());
    let id = note(
        &phone.library,
        "Full",
        &sketch::to_fence(&full),
    );
    let session = phone.library.sketch(&id, "sk-f").unwrap();
    let many: Vec<PenSample> = (0..300)
        .map(|i| PenSample {
            x: f64::from(i % 300),
            y: f64::from((i * 7) % 240),
            pressure: 0.5,
            ..PenSample::default()
        })
        .collect();
    match session.draw(pen(), many, vec![]) {
        Err(KansoError::Invalid { message }) => assert_eq!(message, sketch::SKETCH_FULL),
        other => panic!("expected the full refusal, got {other:?}"),
    }
    // Refused whole: the session is as it was and saves nothing.
    assert!(session.view(vec![]).unwrap().strokes.is_empty());
}

#[test]
fn a_shared_note_sketch_is_read_only() {
    let fence = sketch::to_fence(&sketch::new_sketch("sk-s", 320, 240));
    let mut id = String::new();
    let phone = phone_with(&std::sync::Arc::new(kanso_sync::MemoryServer::new()), |store| {
        id = raw_note(
            store,
            json!({
                "title": "Shared",
                "body": format!("Hi\n\n{fence}"),
                "kansoShare": {"shareId": "sh1", "role": "edit", "ownerName": "ada"},
            }),
        );
    });
    let session = phone.library.sketch(&id, "sk-s").unwrap();
    assert!(session.sketch_info().unwrap().read_only);
    assert!(session.draw(pen(), line(40.0), vec![]).is_err());
    // Copy as canvas: a private canvas, and the shared note is untouched.
    let before = body(&phone, &id);
    let saved = session.open_as_canvas("Shared sketch".into()).unwrap();
    assert_eq!(body(&phone, &id), before);
    assert!(phone.library.is_canvas(&saved.id));
}

#[test]
fn opening_as_canvas_keeps_every_stroke() {
    let phone = lone_phone();
    let id = note_with_sketches(&phone);
    let session = phone.library.sketch(&id, "sk-b").unwrap();
    for y in [20.0, 60.0, 100.0] {
        session.draw(pen(), line(y), vec![]).unwrap();
    }
    let saved = session.open_as_canvas("Notes sketch".into()).unwrap();
    let text = body(&phone, &id);
    assert!(sketch::find_by_id(&text, "sk-b").is_none());
    assert!(text.contains(&format!("note:{}", saved.id)) && text.contains("w:480"));
    assert!(sketch::find_by_id(&text, "sk-a").is_some());
    let canvas = phone.library.canvas(&saved.id).unwrap();
    assert_eq!(canvas.view(vec![]).unwrap().strokes.len(), 3);
    assert_eq!(canvas.view(vec![]).unwrap().title, "Notes sketch");
    // The fence is gone, so a second promotion stops and says so.
    assert!(session.open_as_canvas("Again".into()).is_err());
}

#[test]
fn a_sketch_the_note_lacks_is_not_found() {
    let phone = lone_phone();
    let id = note_with_sketches(&phone);
    assert!(matches!(
        phone.library.sketch(&id, "sk-gone"),
        Err(KansoError::NotFound)
    ));
}

proptest! {
    /// Inserting puts one empty sketch at the cursor, whatever the text and
    /// wherever the cursor is, and the text either side is kept.
    #[test]
    fn prop_insert_keeps_the_text(
        before in "[a-z é#\n]{0,40}",
        after in "[a-z é#\n]{0,40}",
        which in 0usize..3,
    ) {
        let text = format!("{before}{after}");
        let at = before.encode_utf16().count() as u32;
        let (w, h) = sketch::SIZES[which];
        let done = sketch_insert(text.clone(), at, w, h).unwrap();
        let found = sketch::sketches(&done.body);
        prop_assert_eq!(found.len(), 1);
        prop_assert_eq!(found[0].1.kanso_sketch_id.as_deref(), Some(done.id.as_str()));
        prop_assert_eq!(found[0].1.kanso_sketch.map(|s| (s.w, s.h)), Some((w, h)));
        let squeeze = |s: &str| s.split_whitespace().collect::<String>();
        let mut without = done.body.clone();
        without.replace_range(found[0].0.range.clone(), "");
        prop_assert_eq!(squeeze(&without), squeeze(&text));
        // The cursor sits just after the fence.
        let end: u32 = done.body[..found[0].0.range.end].encode_utf16().count() as u32;
        prop_assert_eq!(done.cursor, end);
    }
}
