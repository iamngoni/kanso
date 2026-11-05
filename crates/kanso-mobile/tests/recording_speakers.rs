//! Rename Speakers on a phone (issue #38, step 13): the names listed, a
//! write of just the changed ones with one Undo, and no write for a note
//! with nobody to rename or a locked one.

mod common;

use common::*;
use kanso_core::DocType;
use kanso_engine::recording_note::{Segment, render_transcript};
use kanso_mobile::{KansoError, SpeakerChange};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};
use serde_json::json;

const FILE: &str = "file:abc123";

fn body_with(names: &[Option<&str>]) -> String {
    let segments: Vec<Segment> = names
        .iter()
        .enumerate()
        .map(|(i, who)| Segment {
            start_secs: i as u32 * 20,
            text: format!("Paragraph {i}."),
            speaker: who.map(str::to_owned),
        })
        .collect();
    let transcript = render_transcript(FILE, &segments).unwrap();
    format!("# Meeting\n\n[Audio · 2 min](inkdrop://{FILE})\n\n{transcript}\n\nMy notes.")
}

fn change(from: &str, to: &str) -> SpeakerChange {
    SpeakerChange {
        from: from.into(),
        to: to.into(),
    }
}

// no-prop-test: one fixed note, listed and renamed.
#[test]
fn the_names_are_listed_in_order_and_renaming_writes_one_undoable_change() {
    let phone = lone_phone();
    let body = body_with(&[
        Some("You"),
        Some("Speaker 1"),
        Some("Speaker 2"),
        Some("Speaker 1"),
    ]);
    let id = note(&phone.library, "Meeting", &body);
    let names = phone.library.speaker_names(&id).unwrap();
    let shown: Vec<_> = names
        .iter()
        .map(|n| (n.name.as_str(), n.paragraphs))
        .collect();
    assert_eq!(shown, vec![("You", 1), ("Speaker 1", 2), ("Speaker 2", 1)]);

    let renamed = phone
        .library
        .rename_speakers(&id, vec![change("Speaker 1", "Priya")])
        .unwrap();
    let store = Store::open(phone.path()).unwrap();
    let after = store.note(&id).unwrap().unwrap().body;
    assert_eq!(after.matches("Priya: Paragraph").count(), 2);
    assert!(after.contains("Speaker 2: Paragraph 2") && after.ends_with("My notes."));

    let ticket = renamed.undo.expect("one Undo");
    assert_eq!(phone.library.undo(ticket).unwrap(), 1);
    assert_eq!(store.note(&id).unwrap().unwrap().body, body);
}

// no-prop-test: three notes with nothing to rename, and a locked one.
#[test]
fn nobody_to_rename_is_an_empty_list_and_a_locked_note_is_refused() {
    let phone = lone_phone();
    let plain = note(&phone.library, "Plain", "words");
    assert!(phone.library.speaker_names(&plain).unwrap().is_empty());
    let silent = note(&phone.library, "Silent", &body_with(&[None, None]));
    assert!(phone.library.speaker_names(&silent).unwrap().is_empty());
    assert!(phone.library.speaker_names("note:nope").unwrap().is_empty());
    assert!(
        phone
            .library
            .rename_speakers(&silent, vec![change("Speaker 1", "Priya")])
            .is_err()
    );

    let locked = note(&phone.library, "Locked", &body_with(&[Some("Speaker 1")]));
    let store = Store::open(phone.path()).unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": locked, "kansoLock": {"salt": "abc"}}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    assert!(phone.library.speaker_names(&locked).unwrap().is_empty());
    assert_eq!(
        phone
            .library
            .rename_speakers(&locked, vec![change("Speaker 1", "Priya")])
            .unwrap_err(),
        KansoError::Locked
    );
}

proptest! {
    #![proptest_config(Config {
        cases: 24,
        rng_seed: RngSeed::Fixed(0x0073_7065_616b),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// Whatever is renamed, every paragraph is still there, the prose
    /// around the transcript is untouched, and Undo restores the note.
    #[test]
    fn prop_renaming_keeps_every_paragraph_and_undo_restores(
        who in proptest::collection::vec(0usize..3, 1..8),
        to in "[A-Z][a-z]{2,8}",
        target in 0usize..3,
    ) {
        let phone = lone_phone();
        let labels: Vec<String> = who.iter().map(|w| format!("Speaker {}", w + 1)).collect();
        let refs: Vec<Option<&str>> = labels.iter().map(|l| Some(l.as_str())).collect();
        let body = body_with(&refs);
        let id = note(&phone.library, "Meeting", &body);
        let from = format!("Speaker {}", target + 1);
        match phone.library.rename_speakers(&id, vec![change(&from, &to)]) {
            Err(_) => prop_assert!(!labels.contains(&from)),
            Ok(done) => {
                let store = Store::open(phone.path()).unwrap();
                let after = store.note(&id).unwrap().unwrap().body;
                prop_assert_eq!(after.matches("](kanso-audio://").count(), labels.len());
                prop_assert!(after.starts_with("# Meeting\n\n[Audio"));
                prop_assert!(after.ends_with("\n\nMy notes."));
                prop_assert_eq!(after.matches(&format!("{to}: Paragraph")).count(),
                    labels.iter().filter(|l| **l == from).count());
                phone.library.undo(done.undo.unwrap()).unwrap();
                prop_assert_eq!(store.note(&id).unwrap().unwrap().body, body);
            }
        }
    }
}
