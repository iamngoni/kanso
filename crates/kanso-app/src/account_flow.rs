//! The account tasks (docs/ACCOUNTS.md › Account flows, › Library binding,
//! › Services). Each runs on a thread named `kanso-account`, reports its
//! steps to [`Services`] (the Account page's button) and to the form that
//! started it, and finishes on the main thread. A task that re-binds the
//! library or touches its Keychain item stops and joins the running sync
//! first, on its own thread, with "Stopping sync…" as its step.
//!
//! Without a backend and a Keychain in [`Services`] (tests and snapshots)
//! no task starts: it ends at once with "Can't reach the server".

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use async_channel::{Receiver, Sender};
use gpui_kit::component::button::Button;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_store::Store;
use kanso_sync::keychain;
use kanso_sync::{
    AccountError, ApprovalKeys, Backend, BindError, Choice, DeviceIdentity, DeviceInfo, Flow,
    LocalLibrary, LockedSession, Machine, Password, Plan, RecoveryKey, RequestState, RotationError,
    SecretError, SecretStore, Session, SignOutReason, SignedUp, Step, cancel_request,
    complete_request, finish_unlock, meta_keys, name_device, poll_approval, request_approval,
};

use crate::account_words::{self as words, Answer, ErrorContext, Question};
use crate::clock::Clock;
use crate::config::{AccountRef, is_default_server};
use crate::dialogs::{self, Described, Progress};
use crate::services::{
    self, AccountDetails, AccountState, PendingKey, RunningTask, Services, TaskKind, notify_windows,
};
use crate::settings::{OpenAccountSettings, SettingsPage};
use crate::workspace::AppState;

/// When the library itself couldn't be changed.
const LIBRARY_TROUBLE: &str = "Kanso couldn't update this device's library. Check there is free space on this device, then try again.";
/// When a task's thread stopped without an answer.
const SOMETHING_WRONG: &str = "Something went wrong. Try again.";
/// When another account task is still running.
const BUSY: &str = "Kanso is still busy with your account. Try again in a moment.";

// MARK: Running a task

/// Why a task failed: the words to show, the reason when the server ended
/// this Mac's session on the way, and a new recovery key the server took
/// or may have taken, which shows all the same.
#[derive(Debug)]
struct Failure {
    words: String,
    signed_out: Option<SignOutReason>,
    recovery: Option<Box<PendingKey>>,
    /// The backup a Replace made before it failed: said all the same.
    backup: Option<PathBuf>,
}

impl Failure {
    fn account(err: AccountError, context: ErrorContext) -> Failure {
        // The details go to the log; the words never repeat them.
        eprintln!("kanso: account: {err}");
        let signed_out = match &err {
            AccountError::Unauthorized { reason } => Some(*reason),
            _ => None,
        };
        Failure {
            words: words::error_words(&err, context),
            signed_out,
            recovery: None,
            backup: None,
        }
    }

    /// A call that sets a new recovery key failed. The key it returns
    /// shows, with `note` when Kanso couldn't tell whether the server took
    /// it.
    fn rotation(err: RotationError, context: ErrorContext, note: &'static str) -> Failure {
        Failure {
            recovery: err
                .recovery_key
                .map(|item| Box::new(PendingKey::kept(*item, note))),
            ..Failure::account(err.error, context)
        }
    }

    /// The server took `key`, then this Mac failed: `words` say what
    /// happened instead, and the key shows with them.
    fn after_key(self, words: &'static str, session: &Session, key: RecoveryKey) -> Failure {
        Failure {
            words: words.to_owned(),
            recovery: Some(Box::new(PendingKey {
                key,
                username: session.username.clone(),
                server: session.server.clone(),
                note: Some(words),
            })),
            ..self
        }
    }

    fn keychain(err: SecretError) -> Failure {
        Failure::account(AccountError::Keychain(err), ErrorContext::Account)
    }

    fn library(err: impl std::fmt::Display) -> Failure {
        eprintln!("kanso: account: the library: {err}");
        Failure {
            words: LIBRARY_TROUBLE.to_owned(),
            signed_out: None,
            recovery: None,
            backup: None,
        }
    }

    fn bind(err: BindError) -> Failure {
        match err {
            BindError::Store(err) => Failure::library(err),
            BindError::Keychain(err) => Failure::keychain(err),
        }
    }
}

/// What a task's thread sends back.
enum Message<T> {
    Step(Step),
    Done(T),
}

/// What the tasks reach the world through, taken from [`Services`] and
/// [`AppState`] on the main thread.
#[derive(Clone)]
struct Seams {
    backend: Arc<dyn Backend>,
    secrets: Arc<dyn SecretStore>,
    store: Arc<Store>,
    machine: Option<Machine>,
    backups: PathBuf,
    now: i64,
}

impl Seams {
    fn get(cx: &App) -> Option<Seams> {
        let services = cx.try_global::<Services>()?;
        let state = cx.try_global::<AppState>()?;
        Some(Seams {
            backend: services.backend.clone()?,
            secrets: services.secrets.clone()?,
            store: state.store.clone(),
            machine: services.machine.clone(),
            backups: crate::paths::backups_dir(&state.data_dir),
            now: Clock::now_ms(cx),
        })
    }

    /// This library's identity on this Mac (on the task's thread: reading
    /// the Mac runs `ioreg` and `scutil`).
    fn device(&self) -> Result<DeviceIdentity, Failure> {
        let machine = self.machine.clone().unwrap_or_else(Machine::this_mac);
        kanso_sync::device_identity(&self.store, &machine).map_err(Failure::library)
    }
}

/// Starts `work` on its own thread as the task `kind`, at step `first`.
/// With `stop_sync`, the running sync is cancelled and joined there first,
/// and started again once the task, and any task its `finish` started, has
/// left the same session signed in (a failure, or a cancelled question).
/// Steps reach [`Services`] and
/// `progress`; `finish` gets the result on the main thread (`None` if the
/// thread stopped without one).
fn run<T: Send + 'static>(
    kind: TaskKind,
    first: Step,
    stop_sync: bool,
    progress: Option<Sender<Progress>>,
    cx: &mut App,
    work: impl FnOnce(&dyn Fn(Step)) -> T + Send + 'static,
    finish: impl FnOnce(Option<T>, &mut App) + 'static,
) {
    let services = cx.global_mut::<Services>();
    let sync = if stop_sync {
        services.take_sync_handle()
    } else {
        None
    };
    // A chain of tasks remembers the session of the first stop.
    if sync.is_some() && services.stopped_for.is_none() {
        services.stopped_for = services.session();
    }
    let first = if sync.is_some() {
        Step::StoppingSync
    } else {
        first
    };
    services.task = Some(RunningTask { kind, step: first });
    if let Some(progress) = &progress {
        progress.try_send(Progress::Step(first)).ok();
    }
    let (tx, rx) = async_channel::unbounded::<Message<T>>();
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            let step = |step: Step| {
                tx.try_send(Message::Step(step)).ok();
            };
            if let Some(sync) = sync {
                sync.cancel_and_join();
            }
            let result = work(&step);
            tx.try_send(Message::Done(result)).ok();
        });
    if let Err(err) = spawned {
        // The sync handle went with the closure: that sync runs on,
        // detached, to its end. Nothing was changed.
        eprintln!("kanso: couldn't start an account task: {err}");
        cx.global_mut::<Services>().task = None;
        finish(None, cx);
        resume_stopped_sync(cx);
        cx.refresh_windows();
        return;
    }
    cx.refresh_windows();
    cx.spawn(async move |cx| {
        let mut result = None;
        while let Ok(message) = rx.recv().await {
            match message {
                Message::Step(step) => {
                    if let Some(progress) = &progress {
                        progress.try_send(Progress::Step(step)).ok();
                    }
                    cx.update(|cx| {
                        if let Some(task) = &mut cx.global_mut::<Services>().task {
                            task.step = step;
                        }
                        cx.refresh_windows();
                    });
                }
                Message::Done(value) => {
                    result = Some(value);
                    break;
                }
            }
        }
        cx.update(|cx| {
            cx.global_mut::<Services>().task = None;
            finish(result, cx);
            resume_stopped_sync(cx);
            check_unconfirmed_key(cx);
            cx.refresh_windows();
        });
    })
    .detach();
}

/// Once no task runs (a task's `finish` may have started the next of its
/// chain), the sync a task stopped starts again if its session is still
/// signed in (A8).
fn resume_stopped_sync(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    if services.task.is_some() {
        return;
    }
    let Some(before) = services.stopped_for.take() else {
        return;
    };
    if services
        .session()
        .is_some_and(|now| Arc::ptr_eq(&now, &before))
    {
        services::resume_sync(cx);
    }
}

fn done(tx: &Sender<Progress>, result: Result<(), String>) {
    tx.try_send(Progress::Done(result)).ok();
}

/// Ends a task with its failure; a session the server ended is ended here
/// too, and a new recovery key that came with the failure shows.
fn failed(tx: &Sender<Progress>, failure: Option<Failure>, cx: &mut App) {
    let Some(failure) = failure else {
        done(tx, Err(SOMETHING_WRONG.to_owned()));
        return;
    };
    if let Some(reason) = failure.signed_out
        && cx.global::<Services>().signed_in()
    {
        services::end_session(reason, cx);
    }
    if let Some(key) = failure.recovery {
        show_pending(*key, cx);
    }
    if let Some(path) = failure.backup {
        notify_backup(path, cx);
    }
    done(tx, Err(failure.words));
}

/// Saves `session` as the library's Keychain item, then binds the library
/// with `choice` (A1, sync 7): a library is never bound to an account whose
/// item is missing. When binding fails the item is put back as it was (gone,
/// or another account's), and a backup a Replace made first comes back with
/// the failure. `None` leaves the binding as it is.
fn save_and_bind(
    s: &Seams,
    library_id: &str,
    session: &Session,
    choice: Option<Choice>,
) -> Result<Option<PathBuf>, Failure> {
    let item = keychain::account_item(library_id);
    let before = s.secrets.get(&item).map_err(Failure::keychain)?;
    keychain::save_session(s.secrets.as_ref(), library_id, session).map_err(Failure::keychain)?;
    let Some(choice) = choice else {
        return Ok(None);
    };
    let backups_before = backups_in(&s.backups);
    match kanso_sync::bind(&s.store, session, choice, &s.backups, s.now) {
        Ok(backup) => Ok(backup),
        Err(err) => {
            let put_back = match &before {
                Some(raw) => s.secrets.set(&item, raw),
                None => s.secrets.delete(&item),
            };
            if let Err(err) = put_back {
                eprintln!("kanso: putting the library's Keychain item back: {err}");
            }
            let mut failure = Failure::bind(err);
            if choice == Choice::Replace {
                failure.backup = backups_in(&s.backups)
                    .into_iter()
                    .find(|path| !backups_before.contains(path));
            }
            Err(failure)
        }
    }
}

/// The files in the backups folder, to find the one a failed Replace made.
fn backups_in(dir: &std::path::Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    paths
}

/// The seams, or the task's end when there are none or a task runs.
fn begin(tx: &Sender<Progress>, cx: &App) -> Option<Seams> {
    if cx
        .try_global::<Services>()
        .is_some_and(|services| services.task.is_some() || services.restoring)
    {
        done(tx, Err(BUSY.to_owned()));
        return None;
    }
    let seams = Seams::get(cx);
    if seams.is_none() {
        done(tx, Err(words::CANT_REACH.to_owned()));
    }
    seams
}

/// The seams and the session, for a task while signed in.
fn begin_signed_in(tx: &Sender<Progress>, cx: &App) -> Option<(Seams, Arc<Session>, String)> {
    let seams = begin(tx, cx)?;
    let services = cx.global::<Services>();
    match (services.session(), services.library_id.clone()) {
        (Some(session), Some(library_id)) => Some((seams, session, library_id)),
        _ => {
            done(tx, Err(SOMETHING_WRONG.to_owned()));
            None
        }
    }
}

/// Asks `question` in `window`, then calls `answer` once with the choice
/// (Cancel when the dialog is dismissed or the window is gone).
fn ask(
    window: AnyWindowHandle,
    question: Question,
    cx: &mut App,
    answer: impl FnOnce(Choice, &mut App) + 'static,
) {
    type Reply = Rc<RefCell<Option<Box<dyn FnOnce(Choice, &mut App)>>>>;
    let reply: Reply = Rc::new(RefCell::new(Some(Box::new(answer))));
    let answered = reply.clone();
    let choices: Vec<Choice> = question.answers.iter().map(|a| a.choice).collect();
    let described: Vec<Described> = question
        .answers
        .iter()
        .map(|a| Described {
            label: a.label.clone().into(),
            description: a.description.clone().into(),
        })
        .collect();
    let Question { title, body, .. } = question;
    let shown = window.update(cx, move |_, window, cx| {
        dialogs::choose(title, body, described, window, cx, move |picked, _, cx| {
            let choice = picked
                .and_then(|ix| choices.get(ix).copied())
                .unwrap_or(Choice::Cancel);
            let answer = answered.borrow_mut().take();
            if let Some(answer) = answer {
                answer(choice, cx);
            }
        });
    });
    if shown.is_err() {
        let answer = reply.borrow_mut().take();
        if let Some(answer) = answer {
            answer(Choice::Cancel, cx);
        }
    }
}

// MARK: Signing in

/// The new recovery key, shown on the Account page until "I've saved it".
fn show_key(session: &Session, key: RecoveryKey, cx: &mut App) {
    show_pending(
        PendingKey {
            key,
            username: session.username.clone(),
            server: session.server.clone(),
            note: None,
        },
        cx,
    );
}

/// Shows `key` on the Account page until "I've saved it". When that page
/// isn't showing, a notification leads there.
fn show_pending(key: PendingKey, cx: &mut App) {
    let services = cx.global_mut::<Services>();
    services.key_check_due = key.note.and_then(words::key_checked).is_some();
    services.pending_key = Some(key);
    // Once the task that showed it is done, or the launch check.
    cx.defer(check_unconfirmed_key);
    // Windows can't be read from inside their own updates.
    cx.defer(|cx| {
        if !account_page_showing(cx) {
            notify_windows(
                || {
                    Notification::warning(words::KEY_READY)
                        .id::<KeyReady>()
                        .action(|_, _, _| {
                            Button::new("show-recovery-key")
                                .label("Show key")
                                .debug_selector(|| "show-recovery-key".into())
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(Box::new(OpenAccountSettings), cx)
                                })
                        })
                },
                cx,
            );
        }
    });
}

/// The "Show key" notification's ID: a second key replaces it.
struct KeyReady;

/// Asks the server once whether it took the shown key, when Kanso couldn't
/// tell (review-verify 6). A key the server holds is the account's now, and
/// the screen says so; one it doesn't hold goes, screen and Keychain item,
/// since nothing changed, and a notification says that. It waits for a
/// running task and the launch check, and is asked once per key shown (and
/// so once per launch for a key kept from before): `recover/start` counts
/// against the server's per-address limit. A failed ask leaves the key as
/// it is, its words saying to keep the old key too.
fn check_unconfirmed_key(cx: &mut App) {
    let Some(services) = cx.try_global::<Services>() else {
        return;
    };
    if !services.key_check_due || services.task.is_some() || services.restoring {
        return;
    }
    let Some(shown) = services.pending_key.clone() else {
        return;
    };
    let Some(checked) = shown.note.and_then(words::key_checked) else {
        return;
    };
    let Some(seams) = Seams::get(cx) else {
        return;
    };
    let library_id = services.library_id.clone();
    cx.global_mut::<Services>().key_check_due = false;
    let asked = shown.key.clone();
    run(
        TaskKind::CheckKey,
        Step::Contacting,
        false,
        None,
        cx,
        move |_| {
            let held = kanso_sync::recovery_key_held(
                seams.backend.as_ref(),
                &shown.server,
                &shown.username,
                &shown.key,
            )?;
            settle_kept_key(&seams, library_id, &shown.key, held);
            Ok::<bool, AccountError>(held)
        },
        move |result, cx| {
            let services = cx.global_mut::<Services>();
            let same = services
                .pending_key
                .as_ref()
                .is_some_and(|now| now.key == asked);
            match result {
                Some(Ok(true)) => {
                    if let Some(now) = services.pending_key.as_mut().filter(|_| same) {
                        now.note = Some(checked.taken);
                    }
                }
                Some(Ok(false)) => {
                    if same {
                        services.pending_key = None;
                    }
                    // In place of its "Show key", if that still shows.
                    notify_windows(
                        move || Notification::warning(checked.not_taken).id::<KeyReady>(),
                        cx,
                    );
                }
                Some(Err(err)) => {
                    eprintln!("kanso: asking whether the server took the recovery key: {err}");
                }
                None => {}
            }
        },
    );
}

/// The Keychain's unsaved key once the server answered about `key`:
/// confirmed when the server holds it, gone when it doesn't. An item that
/// holds another key by then, or is confirmed already, stays as it is.
fn settle_kept_key(s: &Seams, library_id: Option<String>, key: &RecoveryKey, held: bool) {
    let library_id = match library_id {
        Some(library_id) => library_id,
        None => match s.device() {
            Ok(device) => device.library_id,
            Err(_) => return,
        },
    };
    let secrets = s.secrets.as_ref();
    let settled =
        keychain::load_any_recovery_key(secrets, &library_id).and_then(|kept| match kept {
            Some(mut kept) if kept.unconfirmed && kept.recovery_key == *key => {
                if held {
                    kept.unconfirmed = false;
                    keychain::save_recovery_key(secrets, &library_id, &kept)
                } else {
                    keychain::delete_recovery_key(secrets, &library_id)
                }
            }
            _ => Ok(()),
        });
    if let Err(err) = settled {
        eprintln!("kanso: settling the unconfirmed recovery key: {err}");
    }
}

/// Whether a window shows Settings › Account.
fn account_page_showing(cx: &mut App) -> bool {
    let mut showing = false;
    crate::workspace::for_each_workspace(cx, |workspace, cx| {
        showing |= workspace.settings_open()
            && workspace.settings.read(cx).page() == SettingsPage::Account;
    });
    showing
}

/// After the binding and the Keychain item: the config, the window, and
/// sync (docs/ACCOUNTS.md › Library binding › Order and cancellation).
/// `turn_on` sets Sync automatically, as signing in does.
fn signed_in(session: Session, library_id: String, turn_on: bool, cx: &mut App) {
    let account = AccountRef {
        origin: session.origin.clone(),
        id: session.account_id.clone(),
        username: session.username.clone(),
    };
    AppState::update_config(cx, |config| {
        config.sync.account = Some(account);
        if turn_on {
            config.sync.enabled = true;
        }
    });
    let services = cx.global_mut::<Services>();
    services.library_id = Some(library_id);
    services.set_signed_in(Arc::new(session));
    services::apply_sync_settings(cx);
    refresh_details(cx);
    crate::device_approval::start_events(cx);
    crate::device_approval::refresh(cx);
}

/// Signed out here: the session, the details and the config's account go;
/// the notes stay.
fn forget_account(cx: &mut App) {
    crate::device_approval::stop_events(cx);
    let services = cx.global_mut::<Services>();
    services.clear_session(AccountState::SignedOut);
    services.pending_key = None;
    crate::collab::signed_out(cx);
    AppState::update_config(cx, |config| config.sync.account = None);
    cx.refresh_windows();
}

/// Creates an account on `server` and signs this Mac in with it (Settings
/// › Account › Create account). Notes that came from another account are
/// asked about in `window` first.
pub fn sign_up(
    server: String,
    username: String,
    password: Password,
    window: AnyWindowHandle,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let origin = kanso_sync::origin(&server).unwrap_or_default();
    let thread = seams.clone();
    run(
        TaskKind::SignUp,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        // No account ID yet: a new account is never the library's own.
        move |_| kanso_sync::local_library(&thread.store, &origin, "").map_err(Failure::library),
        move |result, cx| match result {
            Some(Ok(library)) => match kanso_sync::plan(Flow::SignUp, &library) {
                Plan::AskBeforeSignUp { from, .. } => {
                    let (title, body, upload) = words::upload_question(&from);
                    let question = Question {
                        title,
                        body,
                        answers: vec![Answer {
                            choice: Choice::Add,
                            label: upload.to_owned(),
                            description: "They're encrypted and uploaded to the new account. Nothing changes in the other one.".to_owned(),
                        }],
                    };
                    ask(window, question, cx, move |choice, cx| match choice {
                        Choice::Cancel => done(&tx, Ok(())),
                        _ => create(seams, server, username, password, tx, cx),
                    });
                }
                _ => create(seams, server, username, password, tx, cx),
            },
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

fn create(
    seams: Seams,
    server: String,
    username: String,
    password: Password,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let thread = seams.clone();
    run(
        TaskKind::SignUp,
        Step::DerivingKeys,
        true,
        Some(tx.clone()),
        cx,
        move |step| {
            let s = thread;
            let device = s.device()?;
            // The new key is in the Keychain before the request goes.
            let signed = kanso_sync::sign_up(
                s.backend.as_ref(),
                s.secrets.as_ref(),
                &server,
                &username,
                &password,
                &device,
                step,
            )
            .map_err(|err| {
                Failure::rotation(err, ErrorContext::SignUp, words::KEY_UNCONFIRMED_SIGN_UP)
            })?;
            step(Step::Saving);
            let bound = save_and_bind(&s, &device.library_id, &signed.session, Some(Choice::Add));
            if let Err(failure) = bound {
                // Nothing here uses the new session: the server forgets it.
                // The account stays, and so does its recovery key, in the
                // Keychain and on screen.
                kanso_sync::discard_session(
                    s.backend.as_ref(),
                    s.secrets.as_ref(),
                    &signed.session,
                )
                .ok();
                let SignedUp {
                    session,
                    recovery_key,
                } = signed;
                return Err(failure.after_key(
                    words::CREATED_NOT_SIGNED_IN,
                    &session,
                    recovery_key,
                ));
            }
            Ok((signed, device.library_id))
        },
        move |result, cx| match result {
            Some(Ok((signed, library_id))) => {
                let SignedUp {
                    session,
                    recovery_key,
                } = signed;
                show_key(&session, recovery_key, cx);
                signed_in(session, library_id, true, cx);
                done(&tx, Ok(()));
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
}

/// An account this Mac has proved it may use, before anything is saved.
struct Authenticated {
    session: Session,
    evicted: Option<DeviceInfo>,
    library: LocalLibrary,
    library_id: String,
    /// A recovery's new key, in the Keychain already.
    recovery_key: Option<RecoveryKey>,
}

type Authenticate = Box<
    dyn FnOnce(
            &Seams,
            &DeviceIdentity,
            &dyn Fn(Step),
        ) -> Result<(Session, Option<DeviceInfo>, Option<RecoveryKey>), Failure>
        + Send,
>;

/// Signs this Mac in to `username`'s account (Settings › Account › Sign
/// in), asking in `window` what to do with notes already here.
pub fn sign_in(
    server: String,
    username: String,
    password: Password,
    window: AnyWindowHandle,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    authenticate(
        Flow::SignIn,
        Some(window),
        seams,
        tx,
        cx,
        Box::new(move |s, device, step| {
            let signed = kanso_sync::sign_in(
                s.backend.as_ref(),
                &server,
                &username,
                &password,
                device,
                step,
            )
            .map_err(|err| Failure::account(err, ErrorContext::SignIn))?;
            Ok((signed.session, signed.evicted, None))
        }),
    );
    rx
}

/// Sets a new password with the recovery key, and signs this Mac in
/// ("Forgot your password?"). Signed out, it asks in `window` what to do
/// with notes already here; signed in, it replaces this Mac's session.
pub fn recover(
    server: String,
    username: String,
    key: RecoveryKey,
    password: Password,
    window: Option<AnyWindowHandle>,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    authenticate(
        Flow::Recover,
        window,
        seams,
        tx,
        cx,
        Box::new(move |s, device, step| {
            // The new key is in the Keychain before the request goes.
            let recovered = kanso_sync::recover(
                s.backend.as_ref(),
                s.secrets.as_ref(),
                &server,
                &username,
                &key,
                &password,
                device,
                step,
            )
            .map_err(|err| {
                Failure::rotation(err, ErrorContext::Recovery, words::KEY_UNCONFIRMED_PASSWORD)
            })?;
            Ok((recovered.session, None, Some(recovered.recovery_key)))
        }),
    );
    rx
}

/// "Forgot your password?" while signed in: the account and server are this
/// Mac's own.
pub fn recover_signed_in(key: RecoveryKey, password: Password, cx: &mut App) -> Receiver<Progress> {
    let account = cx
        .global::<Services>()
        .session()
        .map(|session| (session.server.clone(), session.username.clone()));
    match account {
        Some((server, username)) => recover(server, username, key, password, None, cx),
        None => {
            let (tx, rx) = async_channel::unbounded();
            done(&tx, Err(SOMETHING_WRONG.to_owned()));
            rx
        }
    }
}

fn authenticate(
    flow: Flow,
    window: Option<AnyWindowHandle>,
    seams: Seams,
    tx: Sender<Progress>,
    cx: &mut App,
    auth: Authenticate,
) {
    let kind = match flow {
        Flow::SignUp => TaskKind::SignUp,
        Flow::SignIn => TaskKind::SignIn,
        Flow::Recover => TaskKind::Recover,
    };
    // Signed in, the running sync stops first: its session is about to be
    // replaced, and nothing may sync in between.
    let was_signed_in = cx.global::<Services>().signed_in();
    let thread = seams.clone();
    run(
        kind,
        Step::Contacting,
        was_signed_in,
        Some(tx.clone()),
        cx,
        move |step| {
            let s = thread;
            let device = s.device()?;
            let (session, evicted, recovery_key) = auth(&s, &device, step)?;
            let library =
                match kanso_sync::local_library(&s.store, &session.origin, &session.account_id) {
                    Ok(library) => library,
                    Err(err) => {
                        let failure = Failure::library(err);
                        kanso_sync::discard_session(
                            s.backend.as_ref(),
                            s.secrets.as_ref(),
                            &session,
                        )
                        .ok();
                        return Err(match recovery_key {
                            Some(key) => {
                                failure.after_key(words::CHANGED_NOT_SIGNED_IN, &session, key)
                            }
                            None => failure,
                        });
                    }
                };
            Ok(Authenticated {
                session,
                evicted,
                library,
                library_id: device.library_id,
                recovery_key,
            })
        },
        move |result, cx| {
            let auth = match result {
                Some(Ok(authenticated)) => authenticated,
                Some(Err(failure)) => return failed(&tx, Some(failure), cx),
                None => return failed(&tx, None, cx),
            };
            // A used recovery key is gone: the new one shows at once, even if
            // the binding question is cancelled.
            if let Some(key) = &auth.recovery_key {
                show_key(&auth.session, key.clone(), cx);
            }
            let plan = kanso_sync::plan(flow, &auth.library);
            let question = words::binding_question(&plan, &auth.session.username);
            match (plan, question, window) {
                (Plan::Ask { .. }, Some(question), Some(window)) => {
                    ask(window, question, cx, move |choice, cx| match choice {
                        Choice::Cancel => abandon(seams, auth.session, tx, cx),
                        choice => {
                            bind_and_save(kind, seams, auth, Some(choice), was_signed_in, tx, cx)
                        }
                    });
                }
                (Plan::Ask { .. }, _, _) => abandon(seams, auth.session, tx, cx),
                (Plan::Resume, _, _) => {
                    bind_and_save(kind, seams, auth, None, was_signed_in, tx, cx)
                }
                (Plan::Fresh | Plan::AskBeforeSignUp { .. }, _, _) => {
                    bind_and_save(kind, seams, auth, Some(Choice::Add), was_signed_in, tx, cx)
                }
            }
        },
    );
}

/// Binds the library (unless `choice` is `None`: it already is), saves the
/// Keychain item, then signs the window in.
fn bind_and_save(
    kind: TaskKind,
    seams: Seams,
    auth: Authenticated,
    choice: Option<Choice>,
    was_signed_in: bool,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let Authenticated {
        session,
        evicted,
        library_id,
        recovery_key,
        ..
    } = auth;
    // A Replace backs the library up, then empties it: a write in between
    // would be in neither. Edits so far are saved into the backup first;
    // the editors and the local API wait until it is done (A10). The
    // editors are reached once the window that chose Replace has finished
    // its update.
    let replacing = choice == Some(Choice::Replace);
    let begin = move |cx: &mut App| {
        let (api, restart_api) = if replacing {
            services::hold_writes(cx)
        } else {
            (None, false)
        };
        let thread = seams.clone();
        run(
            kind,
            Step::Saving,
            true,
            Some(tx.clone()),
            cx,
            move |step| {
                let s = thread;
                step(Step::Saving);
                if let Some(api) = api
                    && let Err(err) = api.stop()
                {
                    eprintln!("kanso: the API server stopped with an error: {err:#}");
                }
                match save_and_bind(&s, &library_id, &session, choice) {
                    Ok(backup) => Ok((session, library_id, backup)),
                    Err(failure) => {
                        // Nothing here uses the new session. The library's
                        // Keychain items are as they were: maybe another
                        // account's.
                        kanso_sync::discard_session(
                            s.backend.as_ref(),
                            s.secrets.as_ref(),
                            &session,
                        )
                        .ok();
                        // The password changed all the same.
                        Err(match recovery_key {
                            Some(key) => {
                                failure.after_key(words::CHANGED_NOT_SIGNED_IN, &session, key)
                            }
                            None => failure,
                        })
                    }
                }
            },
            move |result, cx| {
                if replacing {
                    services::release_writes(restart_api, cx);
                }
                bound(result, evicted, was_signed_in, tx, cx)
            },
        );
    };
    if replacing {
        cx.defer(begin);
    } else {
        begin(cx);
    }
}

/// What binding gives: the session, the library's ID, and a Replace's
/// backup.
type Bound = (Session, String, Option<PathBuf>);

/// The end of [`bind_and_save`]: signed in, or the failure.
fn bound(
    result: Option<Result<Bound, Failure>>,
    evicted: Option<DeviceInfo>,
    was_signed_in: bool,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    match result {
        Some(Ok((session, library_id, backup))) => {
            signed_in(session, library_id, !was_signed_in, cx);
            if let Some(device) = evicted {
                let notice = words::evicted_notice(&device);
                notify_windows(move || Notification::info(notice.clone()), cx);
            }
            if let Some(path) = backup {
                notify_backup(path, cx);
            }
            done(&tx, Ok(()));
        }
        Some(Err(failure)) => failed(&tx, Some(failure), cx),
        None => failed(&tx, None, cx),
    }
}

/// The binding question was cancelled: the new session is logged out, and
/// nothing here changes. The library's Keychain items stay: they hold the
/// account this Mac may still be signed in to, and a recovery key that may
/// still be on screen.
fn abandon(seams: Seams, session: Session, tx: Sender<Progress>, cx: &mut App) {
    run(
        TaskKind::SignOut,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| {
            if let Err(err) = kanso_sync::discard_session(
                seams.backend.as_ref(),
                seams.secrets.as_ref(),
                &session,
            ) {
                eprintln!("kanso: signing out the cancelled session: {err}");
            }
        },
        move |_, _| done(&tx, Ok(())),
    );
}

/// "Saved a backup of this Mac's library first", with Show in Finder.
fn notify_backup(path: PathBuf, cx: &mut App) {
    let shown = crate::settings::display_path(&path, &crate::paths::home());
    notify_windows(
        move || {
            let path = path.clone();
            Notification::success(format!(
                "Kanso saved a backup of this device's library first, in {shown}."
            ))
            .action(move |_, _, _| {
                let path = path.clone();
                Button::new("show-backup")
                    .label(crate::paths::SHOW_IN_FILE_MANAGER)
                    .on_click(move |_, _, cx| cx.reveal_path(&path))
            })
        },
        cx,
    );
}

// MARK: Signed in

/// Signs this Mac out of its account. Notes stay; sync stops until it signs
/// in again.
pub fn sign_out(cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some((seams, session, _)) = begin_signed_in(&tx, cx) else {
        return rx;
    };
    run(
        TaskKind::SignOut,
        Step::Contacting,
        true,
        Some(tx.clone()),
        cx,
        move |_| kanso_sync::sign_out(seams.backend.as_ref(), seams.secrets.as_ref(), &session),
        move |result, cx| {
            // Signed out here whatever the server said: without the config's
            // account, a leftover Keychain item is never loaded again. The
            // logout was tried (or queued) either way.
            if let Some(Err(err)) = &result {
                eprintln!("kanso: signing out: {err}");
                if matches!(err, AccountError::Keychain(_)) {
                    notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
                }
            }
            forget_account(cx);
            done(&tx, Ok(()));
        },
    );
    rx
}

/// Changes the account's password, and with it the recovery key: the
/// other Macs ask for the new password, and the Account page shows the new
/// key.
pub fn change_password(current: Password, new: Password, cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some((seams, session, _)) = begin_signed_in(&tx, cx) else {
        return rx;
    };
    let shown = session.clone();
    run(
        TaskKind::ChangePassword,
        Step::DerivingKeys,
        false,
        Some(tx.clone()),
        cx,
        move |step| {
            // The new key is in the Keychain before the request goes.
            kanso_sync::change_password(
                seams.backend.as_ref(),
                seams.secrets.as_ref(),
                &session,
                &current,
                &new,
                step,
            )
            .map_err(|err| {
                Failure::rotation(
                    err,
                    ErrorContext::CurrentPassword,
                    words::KEY_UNCONFIRMED_PASSWORD,
                )
            })
        },
        move |result, cx| match result {
            Some(Ok(changed)) => {
                show_pending(
                    PendingKey {
                        key: changed.recovery_key,
                        username: shown.username.clone(),
                        server: shown.server.clone(),
                        note: Some(words::KEY_AFTER_PASSWORD),
                    },
                    cx,
                );
                notify_windows(|| Notification::success(words::PASSWORD_CHANGED), cx);
                done(&tx, Ok(()));
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

/// Replaces the recovery key; the Account page then shows the new one.
pub fn new_recovery_key(current: Password, cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some((seams, session, _)) = begin_signed_in(&tx, cx) else {
        return rx;
    };
    let shown = session.clone();
    run(
        TaskKind::NewRecoveryKey,
        Step::DerivingKeys,
        false,
        Some(tx.clone()),
        cx,
        move |step| {
            // The new key is in the Keychain before the request goes.
            kanso_sync::new_recovery_key(
                seams.backend.as_ref(),
                seams.secrets.as_ref(),
                &session,
                &current,
                step,
            )
            .map_err(|err| {
                Failure::rotation(err, ErrorContext::CurrentPassword, words::KEY_UNCONFIRMED)
            })
        },
        move |result, cx| match result {
            Some(Ok(key)) => {
                show_key(&shown, key, cx);
                done(&tx, Ok(()));
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

/// Deletes the account and its server data, then unbinds the library.
/// Notes on this Mac stay.
pub fn delete_account(password: Password, typed: String, cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some((seams, session, library_id)) = begin_signed_in(&tx, cx) else {
        return rx;
    };
    let username = session.username.clone();
    run(
        TaskKind::DeleteAccount,
        Step::DerivingKeys,
        true,
        Some(tx.clone()),
        cx,
        move |step| {
            kanso_sync::delete_account(seams.backend.as_ref(), &session, &password, &typed, step)
                .map_err(|err| Failure::account(err, ErrorContext::CurrentPassword))?;
            step(Step::Saving);
            kanso_sync::unbind(&seams.store, seams.secrets.as_ref()).map_err(Failure::bind)?;
            keychain::delete_recovery_key(seams.secrets.as_ref(), &library_id).ok();
            Ok(())
        },
        move |result, cx| match result {
            Some(Ok(())) => {
                forget_account(cx);
                notify_windows(
                    move || {
                        Notification::info(format!(
                            "Deleted {username}'s account. The notes on this device stay here."
                        ))
                    },
                    cx,
                );
                done(&tx, Ok(()));
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

/// Signs another device of the account out, and says how it went: nothing
/// else waits on it once its dialog closes (A5). A device the server no
/// longer knows is signed out already.
pub fn revoke_device(device_id: String, name: String, cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some((seams, session, _)) = begin_signed_in(&tx, cx) else {
        if let Ok(Progress::Done(Err(words))) = rx.try_recv() {
            let told = words.clone();
            notify_windows(move || Notification::error(told.clone()), cx);
            done(&tx, Err(words));
        }
        return rx;
    };
    run(
        TaskKind::RevokeDevice,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| match kanso_sync::revoke_device(seams.backend.as_ref(), &session, &device_id) {
            Ok(()) | Err(AccountError::NotFound) => Ok(()),
            Err(err) => Err(Failure::account(err, ErrorContext::Account)),
        },
        move |result, cx| {
            match result {
                Some(Ok(())) => {
                    let notice = words::revoked_notice(&name);
                    notify_windows(move || Notification::success(notice.clone()), cx);
                    done(&tx, Ok(()));
                }
                failure => {
                    let failure = failure.and_then(Result::err);
                    let told = failure
                        .as_ref()
                        .map_or(SOMETHING_WRONG.to_owned(), |failure| failure.words.clone());
                    notify_windows(move || Notification::error(told.clone()), cx);
                    failed(&tx, failure, cx);
                }
            }
            refresh_details(cx);
        },
    );
    rx
}

/// "I've saved my recovery key": the key screen and its Keychain item go,
/// unless the item holds another key by then.
pub fn key_saved(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    let saved = services.pending_key.take();
    let library_id = services.library_id.clone();
    if let (Some(saved), Some(seams)) = (saved, Seams::get(cx)) {
        cx.background_spawn(async move {
            // Before the launch check, as after a failed sign-up in tests.
            let library_id = match library_id {
                Some(library_id) => library_id,
                None => match seams.device() {
                    Ok(device) => device.library_id,
                    Err(_) => return,
                },
            };
            let secrets = seams.secrets.as_ref();
            let removed =
                keychain::load_any_recovery_key(secrets, &library_id).and_then(|kept| match kept {
                    Some(kept) if kept.recovery_key == saved.key => {
                        keychain::delete_recovery_key(secrets, &library_id)
                    }
                    _ => Ok(()),
                });
            if let Err(err) = removed {
                eprintln!("kanso: removing the saved recovery key: {err}");
            }
        })
        .detach();
    }
    cx.refresh_windows();
}

// MARK: Servers

/// The server the forms use: `None` for the default.
pub fn set_server(server: Option<String>, cx: &mut App) {
    let server = server.filter(|server| !is_default_server(server));
    AppState::update_config(cx, |config| config.sync.server = server);
    check_signup(cx);
    cx.refresh_windows();
}

/// "Use a different server…": the address must be a Kanso server, which
/// says whether it takes new accounts.
pub fn use_server(input: String, cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let endpoint = match kanso_sync::check_endpoint(&input) {
        Ok(endpoint) => endpoint,
        Err(err) => {
            done(&tx, Err(words::endpoint_words(&err).to_owned()));
            return rx;
        }
    };
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    run(
        TaskKind::CheckServer,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| {
            kanso_sync::server_info(seams.backend.as_ref(), &endpoint)
                .map(|signup| (endpoint, signup))
                .map_err(|err| Failure::account(err, ErrorContext::Server))
        },
        move |result, cx| match result {
            Some(Ok((endpoint, signup))) => {
                cx.global_mut::<Services>().signup = Some((endpoint.clone(), signup));
                AppState::update_config(cx, |config| {
                    config.sync.server = (!is_default_server(&endpoint)).then_some(endpoint)
                });
                done(&tx, Ok(()));
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

/// The server the forms use, as a URL.
pub fn form_server(cx: &App) -> String {
    cx.global::<AppState>()
        .config
        .sync
        .server
        .clone()
        .unwrap_or_else(|| kanso_sync::DEFAULT_SERVER.to_owned())
}

/// Asks the forms' server whether it takes new accounts, for the Create
/// account form: again each time the form opens, since a server's answer
/// changes as it fills (A6). One question per server at a time; an answer
/// is kept only if the form still shows that server.
pub fn check_signup(cx: &mut App) {
    let server = form_server(cx);
    let services = cx.global_mut::<Services>();
    let Some(backend) = services.backend.clone() else {
        return;
    };
    if services.signup_asking.as_deref() == Some(server.as_str()) {
        return;
    }
    services.signup_asking = Some(server.clone());
    let (tx, rx) = async_channel::bounded(1);
    let asked = server.clone();
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            tx.try_send(kanso_sync::server_info(backend.as_ref(), &asked))
                .ok();
        });
    if spawned.is_err() {
        cx.global_mut::<Services>().signup_asking = None;
        return;
    }
    cx.spawn(async move |cx| {
        let answer = rx.recv().await.ok();
        cx.update(|cx| {
            let shown = form_server(cx);
            let services = cx.global_mut::<Services>();
            if services.signup_asking.as_deref() == Some(server.as_str()) {
                services.signup_asking = None;
            }
            if let Some(Ok(signup)) = answer
                && shown == server
            {
                services.signup = Some((server, signup));
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

// MARK: Storage and devices

/// Fetches the account's details again after a sync that changed what the
/// server holds. A fetch already running may have been answered before the
/// sync's changes landed, so it is followed by another.
pub fn refresh_details_after_sync(cx: &mut App) {
    let details = &mut cx.global_mut::<Services>().details;
    if details.loading {
        details.stale = true;
        return;
    }
    refresh_details(cx);
}

/// Fetches the account's storage and devices for the Account page.
pub fn refresh_details(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    let (Some(session), Some(backend)) = (services.session(), services.backend.clone()) else {
        return;
    };
    if services.details.loading {
        return;
    }
    services.details.loading = true;
    let (tx, rx) = async_channel::bounded(1);
    let asked = session.clone();
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            let usage = kanso_sync::usage(backend.as_ref(), &asked);
            let devices = kanso_sync::devices(backend.as_ref(), &asked);
            tx.try_send((usage, devices)).ok();
        });
    if spawned.is_err() {
        cx.global_mut::<Services>().details.loading = false;
        return;
    }
    cx.spawn(async move |cx| {
        let answer = rx.recv().await.ok();
        cx.update(|cx| {
            let services = cx.global_mut::<Services>();
            let current = services
                .session()
                .is_some_and(|now| Arc::ptr_eq(&now, &session));
            if !current {
                return;
            }
            let stale = services.details.stale;
            let Some((usage, devices)) = answer else {
                services.details.loading = false;
                services.details.stale = false;
                services.details.failed = Some(SOMETHING_WRONG.to_owned());
                cx.refresh_windows();
                return;
            };
            let ended = [usage.as_ref().err(), devices.as_ref().err()]
                .into_iter()
                .flatten()
                .find_map(|err| match err {
                    AccountError::Unauthorized { reason } => Some(*reason),
                    _ => None,
                });
            if let Some(reason) = ended {
                services::end_session(reason, cx);
                return;
            }
            let failed = [usage.as_ref().err(), devices.as_ref().err()]
                .into_iter()
                .flatten()
                .next()
                .map(|err| words::error_words(err, ErrorContext::Account));
            let mut details = AccountDetails {
                usage: usage.ok(),
                devices: devices.ok(),
                loading: false,
                stale: false,
                failed,
            };
            if let Some(devices) = &mut details.devices {
                words::sort_devices(devices);
            }
            services.details = details;
            cx.refresh_windows();
            if stale {
                refresh_details(cx);
            }
        });
    })
    .detach();
}

// MARK: Launch

/// At launch: signed in only when the config's account, the library's
/// binding and the Keychain item agree (security re-check 3). An unsaved
/// recovery key shows whatever the account. Account tasks wait while it
/// reads (`restoring`), and its answer is dropped if a session appeared or
/// the config's account changed meanwhile (A4). Pending logouts are retried
/// after the answer is sent.
pub fn restore(cx: &mut App) {
    let Some(seams) = Seams::get(cx) else {
        return;
    };
    let account = cx.global::<AppState>().config.sync.account.clone();
    let asked = account.clone();
    let (tx, rx) = async_channel::bounded(1);
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            let device = match seams.device() {
                Ok(device) => device,
                Err(failure) => {
                    eprintln!("kanso: this device's identity: {}", failure.words);
                    tx.try_send(None).ok();
                    return;
                }
            };
            // An unsaved recovery key shows whatever this Mac is signed in
            // to: a cancelled recovery's, or one whose answer was lost.
            let kept = keychain::load_any_recovery_key(seams.secrets.as_ref(), &device.library_id)
                .unwrap_or_else(|err| {
                    eprintln!("kanso: reading the unsaved recovery key: {err}");
                    None
                });
            let loaded = match &account {
                Some(account) => {
                    let bound = |key| seams.store.meta(key).ok().flatten();
                    let agree = bound(meta_keys::ORIGIN).as_deref()
                        == Some(account.origin.as_str())
                        && bound(meta_keys::ACCOUNT_ID).as_deref() == Some(account.id.as_str());
                    if agree {
                        keychain::load_session(
                            seams.secrets.as_ref(),
                            &device.library_id,
                            &account.origin,
                            &account.id,
                        )
                    } else {
                        Ok(None)
                    }
                }
                None => Ok(None),
            };
            tx.try_send(Some((device.library_id.clone(), loaded, kept)))
                .ok();
            // The session is in; old tokens can take their time.
            kanso_sync::flush_pending_logouts(
                seams.backend.as_ref(),
                seams.secrets.as_ref(),
                &device.library_id,
            );
        });
    if spawned.is_err() {
        return;
    }
    cx.global_mut::<Services>().restoring = true;
    cx.spawn(async move |cx| {
        let answer = rx.recv().await.ok().flatten();
        cx.update(|cx| {
            cx.global_mut::<Services>().restoring = false;
            let Some((library_id, loaded, kept)) = answer else {
                cx.refresh_windows();
                return;
            };
            let services = cx.global_mut::<Services>();
            services.library_id = Some(library_id);
            if let Some(item) = kept
                && services.pending_key.is_none()
            {
                show_pending(PendingKey::kept(item, words::KEY_UNCONFIRMED), cx);
            }
            // Something signed in or out meanwhile: its state stands.
            let services = cx.global::<Services>();
            let moved = services.session().is_some()
                || services.task.is_some()
                || cx.global::<AppState>().config.sync.account != asked;
            if moved {
                cx.refresh_windows();
                return;
            }
            match loaded {
                Ok(Some(session)) => {
                    let services = cx.global_mut::<Services>();
                    services.set_signed_in(Arc::new(session));
                    services::apply_sync_settings(cx);
                    refresh_details(cx);
                    crate::device_approval::start_events(cx);
                    crate::device_approval::refresh(cx);
                }
                // The three don't agree: signed out, and the config says so.
                Ok(None) => {
                    if cx.global::<AppState>().config.sync.account.is_some() {
                        AppState::update_config(cx, |config| config.sync.account = None);
                    }
                }
                // The Keychain wouldn't say (locked): signed out for now,
                // with the account kept for the next launch.
                Err(err) => eprintln!("kanso: reading the account from the Keychain: {err}"),
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

// MARK: Device approval

/// What the waiting flow tells the page.
#[allow(dead_code)] // step 2's locked sign-in calls the flow below
enum WaitUpdate {
    OtherDevices(usize),
    Request(String),
    Code(String),
    Tick,
    Done(Result<(Session, String), WaitEnd>),
}

/// How the waiting flow ended without an unlocked session.
#[allow(dead_code)] // step 2's locked sign-in calls the flow below
enum WaitEnd {
    NoDevices,
    Cancelled,
    Expired,
    Failed(Failure),
}

impl WaitEnd {
    fn phase(&self) -> crate::device_approval::WaitPhase {
        match self {
            WaitEnd::NoDevices => crate::device_approval::WaitPhase::NoDevices,
            WaitEnd::Cancelled => crate::device_approval::WaitPhase::Cancelled,
            WaitEnd::Expired => crate::device_approval::WaitPhase::Expired,
            WaitEnd::Failed(failure) => {
                crate::device_approval::WaitPhase::Failed(failure.words.clone())
            }
        }
    }
}

/// Asks the account's other devices to approve `locked`, polls the request,
/// posts the sealed name and unlocks. Runs at most one wait at a time; the
/// Account page shows its code and offers the recovery key after two
/// minutes (or at once with no other device).
#[allow(dead_code)] // step 2's locked sign-in calls this
pub fn wait_for_device_approval(locked: LockedSession, device: DeviceIdentity, cx: &mut App) {
    let Some(seams) = Seams::get(cx) else {
        return;
    };
    if cx
        .try_global::<Services>()
        .is_some_and(|services| services.task.is_some() || services.approval.waiting.is_some())
    {
        return;
    }
    let started = Clock::now_ms(cx);
    let mut state = crate::device_approval::WaitingState::requesting(started);
    state.phase = crate::device_approval::WaitPhase::Requesting;
    let cancel = state.cancel.clone();
    cx.global_mut::<Services>().approval.waiting = Some(state);
    let (tx, rx) = async_channel::unbounded::<WaitUpdate>();
    let thread_backend = seams.backend.clone();
    let thread_secrets = seams.secrets.clone();
    let thread_store = seams.store.clone();
    let thread_backups = seams.backups.clone();
    let thread = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            let now = || {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
            };
            let result = (|| -> Result<(Session, String), WaitEnd> {
                let keys = ApprovalKeys::generate();
                let request = request_approval(thread_backend.as_ref(), &locked, &keys)
                    .map_err(|err| WaitEnd::Failed(Failure::account(err, ErrorContext::Account)))?;
                tx.try_send(WaitUpdate::OtherDevices(request.other_devices))
                    .ok();
                tx.try_send(WaitUpdate::Request(request.id.clone())).ok();
                if request.other_devices == 0 {
                    // Nobody could answer: don't leave a request behind.
                    cancel_request(thread_backend.as_ref(), &locked, &request.id).ok();
                    return Err(WaitEnd::NoDevices);
                }
                let mut approval = None;
                loop {
                    if cancel.load(std::sync::atomic::Ordering::SeqCst) {
                        cancel_request(thread_backend.as_ref(), &locked, &request.id).ok();
                        return Err(WaitEnd::Cancelled);
                    }
                    if now() >= request.expires_at {
                        return Err(WaitEnd::Expired);
                    }
                    match poll_approval(thread_backend.as_ref(), &locked, &request) {
                        Ok(RequestState::Waiting) => {}
                        Ok(RequestState::Challenged { approver_epk, .. }) => {
                            if approval.is_none() {
                                let code =
                                    complete_request(&locked, &request, &keys, &approver_epk)
                                        .map_err(|err| {
                                            WaitEnd::Failed(Failure::account(
                                                err,
                                                ErrorContext::Account,
                                            ))
                                        })?;
                                tx.try_send(WaitUpdate::Code(code.code())).ok();
                                name_device(
                                    thread_backend.as_ref(),
                                    &locked,
                                    &request,
                                    &code,
                                    &device,
                                )
                                .map_err(|err| {
                                    WaitEnd::Failed(Failure::account(err, ErrorContext::Account))
                                })?;
                                approval = Some(code);
                            }
                        }
                        Ok(RequestState::Approved(approved)) => {
                            let approval = approval.ok_or_else(|| {
                                WaitEnd::Failed(Failure::account(
                                    AccountError::Protocol("approved without a code".into()),
                                    ErrorContext::Account,
                                ))
                            })?;
                            let unlocked = finish_unlock(
                                thread_backend.as_ref(),
                                &locked,
                                &approval,
                                &approved,
                                &device,
                            )
                            .map_err(|err| {
                                WaitEnd::Failed(Failure::account(err, ErrorContext::Account))
                            })?;
                            let library = match kanso_sync::local_library(
                                &thread_store,
                                &unlocked.session.origin,
                                &unlocked.session.account_id,
                            ) {
                                Ok(library) => library,
                                Err(err) => return Err(WaitEnd::Failed(Failure::library(err))),
                            };
                            let choice = match kanso_sync::plan(Flow::SignIn, &library) {
                                Plan::Resume => None,
                                _ => Some(Choice::Add),
                            };
                            let seams = Seams {
                                backend: thread_backend.clone(),
                                secrets: thread_secrets.clone(),
                                store: thread_store.clone(),
                                machine: None,
                                backups: thread_backups.clone(),
                                now: now(),
                            };
                            save_and_bind(&seams, &device.library_id, &unlocked.session, choice)
                                .map_err(WaitEnd::Failed)?;
                            return Ok((unlocked.session, device.library_id));
                        }
                        Ok(RequestState::Cancelled) => return Err(WaitEnd::Cancelled),
                        Ok(RequestState::Expired) => return Err(WaitEnd::Expired),
                        Err(err) => {
                            // A transient failure leaves the request open; the
                            // next poll tries again. A final one ends the wait.
                            if matches!(err, AccountError::Unauthorized { .. }) {
                                return Err(WaitEnd::Failed(Failure::account(
                                    err,
                                    ErrorContext::Account,
                                )));
                            }
                        }
                    }
                    tx.try_send(WaitUpdate::Tick).ok();
                    std::thread::sleep(crate::device_approval::POLL_EVERY);
                }
            })();
            tx.try_send(WaitUpdate::Done(result)).ok();
        });
    if thread.is_err() {
        cx.global_mut::<Services>().approval.waiting = None;
        return;
    }
    cx.spawn(async move |cx| {
        while let Ok(update) = rx.recv().await {
            let mut finish = None;
            cx.update(|cx| {
                let now = Clock::now_ms(cx);
                let services = cx.global_mut::<Services>();
                let Some(waiting) = services.approval.waiting.as_mut() else {
                    return;
                };
                match update {
                    WaitUpdate::OtherDevices(count) => {
                        waiting.know_devices(count, now);
                        if count == 0 {
                            waiting.phase = crate::device_approval::WaitPhase::NoDevices;
                        }
                    }
                    WaitUpdate::Request(id) => waiting.request_id = Some(id),
                    WaitUpdate::Code(code) => {
                        waiting.code = Some(code.clone());
                        waiting.phase = crate::device_approval::WaitPhase::Compare { code };
                    }
                    WaitUpdate::Tick => waiting.offer_recovery(now),
                    WaitUpdate::Done(result) => {
                        finish = Some(result);
                    }
                }
                cx.refresh_windows();
            });
            if let Some(result) = finish {
                match result {
                    Ok((session, library_id)) => {
                        cx.update(|cx| {
                            cx.global_mut::<Services>().approval.waiting = None;
                            crate::account_flow::signed_in(session, library_id, true, cx);
                            crate::device_approval::start_events(cx);
                        });
                    }
                    Err(end) => {
                        cx.update(|cx| {
                            let services = cx.global_mut::<Services>();
                            if let Some(waiting) = services.approval.waiting.as_mut() {
                                waiting.phase = end.phase();
                                waiting.recovery_offered = true;
                                if let WaitEnd::Failed(failure) = &end {
                                    eprintln!("kanso: device approval: {}", failure.words);
                                }
                            }
                            cx.refresh_windows();
                        });
                    }
                }
                break;
            }
        }
    })
    .detach();
    cx.refresh_windows();
}

/// Closes the waiting page after a cancellation or an expiry.
pub fn dismiss_device_approval(cx: &mut App) {
    cx.global_mut::<Services>().approval.waiting = None;
    cx.refresh_windows();
}

/// The waiting page's Cancel: stops the poll and asks the server to cancel
/// the request. A request that is gone counts as cancelled.
pub fn cancel_device_approval(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    let Some(waiting) = services.approval.waiting.as_mut() else {
        return;
    };
    waiting
        .cancel
        .store(true, std::sync::atomic::Ordering::SeqCst);
    waiting.phase = crate::device_approval::WaitPhase::Cancelled;
    cx.refresh_windows();
}
