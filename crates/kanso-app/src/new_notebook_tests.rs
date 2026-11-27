//! New Notebook as the person uses it (issues #390 slice 29 and #514): the
//! name, the choice of a blank notebook, a starter or a Video studio, and
//! Create.

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

    /// Presses `key` `times` times with what has the keyboard.
    fn press(&self, cx: &mut TestAppContext, key: &str, times: usize) {
        self.draw(cx);
        cx.simulate_keystrokes(self.window, &vec![key; times].join(" "));
        cx.run_until_parked();
    }

    /// Chooses the first card; the keyboard stays in the name, where Up
    /// and Down move the choice.
    fn first_card(&self, cx: &mut TestAppContext) {
        self.click(cx, "notebook-kind-0");
    }

    /// Chooses starter `ix` with the keyboard: the first, then Down.
    fn choose(&self, cx: &mut TestAppContext, ix: usize) {
        self.first_card(cx);
        self.press(cx, "down", ix);
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
                let sidebar = self.workspace.read(cx).sidebar.clone();
                crate::new_notebook_sheet::open(library, sidebar, parent, window, cx)
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
fn the_menu_item_offers_every_starter_after_the_name(cx: &mut TestAppContext) {
    let rig = rig(cx);
    rig.open_from_menu(cx);
    for ix in 0..NotebookKind::ALL.len() {
        let selector: &'static str = Box::leak(format!("notebook-kind-{ix}").into_boxed_str());
        assert!(rig.drawn(cx, selector), "{selector} is offered");
    }
    assert!(rig.drawn(cx, "notebook-name"));
    assert!(rig.drawn(cx, "notebook-starters"));
    assert!(!rig.drawn(cx, "notebook-error"));
    // The name comes first, above the list.
    let mut vcx = VisualTestContext::from_window(rig.window, cx);
    let name = vcx.debug_bounds("notebook-name").unwrap();
    let list = vcx.debug_bounds("notebook-starters").unwrap();
    assert!(name.bottom() <= list.top());
    // The list scrolls past its largest height.
    assert!(list.size.height <= gpui_kit::px(crate::new_notebook_sheet::LIST_MAX_HEIGHT + 1.));
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
    rig.type_text(cx, "Channel");
    rig.choose(cx, 7);
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
    rig.type_text(cx, "Plain");
    rig.choose(cx, 7);
    assert_eq!(cx.read(|cx| sheet.read(cx).kind), NotebookKind::VideoStudio);
    rig.press(cx, "up", 7);
    assert_eq!(cx.read(|cx| sheet.read(cx).kind), NotebookKind::Blank);
    rig.click(cx, "notebook-create");
    let book = rig.book_named("Plain").unwrap();
    assert!(rig.store.notebook_board(&book.id).unwrap().is_none());
}

#[gpui_kit::test]
fn create_does_nothing_without_a_name(cx: &mut TestAppContext) {
    let rig = rig(cx);
    let sheet = rig.open(cx, None);
    rig.type_text(cx, "   ");
    rig.choose(cx, 7);
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
    rig.type_text(cx, "Channel");
    rig.choose(cx, 7);
    rig.click(cx, "notebook-create");
    let book = rig.book_named("Channel").unwrap();
    assert_eq!(book.parent_book_id.as_deref(), Some(parent.as_str()));
    assert!(rig.store.notebook_board(&book.id).unwrap().is_some());
}

#[gpui_kit::test]
fn a_starter_makes_its_layout_and_goes_to_its_first_note(cx: &mut TestAppContext) {
    let rig = rig(cx);
    rig.open_from_menu(cx);
    rig.type_text(cx, "Acme");
    rig.click(cx, "notebook-kind-1");
    rig.click(cx, "notebook-create");
    let book = rig.book_named("Acme").expect("the notebook is made");
    assert_eq!(crate::book_icons::chosen(&book).as_deref(), Some("rocket"));
    let inside: Vec<String> = rig
        .store
        .books()
        .unwrap()
        .into_iter()
        .filter(|b| b.parent_book_id.as_deref() == Some(book.id.as_str()))
        .map(|b| b.name)
        .collect();
    assert_eq!(inside.len(), 9);
    assert_eq!(rig.store.templates().unwrap().len(), 9);
    assert_eq!(rig.store.tags().unwrap().len(), 6);
    // The root shows, open in the sidebar, with the first note selected.
    assert_eq!(rig.filter(cx), Filter::Book(book.id.clone()));
    let sidebar = cx.read(|cx| rig.workspace.read(cx).sidebar.clone());
    assert!(cx.read(|cx| sidebar.read(cx).is_book_open(&book.id)));
    let selected = cx
        .read(|cx| rig.workspace.read(cx).library.read(cx).selected.clone())
        .expect("a note is selected");
    let note = rig.store.note(&selected).unwrap().unwrap();
    assert_eq!(note.title, "Feature plan");
    let features = rig
        .store
        .books()
        .unwrap()
        .into_iter()
        .find(|b| b.name == "Features")
        .unwrap();
    assert_eq!(note.book_id, features.id);
}

#[gpui_kit::test]
fn up_and_down_move_the_choice_and_stop_at_the_ends(cx: &mut TestAppContext) {
    let rig = rig(cx);
    let sheet = rig.open(cx, None);
    rig.first_card(cx);
    rig.press(cx, "up", 2);
    assert_eq!(cx.read(|cx| sheet.read(cx).kind), NotebookKind::Blank);
    rig.press(cx, "down", 1);
    assert_eq!(cx.read(|cx| sheet.read(cx).kind), NotebookKind::Product);
    rig.press(cx, "down", 20);
    assert_eq!(cx.read(|cx| sheet.read(cx).kind), NotebookKind::VideoStudio);
}

#[gpui_kit::test]
fn a_locked_notebook_only_takes_blank_and_video_studio(cx: &mut TestAppContext) {
    let rig = rig(cx);
    let secret = rig
        .store
        .put(
            kanso_core::DocType::Book,
            serde_json::json!({"name": "Secret"}),
            kanso_store::Put::Upsert,
            kanso_store::Origin::Ui,
        )
        .unwrap()
        .id;
    let mut book = rig.store.get_api(&secret, None, false).unwrap();
    book.as_object_mut().unwrap().remove("count");
    book["kansoLock"] =
        serde_json::json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"});
    rig.store.lock_book(book, Vec::new()).unwrap();
    let library = cx.read(|cx| rig.workspace.read(cx).library.clone());
    library.update(cx, |library, cx| library.reload(cx));
    cx.run_until_parked();

    let sheet = rig.open(cx, Some(secret.clone()));
    rig.type_text(cx, "Channel");
    for ix in 1..=6 {
        let selector: &'static str = Box::leak(format!("notebook-kind-hint-{ix}").into_boxed_str());
        assert!(rig.drawn(cx, selector), "{selector} says why");
    }
    assert!(!rig.drawn(cx, "notebook-kind-hint-0"));
    assert!(!rig.drawn(cx, "notebook-kind-hint-7"));
    // A disabled card doesn't take the choice, by click or by key.
    rig.click(cx, "notebook-kind-1");
    assert_eq!(cx.read(|cx| sheet.read(cx).kind), NotebookKind::Blank);
    rig.first_card(cx);
    rig.press(cx, "down", 3);
    assert_eq!(cx.read(|cx| sheet.read(cx).kind), NotebookKind::VideoStudio);
    rig.click(cx, "notebook-create");
    let studio = rig.book_named("Channel").expect("a studio goes in");
    assert_eq!(studio.parent_book_id.as_deref(), Some(secret.as_str()));
}
