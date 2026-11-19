//! Example tests for the importer against synthetic PouchDB libraries.

mod common;

use std::path::{Path, PathBuf};

use common::{
    Fixture, Revision, binary_string, book_fields, file_fields, note_fields, rev, tag_fields,
};
use kanso_core::DocType;
use kanso_import::pouch::escape_blob;
use kanso_import::{ImportReport, Phase, import};
use kanso_store::{Attachment, Store};
use serde_json::{Value, json};

struct Env {
    dir: tempfile::TempDir,
    source: PathBuf,
    fixture: Fixture,
    store: Store,
}

fn env() -> Env {
    let dir = tempfile::tempdir().unwrap();
    let db_dir = dir.path().join("library/db");
    std::fs::create_dir_all(&db_dir).unwrap();
    let source = db_dir.join("5c172db466294e6a0b7855cdba0b8d18.sqlite");
    let fixture = Fixture::create(&source);
    let store = Store::open(dir.path().join("kanso/kanso.db")).unwrap();
    Env {
        dir,
        source,
        fixture,
        store,
    }
}

fn run(env: &Env) -> ImportReport {
    import(&env.source, &env.store, |_| {}).unwrap()
}

/// Every byte value, so the binary-string decoding is exercised end to end.
fn png_bytes() -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
    bytes.extend(0..=255u8);
    bytes
}

/// A small database shaped like the real one: nested and non-random notebook
/// IDs, a trashed note, a deleted note, a note with conflicting revisions,
/// links, an embedded image, activity docs, design docs and `config`.
fn realistic(fixture: &Fixture) {
    fixture.put_live("book:inbox", &rev(4, 1), book_fields("Inbox", None));
    fixture.put_live(
        "book:Wk1-a_b",
        &rev(2, 2),
        book_fields("Work", Some("book:inbox")),
    );
    fixture.put_doc(
        "book:gone",
        &[
            Revision::live(&rev(1, 3), book_fields("Gone", None)),
            Revision::deleted(&rev(2, 3)),
        ],
        1,
    );
    fixture.put_live("tag:red", &rev(1, 4), tag_fields("urgent", "red"));
    fixture.put_live("tag:blu", &rev(3, 5), tag_fields("later", "blue"));

    let png = png_bytes();
    fixture.put_live(
        "file:img1",
        &rev(1, 6),
        file_fields("shot.png", &png, "image/png"),
    );
    fixture.put_attachment(&png);

    let body_a = "# A\n- [ ] one\n- [x] two\nSee [B](kanso://note/Bb-2_x) and \
                  [C](kanso://note/c3)\n![shot](kanso://file:img1)\n";
    let mut a = note_fields("Note A", body_a, "book:Wk1-a_b");
    a["tags"] = json!(["tag:red", "tag:blu"]);
    a["sourceTemplateId"] = json!("note:tpl00001");
    fixture.put_live("note:aaaa1111", &rev(7, 7), a);

    // Three revisions; PouchDB's winner is the second (a conflict lost).
    fixture.put_doc(
        "note:Bb-2_x",
        &[
            Revision::live(&rev(1, 8), note_fields("B old", "old", "book:inbox")),
            Revision::live(&rev(2, 9), note_fields("B", "current", "book:inbox")),
            Revision::live(
                &rev(2, 10),
                note_fields("B conflict", "loser", "book:inbox"),
            ),
        ],
        1,
    );
    fixture.put_live(
        "note:c3",
        &rev(1, 11),
        note_fields(
            "C",
            "[x](kanso://note/missing) ![y](kanso://file:nope)",
            "book:inbox",
        ),
    );
    fixture.put_live(
        "note:trash001",
        &rev(3, 12),
        note_fields("Old", "bye", "trash"),
    );
    fixture.put_doc(
        "note:dead0001",
        &[
            Revision::live(&rev(1, 13), note_fields("Dead", "x", "book:inbox")),
            Revision::deleted(&rev(2, 13)),
        ],
        1,
    );
    // Real notes sometimes lack `pinned` and the task counts.
    let mut partial = note_fields("Partial", "- [ ] a\n", "book:inbox");
    let partial_map = partial.as_object_mut().unwrap();
    partial_map.remove("pinned");
    partial_map.remove("numOfTasks");
    partial_map.remove("numOfCheckedTasks");
    fixture.put_live("note:partial1", &rev(2, 14), partial);

    let activity = json!({
        "items": {"note:aaaa1111": [{"action": "update", "at": 1}]},
        "createdAt": 1, "updatedAt": 2,
    });
    fixture.put_live("activity:a1", &rev(5, 15), activity.clone());
    fixture.put_live("activity:a2", &rev(1, 16), activity);
    fixture.put_live("_design/index_notes", "3-aa", json!({"views": {"x": {}}}));
    fixture.put_live("_design/mobile", "1-bb", json!({"filters": {}}));
    fixture.put_live("config", "9-cc", json!({"core": {}, "updatedAt": 5}));
    fixture.put_live("widget:w1", "1-dd", json!({"name": "unknown kind"}));
}

#[test]
fn imports_a_realistic_database() {
    let env = env();
    realistic(&env.fixture);
    let report = run(&env);

    assert_eq!(report.notes, 5);
    assert_eq!(report.trashed_notes, 1);
    assert_eq!(report.books, 2);
    assert_eq!(report.tags, 2);
    assert_eq!(report.files, 1);
    assert_eq!(report.archived, 2);
    assert_eq!(report.skipped_design_and_config, 3);
    assert_eq!(report.skipped_deleted, 2);
    assert_eq!(report.skipped_unknown, 1);
    assert_eq!(report.unchanged, 0);
    assert_eq!(report.replaced, 0);
    assert!(report.attachment_mismatches.is_empty());
    assert!(report.rejected.is_empty());
    assert!(report.unverified.is_empty());
    assert_eq!(report.notes_linking_notes, 2);
    assert_eq!(report.notes_embedding_files, 2);
    assert_eq!(report.dangling_note_links, 1);
    assert_eq!(report.dangling_file_links, 1);
    assert_eq!(report.source.notes, 5);
    assert_eq!(report.source.trashed_notes, 1);
    assert_eq!(report.source.activities, 2);
    assert!(report.is_clean());

    let counts = env.store.counts().unwrap();
    assert_eq!(counts[&DocType::Note], 5);
    assert_eq!(counts[&DocType::Book], 2);
    assert_eq!(counts[&DocType::Tag], 2);
    assert_eq!(counts[&DocType::File], 1);

    // The winning revision, not the newest row or a losing conflict.
    let b = env.store.get("note:Bb-2_x").unwrap().unwrap();
    assert_eq!(b.rev, rev(2, 9));
    assert_eq!(b.fields["title"], "B");
    assert_eq!(b.fields["body"], "current");
    assert!(env.store.get("note:dead0001").unwrap().is_none());
    assert!(env.store.get("book:gone").unwrap().is_none());

    let a = env.store.note("note:aaaa1111").unwrap().unwrap();
    assert_eq!(a.rev.as_deref(), Some(rev(7, 7).as_str()));
    assert_eq!(a.tags, vec!["tag:red", "tag:blu"]);
    assert_eq!((a.num_of_tasks, a.num_of_checked_tasks), (2, 1));
    assert_eq!(a.extra["sourceTemplateId"], "note:tpl00001");

    let work = env.store.get("book:Wk1-a_b").unwrap().unwrap();
    assert_eq!(work.fields["parentBookId"], "book:inbox");
    assert!(!work.fields.contains_key("count"), "count is derived");

    let (content_type, attachment) = env.store.attachment("file:img1").unwrap().unwrap();
    assert_eq!(content_type, "image/png");
    assert_eq!(attachment.data, png_bytes());
    let file = env.store.get("file:img1").unwrap().unwrap();
    assert!(!file.fields.contains_key("_attachments"));
    assert_eq!(file.fields["contentLength"], png_bytes().len());

    // Missing fields get Kanso's defaults.
    let partial = env.store.note("note:partial1").unwrap().unwrap();
    assert!(!partial.pinned);
    assert_eq!(partial.num_of_tasks, 1);

    let summary = report.to_string();
    assert!(
        summary.contains(
            "Imported: 5 notes (1 in trash), 2 notebooks, 2 tags, 1 file; archived 2 activity docs."
        ),
        "{summary}"
    );
    assert!(
        summary.contains("Skipped: 3 design/config, 2 deleted, 1 unknown."),
        "{summary}"
    );
    assert!(summary.contains("Links: 2 notes link other notes (1 dangling targets); 2 embed files (1 dangling targets)."), "{summary}");
    assert!(summary.contains("Attachment mismatches: 0."), "{summary}");
}

#[test]
fn activity_docs_are_archived_verbatim_with_id_and_rev() {
    let env = env();
    let fields = json!({"items": {"note:x": [{"action": "create", "at": 3}]}, "createdAt": 3, "updatedAt": 4});
    env.fixture
        .put_live("activity:Zz9", &rev(2, 1), fields.clone());
    run(&env);
    let conn = rusqlite::Connection::open(env.store.path()).unwrap();
    let json: String = conn
        .query_row(
            "SELECT json FROM archive WHERE id = 'activity:Zz9'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let mut expected = fields;
    expected["_id"] = json!("activity:Zz9");
    expected["_rev"] = json!(rev(2, 1));
    assert_eq!(serde_json::from_str::<Value>(&json).unwrap(), expected);
}

#[test]
fn second_import_reports_everything_unchanged() {
    let env = env();
    realistic(&env.fixture);
    let first = run(&env);
    let seq = env.store.last_seq().unwrap();
    let second = run(&env);
    assert_eq!(second.written(), 0);
    assert_eq!(second.unchanged, first.written());
    assert_eq!(second.unchanged, 5 + 2 + 2 + 1 + 2);
    assert_eq!(env.store.last_seq().unwrap(), seq);
    assert_eq!(second.notes_linking_notes, first.notes_linking_notes);
    assert!(second.to_string().contains("Unchanged: 12."), "{second}");
}

#[test]
fn a_newer_source_revision_replaces_the_stored_one() {
    let env = env();
    env.store
        .import_doc(
            DocType::Note,
            "note:upd00001",
            &rev(1, 1),
            common::object(note_fields("Old", "old", "book:inbox")),
            None,
        )
        .unwrap();
    env.fixture.put_live(
        "note:upd00001",
        &rev(2, 2),
        note_fields("New", "new", "book:inbox"),
    );
    let report = run(&env);
    assert_eq!((report.notes, report.replaced), (1, 1));
    assert_eq!(
        env.store.get("note:upd00001").unwrap().unwrap().fields["title"],
        "New"
    );
}

#[test]
fn corrupt_and_missing_attachments_are_reported_and_skipped() {
    let env = env();
    let good = b"good bytes".to_vec();
    env.fixture.put_live(
        "file:good",
        &rev(1, 1),
        file_fields("g.png", &good, "image/png"),
    );
    env.fixture.put_attachment(&good);

    // Same digest as advertised, but the stored bytes differ.
    let claimed = b"original".to_vec();
    env.fixture.put_live(
        "file:flip",
        &rev(1, 2),
        file_fields("f.png", &claimed, "image/png"),
    );
    env.fixture.put_attachment_body(
        &Attachment::new(claimed.clone()).digest(),
        &binary_string(b"originaL"),
        false,
    );
    // A character above U+00FF can't come from a binary string.
    let wide = b"wide".to_vec();
    env.fixture.put_live(
        "file:wide",
        &rev(1, 3),
        file_fields("w.png", &wide, "image/png"),
    );
    env.fixture.put_attachment_body(
        &Attachment::new(wide.clone()).digest(),
        "wi\u{2603}e",
        false,
    );
    // No body at all.
    env.fixture.put_live(
        "file:none",
        &rev(1, 4),
        file_fields("n.png", b"nothing", "image/png"),
    );
    // A length that disagrees with the bytes.
    let short = b"short".to_vec();
    let mut fields = file_fields("s.png", &short, "image/png");
    fields["_attachments"]["index"]["length"] = json!(99);
    env.fixture.put_live("file:len", &rev(1, 5), fields);
    env.fixture.put_attachment(&short);

    let report = run(&env);
    assert_eq!(report.files, 1);
    let mut ids: Vec<&str> = report
        .attachment_mismatches
        .iter()
        .map(|line| line.split_once(": ").unwrap().0)
        .collect();
    ids.sort();
    assert_eq!(ids, ["file:flip", "file:len", "file:none", "file:wide"]);
    let text = report.attachment_mismatches.join("\n");
    assert!(text.contains("digest mismatch"), "{text}");
    assert!(text.contains("U+2603"), "{text}");
    assert!(text.contains("no attachment body"), "{text}");
    assert!(text.contains("length mismatch"), "{text}");
    for id in ["file:flip", "file:wide", "file:none", "file:len"] {
        assert!(env.store.get(id).unwrap().is_none(), "{id} was imported");
    }
    assert!(!report.is_clean());
    assert!(report.to_string().contains("Attachment mismatches: 4."));
}

#[test]
fn escaped_attachment_bodies_are_unescaped() {
    let env = env();
    let bytes = vec![0u8, 1, 2, 1, 1, 0, 255, 2, 2];
    env.fixture.put_live(
        "file:esc",
        &rev(1, 1),
        file_fields("e.png", &bytes, "image/png"),
    );
    env.fixture.put_attachment_body(
        &Attachment::new(bytes.clone()).digest(),
        &escape_blob(&binary_string(&bytes)),
        true,
    );
    let report = run(&env);
    assert!(report.is_clean(), "{report}");
    assert_eq!(
        env.store.attachment("file:esc").unwrap().unwrap().1.data,
        bytes
    );
}

#[test]
fn unreadable_and_invalid_documents_are_rejected_not_fatal() {
    let env = env();
    env.fixture
        .conn
        .execute(
            r#"INSERT INTO "by-sequence" (json, deleted, doc_id, rev) VALUES ('{not json', 0, 'note:broken1', '1-ab')"#,
            [],
        )
        .unwrap();
    let seq = env.fixture.conn.last_insert_rowid();
    env.fixture
        .conn
        .execute(
            r#"INSERT INTO "document-store" (id, json, winningseq, max_seq) VALUES ('note:broken1', '{}', ?1, ?1)"#,
            [seq],
        )
        .unwrap();
    // The store refuses a note without a bookId.
    env.fixture.put_live(
        "note:nobook01",
        &rev(1, 2),
        json!({"title": "x", "body": "y"}),
    );
    env.fixture.put_live(
        "note:fine0001",
        &rev(1, 3),
        note_fields("ok", "ok", "book:inbox"),
    );

    let report = run(&env);
    assert_eq!(report.notes, 1);
    assert_eq!(report.rejected.len(), 2, "{:?}", report.rejected);
    assert!(report.rejected[0].starts_with("note:broken1: its JSON does not parse"));
    assert!(report.rejected[1].starts_with("note:nobook01: "));
    assert!(report.to_string().contains("Rejected: 2."));
}

#[test]
fn progress_walks_the_phases_in_order() {
    let env = env();
    realistic(&env.fixture);
    let mut events = Vec::new();
    import(&env.source, &env.store, |p| events.push(p)).unwrap();
    let mut phases: Vec<Phase> = events.iter().map(|p| p.phase).collect();
    phases.dedup();
    assert_eq!(
        phases,
        [
            Phase::Snapshot,
            Phase::Books,
            Phase::Tags,
            Phase::Files,
            Phase::Notes,
            Phase::Activities,
            Phase::Verify
        ]
    );
    let notes: Vec<_> = events.iter().filter(|p| p.phase == Phase::Notes).collect();
    assert_eq!(notes.first().unwrap().done, 0);
    assert_eq!(notes.last().unwrap().done, 5);
    assert!(notes.iter().all(|p| p.total == 5));
    assert!(events.iter().all(|p| p.done <= p.total));
}

fn file_bytes(path: &Path) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

#[test]
fn snapshots_while_the_source_app_writes_and_never_touches_its_files() {
    let env = env();
    realistic(&env.fixture);
    // The source app is "running": another connection holds an open write transaction.
    let source_app = rusqlite::Connection::open(&env.source).unwrap();
    source_app.execute_batch("BEGIN IMMEDIATE").unwrap();
    source_app
        .execute(
            r#"INSERT INTO "by-sequence" (json, deleted, doc_id, rev) VALUES (?1, 0, 'note:late0001', '1-ff')"#,
            [note_fields("Late", "late", "book:inbox").to_string()],
        )
        .unwrap();
    let seq = source_app.last_insert_rowid();
    source_app
        .execute(
            r#"INSERT INTO "document-store" (id, json, winningseq, max_seq) VALUES ('note:late0001', '{}', ?1, ?1)"#,
            [seq],
        )
        .unwrap();

    let wal = PathBuf::from(format!("{}-wal", env.source.display()));
    let before = (file_bytes(&env.source), file_bytes(&wal));
    assert!(before.1.is_some(), "the fixture keeps data in the WAL");
    let mtime = std::fs::metadata(&env.source).unwrap().modified().unwrap();

    let report = run(&env);
    assert_eq!(report.notes, 5, "uncommitted writes are not seen");
    assert_eq!((file_bytes(&env.source), file_bytes(&wal)), before);
    assert_eq!(
        std::fs::metadata(&env.source).unwrap().modified().unwrap(),
        mtime
    );

    // The import held no lock: the app's transaction still commits.
    source_app.execute_batch("COMMIT").unwrap();
    let report = run(&env);
    assert_eq!((report.notes, report.unchanged), (1, 12));
    assert!(env.store.get("note:late0001").unwrap().is_some());
}

#[test]
fn refuses_files_that_are_not_pouchdb_libraries() {
    let env = env();
    let other = env.dir.path().join("other.sqlite");
    rusqlite::Connection::open(&other)
        .unwrap()
        .execute_batch("CREATE TABLE t (x)")
        .unwrap();
    let err = import(&other, &env.store, |_| {}).unwrap_err();
    assert!(
        format!("{err:#}").contains("not a PouchDB library"),
        "{err:#}"
    );

    let missing = env.dir.path().join("missing.sqlite");
    let err = import(&missing, &env.store, |_| {}).unwrap_err();
    assert!(format!("{err:#}").contains("does not exist"), "{err:#}");
    assert!(!missing.exists(), "a missing source is not created");
}
