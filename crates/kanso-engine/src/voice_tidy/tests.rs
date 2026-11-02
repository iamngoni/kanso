use super::*;
use crate::voice_capture::{Engine as CaptureEngine, VoiceCapture, save_voice_capture};
use kanso_ai::tidy::{Action, Quoted};
use kanso_core::capture_mark::read_tidy;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

const ID: &str = "0123456789abcdef0123456789abcdef";
const SAID: &str = "remind me to call the plumber about the leak on tuesday";

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn saved(store: &Store, id: &str, said: &str, duration_ms: u64) -> String {
    let cap = VoiceCapture {
        id: id.into(),
        started_ms: 1_700_000_000_000,
        duration_ms,
        transcript: said.into(),
        segments: Vec::new(),
        engine: CaptureEngine::SystemOnDevice,
        audio_file: None,
    };
    save_voice_capture(store, &cap, false, false, Origin::Ui)
        .unwrap()
        .unwrap()
        .id
}

fn reply() -> Tidy {
    Tidy {
        title: Some("Plumber".into()),
        actions: vec![
            Action {
                text: "Call the plumber about the leak".into(),
                who: None,
                when: Some("Tuesday".into()),
                quotes: vec!["call the plumber about the leak".into()],
            },
            Action {
                text: "Invented".into(),
                who: None,
                when: None,
                quotes: vec!["never said".into()],
            },
        ],
        decisions: vec![Quoted {
            text: "Nothing".into(),
            quotes: vec![],
        }],
        ..Tidy::default()
    }
}

#[test]
fn tidy_replaces_the_body_keeps_the_transcript_and_marks_it_done() {
    let (_dir, store) = store();
    let id = saved(&store, ID, SAID, 24_000);
    let source = tidy_source(&store, &id, true).unwrap();
    assert_eq!(source.kind, TidyKind::Note);
    let applied = apply_tidy(&store, &source, &reply(), Origin::Ui).unwrap();
    assert_eq!(applied.dropped, 2);
    assert!(!applied.inserted_above);
    let note = store.note(&id).unwrap().unwrap();
    assert_eq!(note.title, "Plumber");
    assert!(
        note.body
            .contains("- [ ] Call the plumber about the leak — Tuesday")
    );
    assert!(!note.body.contains("Invented"));
    assert_eq!(read_tidy(&note.extra), Some(TidyState::Done));
    // The raw transcript is still there, and tidying again reads it.
    let again = tidy_source(&store, &id, true).unwrap();
    assert_eq!(
        again.transcript,
        "Remind me to call the plumber about the leak on tuesday"
    );
    // The earlier version is in the history for Undo.
    assert!(!store.revisions(&id).unwrap().is_empty());
}

#[test]
fn a_note_edited_meanwhile_keeps_the_edit_and_gets_the_tidy_part_above() {
    let (_dir, store) = store();
    let id = saved(&store, ID, SAID, 24_000);
    let source = tidy_source(&store, &id, true).unwrap();
    let body = format!("{}\n\nMy own addition", source.body);
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": body}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let applied = apply_tidy(&store, &source, &reply(), Origin::Ui).unwrap();
    assert!(applied.inserted_above);
    let note = store.note(&id).unwrap().unwrap();
    assert!(note.body.starts_with("# Plumber"));
    assert!(note.body.ends_with("My own addition"));
    assert_ne!(note.title, "Plumber", "a changed note keeps its title");
}

#[test]
fn only_voice_notes_that_may_be_sent_are_tidied() {
    let (_dir, store) = store();
    store
        .role_book(kanso_core::roles::Role::Inbox, Origin::Ui)
        .unwrap();
    let plain = store
        .put(
            DocType::Note,
            json!({"title": "t", "body": "b", "bookId": "book:inbox"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    assert!(matches!(
        tidy_source(&store, &plain, true),
        Err(TidyError::NotVoice)
    ));
    assert!(matches!(
        tidy_source(&store, "note:nope", true),
        Err(TidyError::Missing)
    ));
    let id = saved(&store, ID, SAID, 24_000);
    store
        .put(
            DocType::Note,
            json!({"_id": id, "tags": []}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let tag = store
        .put(
            DocType::Tag,
            json!({"name": "ai-exclude"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    store
        .put(
            DocType::Note,
            json!({"_id": id, "tags": [tag]}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert!(matches!(
        tidy_source(&store, &id, true),
        Err(TidyError::KeptOut)
    ));
}

#[test]
fn a_long_recording_is_a_meeting_only_when_meetings_are_on() {
    let (_dir, store) = store();
    let id = saved(&store, ID, SAID, 5 * 60_000);
    assert_eq!(
        tidy_source(&store, &id, true).unwrap().kind,
        TidyKind::Meeting
    );
    assert_eq!(
        tidy_source(&store, &id, false).unwrap().kind,
        TidyKind::Note
    );
}

proptest! {
    #![proptest_config(Config {
        cases: 16,
        rng_seed: RngSeed::Fixed(0x0074_6964_7902),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// However the model's reply reads, the written note's body still holds
    /// the transcript verbatim and the tasks written all have real quotes.
    #[test]
    fn prop_tidy_keeps_the_transcript_and_only_quoted_tasks(
        said in "[a-z]{3,8}( [a-z]{3,8}){4,12}",
        texts in proptest::collection::vec(("[A-Z][a-z]{3,8}", any::<bool>()), 0..4),
    ) {
        let (_dir, store) = store();
        let id = saved(&store, ID, &said, 10_000);
        let source = tidy_source(&store, &id, true).unwrap();
        let first_words: Vec<&str> = said.split(' ').take(2).collect();
        let real = first_words.join(" ");
        let t = Tidy {
            actions: texts
                .iter()
                .map(|(text, quoted)| Action {
                    text: text.clone(),
                    who: None,
                    when: None,
                    quotes: if *quoted { vec![real.clone()] } else { vec!["zzzz qqqq".into()] },
                })
                .collect(),
            ..Tidy::default()
        };
        apply_tidy(&store, &source, &t, Origin::Ui).unwrap();
        let note = store.note(&id).unwrap().unwrap();
        let split = split_body(&note.body).unwrap();
        prop_assert_eq!(split.transcript.to_lowercase(), said.to_lowercase());
        let tasks = note.body.lines().filter(|l| l.starts_with("- [ ] ")).count();
        prop_assert_eq!(tasks, texts.iter().filter(|(_, q)| *q).count());
    }
}
