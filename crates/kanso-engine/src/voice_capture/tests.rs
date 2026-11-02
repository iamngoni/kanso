use super::*;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};
use serde_json::json;

const ID: &str = "0123456789abcdef0123456789abcdef";

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn capture(id: &str, transcript: &str) -> VoiceCapture {
    VoiceCapture {
        id: id.into(),
        started_ms: 1_700_000_000_000,
        duration_ms: 24_000,
        transcript: transcript.into(),
        segments: Vec::new(),
        engine: Engine::SystemOnDevice,
        audio_file: None,
    }
}

fn lock_inbox(store: &Store) {
    store.role_book(Role::Inbox, Origin::Ui).unwrap();
    let book = json!({"_id": "book:inbox", "name": "Inbox",
        "kansoLock": {"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"}});
    store
        .put(DocType::Book, book, Put::Merge, Origin::Sync)
        .unwrap();
}

#[test]
fn a_capture_lands_in_the_inbox_with_its_record() {
    let (_dir, store) = store();
    let saved = save_voice_capture(
        &store,
        &capture(
            ID,
            "remind me to call the plumber about the leak. and the onboarding idea",
        ),
        false,
        true,
        Origin::Ui,
    )
    .unwrap()
    .unwrap();
    assert_eq!(saved.id, format!("note:cap-{ID}"));
    let note = store.note(&saved.id).unwrap().unwrap();
    assert_eq!(note.book_id, "book:inbox");
    assert_eq!(note.title, "Remind me to call the plumber about the leak");
    assert!(
        note.body
            .starts_with("Remind me to call the plumber about the leak. And the")
    );
    let mark = &note.extra["kansoCapture"];
    assert_eq!(mark["kind"], "voice");
    assert_eq!(mark["engine"], "system-on-device");
    assert_eq!(mark["duration_ms"], 24_000);
    assert_eq!(mark["tidy"], "pending");
}

#[test]
fn nothing_heard_saves_nothing_and_a_bad_id_is_refused() {
    let (_dir, store) = store();
    assert_eq!(
        save_voice_capture(&store, &capture(ID, "  \n "), false, false, Origin::Ui).unwrap(),
        None
    );
    assert!(store.get(&format!("note:cap-{ID}")).unwrap().is_none());
    let err =
        save_voice_capture(&store, &capture("nope", "hi"), false, false, Origin::Ui).unwrap_err();
    assert!(matches!(err, CaptureError::BadId));
}

#[test]
fn a_locked_inbox_refuses_before_writing_anything() {
    let (_dir, store) = store();
    lock_inbox(&store);
    let err =
        save_voice_capture(&store, &capture(ID, "secret"), false, false, Origin::Ui).unwrap_err();
    assert!(matches!(err, CaptureError::Locked), "{err}");
    assert!(store.get(&format!("note:cap-{ID}")).unwrap().is_none());
}

#[test]
fn saving_twice_makes_one_note_and_changes_nothing() {
    let (_dir, store) = store();
    let first = save_voice_capture(&store, &capture(ID, "one thing"), false, false, Origin::Ui)
        .unwrap()
        .unwrap();
    let rev = store.get(&first.id).unwrap().unwrap().rev;
    let again = save_voice_capture(
        &store,
        &capture(ID, "other words"),
        false,
        false,
        Origin::Ui,
    )
    .unwrap()
    .unwrap();
    assert!(again.existed && !first.existed);
    assert_eq!(store.get(&first.id).unwrap().unwrap().rev, rev);
}

#[test]
fn kept_audio_is_attached_once_and_linked_and_big_audio_is_left_out() {
    let (dir, store) = store();
    let audio = dir.path().join("rec.m4a");
    std::fs::write(&audio, b"audio bytes").unwrap();
    let mut cap = capture(ID, "hello there");
    cap.audio_file = Some(audio.clone());
    let saved = save_voice_capture(&store, &cap, true, false, Origin::Ui)
        .unwrap()
        .unwrap();
    let body = store.note(&saved.id).unwrap().unwrap().body;
    assert!(
        body.ends_with(&format!("(inkdrop://{})", audio_file_id(ID))),
        "{body}"
    );
    assert!(store.get(&audio_file_id(ID)).unwrap().is_some());

    // Not asked to keep: no link.
    let other = "f".repeat(32);
    let mut cap = capture(&other, "no audio please");
    cap.audio_file = Some(audio.clone());
    let saved = save_voice_capture(&store, &cap, false, false, Origin::Ui)
        .unwrap()
        .unwrap();
    assert!(
        !store
            .note(&saved.id)
            .unwrap()
            .unwrap()
            .body
            .contains("inkdrop://")
    );

    // Over the limit: the note is saved, the audio is not.
    let big = dir.path().join("big.m4a");
    std::fs::write(&big, vec![0u8; MAX_ATTACHMENT_SIZE + 1]).unwrap();
    let third = "a".repeat(32);
    let mut cap = capture(&third, "long one");
    cap.audio_file = Some(big);
    let saved = save_voice_capture(&store, &cap, true, false, Origin::Ui)
        .unwrap()
        .unwrap();
    assert_eq!(saved.problems.len(), 1);
    assert!(saved.problems[0].contains("over 10 MB"));
    assert!(store.get(&audio_file_id(&third)).unwrap().is_none());
}

#[test]
fn meetings_are_longer_than_three_minutes_when_asked() {
    assert!(is_meeting(MEETING_AFTER_MS + 1, true));
    assert!(!is_meeting(MEETING_AFTER_MS, true));
    assert!(!is_meeting(MEETING_AFTER_MS * 10, false));
}

proptest! {
    #![proptest_config(Config {
        cases: 24,
        rng_seed: RngSeed::Fixed(0x0063_6170_0001),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// Saving the same capture again, whatever it says the second time, leaves
    /// the one note as the first save made it.
    #[test]
    fn prop_a_retry_makes_one_note(
        id in "[0-9a-f]{32}",
        first in "[a-z ]{1,40}",
        second in "[a-z ]{1,40}",
    ) {
        let (_dir, store) = store();
        let one = save_voice_capture(&store, &capture(&id, &first), false, false, Origin::Ui).unwrap();
        let two = save_voice_capture(&store, &capture(&id, &second), false, false, Origin::Ui).unwrap();
        match one {
            None => prop_assert!(first.trim().is_empty() && two.is_some() == !second.trim().is_empty()),
            Some(saved) => {
                prop_assert_eq!(two.unwrap().id, saved.id.clone());
                let body = store.note(&saved.id).unwrap().unwrap().body;
                prop_assert_eq!(
                    body.split_whitespace().map(str::to_lowercase).collect::<Vec<_>>(),
                    first.split_whitespace().map(str::to_lowercase).collect::<Vec<_>>()
                );
            }
        }
    }
}

#[test]
fn spoken_text_saves_where_asked_with_the_record() {
    let (_dir, store) = store();
    let id = new_capture_id();
    assert!(is_drop_id(&id));
    let saved = save_voice_text(
        &store,
        &id,
        "# Plumber\n\n- [ ] Call\n\n<details>x</details>",
        None,
        9_000,
        Engine::Parakeet,
        TidyState::Done,
        Origin::Ui,
    )
    .unwrap()
    .unwrap();
    let note = store.note(&saved.id).unwrap().unwrap();
    assert_eq!(note.title, "Plumber");
    assert_eq!(note.book_id, "book:inbox");
    assert_eq!(note.extra["kansoCapture"]["tidy"], "done");
    assert_eq!(note.extra["kansoCapture"]["engine"], "parakeet");
    let again = save_voice_text(
        &store,
        &id,
        "other",
        None,
        1,
        Engine::Parakeet,
        TidyState::None,
        Origin::Ui,
    )
    .unwrap()
    .unwrap();
    assert!(again.existed);
    // Untidied words keep all of the text and are titled by their first sentence.
    let two = new_capture_id();
    let saved = save_voice_text(
        &store,
        &two,
        "buy milk. and eggs",
        None,
        1,
        Engine::Parakeet,
        TidyState::None,
        Origin::Ui,
    )
    .unwrap()
    .unwrap();
    let note = store.note(&saved.id).unwrap().unwrap();
    assert_eq!(
        (note.title.as_str(), note.body.as_str()),
        ("Buy milk", "buy milk. and eggs")
    );
    // Empty and locked.
    assert_eq!(
        save_voice_text(
            &store,
            &new_capture_id(),
            " ",
            None,
            1,
            Engine::Parakeet,
            TidyState::None,
            Origin::Ui
        )
        .unwrap(),
        None
    );
    lock_inbox(&store);
    assert!(matches!(
        save_voice_text(
            &store,
            &new_capture_id(),
            "x y",
            None,
            1,
            Engine::Parakeet,
            TidyState::None,
            Origin::Ui
        ),
        Err(CaptureError::Locked)
    ));
}
