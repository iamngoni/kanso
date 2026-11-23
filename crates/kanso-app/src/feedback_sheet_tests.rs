//! The Send feedback sheet against a fake server: the states the person
//! sees, and what reaches the outbox and the wire.

use std::sync::{Arc, Mutex};

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext, VisualTestContext};
use kanso_engine::feedback::{self, Kind};
use kanso_store::{OutboxState, Store};
use kanso_sync::MemorySecrets;
use kanso_sync::feedback::{Request, Response, Transport};

use crate::config::AppConfig;
use crate::feedback_app::{self, TransportOverride};
use crate::feedback_sheet::{CreditKind, FeedbackSheet, Lookup, SEARCH_DEBOUNCE, SendFeedback};
use crate::services::Services;
use crate::workspace::{AppState, Workspace};

/// A server that answers every request the same way (the user search from
/// `users`) and remembers them.
struct Fake {
    answer: Result<Response, String>,
    users: Mutex<Result<Response, String>>,
    seen: Mutex<Vec<Request>>,
}

impl Transport for Fake {
    fn send(&self, request: &Request) -> Result<Response, String> {
        self.seen.lock().unwrap().push(request.clone());
        if request.path.starts_with("/v1/feedback/users") {
            return self.users.lock().unwrap().clone();
        }
        self.answer.clone()
    }
}

fn reply(status: u16, body: &str) -> Result<Response, String> {
    Ok(Response {
        status,
        body: body.as_bytes().to_vec(),
    })
}

struct Rig {
    window: AnyWindowHandle,
    store: Arc<Store>,
    fake: Arc<Fake>,
    _dir: tempfile::TempDir,
}

fn rig(cx: &mut TestAppContext, answer: Result<Response, String>) -> Rig {
    cx.executor().allow_parking();
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let fake = Arc::new(Fake {
        answer,
        users: Mutex::new(reply(200, r#"{"users":[]}"#)),
        seen: Mutex::new(Vec::new()),
    });
    let (state, transport) = (store.clone(), fake.clone());
    let data_dir = dir.path().to_path_buf();
    let (window, _view): (_, Entity<Workspace>) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store: state,
            data_dir,
            config: AppConfig::default(),
        });
        let mut services = Services::default();
        services.secrets = Some(Arc::new(MemorySecrets::new()));
        cx.set_global(services);
        cx.set_global(TransportOverride(transport));
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    Rig {
        window,
        store,
        fake,
        _dir: dir,
    }
}

fn open_sheet(cx: &mut TestAppContext, rig: &Rig) -> Entity<FeedbackSheet> {
    let sheet = rig
        .window
        .update(cx, |_, window, cx| crate::feedback_sheet::open(window, cx))
        .unwrap();
    cx.run_until_parked();
    sheet
}

fn type_text(cx: &mut TestAppContext, rig: &Rig, sheet: &Entity<FeedbackSheet>, text: &str) {
    let text = text.to_owned();
    let sheet = sheet.clone();
    rig.window
        .update(cx, move |_, window, cx| {
            let area = sheet.read(cx).text.clone();
            area.update(cx, |area, cx| area.set_value(text, window, cx));
        })
        .unwrap();
    cx.run_until_parked();
}

fn press_send(cx: &mut TestAppContext, rig: &Rig, sheet: &Entity<FeedbackSheet>) {
    let target = sheet.clone();
    rig.window
        .update(cx, move |_, window, cx| {
            target.update(cx, |s, cx| s.send(window, cx))
        })
        .unwrap();
    // The send runs on a worker thread; wait for its answer to land.
    for _ in 0..300 {
        cx.run_until_parked();
        if !cx.read(|cx| sheet.read(cx).busy) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[gpui_kit::test]
fn the_action_opens_the_sheet_and_the_kind_changes_the_hint(cx: &mut TestAppContext) {
    let rig = rig(cx, reply(201, r#"{"id":"x","status":"received"}"#));
    rig.window
        .update(cx, |_, window, cx| {
            window.dispatch_action(Box::new(SendFeedback), cx)
        })
        .unwrap();
    cx.run_until_parked();
    let sheet = open_sheet(cx, &rig);
    let (kind, shown) = cx.read(|cx| {
        let s = sheet.read(cx);
        (s.kind, s.hint)
    });
    assert_eq!(kind, Kind::Bug);
    assert_eq!(shown, "What happened, and what did you expect?");
    let target = sheet.clone();
    rig.window
        .update(cx, move |_, window, cx| {
            target.update(cx, |s, cx| s.set_kind(Kind::Idea, window, cx))
        })
        .unwrap();
    let shown = cx.read(|cx| sheet.read(cx).hint);
    assert_eq!(shown, "What would you like Kanso to do?");
}

#[gpui_kit::test]
fn a_short_report_is_refused_and_nothing_is_saved(cx: &mut TestAppContext) {
    let rig = rig(cx, reply(201, r#"{"id":"x","status":"received"}"#));
    let sheet = open_sheet(cx, &rig);
    type_text(cx, &rig, &sheet, "too short");
    press_send(cx, &rig, &sheet);
    let error = cx.read(|cx| sheet.read(cx).error.clone());
    assert_eq!(
        error.as_deref(),
        Some("Say a little more (at least 10 characters).")
    );
    assert!(rig.store.feedback_all().unwrap().is_empty());
    assert!(rig.fake.seen.lock().unwrap().is_empty());
}

#[gpui_kit::test]
fn sending_saves_then_posts_once(cx: &mut TestAppContext) {
    let rig = rig(cx, reply(201, r#"{"id":"x","status":"received"}"#));
    let sheet = open_sheet(cx, &rig);
    type_text(cx, &rig, &sheet, "Export to PDF cuts off wide tables.");
    press_send(cx, &rig, &sheet);
    let seen = rig.fake.seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 1);
    assert_eq!(
        (seen[0].method, seen[0].path.as_str()),
        ("POST", "/v1/feedback")
    );
    let body: serde_json::Value = serde_json::from_slice(&seen[0].body).unwrap();
    assert_eq!(body["text"], "Export to PDF cuts off wide tables.");
    assert_eq!(body["kind"], "bug");
    assert_eq!(body["credit"], serde_json::Value::Null);
    // Nothing of a note is in it.
    assert_eq!(body.as_object().unwrap().len(), 7);
    // Anonymous: the install's key, no bearer token.
    let names: Vec<&str> = seen[0].headers.iter().map(|(n, _)| *n).collect();
    assert!(names.contains(&"x-kanso-feedback-key") && !names.contains(&"authorization"));
    let rows = rig.store.feedback_all().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].state, OutboxState::Sent);
}

#[gpui_kit::test]
fn an_agent_report_prefills_the_sheet_and_waits_for_send(cx: &mut TestAppContext) {
    let rig = rig(cx, reply(201, r#"{"id":"x","status":"received"}"#));
    let report = kanso_server::AgentReport::new(
        kanso_server::ReportKind::Idea,
        "A tool to list notes by date would help.",
        Some("list-notes"),
        None,
    )
    .unwrap();
    let (kind, text) = (crate::feedback_sheet::kind_of(report.kind), report.text());
    let sheet = rig
        .window
        .update(cx, move |_, window, cx| {
            crate::feedback_sheet::open_prefilled(kind, text, window, cx)
        })
        .unwrap();
    cx.run_until_parked();
    let (kind, shown) = cx.read(|cx| {
        let s = sheet.read(cx);
        (s.kind, s.text.read(cx).value().to_string())
    });
    assert_eq!(kind, Kind::Idea);
    assert_eq!(shown, report.text());
    assert!(shown.starts_with("Reported by an AI agent:\n"));
    assert!(crate::feedback_sheet::can_send(shown.chars().count(), None));
    // Nothing leaves until the person presses Send.
    assert!(rig.fake.seen.lock().unwrap().is_empty());
    assert!(rig.store.feedback_all().unwrap().is_empty());
    press_send(cx, &rig, &sheet);
    let seen = rig.fake.seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 1);
    let body: serde_json::Value = serde_json::from_slice(&seen[0].body).unwrap();
    assert_eq!(body["kind"], "idea");
    assert_eq!(body["text"], report.text());
}

#[gpui_kit::test]
fn offline_keeps_the_report_waiting(cx: &mut TestAppContext) {
    let rig = rig(cx, Err("offline".into()));
    let sheet = open_sheet(cx, &rig);
    type_text(cx, &rig, &sheet, "Export to PDF cuts off wide tables.");
    press_send(cx, &rig, &sheet);
    let rows = rig.store.feedback_all().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].state, OutboxState::Queued);
}

#[gpui_kit::test]
fn a_400_keeps_the_sheet_open_with_the_field(cx: &mut TestAppContext) {
    let rig = rig(
        cx,
        reply(400, r#"{"error":"invalid","field":"app.version"}"#),
    );
    let sheet = open_sheet(cx, &rig);
    type_text(cx, &rig, &sheet, "Export to PDF cuts off wide tables.");
    press_send(cx, &rig, &sheet);
    let error = cx.read(|cx| sheet.read(cx).error.clone());
    assert_eq!(
        error.as_deref(),
        Some("Kanso couldn't send this report (app.version). Edit it and try again.")
    );
    assert!(rig.store.feedback_all().unwrap().is_empty());
}

/// A chosen picture is drawn in the sheet before anything is sent, and its
/// EXIF (a photo's GPS position) is gone from what is declared and kept.
#[gpui_kit::test]
fn a_chosen_picture_is_shown_and_loses_its_exif(cx: &mut TestAppContext) {
    let rig = rig(cx, reply(201, r#"{"id":"x","status":"received"}"#));
    let sheet = open_sheet(cx, &rig);
    let mut jpeg = vec![0xff, 0xd8];
    let exif = b"Exif\0\0GPS-51.5,-0.12";
    jpeg.extend([0xff, 0xe1]);
    jpeg.extend(((exif.len() + 2) as u16).to_be_bytes());
    jpeg.extend(exif);
    jpeg.extend([0xff, 0xda, 0x00, 0x04, 9, 9, 0x12, 0x34, 0xff, 0xd9]);
    let target = sheet.clone();
    rig.window
        .update(cx, move |_, _, cx| {
            target.update(cx, |s, cx| s.set_picture(jpeg, "photo.jpg".into(), cx))
        })
        .unwrap();
    rig.window
        .update(cx, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    cx.run_until_parked();
    let mut vcx = VisualTestContext::from_window(rig.window, cx);
    assert!(vcx.debug_bounds("feedback-thumbnail").is_some());
    type_text(cx, &rig, &sheet, "The export cuts off wide tables.");
    press_send(cx, &rig, &sheet);
    // Declared as the 12 bytes left once the EXIF segment is gone.
    let row = &rig.store.feedback_all().unwrap()[0];
    assert!(row.payload.contains("\"image/jpeg\""));
    assert!(row.payload.contains("\"bytes\": 12"), "{}", row.payload);
    let posted = &rig.fake.seen.lock().unwrap()[0];
    assert!(!String::from_utf8_lossy(&posted.body).contains("GPS"));
}

// ------------------------------------------------------------- the credit

const USERS: &str = r#"{"users":[
{"login":"octocat","avatar_url":"https://avatars.githubusercontent.com/u/583231?v=4"},
{"login":"octokit","avatar_url":"https://avatars.githubusercontent.com/u/3430433?v=4"}]}"#;

fn set_credit(cx: &mut TestAppContext, rig: &Rig, sheet: &Entity<FeedbackSheet>, kind: CreditKind) {
    let target = sheet.clone();
    rig.window
        .update(cx, move |_, _, cx| {
            target.update(cx, |s, cx| {
                s.credit = true;
                s.set_credit_kind(kind, cx);
            })
        })
        .unwrap();
    cx.run_until_parked();
}

fn type_search(cx: &mut TestAppContext, rig: &Rig, sheet: &Entity<FeedbackSheet>, text: &str) {
    let (text, sheet) = (text.to_owned(), sheet.clone());
    rig.window
        .update(cx, move |_, window, cx| {
            let input = sheet.read(cx).github.clone();
            input.update(cx, |input, cx| input.set_value(text, window, cx));
            // A typed change emits Change, which calls this; a programmatic one may not.
            sheet.update(cx, |s, cx| s.query_changed(cx));
        })
        .unwrap();
    cx.run_until_parked();
}

/// Lets the debounce pass and the search come back.
fn settle_search(cx: &mut TestAppContext, sheet: &Entity<FeedbackSheet>) {
    cx.executor().advance_clock(SEARCH_DEBOUNCE);
    for _ in 0..300 {
        cx.run_until_parked();
        if !matches!(
            cx.read(|cx| sheet.read(cx).lookup.clone()),
            Lookup::Searching
        ) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn searches(rig: &Rig) -> Vec<String> {
    rig.fake
        .seen
        .lock()
        .unwrap()
        .iter()
        .filter(|r| r.path.starts_with("/v1/feedback/users"))
        .map(|r| r.path.clone())
        .collect()
}

fn pick(cx: &mut TestAppContext, rig: &Rig, sheet: &Entity<FeedbackSheet>, ix: usize) {
    let target = sheet.clone();
    rig.window
        .update(cx, move |_, window, cx| {
            let user = match &target.read(cx).lookup {
                Lookup::Found(users) => users[ix].clone(),
                other => panic!("nothing to pick: {other:?}"),
            };
            target.update(cx, |s, cx| s.pick(user, window, cx));
        })
        .unwrap();
    cx.run_until_parked();
}

fn draw(cx: &mut TestAppContext, rig: &Rig) -> VisualTestContext {
    rig.window
        .update(cx, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    cx.run_until_parked();
    VisualTestContext::from_window(rig.window, cx)
}

fn posted_credit(rig: &Rig) -> serde_json::Value {
    let seen = rig.fake.seen.lock().unwrap();
    let post = seen
        .iter()
        .find(|r| r.path == "/v1/feedback")
        .expect("a report was posted");
    serde_json::from_slice::<serde_json::Value>(&post.body).unwrap()["credit"].clone()
}

#[gpui_kit::test]
fn a_plain_name_is_sent_as_the_tagged_name(cx: &mut TestAppContext) {
    let rig = rig(cx, reply(201, r#"{"id":"x","status":"received"}"#));
    let sheet = open_sheet(cx, &rig);
    type_text(cx, &rig, &sheet, "Export to PDF cuts off wide tables.");
    let target = sheet.clone();
    rig.window
        .update(cx, move |_, window, cx| {
            target.update(cx, |s, cx| {
                s.credit = true;
                s.name
                    .update(cx, |name, cx| name.set_value("Tendai M", window, cx));
            })
        })
        .unwrap();
    cx.run_until_parked();
    assert_eq!(cx.read(|cx| sheet.read(cx).credit_kind), CreditKind::Name);
    press_send(cx, &rig, &sheet);
    assert_eq!(
        posted_credit(&rig),
        serde_json::json!({"kind": "name", "name": "Tendai M"})
    );
    assert!(searches(&rig).is_empty());
}

#[gpui_kit::test]
fn typing_a_github_name_searches_once_after_the_pause(cx: &mut TestAppContext) {
    let rig = rig(cx, reply(201, r#"{"id":"x","status":"received"}"#));
    *rig.fake.users.lock().unwrap() = reply(200, USERS);
    let sheet = open_sheet(cx, &rig);
    set_credit(cx, &rig, &sheet, CreditKind::Github);
    // Three keystrokes inside the pause make one search, for the last text.
    for text in ["oc", "oct", "octo"] {
        type_search(cx, &rig, &sheet, text);
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(100));
        cx.run_until_parked();
    }
    assert!(searches(&rig).is_empty());
    assert_eq!(
        cx.read(|cx| sheet.read(cx).lookup.clone()),
        Lookup::Searching
    );
    settle_search(cx, &sheet);
    assert_eq!(searches(&rig), vec!["/v1/feedback/users?q=octo".to_owned()]);
    let found = cx.read(|cx| sheet.read(cx).lookup.clone());
    let Lookup::Found(users) = found else {
        panic!("{found:?}")
    };
    assert_eq!(
        users.iter().map(|u| u.login.as_str()).collect::<Vec<_>>(),
        ["octocat", "octokit"]
    );
    let mut vcx = draw(cx, &rig);
    assert!(vcx.debug_bounds("feedback-github-results").is_some());
    assert!(vcx.debug_bounds("feedback-github-user-0").is_some());
    assert!(vcx.debug_bounds("feedback-github-user-1").is_some());
}

#[gpui_kit::test]
fn too_little_text_asks_nothing(cx: &mut TestAppContext) {
    let rig = rig(cx, reply(201, r#"{"id":"x","status":"received"}"#));
    let sheet = open_sheet(cx, &rig);
    set_credit(cx, &rig, &sheet, CreditKind::Github);
    for text in ["o", "", "a b", "octo in:login"] {
        type_search(cx, &rig, &sheet, text);
        settle_search(cx, &sheet);
        assert_eq!(cx.read(|cx| sheet.read(cx).lookup.clone()), Lookup::Idle);
    }
    assert!(searches(&rig).is_empty());
}

#[gpui_kit::test]
fn picking_a_user_credits_their_login_and_enables_send(cx: &mut TestAppContext) {
    let rig = rig(cx, reply(201, r#"{"id":"x","status":"received"}"#));
    *rig.fake.users.lock().unwrap() = reply(200, USERS);
    let sheet = open_sheet(cx, &rig);
    type_text(cx, &rig, &sheet, "Export to PDF cuts off wide tables.");
    set_credit(cx, &rig, &sheet, CreditKind::Github);
    // Nobody picked: not sendable, and sending says what to do.
    let credit = cx.read(|cx| sheet.read(cx).credit(cx));
    assert!(!crate::feedback_sheet::can_send(36, credit.as_ref()));
    press_send(cx, &rig, &sheet);
    assert_eq!(
        cx.read(|cx| sheet.read(cx).error.clone()).as_deref(),
        Some(feedback_app::GITHUB_PICK_FIRST)
    );
    assert!(rig.store.feedback_all().unwrap().is_empty());

    type_search(cx, &rig, &sheet, "octo");
    settle_search(cx, &sheet);
    pick(cx, &rig, &sheet, 0);
    assert_eq!(
        cx.read(|cx| sheet.read(cx).picked.clone()).map(|u| u.login),
        Some("octocat".into())
    );
    let mut vcx = draw(cx, &rig);
    assert!(vcx.debug_bounds("feedback-github-picked").is_some());
    assert!(vcx.debug_bounds("feedback-github-results").is_none());
    let credit = cx.read(|cx| sheet.read(cx).credit(cx));
    assert!(crate::feedback_sheet::can_send(36, credit.as_ref()));

    press_send(cx, &rig, &sheet);
    assert_eq!(
        posted_credit(&rig),
        serde_json::json!({"kind": "github", "login": "octocat"})
    );
    // "Show what's sent" would have said the same: it is built from the same draft.
    let row = &rig.store.feedback_all().unwrap()[0];
    assert!(
        row.payload.contains("\"login\": \"octocat\""),
        "{}",
        row.payload
    );
}

#[gpui_kit::test]
fn changing_the_pick_and_switching_to_a_name_keep_each_choice_apart(cx: &mut TestAppContext) {
    let rig = rig(cx, reply(201, r#"{"id":"x","status":"received"}"#));
    *rig.fake.users.lock().unwrap() = reply(200, USERS);
    let sheet = open_sheet(cx, &rig);
    set_credit(cx, &rig, &sheet, CreditKind::Github);
    type_search(cx, &rig, &sheet, "octo");
    settle_search(cx, &sheet);
    pick(cx, &rig, &sheet, 1);
    // Back to a name: the name is the credit, the pick is remembered.
    set_credit(cx, &rig, &sheet, CreditKind::Name);
    assert!(matches!(
        cx.read(|cx| sheet.read(cx).credit(cx)),
        Some(feedback::Credit::Name(_))
    ));
    set_credit(cx, &rig, &sheet, CreditKind::Github);
    assert_eq!(
        cx.read(|cx| sheet.read(cx).credit(cx)),
        Some(feedback::Credit::Github("octokit".into()))
    );
    // Change takes the pick back; the credit is then unsendable until one is picked.
    let target = sheet.clone();
    rig.window
        .update(cx, move |_, _, cx| target.update(cx, |s, cx| s.unpick(cx)))
        .unwrap();
    assert_eq!(
        cx.read(|cx| sheet.read(cx).credit(cx)),
        Some(feedback::Credit::Github(String::new()))
    );
}

#[gpui_kit::test]
fn a_failed_search_says_so_and_a_name_still_works(cx: &mut TestAppContext) {
    let rig = rig(cx, reply(201, r#"{"id":"x","status":"received"}"#));
    *rig.fake.users.lock().unwrap() = reply(503, r#"{"error":"search_unavailable"}"#);
    let sheet = open_sheet(cx, &rig);
    set_credit(cx, &rig, &sheet, CreditKind::Github);
    type_search(cx, &rig, &sheet, "octo");
    settle_search(cx, &sheet);
    assert_eq!(
        cx.read(|cx| sheet.read(cx).lookup.clone()),
        Lookup::Unavailable
    );
    let mut vcx = draw(cx, &rig);
    assert!(vcx.debug_bounds("feedback-github-unavailable").is_some());
    // Offline is the same.
    *rig.fake.users.lock().unwrap() = Err("offline".into());
    type_search(cx, &rig, &sheet, "octok");
    settle_search(cx, &sheet);
    assert_eq!(
        cx.read(|cx| sheet.read(cx).lookup.clone()),
        Lookup::Unavailable
    );
    set_credit(cx, &rig, &sheet, CreditKind::Name);
    assert!(matches!(
        cx.read(|cx| sheet.read(cx).credit(cx)),
        Some(feedback::Credit::Name(_))
    ));
}

#[gpui_kit::test]
fn no_results_is_its_own_state(cx: &mut TestAppContext) {
    let rig = rig(cx, reply(201, r#"{"id":"x","status":"received"}"#));
    let sheet = open_sheet(cx, &rig);
    set_credit(cx, &rig, &sheet, CreditKind::Github);
    type_search(cx, &rig, &sheet, "zzzzzz");
    settle_search(cx, &sheet);
    assert_eq!(
        cx.read(|cx| sheet.read(cx).lookup.clone()),
        Lookup::Found(vec![])
    );
}
