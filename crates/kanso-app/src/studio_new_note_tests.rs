//! New notes made while a Video studio is shown land in it (issue #513).

use std::sync::Arc;

use gpui_kit::{
    AnyWindowHandle, AppContext as _, Entity, Modifiers, TestAppContext, VisualTestContext,
};
use kanso_core::roles::Role;
use kanso_store::{Origin, Store};

use crate::config::AppConfig;
use crate::library::Filter;
use crate::services::Services;
use crate::workspace::{AppState, NewNote, NewNotebook, Workspace};

struct Rig {
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
    store: Arc<Store>,
    _dir: tempfile::TempDir,
}

fn rig(cx: &mut TestAppContext) -> Rig {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let state = store.clone();
    let data_dir = dir.path().to_path_buf();
    let (window, workspace) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store: state,
            data_dir,
            config: AppConfig::default(),
        });
        cx.set_global(Services::default());
        crate::workspace::bind_keys(cx);
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    Rig {
        window,
        workspace,
        store,
        _dir: dir,
    }
}

impl Rig {
    fn draw(&self, cx: &mut TestAppContext) {
        self.window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
    }

    fn click(&self, cx: &mut TestAppContext, selector: &'static str) {
        self.draw(cx);
        let mut vcx = VisualTestContext::from_window(self.window, cx);
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        vcx.simulate_click(bounds.center(), Modifiers::none());
        cx.run_until_parked();
    }

    /// Makes a Video studio through the New Notebook sheet.
    fn make_studio(&self, cx: &mut TestAppContext, name: &str) -> String {
        self.window
            .update(cx, |_, window, cx| {
                window.dispatch_action(Box::new(NewNotebook), cx)
            })
            .unwrap();
        cx.run_until_parked();
        // Down from the name to Video studio: it is below the list's fold.
        let studio = kanso_engine::presets::NotebookKind::ALL
            .iter()
            .position(|kind| *kind == kanso_engine::presets::NotebookKind::VideoStudio)
            .expect("Video studio is offered");
        self.draw(cx);
        cx.simulate_keystrokes(self.window, &vec!["down"; studio].join(" "));
        cx.run_until_parked();
        self.draw(cx);
        cx.simulate_input(self.window, name);
        cx.run_until_parked();
        self.click(cx, "notebook-create");
        self.store
            .books()
            .unwrap()
            .into_iter()
            .find(|b| b.name == name)
            .expect("the studio is made")
            .id
    }

    fn new_note(&self, cx: &mut TestAppContext) {
        self.window
            .update(cx, |_, window, cx| {
                window.dispatch_action(Box::new(NewNote), cx)
            })
            .unwrap();
        cx.run_until_parked();
    }

    fn notes_in(&self, book: &str) -> usize {
        self.store.notes_in_books(&[book.to_owned()]).unwrap().len()
    }
}

#[gpui_kit::test]
fn a_new_note_in_a_fresh_studio_lands_in_it(cx: &mut TestAppContext) {
    let rig = rig(cx);
    let studio = rig.make_studio(cx, "Video studio");
    let templates = rig.store.role_book(Role::Templates, Origin::Ui).unwrap();
    assert_eq!(
        cx.read(|cx| rig.workspace.read(cx).library.read(cx).filter.clone()),
        Filter::Book(studio.clone())
    );
    rig.new_note(cx);
    assert_eq!(rig.notes_in(&studio), 1, "the note is in the studio");
    assert_eq!(rig.notes_in(&templates), 2, "only the two templates");
    rig.new_note(cx);
    assert_eq!(rig.notes_in(&studio), 2);
    assert_eq!(rig.notes_in(&templates), 2);
}

/// Away from the studio's own list (All notes, here), the note goes to the
/// first notebook: which was the Templates notebook the studio brought.
#[gpui_kit::test]
fn a_new_note_outside_any_notebook_is_not_a_template(cx: &mut TestAppContext) {
    let rig = rig(cx);
    let studio = rig.make_studio(cx, "Video studio");
    let templates = rig.store.role_book(Role::Templates, Origin::Ui).unwrap();
    let library = cx.read(|cx| rig.workspace.read(cx).library.clone());
    cx.update(|cx| library.update(cx, |library, cx| library.set_filter(Filter::All, cx)));
    cx.run_until_parked();
    rig.new_note(cx);
    assert_eq!(rig.notes_in(&templates), 2, "only the two templates");
    assert_eq!(rig.notes_in(&studio), 1, "the note is in the studio");
}

#[gpui_kit::test]
fn a_new_note_lands_in_a_studio_made_among_other_notebooks(cx: &mut TestAppContext) {
    let rig = rig(cx);
    let put = |doc_type, value| {
        rig.store
            .put(doc_type, value, kanso_store::Put::Upsert, Origin::Ui)
            .unwrap()
            .id
    };
    let work = put(
        kanso_core::DocType::Book,
        serde_json::json!({"name": "Work"}),
    );
    put(
        kanso_core::DocType::Note,
        serde_json::json!({"title": "Plan", "body": "x", "bookId": work}),
    );
    rig.store.role_book(Role::Templates, Origin::Ui).unwrap();
    cx.update(|cx| {
        let library = rig.workspace.read(cx).library.clone();
        library.update(cx, |library, cx| library.reload(cx));
    });
    cx.run_until_parked();
    let studio = rig.make_studio(cx, "Video studio");
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(30));
    cx.run_until_parked();
    rig.draw(cx);
    assert_eq!(
        cx.read(|cx| rig.workspace.read(cx).library.read(cx).filter.clone()),
        Filter::Book(studio.clone())
    );
    rig.new_note(cx);
    assert_eq!(rig.notes_in(&studio), 1, "the note is in the studio");
    assert_eq!(
        cx.read(|cx| rig.workspace.read(cx).library.read(cx).filter.clone()),
        Filter::Book(studio)
    );
}

#[gpui_kit::test]
fn a_note_from_a_template_in_a_fresh_studio_lands_in_it(cx: &mut TestAppContext) {
    let rig = rig(cx);
    let studio = rig.make_studio(cx, "Video studio");
    let templates = rig.store.role_book(Role::Templates, Origin::Ui).unwrap();
    let (library, editor) = cx.read(|cx| {
        let workspace = rig.workspace.read(cx);
        (workspace.library.clone(), workspace.editor.clone())
    });
    let brief = rig.store.templates().unwrap()[0].0.clone();
    rig.window
        .update(cx, |_, window, cx| {
            crate::journal::new_from_template(&library, &editor, &brief, window, cx)
        })
        .unwrap();
    cx.run_until_parked();
    assert_eq!(rig.notes_in(&studio), 1, "the note is in the studio");
    assert_eq!(rig.notes_in(&templates), 2, "only the two templates");
}
