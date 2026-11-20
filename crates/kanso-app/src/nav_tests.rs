//! Back and forward between notes, and copying a note's link, in a
//! headless window: the shortcuts, the mouse's buttons, where the reader
//! comes back to, and notes trashed since. The history's rules are
//! properties in `nav_history.rs`; these check the window wires them up.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::{
    AnyWindowHandle, AppContext as _, Entity, Modifiers, MouseButton, NavigationDirection,
    TestAppContext, VisualTestContext, point, px,
};
use kanso_core::DocType;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::config::AppConfig;
use crate::editor::NoteEditor;
use crate::services::Services;
use crate::workspace::{AppState, CopyKansoLink, CopyNoteLink, GoBack, GoForward, Workspace};

#[cfg(target_os = "macos")]
const BACK: &str = "cmd-[";
#[cfg(target_os = "macos")]
const FORWARD: &str = "cmd-]";
#[cfg(not(target_os = "macos"))]
const BACK: &str = "alt-left";
#[cfg(not(target_os = "macos"))]
const FORWARD: &str = "alt-right";

struct Fixture {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
    book: String,
}

fn setup(cx: &mut TestAppContext) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let data_dir = dir.path().to_path_buf();
    let app_store = store.clone();
    let (window, workspace) = cx.update(|cx| {
        gpui_kit::init(cx);
        crate::workspace::bind_keys(cx);
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
    window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    Fixture {
        _dir: dir,
        store,
        window,
        workspace,
        book,
    }
}

impl Fixture {
    fn note(&self, title: &str, body: &str) -> String {
        self.store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": self.book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

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

    fn select(&self, cx: &mut TestAppContext, id: &str) {
        let library = cx.read(|cx| self.workspace.read(cx).library.clone());
        let id = id.to_owned();
        library.update(cx, |library, cx| library.select(Some(id), cx));
        cx.executor().advance_clock(Duration::from_millis(700));
        self.draw(cx);
    }

    fn editor(&self, cx: &mut TestAppContext) -> Entity<NoteEditor> {
        cx.read(|cx| self.workspace.read(cx).editor.clone())
    }

    fn with_editor<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut NoteEditor, &mut gpui_kit::Window, &mut gpui_kit::Context<NoteEditor>) -> R,
    ) -> R {
        let editor = self.editor(cx);
        let out = self
            .window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| f(editor, window, cx))
            })
            .unwrap();
        cx.run_until_parked();
        out
    }

    fn open_note(&self, cx: &mut TestAppContext) -> Option<String> {
        let editor = self.editor(cx);
        cx.read(|cx| editor.read(cx).note_id().map(str::to_owned))
    }

    fn keys(&self, cx: &mut TestAppContext, keys: &str) {
        cx.simulate_keystrokes(self.window, keys);
        self.draw(cx);
    }

    fn clipboard(&self, cx: &mut TestAppContext) -> Option<String> {
        cx.read_from_clipboard().and_then(|item| item.text())
    }
}

/// ⌘[ and ⌘] (Alt+arrows off macOS) walk the notes opened, and opening
/// another after going back drops the ones ahead.
// no-prop-test: drives one window through keystrokes; the history's rules
// are properties in nav_history.rs.
#[gpui_kit::test]
fn the_shortcuts_go_back_and_forward(cx: &mut TestAppContext) {
    let f = setup(cx);
    let [a, b, c] = ["A", "B", "C"].map(|title| f.note(title, "text"));
    for id in [&a, &b, &c] {
        f.select(cx, id);
    }
    f.keys(cx, BACK);
    assert_eq!(f.open_note(cx), Some(b.clone()));
    f.keys(cx, BACK);
    assert_eq!(f.open_note(cx), Some(a.clone()));
    f.keys(cx, BACK);
    assert_eq!(f.open_note(cx), Some(a.clone()), "nothing further back");
    f.keys(cx, FORWARD);
    assert_eq!(f.open_note(cx), Some(b.clone()));

    // Opening another note from B drops C, which was ahead.
    let d = f.note("D", "text");
    f.select(cx, &d);
    f.keys(cx, FORWARD);
    assert_eq!(f.open_note(cx), Some(d.clone()));
    f.keys(cx, BACK);
    assert_eq!(f.open_note(cx), Some(b));
}

/// Go back and Go forward are available (and so enabled in the menu and
/// palette) only where they would land.
// no-prop-test: three states of one window; reachability is a property in
// nav_history.rs.
#[gpui_kit::test]
fn going_back_and_forward_is_available_only_where_it_lands(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    let b = f.note("B", "text");
    let available = |cx: &mut TestAppContext| {
        f.draw(cx);
        f.window
            .update(cx, |_, window, cx| {
                (
                    window.is_action_available(&GoBack, cx),
                    window.is_action_available(&GoForward, cx),
                )
            })
            .unwrap()
    };
    f.select(cx, &a);
    assert_eq!(available(cx), (false, false));
    f.select(cx, &b);
    assert_eq!(available(cx), (true, false));
    f.keys(cx, BACK);
    assert_eq!(available(cx), (false, true));
}

/// In the note's text, ⌘[ goes back rather than outdenting the line.
// no-prop-test: one keystroke in a focused editor.
#[gpui_kit::test]
fn the_shortcut_goes_back_from_inside_the_text(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "first");
    let b = f.note("B", "    indented");
    f.select(cx, &a);
    f.select(cx, &b);
    f.with_editor(cx, |editor, window, cx| editor.focus_body(window, cx));
    f.draw(cx);
    f.keys(cx, BACK);
    assert_eq!(f.open_note(cx), Some(a));
    assert_eq!(f.store.note(&b).unwrap().unwrap().body, "    indented");
}

/// The mouse's back and forward buttons do what the shortcuts do.
// no-prop-test: two presses in one window.
#[gpui_kit::test]
fn the_mouse_buttons_go_back_and_forward(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    let b = f.note("B", "text");
    f.select(cx, &a);
    f.select(cx, &b);
    let mut vcx = VisualTestContext::from_window(f.window, cx);
    let press = |vcx: &mut VisualTestContext, direction| {
        vcx.simulate_mouse_down(
            point(px(600.), px(300.)),
            MouseButton::Navigate(direction),
            Modifiers::none(),
        );
        vcx.run_until_parked();
    };
    press(&mut vcx, NavigationDirection::Back);
    assert_eq!(f.open_note(&mut vcx), Some(a));
    press(&mut vcx, NavigationDirection::Forward);
    assert_eq!(f.open_note(&mut vcx), Some(b));
}

/// Going back skips a note trashed or deleted since it was opened.
// no-prop-test: one window; which visits are skipped is a property in
// nav_history.rs.
#[gpui_kit::test]
fn going_back_skips_trashed_and_deleted_notes(cx: &mut TestAppContext) {
    let f = setup(cx);
    let [a, b, c, d] = ["A", "B", "C", "D"].map(|title| f.note(title, "text"));
    for id in [&a, &b, &c, &d] {
        f.select(cx, id);
    }
    f.store
        .put(
            DocType::Note,
            json!({"_id": c, "bookId": TRASH_BOOK_ID}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
    f.store.delete(&b, Origin::Api).unwrap();
    cx.run_until_parked();
    f.keys(cx, BACK);
    assert_eq!(f.open_note(cx), Some(a));
}

/// Following a link is a visit too: going back returns to the note the
/// link was in.
// no-prop-test: one link followed in one window.
#[gpui_kit::test]
fn following_a_link_can_be_gone_back_from(cx: &mut TestAppContext) {
    let f = setup(cx);
    let target = f.note("Target", "text");
    let from = f.note("From", "See [[Target]].");
    f.select(cx, &from);
    let workspace = f.workspace.clone();
    let link = target.clone();
    f.window
        .update(cx, |_, window, cx| {
            workspace.update(cx, |w, cx| w.follow_link(link, window, cx))
        })
        .unwrap();
    f.draw(cx);
    assert_eq!(f.open_note(cx), Some(target));
    f.keys(cx, BACK);
    assert_eq!(f.open_note(cx), Some(from));
}

/// Going back puts the caret and the scroll back where they were.
// no-prop-test: one place kept and restored in one window.
#[gpui_kit::test]
fn going_back_returns_to_where_the_reader_was(cx: &mut TestAppContext) {
    let f = setup(cx);
    let long = (0..300)
        .map(|n| format!("line {n}"))
        .collect::<Vec<_>>()
        .join("\n");
    let a = f.note("A", &long);
    let b = f.note("B", "short");
    f.select(cx, &a);
    f.with_editor(cx, |editor, window, cx| {
        editor.focus_body(window, cx);
        editor.reveal_line(150, window, cx);
    });
    f.draw(cx);
    let left = f
        .with_editor(cx, |editor, _, cx| editor.place(cx))
        .unwrap()
        .1;
    assert_eq!(left.caret_line, 150);
    assert!(left.scroll_y < 0., "scrolled down to line 150");

    f.select(cx, &b);
    f.keys(cx, BACK);
    assert_eq!(f.open_note(cx), Some(a));
    let back = f
        .with_editor(cx, |editor, _, cx| editor.place(cx))
        .unwrap()
        .1;
    assert_eq!(back.caret_line, 150);
    assert_eq!(back.scroll_y, left.scroll_y);
}

/// Copy note link copies the `[[Title]]` link, Copy Kanso link the link by
/// ID, and a title an older note has copies the Kanso link instead.
// no-prop-test: the links' forms are properties in note_link.rs.
#[gpui_kit::test]
fn copying_a_notes_link(cx: &mut TestAppContext) {
    let f = setup(cx);
    let plans = f.note("Plans", "text");
    f.select(cx, &plans);
    cx.dispatch_action(f.window, CopyNoteLink);
    f.draw(cx);
    assert_eq!(f.clipboard(cx).as_deref(), Some("[[Plans]]"));

    cx.dispatch_action(f.window, CopyKansoLink);
    f.draw(cx);
    let suffix = plans.strip_prefix("note:").unwrap();
    assert_eq!(
        f.clipboard(cx),
        Some(format!("[Plans](kanso://note/{suffix})"))
    );

    let again = f.note("Plans", "another");
    f.select(cx, &again);
    cx.dispatch_action(f.window, CopyNoteLink);
    f.draw(cx);
    let suffix = again.strip_prefix("note:").unwrap();
    assert_eq!(
        f.clipboard(cx),
        Some(format!("[Plans](kanso://note/{suffix})"))
    );
}
