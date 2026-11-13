//! New Notebook as the person uses it (issue #390, slice 29): the choice of
//! a blank notebook or a Video studio, typing a name, and Create.

use std::sync::Arc;

use gpui_kit::{
    AnyWindowHandle, AppContext as _, Entity, Modifiers, TestAppContext, VisualTestContext,
};
use kanso_core::schema::PropertyType;
use kanso_engine::presets::{NotebookKind, STUDIO_STAGES};
use kanso_store::Store;

use crate::config::AppConfig;
use crate::library::Filter;
use crate::new_notebook_sheet::NewNotebookSheet;
use crate::services::Services;
use crate::sidebar::SidebarEvent;
use crate::workspace::{AppState, NewNotebook, Workspace};

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

    fn drawn(&self, cx: &mut TestAppContext, selector: &'static str) -> bool {
        self.draw(cx);
        let mut vcx = VisualTestContext::from_window(self.window, cx);
        vcx.debug_bounds(selector).is_some()
    }

    /// Clicks the middle of what is drawn with `selector`.
    fn click(&self, cx: &mut TestAppContext, selector: &'static str) {
        self.draw(cx);
        let mut vcx = VisualTestContext::from_window(self.window, cx);
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        vcx.simulate_click(bounds.center(), Modifiers::none());
        cx.run_until_parked();
    }

    /// Types into what has the keyboard.
    fn type_text(&self, cx: &mut TestAppContext, text: &str) {
        self.draw(cx);
        cx.simulate_input(self.window, text);
        cx.run_until_parked();
    }

    fn open_from_menu(&self, cx: &mut TestAppContext) {
        self.window
            .update(cx, |_, window, cx| {
                window.dispatch_action(Box::new(NewNotebook), cx)
            })
            .unwrap();
        cx.run_until_parked();
    }

    /// Opens the sheet directly, for tests that read its state.
    fn open(&self, cx: &mut TestAppContext, parent: Option<String>) -> Entity<NewNotebookSheet> {
        let library = cx.read(|cx| self.workspace.read(cx).library.clone());
        let sheet = self
            .window
            .update(cx, |_, window, cx| {
                crate::new_notebook_sheet::open(library, parent, window, cx)
            })
            .unwrap();
        cx.run_until_parked();
        sheet
    }

    fn filter(&self, cx: &mut TestAppContext) -> Filter {
        cx.read(|cx| self.workspace.read(cx).library.read(cx).filter.clone())
    }

    fn book_named(&self, name: &str) -> Option<kanso_core::Book> {
        self.store
            .books()
            .unwrap()
            .into_iter()
            .find(|b| b.name == name)
    }
}

#[gpui_kit::test]
fn the_menu_item_offers_blank_and_video_studio(cx: &mut TestAppContext) {
    let rig = rig(cx);
    rig.open_from_menu(cx);
    assert!(
        rig.drawn(cx, "notebook-kind-0"),
        "Blank notebook is offered"
    );
    assert!(rig.drawn(cx, "notebook-kind-1"), "Video studio is offered");
    assert!(rig.drawn(cx, "notebook-name"));
    assert!(!rig.drawn(cx, "notebook-error"));
}

#[gpui_kit::test]
fn create_makes_a_blank_notebook_by_default_and_goes_to_it(cx: &mut TestAppContext) {
    let rig = rig(cx);
    rig.open_from_menu(cx);
    rig.type_text(cx, "Recipes");
    rig.click(cx, "notebook-create");
    let book = rig.book_named("Recipes").expect("the notebook is made");
    assert_eq!(rig.filter(cx), Filter::Book(book.id.clone()));
    assert!(rig.store.notebook_board(&book.id).unwrap().is_none());
    assert!(rig.store.notebook_templates(&book.id).unwrap().is_empty());
    assert!(rig.store.templates().unwrap().is_empty());
    assert!(!rig.drawn(cx, "notebook-kind-0"), "the sheet closed");
}

#[gpui_kit::test]
fn choosing_video_studio_makes_the_board_and_templates(cx: &mut TestAppContext) {
    let rig = rig(cx);
    rig.open_from_menu(cx);
    rig.click(cx, "notebook-kind-1");
    rig.type_text(cx, "Channel");
    rig.click(cx, "notebook-create");
    let book = rig.book_named("Channel").expect("the studio is made");
    assert_eq!(rig.filter(cx), Filter::Book(book.id.clone()));
    let board = rig.store.notebook_board(&book.id).unwrap().unwrap();
    assert_eq!(board.by, "stage");
    assert_eq!(board.lanes, STUDIO_STAGES);
    let keys: Vec<(String, PropertyType)> = rig
        .store
        .notebook_properties(&book.id)
        .unwrap()
        .into_iter()
        .map(|d| (d.key, d.kind))
        .collect();
    assert_eq!(
        keys,
        [
            ("stage".to_owned(), PropertyType::Select),
            ("embargo".to_owned(), PropertyType::DateTimeZone),
            ("publish".to_owned(), PropertyType::DateTime),
            ("sponsor".to_owned(), PropertyType::Text),
            ("owner".to_owned(), PropertyType::Person),
        ]
    );
    let first: Vec<(String, bool)> = rig
        .store
        .templates_for(Some(&book.id))
        .unwrap()
        .into_iter()
        .map(|t| (t.title, t.in_notebook))
        .collect();
    assert_eq!(
        first,
        [
            ("Video brief".to_owned(), true),
            ("Shot list".to_owned(), true)
        ]
    );
}

#[gpui_kit::test]
fn the_choice_can_be_changed_back(cx: &mut TestAppContext) {
    let rig = rig(cx);
    let sheet = rig.open(cx, None);
    assert_eq!(cx.read(|cx| sheet.read(cx).kind), NotebookKind::Blank);
    rig.click(cx, "notebook-kind-1");
    assert_eq!(cx.read(|cx| sheet.read(cx).kind), NotebookKind::VideoStudio);
    rig.click(cx, "notebook-kind-0");
    assert_eq!(cx.read(|cx| sheet.read(cx).kind), NotebookKind::Blank);
    rig.type_text(cx, "Plain");
    rig.click(cx, "notebook-create");
    let book = rig.book_named("Plain").unwrap();
    assert!(rig.store.notebook_board(&book.id).unwrap().is_none());
}

#[gpui_kit::test]
fn create_does_nothing_without_a_name(cx: &mut TestAppContext) {
    let rig = rig(cx);
    let sheet = rig.open(cx, None);
    rig.click(cx, "notebook-kind-1");
    rig.type_text(cx, "   ");
    let target = sheet.clone();
    rig.window
        .update(cx, move |_, window, cx| {
            target.update(cx, |s, cx| s.create(window, cx))
        })
        .unwrap();
    cx.run_until_parked();
    assert!(rig.store.books().unwrap().is_empty());
    assert!(rig.store.templates().unwrap().is_empty());
    assert!(rig.drawn(cx, "notebook-kind-0"), "the sheet stays open");
}

#[gpui_kit::test]
fn the_sidebar_makes_a_studio_inside_a_notebook(cx: &mut TestAppContext) {
    let rig = rig(cx);
    let parent = rig
        .store
        .put(
            kanso_core::DocType::Book,
            serde_json::json!({"name": "Work"}),
            kanso_store::Put::Upsert,
            kanso_store::Origin::Ui,
        )
        .unwrap()
        .id;
    let workspace = rig.workspace.clone();
    let inside = parent.clone();
    rig.window
        .update(cx, move |_, window, cx| {
            workspace.update(cx, |workspace, cx| {
                workspace.sidebar_event_for_test(
                    &SidebarEvent::NewBook {
                        parent: Some(inside),
                    },
                    window,
                    cx,
                )
            })
        })
        .unwrap();
    cx.run_until_parked();
    rig.click(cx, "notebook-kind-1");
    rig.type_text(cx, "Channel");
    rig.click(cx, "notebook-create");
    let book = rig.book_named("Channel").unwrap();
    assert_eq!(book.parent_book_id.as_deref(), Some(parent.as_str()));
    assert!(rig.store.notebook_board(&book.id).unwrap().is_some());
}
