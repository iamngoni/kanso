//! A note under embargo on the desktop: the banner under its title, the
//! question before publishing it, making a share link to it and exporting
//! it, and the notice when the embargo lifts.

use std::sync::Arc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::{AnyWindowHandle, App, AppContext as _, Entity, TestAppContext};
use kanso_collab::Role;
use kanso_collab::memory::MemoryCollab;
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use kanso_sync::share::{KansoPublish, MemoryShares};
use kanso_sync::{DeviceIdentity, MemorySecrets, MemoryServer, Password};
use serde_json::json;

use crate::clock::Clock;
use crate::collab_tests::{Device, device};
use crate::config::AppConfig;
use crate::publish::{PublishView, open_publish};
use crate::services::Services;
use crate::workspace::{AppState, Workspace};

/// Friday 13 November 2026, 15:00 UTC.
const NOW: i64 = 1_794_582_000_000;
/// 09:00 in New York on Wednesday 18 November 2026: 14:00 UTC.
const LIFTS: i64 = 1_795_010_400_000;
const BODY: &str =
    "---\nstage: Editing\nembargo: 2026-11-18T09:00 America/New_York\n---\nThe Pixel 11 is fast.\n";

struct Fixture {
    _dir: Option<tempfile::TempDir>,
    store: Arc<Store>,
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
    shares: Arc<MemoryShares>,
    note: String,
}

/// A window with the note `body` open, at [`NOW`] in UTC, signed in.
fn setup(cx: &mut TestAppContext, body: &str) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    open_in(cx, store, Some(dir), body, None)
}

fn open_in(
    cx: &mut TestAppContext,
    store: Arc<Store>,
    dir: Option<tempfile::TempDir>,
    body: &str,
    engine: Option<Arc<kanso_collab::Engine>>,
) -> Fixture {
    cx.executor().allow_parking();
    let book = store
        .put(
            DocType::Book,
            json!({"name": "Studio"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let note = store
        .put(
            DocType::Note,
            json!({"title": "Pixel 11 review", "body": body, "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let shares = Arc::new(MemoryShares::new());
    let mut services = Services::default();
    services.shares = Some(shares.clone());
    let server = MemoryServer::new();
    let username = "alice";
    let session = kanso_sync::sign_up(
        &server,
        &MemorySecrets::new(),
        kanso_sync::DEFAULT_SERVER,
        username,
        &Password::new("correct horse battery", username).unwrap(),
        &DeviceIdentity::from_parts(&format!("l_{:032x}", 1), "Mac", [1; 32]),
        &|_| {},
    )
    .unwrap()
    .session;
    services.set_signed_in(Arc::new(session));
    let data_dir = std::env::temp_dir();
    let app_store = store.clone();
    let (window, workspace) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(Clock::fixed(NOW, 0));
        cx.set_global(AppState {
            store: app_store,
            data_dir,
            config: AppConfig::default(),
        });
        cx.set_global(services);
        if let Some(engine) = engine {
            crate::collab::use_engine(engine, cx);
        }
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    let library = cx.read(|cx| workspace.read(cx).library.clone());
    let id = note.clone();
    library.update(cx, |library, cx| library.select(Some(id), cx));
    cx.run_until_parked();
    Fixture {
        _dir: dir,
        store,
        window,
        workspace,
        shares,
        note,
    }
}

impl Fixture {
    fn editor(&self, cx: &mut TestAppContext) -> Entity<crate::editor::NoteEditor> {
        cx.read(|cx| self.workspace.read(cx).editor.clone())
    }

    fn publish(&self, cx: &mut TestAppContext) -> Entity<PublishView> {
        let editor = self.editor(cx);
        self.window
            .update(cx, |_, window, cx| open_publish(editor, window, cx))
            .unwrap()
            .expect("a note is open")
    }

    fn on_publish(
        &self,
        cx: &mut TestAppContext,
        view: &Entity<PublishView>,
        f: impl FnOnce(&mut PublishView, &mut gpui_kit::Window, &mut gpui_kit::Context<PublishView>),
    ) {
        self.window
            .update(cx, |_, window, cx| {
                view.update(cx, |v, cx| f(v, window, cx))
            })
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        loop {
            cx.run_until_parked();
            if !cx.read(|cx| view.read(cx).is_busy()) {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "the task finished");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    fn kept(&self) -> Option<KansoPublish> {
        let note = self.store.note(&self.note).unwrap().unwrap();
        KansoPublish::from_extra(&note.extra)
    }

    fn dialog_open(&self, cx: &mut TestAppContext) -> bool {
        self.window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap()
    }

    fn workspace_do(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut Workspace, &mut gpui_kit::Window, &mut gpui_kit::Context<Workspace>),
    ) {
        let workspace = self.workspace.clone();
        self.window
            .update(cx, |_, window, cx| {
                workspace.update(cx, |w, cx| f(w, window, cx))
            })
            .unwrap();
        cx.run_until_parked();
    }

    fn go_ahead(&self, cx: &mut TestAppContext) {
        self.window
            .update(cx, |_, window, cx| {
                crate::embargo_gate::go_ahead_for_test(window, cx)
            })
            .unwrap();
        cx.run_until_parked();
    }
}

fn set_now(cx: &mut TestAppContext, now: i64) {
    cx.update(|cx: &mut App| cx.set_global(Clock::fixed(now, 0)));
}

// no-prop-test: drawing; `kanso_core::embargo` has the properties.
#[gpui_kit::test]
fn the_note_shows_a_banner_until_the_embargo_lifts(cx: &mut TestAppContext) {
    let f = setup(cx, BODY);
    let editor = f.editor(cx);
    let banner = cx.read(|cx| editor.read(cx).embargo(cx).map(|e| e.banner()));
    assert_eq!(
        banner.as_deref(),
        Some("Embargoed until Wed 18 Nov, 09:00 New York.")
    );
    set_now(cx, LIFTS);
    assert_eq!(cx.read(|cx| editor.read(cx).embargo(cx)), None);
}

// no-prop-test: one flow in a window.
#[gpui_kit::test]
fn publishing_a_note_under_embargo_asks_first(cx: &mut TestAppContext) {
    let f = setup(cx, BODY);
    let view = f.publish(cx);
    f.on_publish(cx, &view, |v, window, cx| v.run_publish(window, cx));
    let ask = cx
        .read(|cx| view.read(cx).embargo_ask().cloned())
        .expect("it asks");
    assert_eq!(ask.title, "This note is under embargo");
    assert_eq!(
        ask.body,
        "Embargo lifts Wed 18 Nov, 09:00 New York (in 5 days). Publish anyway?"
    );
    assert_eq!(ask.confirm, "Publish anyway");
    assert!(f.shares.is_empty(), "nothing is published while it asks");
    assert_eq!(f.kept(), None);

    // Cancel publishes nothing; asking again asks again.
    f.on_publish(cx, &view, |v, _, cx| v.keep_embargo(cx));
    assert!(cx.read(|cx| view.read(cx).embargo_ask().is_none()));
    f.on_publish(cx, &view, |v, window, cx| v.run_publish(window, cx));
    assert!(cx.read(|cx| view.read(cx).embargo_ask().is_some()));
    assert!(f.shares.is_empty());

    // Publish anyway publishes.
    f.on_publish(cx, &view, |v, window, cx| v.publish_anyway(window, cx));
    assert!(cx.read(|cx| view.read(cx).embargo_ask().is_none()));
    assert!(f.kept().is_some(), "published past the embargo");
}

// no-prop-test: one flow in a window.
#[gpui_kit::test]
fn a_lifted_embargo_publishes_without_asking(cx: &mut TestAppContext) {
    let f = setup(cx, BODY);
    set_now(cx, LIFTS);
    let view = f.publish(cx);
    f.on_publish(cx, &view, |v, window, cx| v.run_publish(window, cx));
    assert!(cx.read(|cx| view.read(cx).embargo_ask().is_none()));
    assert!(f.kept().is_some());
}

// no-prop-test: one flow in a window.
#[gpui_kit::test]
fn exporting_a_note_under_embargo_asks_first(cx: &mut TestAppContext) {
    let f = setup(cx, BODY);
    f.workspace_do(cx, |w, window, cx| w.export_page(false, window, cx));
    assert!(f.dialog_open(cx), "the embargo question");
    assert!(!cx.did_prompt_for_new_path(), "nothing is saved yet");
    f.go_ahead(cx);
    assert!(
        cx.did_prompt_for_new_path(),
        "Export anyway asks where to save"
    );
}

// no-prop-test: one flow in a window.
#[gpui_kit::test]
fn exporting_a_pdf_and_chosen_notes_under_embargo_ask_too(cx: &mut TestAppContext) {
    let f = setup(cx, BODY);
    f.workspace_do(cx, |w, window, cx| w.export_page(true, window, cx));
    assert!(f.dialog_open(cx));
    f.window
        .update(cx, |_, window, cx| window.close_dialog(cx))
        .unwrap();
    cx.run_until_parked();
    let id = f.note.clone();
    f.workspace_do(cx, |w, window, cx| w.export(Some(vec![id]), window, cx));
    assert!(f.dialog_open(cx));
    assert!(!cx.did_prompt_for_paths(), "no folder is asked for yet");
    f.go_ahead(cx);
    assert!(cx.did_prompt_for_paths(), "Export anyway asks for a folder");
}

// no-prop-test: one flow in a window.
#[gpui_kit::test]
fn a_note_without_an_embargo_exports_at_once(cx: &mut TestAppContext) {
    let f = setup(cx, "The Pixel 11 is fast.\n");
    f.workspace_do(cx, |w, window, cx| w.export_page(false, window, cx));
    assert!(!f.dialog_open(cx));
    assert!(cx.did_prompt_for_new_path());
}

// no-prop-test: one flow in a window, against the in-memory server.
#[gpui_kit::test]
fn making_a_share_link_under_embargo_asks_first(cx: &mut TestAppContext) {
    let server = MemoryCollab::new();
    let owner: Device = device(&server, 1);
    let _member: Device = device(&server, 2);
    owner.engine.person("user2").unwrap();
    let f = open_in(
        cx,
        owner.store.clone(),
        None,
        BODY,
        Some(owner.engine.clone()),
    );
    owner
        .engine
        .share(&f.note, &[("user2".to_owned(), Role::Edit)])
        .unwrap();
    let editor = f.editor(cx);
    let sheet = f
        .window
        .update(cx, |_, window, cx| {
            crate::share_sheet::open_share(editor, window, cx)
        })
        .unwrap()
        .expect("a note is open");
    let settle = |cx: &mut TestAppContext| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        loop {
            cx.run_until_parked();
            if !cx.read(|cx| sheet.read(cx).is_busy()) {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "the step finished");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    };
    settle(cx);
    sheet.update(cx, |sheet, cx| sheet.make_link(cx));
    settle(cx);
    let ask = cx
        .read(|cx| sheet.read(cx).embargo_ask().cloned())
        .expect("it asks");
    assert_eq!(ask.confirm, "Make link anyway");
    assert!(
        ask.body.ends_with("Make a share link anyway?"),
        "{}",
        ask.body
    );
    assert!(cx.read(|cx| sheet.read(cx).made_link.is_none()));

    sheet.update(cx, |sheet, cx| sheet.make_link_anyway(cx));
    settle(cx);
    assert!(cx.read(|cx| sheet.read(cx).embargo_ask().is_none()));
    let made = cx.read(|cx| sheet.read(cx).made_link.clone());
    assert!(
        made.is_some(),
        "{:?}",
        cx.read(|cx| sheet.read(cx).error.clone())
    );
}

// no-prop-test: one flow; the engine's reminder properties cover the rules.
#[gpui_kit::test]
fn the_lift_notice_shows_when_the_embargo_lifts(cx: &mut TestAppContext) {
    let f = setup(cx, BODY);
    let store = f.store.clone();
    cx.update(|cx: &mut App| {
        cx.set_app_identity("app.kanso.Test", "Kanso Test");
        crate::reminders::install_for_test(store, cx);
    });
    cx.update(crate::reminders::check);
    assert!(
        cx.shown_system_notifications().is_empty(),
        "nothing before it lifts"
    );
    set_now(cx, LIFTS + 30_000);
    cx.update(crate::reminders::check);
    let shown = cx.shown_system_notifications();
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].title.as_ref(), "Embargo lifted: Pixel 11 review");
    assert_eq!(shown[0].body.as_ref(), "Today 14:00 · Pixel 11 review");
    let actions: Vec<&str> = shown[0].actions.iter().map(|a| a.id.as_ref()).collect();
    assert_eq!(actions, ["dismiss"], "nothing to tick off");
    // Once is enough.
    cx.update(crate::reminders::check);
    assert_eq!(cx.shown_system_notifications().len(), 1);
}
