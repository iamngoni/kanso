//! A PDF's pencil ink once it outgrows the note (issue #43, step 10): the
//! strokes go to an attachment of their own, the note keeps a link to it,
//! and nothing is lost between two writers or on the way back.

use kanso_core::DocType;
use kanso_core::canvas::{Point, Stroke};
use kanso_pdf::Annotations;
use kanso_pdf::ink::{INK_FILE_TYPE, INLINE_INK_BYTES, PageInk, written_len};
use kanso_store::pdf_ink::{Held, load_ink, publish_ink, read_ink_file};
use kanso_store::{Origin, Put, Store};
use serde_json::json;

const FILE: &str = "file:report";
const NOTE: &str = "note:q3";

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    store
        .put(
            DocType::Book,
            json!({"_id": "book:a", "name": "A"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    (dir, store)
}

fn stroke(id: &str, y: f64, points: usize) -> Stroke {
    Stroke {
        id: id.into(),
        tool: "pen".into(),
        color: "#1e1e1e".into(),
        width: 4.0,
        points: (0..points)
            .map(|i| Point::new(f64::from(i as u32 % 900), y, 0.5))
            .collect(),
        ..Stroke::default()
    }
}

/// A page with enough ink to outgrow the note.
fn heavy(page: u32, tag: &str) -> PageInk {
    let mut strokes = Vec::new();
    let mut n = 0;
    while written_len(&[PageInk {
        page,
        strokes: strokes.clone(),
        extra: Default::default(),
    }]) <= INLINE_INK_BYTES + 5_000
    {
        strokes.push(stroke(&format!("{tag}{n}"), 100.0 + f64::from(n), 400));
        n += 1;
    }
    PageInk {
        page,
        strokes,
        extra: Default::default(),
    }
}

fn put_note(store: &Store, id: &str, body: &str) {
    store
        .put(
            DocType::Note,
            json!({"_id": id, "bookId": "book:a", "title": "Q3", "body": body}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
}

#[test]
fn small_ink_stays_in_the_note_and_makes_no_attachment() {
    let (_dir, store) = store();
    let mut a = Annotations::new(FILE, "m");
    a.set_page_ink(PageInk {
        page: 1,
        strokes: vec![stroke("a", 10.0, 20)],
        extra: Default::default(),
    })
    .unwrap();
    publish_ink(&store, &mut a, NOTE, "").unwrap();
    assert!(a.ink_file_id().is_none());
    assert!(a.save_edit("N").unwrap().unwrap().apply("N").len() < INLINE_INK_BYTES);
}

#[test]
fn big_ink_goes_to_an_attachment_the_note_owns_and_reads_back() {
    let (_dir, store) = store();
    let mut a = Annotations::new(FILE, "m");
    a.set_page_ink(heavy(3, "a")).unwrap();
    publish_ink(&store, &mut a, NOTE, "").unwrap();
    let id = a.ink_file_id().expect("the ink has an attachment");
    // It is a file document of the ink type.
    let meta = store.file_meta(&id).unwrap().unwrap();
    assert_eq!(meta.content_type, INK_FILE_TYPE);
    // The note's body holds only the link, and owns the attachment through it.
    let body = a.save_edit("Notes").unwrap().unwrap().apply("Notes");
    assert!(body.len() < 2_000);
    put_note(&store, NOTE, &body);
    assert_eq!(store.files_of_note("note:q3").unwrap(), vec![id.clone()]);
    // Another reader of the note gets the strokes from the attachment.
    let mut b = Annotations::from_body(&body, FILE, "m");
    assert!(b.ink_pending);
    assert!(load_ink(&store, &mut b).unwrap());
    assert_eq!(b.ink(), a.ink());
    assert!(!b.ink_pending);
}

#[test]
fn a_second_save_rewrites_the_same_attachment() {
    let (_dir, store) = store();
    let mut a = Annotations::new(FILE, "m");
    a.set_page_ink(heavy(1, "a")).unwrap();
    publish_ink(&store, &mut a, NOTE, "").unwrap();
    let id = a.ink_file_id().unwrap();
    let body = a.save_edit("N").unwrap().unwrap().apply("N");
    a.set_page_ink(heavy(2, "b")).unwrap();
    // A save that has not yet seen the link takes it from the body.
    let mut fresh = a.clone();
    fresh.block.ink_file = None;
    publish_ink(&store, &mut fresh, NOTE, &body).unwrap();
    assert_eq!(fresh.ink_file_id().as_deref(), Some(id.as_str()));
    match read_ink_file(&store, &id) {
        Held::Found(file, _) => assert_eq!(file.pages.len(), 2),
        other => panic!("{other:?}"),
    }
}

#[test]
fn another_devices_strokes_are_kept_when_this_one_saves() {
    let (_dir, store) = store();
    let mut theirs = Annotations::new(FILE, "m");
    theirs.set_page_ink(heavy(1, "t")).unwrap();
    publish_ink(&store, &mut theirs, NOTE, "").unwrap();
    let body = theirs.save_edit("N").unwrap().unwrap().apply("N");
    // This device read the note before the attachment had arrived, drew on
    // another page, and saves.
    let mut mine = Annotations::from_body(&body, FILE, "m");
    assert!(mine.ink_pending);
    assert!(load_ink(&store, &mut mine).unwrap());
    mine.set_page_ink(heavy(2, "m")).unwrap();
    publish_ink(&store, &mut mine, NOTE, &body).unwrap();
    match read_ink_file(&store, &mine.ink_file_id().unwrap()) {
        Held::Found(file, _) => {
            let pages: Vec<u32> = file.pages.iter().map(|p| p.page).collect();
            assert_eq!(pages, vec![1, 2]);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn ink_whose_attachment_has_not_arrived_is_not_written_over() {
    let (_dir, store) = store();
    let body = {
        let mut a = Annotations::new(FILE, "m");
        a.set_page_ink(heavy(1, "a")).unwrap();
        a.set_ink_file("file:notyet");
        a.save_edit("N").unwrap().unwrap().apply("N")
    };
    let mut mine = Annotations::from_body(&body, FILE, "m");
    assert!(mine.ink_pending);
    // Nothing is drawn meanwhile.
    assert!(mine.set_page_ink(heavy(2, "m")).is_err());
    assert!(!load_ink(&store, &mut mine).unwrap());
    // And nothing is saved over what is not here yet.
    assert!(publish_ink(&store, &mut mine, NOTE, &body).is_err());
}

#[test]
fn a_damaged_attachment_is_reported_and_left_alone() {
    let (_dir, store) = store();
    let mut a = Annotations::new(FILE, "m");
    a.set_page_ink(heavy(1, "a")).unwrap();
    publish_ink(&store, &mut a, NOTE, "").unwrap();
    let id = a.ink_file_id().unwrap();
    let meta = store.file_meta(&id).unwrap().unwrap();
    store
        .put(
            DocType::File,
            json!({
                "_id": id, "_rev": meta.rev, "name": "x.json",
                "contentType": INK_FILE_TYPE, "publicIn": [],
                "_attachments": {"index": {"content_type": INK_FILE_TYPE, "data": "e29d"}},
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    assert!(matches!(read_ink_file(&store, &id), Held::Unreadable(_)));
    assert!(publish_ink(&store, &mut a, NOTE, "").is_err());
}
