//! The Send feedback sheet against a fake server: the states the person
//! sees, and what reaches the outbox and the wire.

use std::sync::{Arc, Mutex};

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use kanso_engine::feedback::Kind;
use kanso_store::{OutboxState, Store};
use kanso_sync::MemorySecrets;
use kanso_sync::feedback::{Request, Response, Transport};

use crate::config::AppConfig;
use crate::feedback_app::TransportOverride;
use crate::feedback_sheet::{FeedbackSheet, SendFeedback};
use crate::services::Services;
use crate::workspace::{AppState, Workspace};

/// A server that answers every request the same way and remembers them.
struct Fake {
    answer: Result<Response, String>,
    seen: Mutex<Vec<Request>>,
}

impl Transport for Fake {
    fn send(&self, request: &Request) -> Result<Response, String> {
        self.seen.lock().unwrap().push(request.clone());
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
