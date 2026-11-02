//! Voice capture on a phone (issue #193): a voice note lands in the Inbox,
//! recordings left outside the app wait to be transcribed and become notes
//! once, the drop-saving pass leaves them alone, and Tidy writes only what
//! the recording backs and never sends a note kept out of AI.

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use common::model::*;
use common::*;
use kanso_core::DocType;
use kanso_mobile::capture::save_inbox;
use kanso_mobile::{AiRun, KEPT_OUT, KansoError, VoiceCaptureInput, VoiceSettings, VoiceTidy};
use kanso_store::{Origin, Put, Store};
use kanso_sync::MemoryServer;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};
use serde_json::json;

const ID: &str = "0123456789abcdef0123456789abcdef";
const SAID: &str = "remind me to call the plumber about the leak on tuesday";

fn inbox_of(phone: &Phone) -> PathBuf {
    phone.dir.path().join("inbox")
}

fn input(id: &str, said: &str, duration_ms: u64) -> VoiceCaptureInput {
    VoiceCaptureInput {
        id: id.into(),
        started_ms: 1_700_000_000_000,
        duration_ms,
        transcript: said.into(),
        on_device: true,
    }
}

/// A voice drop: `inbox/<id>/` with `rec.m4a` and a `drop.json`.
fn write_voice_drop(inbox: &Path, id: &str, duration_ms: u64) -> PathBuf {
    let folder = inbox.join(id);
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("rec.m4a"), b"audio bytes").unwrap();
    let manifest = json!({"version": 1, "kind": "voice", "createdAt": 1_700_000_000_000i64,
        "started_ms": 1_700_000_000_000i64, "duration_ms": duration_ms, "audio": "rec.m4a"});
    fs::write(folder.join("drop.json"), manifest.to_string()).unwrap();
    folder
}

fn inbox_notes(store: &Store) -> Vec<String> {
    store
        .notes_in_books(&["book:inbox".into()])
        .unwrap()
        .into_iter()
        .map(|n| n.id)
        .collect()
}

// no-prop-test: one fixed capture saved, then saved again.
#[test]
fn a_phone_voice_note_lands_in_the_inbox_once() {
    let phone = lone_phone();
    // A default notebook that is not the Inbox must not receive it.
    book(&Store::open(phone.path()).unwrap(), "Elsewhere");
    let saved = phone.library.save_voice(input(ID, SAID, 24_000)).unwrap();
    let note_id = saved.note_id.clone().unwrap();
    assert_eq!(note_id, format!("note:cap-{ID}"));
    assert!(!saved.meeting && !saved.tidy_now);
    let store = Store::open(phone.path()).unwrap();
    let note = store.note(&note_id).unwrap().unwrap();
    assert_eq!(note.book_id, "book:inbox");
    assert_eq!(note.extra["kansoCapture"]["engine"], "system-on-device");
    let again = phone.library.save_voice(input(ID, "different", 1)).unwrap();
    assert!(again.existed);
    assert_eq!(inbox_notes(&store).len(), 1);

    let nothing = phone
        .library
        .save_voice(input(&"b".repeat(32), "  ", 1))
        .unwrap();
    assert_eq!(nothing.note_id, None);

    let mut server = input(&"c".repeat(32), "heard by a server", 1);
    server.on_device = false;
    let saved = phone.library.save_voice(server).unwrap();
    let note = store.note(&saved.note_id.unwrap()).unwrap().unwrap();
    assert_eq!(note.extra["kansoCapture"]["engine"], "system-server");
}

// no-prop-test: one fixed locked Inbox.
#[test]
fn a_locked_inbox_refuses_a_voice_note() {
    let phone = lone_phone();
    let store = Store::open(phone.path()).unwrap();
    store
        .role_book(kanso_core::roles::Role::Inbox, Origin::Ui)
        .unwrap();
    store
        .put(
            DocType::Book,
            json!({"_id": "book:inbox", "name": "Inbox", "kansoLock":
                {"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"}}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    assert_eq!(
        phone.library.save_voice(input(ID, SAID, 5_000)),
        Err(KansoError::Locked)
    );
}

// no-prop-test: a fixed drop through its whole life.
#[test]
fn a_voice_drop_waits_for_the_app_then_becomes_one_note() {
    let phone = lone_phone();
    let inbox = inbox_of(&phone);
    let folder = write_voice_drop(&inbox, ID, 24_000);
    // The pass that saves shares leaves the recording alone, untouched and uncounted.
    let saved = save_inbox(
        &phone.path(),
        &inbox,
        phone.secrets.clone(),
        Arc::new(MemoryServer::new()),
    )
    .unwrap();
    assert!(saved.notes.is_empty() && saved.kept == 0 && saved.problems.is_empty());
    assert!(folder.join("rec.m4a").exists());

    let waiting = phone.library.voice_drops(&inbox).unwrap();
    assert_eq!(waiting.len(), 1);
    assert_eq!(waiting[0].id, ID);
    assert_eq!(waiting[0].duration_ms, 24_000);
    assert!(waiting[0].audio_path.ends_with("rec.m4a"));

    let saved = phone
        .library
        .save_voice_drop(&inbox, ID, SAID.into(), true)
        .unwrap();
    assert!(saved.note_id.is_some());
    assert!(!folder.exists(), "the recording goes once it is a note");
    assert!(phone.library.voice_drops(&inbox).unwrap().is_empty());
    let store = Store::open(phone.path()).unwrap();
    // Short capture, audio kept off: the words only.
    assert!(
        !store
            .note(&saved.note_id.unwrap())
            .unwrap()
            .unwrap()
            .body
            .contains("inkdrop://")
    );
}

// no-prop-test: three fixed recordings: kept by setting, kept as a meeting, and not kept.
#[test]
fn audio_is_kept_for_meetings_and_when_asked_within_the_limit() {
    let phone = lone_phone();
    let inbox = inbox_of(&phone);
    let store = Store::open(phone.path()).unwrap();
    let body = |id: &str| store.note(&format!("note:cap-{id}")).unwrap().unwrap().body;

    let short = "a".repeat(32);
    write_voice_drop(&inbox, &short, 10_000);
    phone
        .library
        .save_voice_drop(&inbox, &short, "short".into(), true)
        .unwrap();
    assert!(!body(&short).contains("inkdrop://"));

    let long = "b".repeat(32);
    write_voice_drop(&inbox, &long, 5 * 60_000);
    let saved = phone
        .library
        .save_voice_drop(&inbox, &long, "a meeting".into(), true)
        .unwrap();
    assert!(saved.meeting);
    assert!(body(&long).contains(&format!("inkdrop://file:cap-{long}-audio")));

    phone
        .library
        .set_voice_settings(VoiceSettings {
            keep_audio: true,
            ..VoiceSettings::default()
        })
        .unwrap();
    let kept = "c".repeat(32);
    write_voice_drop(&inbox, &kept, 10_000);
    phone
        .library
        .save_voice_drop(&inbox, &kept, "kept".into(), true)
        .unwrap();
    assert!(body(&kept).contains("inkdrop://file:cap-"));
}

// no-prop-test: an empty transcript and a missing drop, once each.
#[test]
fn a_recording_with_no_words_is_removed_and_a_missing_one_is_not_found() {
    let phone = lone_phone();
    let inbox = inbox_of(&phone);
    let folder = write_voice_drop(&inbox, ID, 3_000);
    let saved = phone
        .library
        .save_voice_drop(&inbox, ID, " ".into(), true)
        .unwrap();
    assert_eq!(saved.note_id, None);
    assert!(!folder.exists());
    assert_eq!(
        phone.library.save_voice_drop(&inbox, ID, "x".into(), true),
        Err(KansoError::NotFound)
    );
    write_voice_drop(&inbox, ID, 3_000);
    phone.library.discard_voice_drop(&inbox, ID).unwrap();
    assert!(phone.library.voice_drops(&inbox).unwrap().is_empty());
}

proptest! {
    #![proptest_config(Config {
        cases: 12,
        rng_seed: RngSeed::Fixed(0x0076_6f69_6332),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// Whatever recordings wait, the pass that saves shares never touches
    /// them, and saving each makes exactly one note.
    #[test]
    fn prop_recordings_survive_the_share_pass_and_make_one_note_each(
        ids in proptest::collection::hash_set("[0-9a-f]{32}", 1..4),
        words in "[a-z]{3,9}( [a-z]{3,9}){0,5}",
    ) {
        let phone = lone_phone();
        let inbox = inbox_of(&phone);
        for id in &ids {
            write_voice_drop(&inbox, id, 4_000);
        }
        save_inbox(
            &phone.path(),
            &inbox,
            phone.secrets.clone(),
            Arc::new(MemoryServer::new()),
        ).unwrap();
        prop_assert_eq!(phone.library.voice_drops(&inbox).unwrap().len(), ids.len());
        for id in &ids {
            phone.library.save_voice_drop(&inbox, id, words.clone(), true).unwrap();
            // A second go is not found: the recording is gone, the note stays.
            prop_assert_eq!(
                phone.library.save_voice_drop(&inbox, id, words.clone(), true),
                Err(KansoError::NotFound)
            );
        }
        let store = Store::open(phone.path()).unwrap();
        prop_assert_eq!(inbox_notes(&store).len(), ids.len());
    }
}

const REPLY: &str = r#"{"title": "Plumber", "summary": [], "decisions": [],
 "actions": [{"text": "Call the plumber about the leak", "who": null, "when": "Tuesday",
              "quotes": ["call the plumber about the leak"]},
             {"text": "Invented", "quotes": ["never said"]}],
 "sections": []}"#;

fn voice_note(phone: &Phone, ai: &kanso_mobile::AiHandle) -> String {
    let _ = ai;
    phone
        .library
        .save_voice(input(ID, SAID, 24_000))
        .unwrap()
        .note_id
        .unwrap()
}

// no-prop-test: a scripted model with one fixed reply.
#[test]
fn tidy_writes_what_the_recording_backs_and_info_follows() {
    let model = Scripted::new(REPLY);
    let (phone, ai) = ai_phone(&model);
    let id = voice_note(&phone, &ai);
    let info = phone.library.voice_note_info(&id).unwrap().unwrap();
    assert_eq!((info.tidy.as_str(), info.can_tidy), ("none", true));
    let outcome = ai.tidy_note(&id, &AiRun::new()).unwrap();
    assert_eq!(outcome.dropped, 1);
    assert!(!outcome.inserted_above);
    let store = Store::open(phone.path()).unwrap();
    let note = store.note(&id).unwrap().unwrap();
    assert_eq!(note.title, "Plumber");
    assert!(
        note.body
            .contains("- [ ] Call the plumber about the leak — Tuesday")
    );
    assert!(!note.body.contains("Invented"));
    assert!(note.body.contains("<details>"));
    let info = phone.library.voice_note_info(&id).unwrap().unwrap();
    assert_eq!((info.tidy.as_str(), info.can_tidy), ("done", false));
    // Only the transcript and a title went to the model.
    assert!(model.everything_sent().contains("plumber"));
}

// no-prop-test: three fixed refusals, each before anything is sent.
#[test]
fn tidy_refuses_before_sending_when_the_note_may_not_go() {
    let model = Scripted::new(REPLY);
    let (phone, ai) = ai_phone(&model);
    let id = voice_note(&phone, &ai);
    let store = Store::open(phone.path()).unwrap();
    let tag = store.tag_named("ai-exclude", Origin::Ui).unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": id, "tags": [tag]}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(
        ai.tidy_note(&id, &AiRun::new()),
        Err(KansoError::invalid(KEPT_OUT))
    );
    assert_eq!(model.request_count(), 0);
    let info = phone.library.voice_note_info(&id).unwrap().unwrap();
    assert!(!info.can_tidy);
    // A note that isn't a voice note.
    let plain = note(&phone.library, "Plain", "words");
    assert!(ai.tidy_note(&plain, &AiRun::new()).is_err());
    assert_eq!(phone.library.voice_note_info(&plain).unwrap(), None);
    assert_eq!(model.request_count(), 0);
}

// no-prop-test: one unreadable reply.
#[test]
fn a_reply_that_is_not_understood_changes_nothing() {
    let model = Scripted::new("not json");
    let (phone, ai) = ai_phone(&model);
    let id = voice_note(&phone, &ai);
    let store = Store::open(phone.path()).unwrap();
    let before = store.note(&id).unwrap().unwrap();
    let err = ai.tidy_note(&id, &AiRun::new()).unwrap_err();
    assert_eq!(err, KansoError::invalid("The reply wasn't understood."));
    let after = store.note(&id).unwrap().unwrap();
    assert_eq!((before.body, before.rev), (after.body, after.rev));
}

// no-prop-test: Always plus AI on marks a capture to tidy now; Off never does.
#[test]
fn always_marks_a_new_capture_to_be_tidied_and_off_does_not() {
    let model = Scripted::new(REPLY);
    let (phone, _ai) = ai_phone(&model);
    phone
        .library
        .set_voice_settings(VoiceSettings {
            tidy: VoiceTidy::Always,
            ..VoiceSettings::default()
        })
        .unwrap();
    let saved = phone.library.save_voice(input(ID, SAID, 5_000)).unwrap();
    assert!(saved.tidy_now);
    let store = Store::open(phone.path()).unwrap();
    let id = saved.note_id.unwrap();
    assert_eq!(
        store.note(&id).unwrap().unwrap().extra["kansoCapture"]["tidy"],
        "pending"
    );
    phone
        .library
        .set_voice_settings(VoiceSettings {
            tidy: VoiceTidy::Off,
            ..VoiceSettings::default()
        })
        .unwrap();
    let off = phone
        .library
        .save_voice(input(&"d".repeat(32), SAID, 5_000))
        .unwrap();
    assert!(!off.tidy_now);
}
