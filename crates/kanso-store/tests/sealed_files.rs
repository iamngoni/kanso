//! Sealed attachments (issue #44 step 1): a file document marked with
//! `kansoFileLock` holds a sealed container in `blobs`, the ordinary write
//! path can neither replace its bytes nor drop the marker, imports and
//! served output strip the marker, and the store reads the states back.

use base64::Engine as _;
use kanso_core::DocType;
use kanso_core::model::{FILE_LOCK_FIELD, FileLock, MAX_ATTACHMENT_SIZE, MAX_SEALED_ATTACHMENT};
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
    let listed = store.list_api(DocType::File, None, 0).unwrap();
    assert!(listed[0].get(FILE_LOCK_FIELD).is_none());
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
    assert_eq!(meta.name, "renamed.png");

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

    // A malformed marker is not a seal: version, kind, metadata state and the
    // canonical 16-byte seal ID must all be exact.
    for marker in [
        json!({"v": 2, "kind": "file", "seal": "AAAAAAAAAAAAAAAAAAAAAA", "meta": false}),
        json!({"v": 1, "kind": "note", "seal": "AAAAAAAAAAAAAAAAAAAAAA", "meta": false}),
        json!({"v": 1, "kind": "file", "seal": "AAAAAAAAAAAAAAAAAAAAAA", "meta": true}),
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
