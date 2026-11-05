//! Example tests for `sync` over a `MemoryServer` account, sealed end to
//! end: one per conflict path, files, failures, re-merge, repair, quotas and
//! pauses, and the edge cases the property tests generalize
//! (docs/ACCOUNTS.md › Sync with encryption). The server's state is read
//! through `PlainView`, never directly.

mod common;

use std::io::Cursor;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use base64::Engine as _;
use common::*;
use kanso_core::DocType;
use kanso_store::SyncDoc;
use kanso_sync::{
    ApiError, Choice, MemoryServer, PushResult, QuotaKind, QuotaOverride, Route, SyncError,
    SyncOptions, SyncProblem, SyncReport, md5_hex, meta_keys, report_problem,
};
use serde_json::json;

const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::URL_SAFE_NO_PAD;

fn generation(rev: &str) -> u64 {
    rev.parse::<kanso_core::Rev>().unwrap().generation
}

fn hash(rev: &str) -> String {
    rev.parse::<kanso_core::Rev>().unwrap().hash
}

/// Two Macs of `alice` that both have `note:n1` in `book:b1`, synced.
fn two_synced() -> (MemoryServer, Mac, Mac) {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    book(&a.store, "book:b1", "Inbox");
    note(&a.store, "note:n1", "book:b1", "first", 100);
    a.sync(&server);
    b.sync(&server);
    assert_eq!(body(&b.store, "note:n1").as_deref(), Some("first"));
    (server, a, b)
}

/// Rewrites the sealed text of a feed document: `f` gets the raw bytes.
fn reseal(sealed: &str, f: impl FnOnce(&mut Vec<u8>)) -> String {
    let mut raw = B64.decode(sealed).unwrap();
    f(&mut raw);
    B64.encode(raw)
}

#[test]
fn a_note_created_on_one_device_reaches_the_other() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    book(&a.store, "book:b1", "Inbox");
    let created = note(&a.store, "note:n1", "book:b1", "hello", 100);

    let report = a.sync(&server);
    assert_eq!((report.pushed, report.pulled, report.pending), (2, 0, 0));
    assert_eq!(server_docs(&server, &a.session), all_docs(&a.store));

    let report = b.sync(&server);
    assert_eq!((report.pushed, report.pulled), (0, 2));
    assert_eq!(body(&b.store, "note:n1").as_deref(), Some("hello"));
    assert_eq!(b.store.get("note:n1").unwrap().unwrap().rev, created.rev);
    assert_eq!(all_docs(&a.store), all_docs(&b.store));
    let head = server.plain_view(&a.session).docs().last().unwrap().seq;
    assert_eq!(b.cursor(), head);
    // A's cursor catches up with its own pushes at its next pull.
    assert_eq!(a.cursor(), 0);
    assert_eq!(a.sync(&server), SyncReport::default());
    assert_eq!(a.cursor(), head);
}

#[test]
fn syncing_an_up_to_date_store_does_nothing() {
    let (server, a, b) = two_synced();
    a.sync(&server);
    let before = (
        all_docs(&a.store),
        server_docs(&server, &a.session),
        server.stats().count(Route::Push),
    );
    assert_eq!(a.sync(&server), SyncReport::default());
    assert_eq!(b.sync(&server), SyncReport::default());
    let after = (
        all_docs(&a.store),
        server_docs(&server, &a.session),
        server.stats().count(Route::Push),
    );
    assert_eq!(after, before);
}

#[test]
fn newer_local_edit_wins_with_a_higher_generation_and_the_remote_one_is_kept() {
    let (server, a, b) = two_synced();
    let theirs = edit_note(&a.store, "note:n1", "from A", Some(200));
    a.sync(&server);
    let mine = edit_note(&b.store, "note:n1", "from B", Some(300));

    let report = b.sync(&server);
    assert_eq!((report.conflicts_local, report.conflicts_remote), (1, 0));
    assert_eq!(report.pushed, 1);
    assert_eq!(report.conflicted, vec!["note:n1".to_owned()]);
    assert_eq!(body(&b.store, "note:n1").as_deref(), Some("from B"));
    assert!(has_revision(&b.store, "note:n1", &theirs.rev));
    // M2: the winning version beats both generations, with the same content.
    let won = b.store.get("note:n1").unwrap().unwrap().rev;
    assert_eq!(
        generation(&won),
        generation(&theirs.rev).max(generation(&mine.rev)) + 1
    );
    assert_eq!(hash(&won), hash(&mine.rev));

    a.sync(&server);
    assert_eq!(body(&a.store, "note:n1").as_deref(), Some("from B"));
    assert!(has_revision(&a.store, "note:n1", &theirs.rev));
    assert_eq!(live_docs(&a.store), live_docs(&b.store));
    assert_eq!(server_docs(&server, &a.session), all_docs(&a.store));
}

#[test]
fn newer_remote_edit_wins_and_the_local_one_is_kept() {
    let (server, a, b) = two_synced();
    edit_note(&a.store, "note:n1", "from A", Some(300));
    a.sync(&server);
    let mine = edit_note(&b.store, "note:n1", "from B", Some(200));

    let report = b.sync(&server);
    assert_eq!(report.conflicts_remote, 1);
    assert_eq!(report.pushed, 0);
    assert_eq!(body(&b.store, "note:n1").as_deref(), Some("from A"));
    assert!(has_revision(&b.store, "note:n1", &mine.rev));
    assert_eq!(pending(&b.store), 0);
    assert_eq!(live_docs(&a.store), live_docs(&b.store));
}

#[test]
fn a_conflict_found_by_the_push_is_resolved_in_the_same_sync() {
    let (server, a, b) = two_synced();
    let theirs = edit_note(&a.store, "note:n1", "from A", Some(200));
    edit_note(&b.store, "note:n1", "from B", Some(300));
    // A pushes after B pulled but before B pushes.
    let mut first = true;
    let tap = Tap::new(&server, |event| {
        if let Event::Push(_) = event
            && std::mem::take(&mut first)
        {
            a.sync(&server);
        }
        Ok(())
    });

    let report = b.sync(&tap);
    assert_eq!(report.conflicts_local, 1);
    assert_eq!(
        report.pushed, 1,
        "the winning edit goes out in the second round"
    );
    assert_eq!(report.pending, 0);
    assert!(has_revision(&b.store, "note:n1", &theirs.rev));
    a.sync(&server);
    assert_eq!(body(&a.store, "note:n1").as_deref(), Some("from B"));
}

#[test]
fn an_edit_beats_a_deletion() {
    let (server, a, b) = two_synced();
    a.store.delete("note:n1", kanso_store::Origin::Api).unwrap();
    a.sync(&server);
    edit_note(&b.store, "note:n1", "still needed", Some(50));

    let report = b.sync(&server);
    assert_eq!(report.conflicts_local, 1);
    assert_eq!(body(&b.store, "note:n1").as_deref(), Some("still needed"));
    a.sync(&server);
    assert_eq!(body(&a.store, "note:n1").as_deref(), Some("still needed"));
    assert_eq!(all_docs(&a.store), all_docs(&b.store));
}

#[test]
fn a_deletion_reaches_a_clean_copy() {
    let (server, a, b) = two_synced();
    a.store.delete("note:n1", kanso_store::Origin::Api).unwrap();
    a.sync(&server);
    let report = b.sync(&server);
    assert_eq!(report.pulled, 1);
    assert_eq!(report.conflicts(), 0);
    assert!(b.store.get("note:n1").unwrap().is_none());
    assert_eq!(all_docs(&a.store), all_docs(&b.store));
}

#[test]
fn files_travel_sealed_with_their_bytes_and_deletions_drop_them() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    let png = b"\x89PNG\r\n\x1a\n not really".to_vec();
    file(&a.store, "file:img1", &png);

    let report = a.sync(&server);
    assert_eq!(report.files_up, 1);
    let view = server.plain_view(&a.session);
    assert_eq!(view.file("file:img1"), Some(png.clone()));

    let report = b.sync(&server);
    assert_eq!(report.files_down, 1);
    let (content_type, blob) = b.store.attachment("file:img1").unwrap().unwrap();
    assert_eq!(content_type, "image/png");
    assert_eq!(blob.data, png);

    // B already has the bytes of a renamed file and doesn't download them.
    merge(
        &a.store,
        DocType::File,
        json!({"_id": "file:img1", "name": "renamed.png"}),
    )
    .unwrap();
    a.sync(&server);
    let report = b.sync(&server);
    assert_eq!((report.pulled, report.files_down), (1, 0));
    assert_eq!(
        b.store.file_meta("file:img1").unwrap().unwrap().name,
        "renamed.png"
    );

    a.store
        .delete("file:img1", kanso_store::Origin::Api)
        .unwrap();
    a.sync(&server);
    assert_eq!(view.file("file:img1"), None);
    b.sync(&server);
    assert!(b.store.attachment("file:img1").unwrap().is_none());
}

#[test]
fn replaced_file_bytes_reach_the_other_device() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    file(&a.store, "file:img1", b"one");
    a.sync(&server);
    b.sync(&server);
    let mut doc = a.store.get_api("file:img1", None, false).unwrap();
    doc["_attachments"] = file_body("file:img1", "image.png", b"two")["_attachments"].clone();
    put(&a.store, DocType::File, doc).unwrap();
    a.sync(&server);
    let report = b.sync(&server);
    assert_eq!(report.files_down, 1);
    assert_eq!(
        b.store.attachment("file:img1").unwrap().unwrap().1.data,
        b"two"
    );
}

fn file_doc(id: &str, rev: &str, bytes: &[u8]) -> SyncDoc {
    SyncDoc {
        id: id.into(),
        rev: rev.into(),
        deleted: false,
        fields: json!({"name": "late.png", "contentType": "image/png", "md5digest": md5_hex(bytes), "createdAt": 1})
            .as_object()
            .unwrap()
            .clone(),
        base_rev: None,
    }
}

#[test]
fn a_missing_attachment_is_reported_and_fetched_by_a_later_sync() {
    let server = MemoryServer::new();
    let a = sign_up(&server, "alice", 1);
    let b = Mac::new(sign_in(&server, "alice", 2));
    let view = server.plain_view(&a);
    let bytes = b"late bytes".to_vec();
    // Another device pushed the document but its upload never arrived.
    view.push(&[file_doc("file:late", "1-ab", &bytes)]).unwrap();
    let report = b.sync(&server);
    assert_eq!((report.files_missing, report.files_down), (1, 0));
    assert_eq!(server.stats().count(Route::GetFile), 1, "asked once");
    assert!(b.store.get("file:late").unwrap().is_some());
    assert!(b.store.attachment("file:late").unwrap().is_none());
    let report = b.sync(&server);
    assert_eq!((report.files_missing, report.files_down), (1, 0));

    // Nothing new in the feed: the refetch missed it, so it waits a day
    // (review-verify 4), then an ordinary sync asks again.
    view.put_file("file:late", &bytes).unwrap();
    let report = b.sync(&server);
    assert_eq!((report.files_missing, report.files_down), (0, 0));
    assert_eq!(server.stats().count(Route::GetFile), 2, "it waits");
    a_day_later(&b.store);
    let report = b.sync(&server);
    assert_eq!((report.files_missing, report.files_down), (0, 1));
    assert!(!report.remerged);
    assert_eq!(
        b.store.attachment("file:late").unwrap().unwrap().1.data,
        bytes
    );
    let gets = server.stats().count(Route::GetFile);
    assert_eq!(b.sync(&server).files_down, 0);
    assert_eq!(
        server.stats().count(Route::GetFile),
        gets,
        "not asked again"
    );
}

#[test]
fn a_missing_attachment_is_fetched_by_a_full_resync() {
    let server = MemoryServer::new();
    let a = sign_up(&server, "alice", 1);
    let b = Mac::new(sign_in(&server, "alice", 2));
    let view = server.plain_view(&a);
    let bytes = b"late bytes".to_vec();
    view.push(&[file_doc("file:late", "1-ab", &bytes)]).unwrap();
    let off = SyncOptions {
        refetch_files: 0,
        ..SyncOptions::default()
    };
    assert_eq!(b.sync_with(&server, &off).unwrap().files_missing, 1);
    view.put_file("file:late", &bytes).unwrap();
    assert_eq!(b.sync_with(&server, &off).unwrap().files_down, 0);
    b.store.set_meta(meta_keys::LAST_SEQ, "0").unwrap();
    let report = b.sync_with(&server, &off).unwrap();
    assert_eq!(report.files_down, 1);
    assert!(!report.remerged, "a full resync isn't a restore");
    assert_eq!(
        b.store.attachment("file:late").unwrap().unwrap().1.data,
        bytes
    );
}

#[test]
fn opened_bytes_that_dont_match_the_md5_count_as_missing() {
    let server = MemoryServer::new();
    let a = sign_up(&server, "alice", 1);
    let b = Mac::new(sign_in(&server, "alice", 2));
    let view = server.plain_view(&a);
    view.push(&[file_doc("file:img", "1-ab", b"the right bytes")])
        .unwrap();
    view.put_file("file:img", b"other bytes").unwrap();
    let report = b.sync(&server);
    assert_eq!((report.files_missing, report.files_down), (1, 0));
    assert!(b.store.attachment("file:img").unwrap().is_none());
}

#[test]
fn a_lost_push_response_is_recovered_without_a_conflict() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    book(&a.store, "book:b1", "Inbox");
    note(&a.store, "note:n1", "book:b1", "hello", 100);
    server.drop_next_responses(Route::Push, 1);
    assert!(matches!(
        a.try_sync(&server),
        Err(SyncError::Api(ApiError::Transport(_)))
    ));
    assert_eq!(
        pending(&a.store),
        2,
        "nothing is marked synced without an answer"
    );

    let report = a.sync(&server);
    assert_eq!(report.conflicts(), 0);
    assert_eq!(
        report.pushed, 0,
        "the pull showed the server already has them"
    );
    assert_eq!(pending(&a.store), 0);
    assert!(a.store.revisions("note:n1").unwrap().is_empty());
}

#[test]
fn a_lost_upload_response_is_recovered_by_uploading_again() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    file(&a.store, "file:img1", b"bytes");
    server.drop_next_responses(Route::PutFile, 1);
    assert!(matches!(
        a.try_sync(&server),
        Err(SyncError::Api(ApiError::Transport(_)))
    ));
    assert_eq!(pending(&a.store), 1);
    let report = a.sync(&server);
    assert_eq!((report.files_up, report.pushed, report.pending), (1, 1, 0));
    assert_eq!(
        server.plain_view(&a.session).file("file:img1"),
        Some(b"bytes".to_vec())
    );
}

#[test]
fn a_document_the_server_lost_is_pushed_again_with_no_base() {
    let (server, a, _b) = two_synced();
    edit_note(&a.store, "note:n1", "edited", Some(500));
    // The server loses everything between A's pull and its push.
    let mut first = true;
    let tap = Tap::new(&server, |event| {
        if let Event::Push(_) = event
            && std::mem::take(&mut first)
        {
            server.rewind("alice", 0);
        }
        Ok(())
    });
    let report = a.sync(&tap);
    assert_eq!((report.pushed, report.pending), (1, 0));
    assert_eq!(server_live(&server, &a.session).len(), 1);
    // The next pull sees the account rewound and sends the book again.
    let report = a.sync(&server);
    assert!(report.remerged);
    assert_eq!(server_docs(&server, &a.session), all_docs(&a.store));
}

#[test]
fn an_edit_made_while_pushing_goes_out_next_time() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    book(&a.store, "book:b1", "Inbox");
    note(&a.store, "note:n1", "book:b1", "v1", 100);
    let mut first = true;
    let tap = Tap::new(&server, |event| {
        if let Event::Pushed(_) = event
            && std::mem::take(&mut first)
        {
            merge(
                &a.store,
                DocType::Note,
                json!({"_id": "note:n1", "body": "v2"}),
            )
            .unwrap();
        }
        Ok(())
    });
    let report = a.sync(&tap);
    assert_eq!((report.pending, report.conflicts()), (1, 0));

    let report = a.sync(&server);
    assert_eq!(
        report.conflicts(),
        0,
        "our own earlier push is not a conflict"
    );
    assert_eq!(report.pushed, 1);
    b.sync(&server);
    assert_eq!(body(&b.store, "note:n1").as_deref(), Some("v2"));
}

#[test]
fn our_own_push_coming_back_does_not_revert_a_later_merge() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    book(&a.store, "book:b1", "Inbox");
    note(&a.store, "note:n1", "book:b1", "v1", 100);
    a.sync(&server);
    merge(
        &a.store,
        DocType::Note,
        json!({"_id": "note:n1", "status": "active"}),
    )
    .unwrap();
    a.store.set_meta(meta_keys::LAST_SEQ, "0").unwrap();
    let report = a.sync(&server);
    assert_eq!(report.conflicts(), 0);
    assert_eq!(report.pushed, 1);
    assert_eq!(
        a.store.get_api("note:n1", None, false).unwrap()["status"],
        "active"
    );
}

#[test]
fn a_document_too_large_for_the_server_stays_pending_and_is_reported() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    book(&a.store, "book:b1", "Inbox");
    // 400k four-byte characters: 1.6 MB of fields JSON, over the 1,400,000
    // bytes the client pushes.
    note(&a.store, "note:big", "book:b1", &"😀".repeat(400_000), 100);
    note(&a.store, "note:small", "book:b1", "fine", 100);
    let report = a.sync(&server);
    assert_eq!(report.too_large, vec!["note:big".to_owned()]);
    assert_eq!((report.pushed, report.pending), (2, 1));
    // Said, not silent: the status names it instead of "Synced".
    assert_eq!(
        report_problem(&report),
        Some(SyncProblem::TooLarge { count: 1 })
    );
}

#[test]
fn a_pdf_over_ten_mib_syncs_to_the_server_and_to_another_device() {
    let (server, a, b) = two_synced();
    let mut pdf = b"%PDF-1.7".to_vec();
    pdf.resize(12 * 1024 * 1024, 0);
    put(
        &a.store,
        DocType::File,
        json!({
            "_id": "file:bigpdf", "name": "big.pdf", "contentType": "application/pdf",
            "createdAt": 1,
            "_attachments": {"index": {
                "content_type": "application/pdf",
                "data": base64::engine::general_purpose::STANDARD.encode(&pdf),
            }},
        }),
    )
    .unwrap();
    file(&a.store, "file:small", b"\x89PNGsmall");
    let report = a.sync(&server);
    assert!(report.too_large.is_empty(), "{:?}", report.too_large);
    assert_eq!(report.pending, 0);
    assert_eq!(report.pushed, 2);
    b.sync(&server);
    let (kind, got) = b.store.attachment("file:bigpdf").unwrap().unwrap();
    assert_eq!(kind, "application/pdf");
    assert_eq!(got.data, pdf);
}

#[test]
fn a_recording_and_timestamped_transcript_sync_play_and_seek_on_the_second_device() {
    let (server, a, b) = two_synced();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("in-progress");
    let mut options = kanso_audio::RecorderOptions::new(
        kanso_audio::Channels::Mono,
        vec![kanso_audio::RecordSource::Microphone],
    );
    options.id = Some("sync-recording".into());
    options.encode.max_part_frames = 200;
    let mut recorder = kanso_audio::Recorder::create(&root, options).unwrap();
    let pcm: Vec<f32> = (0..48_000 * 2)
        .map(|i| 0.35 * (std::f32::consts::TAU * 440.0 * i as f32 / 48_000.0).sin())
        .collect();
    recorder.push(&pcm).unwrap();
    let audio = recorder.finish().unwrap();
    assert_eq!(audio.parts.len(), 1);
    let bytes = std::fs::read(&audio.parts[0].0).unwrap();
    put(
        &a.store,
        DocType::File,
        json!({
            "_id": "file:recording-sync-1", "name": "Recording part 1.opus",
            "contentType": "audio/ogg", "createdAt": 1,
            "_attachments": {"index": {
                "content_type": "audio/ogg",
                "data": base64::engine::general_purpose::STANDARD.encode(&bytes),
            }},
        }),
    )
    .unwrap();
    note(
        &a.store,
        "note:recording-sync",
        "book:b1",
        "[Audio · 2 sec](inkdrop://file:recording-sync-1)\n\n<!-- kanso:transcript -->\n[00:01](kanso-audio://file:recording-sync-1?t=1) We agreed.\n<!-- /kanso:transcript -->",
        200,
    );

    a.sync(&server);
    b.sync(&server);
    let note = b.store.note("note:recording-sync").unwrap().unwrap();
    assert!(
        note.body
            .contains("kanso-audio://file:recording-sync-1?t=1")
    );
    assert!(note.body.contains("<!-- kanso:transcript -->"));
    let (kind, attachment) = b
        .store
        .attachment("file:recording-sync-1")
        .unwrap()
        .expect("the second device has the synced audio bytes");
    assert_eq!(kind, "audio/ogg");
    let mut player = kanso_audio::OpusReader::open(Cursor::new(attachment.data)).unwrap();
    assert_eq!(player.info().total_samples, pcm.len() as u64);
    player.seek(48_000).unwrap();
    assert_eq!(player.read_to_end().unwrap().len(), 48_000);

    // A file deletion is a sync tombstone; the other device drops the live
    // attachment and cannot keep playing bytes that the note no longer owns.
    a.store
        .delete("file:recording-sync-1", kanso_store::Origin::Api)
        .unwrap();
    a.sync(&server);
    b.sync(&server);
    assert!(
        b.store
            .attachment("file:recording-sync-1")
            .unwrap()
            .is_none()
    );
}

/// A server from before 50 MiB PDFs answers 413 `too_large` for the
/// upload: the PDF stays here, said as too large, and everything else
/// syncs, rather than every sync failing.
// no-prop-test: one refused upload against the fake server.
#[test]
fn a_pdf_an_older_server_refuses_stays_here_and_the_rest_syncs() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let mut pdf = b"%PDF-1.7".to_vec();
    pdf.resize(12 * 1024 * 1024, 0);
    put(
        &a.store,
        DocType::File,
        json!({
            "_id": "file:bigpdf", "name": "big.pdf", "contentType": "application/pdf",
            "createdAt": 1,
            "_attachments": {"index": {
                "content_type": "application/pdf",
                "data": base64::engine::general_purpose::STANDARD.encode(&pdf),
            }},
        }),
    )
    .unwrap();
    file(&a.store, "file:small", b"\x89PNGsmall");
    let old_server = Tap::new(&server, |event| match event {
        Event::PutFile { sealed, .. } if sealed.len() > 10 * 1024 * 1024 + 41 + 4_096 => {
            Err(ApiError::TooLarge)
        }
        _ => Ok(()),
    });
    let report = a.sync_with(&old_server, &SyncOptions::default()).unwrap();
    assert_eq!(report.too_large, vec!["file:bigpdf".to_owned()]);
    assert_eq!((report.pushed, report.pending), (1, 1));
    assert_eq!(
        report_problem(&report),
        Some(SyncProblem::TooLarge { count: 1 })
    );
    // Once the server takes it, it goes.
    let report = a.sync(&server);
    assert!(report.too_large.is_empty(), "{:?}", report.too_large);
    assert_eq!(report.pending, 0);
}

#[test]
fn the_cursor_is_saved_after_each_page() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    book(&a.store, "book:b1", "Inbox");
    for i in 0..4 {
        note(&a.store, &format!("note:n{i}"), "book:b1", "x", 100);
    }
    a.sync(&server);
    let options = SyncOptions {
        page_size: 2,
        ..SyncOptions::default()
    };
    let calls = AtomicUsize::new(0);
    let tap = Tap::new(&server, |event| match event {
        Event::Changes { .. } if calls.fetch_add(1, Ordering::SeqCst) == 2 => {
            Err(ApiError::Transport("offline (simulated)".into()))
        }
        _ => Ok(()),
    });
    assert!(b.sync_with(&tap, &options).is_err());
    assert_eq!(b.cursor(), 4, "two full pages were applied");

    let report = b.sync_with(&server, &options).unwrap();
    assert_eq!(report.pulled, 1);
    assert_eq!(all_docs(&a.store), all_docs(&b.store));
}

#[test]
fn pushes_are_split_into_batches() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    book(&a.store, "book:b1", "Inbox");
    for i in 0..9 {
        note(&a.store, &format!("note:n{i}"), "book:b1", "x", 100);
    }
    let options = SyncOptions {
        batch_docs: 4,
        ..SyncOptions::default()
    };
    let report = a.sync_with(&server, &options).unwrap();
    assert_eq!(report.pushed, 10);
    assert_eq!(server.stats().count(Route::Push), 3);
}

#[test]
fn a_broken_server_copy_of_the_synced_version_is_repaired_and_others_still_sync() {
    let (server, a, b) = two_synced();
    note(&a.store, "note:n2", "book:b1", "second", 100);
    a.sync(&server);
    let view = server.plain_view(&a.session);
    let broken = view.wire_id("note:n1");
    let before = b.store.get("note:n1").unwrap().unwrap().rev;
    // B pulls everything again, and the server's copy of n1, the version B
    // last synced, comes back damaged.
    b.store.set_meta(meta_keys::LAST_SEQ, "0").unwrap();
    let tap = Tap::new(&server, |event| {
        if let Event::Page(page) = event {
            for doc in page.docs.iter_mut().filter(|d| d.id == broken) {
                doc.sealed = reseal(&doc.sealed, |raw| raw[40] ^= 1);
            }
        }
        Ok(())
    });
    let report = b.sync(&tap);
    assert_eq!(report.repaired, vec!["note:n1".to_owned()]);
    assert!(report.skipped.is_empty());
    assert_eq!(report_problem(&report), None, "a repair isn't a problem");
    assert_eq!(body(&b.store, "note:n2").as_deref(), Some("second"));
    // The Mac's copy went up one generation higher, same content.
    let after = b.store.get("note:n1").unwrap().unwrap().rev;
    assert_eq!(generation(&after), generation(&before) + 1);
    assert_eq!(hash(&after), hash(&before));
    assert_eq!(server_docs(&server, &a.session)["note:n1"].0, after);
    assert_eq!(pending(&b.store), 0);
    a.sync(&server);
    assert_eq!(all_docs(&a.store), all_docs(&b.store));
}

#[test]
fn any_other_undecryptable_document_is_skipped_and_the_cursor_moves_past() {
    let (server, a, b) = two_synced();
    edit_note(&a.store, "note:n1", "newer", Some(300));
    note(&a.store, "note:n2", "book:b1", "second", 100);
    a.sync(&server);
    let view = server.plain_view(&a.session);
    let broken = view.wire_id("note:n1");
    let local_before = b.store.get("note:n1").unwrap().unwrap();
    let tap = Tap::new(&server, |event| {
        if let Event::Page(page) = event {
            for doc in page.docs.iter_mut().filter(|d| d.id == broken) {
                doc.sealed = reseal(&doc.sealed, |raw| raw[30] ^= 0x80);
            }
        }
        Ok(())
    });
    let report = b.sync(&tap);
    assert_eq!(report.skipped, vec![broken.clone()]);
    assert!(report.repaired.is_empty());
    assert_eq!(
        report_problem(&report),
        Some(SyncProblem::Skipped { count: 1 })
    );
    assert_eq!(b.store.get("note:n1").unwrap().unwrap(), local_before);
    assert_eq!(body(&b.store, "note:n2").as_deref(), Some("second"));
    let head = view.docs().last().unwrap().seq;
    assert_eq!(b.cursor(), head);
}

#[test]
fn a_newer_format_stops_the_pull_before_it_and_never_overwrites() {
    let (server, a, b) = two_synced();
    note(&a.store, "note:n2", "book:b1", "before", 100);
    a.sync(&server);
    edit_note(&a.store, "note:n1", "from a newer Kanso", Some(300));
    a.sync(&server);
    note(&a.store, "note:n3", "book:b1", "after", 100);
    a.sync(&server);
    let view = server.plain_view(&a.session);
    let newer = view.wire_id("note:n1");
    let seq_before: i64 = view.docs().iter().find(|d| d.id == "note:n2").unwrap().seq;
    note(&b.store, "note:mine", "book:b1", "from B", 100);
    let tap = Tap::new(&server, |event| {
        if let Event::Page(page) = event {
            for doc in page.docs.iter_mut().filter(|d| d.id == newer) {
                doc.sealed = reseal(&doc.sealed, |raw| raw[0] = 2);
            }
        }
        Ok(())
    });
    let report = b.sync(&tap);
    assert!(report.too_new);
    assert_eq!(report_problem(&report), Some(SyncProblem::UpdateKanso));
    assert_eq!(body(&b.store, "note:n1").as_deref(), Some("first"));
    assert_eq!(body(&b.store, "note:n2").as_deref(), Some("before"));
    assert!(
        b.store.get("note:n3").unwrap().is_none(),
        "the pull stopped before it"
    );
    assert_eq!(b.cursor(), seq_before);
    assert_eq!(report.pushed, 1, "pushing continues");

    let report = b.sync(&server);
    assert!(!report.too_new);
    assert_eq!(
        body(&b.store, "note:n1").as_deref(),
        Some("from a newer Kanso")
    );
    assert_eq!(body(&b.store, "note:n3").as_deref(), Some("after"));
}

#[test]
fn a_different_key_id_writes_nothing_and_sends_nothing() {
    let (server, a, b) = two_synced();
    edit_note(&a.store, "note:n1", "newer", Some(300));
    a.sync(&server);
    note(&b.store, "note:mine", "book:b1", "pending", 100);
    let before = snapshot(&b.store);
    let pushes = server.stats().count(Route::Push);
    let tap = Tap::new(&server, |event| {
        if let Event::Page(page) = event {
            page.key_id = "0123456789abcdef".into();
        }
        Ok(())
    });
    let err = b.try_sync(&tap).unwrap_err();
    assert!(matches!(err, SyncError::WrongKey));
    assert_eq!(kanso_sync::problem(&err), Some(SyncProblem::WrongKey));
    assert_eq!(snapshot(&b.store), before);
    assert_eq!(server.stats().count(Route::Push), pushes);
}

#[test]
fn a_restored_server_makes_every_mac_re_merge_without_losing_anything() {
    let (server, a, b) = two_synced();
    note(&a.store, "note:gone", "book:b1", "to delete", 100);
    a.sync(&server);
    b.sync(&server);
    let restore_point = server.snapshot("alice");
    // After the backup: an edit, a creation and a deletion.
    let edited = edit_note(&a.store, "note:n1", "edited after the backup", Some(200));
    note(
        &a.store,
        "note:new",
        "book:b1",
        "created after the backup",
        200,
    );
    a.store
        .delete("note:gone", kanso_store::Origin::Api)
        .unwrap();
    a.sync(&server);
    b.sync(&server);
    let synced = all_docs(&a.store);
    assert_eq!(synced, all_docs(&b.store));

    server.restore("alice", &restore_point);
    let report = a.sync(&server);
    assert!(report.remerged);
    // The restored n1 and the restored live copy of the deleted note are
    // older than what A synced: they come back stale and A's versions go up
    // again, one generation higher.
    let mut stale = report.stale.clone();
    stale.sort();
    assert_eq!(stale, vec!["note:gone".to_owned(), "note:n1".to_owned()]);
    assert_eq!(
        body(&a.store, "note:n1").as_deref(),
        Some("edited after the backup")
    );
    assert!(generation(&a.store.get("note:n1").unwrap().unwrap().rev) > generation(&edited.rev));
    assert!(
        a.store.get("note:gone").unwrap().is_none(),
        "deletions are never lost"
    );
    assert_eq!(a.store.meta(meta_keys::EPOCH).unwrap().unwrap().len(), 16);
    assert_eq!(pending(&a.store), 0);
    let server_now = server_docs(&server, &a.session);
    assert_eq!(server_now, all_docs(&a.store));
    assert!(
        server_now["note:gone"].1,
        "the tombstone is on the server again"
    );
    assert!(server_now.contains_key("note:new"));

    let report = b.sync(&server);
    assert!(report.remerged);
    assert_eq!(all_docs(&b.store), all_docs(&a.store));
    assert_eq!(live_docs(&b.store), live_docs(&a.store));
}

#[test]
fn a_rewound_account_is_re_merged_and_the_lost_documents_go_back() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    book(&a.store, "book:b1", "Inbox");
    note(&a.store, "note:n1", "book:b1", "one", 100);
    note(&a.store, "note:n2", "book:b1", "two", 100);
    a.store.delete("note:n2", kanso_store::Origin::Api).unwrap();
    a.sync(&server);
    // The cursor passes A's own pushes; a rewind below it is detected.
    a.sync(&server);
    assert_eq!(a.cursor(), 3);
    server.rewind("alice", 1);
    assert_eq!(server_docs(&server, &a.session).len(), 1);
    let report = a.sync(&server);
    assert!(report.remerged);
    assert_eq!(
        report.pushed, 2,
        "the note and the tombstone go back with no base"
    );
    assert_eq!(server_docs(&server, &a.session), all_docs(&a.store));
}

#[test]
fn an_interrupted_re_merge_runs_again_at_the_next_sync() {
    let (server, a, _b) = two_synced();
    note(&a.store, "note:n2", "book:b1", "two", 100);
    a.sync(&server);
    server.rewind("alice", 1);
    let options = SyncOptions {
        page_size: 1,
        ..SyncOptions::default()
    };
    let calls = AtomicUsize::new(0);
    let tap = Tap::new(&server, |event| match event {
        Event::Changes { .. } if calls.fetch_add(1, Ordering::SeqCst) == 2 => {
            Err(ApiError::Transport("offline (simulated)".into()))
        }
        _ => Ok(()),
    });
    assert!(a.sync_with(&tap, &options).is_err());
    assert_ne!(server_docs(&server, &a.session), all_docs(&a.store));
    let report = a.sync_with(&server, &options).unwrap();
    assert!(
        report.remerged,
        "the epoch wasn't saved, so it re-merges again"
    );
    assert_eq!(server_docs(&server, &a.session), all_docs(&a.store));
    assert!(!a.sync(&server).remerged);
}

#[test]
fn a_replayed_older_version_is_stale_and_never_reverts_the_mac() {
    let (server, a, b) = two_synced();
    let view = server.plain_view(&a.session);
    let wire = view.wire_id("note:n1");
    // Capture the server's sealed copy of the first version.
    let old = std::sync::Mutex::new(None);
    let capture = Tap::new(&server, |event| {
        if let Event::Page(page) = event {
            for doc in page.docs.iter().filter(|d| d.id == wire) {
                *old.lock().unwrap() = Some(doc.clone());
            }
        }
        Ok(())
    });
    b.store.set_meta(meta_keys::LAST_SEQ, "0").unwrap();
    b.sync(&capture);
    let old = old.into_inner().unwrap().expect("captured");
    edit_note(&a.store, "note:n1", "newer", Some(300));
    a.sync(&server);
    b.sync(&server);
    assert_eq!(body(&b.store, "note:n1").as_deref(), Some("newer"));
    // A hostile server replays the old version at a new sequence number.
    let tap = Tap::new(&server, |event| {
        if let Event::Page(page) = event {
            let mut replay = old.clone();
            replay.seq = page.head_seq + 1;
            page.head_seq += 1;
            page.last_seq = replay.seq;
            page.docs.push(replay);
        }
        Ok(())
    });
    let report = b.sync(&tap);
    assert_eq!(report.stale, vec!["note:n1".to_owned()]);
    assert_eq!(body(&b.store, "note:n1").as_deref(), Some("newer"));
}

#[test]
fn deletions_still_go_through_after_a_413() {
    let (server, a, _b) = two_synced();
    note(&a.store, "note:old", "book:b1", "to delete", 100);
    a.sync(&server);
    let usage = kanso_sync::usage(&server, &a.session).unwrap();
    server.set_quota(
        "alice",
        QuotaOverride {
            docs: Some(usage.docs),
            ..QuotaOverride::default()
        },
    );
    note(&a.store, "note:new1", "book:b1", "new", 100);
    note(&a.store, "note:new2", "book:b1", "new", 100);
    a.store
        .delete("note:old", kanso_store::Origin::Api)
        .unwrap();
    let report = a.sync(&server);
    assert_eq!(report.quota, Some(QuotaKind::Docs));
    assert_eq!(
        report_problem(&report),
        Some(SyncProblem::QuotaExceeded(QuotaKind::Docs))
    );
    assert_eq!((report.pushed, report.pending), (1, 2));
    assert!(server_docs(&server, &a.session)["note:old"].1);
    assert!(!server_docs(&server, &a.session).contains_key("note:new1"));
}

#[test]
fn a_refused_batch_is_not_sent_again_until_usage_drops() {
    let (server, a, _b) = two_synced();
    let usage = kanso_sync::usage(&server, &a.session).unwrap();
    server.set_quota(
        "alice",
        QuotaOverride {
            doc_bytes: Some(usage.doc_bytes),
            ..QuotaOverride::default()
        },
    );
    note(&a.store, "note:new", "book:b1", "new", 100);
    // A tombstone the server never saw also grows the account.
    note(&a.store, "note:tmp", "book:b1", "never synced", 100);
    a.store
        .delete("note:tmp", kanso_store::Origin::Api)
        .unwrap();
    let pushes = server.stats().count(Route::Push);
    let report = a.sync(&server);
    assert_eq!(report.quota, Some(QuotaKind::DocBytes));
    assert_eq!((report.pushed, report.pending), (0, 2));
    assert_eq!(
        server.stats().count(Route::Push) - pushes,
        1,
        "the batch only: nothing in it can go"
    );
    let entry = a.store.meta(meta_keys::QUOTA_DOCS).unwrap().unwrap();
    assert!(
        entry.starts_with("doc_bytes ") && entry.ends_with(" new"),
        "{entry}"
    );

    // Nothing changed on the server: later syncs send nothing and still
    // say the account is full.
    let writes = kanso_sync::usage(&server, &a.session).unwrap().writes_today;
    for _ in 0..2 {
        let report = a.sync(&server);
        assert_eq!(report.quota, Some(QuotaKind::DocBytes));
        assert_eq!((report.pushed, report.pending), (0, 2));
    }
    assert_eq!(server.stats().count(Route::Push) - pushes, 1);
    assert_eq!(
        kanso_sync::usage(&server, &a.session).unwrap().writes_today,
        writes
    );

    // More room: both go, and the entry goes with them.
    server.set_quota("alice", QuotaOverride::default());
    let report = a.sync(&server);
    assert_eq!((report.pushed, report.pending, report.quota), (2, 0, None));
    assert_eq!(a.store.meta(meta_keys::QUOTA_DOCS).unwrap(), None);
}

/// Review-sync 6: an account over its document quota with new notes waiting.
/// Before, every sync sent the refused batch again, spending the day's
/// writes on documents the server would never take. Now the writes grow only
/// by what the server stores: the deletion, then one new note in the room it
/// left, then nothing.
#[test]
fn over_quota_syncs_spend_writes_only_on_what_the_server_stores() {
    let (server, a, _b) = two_synced();
    let usage = kanso_sync::usage(&server, &a.session).unwrap();
    server.set_quota(
        "alice",
        QuotaOverride {
            docs: Some(usage.docs),
            ..QuotaOverride::default()
        },
    );
    for i in 0..3 {
        note(&a.store, &format!("note:new{i}"), "book:b1", "new", 100);
    }
    a.store.delete("note:n1", kanso_store::Origin::Api).unwrap();
    let writes = || kanso_sync::usage(&server, &a.session).unwrap().writes_today;
    let start = writes();
    // The batch of four is refused, then the deletion goes alone.
    let report = a.sync(&server);
    assert_eq!((report.pushed, report.pending), (1, 3));
    assert_eq!(writes() - start, 4 + 1);
    // The deletion left room for one.
    let report = a.sync(&server);
    assert_eq!((report.pushed, report.pending), (1, 2));
    assert_eq!(writes() - start, 4 + 1 + 1);
    for _ in 0..3 {
        let report = a.sync(&server);
        assert_eq!(report.quota, Some(QuotaKind::Docs));
        assert_eq!(
            report_problem(&report),
            Some(SyncProblem::QuotaExceeded(QuotaKind::Docs))
        );
        assert_eq!((report.pushed, report.pending), (0, 2));
    }
    assert_eq!(
        writes() - start,
        6,
        "syncs with nothing that fits cost nothing"
    );
    // An edit of a note the server has still goes at level `new`.
    edit_note(&a.store, "note:new0", "edited", Some(200));
    let report = a.sync(&server);
    assert_eq!((report.pushed, report.pending), (1, 2));
    assert_eq!(writes() - start, 7);
}

#[test]
fn a_refused_edit_holds_edits_but_deletions_still_go() {
    let (server, a, _b) = two_synced();
    note(&a.store, "note:gone", "book:b1", "to delete", 100);
    a.sync(&server);
    let usage = kanso_sync::usage(&server, &a.session).unwrap();
    server.set_quota(
        "alice",
        QuotaOverride {
            doc_bytes: Some(usage.doc_bytes),
            ..QuotaOverride::default()
        },
    );
    edit_note(
        &a.store,
        "note:n1",
        &"a much longer body ".repeat(50),
        Some(200),
    );
    a.store
        .delete("note:gone", kanso_store::Origin::Api)
        .unwrap();
    let report = a.sync(&server);
    assert_eq!(report.quota, Some(QuotaKind::DocBytes));
    assert_eq!((report.pushed, report.pending), (1, 1));
    assert!(server_docs(&server, &a.session)["note:gone"].1);
    let entry = a.store.meta(meta_keys::QUOTA_DOCS).unwrap().unwrap();
    assert!(entry.ends_with(" edits"), "{entry}");
    let pushes = server.stats().count(Route::Push);
    let report = a.sync(&server);
    assert_eq!((report.pushed, report.pending), (0, 1));
    assert_eq!(
        server.stats().count(Route::Push),
        pushes,
        "the edit doesn't fit the room"
    );
    server.set_quota("alice", QuotaOverride::default());
    let report = a.sync(&server);
    assert_eq!((report.pushed, report.pending, report.quota), (1, 0, None));
}

#[test]
fn a_file_upload_quota_error_keeps_its_document_pending() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    server.set_quota(
        "alice",
        QuotaOverride {
            files: Some(0),
            ..QuotaOverride::default()
        },
    );
    book(&a.store, "book:b1", "Inbox");
    file(&a.store, "file:img1", b"bytes");
    let report = a.sync(&server);
    assert_eq!(report.quota, Some(QuotaKind::Files));
    assert_eq!((report.pushed, report.pending, report.files_up), (1, 1, 0));
    assert!(!server_docs(&server, &a.session).contains_key("file:img1"));

    // The upload isn't tried again until there is room; notes still go.
    let uploads = server.stats().count(Route::PutFile);
    note(&a.store, "note:n1", "book:b1", "text", 100);
    let report = a.sync(&server);
    assert_eq!(report.quota, Some(QuotaKind::Files));
    assert_eq!((report.pushed, report.pending), (1, 1));
    assert_eq!(server.stats().count(Route::PutFile), uploads);
    server.set_quota("alice", QuotaOverride::default());
    let report = a.sync(&server);
    assert_eq!(
        (report.files_up, report.pending, report.quota),
        (1, 0, None)
    );
    assert!(server_docs(&server, &a.session).contains_key("file:img1"));
}

#[test]
fn a_429_on_a_push_pauses_syncing() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    server.set_writes_per_day(1);
    book(&a.store, "book:b1", "Inbox");
    note(&a.store, "note:n1", "book:b1", "x", 100);
    let before = kanso_core::now_ms();
    let report = a.sync(&server);
    let until = report.paused_until.expect("paused");
    assert!(until > before);
    assert_eq!(report.paused_limit.as_deref(), Some("writes-account"));
    assert_eq!(report_problem(&report), Some(SyncProblem::Paused { until }));
    assert_eq!((report.pushed, report.pending), (0, 2));
}

#[test]
fn the_servers_daily_cap_is_server_busy() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    server.set_setting("global_writes_per_day", "0");
    book(&a.store, "book:b1", "Inbox");
    let report = a.sync(&server);
    let until = report.paused_until.expect("paused");
    assert_eq!(report.paused_limit.as_deref(), Some("writes-global"));
    assert_eq!(
        report_problem(&report),
        Some(SyncProblem::ServerBusy { until })
    );
    assert_eq!(
        SyncProblem::ServerBusy { until }.schedule(),
        kanso_sync::Schedule::WaitsUntil(until)
    );
    assert_eq!((report.pushed, report.pending), (0, 1));
}

/// Review-sync (j): the server's daily upload cap holds uploads only. The
/// file waits; notes go and pulls go on, and it is no problem.
#[test]
fn the_servers_upload_cap_holds_uploads_and_nothing_else() {
    let (server, a, b) = two_synced();
    server.set_setting("global_upload_bytes_per_day", "1");
    file(&a.store, "file:img1", b"bytes");
    note(&a.store, "note:a", "book:b1", "from A", 100);
    let report = a.sync(&server);
    let until = report.uploads_held_until.expect("held");
    assert!(until > kanso_core::now_ms());
    assert_eq!((report.paused_until, report.quota), (None, None));
    assert_eq!(report_problem(&report), None);
    assert_eq!((report.pushed, report.pending, report.files_up), (1, 1, 0));
    assert_eq!(
        a.store.meta(meta_keys::UPLOADS_UNTIL).unwrap(),
        Some(until.to_string())
    );

    // Held: no upload is tried, and changes still come and go.
    let uploads = server.stats().count(Route::PutFile);
    note(&b.store, "note:b", "book:b1", "from B", 100);
    b.sync(&server);
    let report = a.sync(&server);
    assert_eq!(report.uploads_held_until, Some(until));
    assert_eq!((report.pulled, report.pending), (1, 1));
    assert_eq!(body(&a.store, "note:b").as_deref(), Some("from B"));
    assert_eq!(server.stats().count(Route::PutFile), uploads);

    // Once the time has come, the hold goes and the file goes up.
    server.set_setting("global_upload_bytes_per_day", "1073741824");
    a.store.set_meta(meta_keys::UPLOADS_UNTIL, "1").unwrap();
    let report = a.sync(&server);
    assert_eq!((report.files_up, report.pending), (1, 0));
    assert_eq!(report.uploads_held_until, None);
    assert_eq!(a.store.meta(meta_keys::UPLOADS_UNTIL).unwrap(), None);
}

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

/// Review-verify 5: a 429's `retry_after` never holds this Mac back more
/// than a day, wherever it becomes a time: a push's pause, an upload hold
/// (which is saved), and a back-off.
#[test]
fn a_huge_retry_after_holds_at_most_a_day() {
    let huge = |limit: &str| ApiError::RateLimited {
        retry_after: u64::MAX,
        limit: limit.into(),
    };
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    book(&a.store, "book:b1", "Inbox");
    let pushing = Tap::new(&server, |event| match event {
        Event::Push(_) => Err(huge("writes-account")),
        _ => Ok(()),
    });
    let before = kanso_core::now_ms();
    let until = a.sync(&pushing).paused_until.expect("paused");
    drop(pushing);
    assert!(until > before && until <= kanso_core::now_ms() + DAY_MS);

    file(&a.store, "file:img1", b"bytes");
    let uploading = Tap::new(&server, |event| match event {
        Event::PutFile { .. } => Err(huge("uploads-global")),
        _ => Ok(()),
    });
    let until = a.sync(&uploading).uploads_held_until.expect("held");
    drop(uploading);
    assert!(until <= kanso_core::now_ms() + DAY_MS);
    assert_eq!(
        a.store.meta(meta_keys::UPLOADS_UNTIL).unwrap(),
        Some(until.to_string())
    );

    let busy = Tap::new(&server, |event| match event {
        Event::Changes { .. } => Err(huge("session-account")),
        _ => Ok(()),
    });
    match a.try_sync(&busy) {
        Err(SyncError::BackOff { until }) => {
            assert!(until <= kanso_core::now_ms() + DAY_MS);
        }
        other => panic!("{other:?}"),
    }
}

/// Review-verify 5: holds saved more than a day and an hour ahead (the
/// clock ran ahead when they were saved) are dropped when read, so uploads
/// and pushes go again; within that, they still hold.
#[test]
fn holds_saved_too_far_ahead_are_dropped() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let far = kanso_core::now_ms() + 2 * DAY_MS;
    let keys = [
        meta_keys::UPLOADS_UNTIL,
        meta_keys::QUOTA_DOCS,
        meta_keys::QUOTA_FILES,
    ];
    a.store
        .set_meta(meta_keys::UPLOADS_UNTIL, &far.to_string())
        .unwrap();
    for key in &keys[1..] {
        a.store
            .set_meta(key, &format!("server 0 0 {far} all"))
            .unwrap();
    }
    file(&a.store, "file:img1", b"bytes");
    book(&a.store, "book:b1", "Inbox");
    let report = a.sync(&server);
    assert_eq!(
        (report.files_up, report.pending, report.uploads_held_until),
        (1, 0, None),
        "{report:?}"
    );
    assert_eq!(report.quota, None);
    for key in keys {
        assert_eq!(a.store.meta(key).unwrap(), None, "{key}");
    }

    let near = kanso_core::now_ms() + DAY_MS;
    a.store
        .set_meta(meta_keys::QUOTA_DOCS, &format!("server 0 0 {near} all"))
        .unwrap();
    book(&a.store, "book:b2", "Later");
    let report = a.sync(&server);
    assert_eq!(report.pending, 1, "the entry still holds: {report:?}");
    assert!(a.store.meta(meta_keys::QUOTA_DOCS).unwrap().is_some());
}

fn session_limit() -> ApiError {
    ApiError::RateLimited {
        retry_after: 30,
        limit: "session-account".into(),
    }
}

/// Checks a back-off of about 30 s from `before`.
fn assert_backs_off(result: Result<SyncReport, SyncError>, before: i64) {
    let err = result.unwrap_err();
    let SyncError::BackOff { until } = err else {
        panic!("not a back-off: {err:?}");
    };
    assert!(until >= before + 30_000 && until <= kanso_core::now_ms() + 30_000);
    let problem = kanso_sync::problem(&err).unwrap();
    assert_eq!(problem, SyncProblem::BackOff { until });
    assert_eq!(problem.schedule(), kanso_sync::Schedule::WaitsUntil(until));
}

/// A 429 `session-account` on the feed: the sync stops after the pages it
/// applied, keeps the cursor there, and waits about `retry_after`.
#[test]
fn a_session_limit_on_the_feed_backs_off_and_keeps_the_cursor() {
    let (server, a, b) = two_synced();
    for i in 0..3 {
        note(&a.store, &format!("note:p{i}"), "book:b1", "x", 100);
    }
    a.sync(&server);
    let start = b.cursor();
    let pages = AtomicUsize::new(0);
    let tap = Tap::new(&server, |event| {
        if matches!(event, Event::Changes { .. }) && pages.fetch_add(1, Ordering::SeqCst) == 1 {
            return Err(session_limit());
        }
        Ok(())
    });
    let options = SyncOptions {
        page_size: 1,
        ..SyncOptions::default()
    };
    let before = kanso_core::now_ms();
    assert_backs_off(b.sync_with(&tap, &options), before);
    let first = b.cursor();
    assert!(first > start, "the first page was applied");
    assert_eq!(live_docs(&b.store).len(), 3, "and nothing after it");
    let report = b.sync(&server);
    assert_eq!(report.pulled, 2);
    assert_eq!(live_docs(&b.store), live_docs(&a.store));
}

/// The same on a push, an upload and a download: nothing is marked done,
/// the cursor stays, and the next sync finishes the work.
#[test]
fn a_session_limit_on_a_push_or_a_file_backs_off() {
    for route in ["push", "upload", "download"] {
        let (server, a, b) = two_synced();
        a.sync(&server);
        file(&a.store, "file:img1", b"bytes");
        note(&a.store, "note:a", "book:b1", "from A", 100);
        let (mac, other) = if route == "download" {
            a.sync(&server);
            (&b, &a)
        } else {
            (&a, &b)
        };
        let before_docs = sync_meta(&mac.store);
        let pending_before = pending(&mac.store);
        let tap = Tap::new(&server, |event| match (route, event) {
            ("push", Event::Push(_))
            | ("upload", Event::PutFile { .. })
            | ("download", Event::File(_)) => Err(session_limit()),
            _ => Ok(()),
        });
        let before = kanso_core::now_ms();
        assert_backs_off(mac.try_sync(&tap), before);
        assert_eq!(
            sync_meta(&mac.store),
            before_docs,
            "{route}: cursor or meta moved"
        );
        assert_eq!(pending(&mac.store), pending_before, "{route}");
        let report = mac.sync(&server);
        assert_eq!(report.pending, 0, "{route}");
        other.sync(&server);
        assert_eq!(live_docs(&a.store), live_docs(&b.store), "{route}");
        assert_eq!(
            a.store
                .attachment("file:img1")
                .unwrap()
                .map(|(_, f)| f.data),
            b.store
                .attachment("file:img1")
                .unwrap()
                .map(|(_, f)| f.data),
            "{route}"
        );
    }
}

#[test]
fn a_429_on_an_upload_pauses_syncing_and_keeps_the_file_pending() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    // Under one sealed file: the account's day of upload bytes is used up.
    server.set_upload_bytes_per_day(1);
    book(&a.store, "book:b1", "Inbox");
    file(&a.store, "file:img1", b"bytes");
    let before = kanso_core::now_ms();
    let report = a.sync(&server);
    let until = report.paused_until.expect("paused");
    assert!(until > before);
    assert_eq!(report_problem(&report), Some(SyncProblem::Paused { until }));
    assert_eq!(report.files_up, 0);
    assert!(report.pending >= 1);
    assert!(!server_docs(&server, &a.session).contains_key("file:img1"));

    // With room in the day's budget the next sync uploads it.
    server.set_upload_bytes_per_day(1_073_741_824);
    let report = a.sync(&server);
    assert_eq!(report.paused_until, None);
    assert_eq!((report.files_up, report.pending), (1, 0));
    assert!(server_docs(&server, &a.session).contains_key("file:img1"));
}

#[test]
fn a_cancelled_sync_changes_nothing_and_sends_nothing() {
    let (server, a, b) = two_synced();
    edit_note(&a.store, "note:n1", "newer", Some(300));
    a.sync(&server);
    let before = snapshot(&b.store);
    let requests = server.stats().count(Route::Changes);
    let options = SyncOptions {
        cancel: Some(Arc::new(AtomicBool::new(true))),
        ..SyncOptions::default()
    };
    assert!(matches!(
        b.sync_with(&server, &options),
        Err(SyncError::Cancelled)
    ));
    assert_eq!(kanso_sync::problem(&SyncError::Cancelled), None);
    assert_eq!(snapshot(&b.store), before);
    assert_eq!(server.stats().count(Route::Changes), requests);
}

#[test]
fn a_library_re_bound_during_a_sync_gets_no_write_of_the_old_account() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let bob = sign_up(&server, "bob", 2);
    book(&a.store, "book:b1", "Inbox");
    note(&a.store, "note:n1", "book:b1", "x", 100);
    let after_bind = std::sync::Mutex::new(None);
    let tap = Tap::new(&server, |event| {
        if let Event::Push(_) = event {
            kanso_sync::bind(&a.store, &bob, Choice::Add, &a.dir.path().join("b"), 0).unwrap();
            *after_bind.lock().unwrap() = Some(snapshot(&a.store));
        }
        Ok(())
    });
    assert!(matches!(a.try_sync(&tap), Err(SyncError::Cancelled)));
    assert_eq!(Some(snapshot(&a.store)), after_bind.into_inner().unwrap());
    assert_eq!(
        a.store.meta(meta_keys::LAST_SEQ).unwrap().as_deref(),
        Some("0")
    );
    assert_eq!(
        a.store.meta(meta_keys::ACCOUNT_ID).unwrap(),
        Some(bob.account_id.clone())
    );
}

#[test]
fn a_library_bound_elsewhere_sends_nothing() {
    let server = MemoryServer::new();
    let alice = sign_up(&server, "alice", 1);
    let b = Mac::new(sign_up(&server, "bob", 2));
    note(&b.store, "note:n1", "trash", "bob's", 100);
    assert!(matches!(
        kanso_sync::sync(&b.store, &server, &alice, &SyncOptions::default()),
        Err(SyncError::Cancelled)
    ));
    assert!(server.plain_view(&alice).docs().is_empty());
    assert_eq!(server.stats().count(Route::Changes), 0);
}

#[test]
fn plain_view_pushes_as_another_device() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let view = server.plain_view(&a.session);
    let doc = SyncDoc {
        id: "book:other".into(),
        rev: "1-aa".into(),
        deleted: false,
        fields: json!({"name": "From elsewhere", "createdAt": 1, "updatedAt": 1})
            .as_object()
            .unwrap()
            .clone(),
        base_rev: None,
    };
    assert!(matches!(
        view.push(std::slice::from_ref(&doc)).unwrap()[0],
        PushResult::Ok { .. }
    ));
    let again = view.push(std::slice::from_ref(&doc)).unwrap();
    assert!(matches!(&again[0], PushResult::Conflict { current: Some(c), .. } if c.rev == "1-aa"));
    let lost = SyncDoc {
        id: "book:never".into(),
        base_rev: Some("1-aa".into()),
        ..doc.clone()
    };
    assert_eq!(
        view.push(&[lost]).unwrap(),
        vec![PushResult::Lost {
            id: "book:never".into()
        }]
    );
    a.sync(&server);
    assert_eq!(a.store.books().unwrap()[0].name, "From elsewhere");
}

#[test]
fn an_answer_reaches_the_other_device_and_a_document_of_an_unknown_kind_is_skipped() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    book(&a.store, "book:b1", "Inbox");
    note(
        &a.store,
        "note:n1",
        "book:b1",
        "Capital of France? :: Paris\n",
        100,
    );
    a.sync(&server);
    b.sync(&server);
    let key = kanso_core::cards::card_key("note:n1", "Capital of France?");
    a.store
        .record_review(
            "note:n1",
            &key,
            kanso_core::srs::Grade::Good,
            "dA",
            1_000_000,
        )
        .unwrap();
    a.sync(&server);
    // A later answer from the other device joins it: the review documents
    // are merged by union, so neither answer is lost.
    b.sync(&server);
    b.store
        .record_review(
            "note:n1",
            &key,
            kanso_core::srs::Grade::Easy,
            "dB",
            2_000_000,
        )
        .unwrap();
    b.sync(&server);
    a.sync(&server);
    assert_eq!(
        a.store.card_intervals(&key, 3_000_000).unwrap(),
        b.store.card_intervals(&key, 3_000_000).unwrap(),
        "the two devices agree on the schedule"
    );
    assert!(
        a.store.card_intervals(&key, 3_000_000).unwrap()[2] > 0,
        "both answers were replayed"
    );

    // An older Kanso has never heard of the `review` kind: a document whose
    // kind a device does not know is skipped, the rest still sync, and the
    // pull moves past it.
    note(&a.store, "note:n2", "book:b1", "after", 100);
    a.sync(&server);
    let view = server.plain_view(&a.session);
    let unknown = view.wire_id("note:n2");
    let tap = Tap::new(&server, |event| {
        if let Event::Page(page) = event {
            for doc in page.docs.iter_mut().filter(|d| d.id == unknown) {
                doc.id = format!("zzz:{}", &doc.id["note:".len()..]);
            }
        }
        Ok(())
    });
    let report = b.sync(&tap);
    // A kind from a newer Kanso is left alone and is no "Some items skipped".
    assert!(report.skipped.is_empty(), "{:?}", report.skipped);
    assert_eq!(
        body(&b.store, "note:n1").as_deref(),
        Some("Capital of France? :: Paris\n")
    );
    assert_eq!(b.cursor(), view.docs().last().unwrap().seq);
}

#[test]
fn a_reset_survives_a_device_that_reviewed_offline() {
    use kanso_core::srs::Grade;
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    book(&a.store, "book:b1", "Inbox");
    note(&a.store, "note:n1", "book:b1", "Q :: A\nR :: B\n", 100);
    a.sync(&server);
    b.sync(&server);
    let q = kanso_core::cards::card_key("note:n1", "Q");
    let r = kanso_core::cards::card_key("note:n1", "R");
    a.store
        .record_review("note:n1", &q, Grade::Good, "dA", 1_000_000)
        .unwrap();
    a.sync(&server);
    b.sync(&server);
    // A resets while B, offline, answers another card the same month.
    a.store.reset_review_history_at(2_000_000).unwrap();
    b.store
        .record_review("note:n1", &r, Grade::Easy, "dB", 1_500_000)
        .unwrap();
    a.sync(&server);
    b.sync(&server);
    a.sync(&server);
    for mac in [&a, &b] {
        // Everything answered before the reset stays gone, on both devices.
        let counts = mac
            .store
            .card_counts(&kanso_store::CardScope::All, 9_000_000)
            .unwrap();
        assert_eq!(counts.new, 2, "no history before the reset survives");
        assert_eq!(counts.total, 2);
    }
    // An answer after the reset counts, on both.
    b.store
        .record_review("note:n1", &q, Grade::Good, "dB", 3_000_000)
        .unwrap();
    b.sync(&server);
    a.sync(&server);
    for mac in [&a, &b] {
        assert_eq!(
            mac.store
                .card_counts(&kanso_store::CardScope::All, 9_000_000)
                .unwrap()
                .new,
            1
        );
    }
}

#[test]
fn a_deleted_notes_answers_stay_deleted_against_an_offline_review() {
    use kanso_core::srs::Grade;
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    book(&a.store, "book:b1", "Inbox");
    note(&a.store, "note:n1", "book:b1", "Q :: A\n", 100);
    let q = kanso_core::cards::card_key("note:n1", "Q");
    a.store
        .record_review("note:n1", &q, Grade::Good, "dA", 1_000_000)
        .unwrap();
    a.sync(&server);
    b.sync(&server);
    // A deletes the note (its review documents become tombstones) while B
    // answers offline in the same month.
    a.store.delete("note:n1", kanso_store::Origin::Ui).unwrap();
    b.store
        .record_review("note:n1", &q, Grade::Easy, "dB", 1_500_000)
        .unwrap();
    a.sync(&server);
    b.sync(&server);
    a.sync(&server);
    for mac in [&a, &b] {
        let live = mac
            .store
            .sync_docs()
            .unwrap()
            .into_iter()
            .filter(|d| d.id.starts_with("review:") && !d.deleted)
            .count();
        assert_eq!(live, 0, "the tombstone beats the offline answer");
    }
}
