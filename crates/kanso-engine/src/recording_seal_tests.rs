//! Recording into a locked note (issue #38, step 11): the audio goes in
//! sealed, or not at all.

use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use kanso_core::DocType;
use kanso_core::model::{FileLock, SEALED_FILE_NAME, note_files_from_extra, store_note_files};
use kanso_store::{AttachmentState, Origin, Put, Store};
use kanso_sync::KdfParams;
use kanso_sync::lock::{Envelope, LockKey, LockPassword, SealId, open_container, unwrap_file_key};
use proptest::prelude::*;
use serde_json::json;

use crate::recording_flow::{
    Destination, FlowError, Recorded, check_destination_with, save_recording_or_new_with,
    save_recording_with, save_system_track_with,
};

fn key() -> LockKey {
    LockKey::create(
        &LockPassword::new("correct horse").unwrap(),
        KdfParams::TEST,
    )
    .unwrap()
}

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

/// A locked note holding `body`, owning no files yet.
fn locked_note(store: &Store, key: &LockKey, body: &str) -> String {
    let book = store
        .role_book(kanso_core::roles::Role::Inbox, Origin::Ui)
        .unwrap();
    let made = store
        .put(
            DocType::Note,
            json!({"title": "Private", "body": "", "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    let mut note = store.note(&made.id).unwrap().unwrap();
    Envelope::store(Some(&key.seal(&note.id, body)), &mut note.extra);
    store_note_files(Some(&[]), &mut note.extra);
    store
        .lock_note(serde_json::to_value(&note).unwrap())
        .unwrap();
    made.id
}

fn recorded(dir: &Path, bytes: &[Vec<u8>]) -> Recorded {
    let parts = bytes
        .iter()
        .enumerate()
        .map(|(i, data)| {
            let path = dir.join(format!("part-{i}.opus"));
            std::fs::write(&path, data).unwrap();
            path
        })
        .collect();
    Recorded {
        session_id: "sealed-1".into(),
        parts,
        duration_secs: 600,
        mic_only: true,
        when: "23 Oct 2026, 14:05".into(),
    }
}

/// The text and the bytes of every part of `note_id`, opened with `key`.
fn opened(store: &Store, key: &LockKey, note_id: &str) -> (String, Vec<Vec<u8>>) {
    let note = store.note(note_id).unwrap().unwrap();
    assert!(note.body.is_empty(), "a locked note's stored text is empty");
    let envelope = Envelope::from_extra(&note.extra).unwrap().unwrap();
    let text = key.open(&note.id, &envelope).unwrap().to_string();
    let mut parts = Vec::new();
    for owned in note_files_from_extra(&note.extra).unwrap() {
        let meta = store.file_meta(&owned.file).unwrap().unwrap();
        let marker = FileLock::from_extra(&meta.extra).unwrap();
        assert_eq!(marker.seal, owned.seal);
        let seal = SealId::decode(&marker.seal).unwrap();
        let wrapped = STANDARD.decode(&owned.key).unwrap();
        let file_key =
            unwrap_file_key(key, &envelope, &note.id, &owned.file, &seal, &wrapped).unwrap();
        let stored = store.stored_attachment(&owned.file).unwrap().unwrap();
        let segments = open_container(&owned.file, &seal, &file_key, &stored.data).unwrap();
        parts.push(segments[0].bytes.clone());
    }
    (text, parts)
}

#[test]
fn a_recording_in_a_locked_note_is_sealed_and_opens_with_the_key() {
    let (dir, store) = store();
    let key = key();
    let id = locked_note(&store, &key, "My private prose");
    let dest = Destination::ThisNote {
        note_id: id.clone(),
    };
    let audio = vec![b"OggS first part".to_vec(), b"OggS second part".to_vec()];
    let rec = recorded(dir.path(), &audio);
    // No key: refused up front, and nothing is written.
    assert!(matches!(
        check_destination_with(&store, &dest, None),
        Err(FlowError::Locked)
    ));
    assert!(matches!(
        save_recording_with(&store, &rec, &dest, None, Origin::Ui),
        Err(FlowError::Locked)
    ));
    assert!(store.get("file:rec-sealed-1-1").unwrap().is_none());
    // With the key it goes in sealed.
    assert!(check_destination_with(&store, &dest, Some(&key)).is_ok());
    let saved = save_recording_with(&store, &rec, &dest, Some(&key), Origin::Ui).unwrap();
    assert_eq!(saved.note_id, id);
    let (text, parts) = opened(&store, &key, &id);
    assert!(text.starts_with("My private prose"));
    assert!(text.contains("file:rec-sealed-1-1") && text.contains("file:rec-sealed-1-2"));
    assert_eq!(parts, audio);
    for file in &saved.file_ids {
        assert_eq!(
            store.attachment_state(file).unwrap(),
            AttachmentState::Sealed
        );
        assert!(store.attachment(file).unwrap().is_none());
        let meta = store.file_meta(file).unwrap().unwrap();
        assert_eq!(meta.name, SEALED_FILE_NAME);
    }
    // Saving again adds nothing; a system track lands on its own line.
    let again = save_recording_with(&store, &rec, &dest, Some(&key), Origin::Ui).unwrap();
    assert_eq!(again.file_ids, saved.file_ids);
    let track = dir.path().join("sys.opus");
    std::fs::write(&track, b"OggS system").unwrap();
    save_system_track_with(
        &store,
        &id,
        "sealed-1-sys",
        &[track],
        600,
        Some(&key),
        Origin::Ui,
    )
    .unwrap();
    let (text, parts) = opened(&store, &key, &id);
    assert_eq!(text.matches("file:rec-sealed-1-1").count(), 1);
    assert!(text.contains("System audio"));
    assert_eq!(parts.len(), 3);
}

#[test]
fn a_locked_note_with_no_key_refuses_the_audio_and_no_plain_note_takes_it() {
    let (dir, store) = store();
    let key = key();
    let id = locked_note(&store, &key, "My private prose");
    let dest = Destination::ThisNote {
        note_id: id.clone(),
    };
    let rec = recorded(dir.path(), &[b"OggS x".to_vec()]);
    let notes_before = store.search(&Default::default()).unwrap().len();
    assert!(matches!(
        save_recording_or_new_with(&store, &rec, &dest, None, Origin::Ui),
        Err(FlowError::Locked)
    ));
    // Nothing was stored: no new note, no part, plain or sealed.
    assert_eq!(
        store.search(&Default::default()).unwrap().len(),
        notes_before
    );
    assert!(
        store
            .get(&crate::recording_flow::part_file_id(&rec.session_id, 0))
            .unwrap()
            .is_none()
    );
    let (_, parts) = opened(&store, &key, &id);
    assert!(parts.is_empty());
    // A note that is gone still sends the audio to a new note.
    let gone = Destination::ThisNote {
        note_id: "note:gone".into(),
    };
    let saved = save_recording_or_new_with(&store, &rec, &gone, None, Origin::Ui).unwrap();
    assert!(saved.created);
}

/// A note locked before sealing existed links a file that is still plain:
/// the recording's save seals it with the audio (step 3's locked-note
/// save), where it used to be refused.
#[test]
fn a_note_locked_before_sealing_seals_its_files_with_the_recording() {
    let (dir, store) = store();
    let key = key();
    let picture = store
        .put(
            DocType::File,
            json!({"name": "p.png", "contentType": "image/png", "_attachments": {"index": {
                "content_type": "image/png",
                "data": STANDARD.encode(b"plain picture"),
            }}}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let id = locked_note(&store, &key, &format!("See ![](inkdrop://{picture}) here"));
    // Drop the list, as a note locked by an older Kanso has none.
    let mut note = store.note(&id).unwrap().unwrap();
    store_note_files(None, &mut note.extra);
    store
        .lock_note(serde_json::to_value(&note).unwrap())
        .unwrap();
    let dest = Destination::ThisNote {
        note_id: id.clone(),
    };
    assert!(check_destination_with(&store, &dest, Some(&key)).is_ok());
    let rec = recorded(dir.path(), &[b"OggS x".to_vec()]);
    save_recording_with(&store, &rec, &dest, Some(&key), Origin::Ui).unwrap();
    assert_eq!(
        store.attachment_state(&picture).unwrap(),
        AttachmentState::Sealed
    );
    let (text, parts) = opened(&store, &key, &id);
    assert!(text.contains(&picture));
    let mut parts = parts;
    parts.sort();
    assert_eq!(parts, vec![b"OggS x".to_vec(), b"plain picture".to_vec()]);
}

/// A recording into a new note in a locked notebook: with the key that
/// opens the notebook, the note is made sealed and takes the audio sealed;
/// without it, nothing is stored.
#[test]
fn a_new_note_in_a_locked_notebook_takes_a_recording_sealed() {
    let (dir, store) = store();
    let key = key();
    let book = store
        .put(
            DocType::Book,
            json!({"name": "Private"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    crate::lock::seal_notebook(&store, &book, &key).unwrap();
    let dest = Destination::NewNote {
        book_id: Some(book.clone()),
    };
    let rec = recorded(dir.path(), &[b"OggS secret words".to_vec()]);
    let notes_before = store.search(&Default::default()).unwrap().len();
    assert!(matches!(
        save_recording_with(&store, &rec, &dest, None, Origin::Ui),
        Err(FlowError::Locked)
    ));
    assert_eq!(
        store.search(&Default::default()).unwrap().len(),
        notes_before
    );

    let saved = save_recording_with(&store, &rec, &dest, Some(&key), Origin::Ui).unwrap();
    assert!(saved.created);
    let note = store.note(&saved.note_id).unwrap().unwrap();
    assert_eq!(note.book_id, book);
    assert!(note.is_locked());
    let (text, parts) = opened(&store, &key, &saved.note_id);
    assert!(text.contains(&saved.file_ids[0]));
    assert_eq!(parts, vec![b"OggS secret words".to_vec()]);
    assert!(store.attachment(&saved.file_ids[0]).unwrap().is_none());
    // Saving again adds nothing.
    let again = save_recording_with(&store, &rec, &dest, Some(&key), Origin::Ui).unwrap();
    assert!(!again.created);
    let (text, _) = opened(&store, &key, &saved.note_id);
    assert_eq!(text.matches(&saved.file_ids[0]).count(), 1);
}

#[test]
fn a_note_locked_before_sealing_that_links_nothing_takes_a_recording() {
    let (dir, store) = store();
    let key = key();
    let id = locked_note(&store, &key, "Just words");
    let mut note = store.note(&id).unwrap().unwrap();
    store_note_files(None, &mut note.extra);
    store
        .lock_note(serde_json::to_value(&note).unwrap())
        .unwrap();
    let dest = Destination::ThisNote {
        note_id: id.clone(),
    };
    let rec = recorded(dir.path(), &[b"OggS x".to_vec()]);
    save_recording_with(&store, &rec, &dest, Some(&key), Origin::Ui).unwrap();
    let (_, parts) = opened(&store, &key, &id);
    assert_eq!(parts, vec![b"OggS x".to_vec()]);
}

#[test]
fn a_transcript_goes_into_a_locked_note_sealed_with_its_files_kept() {
    let (dir, store) = store();
    let key = key();
    let id = locked_note(&store, &key, "My private prose");
    let dest = Destination::ThisNote {
        note_id: id.clone(),
    };
    let rec = recorded(dir.path(), &[b"OggS x".to_vec()]);
    save_recording_with(&store, &rec, &dest, Some(&key), Origin::Ui).unwrap();
    let words = "Secret words spoken";
    let add = |body: &str| Some(format!("{body}\n\n{words}\n"));
    // No key: refused, and the note is as it was.
    let before = store.note(&id).unwrap().unwrap();
    assert!(matches!(
        crate::recording_seal::edit_text(&store, &id, None, add, Origin::Ui),
        Err(FlowError::Locked)
    ));
    assert_eq!(store.note(&id).unwrap().unwrap().rev, before.rev);
    // With the key: sealed in, never stored plain, and the audio still opens.
    assert!(crate::recording_seal::edit_text(&store, &id, Some(&key), add, Origin::Ui).unwrap());
    let stored = store.get(&id).unwrap().unwrap();
    assert!(
        !serde_json::to_string(&stored.to_api(Default::default()))
            .unwrap()
            .contains(words)
    );
    let (text, parts) = opened(&store, &key, &id);
    assert!(text.contains("My private prose") && text.contains(words));
    assert_eq!(parts, vec![b"OggS x".to_vec()]);
    // Nothing to change: nothing is written.
    let rev = store.note(&id).unwrap().unwrap().rev;
    assert!(
        !crate::recording_seal::edit_text(&store, &id, Some(&key), |_| None, Origin::Ui).unwrap()
    );
    assert_eq!(store.note(&id).unwrap().unwrap().rev, rev);
}

#[test]
fn another_locks_key_does_not_open_the_note() {
    let (dir, store) = store();
    let id = locked_note(&store, &key(), "prose");
    let other = key();
    let dest = Destination::ThisNote { note_id: id };
    let rec = recorded(dir.path(), &[b"OggS x".to_vec()]);
    assert!(matches!(
        save_recording_with(&store, &rec, &dest, Some(&other), Origin::Ui),
        Err(FlowError::Locked)
    ));
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 24, rng_seed: proptest::test_runner::RngSeed::Fixed(20261017), ..ProptestConfig::default() })]

    /// Recording into a locked note never leaves the audio readable in the
    /// library: the stored bytes hold no run of the plain audio, and they
    /// open, with the key, to exactly what was recorded.
    #[test]
    fn prop_a_locked_notes_audio_is_never_stored_plain(
        audio in proptest::collection::vec(proptest::collection::vec(any::<u8>(), 24..600), 1..4),
        prose in "[a-z ]{0,30}",
    ) {
        let (dir, store) = store();
        let key = key();
        let id = locked_note(&store, &key, &prose);
        let dest = Destination::ThisNote { note_id: id.clone() };
        let rec = recorded(dir.path(), &audio);
        let saved = save_recording_with(&store, &rec, &dest, Some(&key), Origin::Ui).unwrap();
        for (file, plain) in saved.file_ids.iter().zip(&audio) {
            let stored = store.stored_attachment(file).unwrap().unwrap().data;
            let run = &plain[..16];
            prop_assert!(!stored.windows(run.len()).any(|w| w == run), "plain audio in the blob");
        }
        let (text, parts) = opened(&store, &key, &id);
        prop_assert!(text.starts_with(prose.trim_end()));
        prop_assert_eq!(parts, audio);
    }
}
