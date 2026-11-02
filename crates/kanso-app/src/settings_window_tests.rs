//! Settings in a real window: opening and leaving them, the page actions,
//! and controls that reach the editor. Nothing here starts the API or
//! touches the Keychain or the login items.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, Entity, FocusHandle, Modifiers, TestAppContext,
    VisualTestContext, point, px,
};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::account_page::SignedOutForm;
use crate::config::{AppConfig, EditorFont};
use crate::library::Filter;
use crate::palette::Mode;
use crate::services::{AccountState, Services, SyncState};
use crate::settings::{
    OpenAccountSettings, OpenEditorSettings, SettingsPage, register_app_actions,
};
use crate::workspace::{AppState, OpenSettings, Workspace, for_each_workspace};

struct Fixture {
    _dir: tempfile::TempDir,
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
    /// The open note.
    note: String,
    /// What had the keyboard while writing it.
    body: FocusHandle,
}

fn globals(cx: &mut App, store: Arc<Store>, data_dir: std::path::PathBuf) {
    gpui_kit::init(cx);
    cx.set_global(AppState {
        store,
        data_dir,
        config: AppConfig::default(),
    });
    cx.set_global(Services::default());
    crate::workspace::bind_keys(cx);
    crate::icon_themes::init(cx);
}

/// A notebook with enough notes to scroll, the last one open with a long
/// body and the keyboard in it.
fn setup(cx: &mut TestAppContext) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let book = store
        .put(
            DocType::Book,
            json!({"name": "Work"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let body: String = (0..300).map(|n| format!("Line {n}\n")).collect();
    let mut note = String::new();
    for n in 0..40 {
        note = store
            .put(
                DocType::Note,
                json!({"title": format!("Note {n}"), "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
    }
    let (app_store, data_dir) = (store.clone(), dir.path().to_path_buf());
    let (window, workspace) = cx.update(|cx| {
        globals(cx, app_store, data_dir);
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    // Active, as the window in front is: focus changes report blurs.
    window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    let library = cx.read(|cx| workspace.read(cx).library.clone());
    let id = note.clone();
    library.update(cx, |library, cx| library.select(Some(id), cx));
    cx.run_until_parked();
    let body = window
        .update(cx, |_, window, cx| {
            let editor = workspace.read(cx).editor.clone();
            editor.update(cx, |editor, cx| editor.focus_body(window, cx));
            window.focused(cx).expect("the body takes the keyboard")
        })
        .unwrap();
    let f = Fixture {
        _dir: dir,
        window,
        workspace,
        note,
        body,
    };
    f.draw(cx);
    f
}

impl Fixture {
    /// Draws, so focus changes reach their listeners and actions find what
    /// is on screen.
    fn draw(&self, cx: &mut TestAppContext) {
        self.window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
    }

    fn keys(&self, cx: &mut TestAppContext, keys: &str) {
        cx.simulate_keystrokes(self.window, keys);
        cx.executor().advance_clock(Duration::from_millis(200));
        cx.run_until_parked();
        self.draw(cx);
    }

    fn open(&self, cx: &mut TestAppContext) -> bool {
        cx.read(|cx| self.workspace.read(cx).settings_open())
    }

    fn page(&self, cx: &mut TestAppContext) -> SettingsPage {
        cx.read(|cx| self.workspace.read(cx).settings.read(cx).page())
    }

    fn focused(&self, cx: &mut TestAppContext, handle: &FocusHandle) -> bool {
        let handle = handle.clone();
        self.window
            .update(cx, |_, window, _| handle.is_focused(window))
            .unwrap()
    }

    /// The top edge of what is drawn with `selector`.
    fn draw_bounds_y(&self, cx: &mut TestAppContext, selector: &'static str) -> f32 {
        self.draw(cx);
        let mut vcx = VisualTestContext::from_window(self.window, cx);
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        f32::from(bounds.origin.y)
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
        self.draw(cx);
    }

    /// Sends `action` from the workspace, as the palette does.
    fn dispatch(&self, cx: &mut TestAppContext, action: Box<dyn gpui_kit::Action>) {
        let focus = cx.read(|cx| self.workspace.read(cx).focus.clone());
        self.window
            .update(cx, |_, window, cx| {
                focus.dispatch_action(action.as_ref(), window, cx)
            })
            .unwrap();
        cx.run_until_parked();
        self.draw(cx);
    }

    fn gear_active(&self, cx: &mut TestAppContext) -> bool {
        cx.read(|cx| {
            self.workspace
                .read(cx)
                .sidebar
                .read(cx)
                .settings_open_for_test()
        })
    }
}

/// ⌘, puts Settings where the notes were, and Escape brings the notes back
/// exactly: the same note, the list and the text scrolled as they were, and
/// the keyboard in the text.
#[gpui_kit::test]
fn the_scheduled_check_runs_with_automatic_updates_off(cx: &mut TestAppContext) {
    let _fixture = setup(cx);
    cx.update(|cx| {
        AppState::update_config(cx, |config| config.gallery.auto_update = false);
        cx.set_global(crate::settings_gallery::ThemeGallery {
            keys: kanso_gallery::trusted_keys(),
            ..Default::default()
        });
        crate::settings_gallery::auto_check(cx);
        assert_ne!(
            cx.global::<crate::settings_gallery::ThemeGallery>().status,
            crate::settings_gallery::Status::Idle,
            "a due check must run even when automatic updates are off"
        );
    });
}

#[gpui_kit::test]
fn escape_returns_to_the_notes_as_they_were(cx: &mut TestAppContext) {
    let f = setup(cx);
    let (list, editor) = cx.read(|cx| {
        let workspace = f.workspace.read(cx);
        (workspace.note_list.clone(), workspace.editor.clone())
    });
    cx.update(|cx| {
        list.read(cx).set_scroll_offset(point(px(0.), px(-400.)));
        editor.update(cx, |editor, cx| {
            editor.scroll_body_for_test(point(px(0.), px(-900.)), cx)
        });
    });
    f.draw(cx);
    let scrolled = |cx: &mut TestAppContext| {
        cx.read(|cx| {
            (
                list.read(cx).scroll_offset(),
                editor.read(cx).body_scroll_for_test(cx),
            )
        })
    };
    let before = scrolled(cx);
    assert!(before.0.y < px(0.) && before.1.y < px(0.), "{before:?}");

    f.keys(cx, "cmd-,");
    assert!(f.open(cx));
    assert!(f.gear_active(cx));
    assert!(!f.focused(cx, &f.body));
    assert_eq!(f.page(cx), SettingsPage::General);
    // The page column takes the arrow keys.
    f.keys(cx, "down down");
    assert_eq!(f.page(cx), SettingsPage::Appearance);

    f.keys(cx, "escape");
    assert!(!f.open(cx));
    assert!(!f.gear_active(cx));
    assert!(f.focused(cx, &f.body), "the keyboard is back in the text");
    let open = cx.read(|cx| editor.read(cx).note_id().map(str::to_owned));
    assert_eq!(open.as_deref(), Some(f.note.as_str()));
    assert_eq!(scrolled(cx), before);

    // ⌘, again shows the page left last; Back to notes goes back too.
    f.keys(cx, "cmd-,");
    assert_eq!(f.page(cx), SettingsPage::Appearance);
    f.click(cx, "settings-back");
    assert!(!f.open(cx));
    assert!(f.focused(cx, &f.body));
}

/// The page column is in the Tab order: Tab goes on into the page's
/// controls, and Shift-Tab comes back to the column, where the arrows
/// still move between pages.
#[gpui_kit::test]
fn tab_comes_back_to_the_page_column(cx: &mut TestAppContext) {
    let f = setup(cx);
    let settings = cx.read(|cx| f.workspace.read(cx).settings.clone());
    let nav_focused = |cx: &mut TestAppContext| {
        f.window
            .update(cx, |_, window, cx| settings.read(cx).nav_focused(window))
            .unwrap()
    };
    f.keys(cx, "cmd-,");
    assert!(f.open(cx));
    assert!(nav_focused(cx), "Settings open with the column's keyboard");
    let page = f.page(cx);

    f.keys(cx, "tab");
    assert!(!nav_focused(cx), "Tab leaves the column");
    f.keys(cx, "tab");
    assert!(!nav_focused(cx));
    f.keys(cx, "shift-tab");
    f.keys(cx, "shift-tab");
    assert!(nav_focused(cx), "Shift-Tab comes back to the column");
    assert!(f.open(cx));
    assert_eq!(f.page(cx), page, "Tab moves the keyboard, not the page");
    f.keys(cx, "down");
    assert_eq!(f.page(cx), page.step(1), "the arrows move between pages");
}

/// Each page has its command, and the palette runs it.
#[gpui_kit::test]
fn the_palette_opens_each_page(cx: &mut TestAppContext) {
    let f = setup(cx);
    for page in SettingsPage::ALL {
        let label = format!("Settings: {}", page.title());
        let command = crate::commands::all()
            .iter()
            .find(|c| c.label == label)
            .unwrap_or_else(|| panic!("a command for {page:?}"));
        f.dispatch(cx, command.action().unwrap());
        assert!(f.open(cx));
        assert_eq!(f.page(cx), page);
    }
    f.keys(cx, "escape");
    assert!(!f.open(cx));

    // Typed into the palette, from the notes.
    f.keys(cx, "cmd-k");
    let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
    f.window
        .update(cx, |_, window, cx| {
            palette.update(cx, |palette, cx| {
                palette.set_query("settings: account", window, cx)
            })
        })
        .unwrap();
    cx.executor().advance_clock(Duration::from_millis(200));
    cx.run_until_parked();
    f.keys(cx, "enter");
    assert_eq!(cx.read(|cx| palette.read(cx).mode()), None);
    assert!(f.open(cx));
    assert_eq!(f.page(cx), SettingsPage::Account);
    // Escape closes Settings, and the keyboard is back where the palette
    // found it.
    f.keys(cx, "escape");
    assert!(!f.open(cx));
    assert!(f.focused(cx, &f.body));
}

/// Asking for notes leaves Settings: a place in the sidebar (even the one
/// already showing), ⌘N, or a note from ⌘P.
#[gpui_kit::test]
fn asking_for_notes_leaves_settings(cx: &mut TestAppContext) {
    let f = setup(cx);
    let library = cx.read(|cx| f.workspace.read(cx).library.clone());
    let filter = cx.read(|cx| library.read(cx).filter.clone());

    f.keys(cx, "cmd-,");
    library.update(cx, |library, cx| library.set_filter(filter.clone(), cx));
    cx.run_until_parked();
    assert!(!f.open(cx), "the place already showing");

    f.keys(cx, "cmd-,");
    library.update(cx, |library, cx| library.set_filter(Filter::Pinned, cx));
    cx.run_until_parked();
    assert!(!f.open(cx), "another place");
    f.draw(cx);

    f.keys(cx, "cmd-,");
    f.keys(cx, "cmd-n");
    assert!(!f.open(cx), "a new note");
    let editor = cx.read(|cx| f.workspace.read(cx).editor.clone());
    let in_title = f
        .window
        .update(cx, |_, window, cx| {
            editor.read(cx).title_focused(window, cx)
        })
        .unwrap();
    assert!(in_title, "typing goes into the new note's title");

    f.keys(cx, "cmd-,");
    f.keys(cx, "cmd-p");
    let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
    assert_eq!(cx.read(|cx| palette.read(cx).mode()), Some(Mode::Notes));
    f.keys(cx, "enter");
    assert!(!f.open(cx), "a note from ⌘P");
}

/// The Editor page's controls change the open editor at once and are kept
/// in the settings file.
#[gpui_kit::test]
fn editor_controls_reach_the_editor(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.dispatch(cx, Box::new(OpenEditorSettings));
    let editor = cx.read(|cx| f.workspace.read(cx).editor.clone());
    f.click(cx, "editor-font-2");
    f.click(cx, "font-larger");
    f.click(cx, "font-larger");
    f.click(cx, "line-numbers");
    let (font, size, numbers) = cx.read(|cx| {
        let editor = editor.read(cx);
        (
            editor.editor_font,
            editor.font_size,
            editor.allow_line_numbers,
        )
    });
    assert_eq!((font, size, numbers), (EditorFont::Mono, 18., false));
    let saved = cx.read(|cx| {
        let state = cx.global::<AppState>();
        AppConfig::load(&crate::paths::config_path(&state.data_dir)).unwrap()
    });
    assert_eq!(saved.appearance.editor_font, EditorFont::Mono);
    assert_eq!(saved.appearance.editor_font_size, 18.);
    assert!(!saved.appearance.line_numbers);
    // A click on a page in the column shows it.
    f.click(cx, "settings-account");
    assert_eq!(f.page(cx), SettingsPage::Account);
}

fn sign_in_as(cx: &mut TestAppContext, username: &str) {
    let username = username.to_owned();
    cx.update(|cx| {
        cx.global_mut::<Services>().account = AccountState::SignedIn { username };
        cx.refresh_windows();
    });
}

/// Signed out there is nothing to sync with: no Sync now on the page, and
/// the action (the menu's Sync Now) isn't available; signed in, both are.
#[gpui_kit::test]
fn sync_now_needs_an_account(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.dispatch(cx, Box::new(OpenAccountSettings));
    assert!(!f.drawn(cx, "sync-now"));
    assert!(f.drawn(cx, "account-sign-in"));
    let available = |cx: &mut TestAppContext| {
        let focus = cx.read(|cx| f.workspace.read(cx).focus.clone());
        f.window
            .update(cx, |_, window, cx| {
                window.focus(&focus, cx);
                window.is_action_available(&crate::workspace::SyncNow, cx)
            })
            .unwrap()
    };
    assert!(!available(cx), "Sync Now is disabled while signed out");
    f.dispatch(cx, Box::new(crate::workspace::SyncNow));
    let sync = cx.read(|cx| cx.global::<Services>().sync.clone());
    assert_eq!(sync, SyncState::Off);
    sign_in_as(cx, "ngoni");
    f.draw(cx);
    assert!(available(cx), "Sync Now works while signed in");
    f.dispatch(cx, Box::new(OpenAccountSettings));
    assert!(f.drawn(cx, "sync-now"));
    assert!(!f.drawn(cx, "account-sign-in"));
}

fn open_test_window(cx: &mut App) {
    gpui_kit::open_window(Default::default(), cx, |window, cx| {
        cx.new(|cx| Workspace::new(window, cx))
    })
    .unwrap();
}

/// Settings from the menu bar with no window open: the main window opens,
/// showing Settings; with it open, it is used rather than another opened.
#[gpui_kit::test]
fn settings_open_a_window_when_none_is_open(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let data_dir = dir.path().to_path_buf();
    cx.update(|cx| {
        globals(cx, store, data_dir);
        register_app_actions(cx, open_test_window);
    });
    assert!(cx.read(|cx| cx.windows().is_empty()));
    cx.update(|cx| cx.dispatch_action(&OpenAccountSettings));
    cx.run_until_parked();
    let shown = |cx: &mut TestAppContext| {
        cx.update(|cx| {
            let mut shown = Vec::new();
            for_each_workspace(cx, |workspace, cx| {
                shown.push(
                    workspace
                        .settings_open()
                        .then(|| workspace.settings.read(cx).page()),
                );
            });
            shown
        })
    };
    assert_eq!(shown(cx), [Some(SettingsPage::Account)]);
    cx.update(|cx| cx.dispatch_action(&OpenSettings));
    cx.run_until_parked();
    assert_eq!(shown(cx), [Some(SettingsPage::Account)]);
}

/// The same with notes, the last one kept from before: the window opens on
/// Settings and stays there, with the last note open behind them. The app
/// picking that note itself doesn't take Settings away.
#[gpui_kit::test]
fn settings_stay_when_the_window_opens_on_a_library(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let mut last = String::new();
    for n in 0..3 {
        last = store
            .put(
                DocType::Note,
                json!({"title": format!("Note {n}"), "body": "x", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
    }
    let data_dir = dir.path().to_path_buf();
    let kept = last.clone();
    cx.update(|cx| {
        globals(cx, store, data_dir);
        cx.global_mut::<AppState>().config.last_note = Some(kept);
        register_app_actions(cx, open_test_window);
    });
    cx.update(|cx| cx.dispatch_action(&OpenAccountSettings));
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    let shown = cx.update(|cx| {
        let mut shown = Vec::new();
        for_each_workspace(cx, |workspace, cx| {
            shown.push((
                workspace
                    .settings_open()
                    .then(|| workspace.settings.read(cx).page()),
                workspace.editor.read(cx).note_id().map(str::to_owned),
            ));
        });
        shown
    });
    assert_eq!(shown, [(Some(SettingsPage::Account), Some(last))]);
}

/// What acts on the open note brings it back from behind Settings first,
/// so the note it changes is in view; the palette over Settings offers no
/// note actions at all.
#[gpui_kit::test]
fn note_actions_leave_settings_first(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.keys(cx, "cmd-,");
    assert!(f.open(cx));
    f.dispatch(cx, Box::new(crate::workspace::TrashNote));
    assert!(!f.open(cx), "Settings closed first");
    let store = cx.read(|cx| cx.global::<AppState>().store.clone());
    let note = store.note(&f.note).unwrap().unwrap();
    assert!(note.is_trashed());
    // Trashing the showing note closes its tab, so what is in view is the
    // next note, or nothing.
    let open = cx.read(|cx| {
        let editor = f.workspace.read(cx).editor.clone();
        editor.read(cx).note_id().map(str::to_owned)
    });
    assert_ne!(open.as_deref(), Some(f.note.as_str()), "its tab closed");

    for action in [
        Box::new(crate::workspace::TogglePinned) as Box<dyn gpui_kit::Action>,
        Box::new(crate::workspace::ShowPreview),
    ] {
        f.keys(cx, "cmd-,");
        f.dispatch(cx, action);
        assert!(!f.open(cx));
    }

    f.keys(cx, "cmd-,");
    f.keys(cx, "cmd-k");
    let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
    let labels = cx.read(|cx| palette.read(cx).item_labels());
    assert!(
        labels.iter().any(|l| l == "Settings: Account"),
        "{labels:?}"
    );
    assert!(labels.iter().any(|l| l == "Back to notes"), "{labels:?}");
    let note_commands: Vec<_> = crate::commands::all()
        .iter()
        .filter(|c| {
            matches!(
                c.needs,
                crate::commands::Needs::Note | crate::commands::Needs::LiveNote
            )
        })
        .map(|c| c.label)
        .collect();
    assert!(!note_commands.is_empty());
    for label in note_commands {
        assert!(!labels.iter().any(|l| l == label), "{label} over Settings");
    }
    f.keys(cx, "escape");
    f.keys(cx, "escape");
    assert!(!f.open(cx));
    // Back with the notes, the palette offers them again. Choose another,
    // since the one trashed above closed its tab.
    let store = cx.read(|cx| cx.global::<AppState>().store.clone());
    let other = store
        .search_summaries(&Default::default())
        .unwrap()
        .into_iter()
        .find(|note| note.id != f.note)
        .unwrap()
        .id;
    let library = cx.read(|cx| f.workspace.read(cx).library.clone());
    library.update(cx, |library, cx| library.select(Some(other), cx));
    f.draw(cx);
    f.keys(cx, "cmd-k");
    let labels = cx.read(|cx| palette.read(cx).item_labels());
    assert!(labels.iter().any(|l| l == "Show history…"), "{labels:?}");
    assert!(!labels.iter().any(|l| l == "Back to notes"), "{labels:?}");
}

/// With other methods advertised, Create account lists them with the
/// username one; that shows the password form in their place, and Cancel
/// there goes back to the list.
#[gpui_kit::test]
fn create_account_offers_the_username_form_as_one_method(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.dispatch(cx, Box::new(OpenAccountSettings));
    f.click(cx, "account-create");
    cx.update(|cx| {
        let services = cx.global_mut::<Services>();
        services.auth = Some((
            kanso_sync::DEFAULT_SERVER.to_owned(),
            kanso_sync::ServerAuth {
                password: true,
                providers: vec![kanso_sync::Provider::Github],
                ..Default::default()
            },
        ));
        services.signup = Some((
            kanso_sync::DEFAULT_SERVER.to_owned(),
            kanso_sync::SignupState::Open,
        ));
    });
    f.draw(cx);
    assert!(f.drawn(cx, "account-provider-github"));
    assert!(f.drawn(cx, "account-username-create"));
    assert!(!f.drawn(cx, "account-username"));
    assert!(!f.drawn(cx, "account-submit"));
    f.click(cx, "account-username-create");
    assert!(f.drawn(cx, "account-username"));
    assert!(f.drawn(cx, "account-password"));
    assert!(!f.drawn(cx, "account-provider-github"));
    f.click(cx, "account-cancel");
    assert!(f.drawn(cx, "account-username-create"));
    assert!(!f.drawn(cx, "account-username"));
}

/// Signed out, the page offers the two ways in; each opens its form, which
/// checks what was typed before anything starts, and says plainly why it
/// can't reach a server when there is none (tests have no backend).
#[gpui_kit::test]
fn the_signed_out_forms_check_before_they_start(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.dispatch(cx, Box::new(OpenAccountSettings));
    let page = cx.read(|cx| f.workspace.read(cx).settings.read(cx).account.clone());
    f.click(cx, "account-create");
    assert_eq!(
        cx.read(|cx| page.read(cx).form()),
        SignedOutForm::CreateAccount
    );
    for selector in [
        "account-username",
        "account-password",
        "account-confirm",
        "use-different-server",
    ] {
        assert!(f.drawn(cx, selector), "{selector}");
    }
    let fill = |cx: &mut TestAppContext, values: [&str; 4]| {
        f.window
            .update(cx, |_, window, cx| {
                page.update(cx, |page, cx| page.fill(values, window, cx))
            })
            .unwrap();
        f.draw(cx);
    };
    let error = |cx: &mut TestAppContext| cx.read(|cx| page.read(cx).error().map(str::to_owned));
    fill(
        cx,
        ["ngoni", "correct horse battery", "correct horse batter", ""],
    );
    f.click(cx, "account-submit");
    assert_eq!(error(cx).as_deref(), Some("The passwords don't match."));
    fill(cx, ["ngoni", "short", "short", ""]);
    f.click(cx, "account-submit");
    assert_eq!(
        error(cx).as_deref(),
        Some("Use at least 12 characters. A few random words work well.")
    );
    fill(
        cx,
        [
            "ngoni",
            "correct horse battery",
            "correct horse battery",
            "",
        ],
    );
    f.click(cx, "account-submit");
    assert_eq!(
        error(cx).as_deref(),
        Some("Can't reach the server. Your notes are safe on this device.")
    );
    // Nothing ran, so nothing is left running.
    assert!(cx.read(|cx| cx.global::<Services>().task.is_none()));
    assert_eq!(
        cx.read(|cx| cx.global::<Services>().account.clone()),
        AccountState::SignedOut
    );
    // Sign in, and from there the recovery form.
    f.click(cx, "account-cancel");
    f.click(cx, "account-sign-in");
    assert_eq!(cx.read(|cx| page.read(cx).form()), SignedOutForm::SignIn);
    assert!(!f.drawn(cx, "account-confirm"));
    f.click(cx, "account-forgot");
    assert_eq!(cx.read(|cx| page.read(cx).form()), SignedOutForm::Recover);
    fill(
        cx,
        [
            "ngoni",
            "correct horse battery",
            "correct horse battery",
            "nope",
        ],
    );
    f.click(cx, "account-submit");
    assert_eq!(
        error(cx).as_deref(),
        Some("A recovery key has 28 letters and digits, in groups of four.")
    );
}

/// The palette and the welcome screen lead to the Sign in form; the
/// palette offers what fits the account.
#[gpui_kit::test]
fn sign_in_is_offered_where_it_fits(cx: &mut TestAppContext) {
    let f = setup(cx);
    let labels = |cx: &mut TestAppContext| {
        f.keys(cx, "cmd-k");
        let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
        let labels = cx.read(|cx| palette.read(cx).item_labels());
        f.keys(cx, "escape");
        labels
    };
    let signed_out = labels(cx);
    for label in ["Sign in to sync…", "Create account…", "Settings: Account"] {
        assert!(
            signed_out.iter().any(|l| l == label),
            "{label}: {signed_out:?}"
        );
    }
    for label in [
        "Sync now",
        "Sign out",
        "Change password…",
        "New recovery key…",
    ] {
        assert!(
            !signed_out.iter().any(|l| l == label),
            "{label}: {signed_out:?}"
        );
    }
    f.dispatch(cx, Box::new(crate::workspace::ShowSignIn));
    assert!(f.open(cx));
    assert_eq!(f.page(cx), SettingsPage::Account);
    let page = cx.read(|cx| f.workspace.read(cx).settings.read(cx).account.clone());
    assert_eq!(cx.read(|cx| page.read(cx).form()), SignedOutForm::SignIn);
    f.keys(cx, "escape");
    sign_in_as(cx, "ngoni");
    let signed_in = labels(cx);
    for label in [
        "Sync now",
        "Sign out",
        "Change password…",
        "New recovery key…",
    ] {
        assert!(
            signed_in.iter().any(|l| l == label),
            "{label}: {signed_in:?}"
        );
    }
    assert!(
        !signed_in.iter().any(|l| l == "Sign in to sync…"),
        "{signed_in:?}"
    );
}

/// A conflict names its note on the Account page, and Open note goes to it.
#[gpui_kit::test]
fn a_conflicted_note_opens_from_the_sync_page(cx: &mut TestAppContext) {
    let f = setup(cx);
    let library = cx.read(|cx| f.workspace.read(cx).library.clone());
    let other = cx.read(|cx| {
        library
            .read(cx)
            .notes
            .iter()
            .map(|n| n.id.clone())
            .find(|id| *id != f.note)
            .unwrap()
    });
    sign_in_as(cx, "ngoni");
    cx.update(|cx| {
        cx.global_mut::<Services>().sync = SyncState::Synced {
            at_ms: 0,
            conflicts: 1,
            conflicted: vec!["book:gone".into(), other.clone()],
            problem: None,
        };
    });
    f.dispatch(cx, Box::new(OpenAccountSettings));
    f.click(cx, "open-conflicted-note");
    assert!(!f.open(cx));
    let open = cx.read(|cx| {
        let editor = f.workspace.read(cx).editor.clone();
        editor.read(cx).note_id().map(str::to_owned)
    });
    assert_eq!(open, Some(other));
}

/// The readable width can be turned off, for the editor at once and in the
/// settings file.
#[gpui_kit::test]
fn the_readable_width_can_be_turned_on(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.dispatch(cx, Box::new(OpenEditorSettings));
    let editor = cx.read(|cx| f.workspace.read(cx).editor.clone());
    assert!(!cx.read(|cx| editor.read(cx).readable_width));
    f.click(cx, "readable-width");
    assert!(cx.read(|cx| editor.read(cx).readable_width));
    let saved = cx.read(|cx| {
        let state = cx.global::<AppState>();
        AppConfig::load(&crate::paths::config_path(&state.data_dir)).unwrap()
    });
    assert!(saved.appearance.readable_width);
}

impl Fixture {
    fn focus_mode(&self, cx: &mut TestAppContext) -> bool {
        cx.read(|cx| self.workspace.read(cx).focus_mode)
    }

    fn drawn(&self, cx: &mut TestAppContext, selector: &'static str) -> bool {
        self.draw(cx);
        VisualTestContext::from_window(self.window, cx)
            .debug_bounds(selector)
            .is_some()
    }

    fn saved_focus_mode(&self, cx: &mut TestAppContext) -> bool {
        cx.read(|cx| {
            let state = cx.global::<AppState>();
            AppConfig::load(&crate::paths::config_path(&state.data_dir))
                .unwrap()
                .layout
                .focus_mode
        })
    }
}

/// Focus mode and Settings never show together: ⌘, from focus mode shows
/// Settings beside the sidebar, and going back brings focus mode back with
/// the keyboard in the text. ⌘⇧D from Settings goes to the note, alone.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn settings_and_focus_mode_take_turns(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.keys(cx, "cmd-shift-d");
    assert!(f.focus_mode(cx));
    assert!(!f.drawn(cx, "sidebar-pane"));

    f.keys(cx, "cmd-,");
    assert!(f.open(cx));
    assert!(!f.focus_mode(cx), "Settings leave focus mode");
    assert!(!f.saved_focus_mode(cx));
    assert!(f.drawn(cx, "sidebar-pane"));
    assert!(f.drawn(cx, "settings-pane"));
    assert!(!f.drawn(cx, "editor-pane"));
    assert!(!f.drawn(cx, "list-pane"));

    f.keys(cx, "escape");
    assert!(!f.open(cx));
    assert!(f.focus_mode(cx), "going back brings focus mode back");
    assert!(f.saved_focus_mode(cx));
    assert!(f.focused(cx, &f.body));
    assert!(!f.drawn(cx, "sidebar-pane"));
    assert!(f.drawn(cx, "editor-pane"));

    // ⌘⇧D in Settings: back to the note, in focus mode.
    f.keys(cx, "cmd-shift-d");
    assert!(!f.focus_mode(cx));
    f.keys(cx, "cmd-, cmd-shift-d");
    assert!(!f.open(cx));
    assert!(f.focus_mode(cx));
    assert!(f.focused(cx, &f.body));

    // Leaving Settings for a place in the sidebar shows the list, not focus
    // mode.
    f.keys(cx, "cmd-,");
    let library = cx.read(|cx| f.workspace.read(cx).library.clone());
    library.update(cx, |library, cx| library.set_filter(Filter::Pinned, cx));
    cx.run_until_parked();
    assert!(!f.open(cx));
    assert!(!f.focus_mode(cx));
    assert!(f.drawn(cx, "list-pane"));
}

/// ⌘\ folds the sidebar to its rail beside Settings too, and the rail's gear
/// is the place shown. Escape goes first to the palette over Settings, then
/// to Settings, wherever the keyboard is in the window.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn the_rail_palette_and_escape_in_settings(cx: &mut TestAppContext) {
    let f = setup(cx);
    let sidebar = cx.read(|cx| f.workspace.read(cx).sidebar.clone());
    f.keys(cx, "cmd-,");
    f.keys(cx, "cmd-\\");
    assert!(f.open(cx), "Settings stay");
    assert!(cx.read(|cx| sidebar.read(cx).is_collapsed()));
    assert!(f.gear_active(cx));
    assert!(f.drawn(cx, "sidebar-pane"));
    assert!(f.drawn(cx, "settings-pane"));
    f.draw(cx);
    let (rail, page) = {
        let mut vcx = VisualTestContext::from_window(f.window, cx);
        (
            vcx.debug_bounds("sidebar-pane").unwrap(),
            vcx.debug_bounds("settings-pane").unwrap(),
        )
    };
    assert_eq!(f32::from(rail.size.width), crate::layout::rail_width(false));
    assert_eq!(
        page.left(),
        rail.right(),
        "Settings start where the rail ends"
    );

    // The palette takes the first Escape, Settings the next.
    let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
    f.keys(cx, "cmd-k");
    assert!(cx.read(|cx| palette.read(cx).is_open()));
    f.keys(cx, "escape");
    assert!(!cx.read(|cx| palette.read(cx).is_open()));
    assert!(f.open(cx));
    f.keys(cx, "escape");
    assert!(!f.open(cx));
    assert!(f.focused(cx, &f.body));

    // With the keyboard on the window itself rather than in Settings.
    f.keys(cx, "cmd-,");
    let focus = cx.read(|cx| f.workspace.read(cx).focus.clone());
    f.window
        .update(cx, |_, window, cx| window.focus(&focus, cx))
        .unwrap();
    f.keys(cx, "escape");
    assert!(!f.open(cx));

    f.keys(cx, "cmd-\\");
    assert!(!cx.read(|cx| sidebar.read(cx).is_collapsed()));
}

/// In focus mode the list is out of sight, so asking for its search box
/// brings it back rather than typing into nothing.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn search_leaves_focus_mode(cx: &mut TestAppContext) {
    let f = setup(cx);
    let list = cx.read(|cx| f.workspace.read(cx).note_list.clone());
    let search = cx.read(|cx| gpui_kit::Focusable::focus_handle(list.read(cx).search.read(cx), cx));
    for keys in ["cmd-l", "cmd-shift-f"] {
        f.keys(cx, "cmd-shift-d");
        assert!(f.focus_mode(cx));
        f.keys(cx, keys);
        assert!(!f.focus_mode(cx), "{keys}");
        assert!(f.drawn(cx, "list-pane"), "{keys}");
        assert!(f.focused(cx, &search), "{keys}");
    }
}

/// Over Settings the palette offers Back to notes, which goes back as Escape
/// does; over the notes it isn't offered.
#[gpui_kit::test]
fn the_palette_goes_back_to_the_notes(cx: &mut TestAppContext) {
    let f = setup(cx);
    let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
    let ask = |cx: &mut TestAppContext| {
        f.keys(cx, "cmd-k");
        f.window
            .update(cx, |_, window, cx| {
                palette.update(cx, |palette, cx| {
                    palette.set_query("back to notes", window, cx)
                })
            })
            .unwrap();
        cx.executor().advance_clock(Duration::from_millis(200));
        cx.run_until_parked();
        cx.read(|cx| palette.read(cx).item_labels())
    };
    let labels = ask(cx);
    assert!(!labels.iter().any(|l| l == "Back to notes"), "{labels:?}");
    f.keys(cx, "escape");

    f.keys(cx, "cmd-,");
    assert!(f.open(cx));
    let labels = ask(cx);
    assert_eq!(
        labels.first().map(String::as_str),
        Some("Back to notes"),
        "{labels:?}"
    );
    f.keys(cx, "enter");
    assert_eq!(cx.read(|cx| palette.read(cx).mode()), None);
    assert!(!f.open(cx));
    assert!(f.focused(cx, &f.body), "the keyboard is back in the text");
}

/// Back up now leaves its copy beside the daily backups under a name their
/// rotation passes over, so it never costs one of them.
#[gpui_kit::test]
fn back_up_now_keeps_its_copy_apart(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.keys(cx, "cmd-,");
    assert_eq!(f.page(cx), SettingsPage::General);
    // The page is long enough that Backups is below the fold.
    cx.update(|cx| {
        f.workspace
            .read(cx)
            .settings
            .clone()
            .update(cx, |settings, _| settings.scroll_to_end_for_test())
    });
    f.draw(cx);
    // Backups is above the end of the page now: bring it into view.
    let y = f.draw_bounds_y(cx, "backup-now");
    cx.update(|cx| {
        f.workspace
            .read(cx)
            .settings
            .clone()
            .update(cx, |settings, _| settings.scroll_by_for_test(300. - y))
    });
    f.click(cx, "backup-now");
    cx.run_until_parked();
    let dir = crate::paths::backups_dir(f._dir.path());
    let names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names.len(), 1, "{names:?}");
    assert!(
        names[0].starts_with("kanso-manual-") && names[0].ends_with(".db"),
        "{names:?}"
    );
}

/// Picking themes on the Themes page shows them at once, in the panes and
/// the component library alike, and saves the choice. Following the system
/// (the default) a dark theme takes the dark slot without changing a light
/// window; once following stops, the UI theme is the one clicked.
#[gpui_kit::test]
fn picking_a_theme_applies_it_everywhere(cx: &mut TestAppContext) {
    use crate::config::{KANSO_DARK, KANSO_LIGHT, ThemePreference};
    use crate::settings::OpenAppearanceSettings;
    use crate::theme::{Colors, DARK, LIGHT};
    use gpui_kit::component::ActiveTheme as _;
    let f = setup(cx);
    f.dispatch(cx, Box::new(OpenAppearanceSettings));
    assert_eq!(f.page(cx), SettingsPage::Appearance);
    let state = |cx: &mut TestAppContext| {
        cx.read(|cx| {
            (
                *Colors::get(cx),
                cx.theme().is_dark(),
                cx.global::<AppState>().config.appearance.themes.clone(),
            )
        })
    };
    let (colors, dark, themes) = state(cx);
    assert!(themes.follow_system);
    // The test platform's windows are light.
    assert_eq!((colors, dark), (LIGHT, false));

    // Following the system, Nord becomes the dark theme; the window stays
    // light.
    f.click(cx, "theme-ui-nord");
    let (colors, dark, themes) = state(cx);
    assert_eq!((colors, dark), (LIGHT, false));
    assert_eq!(
        (themes.light.as_str(), themes.dark.as_str()),
        (KANSO_LIGHT, "Nord")
    );

    // Not following, a click shows the theme.
    f.click(cx, "themes-follow-system");
    f.click(cx, "theme-ui-kanso-dark");
    let (colors, dark, themes) = state(cx);
    assert_eq!((colors, dark), (DARK, true));
    assert!(!themes.follow_system);
    assert_eq!(themes.ui, KANSO_DARK);
    let editor_background = cx.read(|cx| cx.theme().highlight_theme.style.editor_background);
    assert_eq!(editor_background, Some(DARK.editor.into()));
    let saved = cx.read(|cx| {
        let state = cx.global::<AppState>();
        AppConfig::load(&crate::paths::config_path(&state.data_dir)).unwrap()
    });
    assert_eq!(saved.appearance.themes, themes);
    assert_eq!(saved.appearance.theme, ThemePreference::Dark);
    f.click(cx, "theme-ui-kanso-light");
    assert_eq!(state(cx).0, LIGHT);
}

/// Every icon path asked for as windows draw, with the path drawn for it.
type Seen = std::rc::Rc<std::cell::RefCell<Vec<(String, String)>>>;

/// Wraps the icon hook `icon_themes::init` set, keeping what it is asked
/// and what it answers.
fn record_icons(cx: &mut App) -> Seen {
    let seen = Seen::default();
    let kept = seen.clone();
    gpui_kit::component::set_icon_resolver(cx, move |path, cx| {
        let drawn = cx
            .try_global::<crate::icon_themes::ActiveIconTheme>()
            .and_then(|active| active.resolve(path));
        kept.borrow_mut().push((
            path.to_owned(),
            drawn
                .as_ref()
                .map_or_else(|| path.to_owned(), |d| d.to_string()),
        ));
        drawn
    });
    seen
}

/// What `seen` drew for `path` since it was last emptied.
fn drawn_for(seen: &Seen, path: &str) -> Vec<String> {
    let mut drawn: Vec<String> = seen
        .borrow()
        .iter()
        .filter(|(asked, _)| asked == path)
        .map(|(_, drawn)| drawn.clone())
        .collect();
    drawn.dedup();
    drawn
}

/// A gpui-kit control with an icon of its own, a library slot and a UI
/// icon, side by side.
struct IconSampler;

impl gpui_kit::Render for IconSampler {
    fn render(
        &mut self,
        _: &mut gpui_kit::Window,
        _: &mut gpui_kit::Context<Self>,
    ) -> impl gpui_kit::IntoElement {
        use gpui_kit::ParentElement as _;
        gpui_kit::div()
            .child(gpui_kit::component::checkbox::Checkbox::new("sample").checked(true))
            .child(crate::icon_themes::library_icon(
                crate::icon_themes::Slot::Notebook,
            ))
            .child(gpui_kit::component::Icon::new(
                gpui_kit::assets::IconName::Search,
            ))
    }
}

/// Switching the icon theme redraws, in the same window and with no new
/// one opened, a gpui-kit control's own icon (the checkbox's check) and
/// Kanso's icons alike, each under a path of the new theme's, so GPUI
/// draws it afresh rather than from what it kept of the old theme. Back
/// on Kanso's own icons, each path is the built-in one again.
#[gpui_kit::test]
fn switching_icon_theme_redraws_controls_and_kanso_icons_in_one_window(cx: &mut TestAppContext) {
    let (window, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_reduce_motion(true);
        crate::icon_themes::init(cx);
        gpui_kit::open_window(Default::default(), cx, |_, cx| cx.new(|_| IconSampler)).unwrap()
    });
    let seen = cx.update(record_icons);
    let draw = |cx: &mut TestAppContext| {
        window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
    };
    draw(cx);
    assert_eq!(drawn_for(&seen, "icons/check.svg"), vec!["icons/check.svg"]);
    assert_eq!(
        drawn_for(&seen, "icons/library/notebook.svg"),
        vec!["icons/notebook.svg"]
    );
    assert_eq!(
        drawn_for(&seen, "icons/search.svg"),
        vec!["icons/search.svg"]
    );

    // Phosphor, chosen as Settings and the palette choose it: no draw is
    // asked for; choosing redraws the window.
    seen.borrow_mut().clear();
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let data_dir = dir.path().to_path_buf();
    cx.update(|cx| {
        cx.set_global(AppState {
            store,
            data_dir,
            config: AppConfig::default(),
        });
    });
    let phosphor = crate::icon_themes::builtin("phosphor").clone();
    cx.update(|cx| {
        cx.global_mut::<AppState>().config.appearance.icon_theme = Some("phosphor".into());
        crate::icon_themes::apply(cx);
    });
    cx.run_until_parked();
    let key = format!("icons/@{}/", phosphor.key);
    for path in [
        "icons/check.svg",
        "icons/library/notebook.svg",
        "icons/search.svg",
    ] {
        let drawn = drawn_for(&seen, path);
        assert!(
            !drawn.is_empty() && drawn.iter().all(|d| d.starts_with(&key)),
            "{path} drew {drawn:?}"
        );
    }
    assert_eq!(cx.update(|cx| cx.windows().len()), 1);
    let id = window.window_id();
    assert!(cx.update(|cx| cx.windows()[0].window_id() == id));

    seen.borrow_mut().clear();
    cx.update(|cx| {
        cx.global_mut::<AppState>().config.appearance.icon_theme = None;
        crate::icon_themes::apply(cx);
    });
    cx.run_until_parked();
    assert_eq!(drawn_for(&seen, "icons/check.svg"), vec!["icons/check.svg"]);
}

/// Picking an icon theme's card on the Icon Themes page draws the main window's
/// icons with it at once (the sidebar's among them), and saves the choice;
/// the palette lists the themes with the one in use checked.
#[gpui_kit::test]
fn picking_an_icon_theme_in_settings_applies_it(cx: &mut TestAppContext) {
    use crate::settings::OpenIconThemeSettings;
    let f = setup(cx);
    let seen = cx.update(record_icons);
    f.dispatch(cx, Box::new(OpenIconThemeSettings));
    assert_eq!(f.page(cx), SettingsPage::IconThemes);
    f.draw(cx);
    seen.borrow_mut().clear();
    f.click(cx, "icon-theme-phosphor");
    let phosphor = crate::icon_themes::builtin("phosphor").clone();
    let saved = cx.read(|cx| {
        let state = cx.global::<AppState>();
        AppConfig::load(&crate::paths::config_path(&state.data_dir)).unwrap()
    });
    assert_eq!(saved.appearance.icon_theme.as_deref(), Some("phosphor"));
    assert_eq!(
        cx.read(|cx| crate::icon_themes::active(cx).id.clone()),
        "phosphor"
    );
    let key = format!("icons/@{}/", phosphor.key);
    // Drawn before the click with Kanso's own icon, then with Phosphor's.
    let drawn = drawn_for(&seen, "icons/library/all-notes.svg");
    assert!(
        drawn.last().is_some_and(|d| d.starts_with(&key)),
        "the sidebar's All Notes drew {drawn:?}"
    );
    // Lucide again, from its card.
    f.click(cx, "icon-theme-lucide");
    let saved = cx.read(|cx| {
        let state = cx.global::<AppState>();
        AppConfig::load(&crate::paths::config_path(&state.data_dir)).unwrap()
    });
    assert_eq!(saved.appearance.icon_theme, None);
    assert_eq!(
        cx.read(|cx| crate::icon_themes::active(cx).id.clone()),
        "lucide"
    );
}

/// Choosing System Dictation in Settings › Editor and pressing the note's
/// mic puts the keyboard in the note, where the system types (test windows
/// draw no frames, so the system is never asked). The press used to update
/// the editor from inside its own click, and crash.
#[gpui_kit::test]
fn system_dictation_starts_from_the_mic(cx: &mut TestAppContext) {
    use crate::config::VoiceEngine;
    use crate::settings::CloseSettings;
    let f = setup(cx);
    f.dispatch(cx, Box::new(OpenEditorSettings));
    cx.update(|cx| {
        f.workspace
            .read(cx)
            .settings
            .clone()
            .update(cx, |settings, _| settings.scroll_to_end_for_test())
    });
    f.click(cx, "voice-engine-1");
    assert_eq!(
        cx.read(|cx| cx.global::<AppState>().config.voice.engine),
        VoiceEngine::System
    );
    f.dispatch(cx, Box::new(CloseSettings));
    f.click(cx, "dictate");
    let focused = cx.update(|cx| {
        f.window
            .update(cx, |_, window, cx| window.focused(cx))
            .unwrap()
    });
    assert_eq!(focused, Some(f.body.clone()));
}

/// Settings › Editor has font pickers for writing, monospace and the
/// preview, and the preview's text size steps on its own, apart from the
/// editor's, and is saved.
#[gpui_kit::test]
fn the_preview_has_its_own_text_size_and_font(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.dispatch(cx, Box::new(OpenEditorSettings));
    for picker in ["font-writing", "font-code", "font-preview"] {
        assert!(f.drawn(cx, picker), "{picker}");
    }
    let sizes = |cx: &mut TestAppContext| {
        cx.read(|cx| {
            let appearance = &cx.global::<AppState>().config.appearance;
            (appearance.editor_font_size, appearance.preview_font_size)
        })
    };
    let (editor, preview) = sizes(cx);
    f.click(cx, "preview-larger");
    assert_eq!(sizes(cx), (editor, preview + 1.));
    let saved = cx.read(|cx| {
        let state = cx.global::<AppState>();
        AppConfig::load(&crate::paths::config_path(&state.data_dir)).unwrap()
    });
    assert_eq!(saved.appearance.preview_font_size, preview + 1.);
    assert_eq!(saved.appearance.editor_font_size, editor);
}

/// Settings › Editor › Diagrams chooses what draws Mermaid: it's saved, and
/// diagrams drawn after it use the new engine.
#[gpui_kit::test]
fn the_diagram_renderer_is_chosen_and_saved(cx: &mut TestAppContext) {
    use crate::config::DiagramRenderer;
    use kanso_render::{Job, Kind, MermaidEngine, Palette};

    let f = setup(cx);
    f.dispatch(cx, Box::new(OpenEditorSettings));
    let chosen = |cx: &mut TestAppContext| {
        cx.read(|cx| cx.global::<AppState>().config.appearance.diagram_renderer)
    };
    let next_job = || Job::new(Kind::Mermaid, "flowchart LR\n  a --> b\n", Palette::LIGHT).mermaid;
    assert_eq!(chosen(cx), DiagramRenderer::Pele);
    f.click(cx, "diagram-renderer-1");
    assert_eq!(chosen(cx), DiagramRenderer::Kanso);
    assert_eq!(next_job(), MermaidEngine::MermaidSvg);
    let saved = cx.read(|cx| {
        let state = cx.global::<AppState>();
        AppConfig::load(&crate::paths::config_path(&state.data_dir)).unwrap()
    });
    assert_eq!(saved.appearance.diagram_renderer, DiagramRenderer::Kanso);
    f.click(cx, "diagram-renderer-0");
    assert_eq!(chosen(cx), DiagramRenderer::Pele);
    assert_eq!(next_job(), MermaidEngine::Pele);
}
