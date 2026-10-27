//! Example tests for the store's documented behaviour.

use base64::Engine as _;
use kanso_core::{DocType, Query};
use kanso_store::{
    ApplyOutcome, Attachment, BackupPolicy, Origin, Put, SearchParams, Sort, Store, StoreError,
    SyncDoc,
};
use serde_json::{Value, json};

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn create(store: &Store, doc_type: DocType, body: Value) -> String {
    store
        .put(doc_type, body, Put::Upsert, Origin::Api)
        .unwrap()
        .id
}

fn ids(store: &Store, query: &str) -> Vec<String> {
    let params = SearchParams {
        query: Query::parse(query),
        sort: Sort::Title,
        descending: false,
        ..SearchParams::default()
    };
    store
        .search(&params)
        .unwrap()
        .into_iter()
        .map(|d| d.id)
        .collect()
}

#[test]
fn creates_documents_with_defaults_and_derived_fields() {
    let (_dir, store) = store();
    let book = create(&store, DocType::Book, json!({"name": "Blog"}));
    let tag = create(
        &store,
        DocType::Tag,
        json!({"name": "rust", "color": "red"}),
    );
    let note = create(
        &store,
        DocType::Note,
        json!({"bookId": book, "title": "Hello", "body": "- [ ] a\n- [x] b", "tags": [tag]}),
    );
    assert!(note.starts_with("note:"));

    let doc = store.get_api(&note, None, false).unwrap();
    assert_eq!(doc["_id"], note);
    assert!(doc["_rev"].as_str().unwrap().starts_with("1-"));
    assert_eq!(doc["status"], "none");
    assert_eq!(doc["share"], "private");
    assert_eq!(doc["pinned"], false);
    assert_eq!(doc["doctype"], "markdown");
    assert_eq!(doc["numOfTasks"], 2);
    assert_eq!(doc["numOfCheckedTasks"], 1);
    assert!(doc["createdAt"].as_i64().unwrap() > 0);

    assert_eq!(store.get_api(&book, None, false).unwrap()["count"], 1);
    assert_eq!(store.get_api(&tag, None, false).unwrap()["count"], 1);
    assert_eq!(store.tags().unwrap()[0].count, 1);
}

#[test]
fn creating_with_a_given_id_keeps_it() {
    let (_dir, store) = store();
    create(
        &store,
        DocType::Book,
        json!({"_id": "book:inbox", "name": "Inbox"}),
    );
    let id = create(
        &store,
        DocType::Note,
        json!({"_id": "note:abc-1_2", "bookId": "book:inbox"}),
    );
    assert_eq!(id, "note:abc-1_2");
}

#[test]
fn rejects_invalid_references_with_the_api_messages() {
    let (_dir, store) = store();
    let err = store
        .put(
            DocType::Note,
            json!({"_id": "note:x1", "bookId": "book:nope"}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap_err();
    assert_eq!(err.to_string(), "The note 'note:x1' has an invalid bookId.");
    let book = create(&store, DocType::Book, json!({"name": "B"}));
    let err = store
        .put(
            DocType::Note,
            json!({"bookId": book, "tags": ["tag:none"]}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap_err();
    assert!(matches!(err, StoreError::Invalid(m) if m.contains("invalid tag 'tag:none'")));
    // The trash pseudo-book is always valid.
    create(&store, DocType::Note, json!({"bookId": "trash"}));
}

#[test]
fn replacing_requires_the_current_rev() {
    let (_dir, store) = store();
    let book = create(&store, DocType::Book, json!({"name": "B"}));
    let created = store
        .put(
            DocType::Note,
            json!({"bookId": book, "title": "v1"}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();

    let stale =
        json!({"_id": created.id, "_rev": "1-00000000000000000000000000000000", "bookId": book});
    assert!(matches!(
        store.put(DocType::Note, stale, Put::Upsert, Origin::Api),
        Err(StoreError::Conflict)
    ));
    let missing = json!({"_id": created.id, "bookId": book});
    assert!(matches!(
        store.put(DocType::Note, missing, Put::Upsert, Origin::Api),
        Err(StoreError::Conflict)
    ));

    // update-note's flow: GET, merge client-side, POST the whole document back.
    let mut doc = store
        .get_api(&created.id, Some(&created.rev), false)
        .unwrap();
    doc["title"] = "v2".into();
    let updated = store
        .put(DocType::Note, doc.clone(), Put::Upsert, Origin::Api)
        .unwrap();
    assert!(updated.rev.starts_with("2-"));
    assert_eq!(store.note(&created.id).unwrap().unwrap().title, "v2");

    // GET with an old rev finds nothing.
    assert!(matches!(
        store.get_api(&created.id, Some(&created.rev), false),
        Err(StoreError::NotFound)
    ));

    // Writing identical content keeps the revision.
    let mut same = store.get_api(&created.id, None, false).unwrap();
    same["numOfTasks"] = 99.into(); // derived fields are recomputed, not taken
    let again = store
        .put(DocType::Note, same, Put::Upsert, Origin::Api)
        .unwrap();
    assert_eq!(again.rev, updated.rev);
}

#[test]
fn merge_updates_fields_and_checks_rev_only_when_given() {
    let (_dir, store) = store();
    let parent = create(&store, DocType::Book, json!({"name": "Parent"}));
    let child = store
        .put(
            DocType::Book,
            json!({"name": "Child", "parentBookId": parent}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();

    let renamed = store
        .put(
            DocType::Book,
            json!({"_id": child.id, "name": "Renamed"}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
    let doc = store.get_api(&child.id, None, false).unwrap();
    assert_eq!(doc["name"], "Renamed");
    assert_eq!(doc["parentBookId"], parent);

    let stale = json!({"_id": child.id, "_rev": child.rev, "name": "X"});
    assert!(matches!(
        store.put(DocType::Book, stale, Put::Merge, Origin::Api),
        Err(StoreError::Conflict)
    ));
    let current = json!({"_id": child.id, "_rev": renamed.rev, "parentBookId": null});
    store
        .put(DocType::Book, current, Put::Merge, Origin::Api)
        .unwrap();
    assert_eq!(
        store.get_api(&child.id, None, false).unwrap()["parentBookId"],
        Value::Null
    );

    // A notebook can't be nested inside its own descendant.
    let cycle = json!({"_id": parent, "parentBookId": child.id});
    store
        .put(
            DocType::Book,
            json!({"_id": child.id, "parentBookId": parent}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
    assert!(matches!(
        store.put(DocType::Book, cycle, Put::Merge, Origin::Api),
        Err(StoreError::Invalid(_))
    ));

    assert!(matches!(
        store.put(
            DocType::Tag,
            json!({"_id": "tag:missing", "name": "x"}),
            Put::Merge,
            Origin::Api
        ),
        Err(StoreError::NotFound)
    ));
}

#[test]
fn deleting_a_book_trashes_its_notes_and_removes_sub_books() {
    let (_dir, store) = store();
    let parent = create(&store, DocType::Book, json!({"name": "Parent"}));
    let child = create(
        &store,
        DocType::Book,
        json!({"name": "Child", "parentBookId": parent}),
    );
    let n1 = create(
        &store,
        DocType::Note,
        json!({"bookId": parent, "title": "one"}),
    );
    let n2 = create(
        &store,
        DocType::Note,
        json!({"bookId": child, "title": "two"}),
    );

    store.delete(&parent, Origin::Api).unwrap();
    assert!(store.get(&parent).unwrap().is_none());
    assert!(store.get(&child).unwrap().is_none());
    assert_eq!(store.note(&n1).unwrap().unwrap().book_id, "trash");
    assert_eq!(store.note(&n2).unwrap().unwrap().book_id, "trash");
    assert!(ids(&store, "").is_empty());
    assert_eq!(ids(&store, "bookId:trash").len(), 2);
    assert_eq!(store.status_counts().unwrap().trash, 2);
}

#[test]
fn deleting_a_tag_removes_it_from_notes() {
    let (_dir, store) = store();
    let book = create(&store, DocType::Book, json!({"name": "B"}));
    let tag = create(&store, DocType::Tag, json!({"name": "t"}));
    let note = create(
        &store,
        DocType::Note,
        json!({"bookId": book, "tags": [tag]}),
    );
    store.delete(&tag, Origin::Api).unwrap();
    assert!(store.note(&note).unwrap().unwrap().tags.is_empty());
    assert!(matches!(
        store.delete(&tag, Origin::Api),
        Err(StoreError::NotFound)
    ));
}

#[test]
fn search_follows_the_api_syntax() {
    let (_dir, store) = store();
    let blog = create(&store, DocType::Book, json!({"name": "Desktop App"}));
    let other = create(
        &store,
        DocType::Book,
        json!({"_id": "book:kGlLniaV", "name": "Other"}),
    );
    let js = create(&store, DocType::Tag, json!({"name": "JavaScript"}));
    let a = create(
        &store,
        DocType::Note,
        json!({"bookId": blog, "title": "Sprint 10.0", "body": "Debounce the input", "tags": [js], "status": "active"}),
    );
    let b = create(
        &store,
        DocType::Note,
        json!({"bookId": other, "title": "Links", "body": "see inkdrop://note/t0iLc_TS ok", "status": "onHold"}),
    );
    let c = create(
        &store,
        DocType::Note,
        json!({"bookId": other, "title": "Go", "body": "short words"}),
    );
    let trashed = create(
        &store,
        DocType::Note,
        json!({"bookId": "trash", "title": "Old debounce"}),
    );

    assert_eq!(ids(&store, "debounce"), vec![a.clone()]);
    assert_eq!(ids(&store, r#"title:"sprint 10""#), vec![a.clone()]);
    assert_eq!(ids(&store, r#"book:"desktop app""#), vec![a.clone()]);
    assert_eq!(ids(&store, "bookId:kGlLniaV"), vec![c.clone(), b.clone()]);
    assert_eq!(ids(&store, "tag:javascript"), vec![a.clone()]);
    assert_eq!(ids(&store, &format!("tagId:{js}")), vec![a.clone()]);
    assert_eq!(ids(&store, "status:onhold"), vec![b.clone()]);
    assert_eq!(ids(&store, "-bookId:kGlLniaV"), vec![a.clone()]);
    assert_eq!(ids(&store, "inkdrop://note/t0iLc_TS"), vec![b.clone()]);
    assert_eq!(ids(&store, "Go"), vec![c.clone()]); // shorter than a trigram
    assert_eq!(ids(&store, "bookId:trash"), vec![trashed]);
    assert!(ids(&store, "t0iLc%TS").is_empty()); // `%` and `_` are not wildcards

    let params = SearchParams {
        sort: Sort::Title,
        descending: true,
        limit: Some(2),
        skip: 1,
        ..SearchParams::default()
    };
    let titles: Vec<String> = store
        .search_summaries(&params)
        .unwrap()
        .into_iter()
        .map(|s| s.title)
        .collect();
    assert_eq!(titles, vec!["Links", "Go"]);
}

/// Sorted by title, notes without one ("Untitled" in the list) come after
/// the named ones, whichever the direction.
#[test]
fn blank_titles_sort_last() {
    let (_dir, store) = store();
    let book = create(&store, DocType::Book, json!({"name": "B"}));
    for title in ["", "b", "A", " "] {
        create(
            &store,
            DocType::Note,
            json!({"bookId": book, "title": title}),
        );
    }
    let titles = |descending: bool| -> Vec<String> {
        let params = SearchParams {
            sort: Sort::Title,
            descending,
            ..SearchParams::default()
        };
        store
            .search_summaries(&params)
            .unwrap()
            .into_iter()
            .map(|s| s.title)
            .collect()
    };
    let ascending = titles(false);
    assert_eq!(ascending[..2], ["A", "b"]);
    assert!(ascending[2..].iter().all(|t| t.trim().is_empty()));
    let descending = titles(true);
    assert_eq!(descending[..2], ["b", "A"]);
    assert!(descending[2..].iter().all(|t| t.trim().is_empty()));
}

#[test]
fn recent_notes_are_counted_without_the_trash() {
    let (_dir, store) = store();
    let book = create(&store, DocType::Book, json!({"name": "B"}));
    for (at, book_id) in [(100, book.as_str()), (200, book.as_str()), (300, "trash")] {
        create(
            &store,
            DocType::Note,
            json!({"bookId": book_id, "title": "n", "updatedAt": at}),
        );
    }
    assert_eq!(store.count_recent(0).unwrap(), 2);
    assert_eq!(store.count_recent(150).unwrap(), 1);
    assert_eq!(store.count_recent(201).unwrap(), 0);
}

/// A search's snippet starts near the match, even far into a long note;
/// without a search it is the note's start.
#[test]
fn search_snippets_show_the_match() {
    let (_dir, store) = store();
    let book = create(&store, DocType::Book, json!({"name": "B"}));
    let body = format!(
        "Intro.\n\n{}\n\nThe Sync engine pushes changes.",
        "x ".repeat(1500)
    );
    create(
        &store,
        DocType::Note,
        json!({"bookId": book, "title": "Far", "body": body}),
    );
    let near = "We sync on launch.";
    create(
        &store,
        DocType::Note,
        json!({"bookId": book, "title": "Near", "body": near}),
    );
    let snippets = |query: &str| -> Vec<(String, String)> {
        let params = SearchParams {
            query: Query::parse(query),
            sort: Sort::Title,
            descending: false,
            ..SearchParams::default()
        };
        store
            .search_summaries(&params)
            .unwrap()
            .into_iter()
            .map(|s| (s.title, s.snippet))
            .collect()
    };
    let found = snippets("sync");
    assert_eq!(found[0].0, "Far");
    assert_eq!(found[0].1, "…The Sync engine pushes changes.");
    assert_eq!(found[1].1, near);
    // Title-only and negated words don't move the snippet.
    let titled = snippets("title:far");
    assert!(titled[0].1.starts_with("Intro. x x"), "{}", titled[0].1);
}

#[test]
fn files_store_bytes_and_return_them_on_request() {
    let (_dir, store) = store();
    let bytes = b"\x89PNG\r\n\x1a\nfake".to_vec();
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let created = store
        .put(
            DocType::File,
            json!({"name": "a.png", "contentType": "image/png", "contentLength": 999, "publicIn": [],
                   "_attachments": {"index": {"content_type": "image/png", "data": encoded}}}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();
    let stub = store.get_api(&created.id, None, false).unwrap();
    assert_eq!(stub["contentLength"], bytes.len());
    assert_eq!(stub["_attachments"]["index"]["stub"], true);
    assert_eq!(
        stub["_attachments"]["index"]["digest"],
        Attachment::new(bytes.clone()).digest()
    );
    let full = store.get_api(&created.id, None, true).unwrap();
    assert_eq!(full["_attachments"]["index"]["data"], encoded);
    assert_eq!(
        store.attachment(&created.id).unwrap().unwrap().1.data,
        bytes
    );

    let bad = json!({"name": "x.exe", "contentType": "application/x", "_attachments": {"index": {"data": encoded}}});
    assert!(
        store
            .put(DocType::File, bad, Put::Upsert, Origin::Api)
            .is_err()
    );
}

#[test]
fn files_take_pdfs_and_still_refuse_other_documents() {
    let (_dir, store) = store();
    let data = base64::engine::general_purpose::STANDARD.encode(b"%PDF-1.7 fake");
    let file = |content_type: &str| {
        json!({"name": "a.pdf", "contentType": content_type,
               "_attachments": {"index": {"content_type": content_type, "data": data}}})
    };
    let pdf = store
        .put(
            DocType::File,
            file("application/pdf"),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();
    let (content_type, blob) = store.attachment(&pdf.id).unwrap().unwrap();
    assert_eq!(content_type, "application/pdf");
    assert_eq!(blob.data, b"%PDF-1.7 fake");
    for other in ["application/msword", "application/zip", "text/html"] {
        let err = store
            .put(DocType::File, file(other), Put::Upsert, Origin::Api)
            .unwrap_err();
        assert!(matches!(err, StoreError::Invalid(_)), "{other}: {err}");
    }
}

#[test]
fn api_writes_are_snapshotted_and_ui_autosaves_throttled() {
    let (_dir, store) = store();
    let book = create(&store, DocType::Book, json!({"name": "B"}));
    let note = create(&store, DocType::Note, json!({"bookId": book, "body": "v1"}));
    for (i, origin) in [Origin::Ui, Origin::Ui, Origin::Api]
        .into_iter()
        .enumerate()
    {
        let mut doc = store.get_api(&note, None, false).unwrap();
        doc["body"] = format!("v{}", i + 2).into();
        store.put(DocType::Note, doc, Put::Upsert, origin).unwrap();
    }
    let bodies: Vec<String> = store
        .revisions(&note)
        .unwrap()
        .into_iter()
        .map(|r| r.fields["body"].as_str().unwrap().to_owned())
        .collect();
    // v1 by the first UI save, v2's UI snapshot skipped, v3 by the API write.
    assert_eq!(bodies, vec!["v3", "v1"]);
}

#[test]
fn sync_bookkeeping_and_conflicts() {
    let (_dir, store) = store();
    let book = create(&store, DocType::Book, json!({"name": "B"}));
    let note = store
        .put(
            DocType::Note,
            json!({"bookId": book, "body": "local", "updatedAt": 100}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();
    let pending = store.pending_push(10).unwrap();
    assert_eq!(pending.len(), 2);
    assert!(pending.iter().all(|d| d.base_rev.is_none()));
    for doc in &pending {
        store.mark_synced(&doc.id, &doc.rev, None).unwrap();
    }
    assert!(store.pending_push(10).unwrap().is_empty());

    // A clean local copy takes the remote version.
    let remote = |rev: &str, body: &str, updated: i64| SyncDoc {
        id: note.id.clone(),
        rev: rev.into(),
        deleted: false,
        fields: json!({"bookId": book, "body": body, "updatedAt": updated})
            .as_object()
            .unwrap()
            .clone(),
        base_rev: None,
    };
    assert_eq!(
        store
            .apply_remote(&remote("2-aa", "remote", 200), None, None)
            .unwrap(),
        ApplyOutcome::Applied
    );
    assert_eq!(store.note(&note.id).unwrap().unwrap().body, "remote");
    assert_eq!(
        store
            .apply_remote(&remote("2-aa", "remote", 200), None, None)
            .unwrap(),
        ApplyOutcome::Unchanged
    );
    assert!(store.pending_push(10).unwrap().is_empty());

    // Both changed: the newer edit wins and the loser is kept as history.
    let mut doc = store.get_api(&note.id, None, false).unwrap();
    doc["body"] = "mine".into();
    doc["updatedAt"] = 300.into();
    store
        .put(DocType::Note, doc, Put::Upsert, Origin::Ui)
        .unwrap();
    assert!(matches!(
        store
            .apply_remote(&remote("3-bb", "theirs", 250), None, None)
            .unwrap(),
        ApplyOutcome::LocalWon { .. }
    ));
    assert_eq!(store.note(&note.id).unwrap().unwrap().body, "mine");
    let pending = store.pending_push(10).unwrap();
    assert_eq!(pending[0].base_rev.as_deref(), Some("3-bb"));
    assert!(
        store
            .revisions(&note.id)
            .unwrap()
            .iter()
            .any(|r| r.fields["body"] == "theirs")
    );

    assert_eq!(
        store
            .apply_remote(&remote("4-cc", "newest", 400), None, None)
            .unwrap(),
        ApplyOutcome::RemoteWon
    );
    assert_eq!(store.note(&note.id).unwrap().unwrap().body, "newest");
    assert!(
        store
            .revisions(&note.id)
            .unwrap()
            .iter()
            .any(|r| r.fields["body"] == "mine")
    );
}

#[test]
fn backups_are_made_when_due_and_rotated() {
    let (dir, store) = store();
    let policy = BackupPolicy {
        dir: dir.path().join("backups"),
        keep: 2,
        min_interval_ms: 1000,
    };
    assert!(store.backup_if_due(&policy, 10_000).unwrap().is_some());
    assert!(store.backup_if_due(&policy, 10_500).unwrap().is_none());
    assert!(store.backup_if_due(&policy, 11_000).unwrap().is_some());
    let newest = store.backup_if_due(&policy, 12_000).unwrap().unwrap();
    assert_eq!(std::fs::read_dir(&policy.dir).unwrap().count(), 2);
    let copy = Store::open(&newest).unwrap();
    assert_eq!(copy.counts().unwrap(), store.counts().unwrap());
}

/// A backup made with "Back up now" is kept beside fourteen daily ones:
/// it doesn't push the oldest daily one out, isn't pruned itself, and
/// doesn't stand in for the next daily backup.
#[test]
fn manual_backups_stay_out_of_the_daily_rotation() {
    let (dir, store) = store();
    let day = 24 * 60 * 60 * 1000;
    let policy = BackupPolicy::daily(dir.path().join("backups"));
    let start = 1_000 * day;
    for n in 0..14 {
        assert!(
            store
                .backup_if_due(&policy, start + n * day)
                .unwrap()
                .is_some()
        );
    }
    let oldest = policy.dir.join(format!("kanso-{start}.db"));
    assert!(oldest.exists());
    // An hour after the last daily one, then the day after it.
    let manual = store
        .backup_now(&policy.dir, start + 13 * day + 3_600_000)
        .unwrap();
    assert!(manual.exists());
    let daily = store
        .backup_if_due(&policy, start + 14 * day)
        .unwrap()
        .expect("the manual backup doesn't stand in for the daily one");
    assert!(manual.exists(), "the manual backup is kept");
    assert!(daily.exists());
    assert!(!oldest.exists(), "fourteen daily backups are kept");
    assert_eq!(std::fs::read_dir(&policy.dir).unwrap().count(), 15);
    assert_eq!(
        Store::open(&manual).unwrap().counts().unwrap(),
        store.counts().unwrap()
    );
}

#[test]
fn imports_keep_revisions_and_are_idempotent() {
    let (_dir, store) = store();
    let fields = json!({"title": "T", "body": "B", "bookId": "book:gone", "createdAt": 5, "updatedAt": 6, "count": 3})
        .as_object()
        .unwrap()
        .clone();
    assert!(
        store
            .import_doc(DocType::Note, "note:imp1", "7-abc", fields.clone(), None)
            .unwrap()
    );
    assert!(
        !store
            .import_doc(DocType::Note, "note:imp1", "7-abc", fields, None)
            .unwrap()
    );
    let doc = store.get_api("note:imp1", None, false).unwrap();
    assert_eq!(doc["_rev"], "7-abc");
    assert_eq!(doc["createdAt"], 5);
    assert!(doc.get("count").is_none());
}

#[test]
fn subscribers_see_committed_changes() {
    let (_dir, store) = store();
    let changes = store.subscribe();
    let book = create(&store, DocType::Book, json!({"name": "B"}));
    let change = changes.try_recv().unwrap();
    assert_eq!(change.id, book);
    assert_eq!(change.origin, Origin::Api);
    assert!(changes.try_recv().is_err());
    let failed = store.put(
        DocType::Note,
        json!({"bookId": "book:none"}),
        Put::Upsert,
        Origin::Api,
    );
    assert!(failed.is_err());
    assert!(changes.try_recv().is_err());
}

#[test]
fn recorded_revisions_show_up_in_history() {
    let (_dir, store) = store();
    let book = create(&store, DocType::Book, json!({"name": "B"}));
    let note = create(
        &store,
        DocType::Note,
        json!({"bookId": book, "title": "T", "body": "mine"}),
    );
    let fields = json!({"title": "T", "body": "unsaved draft"})
        .as_object()
        .unwrap()
        .clone();
    store
        .record_revision(&note, "1-draft", &fields, Origin::Ui)
        .unwrap();
    let history = store.revisions(&note).unwrap();
    assert_eq!(history[0].fields["body"], "unsaved draft");
    assert_eq!(history[0].origin, "ui");
    // One kept as of an earlier time lists by that time, behind the newer.
    store
        .record_revision_at(&note, "0-old", &fields, Origin::Sync, 5_000)
        .unwrap();
    let history = store.revisions(&note).unwrap();
    let last = history.last().unwrap();
    assert_eq!((last.rev.as_str(), last.saved_at), ("0-old", 5_000));
    assert_eq!(last.origin, "sync");
}

#[test]
fn reimport_keeps_kanso_edits_and_takes_newer_source_edits() {
    let (_dir, store) = store();
    let book = create(&store, DocType::Book, json!({"name": "B"}));
    let fields = |body: &str| {
        json!({"title": "T", "body": body, "bookId": book, "createdAt": 1, "updatedAt": 2})
            .as_object()
            .unwrap()
            .clone()
    };
    assert!(
        store
            .import_doc(DocType::Note, "note:n1", "3-aaa", fields("imported"), None)
            .unwrap()
    );
    // Edited in Kanso: generation 4.
    let edited = store
        .put(
            DocType::Note,
            json!({"_id": "note:n1", "body": "kanso"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert!(edited.rev.starts_with("4-"));
    // Re-importing the old source version leaves the Kanso edit alone.
    assert!(
        !store
            .import_doc(DocType::Note, "note:n1", "3-aaa", fields("imported"), None)
            .unwrap()
    );
    assert_eq!(store.note("note:n1").unwrap().unwrap().body, "kanso");
    // A newer source edit wins, and the Kanso version stays in history.
    assert!(
        store
            .import_doc(
                DocType::Note,
                "note:n1",
                "5-bbb",
                fields("imported again"),
                None
            )
            .unwrap()
    );
    assert_eq!(
        store.note("note:n1").unwrap().unwrap().body,
        "imported again"
    );
    let history = store.revisions("note:n1").unwrap();
    assert_eq!(history[0].fields["body"], "kanso");
    assert_eq!(history[0].origin, "import");
}

#[test]
fn merge_many_changes_every_note_or_none() {
    let (_dir, store) = store();
    let book = create(&store, DocType::Book, json!({"name": "Blog"}));
    let other = create(&store, DocType::Book, json!({"name": "Work"}));
    let a = create(&store, DocType::Note, json!({"bookId": book, "title": "a"}));
    let b = create(&store, DocType::Note, json!({"bookId": book, "title": "b"}));
    let written = store
        .merge_many(
            DocType::Note,
            vec![
                json!({"_id": a, "bookId": other, "pinned": true}),
                json!({"_id": b, "bookId": other}),
            ],
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(written.len(), 2);
    assert_eq!(store.note(&a).unwrap().unwrap().book_id, other);
    assert!(store.note(&a).unwrap().unwrap().pinned);
    assert_eq!(
        store.note(&a).unwrap().unwrap().title,
        "a",
        "a merge keeps the rest"
    );
    assert_eq!(store.note(&b).unwrap().unwrap().book_id, other);

    // One bad merge and the others don't happen either.
    let refused = store.merge_many(
        DocType::Note,
        vec![
            json!({"_id": a, "bookId": book}),
            json!({"_id": "note:missing", "bookId": book}),
        ],
        Origin::Ui,
    );
    assert!(refused.is_err());
    assert_eq!(store.note(&a).unwrap().unwrap().book_id, other);
}
