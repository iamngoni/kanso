//! Sealed attachments (issue #44 step 1): a file document marked with
//! `kansoFileLock` holds a sealed container in `blobs`, the ordinary write
//! path can neither replace its bytes nor drop the marker, imports and
//! served output strip the marker, and the store reads the states back.

use base64::Engine as _;
use kanso_core::DocType;
use kanso_core::model::{
    FILE_LOCK_FIELD, FileLock, KANSO_FILES_FIELD, LOCK_FIELD, MAX_ATTACHMENT_SIZE,
    MAX_SEALED_ATTACHMENT, NoteFile, OCR_FIELD, SEALED_FILE_CONTENT_TYPE, SEALED_FILE_NAME,
    UNSEALED_FIELD,
};
use kanso_store::{Attachment, AttachmentState, Origin, Put, Store, StoreError};
use proptest::prelude::*;
use serde_json::json;

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

/// Creates a plain file with `bytes` and returns its ID.
fn plain(store: &Store, bytes: &[u8]) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    store
        .put(
            DocType::File,
            json!({"name": "shot.png", "contentType": "image/png",
                   "_attachments": {"index": {"content_type": "image/png", "data": encoded}}}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id
}

/// Seals `id` with `container` as its stored bytes, the way the app will.
fn seal(store: &Store, id: &str, container: &[u8]) -> String {
    let mut file = store.file_meta(id).unwrap().unwrap();
    let seal_id = (0u8..16).collect::<Vec<_>>();
    FileLock::store(
        Some(&FileLock::sealed(
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(seal_id),
        )),
        &mut file.extra,
    );
    store
        .put_sealed_file(file, Attachment::new(container.to_vec()))
        .unwrap()
        .rev
}

fn container(bytes: &[u8]) -> Vec<u8> {
    // The store never parses containers; a `0x01`-headed blob stands for one.
    let mut container = vec![0x01, 0x01, 0x00];
    container.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    container.extend_from_slice(bytes);
    container
}

#[test]
fn a_sealed_file_reads_as_sealed_and_never_as_plain() {
    let (_dir, store) = store();
    let id = plain(&store, b"plain bytes");
    assert_eq!(store.attachment_state(&id).unwrap(), AttachmentState::Plain);
    let sealed = container(b"sealed bytes");
    seal(&store, &id, &sealed);
    assert_eq!(
        store.attachment_state(&id).unwrap(),
        AttachmentState::Sealed
    );
    assert_eq!(store.stored_attachment(&id).unwrap().unwrap().data, sealed);
    assert_eq!(store.attachment(&id).unwrap(), None);
    // The served file never carries the marker, and the digest and length
    // describe the stored container.
    let served = store.get_api(&id, None, false).unwrap();
    assert!(served.get(FILE_LOCK_FIELD).is_none());
    assert_eq!(served["contentLength"], sealed.len());
    assert_eq!(served["_attachments"]["index"]["length"], sealed.len());
    assert_eq!(
        served["_attachments"]["index"]["digest"],
        Attachment::new(sealed.clone()).digest()
    );
    // Asking for attachment bytes does not bypass the lock in the API.
    let asked = store.get_api(&id, None, true).unwrap();
    assert_eq!(asked["locked"], true);
    assert_eq!(asked["_attachments"]["index"]["stub"], true);
    assert!(asked["_attachments"]["index"].get("data").is_none());
    let listed = store.list_api(DocType::File, None, 0).unwrap();
    assert!(listed[0].get(FILE_LOCK_FIELD).is_none());
    assert_eq!(listed[0]["locked"], true);
}

#[test]
fn a_served_sealed_file_has_no_real_name_or_type() {
    let (_dir, store) = store();
    let id = plain(&store, b"private attachment");
    let mut file = store.file_meta(&id).unwrap().unwrap();
    let mut lock = FileLock::sealed("AAAAAAAAAAAAAAAAAAAAAA".into());
    lock.meta = true;
    FileLock::store(Some(&lock), &mut file.extra);
    store
        .put_sealed_file(file, Attachment::new(container(b"opaque")))
        .unwrap();

    let raw = store.get(&id).unwrap().unwrap();
    assert_eq!(raw.fields["name"], SEALED_FILE_NAME);
    assert_eq!(raw.fields["contentType"], SEALED_FILE_CONTENT_TYPE);
    assert_eq!(raw.fields["publicIn"], json!([]));
    assert!(FileLock::from_extra(&raw.fields).unwrap().meta);
    let raw_json = serde_json::to_string(&raw.fields).unwrap();
    assert!(!raw_json.contains("shot.png"));
    assert!(!raw_json.contains("image/png"));

    let served = store.get_api(&id, None, true).unwrap();
    assert_eq!(served["name"], SEALED_FILE_NAME);
    assert_eq!(served["contentType"], SEALED_FILE_CONTENT_TYPE);
    assert_eq!(
        served["_attachments"]["index"]["content_type"],
        SEALED_FILE_CONTENT_TYPE
    );
    assert_eq!(served["publicIn"], json!([]));
    assert_eq!(served["locked"], true);
    assert_eq!(served["_attachments"]["index"]["stub"], true);
    assert!(served["_attachments"]["index"].get("data").is_none());
    let listed = store.list_api(DocType::File, None, 0).unwrap();
    assert_eq!(listed[0]["name"], SEALED_FILE_NAME);
    assert_eq!(listed[0]["contentType"], SEALED_FILE_CONTENT_TYPE);
    assert!(!listed[0].to_string().contains("shot.png"));
    assert!(!listed[0].to_string().contains("image/png"));
    assert_eq!(
        store.file_meta(&id).unwrap().unwrap().name,
        SEALED_FILE_NAME
    );
}

#[test]
fn file_ownership_unions_note_lists_and_bodies_and_survives_trash_and_rebuild() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kanso.db");
    let store = Store::open(&path).unwrap();
    let book = store
        .put(
            DocType::Book,
            json!({"_id":"book:b", "name":"B"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let plain_file = plain(&store, b"plain");
    let sealed_file = plain(&store, b"sealed");
    let shared_file = plain(&store, b"shared");
    let mut locked = json!({
        "_id":"note:locked", "title":"Locked", "body":"", "bookId":book,
        LOCK_FIELD:{"v":1},
        KANSO_FILES_FIELD:[
            {"file":sealed_file, "v":1, "seal":"AAAAAAAAAAAAAAAAAAAAAA", "key":"wrapped"},
            {"file":shared_file, "v":1, "seal":"AQEBAQEBAQEBAQEBAQEBAQ", "key":"wrapped"}
        ]
    });
    store
        .put(DocType::Note, locked.take(), Put::Upsert, Origin::Ui)
        .unwrap();
    store.put(DocType::Note, json!({
        "_id":"note:plain", "title":"Plain", "body":format!("![](inkdrop://{}) ![](inkdrop://{})", plain_file, shared_file), "bookId":book
    }), Put::Upsert, Origin::Ui).unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id":"note:locked", "bookId":"trash"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();

    let locked_files: std::collections::BTreeSet<_> = store
        .files_of_note("note:locked")
        .unwrap()
        .into_iter()
        .collect();
    assert_eq!(
        locked_files,
        [sealed_file.clone(), shared_file.clone()].into()
    );
    assert_eq!(
        store.file_owners(&shared_file).unwrap(),
        ["note:locked", "note:plain"]
    );
    assert_eq!(store.file_owners(&sealed_file).unwrap(), ["note:locked"]);
    store
        .set_meta(kanso_store::DERIVED_VERSION_KEY, "1")
        .unwrap();
    drop(store);

    let store = Store::open(&path).unwrap();
    let rebuilt_files: std::collections::BTreeSet<_> = store
        .files_of_note("note:locked")
        .unwrap()
        .into_iter()
        .collect();
    assert_eq!(rebuilt_files, [sealed_file, shared_file.clone()].into());
    assert_eq!(
        store.file_owners(&shared_file).unwrap(),
        ["note:locked", "note:plain"]
    );
    assert_eq!(
        store
            .meta(kanso_store::DERIVED_VERSION_KEY)
            .unwrap()
            .as_deref(),
        Some("2")
    );
}

#[test]
fn ownership_falls_back_to_raw_body_for_unsealed_notes_in_locked_notebooks() {
    let (_dir, store) = store();
    let file = plain(&store, b"attachment");
    store
        .put(
            DocType::Book,
            json!({
                "_id":"book:locked", "name":"Locked", LOCK_FIELD:{"v":1}
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    store.put(DocType::Note, json!({
        "_id":"note:in-book", "title":"Remote", "body":format!("![](inkdrop://{})", file), "bookId":"book:locked"
    }), Put::Upsert, Origin::Sync).unwrap();
    store.put(DocType::Note, json!({
        "_id":"note:sealed-in-book", "title":"Sealed", "body":format!("![](inkdrop://{})", file),
        "bookId":"book:locked", LOCK_FIELD:{"v":1}
    }), Put::Upsert, Origin::Sync).unwrap();
    assert_eq!(
        store.files_of_note("note:in-book").unwrap(),
        std::slice::from_ref(&file)
    );
    assert_eq!(store.files_of_note("note:sealed-in-book").unwrap(), [file]);
    assert_eq!(
        store.get_api("note:in-book", None, false).unwrap()["body"],
        ""
    );
}

#[test]
fn local_unlocked_notes_cannot_link_sealed_files_and_note_files_are_app_only() {
    let (_dir, store) = store();
    let book = store
        .put(
            DocType::Book,
            json!({"_id":"book:b", "name":"B"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let file = plain(&store, b"attachment");
    seal(&store, &file, &container(b"sealed"));
    let err = store.put(DocType::Note, json!({
        "_id":"note:bad", "title":"Bad", "body":format!("![](inkdrop://{})", file), "bookId":book
    }), Put::Upsert, Origin::Ui).unwrap_err();
    assert!(
        matches!(err, StoreError::Invalid(message) if message == "That attachment belongs to a locked note.")
    );

    let sealed = store
        .put(
            DocType::Note,
            json!({
                "_id":"note:owner", "title":"Owner", "body":"", "bookId":book,
                LOCK_FIELD:{"v":1}, KANSO_FILES_FIELD:[
                    {"file":file, "v":1, "seal":"AAAAAAAAAAAAAAAAAAAAAA", "key":"wrapped"}
                ]
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    assert!(
        store
            .get_api(&sealed, None, false)
            .unwrap()
            .get(KANSO_FILES_FIELD)
            .is_none()
    );
    store
        .put(
            DocType::Book,
            json!({
                "_id":"book:locked", "name":"Locked", LOCK_FIELD:{"v":1}
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    assert!(store.put(DocType::Note, json!({
        "_id":"note:empty-in-locked-book", "title":"Empty", "body":"", "bookId":"book:locked"
    }), Put::Upsert, Origin::Ui).is_ok());
}

#[test]
fn imports_strip_note_file_ownership() {
    let (_dir, store) = store();
    let fields = json!({
        "title":"Imported", "body":"", "bookId":"trash",
        KANSO_FILES_FIELD:[{"file":"file:sealed", "v":1, "seal":"AAAAAAAAAAAAAAAAAAAAAA", "key":"wrapped"}]
    })
    .as_object()
    .unwrap()
    .clone();
    assert!(
        store
            .import_doc(
                DocType::Note,
                "note:imported",
                "1-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                fields,
                None,
            )
            .unwrap()
    );
    assert!(
        store
            .get("note:imported")
            .unwrap()
            .unwrap()
            .fields
            .get(KANSO_FILES_FIELD)
            .is_none()
    );
}

#[test]
fn a_local_write_cannot_replace_a_sealed_file() {
    let (_dir, store) = store();
    let id = plain(&store, b"before");
    let sealed = container(b"sealed");
    seal(&store, &id, &sealed);

    let fresh = base64::engine::general_purpose::STANDARD.encode(b"after");
    for mode in [Put::Upsert, Put::Merge] {
        let rev = store.get_api(&id, None, false).unwrap()["_rev"]
            .as_str()
            .unwrap()
            .to_owned();
        let body = json!({"_id": id, "_rev": rev, "name": "shot.png",
                          "contentType": "image/png",
                          "_attachments": {"index": {"content_type": "image/png", "data": fresh}}});
        let err = store
            .put(DocType::File, body, mode, Origin::Api)
            .unwrap_err();
        assert!(matches!(err, StoreError::Invalid(_)), "{mode:?}: {err}");
    }
    assert_eq!(store.stored_attachment(&id).unwrap().unwrap().data, sealed);
}

#[test]
fn a_local_write_cannot_drop_the_seal_marker() {
    let (_dir, store) = store();
    let id = plain(&store, b"before");
    seal(&store, &id, &container(b"sealed"));

    // A whole-document replace without the marker keeps it (the API passes
    // the marker through `app_only_fields`; the store holds it regardless).
    let mut body = store.get_api(&id, None, false).unwrap();
    body["name"] = "renamed.png".into();
    store
        .put(DocType::File, body, Put::Upsert, Origin::Api)
        .unwrap();
    assert_eq!(
        store.attachment_state(&id).unwrap(),
        AttachmentState::Sealed
    );
    let meta = store.file_meta(&id).unwrap().unwrap();
    assert!(meta.extra.contains_key(FILE_LOCK_FIELD));
    assert_eq!(meta.name, "shot.png");
    assert_eq!(meta.content_type, "image/png");

    // A merge that tries to null the marker keeps it too.
    let rev = store.file_meta(&id).unwrap().unwrap().rev.unwrap();
    store
        .put(
            DocType::File,
            json!({"_id": id, "_rev": rev, FILE_LOCK_FIELD: null}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
    assert_eq!(
        store.attachment_state(&id).unwrap(),
        AttachmentState::Sealed
    );

    // An ordinary write can't plant the marker on a plain file either.
    let plain_id = plain(&store, b"still plain");
    let rev = store.file_meta(&plain_id).unwrap().unwrap().rev.unwrap();
    store
        .put(
            DocType::File,
            json!({"_id": plain_id, "_rev": rev, FILE_LOCK_FIELD:
                   {"v": 1, "kind": "file", "seal": "AAAAAAAAAAAAAAAAAAAAAA", "meta": false}}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
    assert_eq!(
        store.attachment_state(&plain_id).unwrap(),
        AttachmentState::Plain
    );
}

#[test]
fn an_imported_file_never_gains_the_seal_marker() {
    let (_dir, store) = store();
    store
        .import_doc(
            DocType::File,
            "file:imp1",
            "5-abc",
            json!({"name": "x.png", "contentType": "image/png",
                   FILE_LOCK_FIELD: {"v": 1, "kind": "file", "seal": "AAAAAAAAAAAAAAAAAAAAAA", "meta": false}})
            .as_object()
            .unwrap()
            .clone(),
            Some(Attachment::new(b"bytes".to_vec())),
        )
        .unwrap();
    assert_eq!(
        store.attachment_state("file:imp1").unwrap(),
        AttachmentState::Plain
    );
    let doc = store.get("file:imp1").unwrap().unwrap();
    assert!(doc.fields.get(FILE_LOCK_FIELD).is_none());
}

#[test]
fn a_deleted_or_never_made_file_reads_as_missing() {
    let (_dir, store) = store();
    assert_eq!(
        store.attachment_state("file:nope").unwrap(),
        AttachmentState::Missing
    );
    let id = plain(&store, b"bytes");
    seal(&store, &id, &container(b"sealed"));
    assert_eq!(
        store.attachment_state(&id).unwrap(),
        AttachmentState::Sealed
    );
    store.delete(&id, Origin::Api).unwrap();
    assert_eq!(
        store.attachment_state(&id).unwrap(),
        AttachmentState::Missing
    );
    assert_eq!(store.stored_attachment(&id).unwrap(), None);
}

#[test]
fn put_sealed_file_refuses_an_unmarked_file_and_an_oversized_container() {
    let (_dir, store) = store();
    let id = plain(&store, b"bytes");

    // The marker must be set: it is the whole point of the sealed path.
    let err = store
        .put_sealed_file(
            store.file_meta(&id).unwrap().unwrap(),
            Attachment::new(container(b"sealed")),
        )
        .unwrap_err();
    assert!(matches!(err, StoreError::Invalid(_)), "{err}");

    // A malformed marker is not a seal: version, kind and the canonical
    // 16-byte seal ID must all be exact.
    for marker in [
        json!({"v": 2, "kind": "file", "seal": "AAAAAAAAAAAAAAAAAAAAAA", "meta": false}),
        json!({"v": 1, "kind": "note", "seal": "AAAAAAAAAAAAAAAAAAAAAA", "meta": false}),
        json!({"v": 1, "kind": "file", "seal": "short", "meta": false}),
        json!({"v": 1, "kind": "file", "seal": "AAAAAAAAAAAAAAAAAAAAAA=", "meta": false}),
    ] {
        let mut meta = store.file_meta(&id).unwrap().unwrap();
        meta.extra.insert(FILE_LOCK_FIELD.into(), marker);
        let err = store
            .put_sealed_file(meta, Attachment::new(container(b"sealed")))
            .unwrap_err();
        assert!(matches!(err, StoreError::Invalid(_)), "{err}");
    }

    // The container must fit MAX_SEALED_ATTACHMENT: the plaintext limit
    // plus the container's overhead, never a byte more.
    let mut meta = store.file_meta(&id).unwrap().unwrap();
    FileLock::store(
        Some(&FileLock::sealed("AAAAAAAAAAAAAAAAAAAAAA".into())),
        &mut meta.extra,
    );
    let too_big = vec![0x5a; MAX_SEALED_ATTACHMENT + 1];
    let err = store
        .put_sealed_file(meta.clone(), Attachment::new(too_big))
        .unwrap_err();
    assert!(matches!(err, StoreError::Invalid(_)), "{err}");

    // Exactly at the bound it is accepted, and the ordinary path keeps its
    // 10 MiB plaintext limit unchanged.
    let at_bound = vec![0x5a; MAX_SEALED_ATTACHMENT];
    store
        .put_sealed_file(meta, Attachment::new(at_bound.clone()))
        .unwrap();
    assert_eq!(
        store.stored_attachment(&id).unwrap().unwrap().data,
        at_bound
    );
    let plain_id = plain(&store, b"small");
    let too_big_plain =
        base64::engine::general_purpose::STANDARD.encode(vec![0x5a; MAX_ATTACHMENT_SIZE + 1]);
    let err = store
        .put(
            DocType::File,
            json!({"name": "big.png", "contentType": "image/png",
                   "_attachments": {"index": {"content_type": "image/png", "data": too_big_plain}}}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap_err();
    assert!(matches!(err, StoreError::Invalid(_)), "{err}");
    assert!(plain_id.starts_with("file:"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Invariant: whatever the sealed bytes, the store keeps them exactly,
    /// reads the file as Sealed and never as Plain, refuses an ordinary
    /// write, and keeps the marker on a local merge.
    #[test]
    fn prop_a_sealed_file_keeps_its_container(bytes in proptest::collection::vec(any::<u8>(), 0..=8_192)) {
        let (_dir, store) = store();
        let id = plain(&store, b"before");
        let sealed = container(&bytes);
        seal(&store, &id, &sealed);
        prop_assert_eq!(store.attachment_state(&id).unwrap(), AttachmentState::Sealed);
        prop_assert_eq!(store.stored_attachment(&id).unwrap().unwrap().data, sealed);
        let rev = store.get_api(&id, None, false).unwrap()["_rev"].as_str().unwrap().to_owned();
        let body = json!({"_id": id, "_rev": rev, "name": "other.png", "contentType": "image/png"});
        store.put(DocType::File, body, Put::Upsert, Origin::Api).unwrap();
        prop_assert_eq!(store.attachment_state(&id).unwrap(), AttachmentState::Sealed);
    }
}

#[test]
fn note_file_lists_are_written_and_must_name_sealed_files_before_lock() {
    let (_dir, store) = store();
    let file = plain(&store, b"plain");
    let book = store
        .put(
            DocType::Book,
            json!({"_id":"book:files", "name":"Files"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let note = store
        .put(
            DocType::Note,
            json!({"_id":"note:files", "title":"Files", "body":"", "bookId":book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    let entry = NoteFile {
        file: file.clone(),
        v: 1,
        seal: "AAAAAAAAAAAAAAAAAAAAAA".into(),
        key: "wrapped".into(),
        extra: Default::default(),
    };
    let listed = store
        .set_note_files(&note.id, std::slice::from_ref(&entry))
        .unwrap();
    assert_eq!(
        store
            .get_api(&note.id, None, false)
            .unwrap()
            .get(KANSO_FILES_FIELD),
        None
    );
    let mut fields = store.get(&note.id).unwrap().unwrap().fields;
    assert_eq!(fields[KANSO_FILES_FIELD][0]["file"], file);
    fields.insert(LOCK_FIELD.into(), json!({"v":1}));
    fields.insert("body".into(), "".into());
    fields.insert("_id".into(), note.id.clone().into());
    fields.insert("_rev".into(), listed.rev.clone().into());
    let err = store
        .lock_note(serde_json::Value::Object(fields))
        .unwrap_err();
    assert!(
        matches!(err, StoreError::Invalid(message) if message == "Every attachment must be sealed before the note is locked.")
    );

    seal(&store, &file, &container(b"sealed"));
    let mut fields = store.get(&note.id).unwrap().unwrap().fields;
    fields.insert(LOCK_FIELD.into(), json!({"v":1}));
    fields.insert("body".into(), "".into());
    fields.insert("_id".into(), note.id.clone().into());
    fields.insert("_rev".into(), listed.rev.into());
    store.lock_note(serde_json::Value::Object(fields)).unwrap();
}

#[test]
fn sealing_and_unsealing_drop_ocr_history_and_transition_markers() {
    let (_dir, store) = store();
    let id = plain(&store, b"image bytes");
    let md5 = store.file_meta(&id).unwrap().unwrap().md5digest.unwrap();
    assert!(
        store
            .set_image_text(&id, &md5, "secret words", "test")
            .unwrap()
    );
    assert_eq!(
        store.image_text(&id).unwrap().as_deref(),
        Some("secret words")
    );
    assert!(!store.revisions(&id).unwrap().is_empty());

    let sealed = container(b"sealed image");
    seal(&store, &id, &sealed);
    let meta = store.file_meta(&id).unwrap().unwrap();
    let expected_seal = FileLock::from_extra(&meta.extra).unwrap().seal;
    assert!(meta.extra.get(OCR_FIELD).is_none(), "{:?}", meta.extra);
    assert_eq!(store.image_text(&id).unwrap(), None);
    assert!(
        !store
            .set_image_text(&id, &md5, "should not return", "test")
            .unwrap()
    );
    assert_eq!(store.image_text(&id).unwrap(), None);
    assert!(store.revisions(&id).unwrap().is_empty());

    let written = store
        .put_unsealed_file(meta, Attachment::new(b"image bytes".to_vec()))
        .unwrap();
    let plain = store.get(&id).unwrap().unwrap();
    assert_eq!(plain.fields[UNSEALED_FIELD]["seal"], expected_seal);
    assert!(plain.fields.get(OCR_FIELD).is_none());
    assert_eq!(store.image_text(&id).unwrap(), None);
    assert!(store.revisions(&id).unwrap().is_empty());

    let mut meta = store.file_meta(&id).unwrap().unwrap();
    FileLock::store(
        Some(&FileLock::sealed("AQEBAQEBAQEBAQEBAQEBAQ".into())),
        &mut meta.extra,
    );
    meta.rev = Some(written.rev);
    store
        .put_sealed_file(meta, Attachment::new(sealed))
        .unwrap();
    let resealed = store.get(&id).unwrap().unwrap();
    assert!(resealed.fields.get(UNSEALED_FIELD).is_none());
    assert!(resealed.fields.get(OCR_FIELD).is_none());
    assert!(store.revisions(&id).unwrap().is_empty());
}

#[test]
fn unsealing_writes_a_plain_blob_and_the_sync_marker() {
    let (_dir, store) = store();
    let id = plain(&store, b"old plain");
    seal(&store, &id, &container(b"sealed"));
    let file = store.file_meta(&id).unwrap().unwrap();
    let seal = FileLock::from_extra(&file.extra).unwrap().seal;
    let written = store
        .put_unsealed_file(file, Attachment::new(b"new plain".to_vec()))
        .unwrap();
    assert_eq!(store.attachment_state(&id).unwrap(), AttachmentState::Plain);
    assert_eq!(
        store.stored_attachment(&id).unwrap().unwrap().data,
        b"new plain"
    );
    let stored = store.get(&id).unwrap().unwrap();
    assert_eq!(stored.fields[UNSEALED_FIELD]["seal"], seal);
    let served = store.get_api(&id, None, true).unwrap();
    assert!(served.get(UNSEALED_FIELD).is_none());
    assert!(served["_attachments"]["index"].get("data").is_some());
    assert_eq!(written.id, id);
}
