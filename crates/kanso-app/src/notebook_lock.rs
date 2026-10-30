//! Locked notebooks in the app: locking one seals every note in it and in
//! the notebooks inside it, opening one asks for the lock password (or
//! Touch ID, or Windows Hello) once, and a note moved or made there later
//! is sealed as it arrives (`editor.rs`, `locked::reveal`). The store reads
//! every note in a locked notebook as locked, sealed or not, so its text
//! stays out of search, the local API and related notes
//! (`kanso_store`'s `locked_books`).
//!
//! What a notebook's lock does to notes that were locked before, moved out,
//! or unlocked again is decided here and in the store:
//!
//! - A note locked on its own keeps its own seal, and its own password.
//! - A note moved out of a locked notebook stays locked; Remove Lock makes
//!   it plain again.
//! - Removing a notebook's lock leaves its notes locked; notes put there
//!   afterwards aren't.
//! - Locking a notebook again (Lock Notes Now, quitting, or ten minutes
//!   after unlocking) forgets the keys, so its notes close.

use std::rc::Rc;
use std::sync::Arc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_sync::lock::{LockKey, LockPassword};

pub use kanso_engine::lock::{seal_notebook, take_lock_off};

use crate::dialogs::{self, Field, FormSpec, FormView, Submit};
use crate::editor::NoteEditor;
use crate::locked::{
    LockKeys, Target, after_password, any_lock, key_for, password, run_in_background, sentence,
    store, touch_id_on, with_touch_id,
};

/// What runs once a key is in hand.
type WithKey = Rc<dyn Fn(Arc<LockKey>, &mut Window, &mut App)>;

fn book_name(book_id: &str, cx: &App) -> String {
    store(cx)
        .books()
        .ok()
        .and_then(|books| books.into_iter().find(|book| book.id == book_id))
        .map(|book| book.name)
        .unwrap_or_else(|| "the notebook".into())
}

/// Lock Notebook: seals every note in the notebook and the notebooks inside
/// it. The first time this session, Touch ID or Windows Hello (when on) or
/// the lock password opens the keys, or a password is made when nothing is
/// locked yet. Returns the form when it asks for a password.
pub fn lock_notebook(
    book_id: String,
    editor: Entity<NoteEditor>,
    window: &mut Window,
    cx: &mut App,
) -> Option<Entity<FormView>> {
    editor.update(cx, |editor, cx| editor.flush(cx));
    let name = book_name(&book_id, cx);
    let done: WithKey = {
        let editor = editor.clone();
        Rc::new(
            move |key, window, cx| match seal_notebook(&store(cx), &book_id, &key) {
                Ok(_) => {
                    crate::metrics::record(cx, crate::metrics::Feature::Lock);
                    editor.update(cx, |editor, cx| editor.lock_keys_changed(window, cx));
                    cx.refresh_windows();
                }
                Err(err) => window.push_notification(
                    Notification::error(format!("Couldn't lock the notebook: {err}")),
                    cx,
                ),
            },
        )
    };
    if let Some(key) = LockKeys::current(cx) {
        dialogs::confirm(
            "Lock this notebook?",
            format!(
                "Every note in \u{201c}{name}\u{201d} and the notebooks inside it will only open with your lock password, and so will notes put there later. Their earlier versions are deleted, and they won't show in search, backlinks or tasks while locked."
            ),
            "Lock",
            window,
            cx,
            move |window, cx| done(key.clone(), window, cx),
        );
        return None;
    }
    let Some(target) = any_lock(&store(cx)) else {
        return Some(new_password_form(name, done, window, cx));
    };
    let ask = {
        let name = name.clone();
        Rc::new(
            move |target: Target, done: WithKey, window: &mut Window, cx: &mut App| {
                password_form(
                    "Lock this notebook",
                    "Lock",
                    format!(
                        "Type your lock password to lock \u{201c}{name}\u{201d} and every note in it. Their earlier versions are deleted."
                    ),
                    target,
                    done,
                    window,
                    cx,
                )
            },
        )
    };
    if touch_id_on(cx) {
        let (fallback, fallback_target, fallback_done) =
            (ask.clone(), target.clone(), done.clone());
        with_touch_id(
            window,
            cx,
            move |window, cx| match LockKeys::current(cx) {
                Some(key) => done(key, window, cx),
                None => {
                    ask(target, done, window, cx);
                }
            },
            move |window, cx| {
                fallback(fallback_target, fallback_done, window, cx);
            },
        );
        return None;
    }
    Some(ask(target, done, window, cx))
}

/// Opening a locked notebook: asks once for Touch ID, Windows Hello or the
/// lock password, unless a key this session holds already opens it. Its
/// notes open with the key, and those not sealed yet are sealed. Returns the
/// form when it asks for a password.
pub fn open_notebook(
    book_id: &str,
    editor: Entity<NoteEditor>,
    window: &mut Window,
    cx: &mut App,
) -> Option<Entity<FormView>> {
    let target = Target::of_book(book_id, &store(cx))?;
    if LockKeys::opens(&target, cx) {
        return None;
    }
    let name = book_name(&target.id, cx);
    let opened: WithKey = Rc::new(move |_, window, cx| {
        editor.update(cx, |editor, cx| editor.lock_keys_changed(window, cx));
        cx.refresh_windows();
    });
    let ask = Rc::new(
        move |target: Target, opened: WithKey, window: &mut Window, cx: &mut App| {
            password_form(
                "Unlock notebook",
                "Unlock",
                format!(
                    "\u{201c}{name}\u{201d} is locked. Its notes open with your lock password."
                ),
                target,
                opened,
                window,
                cx,
            )
        },
    );
    if touch_id_on(cx) {
        let (fallback, fallback_target, fallback_opened) =
            (ask.clone(), target.clone(), opened.clone());
        with_touch_id(
            window,
            cx,
            move |window, cx| {
                // Keys kept before the notebook was locked elsewhere don't
                // open it.
                let key = target
                    .envelope
                    .as_ref()
                    .ok()
                    .and_then(|envelope| LockKeys::fitting(cx, envelope));
                match key {
                    Some(key) => opened(key, window, cx),
                    None => {
                        ask(target, opened, window, cx);
                    }
                }
            },
            move |window, cx| {
                fallback(fallback_target, fallback_opened, window, cx);
            },
        );
        return None;
    }
    Some(ask(target, opened, window, cx))
}

/// Remove Notebook Lock: notes put in the notebook stop being locked. The
/// notes in it now stay locked, each with its own seal, so no text turns
/// plain without the note being asked for. The notebook must be open first.
pub fn remove_notebook_lock(
    book_id: String,
    editor: Entity<NoteEditor>,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(target) = Target::of_book(&book_id, &store(cx)).filter(|t| t.id == book_id) else {
        return;
    };
    if !LockKeys::opens(&target, cx) {
        open_notebook(&book_id, editor, window, cx);
        return;
    }
    let name = book_name(&book_id, cx);
    dialogs::confirm(
        "Remove the notebook's lock?",
        format!(
            "Notes put in \u{201c}{name}\u{201d} from now on aren't locked. The notes in it now stay locked: remove a note's lock to keep its text as plain text again."
        ),
        "Remove Lock",
        window,
        cx,
        move |window, cx| {
            if let Err(err) = take_lock_off(&store(cx), &book_id) {
                window.push_notification(
                    Notification::error(format!("Couldn't remove the lock: {err}")),
                    cx,
                );
            }
        },
    );
}

/// Asks for the lock password, tries it on `target`, keeps the key it gives
/// for the session and runs `then` with it.
#[allow(clippy::too_many_arguments)]
fn password_form(
    title: &'static str,
    button: &'static str,
    description: String,
    target: Target,
    then: WithKey,
    window: &mut Window,
    cx: &mut App,
) -> Entity<FormView> {
    let spec = FormSpec::new(title, button)
        .description(description)
        .field(Field::secret("Lock password"));
    dialogs::form(spec, window, cx, move |values, window, cx| {
        let password = match LockPassword::new(&values[0]) {
            Ok(password) => password,
            // A shorter text can't be the password.
            Err(_) => {
                return Submit::Invalid(sentence(&kanso_sync::lock::LockError::WrongPassword));
            }
        };
        let (tried, target) = (target.clone(), target.clone());
        let then = then.clone();
        run_in_background(
            window,
            cx,
            move || key_for(&tried, &password),
            move |key, window, cx| {
                LockKeys::add(key, cx);
                after_password(window, cx);
                let envelope = target.envelope.as_ref().ok();
                let key = envelope
                    .and_then(|envelope| LockKeys::fitting(cx, envelope))
                    .or_else(|| LockKeys::current(cx))
                    .ok_or_else(|| "The notes were locked again.".to_owned())?;
                then(key, window, cx);
                Ok(())
            },
        )
    })
}

/// Locking the first notebook when nothing is locked yet: a new lock
/// password, typed twice.
fn new_password_form(
    name: String,
    then: WithKey,
    window: &mut Window,
    cx: &mut App,
) -> Entity<FormView> {
    let spec = FormSpec::new("Set a lock password", "Lock")
        .description(format!(
            "Locked notes open with this password, on this device and any other that syncs with it. Kanso can't recover it: forget it, and the notes locked with it can't be read. The earlier versions of the notes in \u{201c}{name}\u{201d} are deleted."
        ))
        .field(Field::secret("Lock password").hint("At least 8 characters."))
        .field(Field::secret("Type it again"));
    dialogs::form(spec, window, cx, move |values, window, cx| {
        if values[0] != values[1] {
            return Submit::Invalid("The passwords don't match.".into());
        }
        let password = match password(&values[0]) {
            Ok(password) => password,
            Err(submit) => return submit,
        };
        let params = LockKeys::params(cx);
        let then = then.clone();
        run_in_background(
            window,
            cx,
            move || LockKey::create(&password, params).map_err(|err| sentence(&err)),
            move |key, window, cx| {
                LockKeys::replace(Some(key), cx);
                let key = LockKeys::current(cx).expect("a key was just set");
                after_password(window, cx);
                then(key, window, cx);
                Ok(())
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gpui_kit::component::WindowExt as _;
    use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
    use kanso_core::DocType;
    use kanso_store::{Origin, Put, Store};
    use kanso_sync::lock::{Envelope, LockKey, LockPassword};
    use serde_json::json;

    use super::{lock_notebook, open_notebook, take_lock_off};
    use crate::config::AppConfig;
    use crate::dialogs::FormView;
    use crate::editor::NoteEditor;
    use crate::library::Filter;
    use crate::locked::{LockKeys, LockView, lock_now, remove_lock};
    use crate::services::Services;
    use crate::workspace::{AppState, Workspace};

    const PASSWORD: &str = "correct horse";

    struct Fixture {
        _dir: tempfile::TempDir,
        store: Arc<Store>,
        window: AnyWindowHandle,
        workspace: Entity<Workspace>,
        editor: Entity<NoteEditor>,
        /// "Private", with "Deeper" inside it, and "Open" beside them.
        private: String,
        deeper: String,
        open: String,
        /// A note in each.
        plan: String,
        deep: String,
        outside: String,
    }

    fn setup(
        cx: &mut TestAppContext,
        touch_id: Option<Arc<crate::touch_id::FakeTouchId>>,
    ) -> Fixture {
        cx.executor().allow_parking();
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        let book = |name: &str, parent: Option<&str>| {
            store
                .put(
                    DocType::Book,
                    json!({"name": name, "parentBookId": parent}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        };
        let private = book("Private", None);
        let deeper = book("Deeper", Some(&private));
        let open = book("Open", None);
        let note = |book: &str, title: &str, body: &str| {
            store
                .put(
                    DocType::Note,
                    json!({"title": title, "body": body, "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        };
        let plan = note(&private, "Plan", "The secret plan.");
        let deep = note(&deeper, "Deep", "Deeper secrets.");
        let outside = note(&open, "Outside", "Plain words.");
        let (data_dir, app_store) = (dir.path().to_path_buf(), store.clone());
        let (window, workspace) = cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_global(AppState {
                store: app_store,
                data_dir,
                config: AppConfig::default(),
            });
            let mut services = Services::default();
            if let Some(touch_id) = touch_id {
                services.secrets = Some(Arc::new(kanso_sync::MemorySecrets::new()));
                cx.set_global(crate::touch_id::TouchId(touch_id));
            }
            cx.set_global(services);
            LockKeys::use_test_params(cx);
            gpui_kit::open_window(Default::default(), cx, |window, cx| {
                cx.new(|cx| Workspace::new(window, cx))
            })
            .unwrap()
        });
        cx.run_until_parked();
        let editor = cx.read(|cx| workspace.read(cx).editor.clone());
        let f = Fixture {
            _dir: dir,
            store,
            window,
            workspace,
            editor,
            private,
            deeper,
            open,
            plan,
            deep,
            outside,
        };
        f.select(cx, &f.plan.clone());
        f
    }

    impl Fixture {
        fn select(&self, cx: &mut TestAppContext, id: &str) {
            let library = cx.read(|cx| self.workspace.read(cx).library.clone());
            let id = id.to_owned();
            library.update(cx, |library, cx| library.select(Some(id), cx));
            cx.run_until_parked();
        }

        fn dialog_open(&self, cx: &mut TestAppContext) -> bool {
            self.window
                .update(cx, |_, window, cx| window.has_active_dialog(cx))
                .unwrap()
        }

        fn close_dialog(&self, cx: &mut TestAppContext) {
            self.window
                .update(cx, |_, window, cx| window.close_dialog(cx))
                .unwrap();
            cx.run_until_parked();
        }

        fn submit(&self, cx: &mut TestAppContext, form: &Entity<FormView>, values: &[&str]) {
            self.window
                .update(cx, |_, window, cx| {
                    form.update(cx, |form, cx| {
                        form.fill(values, window, cx);
                        form.submit(window, cx);
                    })
                })
                .unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
            loop {
                cx.run_until_parked();
                if !cx.read(|cx| form.read(cx).is_running()) {
                    break;
                }
                assert!(std::time::Instant::now() < deadline, "the task finished");
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        }

        fn lock(&self, cx: &mut TestAppContext) -> Option<Entity<FormView>> {
            let (book, editor) = (self.private.clone(), self.editor.clone());
            self.window
                .update(cx, |_, window, cx| lock_notebook(book, editor, window, cx))
                .unwrap()
        }

        fn open_notebook(&self, cx: &mut TestAppContext) -> Option<Entity<FormView>> {
            let (book, editor) = (self.private.clone(), self.editor.clone());
            self.window
                .update(cx, |_, window, cx| open_notebook(&book, editor, window, cx))
                .unwrap()
        }

        /// Opens `id` in a new tab beside the one showing, as a middle click
        /// in the list does.
        fn open_in_new_tab(&self, cx: &mut TestAppContext, id: &str) {
            let (id, workspace) = (id.to_owned(), self.workspace.clone());
            self.window
                .update(cx, |_, window, cx| {
                    workspace.update(cx, |workspace, cx| {
                        workspace.open_note_in_new_tab(&id, window, cx)
                    })
                })
                .unwrap();
            cx.run_until_parked();
        }

        fn show_tab(&self, cx: &mut TestAppContext, index: usize) {
            let workspace = self.workspace.clone();
            self.window
                .update(cx, |_, _, cx| {
                    workspace.update(cx, |workspace, cx| workspace.show_tab(index, cx))
                })
                .unwrap();
            cx.run_until_parked();
        }

        fn lock_now(&self, cx: &mut TestAppContext) {
            self.window
                .update(cx, |_, window, cx| lock_now(&self.editor, window, cx))
                .unwrap();
            cx.run_until_parked();
        }

        fn view(&self, cx: &mut TestAppContext) -> LockView {
            cx.read(|cx| self.editor.read(cx).lock_view())
        }

        fn text(&self, cx: &mut TestAppContext) -> String {
            cx.read(|cx| self.editor.read(cx).body_text(cx).to_string())
        }

        /// The note's text as the store keeps it, opened with the password:
        /// `None` when it isn't sealed. The store's body is always empty.
        fn sealed_text(&self, id: &str) -> Option<String> {
            let note = self.store.note(id).unwrap().unwrap();
            let envelope = Envelope::from_extra(&note.extra)?.unwrap();
            assert_eq!(note.body, "", "the store never has the text");
            let key =
                LockKey::for_envelope(&LockPassword::new(PASSWORD).unwrap(), &envelope).unwrap();
            key.open(id, &envelope).ok().map(|body| body.to_string())
        }
    }

    #[gpui_kit::test]
    fn locking_a_notebook_seals_its_notes_asks_once_and_seals_what_comes_later(
        cx: &mut TestAppContext,
    ) {
        let f = setup(cx, None);
        // Nothing is locked yet: a password is made.
        let form = f.lock(cx).expect("the first lock sets a password");
        f.submit(cx, &form, &[PASSWORD, PASSWORD]);
        assert_eq!(f.sealed_text(&f.plan).as_deref(), Some("The secret plan."));
        assert_eq!(f.sealed_text(&f.deep).as_deref(), Some("Deeper secrets."));
        assert_eq!(f.sealed_text(&f.outside), None);
        assert_eq!(
            f.store.locked_books().unwrap(),
            [f.private.clone(), f.deeper.clone()].into_iter().collect()
        );
        // The open note stays open, as a locked note does.
        assert_eq!(f.view(cx), LockView::Open);
        assert_eq!(f.text(cx), "The secret plan.");

        // Not publishable, open or not.
        let editor = f.editor.clone();
        let sheet = f
            .window
            .update(cx, |_, window, cx| {
                crate::publish::open_publish(editor, window, cx)
            })
            .unwrap();
        assert!(sheet.is_none());

        // Locked again, its notes close; opening the notebook asks once.
        f.lock_now(cx);
        assert_eq!(f.view(cx), LockView::Closed(None));
        assert_eq!(f.text(cx), "");
        let form = f.open_notebook(cx).expect("a closed notebook asks");
        f.submit(cx, &form, &["battery staple"]);
        assert_eq!(f.view(cx), LockView::Closed(None));
        f.submit(cx, &form, &[PASSWORD]);
        assert_eq!(f.view(cx), LockView::Open);
        assert_eq!(f.text(cx), "The secret plan.");
        assert!(
            f.open_notebook(cx).is_none(),
            "open already: it doesn't ask again"
        );
        f.close_dialog(cx);

        // A note made in it is sealed as soon as it shows.
        let made = f
            .store
            .put(
                DocType::Note,
                json!({"title": "New", "body": "", "bookId": f.deeper}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        f.select(cx, &made);
        assert_eq!(f.view(cx), LockView::Open);
        assert_eq!(f.sealed_text(&made).as_deref(), Some(""));

        // A note moved in goes sealed, and stays open.
        f.select(cx, &f.outside.clone());
        assert_eq!(f.view(cx), LockView::Plain);
        let book = f.private.clone();
        f.editor.update(cx, |editor, cx| editor.set_book(book, cx));
        cx.run_until_parked();
        assert_eq!(f.sealed_text(&f.outside).as_deref(), Some("Plain words."));
        assert_eq!(f.view(cx), LockView::Open);
        assert_eq!(f.text(cx), "Plain words.");
        assert!(f.store.revisions(&f.outside).unwrap().is_empty());

        // A note in a locked notebook keeps its lock.
        f.window
            .update(cx, |_, window, cx| {
                remove_lock(f.editor.clone(), window, cx)
            })
            .unwrap();
        cx.run_until_parked();
        assert!(!f.dialog_open(cx));
        assert!(f.sealed_text(&f.outside).is_some());

        // Moved out, it stays locked.
        let open = f.open.clone();
        f.editor.update(cx, |editor, cx| editor.set_book(open, cx));
        cx.run_until_parked();
        assert_eq!(f.sealed_text(&f.outside).as_deref(), Some("Plain words."));

        // The notebook's lock taken off, its notes keep theirs.
        take_lock_off(&f.store, &f.private).unwrap();
        assert!(f.store.locked_books().unwrap().is_empty());
        assert_eq!(f.sealed_text(&f.plan).as_deref(), Some("The secret plan."));
    }

    #[gpui_kit::test]
    fn locking_a_notebook_never_reveals_a_background_tabs_text(cx: &mut TestAppContext) {
        let f = setup(cx, None);
        // The secret note stays open in the first tab, in the background;
        // the plain note shows beside it.
        let outside = f.outside.clone();
        f.open_in_new_tab(cx, &outside);
        let ids = cx.read(|cx| f.workspace.read(cx).tabs.ids());
        assert_eq!(ids, [f.plan.clone(), outside.clone()]);

        // Lock the notebook while the secret note is only a background tab.
        let form = f.lock(cx).expect("the first lock sets a password");
        f.submit(cx, &form, &[PASSWORD, PASSWORD]);
        // And forget the keys, as Lock Notes Now or ten minutes does.
        f.lock_now(cx);

        f.show_tab(cx, 0);
        assert_eq!(f.view(cx), LockView::Closed(None), "it opens closed");
        assert_eq!(f.text(cx), "", "no text is revealed");
        assert_eq!(
            f.store.note(&f.plan).unwrap().unwrap().body,
            "",
            "the store never has the text"
        );
        assert_eq!(
            f.sealed_text(&f.plan).as_deref(),
            Some("The secret plan."),
            "the text is sealed, not lost"
        );
    }

    #[gpui_kit::test]
    fn notes_left_plain_in_a_locked_notebook_are_sealed_once_it_opens(cx: &mut TestAppContext) {
        let f = setup(cx, None);
        let form = f.lock(cx).unwrap();
        f.submit(cx, &form, &[PASSWORD, PASSWORD]);
        f.lock_now(cx);
        // An older Kanso on another device writes a plain note into it.
        let late = f
            .store
            .put(
                DocType::Note,
                json!({"title": "Late", "body": "Written elsewhere.", "bookId": f.deeper}),
                Put::Upsert,
                Origin::Sync,
            )
            .unwrap()
            .id;
        assert_eq!(f.store.unsealed_in_locked_books().unwrap().len(), 1);
        // It shows closed, as the others do, until the notebook opens.
        f.select(cx, &late);
        assert_eq!(f.view(cx), LockView::Closed(None));
        assert_eq!(f.text(cx), "");
        let form = f.open_notebook(cx).unwrap();
        f.submit(cx, &form, &[PASSWORD]);
        assert!(f.store.unsealed_in_locked_books().unwrap().is_empty());
        assert_eq!(f.sealed_text(&late).as_deref(), Some("Written elsewhere."));
        assert_eq!(f.view(cx), LockView::Open);
        assert_eq!(f.text(cx), "Written elsewhere.");
    }

    #[gpui_kit::test]
    fn touch_id_opens_a_locked_notebook_without_the_password(cx: &mut TestAppContext) {
        let touch = crate::touch_id::FakeTouchId::new(true);
        let f = setup(cx, Some(touch.clone()));
        let form = f.lock(cx).unwrap();
        f.submit(cx, &form, &[PASSWORD, PASSWORD]);
        f.close_dialog(cx);
        f.window
            .update(cx, |_, window, cx| {
                crate::locked::turn_on_touch_id(window, cx)
            })
            .unwrap();
        cx.run_until_parked();
        f.lock_now(cx);
        assert_eq!(f.view(cx), LockView::Closed(None));
        assert!(f.open_notebook(cx).is_none(), "a fingerprint, not a form");
        cx.run_until_parked();
        assert_eq!(f.view(cx), LockView::Open);
        assert!(!f.dialog_open(cx));
        assert_eq!(touch.asked(), 2);
    }

    #[gpui_kit::test]
    fn showing_a_locked_notebook_asks_for_the_password(cx: &mut TestAppContext) {
        let f = setup(cx, None);
        let form = f.lock(cx).unwrap();
        f.submit(cx, &form, &[PASSWORD, PASSWORD]);
        f.lock_now(cx);
        let library = cx.read(|cx| f.workspace.read(cx).library.clone());
        let deeper = f.deeper.clone();
        library.update(cx, |library, cx| {
            library.set_filter(Filter::Book(deeper), cx)
        });
        cx.run_until_parked();
        assert!(f.dialog_open(cx), "the notebook asks as it opens");
        f.close_dialog(cx);
        let open = f.open.clone();
        library.update(cx, |library, cx| library.set_filter(Filter::Book(open), cx));
        cx.run_until_parked();
        assert!(!f.dialog_open(cx));
    }
}
