//! Summarise Recording on a phone (issue #38, step 12): the offer, what is
//! sent, the draft holding only what the recording backs, a write of just
//! the ticked action items with one Undo, and no request for a note kept
//! out of AI.

mod common;

use common::model::*;
use common::*;
use kanso_core::DocType;
use kanso_engine::recording_note::{Segment, render_transcript};
use kanso_mobile::{AiRun, KEPT_OUT, KansoError};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};
use serde_json::json;

const FILE: &str = "file:abc123";
const REPLY: &str = r#"{"summary": ["The beta ships on the sixth."],
 "decisions": [{"text": "Ship the beta on the 6th", "quotes": ["ship it on the sixth"]}],
 "actions": [{"text": "Send the onboarding draft", "who": "Priya", "when": "by the 30th",
              "quotes": ["send the onboarding draft"]},
             {"text": "Call the board", "who": null, "when": null, "quotes": ["call the board"]},
             {"text": "Invented", "quotes": ["never said"]}]}"#;

fn body() -> String {
    let seg = |start_secs, text: &str| Segment {
        start_secs,
        text: text.into(),
        speaker: None,
    };
    let transcript = render_transcript(
        FILE,
        &[
            seg(0, "We started with the roadmap."),
            seg(22, "The beta is on track, so ship it on the sixth."),
            seg(47, "Priya will send the onboarding draft by the thirtieth."),
            seg(70, "And somebody has to call the board."),
        ],
    )
    .unwrap();
    format!("# Meeting\n\n[Audio · 2 min](inkdrop://{FILE})\n\n{transcript}")
}

// no-prop-test: one fixed note, read for the offer.
#[test]
fn the_offer_follows_the_note_and_ai() {
    let model = Scripted::new(REPLY);
    let (phone, ai) = ai_phone(&model);
    let id = note(&phone.library, "Meeting", &body());
    let offer = ai.summary_offer(&id).unwrap();
    assert!(offer.available && offer.words > 20 && !offer.replaces);
    assert!(
        offer
            .send_line
            .contains("The audio is not sent. The transcript is."),
        "{}",
        offer.send_line
    );
    let plain = note(&phone.library, "Plain", "words");
    assert!(!ai.summary_offer(&plain).unwrap().available);
    assert!(!ai.summary_offer("note:nope").unwrap().available);
    assert_eq!(model.request_count(), 0, "an offer sends nothing");
}

// no-prop-test: one fixed reply; the engine has the properties.
#[test]
fn a_draft_holds_only_what_the_recording_backs_and_writes_nothing() {
    let model = Scripted::new(REPLY);
    let (phone, ai) = ai_phone(&model);
    let id = note(&phone.library, "Meeting", &body());
    let draft = ai.summarize_recording(&id, &AiRun::new()).unwrap();
    assert_eq!((draft.decisions.len(), draft.actions.len()), (1, 2));
    assert_eq!(draft.dropped, 1);
    assert_eq!(draft.actions[0].moment, 47);
    let sent = model.everything_sent();
    assert!(sent.contains("[00:22] The beta is on track"));
    assert!(!sent.contains("inkdrop") && !sent.contains("kanso-audio"));
    let store = Store::open(phone.path()).unwrap();
    assert_eq!(store.note(&id).unwrap().unwrap().body, body());
}

// no-prop-test: one fixed summary, written and taken back.
#[test]
fn apply_writes_the_ticked_items_and_undo_puts_the_note_back() {
    let model = Scripted::new(REPLY);
    let (phone, ai) = ai_phone(&model);
    let id = note(&phone.library, "Meeting", &body());
    let draft = ai.summarize_recording(&id, &AiRun::new()).unwrap();
    let applied = ai.apply_recording_summary(&id, draft, vec![0]).unwrap();
    assert!(!applied.replaced);
    let store = Store::open(phone.path()).unwrap();
    let after = store.note(&id).unwrap().unwrap().body;
    assert!(after.contains("- [ ] Send the onboarding draft — Priya, by the 30th"));
    assert!(!after.contains("Call the board"));
    assert!(after.find("kanso:summary").unwrap() < after.find("kanso:transcript").unwrap());
    let ticket = applied.undo.expect("one Undo");
    assert_eq!(phone.library.undo(ticket).unwrap(), 1);
    assert_eq!(store.note(&id).unwrap().unwrap().body, body());
    assert!(ai.summary_offer(&id).unwrap().available);
}

// no-prop-test: three fixed refusals, each before anything is sent.
#[test]
fn a_note_kept_out_of_ai_is_refused_before_anything_is_sent() {
    let model = Scripted::new(REPLY);
    let (phone, ai) = ai_phone(&model);
    let id = note(&phone.library, "Meeting", &body());
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
        ai.summarize_recording(&id, &AiRun::new()),
        Err(KansoError::invalid(KEPT_OUT))
    );
    assert!(!ai.summary_offer(&id).unwrap().available);
    let locked = note(&phone.library, "Locked", &body());
    store
        .put(
            DocType::Note,
            json!({"_id": locked, "kansoLock": {"salt": "abc"}}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    assert!(ai.summarize_recording(&locked, &AiRun::new()).is_err());
    assert!(!ai.summary_offer(&locked).unwrap().available);
    assert_eq!(model.request_count(), 0);
}

// no-prop-test: one unreadable reply.
#[test]
fn a_reply_that_is_not_understood_changes_nothing() {
    let model = Scripted::new("not json");
    let (phone, ai) = ai_phone(&model);
    let id = note(&phone.library, "Meeting", &body());
    let err = ai.summarize_recording(&id, &AiRun::new()).unwrap_err();
    assert_eq!(err, KansoError::invalid("The reply wasn't understood."));
    let store = Store::open(phone.path()).unwrap();
    assert_eq!(store.note(&id).unwrap().unwrap().body, body());
}

proptest! {
    #![proptest_config(Config {
        cases: 24,
        rng_seed: RngSeed::Fixed(0x0070_6873_756d),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// Whichever action items are ticked, exactly those are written, once
    /// each, and the transcript is left whole.
    #[test]
    fn prop_exactly_the_ticked_items_are_written(ticked in proptest::collection::btree_set(0u32..2, 0..3)) {
        let model = Scripted::new(REPLY);
        let (phone, ai) = ai_phone(&model);
        let id = note(&phone.library, "Meeting", &body());
        let draft = ai.summarize_recording(&id, &AiRun::new()).unwrap();
        ai.apply_recording_summary(&id, draft, ticked.iter().copied().collect()).unwrap();
        let store = Store::open(phone.path()).unwrap();
        let after = store.note(&id).unwrap().unwrap().body;
        prop_assert_eq!(after.matches("- [ ] ").count(), ticked.len());
        prop_assert_eq!(after.contains("Send the onboarding draft"), ticked.contains(&0));
        prop_assert_eq!(after.contains("Call the board ("), ticked.contains(&1));
        prop_assert!(after.contains("<!-- /kanso:transcript -->"));
    }
}
