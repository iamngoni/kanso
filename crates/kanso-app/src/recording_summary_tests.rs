//! Summarise Recording through the real sheet, headless: what is sent, the
//! review, Apply writing only what was ticked, failure leaving the note
//! alone, and notes the AI may not read being refused before anything goes.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use kanso_core::DocType;
use kanso_engine::recording_note::{Segment, render_transcript};
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::ai_feature::Maker;
use crate::ai_feature::testing::{Injected, Recorder, fake_agent, maker_of};
use crate::config::AppConfig;
use crate::recording_summary::{self, Phase, SummarySheet};
use crate::services::Services;
use crate::workspace::{AppState, Workspace};

const FILE: &str = "file:abc123";
const REPLY: &str = r#"{"summary": ["The beta ships on the sixth."],
 "decisions": [{"text": "Ship the beta on the 6th", "quotes": ["ship it on the sixth"]}],
 "actions": [{"text": "Send the onboarding draft", "who": "Priya", "when": "by the 30th",
              "quotes": ["send the onboarding draft"]},
             {"text": "Call the board", "who": null, "when": null,
              "quotes": ["call the board"]},
             {"text": "Invented", "quotes": ["never said"]}]}"#;

struct Fixture {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    window: AnyWindowHandle,
    recorder: Arc<Recorder>,
}

fn setup(cx: &mut TestAppContext, ai: bool, reply: &str) -> Fixture {
    let recorder = Recorder::new(reply);
    let maker = recorder.maker();
    setup_with(cx, ai, recorder, maker)
}

/// The fixture over `maker`, which the features get in place of the
/// settings' backend; `recorder` is what the asserts read.
fn setup_with(cx: &mut TestAppContext, ai: bool, recorder: Arc<Recorder>, maker: Maker) -> Fixture {
    // The worker thread wakes the sheet's task from outside the scheduler.
    cx.executor().allow_parking();
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    store
        .role_book(kanso_core::roles::Role::Inbox, Origin::Ui)
        .unwrap();
    let (app_store, data_dir) = (store.clone(), dir.path().to_path_buf());
    let window = cx.update(|cx| {
        gpui_kit::init(cx);
        let mut config = AppConfig::default();
        config.ai.enabled = ai;
        cx.set_global(AppState {
            store: app_store,
            data_dir,
            config,
        });
        cx.set_global(Services::default());
        cx.set_global(Injected(maker));
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
        .0
    });
    cx.run_until_parked();
    Fixture {
        _dir: dir,
        store,
        window,
        recorder,
    }
}

fn seg(start_secs: u32, text: &str) -> Segment {
    Segment {
        start_secs,
        text: text.into(),
        speaker: None,
    }
}

fn body() -> String {
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

fn note(f: &Fixture, body: &str) -> String {
    f.store
        .put(
            DocType::Note,
            json!({"title": "Meeting — 23 Oct 2026", "body": body, "bookId": "book:inbox"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn open(f: &Fixture, id: &str, cx: &mut TestAppContext) -> Option<Entity<SummarySheet>> {
    let id = id.to_owned();
    cx.update_window(f.window, |_, window, cx| {
        recording_summary::open(id, window, cx)
    })
    .unwrap()
}

fn phase(sheet: &Entity<SummarySheet>, cx: &mut TestAppContext) -> Phase {
    cx.read(|cx| sheet.read(cx).phase().clone())
}

fn settle(sheet: &Entity<SummarySheet>, cx: &mut TestAppContext) -> Phase {
    for _ in 0..1000 {
        let now = phase(sheet, cx);
        if now != Phase::Working {
            return now;
        }
        std::thread::sleep(Duration::from_millis(10));
        cx.run_until_parked();
    }
    panic!("the summary never settled");
}

fn send(sheet: &Entity<SummarySheet>, f: &Fixture, cx: &mut TestAppContext) {
    cx.update_window(f.window, |_, _, cx| {
        sheet.update(cx, |sheet, cx| sheet.send(cx))
    })
    .unwrap();
}

fn tick(
    sheet: &Entity<SummarySheet>,
    f: &Fixture,
    index: usize,
    on: bool,
    cx: &mut TestAppContext,
) {
    cx.update_window(f.window, |_, _, cx| {
        sheet.update(cx, |sheet, cx| sheet.toggle_for_test(index, on, cx))
    })
    .unwrap();
}

fn apply(sheet: &Entity<SummarySheet>, f: &Fixture, cx: &mut TestAppContext) {
    cx.update_window(f.window, |_, window, cx| {
        sheet.update(cx, |sheet, cx| sheet.apply_for_test(window, cx))
    })
    .unwrap();
}

// no-prop-test: drives the sheet; `ground` and `apply_summary` have the properties.
#[gpui_kit::test]
fn nothing_is_sent_until_send_and_only_the_transcript_goes(cx: &mut TestAppContext) {
    let f = setup(cx, true, REPLY);
    let id = note(&f, &body());
    let sheet = open(&f, &id, cx).expect("the sheet opens");
    assert_eq!(phase(&sheet, cx), Phase::Confirm);
    assert!(f.recorder.sent().is_empty(), "nothing is sent on open");

    send(&sheet, &f, cx);
    assert_eq!(settle(&sheet, cx), Phase::Review);
    let sent = f.recorder.sent();
    assert_eq!(sent.len(), 1);
    assert!(sent[0].user.contains("[00:22] The beta is on track"));
    assert!(sent[0].user.contains("Title: Meeting — 23 Oct 2026"));
    assert!(!sent[0].user.contains("inkdrop"), "no file id goes out");
    assert!(!sent[0].user.contains("kanso-audio"), "no link goes out");
    assert_eq!(*f.recorder.tool_asks.lock().unwrap(), 0, "no tools");
    // Nothing is written until Apply.
    assert_eq!(f.store.note(&id).unwrap().unwrap().body, body());
}

// no-prop-test: drives the sheet; the engine has the properties.
#[gpui_kit::test]
fn apply_writes_the_ticked_items_above_the_transcript_once(cx: &mut TestAppContext) {
    let f = setup(cx, true, REPLY);
    let id = note(&f, &body());
    let sheet = open(&f, &id, cx).unwrap();
    send(&sheet, &f, cx);
    assert_eq!(settle(&sheet, cx), Phase::Review);
    // The invented item was never offered; untick the second real one.
    tick(&sheet, &f, 1, false, cx);
    apply(&sheet, &f, cx);
    let after = f.store.note(&id).unwrap().unwrap().body;
    assert!(after.contains("## Summary\n\n- The beta ships on the sixth."));
    assert!(after.contains("- [ ] Send the onboarding draft — Priya, by the 30th"));
    assert!(!after.contains("Call the board"), "unticked is not written");
    assert!(!after.contains("Invented"));
    assert!(
        after.find("kanso:summary").unwrap() < after.find("kanso:transcript").unwrap(),
        "above the transcript"
    );
}

// no-prop-test: drives the sheet.
#[gpui_kit::test]
fn a_failed_reply_changes_nothing_and_says_so(cx: &mut TestAppContext) {
    let f = setup(cx, true, "no json at all");
    let id = note(&f, &body());
    let sheet = open(&f, &id, cx).unwrap();
    send(&sheet, &f, cx);
    assert!(matches!(settle(&sheet, cx), Phase::Failed(_)));
    assert_eq!(f.store.note(&id).unwrap().unwrap().body, body());
}

// no-prop-test: a reply with nothing the transcript backs.
#[gpui_kit::test]
fn a_reply_with_nothing_backed_is_not_offered(cx: &mut TestAppContext) {
    let reply =
        r#"{"summary": [], "decisions": [{"text": "x", "quotes": ["never said"]}], "actions": []}"#;
    let f = setup(cx, true, reply);
    let id = note(&f, &body());
    let sheet = open(&f, &id, cx).unwrap();
    send(&sheet, &f, cx);
    assert!(matches!(settle(&sheet, cx), Phase::Failed(_)));
}

// no-prop-test: drives the command's gate; `summary_source` has the properties.
#[gpui_kit::test]
fn notes_the_ai_may_not_read_open_no_sheet_and_send_nothing(cx: &mut TestAppContext) {
    let f = setup(cx, true, REPLY);
    let locked = note(&f, &body());
    f.store
        .put(
            DocType::Note,
            json!({"_id": locked, "kansoLock": {"salt": "abc"}}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    assert!(open(&f, &locked, cx).is_none());
    let plain = note(&f, "Just words, no recording.");
    assert!(open(&f, &plain, cx).is_none());
    assert!(f.recorder.sent().is_empty());
}

// no-prop-test: the command's gate with AI off.
#[gpui_kit::test]
fn with_ai_off_the_command_does_nothing(cx: &mut TestAppContext) {
    let f = setup(cx, false, REPLY);
    let id = note(&f, &body());
    assert!(open(&f, &id, cx).is_none());
    assert!(f.recorder.sent().is_empty());
}

// no-prop-test: drives the sheet; `summary_source` has the properties.
#[gpui_kit::test]
fn a_note_locked_while_the_sheet_is_open_sends_nothing(cx: &mut TestAppContext) {
    let f = setup(cx, true, REPLY);
    let id = note(&f, &body());
    let sheet = open(&f, &id, cx).expect("the sheet opens");
    // Locked (or synced in locked) before Send is pressed.
    f.store
        .put(
            DocType::Note,
            json!({"_id": id, "kansoLock": {"salt": "abc"}}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    send(&sheet, &f, cx);
    let Phase::Failed(reason) = phase(&sheet, cx) else {
        panic!("sent anyway");
    };
    assert!(reason.contains("kept out of AI"), "{reason}");
    assert!(f.recorder.sent().is_empty(), "nothing is sent");
}

// no-prop-test: drives the sheet; the engine has the properties.
#[gpui_kit::test]
fn a_note_tagged_ai_exclude_after_review_is_not_written(cx: &mut TestAppContext) {
    let f = setup(cx, true, REPLY);
    let id = note(&f, &body());
    let sheet = open(&f, &id, cx).unwrap();
    send(&sheet, &f, cx);
    assert_eq!(settle(&sheet, cx), Phase::Review);
    let tag = f
        .store
        .put(
            DocType::Tag,
            json!({"name": "AI-Exclude"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    f.store
        .put(
            DocType::Note,
            json!({"_id": id, "tags": [tag]}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    apply(&sheet, &f, cx);
    assert_eq!(f.store.note(&id).unwrap().unwrap().body, body());
}

// no-prop-test: drives the sheet over a note with two recordings.
#[gpui_kit::test]
fn only_the_first_recording_is_sent_and_the_sheet_says_so(cx: &mut TestAppContext) {
    let f = setup(cx, true, REPLY);
    let second =
        render_transcript("file:second9", &[seg(0, "A private aside about salaries.")]).unwrap();
    let id = note(&f, &format!("{}\n\n{second}", body()));
    let sheet = open(&f, &id, cx).unwrap();
    let words = cx.read(|cx| sheet.read(cx).confirm_text().to_owned());
    assert!(words.contains("This note has 2 recordings"), "{words}");
    send(&sheet, &f, cx);
    assert_eq!(settle(&sheet, cx), Phase::Review);
    let sent = f.recorder.sent();
    assert!(!sent[0].user.contains("salaries"), "the second stays");
}

// no-prop-test: a backend that cannot be made, as a missing key is.
#[gpui_kit::test]
fn a_missing_key_fails_with_a_line_and_changes_nothing(cx: &mut TestAppContext) {
    let maker: Maker = Arc::new(|| Err(kanso_ai::AiError::MissingKey.to_string()));
    let f = setup_with(cx, true, Recorder::new(REPLY), maker);
    let id = note(&f, &body());
    let sheet = open(&f, &id, cx).unwrap();
    send(&sheet, &f, cx);
    let Phase::Failed(reason) = settle(&sheet, cx) else {
        panic!("no failure");
    };
    assert!(reason.contains("Settings › AI"), "{reason}");
    assert_eq!(f.store.note(&id).unwrap().unwrap().body, body());
}

// no-prop-test: the in-process fake agent answering with a fixed reply.
#[gpui_kit::test]
fn an_agent_answers_through_the_same_review(cx: &mut TestAppContext) {
    let agent = fake_agent(
        kanso_acp::fake::Script::Answer(REPLY.to_owned()),
        "recording-summary",
    );
    let f = setup_with(cx, true, Recorder::new(REPLY), maker_of(agent));
    let id = note(&f, &body());
    let sheet = open(&f, &id, cx).unwrap();
    send(&sheet, &f, cx);
    assert_eq!(settle(&sheet, cx), Phase::Review);
    apply(&sheet, &f, cx);
    let after = f.store.note(&id).unwrap().unwrap().body;
    assert!(after.contains("- [ ] Send the onboarding draft — Priya, by the 30th"));
    assert!(!after.contains("Invented"));
}

// no-prop-test: the in-process fake agent answering in prose, not the JSON
// the sheet reads.
#[gpui_kit::test]
fn an_agent_reply_in_prose_fails_and_changes_nothing(cx: &mut TestAppContext) {
    let agent = fake_agent(
        kanso_acp::fake::Script::Answer("Sorry, I can't summarise that.".to_owned()),
        "recording-summary-prose",
    );
    let f = setup_with(cx, true, Recorder::new(REPLY), maker_of(agent));
    let id = note(&f, &body());
    let sheet = open(&f, &id, cx).unwrap();
    send(&sheet, &f, cx);
    assert!(matches!(settle(&sheet, cx), Phase::Failed(_)));
    assert_eq!(f.store.note(&id).unwrap().unwrap().body, body());
}
