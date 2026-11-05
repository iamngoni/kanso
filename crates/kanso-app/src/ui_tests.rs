//! The workspace in a headless window: autosave, switching notes, and
//! changes made elsewhere while a note is open. These are where an editor
//! loses data, so they run against the real views and store.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

use crate::config::AppConfig;
use crate::services::Services;
use crate::workspace::{AppState, Workspace};

struct Fixture {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
    notes: [String; 2],
}

fn note_body(store: &Store, id: &str) -> String {
    store.note(id).unwrap().unwrap().body
}

/// Whether `text` is the body of a live note or of a version in history.
fn persisted(store: &Store, ids: &[String], text: &str) -> bool {
    ids.iter().any(|id| {
        store.note(id).unwrap().is_some_and(|n| n.body == text)
            || store
                .revisions(id)
                .unwrap()
                .iter()
                .any(|r| r.fields.get("body").and_then(|v| v.as_str()) == Some(text))
    })
}

fn setup(cx: &mut TestAppContext) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let note = |title: &str| {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": "start", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    };
    let notes = [note("a"), note("b")];
    let data_dir = dir.path().to_path_buf();
    let app_store = store.clone();
    let (window, workspace) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store: app_store,
            data_dir,
            config: AppConfig::default(),
        });
        cx.set_global(Services::default());
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    Fixture {
        _dir: dir,
        store,
        window,
        workspace,
        notes,
    }
}

impl Fixture {
    fn select(&self, cx: &mut TestAppContext, id: &str) {
        let library = cx.read(|cx| self.workspace.read(cx).library.clone());
        let id = id.to_owned();
        library.update(cx, |library, cx| library.select(Some(id), cx));
        cx.run_until_parked();
    }

    fn type_body(&self, cx: &mut TestAppContext, body: &str) {
        let editor = cx.read(|cx| self.workspace.read(cx).editor.clone());
        self.window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| editor.edit_for_test(body, window, cx))
            })
            .unwrap();
    }

    fn editor<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&crate::editor::NoteEditor, &gpui_kit::App) -> R,
    ) -> R {
        cx.read(|cx| {
            let editor = self.workspace.read(cx).editor.clone();
            f(editor.read(cx), cx)
        })
    }

    fn update_editor(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(
            &mut crate::editor::NoteEditor,
            &mut gpui_kit::Window,
            &mut gpui_kit::Context<crate::editor::NoteEditor>,
        ),
    ) {
        let editor = cx.read(|cx| self.workspace.read(cx).editor.clone());
        self.window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| f(editor, window, cx))
            })
            .unwrap();
        cx.run_until_parked();
    }

    /// Draws, so focus, actions and element bounds find the views.
    fn draw(&self, cx: &mut TestAppContext) {
        cx.run_until_parked();
        self.window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
    }

    fn workspace<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&Workspace, &gpui_kit::App) -> R,
    ) -> R {
        cx.read(|cx| f(self.workspace.read(cx), cx))
    }

    /// The settings as written to disk.
    fn saved_config(&self) -> AppConfig {
        AppConfig::load(&crate::paths::config_path(self._dir.path())).unwrap()
    }

    fn open_note(&self, cx: &mut TestAppContext) -> Option<String> {
        self.editor(cx, |e, _| e.note_id().map(str::to_owned))
    }

    fn edit_elsewhere(&self, cx: &mut TestAppContext, id: &str, body: &str) {
        self.store
            .put(
                DocType::Note,
                json!({"_id": id, "body": body}),
                Put::Merge,
                Origin::Api,
            )
            .unwrap();
        cx.run_until_parked();
    }
}

#[gpui_kit::test]
fn autosave_writes_after_a_pause(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.type_body(cx, "typed");
    cx.run_until_parked();
    assert_eq!(
        note_body(&f.store, &f.notes[0]),
        "start",
        "saved before the pause"
    );
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    assert_eq!(note_body(&f.store, &f.notes[0]), "typed");
}

#[gpui_kit::test]
fn switching_notes_saves_pending_edits(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.type_body(cx, "unsaved yet");
    f.select(cx, &f.notes[1]);
    assert_eq!(note_body(&f.store, &f.notes[0]), "unsaved yet");
    assert_eq!(f.editor(cx, |e, cx| e.body_for_test(cx)), "start");
}

#[gpui_kit::test]
fn a_change_elsewhere_reloads_a_clean_note(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.edit_elsewhere(cx, &f.notes[0], "from the api");
    assert_eq!(f.editor(cx, |e, cx| e.body_for_test(cx)), "from the api");
    assert!(!f.editor(cx, |e, _| e.has_incoming()));
}

#[gpui_kit::test]
fn keeping_mine_over_a_change_elsewhere_keeps_theirs_in_history(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.type_body(cx, "mine");
    f.edit_elsewhere(cx, &id, "theirs");
    assert!(f.editor(cx, |e, _| e.has_incoming()), "asks which to keep");
    // The pending autosave must not overwrite theirs while the question is open.
    cx.executor().advance_clock(Duration::from_secs(2));
    cx.run_until_parked();
    assert_eq!(note_body(&f.store, &id), "theirs");

    let editor = cx.read(|cx| f.workspace.read(cx).editor.clone());
    editor.update(cx, |editor, cx| editor.keep_mine(cx));
    cx.run_until_parked();
    assert_eq!(note_body(&f.store, &id), "mine");
    let history = f.store.revisions(&id).unwrap();
    assert!(
        history.iter().any(|r| r.fields["body"] == "theirs"),
        "theirs is in history"
    );
}

#[gpui_kit::test]
fn using_theirs_keeps_my_draft_in_history(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.type_body(cx, "my draft");
    f.edit_elsewhere(cx, &id, "theirs");
    let editor = cx.read(|cx| f.workspace.read(cx).editor.clone());
    f.window
        .update(cx, |_, window, cx| {
            editor.update(cx, |editor, cx| editor.take_theirs(window, cx))
        })
        .unwrap();
    cx.run_until_parked();
    assert_eq!(f.editor(cx, |e, cx| e.body_for_test(cx)), "theirs");
    assert_eq!(note_body(&f.store, &id), "theirs");
    let history = f.store.revisions(&id).unwrap();
    assert!(
        history.iter().any(|r| r.fields["body"] == "my draft"),
        "my draft is in history"
    );
}

#[gpui_kit::test]
fn quitting_from_a_window_saves_the_open_note(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(|cx| {
        cx.on_action(|_: &crate::workspace::Quit, cx| crate::quit(cx));
    });
    f.select(cx, &f.notes[0]);
    f.type_body(cx, "typed then quit");
    // ⌘Q arrives while the window is being updated.
    cx.dispatch_action(f.window, crate::workspace::Quit);
    assert_eq!(note_body(&f.store, &f.notes[0]), "typed then quit");
}

/// "Update ready · Restart" quits. Its click (and the sheet's "Restart
/// now") runs while its window is being updated, so a quit that goes
/// through `App::dispatch_action` would be dropped: GPUI can't update the
/// window that is already updating, and only logs that.
#[gpui_kit::test]
fn restart_from_the_update_pill_quits_and_saves(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(|cx| {
        cx.on_action(|_: &crate::workspace::Quit, cx| crate::quit(cx));
        let mut updates = crate::updates::Updates::default();
        updates.phase = crate::updates::Phase::Ready(kanso_update::Pending {
            version: "9.9.9".into(),
            payload: "/x/.kanso-update/unpacked/Kanso.app".into(),
            target: "/x/Kanso.app".into(),
            pub_date: String::new(),
            release_url: String::new(),
            notes: String::new(),
            seal: Default::default(),
        });
        cx.set_global(updates);
    });
    f.select(cx, &f.notes[0]);
    f.type_body(cx, "typed then restarted");
    // As in the app: the clicked window is the active one.
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    assert_eq!(cx.update(|cx| cx.active_window()), Some(f.window));
    f.window
        .update(cx, |_, _, cx| crate::updates::restart_now(cx))
        .unwrap();
    cx.run_until_parked();
    // Kanso's quit saves the open note (the test platform's own quit does
    // nothing), so the saved draft is the sign it ran.
    assert_eq!(note_body(&f.store, &f.notes[0]), "typed then restarted");
}

#[gpui_kit::test]
fn quitting_the_app_saves_the_open_note(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.type_body(cx, "typed then logged out");
    // As from the Dock or at logout, without the Quit action.
    cx.quit();
    assert_eq!(note_body(&f.store, &f.notes[0]), "typed then logged out");
}

#[gpui_kit::test]
fn leaving_a_note_before_answering_keeps_mine(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.type_body(cx, "mine before switch");
    f.edit_elsewhere(cx, &id, "theirs");
    assert!(f.editor(cx, |e, _| e.has_incoming()));
    f.select(cx, &f.notes[1]);
    assert_eq!(note_body(&f.store, &id), "mine before switch");
    let history = f.store.revisions(&id).unwrap();
    assert!(
        history.iter().any(|r| r.fields["body"] == "theirs"),
        "theirs is in history"
    );
}

#[gpui_kit::test]
fn a_note_deleted_elsewhere_asks_before_dropping_edits(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.type_body(cx, "unsaved when deleted");
    f.store.delete(&id, Origin::Sync).unwrap();
    cx.run_until_parked();
    assert!(f.editor(cx, |e, _| e.has_incoming()), "asks what to do");
    assert_eq!(f.open_note(cx).as_deref(), Some(id.as_str()));
    // The pending autosave must not bring the note back on its own.
    cx.executor().advance_clock(Duration::from_secs(2));
    cx.run_until_parked();
    assert!(f.store.note(&id).unwrap().is_none());

    f.update_editor(cx, |editor, _, cx| editor.keep_mine(cx));
    assert_eq!(note_body(&f.store, &id), "unsaved when deleted");
    assert!(!f.editor(cx, |e, _| e.has_incoming()));
}

#[gpui_kit::test]
fn discarding_a_note_deleted_elsewhere_closes_it(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.type_body(cx, "discard me");
    f.store.delete(&id, Origin::Sync).unwrap();
    cx.run_until_parked();
    f.update_editor(cx, |editor, window, cx| editor.take_theirs(window, cx));
    assert_eq!(f.open_note(cx), None);
    assert!(f.store.note(&id).unwrap().is_none(), "stays deleted");
}

#[gpui_kit::test]
fn a_clean_note_deleted_elsewhere_closes(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.store.delete(&id, Origin::Sync).unwrap();
    cx.run_until_parked();
    assert_eq!(f.open_note(cx), None);
    assert!(!f.editor(cx, |e, _| e.has_incoming()));
}

#[gpui_kit::test]
fn keeping_a_note_whose_notebook_is_gone_moves_it(cx: &mut TestAppContext) {
    let f = setup(cx);
    let other = f
        .store
        .put(
            DocType::Book,
            json!({"name": "Gone"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let id = f
        .store
        .put(
            DocType::Note,
            json!({"title": "c", "body": "start", "bookId": other}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    cx.run_until_parked();
    f.select(cx, &id);
    f.type_body(cx, "outlived its notebook");
    f.store.delete(&id, Origin::Sync).unwrap();
    f.store.delete(&other, Origin::Sync).unwrap();
    cx.run_until_parked();
    f.update_editor(cx, |editor, _, cx| editor.keep_mine(cx));
    let note = f.store.note(&id).unwrap().expect("brought back");
    assert_eq!(note.body, "outlived its notebook");
    assert_ne!(note.book_id, other);
}

/// Things a user, another app or another Mac can do while a note is open.
#[derive(Debug, Clone)]
enum Op {
    Type,
    Wait,
    EditElsewhere,
    DeleteElsewhere,
    Switch,
    KeepMine,
    TakeTheirs,
    Save,
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => Just(Op::Type),
        2 => Just(Op::Wait),
        2 => Just(Op::EditElsewhere),
        1 => Just(Op::DeleteElsewhere),
        2 => Just(Op::Switch),
        1 => Just(Op::KeepMine),
        1 => Just(Op::TakeTheirs),
        1 => Just(Op::Save),
    ]
}

/// Runs `ops` and checks after each one that every text typed into the
/// editor is still there or has reached the store (as a live note or in
/// history), and that quitting at the end leaves nothing unsaved. Text
/// replaced by more typing before anything else happened is only keystrokes
/// and may go.
fn typed_text_survives(ops: &[Op]) -> Result<(), TestCaseError> {
    let mut cx = TestAppContext::single();
    let cx = &mut cx;
    let f = setup(cx);
    let ids = f.notes.to_vec();
    let mut current = 0;
    f.select(cx, &ids[current]);
    let mut outstanding: Vec<String> = Vec::new();
    for (step, op) in ops.iter().enumerate() {
        let open = f.open_note(cx);
        match op {
            Op::Type => {
                if open.is_none() {
                    continue;
                }
                let replaced = f.editor(cx, |e, cx| e.body_for_test(cx));
                outstanding.retain(|t| *t != replaced);
                let text = format!("typed {step}");
                f.type_body(cx, &text);
                outstanding.push(text);
            }
            Op::Wait => cx.executor().advance_clock(Duration::from_secs(1)),
            Op::EditElsewhere => {
                if let Some(id) = open {
                    f.store
                        .put(
                            DocType::Note,
                            json!({"_id": id, "body": format!("elsewhere {step}")}),
                            Put::Merge,
                            Origin::Sync,
                        )
                        .ok();
                }
            }
            Op::DeleteElsewhere => {
                if let Some(id) = open {
                    f.store.delete(&id, Origin::Sync).ok();
                }
            }
            Op::Switch => {
                current = 1 - current;
                f.select(cx, &ids[current]);
            }
            Op::KeepMine => f.update_editor(cx, |e, _, cx| e.keep_mine(cx)),
            Op::TakeTheirs => f.update_editor(cx, |e, window, cx| e.take_theirs(window, cx)),
            Op::Save => f.update_editor(cx, |e, _, cx| e.flush(cx)),
        }
        cx.run_until_parked();
        let in_editor = f.editor(cx, |e, cx| e.body_for_test(cx));
        outstanding.retain(|t| !persisted(&f.store, &ids, t));
        for text in &outstanding {
            prop_assert_eq!(
                text,
                &in_editor,
                "'{}' was lost after step {} ({:?})",
                text,
                step,
                op
            );
        }
    }
    cx.quit();
    outstanding.retain(|t| !persisted(&f.store, &ids, t));
    prop_assert!(outstanding.is_empty(), "lost on quit: {:?}", outstanding);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    })]

    /// Model-based: no sequence of edits, changes and deletions elsewhere,
    /// note switches, answers and quitting loses typed text.
    #[test]
    fn prop_typed_text_is_never_lost(ops in prop::collection::vec(op(), 1..24)) {
        typed_text_survives(&ops)?;
    }
}

fn saved_last_note(f: &Fixture) -> Option<String> {
    let path = crate::paths::config_path(f._dir.path());
    AppConfig::load(&path).unwrap().last_note
}

#[gpui_kit::test]
fn the_selected_note_is_saved_once_selection_settles(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.select(cx, &f.notes[1]);
    assert_eq!(saved_last_note(&f), None, "written on every click");
    cx.executor().advance_clock(Duration::from_secs(3));
    cx.run_until_parked();
    assert_eq!(saved_last_note(&f), Some(f.notes[1].clone()));
}

#[gpui_kit::test]
fn quitting_saves_the_selected_note_right_away(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(crate::watch_for_quit);
    f.select(cx, &f.notes[1]);
    cx.quit();
    assert_eq!(saved_last_note(&f), Some(f.notes[1].clone()));
}

/// A press on bare chrome leaves the keyboard in the note: on the editor's
/// top band, where a drag moves the window and a double-click zooms it, and
/// on its footer. The workspace's own focus is only for when nothing else
/// has the keyboard.
#[gpui_kit::test]
fn a_press_on_the_chrome_keeps_the_keyboard(cx: &mut TestAppContext) {
    use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT};
    use gpui_kit::{Modifiers, MouseButton, MouseDownEvent, MouseUpEvent, point, px};

    let f = setup(cx);
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    let body = f
        .window
        .update(cx, |_, window, cx| {
            let editor = f.workspace.read(cx).editor.clone();
            editor.update(cx, |editor, cx| editor.focus_body(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
            window.focused(cx).expect("the body takes the keyboard")
        })
        .unwrap();
    cx.run_until_parked();
    // The middle of the editor pane, between the path and the buttons.
    let (x, height) = f
        .window
        .update(cx, |_, window, cx| {
            let workspace = f.workspace.read(cx);
            let widths = workspace.pane_widths;
            let size = window.viewport_size();
            let left = widths.list
                + if workspace.show_sidebar {
                    widths.sidebar
                } else {
                    0.
                };
            ((left + f32::from(size.width)) / 2., f32::from(size.height))
        })
        .unwrap();
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let press = |vcx: &mut gpui_kit::VisualTestContext, y: f32, click_count: usize| {
        let position = point(px(x), px(y));
        vcx.simulate_event(MouseDownEvent {
            position,
            modifiers: Modifiers::none(),
            button: MouseButton::Left,
            click_count,
            first_mouse: false,
        });
        vcx.simulate_event(MouseUpEvent {
            position,
            modifiers: Modifiers::none(),
            button: MouseButton::Left,
            click_count,
        });
    };
    let body_focused =
        |vcx: &mut gpui_kit::VisualTestContext| vcx.update(|window, _| body.is_focused(window));
    press(&mut vcx, TOP_BAND_HEIGHT / 2., 1);
    assert!(body_focused(&mut vcx), "a press on the top band");
    press(&mut vcx, TOP_BAND_HEIGHT / 2., 2);
    assert!(body_focused(&mut vcx), "a double-click on the top band");
    press(&mut vcx, height - BOTTOM_BAND_HEIGHT / 2., 1);
    assert!(body_focused(&mut vcx), "a press on the footer");
    // Typing afterwards still reaches the note.
    vcx.simulate_input("!");
    let typed = f.editor(cx, |e, cx| e.body_for_test(cx));
    assert!(typed.contains('!'), "{typed:?}");
}

/// A second window, as on the next launch, reading the settings the first
/// one wrote.
fn reopen(cx: &mut TestAppContext) -> Entity<Workspace> {
    reopen_window(cx).1
}

fn reopen_window(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Workspace>) {
    let (window, workspace) = cx.update(|cx| {
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    (window, workspace)
}

/// Presenting saves the typing waiting, walks the slides by key without going
/// past the last, leaves the note as it was, and Esc ends it.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn presenting_walks_the_slides_and_esc_ends_it(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.type_body(cx, "One\n\n---\n\nTwo");
    let present = id.clone();
    f.window
        .update(cx, |_, window, cx| {
            crate::presentation::present(present, window, cx)
        })
        .unwrap();
    cx.run_until_parked();

    let handle = cx
        .update(|cx| cx.windows().into_iter().find(|w| *w != f.window))
        .expect("the slides have a window");
    let view = handle
        .update(cx, |root, _, cx| {
            let root = root.downcast::<gpui_kit::base::Root>().unwrap();
            root.read(cx)
                .view()
                .clone()
                .downcast::<crate::presentation::Presentation>()
                .unwrap()
        })
        .unwrap();
    let shown = |cx: &mut TestAppContext| cx.read(|cx| view.read(cx).shown_for_test(cx));
    assert_eq!(shown(cx), (0, "One".into()));

    handle
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    let mut vcx = gpui_kit::VisualTestContext::from_window(handle, cx);
    vcx.simulate_keystrokes("right");
    vcx.run_until_parked();
    vcx.simulate_keystrokes("space");
    vcx.run_until_parked();
    assert_eq!(shown(&mut vcx), (1, "Two".into()), "the last slide stays");
    vcx.simulate_keystrokes("left");
    vcx.run_until_parked();
    assert_eq!(shown(&mut vcx), (0, "One".into()));
    assert_eq!(note_body(&f.store, &id), "One\n\n---\n\nTwo");

    vcx.simulate_keystrokes("escape");
    vcx.run_until_parked();
    assert_eq!(cx.update(|cx| cx.windows().len()), 1, "Esc closes it");
}

/// A slide with an audio line shows the player card, and a click on it does
/// not take Space or the arrows from the slides.
// no-prop-test: drives one window through clicks and keystrokes.
#[gpui_kit::test]
fn presentation_keeps_the_slide_keys_with_a_player_card_on_show(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.type_body(
        cx,
        "[Audio \u{b7} 3 min](inkdrop://file:ghost-1)\n\n---\n\nTwo\n\n---\n\nThree",
    );
    let present = id.clone();
    f.window
        .update(cx, |_, window, cx| {
            crate::presentation::present(present, window, cx)
        })
        .unwrap();
    cx.run_until_parked();
    let handle = cx
        .update(|cx| cx.windows().into_iter().find(|w| *w != f.window))
        .expect("the slides have a window");
    let view = handle
        .update(cx, |root, _, cx| {
            let root = root.downcast::<gpui_kit::base::Root>().unwrap();
            root.read(cx)
                .view()
                .clone()
                .downcast::<crate::presentation::Presentation>()
                .unwrap()
        })
        .unwrap();
    let shown = |cx: &mut TestAppContext| cx.read(|cx| view.read(cx).shown_for_test(cx)).0;
    handle
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    let mut vcx = gpui_kit::VisualTestContext::from_window(handle, cx);
    for _ in 0..3 {
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }
    let card = vcx
        .debug_bounds("recording-player")
        .expect("the audio slide shows the card");
    vcx.simulate_click(card.center(), gpui_kit::Modifiers::none());
    vcx.run_until_parked();
    vcx.simulate_keystrokes("space");
    vcx.run_until_parked();
    assert_eq!(
        shown(&mut vcx),
        1,
        "Space moved on after a click on the card"
    );
    vcx.simulate_keystrokes("left");
    vcx.run_until_parked();
    assert_eq!(shown(&mut vcx), 0, "the arrows too");
    vcx.simulate_keystrokes("right");
    vcx.run_until_parked();
    assert_eq!(shown(&mut vcx), 1);
}

/// ⌘⇧D leaves the note alone in the window, with the keyboard in its text,
/// and the next launch starts the same way. The palette still opens.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn focus_mode_leaves_the_note_alone_and_is_kept(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    assert!(vcx.debug_bounds("sidebar-pane").is_some());
    assert!(vcx.debug_bounds("list-pane").is_some());

    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.focus_mode));
    assert!(f.saved_config().layout.focus_mode);
    assert_eq!(vcx.debug_bounds("sidebar-pane"), None);
    assert_eq!(vcx.debug_bounds("list-pane"), None);
    let editor = vcx.debug_bounds("editor-pane").unwrap();
    let width = vcx.update(|window, _| window.viewport_size().width);
    assert_eq!(editor.size.width, width, "the editor has the window");
    let editor_view = f.workspace(&mut vcx, |w, _| w.editor.clone());
    assert!(
        vcx.update(|window, cx| editor_view.read(cx).body_focused(window, cx)),
        "the keyboard goes to the text"
    );
    // Typing reaches the note.
    vcx.simulate_input("!");
    assert!(
        f.editor(&mut vcx, |e, cx| e.body_for_test(cx))
            .contains('!')
    );

    // The palette still opens, over the note.
    let palette = f.workspace(&mut vcx, |w, _| w.palette.clone());
    vcx.simulate_keystrokes("cmd-k");
    assert!(vcx.read(|cx| palette.read(cx).is_open()));

    // The next launch starts in focus mode, with the keyboard in the
    // last note's text.
    let (next_window, next) = reopen_window(&mut vcx);
    assert!(vcx.read(|cx| next.read(cx).focus_mode));
    let next_editor = vcx.read(|cx| next.read(cx).editor.clone());
    assert_eq!(
        vcx.read(|cx| next_editor.read(cx).note_id().map(str::to_owned)),
        Some(f.notes[0].clone())
    );
    assert!(
        next_window
            .update(&mut vcx, |_, window, cx| next_editor
                .read(cx)
                .body_focused(window, cx))
            .unwrap(),
        "the keyboard is in the text at launch"
    );

    vcx.simulate_keystrokes("escape");
    assert!(!vcx.read(|cx| palette.read(cx).is_open()));
    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.focus_mode));
    assert!(!f.saved_config().layout.focus_mode);
    assert!(vcx.debug_bounds("sidebar-pane").is_some());
    assert!(vcx.debug_bounds("list-pane").is_some());
}

/// With Live Preview in focus mode on, ⌘⇧D shows the note in Live and
/// leaving shows the mode from before; the view mode kept for next time
/// never changes. A mode picked in focus mode stands when it ends, and with
/// the setting off focus mode leaves the mode alone.
// no-prop-test: drives one window through keystrokes; `FocusLive` has the
// properties.
#[gpui_kit::test]
fn focus_mode_shows_live_preview_when_asked(cx: &mut TestAppContext) {
    use crate::config::ViewMode;

    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let saved = f.saved_config().appearance.view_mode;
    assert_eq!(f.editor(cx, |e, _| e.view_mode), ViewMode::Split);
    let live_in_focus = |on: bool| crate::config::EditorSettings {
        live_preview_in_focus: on,
        ..Default::default()
    };
    f.update_editor(cx, |editor, _, cx| {
        editor.set_settings(live_in_focus(true), cx)
    });
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);

    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.focus_mode));
    assert_eq!(f.editor(&mut vcx, |e, _| e.view_mode), ViewMode::Live);
    assert_eq!(f.saved_config().appearance.view_mode, saved);
    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.focus_mode));
    assert_eq!(f.editor(&mut vcx, |e, _| e.view_mode), ViewMode::Split);
    assert_eq!(f.saved_config().appearance.view_mode, saved);

    // A mode picked in focus mode is the one shown after it.
    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert_eq!(f.editor(&mut vcx, |e, _| e.view_mode), ViewMode::Live);
    vcx.update(|_, cx| {
        f.workspace.update(cx, |workspace, cx| {
            workspace.set_view_mode_for_test(ViewMode::Edit, cx)
        })
    });
    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.focus_mode));
    assert_eq!(f.editor(&mut vcx, |e, _| e.view_mode), ViewMode::Edit);
    assert_eq!(f.saved_config().appearance.view_mode, ViewMode::Edit);

    // With the setting off, focus mode keeps the mode.
    f.update_editor(&mut vcx, |editor, _, cx| {
        editor.set_settings(live_in_focus(false), cx)
    });
    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.focus_mode));
    assert_eq!(f.editor(&mut vcx, |e, _| e.view_mode), ViewMode::Edit);
    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert_eq!(f.editor(&mut vcx, |e, _| e.view_mode), ViewMode::Edit);
    assert_eq!(f.saved_config().appearance.view_mode, ViewMode::Edit);
}

/// Escape leaves focus mode only once nothing else wants it: the first
/// Escape with the palette open closes the palette; the next, from the
/// text, leaves focus mode.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn escape_leaves_focus_mode_last(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    let palette = f.workspace(&mut vcx, |w, _| w.palette.clone());
    vcx.simulate_keystrokes("cmd-k");
    f.draw(&mut vcx);
    assert!(vcx.read(|cx| palette.read(cx).is_open()));

    vcx.simulate_keystrokes("escape");
    f.draw(&mut vcx);
    assert!(
        !vcx.read(|cx| palette.read(cx).is_open()),
        "the palette closes"
    );
    assert!(
        f.workspace(&mut vcx, |w, _| w.focus_mode),
        "focus mode stays"
    );
    let editor = f.workspace(&mut vcx, |w, _| w.editor.clone());
    assert!(vcx.update(|window, cx| editor.read(cx).body_focused(window, cx)));

    vcx.simulate_keystrokes("escape");
    f.draw(&mut vcx);
    assert!(
        !f.workspace(&mut vcx, |w, _| w.focus_mode),
        "the next one leaves"
    );
    assert!(!f.saved_config().layout.focus_mode);
    // Out of focus mode, Escape in the text leaves things as they are: the
    // keyboard stays in the text.
    vcx.simulate_keystrokes("escape");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.focus_mode));
    assert!(vcx.update(|window, cx| editor.read(cx).body_focused(window, cx)));
    // Escape is only ever bound to leave something, never to move the
    // keyboard out of the text, where a deeper binding would win over
    // leaving focus mode.
    let escapes: Vec<&'static str> = vcx.update(|_, cx| {
        let keymap = cx.key_bindings();
        let keymap = keymap.borrow();
        keymap
            .bindings()
            .filter(|binding| {
                binding
                    .keystrokes()
                    .first()
                    .is_some_and(|k| gpui_kit::AsKeystroke::as_keystroke(k).key == "escape")
            })
            .map(|binding| binding.action().name())
            .collect()
    });
    assert!(
        !escapes.iter().any(|name| name.ends_with("FocusNoteList")),
        "{escapes:?}"
    );
}

/// With Vim keys, Escape in Insert mode goes to Normal mode and stays in
/// focus mode; Escape in Normal mode then leaves focus mode as it would
/// without Vim. ⌘ shortcuts work in Normal mode.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn vim_escape_leaves_insert_mode_then_focus_mode(cx: &mut TestAppContext) {
    use crate::vim::Mode;
    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    f.update_editor(cx, |editor, _, cx| {
        editor.set_settings(
            crate::config::EditorSettings {
                vim: true,
                ..Default::default()
            },
            cx,
        )
    });
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.focus_mode));
    let before = f.editor(&mut vcx, |e, cx| e.body_for_test(cx));
    vcx.simulate_keystrokes("i");
    vcx.simulate_input("ok ");
    assert_eq!(f.editor(&mut vcx, |e, _| e.vim_mode()), Some(Mode::Insert));
    vcx.simulate_keystrokes("escape");
    f.draw(&mut vcx);
    assert_eq!(f.editor(&mut vcx, |e, _| e.vim_mode()), Some(Mode::Normal));
    assert!(
        f.workspace(&mut vcx, |w, _| w.focus_mode),
        "focus mode stays"
    );
    assert_eq!(
        f.editor(&mut vcx, |e, cx| e.body_for_test(cx)),
        format!("ok {before}")
    );

    // The palette opens from Normal mode, and Escape closes it.
    let palette = f.workspace(&mut vcx, |w, _| w.palette.clone());
    vcx.simulate_keystrokes("cmd-k");
    f.draw(&mut vcx);
    assert!(vcx.read(|cx| palette.read(cx).is_open()));
    vcx.simulate_keystrokes("escape");
    f.draw(&mut vcx);
    assert!(!vcx.read(|cx| palette.read(cx).is_open()));
    assert!(f.workspace(&mut vcx, |w, _| w.focus_mode));

    // A command half typed takes the first Escape; the next leaves.
    let editor = f.workspace(&mut vcx, |w, _| w.editor.clone());
    assert!(vcx.update(|window, cx| editor.read(cx).body_focused(window, cx)));
    vcx.simulate_keystrokes("d escape");
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.focus_mode));
    vcx.simulate_keystrokes("escape");
    f.draw(&mut vcx);
    assert!(
        !f.workspace(&mut vcx, |w, _| w.focus_mode),
        "Escape in Normal mode leaves focus mode"
    );
    assert_eq!(
        f.editor(&mut vcx, |e, cx| e.body_for_test(cx)),
        format!("ok {before}")
    );
}

/// Focus mode hides the header band, the status and tags and the toolbar,
/// and takes their controls out of the Tab order with them: Tab and
/// Shift-Tab from the title stay in the note, where before Shift-Tab landed
/// on the invisible band. Out of focus mode Shift-Tab from the title does
/// reach the band.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn tab_in_focus_mode_never_lands_on_hidden_controls(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let editor = f.workspace(&mut vcx, |w, _| w.editor.clone());
    // Where the keyboard is after `key` from the title: in the note or not.
    let from_title = |vcx: &mut gpui_kit::VisualTestContext, key: &str| {
        f.update_editor(vcx, |e, window, cx| e.focus_title(window, cx));
        f.draw(vcx);
        assert!(vcx.update(|window, cx| editor.read(cx).title_focused(window, cx)));
        vcx.simulate_keystrokes(key);
        f.draw(vcx);
        vcx.update(|window, cx| {
            let editor = editor.read(cx);
            editor.title_focused(window, cx) || editor.body_focused(window, cx)
        })
    };
    assert!(
        !from_title(&mut vcx, "shift-tab"),
        "out of focus mode the band's controls are in the Tab order"
    );

    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.focus_mode));
    for key in ["tab", "shift-tab", "tab tab", "shift-tab shift-tab"] {
        assert!(
            from_title(&mut vcx, key),
            "{key} from the title stays in the note"
        );
    }
}

/// ⌘⇧D with the palette open closes the palette on the way into focus mode
/// (its field loses the keyboard to the note's text): left open, it would
/// hide the note being typed into, out of Escape's reach.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn focus_mode_closes_the_palette(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let palette = f.workspace(&mut vcx, |w, _| w.palette.clone());
    let editor = f.workspace(&mut vcx, |w, _| w.editor.clone());
    vcx.simulate_keystrokes("cmd-k");
    f.draw(&mut vcx);
    assert!(vcx.read(|cx| palette.read(cx).is_open()));

    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.focus_mode));
    assert!(
        !vcx.read(|cx| palette.read(cx).is_open()),
        "the palette closes"
    );
    assert!(vcx.update(|window, cx| editor.read(cx).body_focused(window, cx)));
    // The next Escape is focus mode's.
    vcx.simulate_keystrokes("escape");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.focus_mode));
}

/// ⇧⌘L takes the keyboard from the note's text to the note list, from
/// focus mode and from Settings too, so the list's arrow keys are always a
/// keystroke away; Return goes back into the note.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn the_keyboard_reaches_the_note_list(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(|cx| {
        crate::workspace::bind_keys(cx);
        crate::note_list::bind_keys(cx);
    });
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    f.update_editor(cx, |editor, window, cx| editor.focus_body(window, cx));
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let (editor, list) = f.workspace(&mut vcx, |w, _| (w.editor.clone(), w.note_list.clone()));
    let list_focused = |vcx: &mut gpui_kit::VisualTestContext| {
        vcx.update(|window, cx| {
            gpui_kit::Focusable::focus_handle(list.read(cx), cx).is_focused(window)
        })
    };
    let body_focused = |vcx: &mut gpui_kit::VisualTestContext| {
        vcx.update(|window, cx| editor.read(cx).body_focused(window, cx))
    };
    assert!(body_focused(&mut vcx));

    vcx.simulate_keystrokes("cmd-shift-l");
    f.draw(&mut vcx);
    assert!(list_focused(&mut vcx));
    vcx.simulate_keystrokes("enter");
    f.draw(&mut vcx);
    assert!(body_focused(&mut vcx));

    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.focus_mode));
    vcx.simulate_keystrokes("cmd-shift-l");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.focus_mode));
    assert!(list_focused(&mut vcx));

    vcx.simulate_keystrokes("cmd-,");
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.settings_open()));
    vcx.simulate_keystrokes("cmd-shift-l");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.settings_open()));
    assert!(list_focused(&mut vcx));
}

/// ⌘\ folds the sidebar to its icon rail rather than hiding it, keeps that
/// for next time, and from focus mode brings everything back.
// no-prop-test: drives one window through keystrokes; the widths' rules
// are properties in layout.rs.
#[gpui_kit::test]
fn the_sidebar_folds_to_its_rail(cx: &mut TestAppContext) {
    use crate::layout::rail_width;

    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let sidebar = f.workspace(&mut vcx, |w, _| w.sidebar.clone());
    let full = vcx.debug_bounds("sidebar-pane").unwrap().size.width;

    vcx.simulate_keystrokes("cmd-\\");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.show_sidebar));
    assert!(!f.saved_config().appearance.show_sidebar);
    assert!(vcx.read(|cx| sidebar.read(cx).is_collapsed()));
    let rail = vcx.debug_bounds("sidebar-pane").expect("the rail stays");
    assert_eq!(f32::from(rail.size.width), rail_width(false));
    assert!(rail.size.width < full);
    // The list starts where the rail ends.
    let list = vcx.debug_bounds("list-pane").unwrap();
    assert_eq!(list.left(), rail.right());

    vcx.simulate_keystrokes("cmd-\\");
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.show_sidebar));
    assert!(f.saved_config().appearance.show_sidebar);
    assert!(!vcx.read(|cx| sidebar.read(cx).is_collapsed()));
    assert_eq!(vcx.debug_bounds("sidebar-pane").unwrap().size.width, full);

    // The mouse does the same: the button at the end of the sidebar's top
    // band folds it, and the one at the top of the rail opens it again.
    let click = |vcx: &mut gpui_kit::VisualTestContext, selector: &'static str| {
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        vcx.simulate_click(bounds.center(), gpui_kit::Modifiers::none());
    };
    click(&mut vcx, "sidebar-toggle");
    f.draw(&mut vcx);
    assert!(vcx.read(|cx| sidebar.read(cx).is_collapsed()));
    assert!(!f.saved_config().appearance.show_sidebar);
    assert_eq!(vcx.debug_bounds("sidebar-toggle"), None);
    click(&mut vcx, "rail-expand");
    f.draw(&mut vcx);
    assert!(!vcx.read(|cx| sidebar.read(cx).is_collapsed()));
    assert!(f.saved_config().appearance.show_sidebar);
    assert_eq!(vcx.debug_bounds("sidebar-pane").unwrap().size.width, full);

    // Folded, then in focus mode: ⌘\ brings the whole sidebar back.
    vcx.simulate_keystrokes("cmd-\\ cmd-shift-d");
    f.draw(&mut vcx);
    assert_eq!(vcx.debug_bounds("sidebar-pane"), None);
    vcx.simulate_keystrokes("cmd-\\");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.focus_mode));
    assert!(f.workspace(&mut vcx, |w, _| w.show_sidebar));
    assert_eq!(vcx.debug_bounds("sidebar-pane").unwrap().size.width, full);
}

/// Opening or closing the Status group is kept for the next launch.
// no-prop-test: one toggle through the settings file; the open rule is a
// property in the sidebar's tests.
#[gpui_kit::test]
fn the_status_group_remembers(cx: &mut TestAppContext) {
    let f = setup(cx);
    let sidebar = f.workspace(cx, |w, _| w.sidebar.clone());
    assert_eq!(f.saved_config().layout.status_open, None);
    let before = cx.read(|cx| sidebar.read(cx).is_status_open(cx));
    sidebar.update(cx, |sidebar, cx| sidebar.toggle_status_group(cx));
    cx.run_until_parked();
    assert_eq!(cx.read(|cx| sidebar.read(cx).is_status_open(cx)), !before);
    assert_eq!(f.saved_config().layout.status_open, Some(!before));
    let next = reopen(cx);
    let next_sidebar = cx.read(|cx| next.read(cx).sidebar.clone());
    assert_eq!(
        cx.read(|cx| next_sidebar.read(cx).is_status_open(cx)),
        !before
    );
    sidebar.update(cx, |sidebar, cx| sidebar.toggle_status_group(cx));
    assert_eq!(f.saved_config().layout.status_open, Some(before));
}

/// Opens the first note in Split in a window wide enough for both sides.
fn split_fixture(cx: &mut TestAppContext, body: &str) -> Fixture {
    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.edit_elsewhere(cx, &f.notes[0], body);
    f.select(cx, &f.notes[0]);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.simulate_resize(gpui_kit::size(gpui_kit::px(1600.), gpui_kit::px(1000.)));
    vcx.simulate_keystrokes("cmd-2");
    // The preview renders after a pause; the divider is placed once the
    // editor has been measured.
    vcx.executor().advance_clock(Duration::from_secs(1));
    for _ in 0..3 {
        f.draw(&mut vcx);
    }
    f
}

/// Dragging the divider moves it, keeps where it was left as a share of
/// the editor for next time, and neither side gets under its minimum, from
/// a drag or from a kept share.
// no-prop-test: drives the real divider with the pointer; the placement
// rules are properties in split.rs.
#[gpui_kit::test]
fn a_dragged_divider_is_kept_and_both_sides_keep_room(cx: &mut TestAppContext) {
    use crate::split::SPLIT_MIN_SIDE;
    use gpui_kit::{Modifiers, MouseButton, point, px};

    let f = split_fixture(cx, "one\ntwo");
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let pane = vcx.debug_bounds("editor-pane").unwrap();
    let width = f32::from(pane.size.width);
    let sizes = f.editor(&mut vcx, |e, cx| e.split_sizes(cx));
    assert_eq!(sizes.len(), 2, "{sizes:?}");
    assert!((sizes[0] - width / 2.).abs() <= 1., "{sizes:?} of {width}");
    assert!(
        (sizes[0] + sizes[1] - width).abs() <= 1.,
        "{sizes:?} of {width}"
    );

    let y = pane.top() + pane.size.height / 2.;
    let drag = |vcx: &mut gpui_kit::VisualTestContext, from: f32, to: f32| {
        let from = point(pane.left() + px(from), y);
        vcx.simulate_mouse_down(from, MouseButton::Left, Modifiers::none());
        // Past the drag threshold, then in steps to where it is let go.
        for step in 1..=4 {
            let x = from.x + (pane.left() + px(to) - from.x) * (step as f32 / 4.);
            vcx.simulate_mouse_move(point(x, y), MouseButton::Left, Modifiers::none());
            f.draw(vcx);
        }
        vcx.simulate_mouse_up(
            point(pane.left() + px(to), y),
            MouseButton::Left,
            Modifiers::none(),
        );
        f.draw(vcx);
    };
    let left = sizes[0];
    drag(&mut vcx, left, width * 0.3);
    let sizes = f.editor(&mut vcx, |e, cx| e.split_sizes(cx));
    assert!((sizes[0] - width * 0.3).abs() <= 2., "{sizes:?} of {width}");
    let kept = f.saved_config().layout.split_ratio;
    assert!((kept - 0.3).abs() < 0.01, "kept {kept}");
    assert!((f.editor(&mut vcx, |e, _| e.split_ratio()) - kept).abs() < 1e-6);

    // Dragged nearly to the edge, the source keeps its minimum.
    drag(&mut vcx, sizes[0], 40.);
    let sizes = f.editor(&mut vcx, |e, cx| e.split_sizes(cx));
    assert!((sizes[0] - SPLIT_MIN_SIDE).abs() <= 1., "{sizes:?}");
    let kept = f.saved_config().layout.split_ratio;
    assert!((kept * width - SPLIT_MIN_SIDE).abs() <= 2., "kept {kept}");

    // Shares kept from a wider window still leave both sides room.
    for (ratio, left) in [
        (0.02, SPLIT_MIN_SIDE),
        (0.98, width - SPLIT_MIN_SIDE),
        (0.6, width * 0.6),
    ] {
        f.update_editor(&mut vcx, |e, _, cx| e.keep_split_ratio(ratio, cx));
        f.draw(&mut vcx);
        f.draw(&mut vcx);
        let sizes = f.editor(&mut vcx, |e, cx| e.split_sizes(cx));
        assert!(
            (sizes[0] - left).abs() <= 1.,
            "{ratio}: {sizes:?} of {width}"
        );
        assert!(sizes[1] >= SPLIT_MIN_SIDE - 1., "{ratio}: {sizes:?}");
    }

    // The next launch puts the divider where it was left.
    let next = reopen(&mut vcx);
    let next_editor = vcx.read(|cx| next.read(cx).editor.clone());
    let saved = f.saved_config().layout.split_ratio;
    assert_eq!(vcx.read(|cx| next_editor.read(cx).split_ratio()), saved);
}

/// Scrolling the source scrolls the preview beside it by the same share:
/// top to top, middle to middle, end to end.
// no-prop-test: the mapping is a property in split.rs; this checks the
// views are wired to it.
#[gpui_kit::test]
fn the_preview_follows_the_source(cx: &mut TestAppContext) {
    use gpui_kit::{Modifiers, ScrollDelta, ScrollWheelEvent, TouchPhase, point, px};

    let body = (1..=300)
        .map(|n| {
            if n % 20 == 1 {
                format!("# Part {n}")
            } else {
                format!("Line {n} of the note.")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let f = split_fixture(cx, &body);
    let (top, max) = f.editor(cx, |e, cx| e.preview_scroll(cx));
    assert_eq!(top, 0.);
    assert!(max > 1000., "the preview can scroll: {max}");
    let scroll = |cx: &mut TestAppContext, share: f32| {
        f.update_editor(cx, |e, _, cx| e.scroll_source_for_test(share, cx));
        for _ in 0..2 {
            f.draw(cx);
        }
        let source = f.editor(cx, |e, cx| e.source_scroll(cx));
        let (top, max) = f.editor(cx, |e, cx| e.preview_scroll(cx));
        (source, top / max)
    };
    // A scroll set from outside the wheel, as by the scrollbar.
    let (source, share) = scroll(cx, 0.5);
    assert!(source > 0., "the source scrolled");
    assert!((share - 0.5).abs() < 0.05, "the preview is {share} down");
    let (_, share) = scroll(cx, 1.);
    assert!(share > 0.95, "the preview is {share} down");
    let (source, share) = scroll(cx, 0.);
    assert_eq!((source, share), (0., 0.));

    // The wheel over the source.
    let pane = gpui_kit::VisualTestContext::from_window(f.window, cx)
        .debug_bounds("editor-pane")
        .unwrap();
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.simulate_event(ScrollWheelEvent {
        position: pane.origin + point(px(150.), pane.size.height / 2.),
        delta: ScrollDelta::Pixels(point(px(0.), px(-600.))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
    });
    f.draw(&mut vcx);
    let source = f.editor(&mut vcx, |e, cx| e.source_scroll(cx));
    let (top, max) = f.editor(&mut vcx, |e, cx| e.preview_scroll(cx));
    assert_eq!(source, 600.);
    assert!(top > 0. && top < max / 2., "{top} of {max}");
}

/// Typing beside the preview keeps it current: the first keystroke shows
/// at once, continuous typing shows at least every 60 ms rather than
/// waiting for a pause, and once typing stops the preview is the source.
/// A long note is shown in chunks, and typing keeps all but the one typed
/// in.
// no-prop-test: drives one window over time; the chunking and the splice
// are properties in preview_chunks.rs.
#[gpui_kit::test]
fn the_preview_keeps_up_with_typing(cx: &mut TestAppContext) {
    use gpui_kit::EntityInputHandler as _;

    let body: String = (1..=150)
        .map(|n| format!("## Part {n}\n\nSome text for part {n}.\n\n- one\n- two\n\n"))
        .collect();
    assert!(
        body.len() > 4 * 1024,
        "past the preview's split on the main thread"
    );
    let f = split_fixture(cx, &body);
    let chunks = f.editor(cx, |e, cx| e.preview_chunks_for_test(cx));
    assert!(chunks >= 150, "a chunk a part or more: {chunks}");
    assert_eq!(f.editor(cx, |e, cx| e.preview_shown_for_test(cx)), body);

    let state = f.editor(cx, |e, _| e.body_state());
    f.update_editor(cx, |_, window, cx| {
        state.update(cx, |state, cx| {
            state.focus(window, cx);
            state.set_cursor_position(gpui_kit::component::input::Position::new(42, 0), window, cx);
        });
    });
    // Twenty keystrokes 25 ms apart, a fast burst.
    let mut shown = Vec::new();
    for ix in 0..20 {
        f.update_editor(cx, |_, window, cx| {
            state.update(cx, |state, cx| {
                state.replace_text_in_range(None, "x", window, cx)
            });
        });
        let now = f.editor(cx, |e, cx| e.preview_shown_for_test(cx));
        if shown.last().is_none_or(|(_, last)| *last != now) {
            shown.push((ix * 25, now));
        }
        cx.executor().advance_clock(Duration::from_millis(25));
        cx.run_until_parked();
    }
    assert_eq!(shown[0].0, 0, "the first keystroke shows at once");
    let gaps: Vec<usize> = shown.windows(2).map(|w| w[1].0 - w[0].0).collect();
    assert!(
        gaps.iter().all(|gap| *gap <= 75),
        "updates every 60 ms or so: {gaps:?}"
    );
    assert!(
        shown.len() >= 7,
        "the preview kept up: {} updates",
        shown.len()
    );

    cx.executor().advance_clock(Duration::from_millis(100));
    f.draw(cx);
    let source = f.editor(cx, |e, cx| e.body_for_test(cx));
    assert!(source.contains(&"x".repeat(20)));
    assert_eq!(f.editor(cx, |e, cx| e.preview_shown_for_test(cx)), source);
    assert_eq!(f.editor(cx, |e, cx| e.preview_chunks_for_test(cx)), chunks);
}

/// Split opened on a long note already scrolled in Edit shows the preview
/// at the same share: the preview parses a long note in the background
/// once Split shows it, and the follow tried before it has a length must
/// not leave it at the top for good.
// no-prop-test: drives one window; the mapping is a property in split.rs.
#[gpui_kit::test]
fn split_opened_on_a_scrolled_note_follows_it(cx: &mut TestAppContext) {
    let body = (1..=300)
        .map(|n| format!("Line {n} of a note long enough to parse apart."))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        body.len() > 8 * 1024,
        "past the preview's synchronous parse"
    );
    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.edit_elsewhere(cx, &f.notes[0], &body);
    f.select(cx, &f.notes[0]);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.simulate_resize(gpui_kit::size(gpui_kit::px(1600.), gpui_kit::px(1000.)));
    vcx.simulate_keystrokes("cmd-1");
    for _ in 0..2 {
        f.draw(&mut vcx);
    }
    f.update_editor(&mut vcx, |e, _, cx| e.scroll_source_for_test(0.5, cx));
    f.draw(&mut vcx);
    assert!(f.editor(&mut vcx, |e, cx| e.source_scroll(cx)) > 0.);

    vcx.simulate_keystrokes("cmd-2");
    vcx.executor().advance_clock(Duration::from_secs(1));
    for _ in 0..3 {
        f.draw(&mut vcx);
    }
    let source = f.editor(&mut vcx, |e, cx| e.source_scroll(cx));
    let mut source_max = None;
    f.update_editor(&mut vcx, |e, _, cx| {
        source_max = e.estimate_source_for_test(cx);
    });
    let source_share = source / source_max.expect("the source is laid out");
    let (top, max) = f.editor(&mut vcx, |e, cx| e.preview_scroll(cx));
    assert!(max > 1000., "the preview can scroll: {max}");
    assert!(source_share > 0.2, "the source is {source_share} down");
    assert!(
        (top / max - source_share).abs() < 0.05,
        "the preview is {} down beside a source {source_share} down",
        top / max
    );
}

/// A window too narrow for the panes as they were left gives the editor
/// its minimum back: the list narrows first, then the sidebar. In Split the
/// two sides share what is left, and the preview never runs past the
/// window, beside the sidebar or its rail. The widths come back with the
/// room.
// no-prop-test: drives one window; `PaneWidths::fitted` and
// `split_min_side` have the properties.
#[gpui_kit::test]
fn a_narrow_window_keeps_the_editor_its_room(cx: &mut TestAppContext) {
    use crate::layout::{EDITOR_MIN_WIDTH, PaneWidths};
    use gpui_kit::{px, size};

    let f = split_fixture(cx, "one\ntwo");
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let wide = PaneWidths {
        sidebar: 300.,
        list: 480.,
    };
    vcx.update(|_, cx| {
        f.workspace.update(cx, |workspace, cx| {
            workspace.pane_widths = wide;
            cx.notify();
        })
    });
    f.draw(&mut vcx);
    let check = |vcx: &mut gpui_kit::VisualTestContext, case: &str| {
        for _ in 0..3 {
            f.draw(vcx);
        }
        let width = vcx.update(|window, _| f32::from(window.viewport_size().width));
        let editor = vcx.debug_bounds("editor-pane").unwrap();
        assert!(
            f32::from(editor.right()) <= width + 0.5,
            "{case}: the editor ends at {:?} in {width}",
            editor.right()
        );
        let sizes = f.editor(vcx, |e, cx| e.split_sizes(cx));
        let room = f32::from(editor.size.width);
        assert_eq!(sizes.len(), 2, "{case}");
        assert!(
            sizes[0] + sizes[1] <= room + 1.,
            "{case}: {sizes:?} in an editor {room} wide"
        );
        // What is drawn, not only what the divider was asked for.
        let preview = vcx.debug_bounds("preview-pane").unwrap();
        assert!(
            f32::from(preview.right()) <= width + 0.5,
            "{case}: the preview ends at {:?} in {width}",
            preview.right()
        );
        room
    };
    vcx.simulate_resize(size(px(1100.), px(720.)));
    let room = check(&mut vcx, "1100 wide");
    assert!(room >= EDITOR_MIN_WIDTH - 0.5, "the editor has {room}");
    assert_eq!(
        vcx.debug_bounds("list-pane").unwrap().size.width,
        px(260.),
        "the list gave up its room first"
    );
    // The rail and back.
    vcx.simulate_keystrokes("cmd-\\");
    check(&mut vcx, "beside the rail");
    vcx.simulate_keystrokes("cmd-\\");
    let room = check(&mut vcx, "the sidebar back");
    assert!(room >= EDITOR_MIN_WIDTH - 0.5, "the editor has {room}");
    // Narrower than the panes' minimums leave room for: the two sides
    // share the editor equally.
    vcx.simulate_resize(size(px(900.), px(700.)));
    let room = check(&mut vcx, "900 wide");
    let sizes = f.editor(&mut vcx, |e, cx| e.split_sizes(cx));
    assert!((sizes[0] - room / 2.).abs() <= 1., "{sizes:?} in {room}");
    // The widths as left come back with the room.
    vcx.simulate_resize(size(px(1800.), px(1000.)));
    check(&mut vcx, "1800 wide");
    assert_eq!(f.workspace(&mut vcx, |w, _| w.pane_widths), wide);
    assert_eq!(vcx.debug_bounds("list-pane").unwrap().size.width, px(480.));
}

/// At the smallest the window gets (half of a 1366-wide screen at 125%),
/// the sidebar has folded to its rail and every pane, the note title and
/// the inspector toggle sit inside the window, at that width and the ones
/// Snap and dragging pass through on the way.
// no-prop-test: drives one window through widths; `sidebar_folds` and
// `PaneWidths::fitted` have the properties.
#[gpui_kit::test]
fn the_smallest_window_holds_the_workspace(cx: &mut TestAppContext) {
    use crate::layout::WINDOW_MIN_SIZE;
    use gpui_kit::{px, size};

    let f = split_fixture(cx, "one\ntwo");
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    for width in [WINDOW_MIN_SIZE.0, 546., 640., 683., 760., 819., 900., 1366.] {
        vcx.simulate_resize(size(px(width), px(WINDOW_MIN_SIZE.1)));
        for _ in 0..3 {
            f.draw(&mut vcx);
        }
        let sidebar = vcx.debug_bounds("sidebar-pane").unwrap();
        let list = vcx.debug_bounds("list-pane").unwrap();
        let editor = vcx.debug_bounds("editor-pane").unwrap();
        for (name, b) in [("sidebar", sidebar), ("list", list), ("editor", editor)] {
            assert!(
                f32::from(b.right()) <= width + 0.5 && f32::from(b.left()) >= -0.5,
                "{name} spans {:?}..{:?} in {width}",
                b.left(),
                b.right()
            );
        }
        assert!(list.right() <= editor.left() + px(0.5), "{width}");
        if width < crate::layout::SIDEBAR_FOLD_BELOW {
            assert!(
                f32::from(sidebar.size.width) < 100.,
                "the sidebar is {:?} wide in {width}",
                sidebar.size.width
            );
            assert!(
                f32::from(editor.size.width) >= 240.,
                "the editor is {:?} wide in {width}",
                editor.size.width
            );
        }
        if let Some(title) = vcx.debug_bounds("note-title") {
            assert!(
                f32::from(title.right()) <= f32::from(editor.right()) + 0.5,
                "{width}"
            );
        }
    }
}

/// In a window too narrow for the full sidebar, Toggle Sidebar opens it for
/// now and folds it again, never flipping the saved choice; once the window
/// is wide again, the saved choice is back.
// no-prop-test: drives one window through keystrokes; `sidebar_folds` has
// the property.
#[gpui_kit::test]
fn the_sidebar_toggle_works_in_a_narrow_window(cx: &mut TestAppContext) {
    use crate::layout::WINDOW_MIN_SIZE;
    use gpui_kit::{px, size};

    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.select(cx, &f.notes[0]);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let sidebar = f.workspace(&mut vcx, |w, _| w.sidebar.clone());
    let narrow = size(px(WINDOW_MIN_SIZE.0), px(WINDOW_MIN_SIZE.1));
    vcx.simulate_resize(narrow);
    f.draw(&mut vcx);
    assert!(vcx.read(|cx| sidebar.read(cx).is_collapsed()));

    vcx.simulate_keystrokes("cmd-\\");
    f.draw(&mut vcx);
    assert!(
        !vcx.read(|cx| sidebar.read(cx).is_collapsed()),
        "opens for now"
    );
    assert!(f.saved_config().appearance.show_sidebar);
    for name in ["sidebar-pane", "list-pane", "editor-pane"] {
        let b = vcx.debug_bounds(name).unwrap();
        assert!(f32::from(b.right()) <= WINDOW_MIN_SIZE.0 + 0.5, "{name}");
    }
    vcx.simulate_keystrokes("cmd-\\");
    f.draw(&mut vcx);
    assert!(vcx.read(|cx| sidebar.read(cx).is_collapsed()));
    assert!(
        f.saved_config().appearance.show_sidebar,
        "the saved choice stays"
    );

    // Opened by hand, then widened and narrowed again: it folds, as saved.
    vcx.simulate_keystrokes("cmd-\\");
    vcx.simulate_resize(size(px(1400.), px(800.)));
    f.draw(&mut vcx);
    assert!(!vcx.read(|cx| sidebar.read(cx).is_collapsed()));
    vcx.simulate_resize(narrow);
    f.draw(&mut vcx);
    assert!(vcx.read(|cx| sidebar.read(cx).is_collapsed()));
}

/// At the window's minimum the editor's toolbar, title and tab bar, the
/// inspector (with Ask) and Settings all stay inside the window, and an
/// open inspector with no room beside the list takes the list's place.
// no-prop-test: drives one window; the fitting rules are properties in
// layout.rs.
#[gpui_kit::test]
fn the_smallest_window_holds_the_editor_inspector_and_settings(cx: &mut TestAppContext) {
    use crate::config::InspectorTab;
    use crate::layout::{EDITOR_COMPACT_MIN, WINDOW_MIN_SIZE};
    use crate::note_menu::NoteCommand;
    use gpui_kit::{px, size};

    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    cx.update(|cx| AppState::update_config(cx, |config| config.ai.enabled = true));
    f.select(cx, &f.notes[0]);
    let workspace = f.workspace.clone();
    let id = f.notes[1].clone();
    f.window
        .update(cx, |_, window, cx| {
            workspace.update(cx, |w, cx| {
                w.run_note_command(&id, NoteCommand::OpenInNewTab, window, cx)
            })
        })
        .unwrap();
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let width = WINDOW_MIN_SIZE.0;
    vcx.simulate_resize(size(px(width), px(WINDOW_MIN_SIZE.1)));
    for _ in 0..3 {
        f.draw(&mut vcx);
    }
    let editor = vcx.debug_bounds("editor-pane").unwrap();
    let dbg = |vcx: &mut gpui_kit::VisualTestContext| {
        ["sidebar-pane", "list-pane", "editor-pane", "inspector-pane"]
            .map(|n| (n, vcx.debug_bounds(n).map(|b| (b.left(), b.right()))))
    };
    assert!(
        f32::from(editor.size.width) >= EDITOR_COMPACT_MIN - 0.5,
        "{:?}",
        dbg(&mut vcx)
    );
    for name in ["tab-bar", "note-title", "inspector-toggle"] {
        let b = vcx
            .debug_bounds(name)
            .unwrap_or_else(|| panic!("{name} is drawn"));
        assert!(
            b.right() <= editor.right() + px(0.5) && b.left() >= editor.left() - px(0.5),
            "{name} spans {:?}..{:?}, the editor {:?}..{:?}",
            b.left(),
            b.right(),
            editor.left(),
            editor.right()
        );
    }

    // Ask opens in the inspector, which takes the list's place.
    workspace.update(&mut vcx, |w, cx| {
        w.show_inspector(Some(InspectorTab::Ask), cx)
    });
    for _ in 0..3 {
        f.draw(&mut vcx);
    }
    let inspector = vcx
        .debug_bounds("inspector-pane")
        .expect("the inspector shows in a narrow window");
    assert_eq!(vcx.debug_bounds("list-pane"), None);
    assert!(f32::from(inspector.right()) <= width + 0.5);
    let editor = vcx.debug_bounds("editor-pane").unwrap();
    assert!(editor.right() <= inspector.left() + px(0.5));
    assert!(f32::from(editor.size.width) >= EDITOR_COMPACT_MIN - 0.5);
    workspace.update(&mut vcx, |w, cx| w.toggle_inspector(cx));
    f.draw(&mut vcx);
    assert!(vcx.debug_bounds("list-pane").is_some());

    // Off macOS the menus in Kanso's own title bar fit beside the window's
    // three buttons (34 points each).
    workspace.update(&mut vcx, |w, cx| {
        crate::workspace::set_menus(cx);
        crate::title_bar::publish_menus(cx);
        w.title_bar = Some(gpui_kit::component::menu::AppMenuBar::new(cx));
    });
    for _ in 0..3 {
        f.draw(&mut vcx);
    }
    let menus = vcx
        .debug_bounds("title-menus")
        .expect("the menus are drawn");
    assert!(
        f32::from(menus.right()) + 3. * 34. <= width,
        "the menus end at {:?}",
        menus.right()
    );

    // Settings fill the window beside the rail.
    vcx.simulate_keystrokes("cmd-,");
    for _ in 0..3 {
        f.draw(&mut vcx);
    }
    let settings = vcx.debug_bounds("settings-pane").expect("Settings open");
    assert!(f32::from(settings.right()) <= width + 0.5);
    assert!(f32::from(settings.size.width) >= width - 100.);
}

/// Another note opens at its top in the preview too, whether the preview
/// was scrolled alone (Preview) or beside a source left at its top (Split).
// no-prop-test: drives one window; the scroll mapping is a property in
// split.rs.
#[gpui_kit::test]
fn another_note_opens_at_the_top_of_its_preview(cx: &mut TestAppContext) {
    let body = (1..=300)
        .map(|n| format!("Line {n} of the note."))
        .collect::<Vec<_>>()
        .join("\n");
    let f = split_fixture(cx, &body);
    f.edit_elsewhere(cx, &f.notes[1], &body);
    for (mode, keys) in [("Split", "cmd-2"), ("Preview", "cmd-3")] {
        let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
        vcx.simulate_keystrokes(keys);
        f.select(&mut vcx, &f.notes[0]);
        for _ in 0..2 {
            f.draw(&mut vcx);
        }
        // Only the preview scrolled; the source stays at its top.
        f.update_editor(&mut vcx, |e, _, cx| e.scroll_preview_for_test(800., cx));
        f.draw(&mut vcx);
        assert_eq!(
            f.editor(&mut vcx, |e, cx| e.preview_scroll(cx).0),
            800.,
            "{mode}"
        );
        f.select(&mut vcx, &f.notes[1]);
        for _ in 0..2 {
            f.draw(&mut vcx);
        }
        assert_eq!(
            f.editor(&mut vcx, |e, cx| e.preview_scroll(cx).0),
            0.,
            "{mode}"
        );
        assert_eq!(
            f.editor(&mut vcx, |e, cx| e.source_scroll(cx)),
            0.,
            "{mode}"
        );
    }
}

/// The lines the preview's scroll range is estimated from are the open
/// text's, even after another text of the same length replaced it: by
/// opening another note, or by a reload from elsewhere.
// no-prop-test: two fixed texts of one length are the case that broke.
#[gpui_kit::test]
fn the_source_lines_follow_a_text_of_the_same_length(cx: &mut TestAppContext) {
    let f = split_fixture(cx, "aaaa\nbbbb");
    f.edit_elsewhere(cx, &f.notes[1], "abcdefgh\n");
    let lines = |cx: &mut TestAppContext| {
        f.update_editor(cx, |e, _, cx| {
            e.estimate_source_for_test(cx);
        });
        f.editor(cx, |e, _| e.source_lines_for_test())
    };
    assert_eq!(lines(cx), Some((9, vec![4, 4])));
    f.select(cx, &f.notes[1]);
    assert_eq!(lines(cx), Some((9, vec![8, 0])));
    // Reloaded from elsewhere, still nine bytes.
    f.edit_elsewhere(cx, &f.notes[1], "a\nb\nc\nd\ne");
    assert_eq!(f.editor(cx, |e, cx| e.body_for_test(cx)), "a\nb\nc\nd\ne");
    assert_eq!(lines(cx), Some((9, vec![1, 1, 1, 1, 1])));
}

/// The title reads cut short with an ellipsis until it is clicked, and is
/// then edited in place.
// no-prop-test: one click on the real view.
#[gpui_kit::test]
fn the_title_is_cut_short_until_it_is_edited(cx: &mut TestAppContext) {
    use gpui_kit::Modifiers;

    let f = setup(cx);
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let title = vcx.debug_bounds("note-title").expect("the title as text");
    let editor = f.workspace(&mut vcx, |w, _| w.editor.clone());
    assert!(!vcx.update(|window, cx| editor.read(cx).title_focused(window, cx)));
    vcx.simulate_click(title.center(), Modifiers::none());
    f.draw(&mut vcx);
    assert!(vcx.update(|window, cx| editor.read(cx).title_focused(window, cx)));
    assert_eq!(vcx.debug_bounds("note-title"), None, "the field instead");
    vcx.simulate_input("!");
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    assert_eq!(f.store.note(&f.notes[0]).unwrap().unwrap().title, "a!");
}

/// The rail's tags button lists the tags, and the one chosen is what the
/// list shows.
// no-prop-test: one event through the palette.
#[gpui_kit::test]
fn the_rail_goes_to_a_tag(cx: &mut TestAppContext) {
    use crate::commands::Picker;
    use crate::library::Filter;
    use crate::palette::Mode;
    use crate::sidebar::SidebarEvent;

    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    let tag = f
        .store
        .put(
            DocType::Tag,
            json!({"name": "mcp"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    f.store
        .put(
            DocType::Note,
            json!({"_id": f.notes[1], "tags": [tag]}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    f.draw(cx);
    let (sidebar, palette, library) = f.workspace(cx, |w, _| {
        (w.sidebar.clone(), w.palette.clone(), w.library.clone())
    });
    sidebar.update(cx, |_, cx| cx.emit(SidebarEvent::GoToTag));
    f.draw(cx);
    assert_eq!(
        cx.read(|cx| palette.read(cx).mode()),
        Some(Mode::Pick(Picker::GoToTag))
    );
    assert_eq!(cx.read(|cx| palette.read(cx).item_labels()), ["mcp"]);
    cx.simulate_keystrokes(f.window, "enter");
    f.draw(cx);
    assert_eq!(
        cx.read(|cx| library.read(cx).filter.clone()),
        Filter::Tag(tag)
    );
    assert_eq!(cx.read(|cx| palette.read(cx).mode()), None);
}

/// While a Replace holds writes (A10), typing waits unsaved and the editor
/// stays on its note; afterwards the edits are saved, unless the Replace
/// took the note away: then the editor asks about them rather than write
/// the note into the new library.
#[gpui_kit::test]
fn edits_wait_while_a_replace_holds_writes(cx: &mut TestAppContext) {
    let f = setup(cx);
    let [first, second] = f.notes.clone();
    f.select(cx, &first);
    f.type_body(cx, "before the hold");
    let (api, restart) = cx.update(crate::services::hold_writes);
    assert!(api.is_none() && !restart, "no API ran here");
    assert_eq!(
        note_body(&f.store, &first),
        "before the hold",
        "saved first"
    );

    f.type_body(cx, "during the hold");
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(5));
    cx.run_until_parked();
    assert_eq!(
        note_body(&f.store, &first),
        "before the hold",
        "the autosave waits"
    );
    f.select(cx, &second);
    let open = f.editor(cx, |editor, _| editor.note_id().map(str::to_owned));
    assert_eq!(open.as_deref(), Some(first.as_str()), "the editor stays");

    cx.update(|cx| crate::services::release_writes(false, cx));
    cx.run_until_parked();
    assert_eq!(note_body(&f.store, &first), "during the hold");

    // The Replace empties the library while the edit waits.
    cx.update(crate::services::hold_writes);
    f.type_body(cx, "the note goes");
    f.store.replace_library(&[]).unwrap();
    cx.run_until_parked();
    cx.update(|cx| crate::services::release_writes(false, cx));
    cx.run_until_parked();
    assert!(f.store.note(&first).unwrap().is_none(), "not written back");
    assert!(f.editor(cx, |editor, _| editor.has_incoming()), "it asks");
    assert_eq!(
        f.editor(cx, |editor, cx| editor.body_for_test(cx)),
        "the note goes",
        "with the edit on screen"
    );
}

/// A new note made while a tag shows gets that tag; one made while a level
/// of the tag tree shows gets the level's own tag, made if only tags inside
/// it existed.
#[gpui_kit::test]
fn new_notes_under_a_tag_get_it(cx: &mut TestAppContext) {
    use crate::library::Filter;
    let f = setup(cx);
    let acme = f.store.tag_named("work/acme", Origin::Ui).unwrap();
    let library = f.workspace(cx, |w, _| w.library.clone());
    let made = |filter: Filter, cx: &mut TestAppContext| {
        library.update(cx, |library, cx| library.set_filter(filter, cx));
        f.draw(cx);
        cx.dispatch_action(f.window, crate::workspace::NewNote);
        cx.run_until_parked();
        let id = cx.read(|cx| library.read(cx).selected.clone()).unwrap();
        f.store.note(&id).unwrap().unwrap().tags
    };
    assert_eq!(
        made(Filter::Tag(acme.clone()), cx),
        std::slice::from_ref(&acme)
    );
    let work = made(Filter::TagPath("Work".into()), cx);
    assert_eq!(work.len(), 1);
    assert_ne!(work[0], acme);
    assert_eq!(f.store.tag_named("work", Origin::Ui).unwrap(), work[0]);
    // Again: the same tag, not another.
    assert_eq!(made(Filter::TagPath("work".into()), cx), work);
}

/// The board opens over the window from its action, a card moves a column
/// with ⇧→ (the note's status changes in the store, the open note's
/// through the editor), Return opens the card's note and Escape closes it.
// no-prop-test: drives one window through keystrokes; the moves' rules
// are properties in board.rs.
#[gpui_kit::test]
fn the_board_moves_cards_with_the_keyboard(cx: &mut TestAppContext) {
    use kanso_core::NoteStatus;
    let f = setup(cx);
    cx.update(|cx| {
        crate::workspace::bind_keys(cx);
        crate::board_view::bind_keys(cx);
    });
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    cx.dispatch_action(f.window, crate::board_view::ShowBoard);
    f.draw(cx);
    assert!(f.workspace(cx, |w, _| w.board_open));
    let board = f.workspace(cx, |w, _| w.board.clone().unwrap());
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let status = |id: &str| f.store.note(id).unwrap().unwrap().status;

    // Both notes start with no status: the first column.
    vcx.simulate_keystrokes("down");
    let first = vcx.read(|cx| {
        board
            .read(cx)
            .board
            .selected_card()
            .map(|c| c.note_id().to_owned())
    });
    let first = first.unwrap();
    vcx.simulate_keystrokes("shift-right shift-right");
    f.draw(&mut vcx);
    assert_eq!(status(&first), NoteStatus::OnHold);
    vcx.simulate_keystrokes("shift-left");
    f.draw(&mut vcx);
    assert_eq!(status(&first), NoteStatus::Active);
    // The open note's body is kept as typed.
    let other = if first == f.notes[0] {
        &f.notes[1]
    } else {
        &f.notes[0]
    };
    assert_eq!(
        f.store.note(other).unwrap().unwrap().status,
        NoteStatus::None
    );

    vcx.simulate_keystrokes("enter");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.board_open));
    assert_eq!(f.open_note(&mut vcx).as_deref(), Some(first.as_str()));

    vcx.dispatch_action(crate::board_view::ShowTaskBoard);
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.board_open));
    vcx.simulate_keystrokes("escape");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.board_open));
}

/// The Journal's calendar opens on today's month, the keyboard moves the
/// day and the month, and Return opens the day's note: made in the Journal
/// when missing, with the calendar closed; a day with a note opens it.
// no-prop-test: drives one window through keystrokes; the month grid's
// rules are properties in calendar.rs.
#[gpui_kit::test]
fn the_calendar_opens_and_makes_a_days_note(cx: &mut TestAppContext) {
    use gpui_kit::component::WindowExt as _;
    let f = setup(cx);
    cx.update(|cx| {
        crate::workspace::bind_keys(cx);
        crate::calendar_view::bind_keys(cx);
    });
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.draw(cx);
    let view = f
        .window
        .update(cx, |_, window, cx| {
            f.workspace.update(cx, |workspace, cx| {
                crate::calendar_view::open_calendar(workspace, window, cx)
            })
        })
        .unwrap();
    f.draw(cx);
    let today = cx.read(|cx| crate::journal::now(cx).date);
    let (focused, month) = cx.read(|cx| {
        let v = view.read(cx);
        (v.focused, (v.year, v.month))
    });
    assert_eq!(focused, today);
    assert_eq!(month, (today.year, today.month));

    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.simulate_keystrokes("pageup left up");
    let picked = vcx.read(|cx| view.read(cx).focused);
    let expected = crate::calendar::add_months(today, -1).add_days(-8);
    assert_eq!(picked, expected);
    vcx.simulate_keystrokes("t");
    assert_eq!(vcx.read(|cx| view.read(cx).focused), today);
    vcx.simulate_keystrokes("left enter");
    f.draw(&mut vcx);
    let yesterday = today.add_days(-1);
    let id = kanso_core::roles::daily_id(yesterday);
    assert_eq!(f.open_note(&mut vcx).as_deref(), Some(id.as_str()));
    assert!(f.store.note(&id).unwrap().is_some(), "made");
    let open = vcx.update(|window, cx| window.has_active_dialog(cx));
    assert!(!open, "the calendar closed");

    // Opened again from the daily note, it is on that note's month and
    // marks the day.
    let view = f
        .window
        .update(&mut vcx, |_, window, cx| {
            f.workspace.update(cx, |workspace, cx| {
                crate::calendar_view::open_calendar(workspace, window, cx)
            })
        })
        .unwrap();
    let (focused, marked) = vcx.read(|cx| {
        let v = view.read(cx);
        (v.focused, v.days.contains(&yesterday))
    });
    assert_eq!(focused, yesterday);
    assert!(marked);
}

/// ⌘= and ⌘+ zoom the whole app in a step, ⌘− out, and ⌘0 back to actual
/// size, kept in the settings and stopping at either end. While the graph
/// has focus the same keys zoom the graph, and the app stays as it is.
// no-prop-test: drives one window through keystrokes; `zoom` has the
// properties of the steps.
#[gpui_kit::test]
fn zoom_keys_zoom_the_app_and_the_graph_keeps_them(cx: &mut TestAppContext) {
    use crate::zoom;
    let f = setup(cx);
    cx.update(|cx| {
        crate::workspace::bind_keys(cx);
        crate::graph_view::bind_keys(cx);
        zoom::register(cx);
    });
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let saved = |vcx: &mut gpui_kit::VisualTestContext| {
        vcx.run_until_parked();
        (
            vcx.read(|cx| cx.global::<AppState>().config.appearance.zoom),
            f.saved_config().appearance.zoom,
        )
    };
    let width = vcx.debug_bounds("sidebar-pane").unwrap().size.width;

    vcx.simulate_keystrokes("cmd-=");
    assert_eq!(saved(&mut vcx), (110, 110));
    assert_eq!(zoom::scale(), 1.1);
    vcx.simulate_keystrokes("cmd-+");
    assert_eq!(saved(&mut vcx), (120, 120));
    f.draw(&mut vcx);
    let zoomed = vcx.debug_bounds("sidebar-pane").unwrap().size.width;
    assert!(
        (f32::from(zoomed) - f32::from(width) * 1.2).abs() < 1.,
        "the sidebar is drawn larger: {width:?} to {zoomed:?}"
    );
    assert_eq!(
        vcx.read(|cx| gpui_kit::component::Theme::global(cx).font_size),
        gpui_kit::px(16. * 1.2),
        "the components' rems follow"
    );

    vcx.simulate_keystrokes("cmd--");
    vcx.simulate_keystrokes("cmd--");
    vcx.simulate_keystrokes("cmd--");
    assert_eq!(saved(&mut vcx), (90, 90));
    for _ in 0..5 {
        vcx.simulate_keystrokes("cmd--");
    }
    assert_eq!(saved(&mut vcx), (80, 80), "stops at the smallest");
    vcx.simulate_keystrokes("cmd-0");
    assert_eq!(saved(&mut vcx), (100, 100));
    assert_eq!(zoom::scale(), 1.);

    // The graph, with focus, takes the keys for its own zoom.
    let workspace = f.workspace.clone();
    vcx.update(|window, cx| workspace.update(cx, |workspace, cx| workspace.show_graph(window, cx)));
    f.draw(&mut vcx);
    let graph = f.workspace(&mut vcx, |w, _| w.graph.clone()).unwrap();
    let before = vcx.read(|cx| graph.read(cx).zoom_for_test());
    vcx.simulate_keystrokes("cmd-=");
    vcx.run_until_parked();
    let after = vcx.read(|cx| graph.read(cx).zoom_for_test());
    assert!(after > before, "the graph zoomed in: {before} to {after}");
    assert_eq!(saved(&mut vcx), (100, 100), "the app didn't");
}

/// "Change Color…" opens the dialog with the keyboard in its hex field:
/// typed hex and Enter color the notebook, kept on it so it syncs; an
/// emptied field takes the color away; a swatch and Done color it again.
#[gpui_kit::test]
fn a_notebook_takes_a_color_from_the_dialog(cx: &mut TestAppContext) {
    use gpui_kit::component::WindowExt as _;
    use gpui_kit::{Modifiers, VisualTestContext};

    use crate::sidebar::SidebarEvent;

    let f = setup(cx);
    let book = f.store.note(&f.notes[0]).unwrap().unwrap().book_id;
    let (sidebar, library) = f.workspace(cx, |w, _| (w.sidebar.clone(), w.library.clone()));
    let kept = |store: &Store| {
        store
            .get(&book)
            .unwrap()
            .unwrap()
            .to_api(Default::default())
            .get(kanso_core::model::BOOK_COLOR_FIELD)
            .cloned()
    };
    let shown = |cx: &mut TestAppContext| {
        cx.read(|cx| {
            let library = library.read(cx);
            let book = library.books.iter().find(|b| b.id == book).unwrap();
            crate::book_colors::book_color(book)
        })
    };
    let open = |cx: &mut TestAppContext| {
        let id = book.clone();
        sidebar.update(cx, |_, cx| cx.emit(SidebarEvent::ChangeBookColor(id)));
        f.draw(cx);
        assert!(
            f.window
                .update(cx, |_, window, cx| window.has_active_dialog(cx))
                .unwrap()
        );
    };

    open(cx);
    cx.simulate_input(f.window, "#f8f8f8");
    cx.simulate_keystrokes(f.window, "enter");
    f.draw(cx);
    assert_eq!(kept(&f.store), Some(json!("#f8f8f8")));
    assert_eq!(shown(cx), Some(0xf8f8f8));

    // The field opens holding the color; emptied, Enter takes it away.
    open(cx);
    cx.simulate_keystrokes(f.window, "cmd-a backspace enter");
    f.draw(cx);
    assert_eq!(kept(&f.store), None);
    assert_eq!(shown(cx), None);

    open(cx);
    // Clicks land where the dialog ends up, once it has slid in.
    std::thread::sleep(
        *gpui_kit::component::dialog::ANIMATION_DURATION + Duration::from_millis(50),
    );
    for selector in ["book-color-blue", "book-color-done"] {
        f.draw(cx);
        let mut vcx = VisualTestContext::from_window(f.window, cx);
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        vcx.simulate_click(bounds.center(), Modifiers::none());
        cx.run_until_parked();
    }
    f.draw(cx);
    assert_eq!(kept(&f.store), Some(json!("#2185d0")));
    assert_eq!(shown(cx), Some(kanso_core::TagColor::Blue.rgb()));

    // Done on the color it already keeps writes nothing.
    let rev = f
        .store
        .get(&book)
        .unwrap()
        .unwrap()
        .to_api(Default::default())["_rev"]
        .clone();
    assert!(rev.is_string(), "{rev}");
    open(cx);
    cx.simulate_keystrokes(f.window, "enter");
    f.draw(cx);
    let after = f
        .store
        .get(&book)
        .unwrap()
        .unwrap()
        .to_api(Default::default())["_rev"]
        .clone();
    assert_eq!(after, rev);
    assert_eq!(kept(&f.store), Some(json!("#2185d0")));
}

#[gpui_kit::test]
fn the_inspector_lists_and_edits_properties(cx: &mut TestAppContext) {
    use crate::config::InspectorTab;

    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.type_body(
        cx,
        "---\nclient: Acme\nrating: 4\ndone: false\n---\n# Note\n\nBody\n",
    );
    let inspector = f.workspace(cx, |workspace, _| workspace.inspector.clone());
    f.window
        .update(cx, |_, _, cx| {
            f.workspace.update(cx, |workspace, cx| {
                workspace.show_inspector(Some(InspectorTab::Outline), cx)
            })
        })
        .unwrap();
    f.draw(cx);

    {
        let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
        assert!(vcx.debug_bounds("inspector-properties").is_some());
        assert!(vcx.debug_bounds("property-client").is_some());
        assert!(vcx.debug_bounds("property-rating").is_some());
        assert!(vcx.debug_bounds("property-done").is_some());
        assert!(vcx.debug_bounds("add-property").is_some());
    }

    let properties = cx.read(|cx| inspector.read(cx).properties_for_test(cx));
    assert_eq!(
        properties
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        ["client", "rating", "done"]
    );

    f.window
        .update(cx, |_, window, cx| {
            inspector.update(cx, |inspector, cx| {
                inspector.set_property_for_test("client", "Globex", window, cx)
            })
        })
        .unwrap();
    let body = f.editor(cx, |editor, cx| editor.body_for_test(cx));
    assert!(body.contains("client: Globex\n"), "{body}");
    assert!(body.ends_with("# Note\n\nBody\n"), "{body}");

    f.window
        .update(cx, |_, window, cx| {
            inspector.update(cx, |inspector, cx| {
                inspector.add_property_for_test("due", "2026-11-01", window, cx)
            })
        })
        .unwrap();
    let body = f.editor(cx, |editor, cx| editor.body_for_test(cx));
    assert!(body.contains("due: 2026-11-01\n"), "{body}");
    assert!(body.ends_with("# Note\n\nBody\n"), "{body}");
}

#[gpui_kit::test]
fn a_property_is_only_written_when_it_changes(cx: &mut TestAppContext) {
    use crate::config::InspectorTab;

    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    let body = "---\nnested:\n  a: 1\n  b: 2\nzip: 01234\nversion: 1.10\nclient: Acme # keep\n---\n# Note\n";
    f.type_body(cx, body);
    let inspector = f.workspace(cx, |workspace, _| workspace.inspector.clone());
    f.window
        .update(cx, |_, _, cx| {
            f.workspace.update(cx, |workspace, cx| {
                workspace.show_inspector(Some(InspectorTab::Outline), cx)
            })
        })
        .unwrap();
    f.draw(cx);

    {
        let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
        assert!(vcx.debug_bounds("property-nested").is_some());
    }
    for (key, text) in [
        ("nested", "  a: 1\n  b: 2\n"),
        ("zip", "1234"),
        ("version", "1.1"),
    ] {
        f.window
            .update(cx, |_, window, cx| {
                inspector.update(cx, |inspector, cx| {
                    inspector.set_property_for_test(key, text, window, cx)
                })
            })
            .unwrap();
        let after = f.editor(cx, |editor, cx| editor.body_for_test(cx));
        assert_eq!(after, body, "{key}");
    }

    f.window
        .update(cx, |_, window, cx| {
            inspector.update(cx, |inspector, cx| {
                inspector.set_property_for_test("client", "Globex", window, cx)
            })
        })
        .unwrap();
    let after = f.editor(cx, |editor, cx| editor.body_for_test(cx));
    assert!(after.contains("client: Globex # keep\n"), "{after}");
}

#[gpui_kit::test]
fn a_duplicate_property_edits_the_value_shown(cx: &mut TestAppContext) {
    use crate::config::InspectorTab;

    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.type_body(cx, "---\na: 1\na: 2\n---\n# Note\n");
    let inspector = f.workspace(cx, |workspace, _| workspace.inspector.clone());
    f.window
        .update(cx, |_, _, cx| {
            f.workspace.update(cx, |workspace, cx| {
                workspace.show_inspector(Some(InspectorTab::Outline), cx)
            })
        })
        .unwrap();
    f.draw(cx);
    let properties = cx.read(|cx| inspector.read(cx).properties_for_test(cx));
    assert_eq!(
        properties
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        ["a", "a"]
    );
    // The first row edits the first `a`, not the last one `get` reads.
    f.window
        .update(cx, |_, window, cx| {
            inspector.update(cx, |inspector, cx| {
                inspector.set_property_at_for_test(0, "3", window, cx)
            })
        })
        .unwrap();
    let body = f.editor(cx, |editor, cx| editor.body_for_test(cx));
    assert!(body.contains("a: 3\na: 2\n"), "{body}");

    // And the second row edits the second `a`.
    f.window
        .update(cx, |_, window, cx| {
            inspector.update(cx, |inspector, cx| {
                inspector.set_property_at_for_test(1, "4", window, cx)
            })
        })
        .unwrap();
    let body = f.editor(cx, |editor, cx| editor.body_for_test(cx));
    assert!(body.contains("a: 3\na: 4\n"), "{body}");
}

#[gpui_kit::test]
fn a_property_edit_is_dropped_when_the_row_changed(cx: &mut TestAppContext) {
    use crate::config::InspectorTab;

    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.type_body(cx, "---\na: 1\nb: 2\n---\n# Note\n");
    let inspector = f.workspace(cx, |workspace, _| workspace.inspector.clone());
    f.window
        .update(cx, |_, _, cx| {
            f.workspace.update(cx, |workspace, cx| {
                workspace.show_inspector(Some(InspectorTab::Outline), cx)
            })
        })
        .unwrap();
    f.draw(cx);

    // A line typed above the open field moves the row: Enter must not write
    // the value onto whichever property is now first.
    f.window
        .update(cx, |_, window, cx| {
            inspector.update(cx, |inspector, cx| {
                inspector.open_property_for_test(0, window, cx)
            })
        })
        .unwrap();
    f.type_body(cx, "---\nx: 0\na: 1\nb: 2\n---\n# Note\n");
    let before = f.editor(cx, |editor, cx| editor.body_for_test(cx));
    f.window
        .update(cx, |_, window, cx| {
            inspector.update(cx, |inspector, cx| {
                inspector.commit_open_property_for_test("9", window, cx)
            })
        })
        .unwrap();
    let after = f.editor(cx, |editor, cx| editor.body_for_test(cx));
    assert_eq!(after, before);

    // A field opened on the row as it stands still writes it.
    f.window
        .update(cx, |_, window, cx| {
            inspector.update(cx, |inspector, cx| {
                inspector.open_property_for_test(0, window, cx);
                inspector.commit_open_property_for_test("7", window, cx);
            })
        })
        .unwrap();
    let body = f.editor(cx, |editor, cx| editor.body_for_test(cx));
    assert!(body.contains("x: 7\n"), "{body}");
}

#[gpui_kit::test]
fn adding_a_raw_property_is_refused(cx: &mut TestAppContext) {
    use crate::config::InspectorTab;

    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.type_body(cx, "---\nnested:\n  a: 1\n---\n# Note\n");
    let inspector = f.workspace(cx, |workspace, _| workspace.inspector.clone());
    f.window
        .update(cx, |_, _, cx| {
            f.workspace.update(cx, |workspace, cx| {
                workspace.show_inspector(Some(InspectorTab::Outline), cx)
            })
        })
        .unwrap();
    f.draw(cx);
    let before = f.editor(cx, |editor, cx| editor.body_for_test(cx));
    f.window
        .update(cx, |_, window, cx| {
            inspector.update(cx, |inspector, cx| {
                inspector.add_property_for_test("nested", "raw text", window, cx)
            })
        })
        .unwrap();
    let after = f.editor(cx, |editor, cx| editor.body_for_test(cx));
    assert_eq!(after, before);
}

#[gpui_kit::test]
fn a_locked_note_shows_no_properties(cx: &mut TestAppContext) {
    use crate::config::InspectorTab;

    let f = setup(cx);
    let book = f.store.books().unwrap()[0].id.clone();
    let id = f
        .store
        .put(
            DocType::Note,
            json!({"title": "Sealed", "body": "", "bookId": book,
                   "kansoLock": {"salt": "s", "nonce": "n", "data": "d"}}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    f.select(cx, &id);
    let inspector = f.workspace(cx, |workspace, _| workspace.inspector.clone());
    f.window
        .update(cx, |_, _, cx| {
            f.workspace.update(cx, |workspace, cx| {
                workspace.show_inspector(Some(InspectorTab::Outline), cx)
            })
        })
        .unwrap();
    f.draw(cx);
    let properties = cx.read(|cx| inspector.read(cx).properties_for_test(cx));
    assert!(properties.is_empty());
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    assert_eq!(vcx.debug_bounds("inspector-properties"), None);
}

#[gpui_kit::test]
fn a_query_view_draws_a_table_in_the_preview(cx: &mut TestAppContext) {
    use crate::config::ViewMode;

    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.type_body(
        cx,
        "---\nclient: Acme\n---\n\n```query\nfrom: has:client\ncolumns: client\nsort: client asc\n```\n",
    );
    let other = f.notes[1].clone();
    f.edit_elsewhere(cx, &other, "---\nclient: Beta\n---\n# Other\n");
    f.window
        .update(cx, |_, _, cx| {
            f.workspace.update(cx, |workspace, cx| {
                workspace.set_view_mode_for_test(ViewMode::Preview, cx)
            })
        })
        .unwrap();
    // The notes are read on the background executor, then the preview draws
    // the table.
    for _ in 0..8 {
        f.draw(cx);
        cx.run_until_parked();
    }
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    assert!(vcx.debug_bounds("kanso-query").is_some());
    assert!(vcx.debug_bounds("kanso-query-row-0").is_some());
    assert!(vcx.debug_bounds("kanso-query-row-1").is_none());

    // A note added elsewhere makes the table out of date. After the burst of
    // saves settles, the table shows it without being reopened.
    let book = f.store.note(&f.notes[0]).unwrap().unwrap().book_id;
    f.store
        .put(
            DocType::Note,
            json!({"title": "Gamma", "body": "---\nclient: Gamma\n---\n# Gamma\n",
                   "bookId": book}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();
    cx.run_until_parked();
    f.draw(cx);
    // The debounce is wall-clock; let it pass, then fire its wake timer.
    std::thread::sleep(Duration::from_millis(350));
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    for _ in 0..8 {
        f.draw(cx);
        cx.run_until_parked();
    }
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    assert!(vcx.debug_bounds("kanso-query-row-1").is_some());
}

fn open_test_window(cx: &mut gpui_kit::App) {
    gpui_kit::open_window(Default::default(), cx, |window, cx| {
        cx.new(|cx| Workspace::new(window, cx))
    })
    .unwrap();
}

/// What the main window's editor shows, and that note's text.
fn shown_note(cx: &mut TestAppContext) -> (Option<String>, String) {
    cx.update(|cx| {
        let (window, workspace) = crate::workspace::main_workspace(cx).expect("a main window");
        let shown = window
            .update(cx, |_, _, cx| {
                workspace
                    .read(cx)
                    .editor
                    .read(cx)
                    .note_id()
                    .map(str::to_owned)
            })
            .unwrap();
        let body = shown
            .as_deref()
            .map(|id| note_body(&cx.global::<AppState>().store, id))
            .unwrap_or_default();
        (shown, body)
    })
}

fn start_without_window(
    cx: &mut TestAppContext,
    dir: &tempfile::TempDir,
    store: &Arc<Store>,
    with_window: bool,
    receiver: async_channel::Receiver<Vec<std::path::PathBuf>>,
) {
    let (data_dir, app_store) = (dir.path().to_path_buf(), store.clone());
    cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store: app_store,
            data_dir,
            config: AppConfig::default(),
        });
        cx.set_global(Services::default());
        if with_window {
            open_test_window(cx);
        }
        crate::open_documents::serve(receiver, open_test_window, cx);
    });
}

/// A launch that carries a file (Finder's Open With starts Kanso, and the
/// file is queued before any window exists), then a file opened while
/// running: each ends with that file's note shown, and no note is made
/// that nobody wrote.
#[gpui_kit::test]
fn files_opened_at_launch_and_later_are_shown_never_a_new_note(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let first = dir.path().join("first.md");
    let second = dir.path().join("second.md");
    std::fs::write(&first, "# First\n\nfrom launch").unwrap();
    std::fs::write(&second, "# Second\n\nwhile running").unwrap();
    // The system's event arrives before the app has a window.
    let (opener, receiver) = crate::open_documents::channel();
    opener.urls(&[url::Url::from_file_path(&first).unwrap().to_string()]);
    start_without_window(cx, &dir, &store, true, receiver);
    cx.run_until_parked();
    let (shown, body) = shown_note(cx);
    assert!(shown.is_some(), "the launch file's note is shown");
    assert!(body.contains("from launch"));
    let count = |store: &Store| store.counts().unwrap()[&DocType::Note];
    assert_eq!(count(&store), 1, "no empty note beside it");

    opener.paths(vec![second]);
    cx.run_until_parked();
    let (shown_now, body) = shown_note(cx);
    assert_ne!(shown_now, shown);
    assert!(body.contains("while running"));
    assert_eq!(count(&store), 2);

    // Opening the first again selects its note, not a copy.
    opener.paths(vec![first]);
    cx.run_until_parked();
    assert_eq!(shown_note(cx).0, shown);
    assert_eq!(count(&store), 2);
}

/// The main window was closed: a file opens it again.
#[gpui_kit::test]
fn a_file_opened_with_no_window_opens_the_window(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let file = dir.path().join("late.md");
    std::fs::write(&file, "# Late\n\nbody").unwrap();
    let (opener, receiver) = crate::open_documents::channel();
    start_without_window(cx, &dir, &store, false, receiver);
    opener.paths(vec![file]);
    cx.run_until_parked();
    assert!(shown_note(cx).1.contains("body"));
    assert_eq!(store.counts().unwrap()[&DocType::Note], 1);
}

/// Live mode hides the Markdown marks off the caret (a heading's `#` off
/// its line, an inline mark off its element), draws headings at the
/// preview's sizes, and reveals marks once the caret comes to them; another
/// mode draws the plain source again.
#[gpui_kit::test]
fn live_mode_hides_the_marks_off_the_caret_line(cx: &mut TestAppContext) {
    use crate::config::ViewMode;
    let heading_scale = |level| kanso_render::typography::heading(level).scale;

    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.window
        .update(cx, |_, _, cx| {
            f.workspace.update(cx, |workspace, cx| {
                workspace.set_view_mode_for_test(ViewMode::Live, cx)
            })
        })
        .unwrap();
    let text = "# Title\n\nSome **bold** text\n";
    let stars = text.find("**").unwrap();
    f.type_body(cx, text);
    f.draw(cx);
    // The caret starts on the heading: its `# ` shows, the stars don't.
    let (shown, live) = f.editor(cx, |e, cx| e.live_for_test(cx));
    assert!(live);
    assert_eq!(shown.line_scales, vec![(0, heading_scale(1))]);
    assert!(shown.concealed.contains(&(stars..stars + 2)), "{shown:?}");
    assert!(
        shown.concealed.iter().all(|range| range.start >= 8),
        "{shown:?}"
    );

    // In the bold text its stars show, and the heading's `# ` hides.
    f.update_editor(cx, |editor, _, cx| {
        editor.select_for_test(stars + 4..stars + 4, cx)
    });
    f.draw(cx);
    let (shown, _) = f.editor(cx, |e, cx| e.live_for_test(cx));
    assert!(shown.concealed.contains(&(0..2)), "{shown:?}");
    assert!(
        shown.concealed.iter().all(|range| range.end <= 8),
        "{shown:?}"
    );
    // The heading keeps its size whichever line the caret is on.
    assert_eq!(shown.line_scales, vec![(0, heading_scale(1))]);

    // Typed text is drawn at once: a new heading below is larger too.
    f.update_editor(cx, |editor, window, cx| editor.focus_body(window, cx));
    f.update_editor(cx, |editor, _, cx| {
        editor.select_for_test(text.len()..text.len(), cx)
    });
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.simulate_input("\n## Next\n");
    f.draw(&mut vcx);
    let (shown, _) = f.editor(&mut vcx, |e, cx| e.live_for_test(cx));
    assert_eq!(
        shown.line_scales,
        vec![(0, heading_scale(1)), (4, heading_scale(2))]
    );

    // Edit mode draws the source as written.
    f.window
        .update(&mut vcx, |_, _, cx| {
            f.workspace.update(cx, |workspace, cx| {
                workspace.set_view_mode_for_test(ViewMode::Edit, cx)
            })
        })
        .unwrap();
    f.draw(&mut vcx);
    let (shown, live) = f.editor(&mut vcx, |e, cx| e.live_for_test(cx));
    assert!(!live);
    assert!(shown.is_empty(), "{shown:?}");
}

/// Opens a note in Live mode with `text` typed, the caret at its end.
fn live_note(cx: &mut TestAppContext, text: &str) -> Fixture {
    use crate::config::ViewMode;
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.window
        .update(cx, |_, _, cx| {
            f.workspace.update(cx, |workspace, cx| {
                workspace.set_view_mode_for_test(ViewMode::Live, cx)
            })
        })
        .unwrap();
    f.type_body(cx, text);
    f.update_editor(cx, |editor, _, cx| {
        editor.select_for_test(text.len()..text.len(), cx)
    });
    f.draw(cx);
    f
}

/// A click on a task's box in Live ticks the task, as in the preview, and
/// the box is drawn ticked.
// no-prop-test: drives the editor; `live_marks` has the properties and the
// vendor's tests click a replacement into this event.
#[gpui_kit::test]
fn live_task_box_click_ticks_the_task(cx: &mut TestAppContext) {
    use gpui_kit::base::input::{EditorPresentationEvent, Ornament};
    let text = "- [ ] Water the plants

Done.
";
    let f = live_note(cx, text);
    let (shown, live) = f.editor(cx, |e, cx| e.live_for_test(cx));
    assert!(live);
    let task = shown
        .replacements
        .iter()
        .find(|r| r.clickable)
        .expect("the task's box")
        .range
        .clone();
    f.update_editor(cx, |editor, _, cx| {
        editor.body_state().update(cx, |_, cx| {
            cx.emit(EditorPresentationEvent::ReplacementClicked { range: task })
        })
    });
    f.draw(cx);
    let body = f.editor(cx, |e, cx| e.body_state().read(cx).value().to_string());
    assert!(body.starts_with("- [x] Water the plants"), "{body}");
    let (shown, _) = f.editor(cx, |e, cx| e.live_for_test(cx));
    assert!(
        shown
            .line_ornaments
            .iter()
            .any(|(_, o)| matches!(o, Ornament::Check { fill: Some(_), .. })),
        "{shown:?}"
    );
}

/// A block Live draws with the preview's renderer shows its source once
/// clicked, with the caret at its start.
// no-prop-test: drives the editor; `live_marks` has the properties.
#[gpui_kit::test]
fn live_block_click_shows_its_source(cx: &mut TestAppContext) {
    use gpui_kit::base::input::EditorPresentationEvent;
    let text = "Intro

```rust
fn main() {}
```

End
";
    let f = live_note(cx, text);
    let (shown, _) = f.editor(cx, |e, cx| e.live_for_test(cx));
    let code = shown
        .block_widgets
        .iter()
        .find(|b| b.lines == (2..5))
        .expect("the code block drawn")
        .lines
        .clone();
    f.update_editor(cx, |editor, _, cx| {
        editor.body_state().update(cx, |_, cx| {
            cx.emit(EditorPresentationEvent::BlockClicked { lines: code })
        })
    });
    f.draw(cx);
    let (shown, _) = f.editor(cx, |e, cx| e.live_for_test(cx));
    assert!(
        shown.block_widgets.iter().all(|b| b.lines.start != 2),
        "{shown:?}"
    );
    let caret = f.editor(cx, |e, cx| e.body_state().read(cx).cursor());
    assert_eq!(caret, text.find("```").unwrap());
}

/// Arrowing up into a heading shows its `#`; leaving it hides it again.
// no-prop-test: drives the editor through keystrokes.
#[gpui_kit::test]
fn live_arrowing_into_a_heading_shows_its_marks(cx: &mut TestAppContext) {
    let text = "# Title
Body text
";
    let f = live_note(cx, text);
    f.update_editor(cx, |editor, window, cx| editor.focus_body(window, cx));
    f.update_editor(cx, |editor, _, cx| editor.select_for_test(9..9, cx));
    f.draw(cx);
    let (shown, _) = f.editor(cx, |e, cx| e.live_for_test(cx));
    assert!(shown.concealed.contains(&(0..2)), "{shown:?}");
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.simulate_keystrokes("up");
    f.draw(&mut vcx);
    let (shown, _) = f.editor(&mut vcx, |e, cx| e.live_for_test(cx));
    assert!(!shown.concealed.contains(&(0..2)), "{shown:?}");
    vcx.simulate_keystrokes("down");
    f.draw(&mut vcx);
    let (shown, _) = f.editor(&mut vcx, |e, cx| e.live_for_test(cx));
    assert!(shown.concealed.contains(&(0..2)), "{shown:?}");
}

/// ⌘4 and the toolbar's Live segment switch to Live mode, which is kept.
#[gpui_kit::test]
fn live_mode_is_chosen_with_its_key_and_kept(cx: &mut TestAppContext) {
    use crate::config::ViewMode;

    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.select(cx, &f.notes[0]);
    f.type_body(cx, "## Plan\n\n- [ ] **Ship** it\n");
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.simulate_keystrokes("cmd-4");
    f.draw(&mut vcx);
    assert_eq!(f.editor(&mut vcx, |e, _| e.view_mode), ViewMode::Live);
    assert_eq!(f.saved_config().appearance.view_mode, ViewMode::Live);
    let (shown, live) = f.editor(&mut vcx, |e, cx| e.live_for_test(cx));
    assert!(live);
    assert_eq!(shown.line_scales.len(), 1, "{shown:?}");
    // Live shows the source alone: no preview beside it.
    assert!(vcx.debug_bounds("preview-pane").is_none());

    vcx.simulate_keystrokes("cmd-2");
    f.draw(&mut vcx);
    assert_eq!(f.editor(&mut vcx, |e, _| e.view_mode), ViewMode::Split);
    let (shown, live) = f.editor(&mut vcx, |e, cx| e.live_for_test(cx));
    assert!(!live && shown.is_empty());
}

/// The Inbox note list shows the Triage bar; Triage opens on it, the keys
/// accept, reject and file a card, filing changes only what was accepted,
/// and a rejection is remembered.
// no-prop-test: drives one window through keystrokes; the chips' rules are
// properties in `kanso_engine::triage`.
#[gpui_kit::test]
fn triage_files_only_what_was_accepted(cx: &mut TestAppContext) {
    use kanso_core::roles::Role;

    let f = setup(cx);
    let inbox = f.store.role_book(Role::Inbox, Origin::Ui).unwrap();
    let book = f.store.note(&f.notes[0]).unwrap().unwrap().book_id;
    let target = f
        .store
        .put(
            DocType::Note,
            json!({"title": "Boiler service", "body": "x", "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let waiting = f
        .store
        .put(
            DocType::Note,
            json!({"title": "Untitled", "bookId": inbox,
                   "body": "Buy milk and eggs. Ask about the boiler service soon."}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    cx.update(crate::workspace::bind_keys);
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    let library = f.workspace(cx, |w, _| w.library.clone());
    library.update(cx, |library, cx| {
        library.set_filter(crate::library::Filter::Book(inbox.clone()), cx)
    });
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    f.draw(&mut vcx);
    assert!(vcx.debug_bounds("triage-bar").is_some(), "the bar shows");

    vcx.dispatch_action(crate::triage_view::ShowTriage);
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.triage_open));
    let triage = f.workspace(&mut vcx, |w, _| w.triage.clone().unwrap());
    vcx.update(|_, cx| triage.update(cx, |t, cx| t.reload_now(cx)));
    f.draw(&mut vcx);
    let chips = |vcx: &mut gpui_kit::VisualTestContext| {
        vcx.read(|cx| {
            triage.read(cx).session().cards[0]
                .chips
                .iter()
                .map(|c| (c.suggestion.field, c.mark))
                .collect::<Vec<_>>()
        })
    };
    use kanso_engine::triage::Field;
    use kanso_engine::triage::session::Mark;
    assert_eq!(
        chips(&mut vcx),
        [
            (Field::Title, Mark::Untouched),
            (Field::Link, Mark::Untouched)
        ]
    );

    // Accept the title, reject the link, file.
    vcx.simulate_keystrokes("y l n");
    f.draw(&mut vcx);
    assert_eq!(
        chips(&mut vcx),
        [
            (Field::Title, Mark::Accepted),
            (Field::Link, Mark::Rejected)
        ]
    );
    assert_eq!(
        f.store.note(&waiting).unwrap().unwrap().title,
        "Untitled",
        "nothing changes before File"
    );
    assert_eq!(f.store.triage_dismissed(&waiting).unwrap().len(), 1);
    vcx.simulate_keystrokes("f");
    f.draw(&mut vcx);
    let note = f.store.note(&waiting).unwrap().unwrap();
    assert_eq!(note.title, "Buy milk and eggs");
    assert!(
        !note.body.contains("Related:"),
        "the rejected link isn't added"
    );
    assert_eq!(note.book_id, inbox, "no notebook was accepted");
    assert!(note.extra.contains_key("kansoTriaged"));
    assert!(f.store.inbox_waiting(&inbox).unwrap().is_empty());
    let _ = target;

    vcx.simulate_keystrokes("escape");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.triage_open));
}

/// Filing a card and then pressing Undo brings the card back into
/// Triage, and the note is stamped as edited by the undo.
// no-prop-test: drives one window through a keystroke and a click; the
// undo's rules are properties in `kanso_engine::triage::file`.
#[gpui_kit::test]
fn triage_undo_brings_the_card_back(cx: &mut TestAppContext) {
    use kanso_core::roles::Role;

    let f = setup(cx);
    let inbox = f.store.role_book(Role::Inbox, Origin::Ui).unwrap();
    let waiting = f
        .store
        .put(
            DocType::Note,
            json!({"title": "Untitled", "bookId": inbox,
                   "body": "Buy milk and eggs. Then call someone."}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let before = f.store.note(&waiting).unwrap().unwrap().updated_at;
    cx.update(crate::workspace::bind_keys);
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    let library = f.workspace(cx, |w, _| w.library.clone());
    library.update(cx, |library, cx| {
        library.set_filter(crate::library::Filter::Book(inbox.clone()), cx)
    });
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    f.draw(&mut vcx);
    vcx.dispatch_action(crate::triage_view::ShowTriage);
    f.draw(&mut vcx);
    let triage = f.workspace(&mut vcx, |w, _| w.triage.clone().unwrap());
    vcx.update(|_, cx| triage.update(cx, |t, cx| t.reload_now(cx)));
    f.draw(&mut vcx);
    let shown = |vcx: &mut gpui_kit::VisualTestContext| {
        vcx.read(|cx| {
            triage
                .read(cx)
                .session()
                .cards
                .iter()
                .any(|c| c.note_id == waiting)
        })
    };
    assert!(shown(&mut vcx));

    // Accept the title and file: the card leaves.
    vcx.simulate_keystrokes("y f");
    f.draw(&mut vcx);
    assert!(!shown(&mut vcx));
    assert_eq!(
        f.store.note(&waiting).unwrap().unwrap().title,
        "Buy milk and eggs"
    );

    // Undo: the note is back as it was, and its card shows again. Once
    // the notice has slid in, which takes real time.
    std::thread::sleep(std::time::Duration::from_millis(600));
    f.draw(&mut vcx);
    let bounds = vcx
        .debug_bounds("undo-triage")
        .expect("the notice has Undo");
    vcx.simulate_click(bounds.center(), gpui_kit::Modifiers::none());
    vcx.update(|_, cx| triage.update(cx, |t, cx| t.reload_now(cx)));
    f.draw(&mut vcx);
    let note = f.store.note(&waiting).unwrap().unwrap();
    assert_eq!(note.title, "Untitled");
    assert!(note.updated_at >= before, "stamped, not restored");
    assert!(shown(&mut vcx), "the card is back");
}

/// "File all accepted" is disabled with nothing accepted, and the
/// Inbox settings are the defaults the spec names.
#[test]
fn the_inbox_settings_default_as_specified() {
    let config = AppConfig::default();
    assert!(config.inbox.show_triage_bar);
    assert!(config.inbox.suggest_titles && config.inbox.suggest_links);
    assert_eq!(config.inbox.max_tags, 3);
    assert!(!config.ai.inbox_suggestions);
    let opts = crate::triage_view::options(&config);
    assert_eq!(opts.max_tags, 3);
    assert_eq!(opts.inbox_id, "book:inbox");
}

/// Overview takes the list's and the editor's place, and its header never
/// clips: through every width, the switch and the menu are on screen inside
/// the page.
// no-prop-test: drives one window through widths.
#[gpui_kit::test]
fn the_overview_header_fits_every_width(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(|cx| AppState::update_config(cx, |config| config.ai.enabled = true));
    f.select(cx, &f.notes[0]);
    let library = f.workspace(cx, |w, _| w.library.clone());
    library.update(cx, |library, cx| {
        library.set_filter(crate::library::Filter::Overview, cx)
    });
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    for width in [1600., 1100., 820., 700., 600., 520.] {
        vcx.simulate_resize(gpui_kit::size(gpui_kit::px(width), gpui_kit::px(800.)));
        vcx.executor().advance_clock(Duration::from_millis(600));
        for _ in 0..3 {
            f.draw(&mut vcx);
        }
        let pane = vcx
            .debug_bounds("page-pane")
            .expect("Overview fills the content area");
        assert_eq!(vcx.debug_bounds("list-pane"), None, "{width}");
        assert_eq!(vcx.debug_bounds("editor-pane"), None, "{width}");
        let header = vcx.debug_bounds("overview-header").unwrap();
        assert!(header.right() <= pane.right(), "{width}");
        for id in ["overview-switch", "overview-more"] {
            let b = vcx
                .debug_bounds(id)
                .unwrap_or_else(|| panic!("{id} missing at {width}"));
            assert!(
                b.right() <= pane.right() && b.left() >= pane.left(),
                "{id} clipped at {width}"
            );
        }
    }
}

/// Open Loops with nothing in it says what the page is for and what to
/// do, in place of a grey line.
// no-prop-test: one render of the empty state.
#[gpui_kit::test]
fn an_empty_page_explains_itself(cx: &mut TestAppContext) {
    let f = setup(cx);
    let library = f.workspace(cx, |w, _| w.library.clone());
    library.update(cx, |library, cx| {
        library.set_filter(crate::library::Filter::OpenLoops, cx)
    });
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.executor().advance_clock(Duration::from_millis(600));
    for _ in 0..3 {
        f.draw(&mut vcx);
    }
    assert!(vcx.debug_bounds("page-empty").is_some());
}

fn settle(f: &Fixture, vcx: &mut gpui_kit::VisualTestContext) {
    vcx.executor().advance_clock(Duration::from_millis(600));
    vcx.run_until_parked();
    for _ in 0..3 {
        f.draw(vcx);
    }
}

/// A note opened any way but a click in the page (the palette, a tab, a
/// link) brings the editor back from the review.
// no-prop-test: drives one window through selections.
#[gpui_kit::test]
fn opening_a_note_leaves_the_page(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    let library = f.workspace(cx, |w, _| w.library.clone());
    library.update(cx, |library, cx| {
        library.set_filter(crate::library::Filter::Overview, cx)
    });
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    settle(&f, &mut vcx);
    assert_eq!(vcx.debug_bounds("editor-pane"), None);
    let other = f.notes[1].clone();
    library.update(&mut vcx, |library, cx| {
        library.select(Some(other.clone()), cx)
    });
    settle(&f, &mut vcx);
    assert!(vcx.debug_bounds("editor-pane").is_some(), "the note shows");
    assert_eq!(vcx.debug_bounds("page-pane"), None);
    let selected = vcx.read(|cx| library.read(cx).selected.clone());
    assert_eq!(selected, Some(other), "the note stays selected");

    // A new note too.
    library.update(&mut vcx, |library, cx| {
        library.set_filter(crate::library::Filter::Overview, cx)
    });
    settle(&f, &mut vcx);
    assert_eq!(vcx.debug_bounds("editor-pane"), None);
    vcx.dispatch_action(crate::workspace::NewNote);
    settle(&f, &mut vcx);
    assert!(vcx.debug_bounds("editor-pane").is_some(), "new note shows");
    assert_eq!(vcx.debug_bounds("page-pane"), None);
}

/// A trashed note opened from a page shows in the Trash and stays the
/// note selected; Escape leaves a page for the place before it.
// no-prop-test: drives one window.
#[gpui_kit::test]
fn a_trashed_note_stays_selected_and_escape_goes_back(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    let library = f.workspace(cx, |w, _| w.library.clone());
    let trashed = f.notes[1].clone();
    f.store
        .put(
            DocType::Note,
            json!({"_id": trashed, "bookId": kanso_core::model::TRASH_BOOK_ID}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    library.update(cx, |library, cx| {
        library.set_filter(crate::library::Filter::Overview, cx)
    });
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    settle(&f, &mut vcx);
    library.update(&mut vcx, |library, cx| {
        library.select(Some(trashed.clone()), cx)
    });
    settle(&f, &mut vcx);
    let (filter, selected) = vcx.read(|cx| {
        let l = library.read(cx);
        (l.filter.clone(), l.selected.clone())
    });
    assert_eq!(filter, crate::library::Filter::Trash);
    assert_eq!(selected, Some(trashed));

    library.update(&mut vcx, |library, cx| {
        library.set_filter(crate::library::Filter::All, cx);
        library.set_filter(crate::library::Filter::Overview, cx)
    });
    settle(&f, &mut vcx);
    vcx.simulate_keystrokes("escape");
    settle(&f, &mut vcx);
    let filter = vcx.read(|cx| library.read(cx).filter.clone());
    assert_eq!(filter, crate::library::Filter::All);
    assert!(vcx.debug_bounds("editor-pane").is_some());
}

/// Choosing a place in the sidebar while Triage shows closes Triage.
// no-prop-test: drives one window.
#[gpui_kit::test]
fn a_place_chosen_closes_triage(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    let library = f.workspace(cx, |w, _| w.library.clone());
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.dispatch_action(crate::triage_view::ShowTriage);
    settle(&f, &mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.triage_open));
    library.update(&mut vcx, |library, cx| {
        library.set_filter(crate::library::Filter::Overview, cx)
    });
    settle(&f, &mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.triage_open));
    assert!(vcx.debug_bounds("page-pane").is_some());
}

/// The error card keeps the raw error behind a Details button.
// no-prop-test: the status-code rule is a property in page_chrome.rs.
#[gpui_kit::test]
fn the_error_card_has_details(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(|cx| AppState::update_config(cx, |config| config.ai.enabled = true));
    f.select(cx, &f.notes[0]);
    let library = f.workspace(cx, |w, _| w.library.clone());
    library.update(cx, |library, cx| {
        library.set_filter(crate::library::Filter::Overview, cx)
    });
    let digest = f.workspace(cx, |w, _| w.digest.clone());
    digest.update(cx, |d, _| d.show_error_for_test("HTTP 401 bad key"));
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    settle(&f, &mut vcx);
    assert!(vcx.debug_bounds("ai-error").is_some());
    assert_eq!(vcx.debug_bounds("ai-details-text"), None);
    let at = vcx.debug_bounds("ai-details").unwrap().center();
    vcx.simulate_click(at, gpui_kit::Modifiers::none());
    settle(&f, &mut vcx);
    assert!(vcx.debug_bounds("ai-details-text").is_some());
}

/// Waits for a worker thread's answer to reach the page: lets the clock and
/// the threads run until `ready` or a few seconds have passed.
fn wait_until(
    f: &Fixture,
    vcx: &mut gpui_kit::VisualTestContext,
    ready: impl Fn(&mut gpui_kit::VisualTestContext) -> bool,
) {
    vcx.executor().allow_parking();
    for _ in 0..200 {
        settle(f, vcx);
        if ready(vcx) {
            return;
        }
        std::thread::sleep(Duration::from_millis(15));
    }
}

/// Draft from This Note refuses while AI is off and, once on, names exactly
/// the notes it will send, counting what it leaves out.
// no-prop-test: drives one window; the counts are `engine::resolve`'s.
#[gpui_kit::test]
fn the_draft_page_names_its_sources(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.dispatch_action(crate::learn_view::DraftFromNote);
    settle(&f, &mut vcx);
    assert!(
        !f.workspace(&mut vcx, |w, _| w.learn_open),
        "nothing opens while AI is off"
    );

    vcx.update(|_, cx| AppState::update_config(cx, |config| config.ai.enabled = true));
    vcx.dispatch_action(crate::learn_view::DraftFromNote);
    settle(&f, &mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.learn_open));
    assert!(vcx.debug_bounds("learn-overlay").is_some());
    assert!(vcx.debug_bounds("learn-sources").is_some());
    assert!(
        vcx.debug_bounds("learn-draft").is_none(),
        "nothing written yet"
    );
}

/// Make Flashcards asks the model about the one note, shows what came back
/// with its source line, and writes only the ticked cards, once.
// no-prop-test: drives one window with a model that says one fixed reply.
#[gpui_kit::test]
fn make_flashcards_adds_the_ticked_cards(cx: &mut TestAppContext) {
    use crate::ai_feature::testing::{Injected, Recorder};

    let f = setup(cx);
    let id = f.notes[0].clone();
    f.store
        .put(
            DocType::Note,
            json!({"_id": id, "body": "Capital of France is Paris.\nCats sleep a lot.\n"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let model = Recorder::new(
        r#"{"q":"Capital of France?","a":"Paris","source":"Capital of France is Paris."}
{"q":"What do cats do?","a":"Sleep","source":"a line the note never had"}"#,
    );
    cx.update(|cx| {
        AppState::update_config(cx, |config| config.ai.enabled = true);
        cx.set_global(Injected(model.maker()));
    });
    f.select(cx, &id);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.dispatch_action(crate::learn_view::MakeFlashcardsFromNote);
    wait_until(&f, &mut vcx, |vcx| {
        vcx.debug_bounds("learn-card-1").is_some()
    });
    assert!(vcx.debug_bounds("learn-card-0").is_some());
    assert!(vcx.debug_bounds("learn-card-1").is_some());
    assert!(
        vcx.debug_bounds("learn-overlay").is_none(),
        "Make Flashcards is a dialog, not the page"
    );
    // Only the note's own text was sent.
    let sent = model.sent();
    assert_eq!(sent.len(), 1);

    let learn = f.workspace(&mut vcx, |w, _| w.learn_modal.view().unwrap());
    f.window
        .update(&mut vcx, |_, window, cx| {
            learn.update(cx, |learn, cx| learn.add_cards(window, cx))
        })
        .unwrap();
    settle(&f, &mut vcx);
    let body = f.store.note(&id).unwrap().unwrap().body;
    assert!(body.contains("## Flashcards"), "{body}");
    assert!(body.contains("Capital of France? :: Paris"), "{body}");
    assert!(
        !body.contains("What do cats do?"),
        "a card whose source is not in the note starts unticked"
    );
}

/// Quiz Me and Make Flashcards open as dialogs over the note, from the
/// actions the menu, the palette and the selection menu run, and closing
/// one gives the note back as it was: the same editor, its text, its caret
/// and the keyboard. Drawing each one is also the check that no view reads
/// its own entity while it renders.
// no-prop-test: drives one window; the logic under it is the engine's.
#[gpui_kit::test]
fn quiz_me_and_flashcards_open_over_the_note_and_close_back_to_it(cx: &mut TestAppContext) {
    use crate::ai_feature::testing::{Injected, Recorder};

    let f = setup(cx);
    let id = f.notes[0].clone();
    let body = "Capital of France is Paris.\nCats sleep a lot.\nWater boils at 100.\nThe end.\n";
    f.store
        .put(
            DocType::Note,
            json!({"_id": id, "body": body}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let model = Recorder::new(
        r#"{"q":"What is the capital of France?","n":1}
{"q":"Capital of France?","a":"Paris","source":"Capital of France is Paris."}"#,
    );
    cx.update(|cx| {
        AppState::update_config(cx, |config| config.ai.enabled = true);
        cx.set_global(Injected(model.maker()));
    });
    f.select(cx, &id);
    f.update_editor(cx, |editor, window, cx| {
        editor.reveal_line(2, window, cx);
        editor.focus_body(window, cx);
    });
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    settle(&f, &mut vcx);
    let caret = f.editor(&mut vcx, |e, cx| e.caret_line(cx));
    assert_eq!(caret, 2);
    let editor_id = f.workspace(&mut vcx, |w, _| w.editor.entity_id());
    let editor = f.workspace(&mut vcx, |w, _| w.editor.clone());
    let text = f.editor(&mut vcx, |e, cx| e.body_text(cx).to_string());
    let focused = |vcx: &mut gpui_kit::VisualTestContext| {
        f.window
            .update(vcx, |_, window, cx| {
                editor.read(cx).body_focused(window, cx)
            })
            .unwrap()
    };
    assert!(focused(&mut vcx));

    // The action is what the Note menu, the palette and the selection menu
    // all run; the command is what the menu bar and other windows send.
    for (name, open, expect) in [
        (
            "quiz",
            Box::new(|vcx: &mut gpui_kit::VisualTestContext| {
                vcx.dispatch_action(crate::learn_view::QuizMeOnNote)
            }) as Box<dyn Fn(&mut gpui_kit::VisualTestContext)>,
            "learn-question",
        ),
        (
            "cards",
            Box::new(|vcx: &mut gpui_kit::VisualTestContext| {
                vcx.dispatch_action(crate::learn_view::MakeFlashcardsFromNote)
            }),
            "learn-card-0",
        ),
    ] {
        open(&mut vcx);
        wait_until(&f, &mut vcx, |vcx| vcx.debug_bounds(expect).is_some());
        assert!(vcx.debug_bounds(expect).is_some(), "{name} is drawn");
        assert!(vcx.debug_bounds("dialog-0").is_some(), "{name} is a dialog");
        assert!(
            vcx.debug_bounds("learn-overlay").is_none(),
            "{name} is not the page"
        );
        assert!(!f.workspace(&mut vcx, |w, _| w.learn_open));
        assert!(!focused(&mut vcx), "the dialog holds the keyboard");

        vcx.simulate_keystrokes("escape");
        settle(&f, &mut vcx);
        assert!(vcx.debug_bounds("dialog-0").is_none(), "{name} closed");
        assert!(focused(&mut vcx), "{name}: the caret is back in the note");
        assert_eq!(f.editor(&mut vcx, |e, cx| e.caret_line(cx)), caret);
        assert_eq!(
            f.editor(&mut vcx, |e, cx| e.body_text(cx).to_string()),
            text
        );
        assert_eq!(
            f.workspace(&mut vcx, |w, _| w.editor.entity_id()),
            editor_id
        );
        assert_eq!(f.open_note(&mut vcx), Some(id.clone()));
    }

    // The note menu's command opens the same dialog.
    f.window
        .update(&mut vcx, |_, window, cx| {
            f.workspace.update(cx, |workspace, cx| {
                workspace.run_note_command(&id, crate::note_menu::NoteCommand::QuizMe, window, cx)
            })
        })
        .unwrap();
    wait_until(&f, &mut vcx, |vcx| {
        vcx.debug_bounds("learn-question").is_some()
    });
    assert!(vcx.debug_bounds("dialog-0").is_some());
    vcx.simulate_keystrokes("escape");
    settle(&f, &mut vcx);
    assert!(vcx.debug_bounds("dialog-0").is_none());
    assert!(focused(&mut vcx));
}

/// Asking twice keeps one dialog. Quiz Me covers the note (it holds the
/// answers) and Make Flashcards does not, and closing either lets the view
/// go and takes the cover with it.
// no-prop-test: drives one window through actions; nothing to generate.
#[gpui_kit::test]
fn a_second_ask_keeps_one_dialog_and_quiz_me_covers_the_note(cx: &mut TestAppContext) {
    use crate::ai_feature::testing::{Injected, Recorder};

    let f = setup(cx);
    let id = f.notes[0].clone();
    let model = Recorder::new(r#"{"q":"What is the capital of France?","n":1}"#);
    cx.update(|cx| {
        AppState::update_config(cx, |config| config.ai.enabled = true);
        cx.set_global(Injected(model.maker()));
    });
    f.select(cx, &id);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);

    vcx.dispatch_action(crate::learn_view::QuizMeOnNote);
    wait_until(&f, &mut vcx, |vcx| vcx.debug_bounds("dialog-0").is_some());
    let first = f.workspace(&mut vcx, |w, _| w.learn_modal.view().unwrap().entity_id());
    vcx.dispatch_action(crate::learn_view::QuizMeOnNote);
    vcx.dispatch_action(crate::learn_view::MakeFlashcardsFromNote);
    settle(&f, &mut vcx);
    assert!(vcx.debug_bounds("dialog-0").is_some());
    assert!(vcx.debug_bounds("dialog-1").is_none(), "no second dialog");
    assert_eq!(
        f.workspace(&mut vcx, |w, _| w.learn_modal.view().unwrap().entity_id()),
        first
    );
    assert!(
        vcx.debug_bounds("learn-quiz-cover").is_some(),
        "the note is hidden"
    );

    vcx.simulate_keystrokes("escape");
    settle(&f, &mut vcx);
    assert!(vcx.debug_bounds("dialog-0").is_none());
    assert!(
        vcx.debug_bounds("learn-quiz-cover").is_none(),
        "the cover goes"
    );
    assert!(
        f.workspace(&mut vcx, |w, _| w.learn_modal.view().is_none()),
        "the view goes with its dialog"
    );

    vcx.dispatch_action(crate::learn_view::MakeFlashcardsFromNote);
    wait_until(&f, &mut vcx, |vcx| vcx.debug_bounds("dialog-0").is_some());
    assert!(
        vcx.debug_bounds("learn-quiz-cover").is_none(),
        "Make Flashcards shows the note"
    );
    vcx.simulate_keystrokes("escape");
    settle(&f, &mut vcx);
    assert!(vcx.debug_bounds("dialog-0").is_none());
}

/// In a note window, the Note menu's Quiz Me opens over that window's note,
/// covered, and the main window is left alone. Closing it gives the window's
/// editor the keyboard back.
// no-prop-test: drives two windows through one action; nothing to generate.
#[gpui_kit::test]
fn quiz_me_in_a_note_window_opens_there(cx: &mut TestAppContext) {
    use crate::ai_feature::testing::{Injected, Recorder};
    use crate::note_window::NoteWindow;

    let f = setup(cx);
    let id = f.notes[0].clone();
    let model = Recorder::new(r#"{"q":"What is the capital of France?","n":1}"#);
    cx.update(|cx| {
        AppState::update_config(cx, |config| config.ai.enabled = true);
        cx.set_global(Injected(model.maker()));
    });
    let library = cx.read(|cx| f.workspace.read(cx).library.clone());
    let (handle, note) = cx.update(|cx| {
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| NoteWindow::new(id.clone(), library, None, window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    let editor = cx.read(|cx| note.read(cx).editor.clone());
    let mut nvcx = gpui_kit::VisualTestContext::from_window(handle, cx);
    handle
        .update(&mut nvcx, |_, window, cx| {
            editor.update(cx, |editor, cx| editor.focus_body(window, cx))
        })
        .unwrap();
    let focused = |vcx: &mut gpui_kit::VisualTestContext| {
        handle
            .update(vcx, |_, window, cx| {
                editor.read(cx).body_focused(window, cx)
            })
            .unwrap()
    };
    nvcx.run_until_parked();
    assert!(focused(&mut nvcx));

    nvcx.dispatch_action(crate::learn_view::QuizMeOnNote);
    nvcx.dispatch_action(crate::learn_view::QuizMeOnNote);
    for _ in 0..200 {
        nvcx.executor().allow_parking();
        nvcx.executor().advance_clock(Duration::from_millis(600));
        nvcx.run_until_parked();
        if nvcx.read(|cx| note.read(cx).learn_modal.view().is_some()) {
            break;
        }
        std::thread::sleep(Duration::from_millis(15));
    }
    nvcx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert!(
        nvcx.debug_bounds("dialog-0").is_some(),
        "opens in the note window"
    );
    assert!(nvcx.debug_bounds("dialog-1").is_none(), "only once");
    assert!(nvcx.debug_bounds("learn-quiz-cover").is_some());
    assert!(!focused(&mut nvcx));
    assert!(
        f.workspace(&mut nvcx, |w, _| w.learn_modal.view().is_none()),
        "not in the main window"
    );

    nvcx.simulate_keystrokes("escape");
    nvcx.executor().advance_clock(Duration::from_millis(600));
    nvcx.run_until_parked();
    nvcx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert!(nvcx.debug_bounds("dialog-0").is_none());
    assert!(nvcx.debug_bounds("learn-quiz-cover").is_none());
    assert!(focused(&mut nvcx), "the caret is back in the window's note");
}

/// A model that fails is told in words, with Try Again, never raw.
// no-prop-test: drives one window; the wording is page_chrome's rule.
#[gpui_kit::test]
fn quiz_me_tells_a_failed_model_in_words(cx: &mut TestAppContext) {
    use crate::ai_feature::testing::Injected;

    let f = setup(cx);
    let id = f.notes[0].clone();
    cx.update(|cx| {
        AppState::update_config(cx, |config| config.ai.enabled = true);
        cx.set_global(Injected(std::sync::Arc::new(|| {
            Err("HTTP 401 unauthorized: bad key".to_owned())
        })));
    });
    f.select(cx, &id);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.dispatch_action(crate::learn_view::QuizMeOnNote);
    wait_until(&f, &mut vcx, |vcx| vcx.debug_bounds("ai-error").is_some());
    assert!(vcx.debug_bounds("ai-error").is_some());
    // The card keeps the raw error behind Details, and says the key was refused.
    assert!(vcx.debug_bounds("ai-details").is_some());
    assert_eq!(vcx.debug_bounds("ai-details-text"), None);
}

/// A person's page shows its live sections under the note, and a note that
/// is not a page shows none. The panel's rules are the engine's.
// no-prop-test: drives one window; the sections are properties in
// `kanso_engine::pages` and `kanso_store`'s entity tests.
#[gpui_kit::test]
fn a_people_page_shows_live_sections_below_its_note(cx: &mut TestAppContext) {
    let f = setup(cx);
    let book = f.store.note(&f.notes[0]).unwrap().unwrap().book_id;
    let page = kanso_engine::entities::create_page(
        &f.store,
        &Default::default(),
        kanso_core::entities::EntityKind::Person,
        "Sam Moyo",
        &["Sam".to_owned()],
        1_700_000_000_000,
    )
    .unwrap();
    f.store
        .put(
            DocType::Note,
            json!({"title": "Weekly", "bookId": book,
                   "body": "Sam said the office opens in March\n- [ ] Send Sam the deck\n"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    f.select(cx, &f.notes[0]);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    f.draw(&mut vcx);
    assert!(
        vcx.debug_bounds("page-panel").is_none(),
        "an ordinary note has no panel"
    );

    f.select(&mut vcx, &page);
    wait_until(&f, &mut vcx, |vcx| vcx.debug_bounds("page-panel").is_some());
    assert!(vcx.debug_bounds("page-panel").is_some());
    assert!(vcx.debug_bounds("page-section-0").is_some());
}

/// People & Projects opens over the window, offers a name that keeps coming
/// up, and makes nothing until Create page is pressed.
// no-prop-test: drives one window through an action; detection is
// properties in `kanso_engine::entities::detect`.
#[gpui_kit::test]
fn people_view_offers_a_name_and_makes_nothing_unasked(cx: &mut TestAppContext) {
    let f = setup(cx);
    let book = f.store.note(&f.notes[0]).unwrap().unwrap().book_id;
    for ix in 0..3 {
        f.store
            .put(
                DocType::Note,
                json!({"title": format!("Lunch {ix}"), "bookId": book,
                       "body": "Lunch with Tendai Chirwa went well\n"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    }
    let settings = kanso_engine::entities::EntitySettings::default();
    let mut last = 0;
    kanso_engine::entities::suggest::refresh(
        &f.store,
        &settings,
        1_700_000_000_000,
        &mut last,
        true,
    )
    .unwrap();
    assert!(f.store.entities().unwrap().is_empty());

    cx.update(crate::workspace::bind_keys);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.dispatch_action(crate::people_view::ShowPeople);
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.people_open));
    let people = f.workspace(&mut vcx, |w, _| w.people.clone().unwrap());
    vcx.update(|_, cx| people.update(cx, |p, cx| p.reload_now(cx)));
    f.draw(&mut vcx);
    assert_eq!(
        vcx.read(|cx| people.read(cx).suggestions().len()),
        1,
        "one name keeps coming up"
    );
    assert!(vcx.debug_bounds("people-suggestion-0").is_some());
    assert!(
        f.store.entities().unwrap().is_empty(),
        "showing the suggestion made nothing"
    );

    vcx.update(|_, cx| {
        people.update(cx, |p, cx| {
            let s = p.suggestions()[0].clone();
            p.dismiss_for_test(&s, true, cx)
        })
    });
    assert!(f.store.entities().unwrap().is_empty());
    vcx.update(|_, cx| people.update(cx, |p, cx| p.reload_now(cx)));
    assert_eq!(vcx.read(|cx| people.read(cx).suggestions().len()), 0);

    vcx.simulate_keystrokes("escape");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.people_open));
}

/// A notebook with notes in it shows its homepage when asked, and picking a
/// note closes the homepage.
// no-prop-test: drives one window; the homepage's content is properties in
// `kanso_engine::pages::notebook` and `kanso_store`'s home tests.
#[gpui_kit::test]
fn a_notebook_opens_to_its_homepage_and_a_note_closes_it(cx: &mut TestAppContext) {
    let f = setup(cx);
    let book = f.store.note(&f.notes[0]).unwrap().unwrap().book_id;
    let library = f.workspace(cx, |w, _| w.library.clone());
    library.update(cx, |l, cx| {
        l.set_filter(crate::library::Filter::Book(book), cx)
    });
    cx.run_until_parked();
    library.update(cx, |l, cx| l.set_home(true, cx));
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    wait_until(&f, &mut vcx, |vcx| {
        vcx.debug_bounds("notebook-home").is_some()
    });
    assert!(vcx.debug_bounds("notebook-home").is_some());
    let home = f.workspace(&mut vcx, |w, _| w.home.clone());
    let (loaded, keys, titles, first) = vcx.read(|cx| {
        let h = home.read(cx);
        (
            h.is_loaded(),
            h.section_keys(),
            h.section_titles(),
            h.first_visit(),
        )
    });
    assert!(loaded);
    assert_eq!(keys.len(), titles.len());
    // The words and order are the engine's, which the phones draw too.
    assert!(
        keys.iter().any(|k| k == "key_notes"),
        "key_notes in {keys:?}"
    );
    assert!(
        !keys.iter().any(|k| k == "about"),
        "no AI section with AI off"
    );
    assert!(!first, "AI is off in a bare window");

    f.select(&mut vcx, &f.notes[1]);
    f.draw(&mut vcx);
    assert!(vcx.debug_bounds("notebook-home").is_none());
}

/// A Home note moved into a notebook that has a Home note of its own loses
/// its mark (issue #201); the other notebook's stays. The editor takes the
/// note without the mark, so typing on saves with no conflict.
// no-prop-test: one move through the real editor; the rule itself is
// `notebook_actions::after_move`'s, tested in kanso-engine.
#[gpui_kit::test]
fn a_home_note_moved_into_a_notebook_with_one_loses_its_mark(cx: &mut TestAppContext) {
    use kanso_core::home::{is_home, mark_home};
    let f = setup(cx);
    let other = f
        .store
        .put(DocType::Book, json!({"name": "C"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let theirs = f
        .store
        .put(
            DocType::Note,
            json!({"title": "About C", "body": mark_home("C's own words."), "bookId": other}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let mine = f.notes[0].clone();
    f.edit_elsewhere(cx, &mine, &mark_home("start"));
    f.select(cx, &mine);
    assert!(is_home(&f.editor(cx, |e, cx| e.body_text(cx).to_string())));
    f.update_editor(cx, |editor, _, cx| editor.set_book(other.clone(), cx));
    cx.run_until_parked();
    assert_eq!(f.store.note(&mine).unwrap().unwrap().book_id, other);
    assert!(
        !is_home(&note_body(&f.store, &mine)),
        "the moved one is ordinary"
    );
    assert!(
        is_home(&note_body(&f.store, &theirs)),
        "C keeps its Home note"
    );
    let text = f.editor(cx, |e, cx| e.body_text(cx).to_string());
    assert!(!is_home(&text), "the editor took the note without the mark");
    f.type_body(cx, &format!("{text}\nmore"));
    f.update_editor(cx, |editor, _, cx| editor.flush(cx));
    assert_eq!(note_body(&f.store, &mine), format!("{text}\nmore"));
    assert!(!is_home(&note_body(&f.store, &mine)));
}

/// New Home note opens the note in a tab of its own; closing that tab
/// shows the notebook's homepage again (issue #201).
// no-prop-test: one window's tabs; nothing to vary.
#[gpui_kit::test]
fn a_new_home_note_returns_to_the_homepage_when_its_tab_closes(cx: &mut TestAppContext) {
    let f = setup(cx);
    let book = f.store.note(&f.notes[0]).unwrap().unwrap().book_id;
    let (library, home) = f.workspace(cx, |w, _| (w.library.clone(), w.home.clone()));
    library.update(cx, |l, cx| {
        l.set_filter(crate::library::Filter::Book(book.clone()), cx)
    });
    cx.run_until_parked();
    library.update(cx, |l, cx| l.set_home(true, cx));
    cx.run_until_parked();
    assert_eq!(
        library.read_with(cx, |l, cx| l.home_book(cx).map(str::to_owned)),
        Some(book.clone())
    );
    home.update(cx, |home, cx| home.new_home(cx));
    cx.run_until_parked();
    let made = f.open_note(cx).expect("the Home note opens");
    assert!(kanso_core::home::is_home(&note_body(&f.store, &made)));
    assert_eq!(
        library.read_with(cx, |l, cx| l.home_book(cx).map(str::to_owned)),
        None
    );
    let (tabs, active) = f.workspace(cx, |w, _| (w.tabs.len(), w.tabs.active_index()));
    assert_eq!(tabs, 2, "in a tab of its own");
    let workspace = f.workspace.clone();
    f.window
        .update(cx, |_, window, cx| {
            workspace.update(cx, |w, cx| w.close_tab(active, window, cx))
        })
        .unwrap();
    cx.run_until_parked();
    assert_eq!(
        library.read_with(cx, |l, cx| l.home_book(cx).map(str::to_owned)),
        Some(book),
        "back on the homepage"
    );
}

/// The first window opens to Overview (and says so once), Back from a note
/// returns to it, and Escape leaves it for the place before.
// no-prop-test: drives one window.
#[gpui_kit::test]
fn kanso_opens_to_overview_and_back_returns_to_it(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    let library = f.workspace(cx, |w, _| w.library.clone());
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let workspace = f.workspace.clone();
    vcx.update(|window, cx| {
        workspace.update(cx, |workspace, cx| workspace.launch_to_overview(window, cx))
    });
    settle(&f, &mut vcx);
    assert_eq!(
        vcx.read(|cx| library.read(cx).filter.clone()),
        crate::library::Filter::Overview
    );
    assert!(vcx.debug_bounds("page-pane").is_some());
    assert_eq!(vcx.debug_bounds("editor-pane"), None);

    // Open a note, then go back: Overview is a visit.
    let other = f.notes[1].clone();
    library.update(&mut vcx, |library, cx| {
        library.select(Some(other.clone()), cx)
    });
    settle(&f, &mut vcx);
    assert!(vcx.debug_bounds("editor-pane").is_some());
    vcx.update(|_, cx| workspace.update(cx, |workspace, cx| workspace.go_back(cx)));
    settle(&f, &mut vcx);
    assert_eq!(
        vcx.read(|cx| library.read(cx).filter.clone()),
        crate::library::Filter::Overview
    );

    vcx.simulate_keystrokes("escape");
    settle(&f, &mut vcx);
    assert_ne!(
        vcx.read(|cx| library.read(cx).filter.clone()),
        crate::library::Filter::Overview
    );
}

/// A reminder for the note Overview loaded quietly at launch still shows
/// that note: it is already open behind the page, so only the page goes.
// no-prop-test: drives one window.
#[gpui_kit::test]
fn a_reminder_for_the_open_note_leaves_overview(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    let library = f.workspace(cx, |w, _| w.library.clone());
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let workspace = f.workspace.clone();
    vcx.update(|window, cx| {
        workspace.update(cx, |workspace, cx| workspace.launch_to_overview(window, cx))
    });
    settle(&f, &mut vcx);
    assert_eq!(vcx.debug_bounds("editor-pane"), None);
    let id = f.notes[0].clone();
    vcx.update(|window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.open_note_at(&id, 0, false, window, cx)
        })
    });
    settle(&f, &mut vcx);
    assert_ne!(
        vcx.read(|cx| library.read(cx).filter.clone()),
        crate::library::Filter::Overview
    );
    assert!(vcx.debug_bounds("editor-pane").is_some());
}

/// With "Last note" chosen, the first window opens to the note.
// no-prop-test: drives one window.
#[gpui_kit::test]
fn last_note_keeps_the_old_launch(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    cx.update(|cx| {
        AppState::update_config(cx, |config| {
            config.briefs.open_to = kanso_engine::brief::OpenTo::LastNote
        })
    });
    let library = f.workspace(cx, |w, _| w.library.clone());
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let workspace = f.workspace.clone();
    vcx.update(|window, cx| {
        workspace.update(cx, |workspace, cx| workspace.launch_to_overview(window, cx))
    });
    settle(&f, &mut vcx);
    assert_ne!(
        vcx.read(|cx| library.read(cx).filter.clone()),
        crate::library::Filter::Overview
    );
    assert!(vcx.debug_bounds("editor-pane").is_some());
}

/// A task due today is on Today; ticking it checks it in its note and
/// offers Undo, which unchecks it.
// no-prop-test: drives one window over a real store.
#[gpui_kit::test]
fn a_due_task_is_ticked_from_overview_and_undone(cx: &mut TestAppContext) {
    let f = setup(cx);
    let today = cx.update(|cx| crate::journal::now(cx).date.iso());
    let id = f.notes[0].clone();
    let doc = f.store.get(&id).unwrap().unwrap();
    f.store
        .put(
            DocType::Note,
            json!({"_id": id, "_rev": doc.rev, "body": format!("- [ ] Send the deck @{today}\n")}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let library = f.workspace(cx, |w, _| w.library.clone());
    library.update(cx, |library, cx| {
        library.set_filter(crate::library::Filter::Overview, cx)
    });
    let digest = f.workspace(cx, |w, _| w.digest.clone());
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    wait_until(&f, &mut vcx, |vcx| {
        vcx.read(|cx| !digest.read(cx).agenda_titles_for_test().is_empty())
    });
    assert_eq!(
        vcx.read(|cx| digest.read(cx).agenda_titles_for_test()),
        ["Send the deck"]
    );
    vcx.update(|_, cx| digest.update(cx, |d, cx| d.tick_first_for_test(cx)));
    let body = f.store.note(&id).unwrap().unwrap().body;
    assert!(body.contains("- [x] Send the deck"), "{body}");
    assert!(vcx.read(|cx| digest.read(cx).can_undo_for_test()));
    vcx.update(|_, cx| digest.update(cx, |d, cx| d.undo_for_test(cx)));
    let body = f.store.note(&id).unwrap().unwrap().body;
    assert!(body.contains("- [ ] Send the deck"), "{body}");
}

/// A canvas note in the fixture's notebook.
fn make_canvas(f: &Fixture, title: &str) -> String {
    let book = f.store.note(&f.notes[0]).unwrap().unwrap().book_id;
    f.store
        .put(
            DocType::Note,
            json!({
                "title": title,
                "doctype": "canvas",
                "bookId": book,
                "body": kanso_core::canvas::to_body(&kanso_core::canvas::Canvas::default()),
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn live_strokes(store: &Store, id: &str) -> usize {
    let body = store.note(id).unwrap().unwrap().body;
    kanso_core::canvas::parse(&body)
        .map(|c| c.live_strokes().count())
        .unwrap_or(0)
}

/// Drags the left button across the canvas surface in steps, then lets go.
fn stroke_across(
    f: &Fixture,
    vcx: &mut gpui_kit::VisualTestContext,
    from: (f32, f32),
    to: (f32, f32),
) {
    use gpui_kit::{Modifiers, MouseButton, point, px};
    let surface = vcx
        .debug_bounds("canvas-surface")
        .expect("the canvas shows");
    let at = |t: f32| {
        point(
            surface.left() + px(from.0 + (to.0 - from.0) * t),
            surface.top() + px(from.1 + (to.1 - from.1) * t),
        )
    };
    vcx.simulate_mouse_down(at(0.), MouseButton::Left, Modifiers::none());
    for step in 1..=8 {
        vcx.simulate_mouse_move(at(step as f32 / 8.), MouseButton::Left, Modifiers::none());
        f.draw(vcx);
    }
    vcx.simulate_mouse_up(at(1.), MouseButton::Left, Modifiers::none());
    f.draw(vcx);
}

/// A canvas opens as a surface to draw on, not as text; a stroke drawn with
/// the pointer is saved to the note at once, as compact JSON the text
/// surfaces never see; and the stroke eraser takes it out again.
// no-prop-test: drives one window with the pointer; `canvas_state` has the
// properties.
#[gpui_kit::test]
fn a_canvas_is_drawn_on_and_saves_its_strokes(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = make_canvas(&f, "Board");
    f.select(cx, &id);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    settle(&f, &mut vcx);
    assert!(f.editor(&mut vcx, |e, _| e.is_canvas_shown()));
    assert!(vcx.debug_bounds("canvas-surface").is_some());
    assert_eq!(f.editor(&mut vcx, |e, cx| e.text_for_test(cx)), "");

    stroke_across(&f, &mut vcx, (120., 140.), (360., 200.));
    assert_eq!(live_strokes(&f.store, &id), 1, "the stroke was saved");
    // The text editor holds nothing, and saved nothing over the canvas.
    assert_eq!(f.editor(&mut vcx, |e, cx| e.text_for_test(cx)), "");
    assert!(kanso_core::canvas::is_canvas(
        &f.store.note(&id).unwrap().unwrap().body
    ));

    // Switch to the stroke eraser and cross the line.
    let canvas = f.editor(&mut vcx, |e, _| e.canvas_view().clone());
    canvas.update(&mut vcx, |view, cx| {
        view.set_tool(crate::canvas_state::Tool::Eraser, cx)
    });
    stroke_across(&f, &mut vcx, (240., 100.), (240., 260.));
    settle(&f, &mut vcx);
    assert_eq!(live_strokes(&f.store, &id), 0, "the stroke was erased");

    // Undo brings it back, and is saved too.
    canvas.update(&mut vcx, |view, cx| view.undo(cx));
    settle(&f, &mut vcx);
    assert_eq!(live_strokes(&f.store, &id), 1);
}

/// Handwriting read in the background (`ink_ocr.rs`) lands in the canvas
/// that is open: the canvas takes the words in place, so its undo steps
/// stay, nothing asks which version to keep, and the next edit is saved on
/// top of the words rather than over them, whichever reaches the editor
/// first, the words or the edit.
// no-prop-test: drives one window; `editor_ink` and the engine have the
// properties.
#[gpui_kit::test]
fn handwriting_read_in_the_background_keeps_the_open_canvas(cx: &mut TestAppContext) {
    use kanso_engine::discovery::ink_ocr::{self, Pass};
    use kanso_engine::discovery::ocr::TextReader;
    struct Words;
    impl TextReader for Words {
        fn engine(&self) -> &'static str {
            "fake"
        }
        fn read(&self, _: &[u8]) -> Result<String, String> {
            Ok("groceries".into())
        }
    }
    let f = setup(cx);
    let id = make_canvas(&f, "Board");
    f.select(cx, &id);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    settle(&f, &mut vcx);
    stroke_across(&f, &mut vcx, (120., 140.), (360., 200.));
    settle(&f, &mut vcx);
    let canvas = f.editor(&mut vcx, |e, _| e.canvas_view().clone());
    let words = |store: &Store| {
        kanso_core::canvas::parse(&store.note(&id).unwrap().unwrap().body)
            .unwrap()
            .kanso_ink_ocr
            .len()
    };

    let mut pass = Pass::new(std::time::Duration::ZERO);
    assert_eq!(ink_ocr::read_next(&f.store, &Words, &mut pass).unwrap(), 1);
    assert_eq!(words(&f.store), 1);
    settle(&f, &mut vcx);
    assert!(!f.editor(&mut vcx, |e, _| e.has_incoming()));
    assert!(
        canvas.update(&mut vcx, |view, _| view
            .state_for_test()
            .unwrap()
            .can_undo()),
        "the drawing's undo steps stay"
    );
    assert_eq!(
        canvas.update(&mut vcx, |view, _| view
            .state_for_test()
            .unwrap()
            .canvas
            .kanso_ink_ocr
            .len()),
        1
    );

    // An edit right behind the words: saved on top of them.
    stroke_across(&f, &mut vcx, (100., 300.), (300., 320.));
    settle(&f, &mut vcx);
    let mut again = Pass::new(std::time::Duration::ZERO);
    assert!(ink_ocr::read_next(&f.store, &Words, &mut again).unwrap() > 0);
    canvas.update(&mut vcx, |view, cx| view.undo(cx));
    settle(&f, &mut vcx);
    assert!(!f.editor(&mut vcx, |e, _| e.has_incoming()));
    assert_eq!(live_strokes(&f.store, &id), 1, "the undo was saved");
    assert!(words(&f.store) >= 1, "and the words with it");
}

/// A card dragged slowly at a high zoom follows the pointer (each step is a
/// fraction of a canvas point, which rounding step by step used to drop),
/// and is saved once, when it is let go; a lasso that only selects saves
/// nothing.
// no-prop-test: drives one window with the pointer.
#[gpui_kit::test]
fn a_card_follows_a_slow_drag_and_saves_once(cx: &mut TestAppContext) {
    use gpui_kit::{Modifiers, MouseButton, point, px};
    let f = setup(cx);
    let id = make_canvas(&f, "Board");
    f.select(cx, &id);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    settle(&f, &mut vcx);
    let canvas = f.editor(&mut vcx, |e, _| e.canvas_view().clone());
    let card = canvas.update(&mut vcx, |view, cx| {
        let state = view.state_for_test().unwrap();
        let card = state.add_text_card(0., 0., "Plan", 1);
        state.view = crate::graph::Viewport {
            center: [130., 60.],
            zoom: 4.,
        };
        state.tool = crate::canvas_state::Tool::Lasso;
        cx.notify();
        card
    });
    settle(&f, &mut vcx);
    let rev = f.store.note(&id).unwrap().unwrap().rev;
    let surface = vcx.debug_bounds("canvas-surface").unwrap();
    let middle = surface.center();
    vcx.simulate_mouse_down(middle, MouseButton::Left, Modifiers::none());
    for step in 1..=12 {
        let at = point(middle.x + px(step as f32), middle.y);
        vcx.simulate_mouse_move(at, MouseButton::Left, Modifiers::none());
        f.draw(&mut vcx);
    }
    assert_eq!(
        f.store.note(&id).unwrap().unwrap().rev,
        rev,
        "nothing is written mid-drag"
    );
    vcx.simulate_mouse_up(
        point(middle.x + px(12.), middle.y),
        MouseButton::Left,
        Modifiers::none(),
    );
    settle(&f, &mut vcx);
    let body = f.store.note(&id).unwrap().unwrap().body;
    let saved = kanso_core::canvas::parse(&body).unwrap();
    assert_eq!(
        saved.node(&card).map(|n| n.x),
        Some(3.),
        "12 points / zoom 4"
    );

    // A lasso drawn round nothing (far from the card) selects nothing and
    // writes nothing.
    canvas.update(&mut vcx, |view, cx| {
        view.state_for_test().unwrap().view.center = [5000., 5000.];
        cx.notify();
    });
    settle(&f, &mut vcx);
    let rev = f.store.note(&id).unwrap().unwrap().rev;
    stroke_across(&f, &mut vcx, (10., 10.), (60., 40.));
    settle(&f, &mut vcx);
    assert_eq!(f.store.note(&id).unwrap().unwrap().rev, rev);
}

/// A card pressed with the lasso is the one Delete removes, but not while
/// its text is being edited: Backspace and Delete in the edit dialog are the
/// field's. Delete is one saved undo step that undo puts back, same ID and
/// place, and saves too. Escape lets the card go.
// no-prop-test: drives one window with the pointer and the keyboard.
#[gpui_kit::test]
fn a_selected_canvas_card_is_deleted_by_the_key_but_not_while_edited(cx: &mut TestAppContext) {
    use gpui_kit::component::WindowExt as _;
    use gpui_kit::{Modifiers, MouseButton, MouseDownEvent, MouseUpEvent};
    let f = setup(cx);
    // The app's keys: the workspace's, and the canvas's Delete, undo, redo
    // and Escape.
    cx.update(|cx| {
        crate::workspace::bind_keys(cx);
        crate::canvas_view::bind_keys(cx);
    });
    let id = make_canvas(&f, "Board");
    f.select(cx, &id);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    settle(&f, &mut vcx);
    let canvas = f.editor(&mut vcx, |e, _| e.canvas_view().clone());
    let card = canvas.update(&mut vcx, |view, cx| {
        let state = view.state_for_test().unwrap();
        let card = state.add_text_card(0., 0., "Plan", 1);
        state.view = crate::graph::Viewport {
            center: [130., 60.],
            zoom: 4.,
        };
        state.tool = crate::canvas_state::Tool::Lasso;
        cx.notify();
        card
    });
    settle(&f, &mut vcx);
    let middle = vcx.debug_bounds("canvas-surface").unwrap().center();
    let press = |vcx: &mut gpui_kit::VisualTestContext, click_count: usize| {
        vcx.simulate_event(MouseDownEvent {
            position: middle,
            modifiers: Modifiers::none(),
            button: MouseButton::Left,
            click_count,
            first_mouse: false,
        });
        vcx.simulate_event(MouseUpEvent {
            position: middle,
            modifiers: Modifiers::none(),
            button: MouseButton::Left,
            click_count,
        });
    };
    let saved = |store: &Store| {
        let body = store.note(&id).unwrap().unwrap().body;
        kanso_core::canvas::parse(&body)
            .unwrap()
            .node(&card)
            .map(|n| (n.x, n.y, n.text.clone()))
    };
    let selected = |vcx: &mut gpui_kit::VisualTestContext| {
        canvas.update(vcx, |view, _| {
            view.state_for_test().unwrap().card_selection.clone()
        })
    };
    let dialog = |vcx: &mut gpui_kit::VisualTestContext| {
        vcx.update(|window, cx| window.has_active_dialog(cx))
    };
    // A short drag selects the card and saves it where it was let go.
    vcx.simulate_mouse_down(middle, MouseButton::Left, Modifiers::none());
    let to = gpui_kit::point(middle.x + gpui_kit::px(4.), middle.y);
    vcx.simulate_mouse_move(to, MouseButton::Left, Modifiers::none());
    vcx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
    settle(&f, &mut vcx);
    assert_eq!(
        selected(&mut vcx),
        Some(card.clone()),
        "the press selects it"
    );
    let before = saved(&f.store).expect("the card was saved");
    assert_eq!(before.0, 1., "4 points / zoom 4");

    // Editing the card: the keys go to the dialog's field.
    press(&mut vcx, 2);
    settle(&f, &mut vcx);
    assert!(dialog(&mut vcx), "a double click edits the card");
    vcx.simulate_keystrokes("backspace delete backspace");
    settle(&f, &mut vcx);
    assert!(
        canvas.update(&mut vcx, |view, _| view
            .state_for_test()
            .unwrap()
            .canvas
            .node(&card)
            .is_some()),
        "the card is still there"
    );
    vcx.simulate_keystrokes("escape");
    settle(&f, &mut vcx);
    assert!(!dialog(&mut vcx));
    assert_eq!(saved(&f.store), Some(before.clone()), "nothing was written");

    // Pressed again and Delete: gone, and saved gone.
    press(&mut vcx, 1);
    settle(&f, &mut vcx);
    vcx.simulate_keystrokes("delete");
    settle(&f, &mut vcx);
    assert_eq!(saved(&f.store), None, "the delete was saved");
    assert_eq!(selected(&mut vcx), None);

    // Undo puts it back where it was, under its ID, and saves that.
    vcx.simulate_keystrokes("cmd-z");
    settle(&f, &mut vcx);
    assert_eq!(saved(&f.store), Some(before.clone()), "the undo was saved");
    vcx.simulate_keystrokes("cmd-shift-z");
    settle(&f, &mut vcx);
    assert_eq!(saved(&f.store), None, "the redo was saved");
    vcx.simulate_keystrokes("cmd-z");
    settle(&f, &mut vcx);

    // Escape lets go of the card: Delete then removes nothing.
    press(&mut vcx, 1);
    settle(&f, &mut vcx);
    assert_eq!(selected(&mut vcx), Some(card.clone()));
    vcx.simulate_keystrokes("escape");
    settle(&f, &mut vcx);
    assert_eq!(selected(&mut vcx), None, "escape lets it go");
    vcx.simulate_keystrokes("delete");
    settle(&f, &mut vcx);
    assert_eq!(saved(&f.store), Some(before));
}

/// Canvases have their own list; All Notes leaves them out; the count of
/// Canvases is the sidebar's.
// no-prop-test: drives one window through filters.
#[gpui_kit::test]
fn canvases_are_listed_apart_from_all_notes(cx: &mut TestAppContext) {
    use crate::library::Filter;
    let f = setup(cx);
    let id = make_canvas(&f, "Board");
    let library = f.workspace(cx, |w, _| w.library.clone());
    let ids_for = |cx: &mut TestAppContext, filter: Filter| {
        library.update(cx, |library, cx| library.set_filter(filter, cx));
        cx.run_until_parked();
        cx.read(|cx| {
            library
                .read(cx)
                .notes
                .iter()
                .map(|n| (n.id.clone(), n.canvas))
                .collect::<Vec<_>>()
        })
    };
    assert_eq!(ids_for(cx, Filter::Canvases), [(id.clone(), true)]);
    let all = ids_for(cx, Filter::All);
    assert!(all.iter().all(|(_, canvas)| !canvas), "{all:?}");
    assert_eq!(all.len(), 2);
    assert_eq!(library.read_with(cx, |l, _| l.counts.canvases), 1);
    assert_eq!(library.read_with(cx, |l, _| l.counts.all), 3);
}

/// A note dragged onto a canvas becomes a card with its title; a trashed
/// note, or the canvas itself, does not.
// no-prop-test: drives one window.
#[gpui_kit::test]
fn notes_dropped_on_a_canvas_become_cards(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = make_canvas(&f, "Board");
    f.store
        .put(
            DocType::Note,
            json!({"_id": f.notes[1], "bookId": kanso_core::model::TRASH_BOOK_ID}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    f.select(cx, &id);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    settle(&f, &mut vcx);
    let canvas = f.editor(&mut vcx, |e, _| e.canvas_view().clone());
    let ids = vec![f.notes[0].clone(), f.notes[1].clone(), id.clone()];
    canvas.update(&mut vcx, |view, cx| view.drop_notes(&ids, cx));
    settle(&f, &mut vcx);
    let saved = kanso_core::canvas::parse(&f.store.note(&id).unwrap().unwrap().body).unwrap();
    let cards: Vec<_> = saved
        .nodes
        .iter()
        .filter(|n| n.kind == "file")
        .map(|n| (n.file.clone().unwrap(), n.title.clone().unwrap()))
        .collect();
    assert_eq!(cards, [(f.notes[0].clone(), "a".to_owned())]);
}

/// The notices this test puts in front of the app: two that always apply.
fn test_notices() -> &'static [kanso_core::notices::Notice] {
    use kanso_core::notices::{Notice, NoticeAction, NoticePage, Platforms};
    Box::leak(Box::new([
        Notice {
            id: "fixture-first-0.9",
            since: semver::Version::new(0, 9, 0),
            title: "First fixture",
            what_changed: "Something changed.",
            steps: &["Do this.", "Then that."],
            action: Some(NoticeAction::OpenSettings(NoticePage::LocalApi)),
            applies: |_| true,
            platforms: Platforms::Desktop,
        },
        Notice {
            id: "fixture-second-0.9",
            since: semver::Version::new(0, 9, 0),
            title: "Second fixture",
            what_changed: "Something else changed.",
            steps: &["Do it."],
            action: None,
            applies: |_| true,
            platforms: Platforms::Both,
        },
    ]))
}

/// An update notice shows as a banner across the window, counts the ones
/// waiting, and Dismiss hides it for good: the next one shows, the choice is
/// in `config.json`, and the palette still lists it. How to fix opens the
/// steps.
// no-prop-test: drives one window; the rules are properties in kanso-core.
#[gpui_kit::test]
fn an_update_notice_shows_until_it_is_dismissed(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    // Nothing is listed yet, so there is no banner.
    assert_eq!(vcx.debug_bounds("notice-banner"), None);
    assert!(vcx.read(crate::notices::banner).is_none());

    vcx.update(|_, cx| {
        cx.set_global(crate::notices::NoticeBoard {
            list: test_notices(),
            previous: None,
        });
        cx.refresh_windows();
    });
    f.draw(&mut vcx);
    assert!(vcx.debug_bounds("notice-banner").is_some());
    let pick = vcx.read(crate::notices::banner).unwrap();
    assert_eq!(pick.notice.id, "fixture-first-0.9");
    assert_eq!(pick.total, 2);
    assert!(vcx.debug_bounds("notice-count").is_some());
    assert!(vcx.debug_bounds("notice-action").is_some());

    let click = |vcx: &mut gpui_kit::VisualTestContext, selector: &'static str| {
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        vcx.simulate_click(bounds.center(), gpui_kit::Modifiers::none());
    };

    // How to fix: the sheet has what changed and the steps.
    click(&mut vcx, "notice-how-to-fix");
    f.draw(&mut vcx);
    assert!(vcx.debug_bounds("notice-what-changed").is_some());
    assert!(vcx.debug_bounds("notice-steps").is_some());
    vcx.simulate_keystrokes("escape");
    f.draw(&mut vcx);
    assert_eq!(vcx.debug_bounds("notice-steps"), None);

    // Dismiss: the next notice takes the banner, alone now.
    click(&mut vcx, "notice-dismiss");
    f.draw(&mut vcx);
    let pick = vcx.read(crate::notices::banner).unwrap();
    assert_eq!(pick.notice.id, "fixture-second-0.9");
    assert_eq!(pick.total, 1);
    assert_eq!(vcx.debug_bounds("notice-count"), None);
    assert_eq!(f.saved_config().notices.dismissed, ["fixture-first-0.9"]);
    // The palette still lists the dismissed one.
    assert_eq!(vcx.read(crate::notices::listed).len(), 2);

    // The last one: no banner, and the palette keeps both until fixed.
    click(&mut vcx, "notice-dismiss");
    f.draw(&mut vcx);
    assert_eq!(vcx.debug_bounds("notice-banner"), None);
    assert_eq!(vcx.read(crate::notices::listed).len(), 2);
    vcx.update(|_, cx| crate::notices::mark_fixed("fixture-first-0.9", cx));
    assert_eq!(vcx.read(crate::notices::listed).len(), 1);
    let saved = f.saved_config().notices;
    assert_eq!(saved.fixed, ["fixture-first-0.9"]);
    assert_eq!(saved.dismissed.len(), 2);
}

/// How to fix's own buttons: Mark as fixed puts the notice away (banner and
/// palette), and the sheet opened again has Done instead of Dismiss, so it
/// can still be closed with the mouse. On macOS the banner leaves the
/// traffic lights their room, zoomed out too.
// no-prop-test: drives one window; the rules are properties in kanso-core.
#[gpui_kit::test]
fn how_to_fix_marks_a_notice_fixed_and_can_be_closed(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.update(|_, cx| {
        cx.set_global(crate::notices::NoticeBoard {
            list: test_notices(),
            previous: None,
        });
        cx.refresh_windows();
    });
    f.draw(&mut vcx);
    if crate::layout::LIGHTS_IN_BAND {
        let lights_end = crate::layout::TRAFFIC_LIGHTS.0 + 2. * 20. + 14.;
        let title = vcx
            .debug_bounds("notice-title")
            .expect("the banner is drawn");
        assert!(f32::from(title.left()) > lights_end, "{title:?}");
        // Zoomed out, the lights stay their size, and so does their room.
        vcx.update(|_, cx| crate::zoom::set(75, cx));
        f.draw(&mut vcx);
        let title = vcx
            .debug_bounds("notice-title")
            .expect("the banner is drawn");
        assert!(f32::from(title.left()) > lights_end, "{title:?}");
        let banner = vcx.debug_bounds("notice-banner").unwrap();
        assert!(f32::from(banner.size.height) >= 36., "{banner:?}");
        vcx.update(|_, cx| crate::zoom::set(crate::zoom::DEFAULT_ZOOM, cx));
        f.draw(&mut vcx);
    }
    let click = |vcx: &mut gpui_kit::VisualTestContext, selector: &'static str| {
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        vcx.simulate_click(bounds.center(), gpui_kit::Modifiers::none());
    };

    click(&mut vcx, "notice-how-to-fix");
    f.draw(&mut vcx);
    assert!(vcx.debug_bounds("notice-sheet-dismiss").is_some());
    assert_eq!(vcx.debug_bounds("notice-sheet-done"), None);
    click(&mut vcx, "notice-sheet-fixed");
    f.draw(&mut vcx);
    assert_eq!(vcx.debug_bounds("notice-steps"), None);
    assert_eq!(f.saved_config().notices.fixed, ["fixture-first-0.9"]);
    let pick = vcx.read(crate::notices::banner).unwrap();
    assert_eq!(pick.notice.id, "fixture-second-0.9");
    assert_eq!(vcx.read(crate::notices::listed).len(), 1);

    // Opened again: no Dismiss, no Mark as fixed, but Done.
    vcx.update(|window, cx| {
        let notice = crate::notices::find("fixture-first-0.9", cx).unwrap();
        crate::notice_views::open_how_to_fix(notice, window, cx);
    });
    f.draw(&mut vcx);
    assert!(vcx.debug_bounds("notice-steps").is_some());
    assert_eq!(vcx.debug_bounds("notice-sheet-dismiss"), None);
    assert_eq!(vcx.debug_bounds("notice-sheet-fixed"), None);
    click(&mut vcx, "notice-sheet-done");
    f.draw(&mut vcx);
    assert_eq!(vcx.debug_bounds("notice-steps"), None);
}

/// The API keys notice shows only for a library whose config recorded a
/// login, and Dismiss keeps the choice.
// no-prop-test: drives one window; `prop_the_api_keys_notice_is_for_libraries_that_had_a_login`
// in kanso-core holds the rule.
#[gpui_kit::test]
fn the_api_keys_notice_shows_only_after_a_login_existed(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let mut notice = kanso_core::notices::NOTICES
        .iter()
        .find(|notice| notice.id == kanso_core::notices::API_KEYS_ID)
        .unwrap()
        .clone();
    // As if the release had been reached.
    notice.since = semver::Version::new(0, 9, 0);
    let list: &'static [kanso_core::notices::Notice] = Box::leak(Box::new([notice]));
    vcx.update(|_, cx| {
        cx.set_global(crate::notices::NoticeBoard {
            list,
            previous: None,
        });
        cx.refresh_windows();
    });
    f.draw(&mut vcx);
    // No login was ever recorded: nothing to say.
    assert!(vcx.read(crate::notices::banner).is_none());
    assert_eq!(vcx.debug_bounds("notice-banner"), None);

    vcx.update(|_, cx| {
        crate::workspace::AppState::update_config(cx, |config| config.had_login = true);
        cx.refresh_windows();
    });
    f.draw(&mut vcx);
    let pick = vcx.read(crate::notices::banner).unwrap();
    assert_eq!(pick.notice.id, kanso_core::notices::API_KEYS_ID);
    assert!(vcx.debug_bounds("notice-banner").is_some());
    assert!(f.saved_config().had_login);

    let bounds = vcx.debug_bounds("notice-dismiss").unwrap();
    vcx.simulate_click(bounds.center(), gpui_kit::Modifiers::none());
    f.draw(&mut vcx);
    assert_eq!(vcx.debug_bounds("notice-banner"), None);
    assert_eq!(
        f.saved_config().notices.dismissed,
        [kanso_core::notices::API_KEYS_ID]
    );
}
