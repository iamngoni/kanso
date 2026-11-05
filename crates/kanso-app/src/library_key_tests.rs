//! The library key's screens in a real window (issue #243 step 11): the
//! save sheet and what it does to the config and the notice, Show key…, the
//! rows in Settings, and Restore from backup…. The library here is encrypted
//! with a demo key; no real key appears.

use std::sync::Arc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, Entity, Modifiers, TestAppContext, VisualTestContext,
};
use kanso_store::{KEY_BYTES, LibraryKey, Store};

use crate::config::AppConfig;
use crate::library_key_sheet::Mode;
use crate::services::Services;
use crate::settings::SettingsPage;
use crate::workspace::{AppState, Workspace};

struct Fixture {
    dir: tempfile::TempDir,
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
}

fn demo_key(seed: u8) -> LibraryKey {
    LibraryKey::from_bytes([seed; KEY_BYTES])
}

fn setup(cx: &mut TestAppContext, key: Option<&LibraryKey>) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open_with_key(dir.path().join("kanso.db"), key).unwrap());
    let data_dir = dir.path().to_path_buf();
    let (window, workspace) = cx.update(|cx: &mut App| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store,
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
    let f = Fixture {
        dir,
        window,
        workspace,
    };
    f.draw(cx);
    f
}

impl Fixture {
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
        VisualTestContext::from_window(self.window, cx)
            .debug_bounds(selector)
            .is_some()
    }

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

    fn dialog_open(&self, cx: &mut TestAppContext) -> bool {
        self.window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap()
    }

    fn saved_config(&self) -> AppConfig {
        AppConfig::load(&crate::paths::config_path(self.dir.path())).unwrap()
    }

    fn open_sheet(
        &self,
        cx: &mut TestAppContext,
        mode: Mode,
    ) -> Option<gpui_kit::WeakEntity<crate::library_key_sheet::LibraryKeySheet>> {
        let opened = self
            .window
            .update(cx, |_, window, cx| {
                crate::library_key_sheet::open_sheet(mode, window, cx)
            })
            .unwrap();
        self.draw(cx);
        opened
    }
}

/// The save sheet shows the key once it's asked for, "Later" leaves it
/// unsaved, and the words are gone from memory once it closes.
#[gpui_kit::test]
fn the_save_sheet_shows_the_words_and_wipes_them_when_it_closes(cx: &mut TestAppContext) {
    let f = setup(cx, Some(&demo_key(3)));
    let sheet = f
        .open_sheet(cx, Mode::Save)
        .expect("an encrypted library has a key");
    assert!(f.drawn(cx, "library-key-words"));
    assert!(f.drawn(cx, "library-key-saved"));
    let held = cx.read(|cx| {
        sheet
            .upgrade()
            .map(|sheet| sheet.read(cx).word_count_for_test())
    });
    assert_eq!(held, Some(kanso_store::KEY_WORDS));

    f.click(cx, "library-key-later");
    assert!(!f.dialog_open(cx));
    assert!(sheet.upgrade().is_none(), "the closed sheet is dropped");
    // Later saves nothing: the sheet comes back at the next launch.
    assert!(!cx.read(crate::library_key_sheet::key_saved));
    assert_eq!(
        cx.read(|cx| cx.global::<AppState>().config.saved_library_key.clone()),
        None
    );
}

/// "I've saved it" records it for good and the notice goes with it.
#[gpui_kit::test]
fn i_have_saved_it_stops_the_sheet_and_the_notice(cx: &mut TestAppContext) {
    let f = setup(cx, Some(&demo_key(4)));
    let mut notice = kanso_core::notices::NOTICES
        .iter()
        .find(|notice| notice.id == kanso_core::notices::LIBRARY_ENCRYPTED_ID)
        .unwrap()
        .clone();
    // As if the release had been reached.
    notice.since = semver::Version::new(0, 9, 0);
    let list: &'static [kanso_core::notices::Notice] = Box::leak(Box::new([notice]));
    cx.update(|cx| {
        cx.set_global(crate::notices::NoticeBoard {
            list,
            previous: None,
        });
        cx.refresh_windows();
    });
    assert!(cx.read(crate::notices::banner).is_some());
    assert!(f.drawn(cx, "notice-banner"));

    let sheet = f.open_sheet(cx, Mode::Save).unwrap();
    f.click(cx, "library-key-saved");
    assert!(!f.dialog_open(cx));
    assert!(sheet.upgrade().is_none());
    assert_eq!(
        f.saved_config().saved_library_key,
        Some(demo_key(4).fingerprint()),
        "the key is named by its fingerprint, never written down"
    );
    assert!(cx.read(crate::library_key_sheet::key_saved));
    assert!(cx.read(crate::notices::banner).is_none());
    assert!(!f.drawn(cx, "notice-banner"));

    // A saved key's sheet has Done, not "I've saved it".
    f.open_sheet(cx, Mode::Show).unwrap();
    assert!(f.drawn(cx, "library-key-done"));
    assert!(!f.drawn(cx, "library-key-saved"));
    f.click(cx, "library-key-done");
    assert!(!f.dialog_open(cx));
}

/// A library that isn't encrypted has no key to show.
#[gpui_kit::test]
fn a_plain_library_has_no_key_sheet_and_no_rows(cx: &mut TestAppContext) {
    let f = setup(cx, None);
    assert!(f.open_sheet(cx, Mode::Save).is_none());
    assert!(!f.dialog_open(cx));
    open_backups(&f, cx);
    assert!(f.drawn(cx, "backup-now"));
    assert!(!f.drawn(cx, "show-library-key"));
    assert!(!f.drawn(cx, "restore-backup"));
}

fn open_backups(f: &Fixture, cx: &mut TestAppContext) {
    f.window
        .update(cx, |_, window, cx| {
            f.workspace.update(cx, |workspace, cx| {
                workspace.open_settings(Some(SettingsPage::General), window, cx)
            })
        })
        .unwrap();
    f.draw(cx);
    cx.update(|cx| {
        f.workspace
            .read(cx)
            .settings
            .clone()
            .update(cx, |settings, _| settings.scroll_to_end_for_test())
    });
    f.draw(cx);
    // Bring Backups into view, as a person scrolling to it would.
    let y = VisualTestContext::from_window(f.window, cx)
        .debug_bounds("backup-now")
        .map(|bounds| f32::from(bounds.origin.y));
    if let Some(y) = y {
        cx.update(|cx| {
            f.workspace
                .read(cx)
                .settings
                .clone()
                .update(cx, |settings, _| settings.scroll_by_for_test(300. - y))
        });
    }
    f.draw(cx);
}

/// Backups gets the rows, and Show key… asks Touch ID before it shows the
/// words: a no leaves them hidden, a yes opens the sheet.
#[gpui_kit::test]
fn show_key_asks_touch_id_first(cx: &mut TestAppContext) {
    let f = setup(cx, Some(&demo_key(5)));
    let touch_id = crate::touch_id::FakeTouchId::new(false);
    cx.update(|cx| cx.set_global(crate::touch_id::TouchId(touch_id.clone())));
    open_backups(&f, cx);
    assert!(f.drawn(cx, "restore-backup"));

    f.click(cx, "show-library-key");
    assert_eq!(touch_id.asked(), 1);
    assert!(!f.dialog_open(cx), "a refused Touch ID shows nothing");

    touch_id.answer(true);
    f.click(cx, "show-library-key");
    assert_eq!(touch_id.asked(), 2);
    assert!(f.dialog_open(cx));
    assert!(f.drawn(cx, "library-key-words"));
}

/// Without Touch ID, Show key… asks before it shows, and Cancel shows
/// nothing.
#[gpui_kit::test]
fn show_key_without_touch_id_asks_to_confirm(cx: &mut TestAppContext) {
    let f = setup(cx, Some(&demo_key(6)));
    open_backups(&f, cx);
    f.click(cx, "show-library-key");
    assert!(f.dialog_open(cx), "the confirmation is up");
    assert!(!f.drawn(cx, "library-key-words"));
}

/// A backup made on another computer asks for the 24 words; wrong words are
/// explained and change nothing; the right ones stage the restore.
#[gpui_kit::test]
fn restoring_a_backup_from_another_computer_asks_for_its_words(cx: &mut TestAppContext) {
    let f = setup(cx, Some(&demo_key(7)));
    // A backup made elsewhere, with another key.
    let theirs = demo_key(9);
    let there = tempfile::tempdir().unwrap();
    let theirs_store = Store::open_with_key(there.path().join("kanso.db"), Some(&theirs)).unwrap();
    theirs_store.set_meta("note.a", "yes").unwrap();
    let backup = theirs_store
        .backup_now(&there.path().join("backups"), 1_000)
        .unwrap();
    drop(theirs_store);

    let sheet = f
        .window
        .update(cx, |_, window, cx| {
            crate::restore_sheet::begin(backup.clone(), window, cx)
        })
        .unwrap()
        .expect("the file is a backup");
    f.draw(cx);
    assert!(f.drawn(cx, "restore-words"));
    let db = crate::paths::db_path(f.dir.path());

    // Not 24 words: said in words, nothing staged.
    f.window
        .update(cx, |_, window, cx| {
            sheet
                .update(cx, |sheet, cx| {
                    sheet.type_words_for_test("one two three", window, cx)
                })
                .unwrap()
        })
        .unwrap();
    f.click(cx, "restore-go");
    let error = cx.read(|cx| {
        sheet
            .upgrade()
            .and_then(|sheet| sheet.read(cx).error_for_test().map(str::to_owned))
    });
    assert!(error.is_some_and(|error| error.contains("24 words")));
    assert!(!kanso_store::restore_staged(&db));

    // Real words that are another library's: explained.
    let wrong = demo_key(11).to_words();
    f.window
        .update(cx, |_, window, cx| {
            sheet
                .update(cx, |sheet, cx| {
                    sheet.type_words_for_test(&wrong, window, cx)
                })
                .unwrap()
        })
        .unwrap();
    f.click(cx, "restore-go");
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    f.draw(cx);
    let error = cx.read(|cx| {
        sheet
            .upgrade()
            .and_then(|sheet| sheet.read(cx).error_for_test().map(str::to_owned))
    });
    assert!(error.is_some_and(|error| error.contains("another library")));
    assert!(!kanso_store::restore_staged(&db));

    // The right words stage the restore, and the field is emptied.
    let right = theirs.to_words();
    f.window
        .update(cx, |_, window, cx| {
            sheet
                .update(cx, |sheet, cx| {
                    sheet.type_words_for_test(&right, window, cx)
                })
                .unwrap()
        })
        .unwrap();
    f.click(cx, "restore-go");
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    f.draw(cx);
    assert!(kanso_store::restore_staged(&db));
    assert!(!f.dialog_open(cx));
    assert!(
        sheet.upgrade().is_none(),
        "the sheet and its words are gone"
    );
}

/// A restore staged by mistake can be called off: Backups says one is
/// waiting, and Cancel restore forgets it before the next launch.
#[gpui_kit::test]
fn a_waiting_restore_can_be_cancelled(cx: &mut TestAppContext) {
    let key = demo_key(12);
    let f = setup(cx, Some(&key));
    let db = crate::paths::db_path(f.dir.path());
    let backup = cx.read(|cx| {
        cx.global::<AppState>()
            .store
            .backup_now(&crate::paths::backups_dir(f.dir.path()), 1_000)
            .unwrap()
    });
    open_backups(&f, cx);
    assert!(!f.drawn(cx, "cancel-restore"));
    kanso_store::stage_restore(&db, &backup, Some(&key), &key).unwrap();
    assert!(f.drawn(cx, "cancel-restore"));
    f.click(cx, "cancel-restore");
    assert!(!kanso_store::restore_staged(&db));
    assert!(!f.drawn(cx, "cancel-restore"));
}

/// A file that isn't a backup is refused before any sheet opens.
#[gpui_kit::test]
fn a_file_that_is_not_a_backup_opens_no_sheet(cx: &mut TestAppContext) {
    let f = setup(cx, Some(&demo_key(8)));
    // An empty file is refused outright.
    let empty = f.dir.path().join("empty.db");
    std::fs::write(&empty, b"").unwrap();
    let opened = f
        .window
        .update(cx, |_, window, cx| {
            crate::restore_sheet::begin(empty, window, cx)
        })
        .unwrap();
    assert!(opened.is_none());
    assert!(!f.dialog_open(cx));
}

/// Regression: a restore into a library signed in to sync was staged, and
/// sync then undid part of it. It is refused up front, in words that say to
/// sign out first, and nothing is staged. Signed out, the same backup opens
/// the sheet.
#[gpui_kit::test]
fn a_restore_is_refused_while_signed_in_to_sync(cx: &mut TestAppContext) {
    use crate::services::AccountState;
    let key = demo_key(13);
    let f = setup(cx, Some(&key));
    let db = crate::paths::db_path(f.dir.path());
    let backup = cx.read(|cx| {
        cx.global::<AppState>()
            .store
            .backup_now(&crate::paths::backups_dir(f.dir.path()), 1_000)
            .unwrap()
    });
    let sign_in = |cx: &mut TestAppContext| {
        cx.update(|cx| {
            cx.global_mut::<Services>().account = AccountState::SignedIn {
                username: "alice".into(),
            };
        });
    };
    let try_begin = |cx: &mut TestAppContext| {
        f.window
            .update(cx, |_, window, cx| {
                crate::restore_sheet::begin(backup.clone(), window, cx)
            })
            .unwrap()
    };

    sign_in(cx);
    assert!(try_begin(cx).is_none(), "signed in: no sheet");
    assert!(!f.dialog_open(cx));
    assert!(!kanso_store::restore_staged(&db));
    assert!(
        crate::library_key_model::RESTORE_SIGNED_IN.contains("Sign out first"),
        "the refusal says to sign out first"
    );

    cx.update(|cx| cx.global_mut::<Services>().account = AccountState::SignedOut);
    let sheet = try_begin(cx).expect("signed out: the sheet opens");
    // Signing in while the sheet is open still stages nothing.
    sign_in(cx);
    f.draw(cx);
    f.click(cx, "restore-go");
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    assert!(!kanso_store::restore_staged(&db));
    let error = cx.read(|cx| {
        sheet
            .upgrade()
            .and_then(|sheet| sheet.read(cx).error_for_test().map(str::to_owned))
    });
    assert!(error.is_some_and(|error| error.contains("Sign out first")));
}

/// Regression (PR #301 review): "signed in" was only the `SignedIn` state,
/// so a restore was staged while a sign-in was under way (an account task),
/// while the launch was still reading the session back from the keychain
/// (the library shows signed out until then), and with a sign-in waiting on
/// its key. Each is refused; a session the server ended leaves nothing
/// behind, so it isn't.
#[gpui_kit::test]
fn a_restore_waits_while_a_sign_in_is_under_way(cx: &mut TestAppContext) {
    use crate::services::{AccountState, RunningTask, TaskKind};
    let key = demo_key(14);
    let f = setup(cx, Some(&key));
    let db = crate::paths::db_path(f.dir.path());
    let backup = cx.read(|cx| {
        cx.global::<AppState>()
            .store
            .backup_now(&crate::paths::backups_dir(f.dir.path()), 1_000)
            .unwrap()
    });
    let try_begin = |cx: &mut TestAppContext| {
        let opened = f
            .window
            .update(cx, |_, window, cx| {
                crate::restore_sheet::begin(backup.clone(), window, cx)
            })
            .unwrap();
        if opened.is_some() {
            f.window
                .update(cx, |_, window, cx| window.close_dialog(cx))
                .unwrap();
        }
        opened.is_some()
    };
    let reset = |cx: &mut TestAppContext| {
        cx.update(|cx| {
            let services = cx.global_mut::<Services>();
            services.account = AccountState::SignedOut;
            services.task = None;
            services.restoring = false;
        });
    };

    reset(cx);
    cx.update(|cx| {
        cx.global_mut::<Services>().task = Some(RunningTask {
            kind: TaskKind::SignIn,
            step: kanso_sync::Step::DerivingKeys,
        });
    });
    assert!(!try_begin(cx), "a sign-in under way");

    reset(cx);
    cx.update(|cx| cx.global_mut::<Services>().restoring = true);
    assert!(!try_begin(cx), "the launch still reading the session");

    reset(cx);
    cx.update(|cx| {
        cx.global_mut::<Services>().account = AccountState::SignedInLocked {
            username: "alice".into(),
        };
    });
    assert!(!try_begin(cx), "a sign-in waiting on its key");

    reset(cx);
    cx.update(|cx| {
        cx.global_mut::<Services>().account = AccountState::SessionEnded {
            username: "alice".into(),
            reason: kanso_sync::SignOutReason::PasswordChanged,
        };
    });
    assert!(try_begin(cx), "a session the server ended is signed out");
    assert!(!kanso_store::restore_staged(&db));
}
