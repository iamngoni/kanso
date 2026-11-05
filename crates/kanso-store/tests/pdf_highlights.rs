//! A PDF's highlights in a note through sync (issue #43, step 6): two
//! devices that highlight the same PDF apart both keep their highlights,
//! whichever version of the note wins, and a deleted one stays deleted.

use std::collections::BTreeMap;

use kanso_core::DocType;
use kanso_pdf::highlights::{Annotations, Located, locate};
use kanso_pdf::model::{Color, Quad};
use kanso_pdf::text::HighlightParts;
use kanso_store::{ApplyOutcome, Origin, Put, Store, SyncDoc};
use serde_json::{Map, Value, json};

const FILE: &str = "file:report";

fn parts(page: u32, words: &str) -> HighlightParts {
    let mut quads = BTreeMap::new();
    quads.insert(page, vec![Quad::new(0.1, 0.1, 0.5, 0.2)]);
    HighlightParts {
        quads,
        text: words.into(),
    }
}

fn saved(a: &Annotations, body: &str) -> String {
    a.save_edit(body)
        .unwrap()
        .map_or_else(|| body.to_owned(), |e| e.apply(body))
}

fn fields(body: &str, updated: i64, extra: &[(&str, Value)]) -> Map<String, Value> {
    let mut f = json!({
        "bookId": "book:a",
        "title": "Q3",
        "body": body,
        "updatedAt": updated,
    })
    .as_object()
    .unwrap()
    .clone();
    for (k, v) in extra {
        f.insert((*k).into(), v.clone());
    }
    f
}

/// The store holds `local` as an unsynced edit; then `remote` arrives.
fn meet(
    local: &Map<String, Value>,
    remote: &Map<String, Value>,
) -> (tempfile::TempDir, Store, ApplyOutcome) {
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
    let mut doc = Value::Object(local.clone());
    doc["_id"] = json!("note:q3");
    store
        .put(DocType::Note, doc, Put::Upsert, Origin::Ui)
        .unwrap();
    let remote = SyncDoc {
        id: "note:q3".into(),
        rev: "5-bb".into(),
        deleted: false,
        fields: remote.clone(),
        base_rev: None,
    };
    let outcome = store.apply_remote(&remote, None, None).unwrap();
    (dir, store, outcome)
}

fn ids_in(store: &Store) -> Vec<String> {
    let body = store.note("note:q3").unwrap().unwrap().body;
    match locate(&body, FILE) {
        Located::Found(b) => {
            let mut ids: Vec<String> = b.highlights.into_iter().map(|h| h.id).collect();
            ids.sort();
            ids
        }
        other => panic!("{other:?}: {body}"),
    }
}

/// The note both devices started from: one highlight, `kept`, and one each
/// side may delete, `old`.
fn start() -> (String, String, String) {
    let mut a = Annotations::new(FILE, "m");
    let kept = a.add(parts(1, "kept"), Color::Yellow, 1).unwrap();
    let old = a.add(parts(2, "old"), Color::Green, 2).unwrap();
    (saved(&a, "# Q3\n\nnotes\n"), kept, old)
}

#[test]
fn highlights_made_on_two_devices_are_both_kept_whichever_wins() {
    let (base, kept, old) = start();
    let mut mac = Annotations::from_body(&base, FILE, "m");
    let on_mac = mac.add(parts(3, "mac"), Color::Blue, 10).unwrap();
    mac.delete(&old).unwrap();
    let mac_body = saved(&mac, &format!("{base}\ntyped on the Mac\n"));
    let mut phone = Annotations::from_body(&base, FILE, "m");
    let on_phone = phone.add(parts(4, "phone"), Color::Pink, 20).unwrap();
    phone.set_comment(&kept, Some("from the phone")).unwrap();
    let phone_body = saved(&phone, &base);

    let mut want = vec![kept.clone(), on_mac, on_phone];
    want.sort();
    // The phone's version is newer, then older.
    for (l_up, r_up) in [(1_000, 2_000), (2_000, 1_000)] {
        let (_dir, store, outcome) = meet(
            &fields(&mac_body, l_up, &[]),
            &fields(&phone_body, r_up, &[]),
        );
        assert!(
            matches!(outcome, ApplyOutcome::LocalWon { .. }),
            "{outcome:?}"
        );
        assert_eq!(
            ids_in(&store),
            want,
            "a highlight was lost ({l_up} vs {r_up})"
        );
        let body = store.note("note:q3").unwrap().unwrap().body;
        let a = Annotations::from_body(&body, FILE, "m");
        assert_eq!(
            a.get(&kept).unwrap().comment.as_deref(),
            Some("from the phone")
        );
        // The winner's text, and one block.
        assert_eq!(body.contains("typed on the Mac"), l_up > r_up, "{body}");
        assert_eq!(body.matches("```kanso-pdf").count(), 1);
        // The other version is in history.
        assert!(store.revisions("note:q3").unwrap().len() >= 2);
    }
}

/// A newer Kanso's block on the winning side is kept byte for byte: this
/// build doesn't read it, so it doesn't merge into it.
#[test]
fn a_newer_block_is_not_merged_into() {
    let (base, ..) = start();
    let mut mac = Annotations::from_body(&base, FILE, "m");
    mac.add(parts(3, "mac"), Color::Blue, 10).unwrap();
    let mac_body = saved(&mac, &base);
    let newer = format!(
        "# Q3\n\n```kanso-pdf\n{{\"file\":\"{FILE}\",\"md5\":\"m\",\"version\":2,\"highlights\":[{{\"id\":\"zz\",\"shape\":\"circle\"}}]}}\n```\n"
    );
    let (_dir, store, outcome) = meet(&fields(&mac_body, 1_000, &[]), &fields(&newer, 2_000, &[]));
    assert!(matches!(outcome, ApplyOutcome::RemoteWon), "{outcome:?}");
    assert_eq!(store.note("note:q3").unwrap().unwrap().body, newer);
}

fn pencil(id: &str, y: f64) -> kanso_pdf::PageInk {
    use kanso_core::canvas::{Point, Stroke};
    kanso_pdf::PageInk {
        page: 1,
        strokes: vec![Stroke {
            id: id.into(),
            tool: "pen".into(),
            color: "#1e1e1e".into(),
            width: 4.0,
            points: vec![Point::new(10.0, y, 0.5), Point::new(300.0, y, 0.7)],
            ..Stroke::default()
        }],
        extra: Default::default(),
    }
}

/// Pencil drawn on two devices apart, on the same page, is all kept
/// whichever version of the note wins (step 10).
#[test]
fn pencil_drawn_on_two_devices_is_kept_whichever_wins() {
    let (base, ..) = start();
    let mut mac = Annotations::from_body(&base, FILE, "m");
    mac.set_page_ink(pencil("onmac", 100.0)).unwrap();
    let mac_body = saved(&mac, &base);
    let mut phone = Annotations::from_body(&base, FILE, "m");
    phone.set_page_ink(pencil("onphone", 400.0)).unwrap();
    let phone_body = saved(&phone, &base);
    for (l_up, r_up) in [(1_000, 2_000), (2_000, 1_000)] {
        let (_dir, store, _) = meet(
            &fields(&mac_body, l_up, &[]),
            &fields(&phone_body, r_up, &[]),
        );
        let body = store.note("note:q3").unwrap().unwrap().body;
        let a = Annotations::from_body(&body, FILE, "m");
        let mut ids: Vec<String> = a.page_ink(1).strokes.iter().map(|s| s.id.clone()).collect();
        ids.sort();
        assert_eq!(ids, vec!["onmac", "onphone"], "{l_up} vs {r_up}");
        assert_eq!(body.matches("```kanso-pdf").count(), 1);
    }
}
