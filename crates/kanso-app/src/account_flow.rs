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
use std::sync::atomic::{AtomicBool, Ordering};

use async_channel::{Receiver, Sender};
use gpui_kit::component::button::Button;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_engine::account::provider::{
    AppleWait, ProviderDone, ProviderRoute, ProviderTaskResult, StartedProviderFlow,
    provider_begin, provider_finish,
};
use kanso_engine::account::{
    Authenticate, Authenticated, Bound, Next, NextUnlock, Restored, WaitEnd, WaitUpdate,
    after_auth, after_unlock, details_from, restore::candidate_from_local,
};
use kanso_sync::api::LoginDevice;
use kanso_sync::keychain;
use kanso_sync::provider::{Provider, ProviderStart};
use kanso_sync::{
    AccountError, Backend, Choice, DeviceIdentity, DeviceInfo, Flow, LockedSession, Password, Plan,
    RecoveryKey, Session, SignedUp, Step,
};

use crate::account_words::{self as words, Answer, ErrorContext, Question};
use crate::clock::Clock;
use crate::config::{AccountRef, is_default_server};
use crate::dialogs::{self, Described, Progress};
use crate::passkey_accounts::{
    self, PasskeyContext, PasskeyFlowError, PasskeySignedUp, RpRule, SyncPasskeyAccounts,
};
use crate::passkey_ceremony::{PlatformCeremony, platform_ceremony};
use crate::provider_browser::Desktop;
use crate::security_key_flow::{OwnedSecurityKey, PinProblem};
use crate::services::{
    self, AccountState, PendingKey, RunningTask, Services, TaskKind, notify_windows,
};
use crate::settings::{OpenAccountSettings, SettingsPage};
use crate::workspace::AppState;

// The account tasks' workers, their failures and decisions are the engine's,
// so the mobile app runs the same ones; this file is the desktop's windows,
// notifications and threads around them.
use kanso_engine::account::Flow as Seams;
pub(crate) use kanso_engine::account::UnlockedCandidate;
pub use kanso_engine::account::auth_answer_is_current;
#[cfg(test)]
pub use kanso_engine::account::provider::locked_provider_session;
pub use kanso_engine::account::provider::save_provider_pending;
pub(crate) use kanso_engine::account::{BUSY, Failure, LIBRARY_TROUBLE, SOMETHING_WRONG};

mod methods;
pub use methods::{ChangeRequest, MethodChange, ProofWith, change_method};

// MARK: Running a task

/// What a task's thread sends back.
enum Message<T> {
    Step(Step),
    Done(T),
}

/// What the tasks reach the world through, taken from [`Services`] and
/// [`AppState`] on the main thread.
fn get_seams(cx: &App) -> Option<Seams> {
    let services = cx.try_global::<Services>()?;
    let state = cx.try_global::<AppState>()?;
    Some(Seams {
        backend: services.backend.clone()?,
        secrets: services.secrets.clone()?,
        store: state.store.clone(),
        machine: services.machine.clone(),
        backups: crate::paths::backups_dir(&state.data_dir),
        now_ms: Clock::now_ms(cx),
    })
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
        services.stop_sync()
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
    services.task_changed();
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
                sync.join();
            }
            let result = work(&step);
            tx.try_send(Message::Done(result)).ok();
        });
    if let Err(err) = spawned {
        // The stopped sync went with the closure, cancelled; nothing
        // waits for it. Nothing was changed.
        eprintln!("kanso: couldn't start an account task: {err}");
        let services = cx.global_mut::<Services>();
        services.task = None;
        services.task_changed();
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
            let services = cx.global_mut::<Services>();
            services.task = None;
            services.task_changed();
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
    if failure.locked_cleanup {
        cx.global_mut::<Services>()
            .clear_session(AccountState::SignedOut);
    }
    if failure.keychain_warning {
        notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
    }
    if let Some(key) = failure.recovery {
        show_pending(*key, cx);
    }
    if let Some(path) = failure.backup {
        notify_backup(path, cx);
    }
    done(tx, Err(failure.words));
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
    let seams = get_seams(cx);
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
    let Some(seams) = get_seams(cx) else {
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
        move |_| seams.check_kept_key(&shown, library_id),
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

/// Leaves a cancelled locked sign-in without changing the prior account's
/// config, recovery-key screen, Keychain item, or library binding.
fn clear_locked_account(cx: &mut App) {
    crate::device_approval::stop_events(cx);
    cx.global_mut::<Services>()
        .clear_session(AccountState::SignedOut);
    crate::collab::signed_out(cx);
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
        move |_| thread.library(&origin, ""),
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
        move |step| thread.create(&server, &username, &password, step),
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
            s.sign_in_password(&server, &username, &password, device, step)
        }),
    );
    rx
}

/// Starts a provider flow on the account worker. Browser callbacks and
/// polling run off the UI thread; only the final, typed result returns to GPUI.
pub fn start_provider(
    provider: Provider,
    creating: bool,
    window: AnyWindowHandle,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let login_device = LoginDevice::ephemeral();
    let server = form_server(cx);
    let server_generation = cx.global::<Services>().auth_generation;
    let task_kind = if creating {
        TaskKind::SignUp
    } else {
        TaskKind::SignIn
    };
    let thread = seams.clone();
    let start_provider = provider.clone();
    let start_device = login_device.clone();
    let start_server = server.clone();
    run(
        task_kind,
        Step::Contacting,
        true,
        Some(tx.clone()),
        cx,
        move |step| {
            thread.start_provider(
                &Desktop,
                &start_server,
                &start_provider,
                &start_device,
                step,
            )
        },
        move |result, cx| match result {
            Some(Ok(started)) => {
                if !provider_request_is_current(cx, &server, server_generation) {
                    // The server changed while start began. Never present an
                    // Apple sheet or begin polling a flow for the old server.
                    done(&tx, Ok(()));
                    return;
                }
                let apple_callback = match (&started.start, &provider) {
                    (ProviderStart::Apple(apple), Provider::Apple) => {
                        match crate::provider_browser::apple_sign_in(apple) {
                            Ok(callback) => {
                                Some(Box::new(crate::provider_browser::AppleReceiver(callback))
                                    as Box<dyn AppleWait>)
                            }
                            Err(_) => {
                                failed(
                                    &tx,
                                    Some(Failure::account(
                                        AccountError::ProviderUnavailable,
                                        ErrorContext::SignIn,
                                    )),
                                    cx,
                                );
                                return;
                            }
                        }
                    }
                    _ => None,
                };
                let cancel = Arc::new(AtomicBool::new(false));
                let (provider_label, user_code, verification_uri) = match &started.start {
                    ProviderStart::Apple(_) => ("Apple", None, None),
                    ProviderStart::Google(_) => ("Google", None, None),
                    ProviderStart::Github(start) => (
                        "GitHub",
                        Some(start.user_code.clone()),
                        Some(start.verification_uri.clone()),
                    ),
                };
                copy_device_code(user_code.as_deref(), cx);
                cx.global_mut::<Services>().provider_wait = Some(crate::services::ProviderWait {
                    provider: provider_label,
                    user_code,
                    verification_uri,
                    cancel: cancel.clone(),
                });
                continue_provider(
                    seams,
                    server,
                    provider,
                    login_device,
                    started,
                    apple_callback,
                    cancel,
                    server_generation,
                    task_kind,
                    window,
                    tx,
                    cx,
                );
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

/// GitHub's page asks for the code: it goes on the clipboard before the
/// page opens, ready to paste.
pub(crate) fn copy_device_code(user_code: Option<&str>, cx: &mut App) {
    if let Some(code) = user_code {
        cx.write_to_clipboard(ClipboardItem::new_string(code.to_owned()));
    }
}

#[allow(clippy::too_many_arguments)] // Keeps the whole durable continuation in one place.
fn continue_provider(
    seams: Seams,
    server: String,
    provider: Provider,
    login_device: LoginDevice,
    started: StartedProviderFlow,
    apple_callback: Option<Box<dyn AppleWait>>,
    cancel: Arc<AtomicBool>,
    server_generation: u64,
    task_kind: TaskKind,
    window: AnyWindowHandle,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let thread = seams.clone();
    let wait_cancel = cancel.clone();
    let work_server = server.clone();
    let work_provider = provider.clone();
    let work_login_device = login_device.clone();
    let result_server = server.clone();
    run(
        task_kind,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |step| {
            thread.finish_provider(
                &Desktop,
                &work_server,
                &work_provider,
                &work_login_device,
                started,
                apple_callback,
                cancel,
                step,
            )
        },
        move |result, cx| {
            // A later flow may have replaced this one while it was finishing.
            // Clear only the wait this task owns.
            clear_provider_wait_if_owned(
                &mut cx.global_mut::<Services>().provider_wait,
                &wait_cancel,
            );
            match result {
                Some(Ok(ProviderTaskResult::Cancelled)) => {
                    done(&tx, Ok(()));
                }
                Some(Ok(ProviderTaskResult::NewAccount(account))) => {
                    if wait_cancel.load(Ordering::Acquire)
                        || !provider_request_is_current(cx, &result_server, server_generation)
                    {
                        done(&tx, Ok(()));
                        return;
                    }
                    cx.global_mut::<Services>().provider_signup =
                        Some(crate::services::ProviderSignup {
                            server: result_server,
                            provider,
                            ticket: account.ticket,
                            suggested_username: account.suggested_username,
                            login_device,
                        });
                    done(&tx, Ok(()));
                }
                Some(Ok(ProviderTaskResult::Locked(locked, device))) => {
                    if wait_cancel.load(Ordering::Acquire)
                        || !provider_request_is_current(cx, &result_server, server_generation)
                    {
                        // The pending item was already made durable on the worker
                        // so a crash could not lose the token. Remove it and sign
                        // the token out now that this result belongs to a server
                        // the UI no longer shows.
                        discard_stale_locked_provider_session(seams, locked, tx, cx);
                        return;
                    }
                    enter_locked(*locked, device, Some(window), cx);
                    done(&tx, Ok(()));
                }
                Some(Err(failure)) => failed(&tx, Some(failure), cx),
                None => failed(&tx, None, cx),
            }
        },
    );
}

/// A provider answer belongs to the exact server form that started it, and
/// to the same server generation. Generation makes A -> B -> A safe: the
/// server string can return to A while the old flow's answer is still stale.
fn provider_request_matches(
    current_server: &str,
    current_generation: u64,
    asked_server: &str,
    asked_generation: u64,
) -> bool {
    current_server == asked_server && current_generation == asked_generation
}

fn provider_request_is_current(cx: &App, server: &str, generation: u64) -> bool {
    provider_request_matches(
        &form_server(cx),
        cx.global::<Services>().auth_generation,
        server,
        generation,
    )
}

/// Provider callbacks finish after the visible wait is gone. A delayed old
/// callback must not clear a newer flow's wait, so ownership is the same
/// cancellation token the wait was created with.
fn clear_provider_wait_if_owned(
    wait: &mut Option<crate::services::ProviderWait>,
    cancel: &Arc<AtomicBool>,
) {
    if wait
        .as_ref()
        .is_some_and(|wait| Arc::ptr_eq(&wait.cancel, cancel))
    {
        *wait = None;
    }
}

/// Cleans up a locked result that became stale during its final callback.
/// The pending Keychain item and server token are removed off the UI thread;
/// the current server's form and any newer provider flow are left alone.
fn discard_stale_locked_provider_session(
    seams: Seams,
    locked: Box<LockedSession>,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    run(
        TaskKind::SignOut,
        Step::Contacting,
        false,
        None,
        cx,
        move |_| seams.discard_locked(&locked),
        move |_, _cx| done(&tx, Ok(())),
    );
}

/// Creates a provider-backed passwordless account after the user confirms an
/// editable username, then follows the same Keychain, binding and recovery
/// key path as username/password signup.
pub fn provider_sign_up(
    pending: crate::services::ProviderSignup,
    username: String,
    window: AnyWindowHandle,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let thread = seams.clone();
    let server = pending.server.clone();
    let asked_server = server.clone();
    run(
        TaskKind::SignUp,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| thread.provider_sign_up_library(&asked_server),
        move |result, cx| match result {
            Some(Ok(library)) => match kanso_sync::plan(Flow::SignUp, &library) {
                Plan::AskBeforeSignUp { from, .. } => {
                    let (title, body, upload) = words::upload_question(&from);
                    ask(
                        window,
                        Question {
                            title,
                            body,
                            answers: vec![Answer {
                                choice: Choice::Add,
                                label: upload.to_owned(),
                                description: "They're encrypted and uploaded to the new account. Nothing changes in the other one.".to_owned(),
                            }],
                        },
                        cx,
                        move |choice, cx| match choice {
                            Choice::Cancel => {
                                cx.global_mut::<Services>().provider_signup = None;
                                done(&tx, Ok(()));
                            }
                            Choice::Add | Choice::Replace => create_provider_signup(
                                seams, server, pending, username, library, tx, cx,
                            ),
                        },
                    );
                }
                _ => create_provider_signup(seams, server, pending, username, library, tx, cx),
            },
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

fn create_provider_signup(
    seams: Seams,
    server: String,
    pending: crate::services::ProviderSignup,
    username: String,
    library: kanso_sync::LocalLibrary,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let thread = seams.clone();
    run(
        TaskKind::SignUp,
        Step::DerivingKeys,
        false,
        Some(tx.clone()),
        cx,
        move |step| thread.create_provider_signup(&server, &pending, &username, library, step),
        move |result, cx| match result {
            Some(Ok(auth)) => {
                cx.global_mut::<Services>().provider_signup = None;
                if let Some(key) = &auth.recovery_key {
                    show_key(&auth.session, key.clone(), cx);
                }
                bind_and_save(
                    TaskKind::SignUp,
                    seams,
                    auth,
                    Some(Choice::Add),
                    false,
                    tx,
                    cx,
                );
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
}

// MARK: Passkeys and security keys

/// Which authenticator a passkey task uses.
pub enum PasskeyMethod {
    /// The platform's sheet: AuthenticationServices on macOS, Windows Hello
    /// and security keys through webauthn.h on Windows, owned by `window`
    /// (`passkey_ceremony::native_window`; 0 when there is none).
    Platform { window: isize },
    /// A Linux security key, with its PIN when it needs one.
    SecurityKey {
        device: kanso_fido::SecurityKeyDevice,
        pin: Option<zeroize::Zeroizing<String>>,
    },
}

impl PasskeyMethod {
    fn is_security_key(&self) -> bool {
        matches!(self, PasskeyMethod::SecurityKey { .. })
    }
}

/// The ceremony a passkey task's thread runs, made on the main thread.
enum Runner {
    /// The platform's sheet (`passkey_ceremony::platform_ceremony`).
    Platform(PlatformCeremony),
    SecurityKey(OwnedSecurityKey),
    #[cfg(test)]
    Test(Arc<dyn kanso_sync::PasskeyCeremony + Send + Sync>),
}

impl Runner {
    /// The runner, and the main-thread task serving a platform ceremony
    /// (kept until the task finishes).
    fn new(method: PasskeyMethod, cx: &mut App) -> (Runner, Option<Task<()>>) {
        #[cfg(test)]
        if let Some(ceremony) = cx.global::<Services>().test_ceremony.clone() {
            return (Runner::Test(ceremony), None);
        }
        match method {
            PasskeyMethod::Platform { window } => {
                let (ceremony, serving) = platform_ceremony(window, cx);
                (Runner::Platform(ceremony), serving)
            }
            PasskeyMethod::SecurityKey { device, pin } => {
                (Runner::SecurityKey(OwnedSecurityKey { device, pin }), None)
            }
        }
    }

    fn rule(&self) -> RpRule {
        match self {
            Runner::Platform(_) => RpRule::platform(),
            Runner::SecurityKey(_) => RpRule::ServerHost,
            #[cfg(test)]
            Runner::Test(_) => RpRule::ServerHost,
        }
    }

    /// Runs `work` with the ceremony; afterwards, what a security key said
    /// about its PIN, if anything.
    fn run<T>(
        self,
        work: impl FnOnce(&dyn kanso_sync::PasskeyCeremony) -> T,
    ) -> (T, Option<PinProblem>) {
        match self {
            Runner::Platform(ceremony) => (work(ceremony.as_ref()), None),
            Runner::SecurityKey(key) => key.with(work),
            #[cfg(test)]
            Runner::Test(ceremony) => (work(ceremony.as_ref()), None),
        }
    }
}

/// How a passkey task ended without an answer.
enum PasskeyEnd {
    /// The sheet or the key was dismissed: the page stays as it was.
    Cancelled,
    /// The platform's ceremony can't run here: the buttons go.
    Unavailable,
    Failed(Failure),
}

impl From<Failure> for PasskeyEnd {
    fn from(failure: Failure) -> Self {
        PasskeyEnd::Failed(failure)
    }
}

fn passkey_end(
    error: PasskeyFlowError,
    pin: Option<PinProblem>,
    security_key: bool,
    context: ErrorContext,
) -> PasskeyEnd {
    use kanso_sync::PasskeyCeremonyError as Ceremony;
    let failed = if security_key {
        words::SECURITY_KEY_FAILED
    } else {
        words::PASSKEY_FAILED
    };
    match pin {
        Some(PinProblem::Required) => return Failure::words(words::SECURITY_KEY_PIN).into(),
        Some(PinProblem::Invalid { retries }) => {
            return Failure::words(&words::security_key_pin_wrong(retries)).into();
        }
        Some(PinProblem::AuthBlocked) => {
            return Failure::words(words::SECURITY_KEY_PIN_AUTH_BLOCKED).into();
        }
        Some(PinProblem::Blocked) => return Failure::words(words::SECURITY_KEY_PIN_BLOCKED).into(),
        None => {}
    }
    match error {
        PasskeyFlowError::Account(err) => Failure::account(err, context).into(),
        PasskeyFlowError::Rotation(err) => Failure::rotation(
            err,
            ErrorContext::SignUp,
            words::KEY_UNCONFIRMED_PASSKEY_SIGN_UP,
        )
        .into(),
        PasskeyFlowError::Ceremony(Ceremony::Cancelled) => PasskeyEnd::Cancelled,
        // A security key that went away is the page's to show again.
        PasskeyFlowError::Ceremony(Ceremony::Unavailable) if security_key => {
            Failure::words(failed).into()
        }
        PasskeyFlowError::Ceremony(Ceremony::Unavailable) => PasskeyEnd::Unavailable,
        PasskeyFlowError::Ceremony(Ceremony::Failed) => Failure::words(failed).into(),
        PasskeyFlowError::WrongRelyingParty => Failure::words(words::PASSKEY_WRONG_SERVER).into(),
    }
}

fn passkey_ended(end: PasskeyEnd, tx: &Sender<Progress>, cx: &mut App) {
    match end {
        PasskeyEnd::Cancelled => done(tx, Ok(())),
        PasskeyEnd::Unavailable => {
            cx.global_mut::<Services>().platform_passkeys_failed = true;
            done(tx, Err(words::PASSKEY_UNAVAILABLE.to_owned()));
        }
        PasskeyEnd::Failed(failure) => failed(tx, Some(failure), cx),
    }
}

/// What a passkey sign-in's thread found.
enum PasskeyIn {
    /// The key came with it: binding as after a password sign-in.
    Unlocked(Box<Authenticated>),
    /// Signed in without the key, its pending item saved: approval, the
    /// password or the recovery key unlocks it. The flag says the
    /// credential answered PRF.
    Locked(Box<LockedSession>, DeviceIdentity, bool),
}

/// Signs in with a passkey or a security key (Settings › Account › Sign in
/// › Continue with a passkey / a security key). `username` is the security
/// key's username-first name; the platform picker takes none.
pub fn passkey_sign_in(
    server: String,
    method: PasskeyMethod,
    username: Option<String>,
    window: AnyWindowHandle,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let security_key = method.is_security_key();
    let (runner, serving) = Runner::new(method, cx);
    let rule = runner.rule();
    let was_signed_in = cx.global::<Services>().signed_in();
    let thread = seams.clone();
    run(
        TaskKind::SignIn,
        Step::Contacting,
        was_signed_in,
        Some(tx.clone()),
        cx,
        move |step| -> Result<PasskeyIn, PasskeyEnd> {
            let s = thread;
            let device = s.device()?;
            let (signed, pin) = runner.run(|ceremony| {
                let context = PasskeyContext {
                    accounts: &SyncPasskeyAccounts,
                    backend: s.backend.as_ref(),
                    secrets: s.secrets.as_ref(),
                    server: &server,
                    device: &device,
                    rule,
                    progress: step,
                };
                passkey_accounts::sign_in(&context, username.as_deref(), ceremony)
            });
            let signed =
                signed.map_err(|err| passkey_end(err, pin, security_key, ErrorContext::Passkey))?;
            match signed.signed {
                kanso_sync::PasskeySignIn::Unlocked(kanso_sync::SignedIn { session, evicted }) => {
                    let auth = s.authenticated(device.library_id, session, evicted, None)?;
                    Ok(PasskeyIn::Unlocked(Box::new(auth)))
                }
                kanso_sync::PasskeySignIn::Locked(locked) => {
                    save_provider_pending(s.backend.as_ref(), s.secrets.as_ref(), &locked)
                        .map_err(Failure::keychain)?;
                    Ok(PasskeyIn::Locked(Box::new(locked), device, signed.prf))
                }
            }
        },
        move |result, cx| {
            drop(serving);
            match result {
                Some(Ok(PasskeyIn::Unlocked(auth))) => settle_authenticated(
                    Flow::SignIn,
                    Some(window),
                    seams,
                    *auth,
                    was_signed_in,
                    tx,
                    cx,
                ),
                Some(Ok(PasskeyIn::Locked(locked, device, prf))) => {
                    enter_locked(*locked, device, Some(window), cx);
                    if !prf {
                        let notice = words::passkey_signs_in_only(security_key);
                        notify_windows(move || Notification::info(notice), cx);
                    }
                    done(&tx, Ok(()));
                }
                Some(Err(end)) => passkey_ended(end, &tx, cx),
                None => failed(&tx, None, cx),
            }
        },
    );
    rx
}

/// Creates an account whose first way in is a new passkey or security key
/// (Settings › Account › Create account › Create with a passkey / a
/// security key), and signs this device in with it. Notes that came from
/// another account are asked about in `window` first, as for a password
/// sign-up.
pub fn passkey_sign_up(
    server: String,
    method: PasskeyMethod,
    username: String,
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
        move |_| thread.library(&origin, ""),
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
                        _ => create_with_passkey(seams, server, username, method, tx, cx),
                    });
                }
                _ => create_with_passkey(seams, server, username, method, tx, cx),
            },
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

fn create_with_passkey(
    seams: Seams,
    server: String,
    username: String,
    method: PasskeyMethod,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let security_key = method.is_security_key();
    let (runner, serving) = Runner::new(method, cx);
    let rule = runner.rule();
    let thread = seams.clone();
    run(
        TaskKind::SignUp,
        Step::Contacting,
        true,
        Some(tx.clone()),
        cx,
        move |step| -> Result<(PasskeySignedUp, String), PasskeyEnd> {
            let s = thread;
            let device = s.device()?;
            let (made, pin) = runner.run(|ceremony| {
                let context = PasskeyContext {
                    accounts: &SyncPasskeyAccounts,
                    backend: s.backend.as_ref(),
                    secrets: s.secrets.as_ref(),
                    server: &server,
                    device: &device,
                    rule,
                    progress: step,
                };
                passkey_accounts::sign_up(&context, &username, ceremony)
            });
            let made =
                made.map_err(|err| passkey_end(err, pin, security_key, ErrorContext::SignUp))?;
            step(Step::Saving);
            let bound =
                s.save_and_bind(&device.library_id, &made.signed.session, Some(Choice::Add));
            if let Err(failure) = bound {
                // As after a password sign-up: the account and its key stay.
                kanso_sync::discard_session(
                    s.backend.as_ref(),
                    s.secrets.as_ref(),
                    &made.signed.session,
                )
                .ok();
                let SignedUp {
                    session,
                    recovery_key,
                } = made.signed;
                return Err(failure
                    .after_key(words::CREATED_NOT_SIGNED_IN_PASSKEY, &session, recovery_key)
                    .into());
            }
            Ok((made, device.library_id))
        },
        move |result, cx| {
            drop(serving);
            match result {
                Some(Ok((made, library_id))) => {
                    let PasskeySignedUp { signed, can_unlock } = made;
                    let SignedUp {
                        session,
                        recovery_key,
                    } = signed;
                    show_key(&session, recovery_key, cx);
                    signed_in(session, library_id, true, cx);
                    let notice = words::passkey_created(security_key, can_unlock);
                    if can_unlock {
                        notify_windows(move || Notification::success(notice), cx);
                    } else {
                        notify_windows(move || Notification::warning(notice), cx);
                    }
                    done(&tx, Ok(()));
                }
                Some(Err(end)) => passkey_ended(end, &tx, cx),
                None => failed(&tx, None, cx),
            }
        },
    );
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
            s.recover_with_key(&server, &username, &key, &password, device, step)
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
        move |step| thread.authenticate(auth, step),
        move |result, cx| match result {
            Some(Ok(auth)) => {
                settle_authenticated(flow, window, seams, auth, was_signed_in, tx, cx)
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
}

/// On the main thread, after `Flow::authenticated`: the binding question when
/// there is one, then the binding and the Keychain item.
fn settle_authenticated(
    flow: Flow,
    window: Option<AnyWindowHandle>,
    seams: Seams,
    auth: Authenticated,
    was_signed_in: bool,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let kind = match flow {
        Flow::SignUp => TaskKind::SignUp,
        Flow::SignIn => TaskKind::SignIn,
        Flow::Recover => TaskKind::Recover,
    };
    // A used recovery key is gone: the new one shows at once, even if
    // the binding question is cancelled.
    if let Some(key) = &auth.recovery_key {
        show_key(&auth.session, key.clone(), cx);
    }
    let next = after_auth(
        flow,
        &auth.library,
        &auth.session.username,
        window.is_some(),
    );
    match (next, window) {
        (Next::Ask(question), Some(window)) => {
            ask(window, question, cx, move |choice, cx| match choice {
                Choice::Cancel => abandon(seams, auth.session, tx, cx),
                choice => bind_and_save(kind, seams, auth, Some(choice), was_signed_in, tx, cx),
            });
        }
        (Next::Ask(_), None) | (Next::Abandon, _) => abandon(seams, auth.session, tx, cx),
        (Next::Bind(choice), _) => bind_and_save(kind, seams, auth, choice, was_signed_in, tx, cx),
    }
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
                step(Step::Saving);
                if let Some(api) = api
                    && let Err(err) = api.stop()
                {
                    eprintln!("kanso: the API server stopped with an error: {err:#}");
                }
                thread.bind_and_save(session, library_id, choice, recovery_key)
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
        move |_| seams.abandon(&session),
        move |_, _| done(&tx, Ok(())),
    );
}

/// Leaves the provider-authenticated UI locked and tries only a saved local
/// full session for an offline key reuse. Approval never starts until the
/// locked item has been written.
pub(crate) fn enter_locked(
    mut locked: LockedSession,
    device: DeviceIdentity,
    window: Option<AnyWindowHandle>,
    cx: &mut App,
) {
    locked.set_library_id(&device.library_id);
    let Some(seams) = get_seams(cx) else {
        return;
    };
    crate::device_approval::stop_events(cx);
    cx.global_mut::<Services>()
        .clear_session(AccountState::SignedOut);
    {
        let services = cx.global_mut::<Services>();
        services.library_id = Some(device.library_id.clone());
        services.locked_has_password = Some(locked.has_password);
        services.locked_session = Some(Arc::new(locked.clone()));
        services.account = AccountState::SignedInLocked {
            username: locked.username.clone(),
        };
    }

    // A matching full item is an offline key source. Its origin, account and
    // key ID are checked by the loader and unlock function before consent.
    if let Some(candidate) = seams.local_unlock(&locked, &device.library_id) {
        finish_unlocked_candidate(candidate, window, None, cx);
        return;
    }
    if locked.approval_resume().is_some() {
        wait_for_device_approval(locked, device, window, cx);
    }
    cx.refresh_windows();
}

/// Runs on the UI thread after every unlock path. The local-library plan is
/// computed here so no worker can bind, choose a disposition, or start sync.
pub(crate) fn finish_unlocked_candidate(
    candidate: UnlockedCandidate,
    window: Option<AnyWindowHandle>,
    completion: Option<Sender<Progress>>,
    cx: &mut App,
) {
    {
        let locked = candidate.locked_session.clone();
        let services = cx.global_mut::<Services>();
        services.library_id = locked.library_id().map(str::to_owned);
        services.locked_has_password = Some(locked.has_password);
        services.locked_session = Some(Arc::new(locked.clone()));
        services.account = AccountState::SignedInLocked {
            username: locked.username.clone(),
        };
    }
    let window = window.or_else(|| cx.active_window());
    let Some(seams) = get_seams(cx) else {
        return;
    };
    let library = match kanso_sync::local_library(
        &seams.store,
        &candidate.session.origin,
        &candidate.session.account_id,
    ) {
        Ok(library) => library,
        Err(error) => {
            eprintln!("kanso: reading the library before unlock promotion: {error}");
            cleanup_candidate(
                seams,
                candidate,
                completion,
                Err(LIBRARY_TROUBLE.to_owned()),
                cx,
            );
            cx.refresh_windows();
            return;
        }
    };
    let next = after_unlock(&library, &candidate.session.username, window.is_some());
    match (next, window) {
        (NextUnlock::Ask(question), Some(window)) => {
            ask(window, question, cx, move |choice, cx| {
                if choice == Choice::Cancel {
                    let Some(seams) = get_seams(cx) else {
                        clear_locked_account(cx);
                        if let Some(tx) = completion {
                            done(&tx, Err("Sign-in was cancelled.".to_owned()));
                        }
                        return;
                    };
                    cleanup_candidate(
                        seams,
                        candidate,
                        completion,
                        Err("Sign-in was cancelled.".to_owned()),
                        cx,
                    );
                    return;
                }
                promote_unlocked_candidate(candidate, Some(choice), seams, completion, cx);
            });
        }
        (NextUnlock::Ask(_), None) | (NextUnlock::Park, _) => {
            // The launch restore has no window handle. Keep the durable
            // pending session and its candidate available for the Account
            // page rather than silently choosing or cancelling.
            cx.global_mut::<Services>().pending_unlocked_candidate = Some(candidate);
            if let Some(tx) = completion {
                done(&tx, Err(SOMETHING_WRONG.to_owned()));
            }
            cx.refresh_windows();
        }
        (NextUnlock::Promote(choice), _) => {
            promote_unlocked_candidate(candidate, choice, seams, completion, cx)
        }
        (NextUnlock::Reject, _) => {
            cleanup_candidate(
                seams,
                candidate,
                completion,
                Err(SOMETHING_WRONG.to_owned()),
                cx,
            );
        }
    }
}

/// Cancels or rejects a candidate without touching the previous full item,
/// binding, recovery item, or library contents. Logout and Keychain cleanup
/// run on the account worker, never on the UI thread.
fn cleanup_candidate(
    seams: Seams,
    candidate: UnlockedCandidate,
    completion: Option<Sender<Progress>>,
    outcome: Result<(), String>,
    cx: &mut App,
) {
    let locked = cx.global::<Services>().locked_session.clone();
    let thread = seams.clone();
    let (internal_tx, _) = async_channel::unbounded();
    let tx = completion.unwrap_or(internal_tx);
    run(
        TaskKind::SignOut,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| thread.cleanup_candidate(locked.as_deref(), &candidate),
        move |keychain_failed, cx| {
            clear_locked_account(cx);
            if keychain_failed == Some(true) {
                notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
            }
            done(&tx, outcome);
        },
    );
}

fn promote_unlocked_candidate(
    candidate: UnlockedCandidate,
    choice: Option<Choice>,
    seams: Seams,
    completion: Option<Sender<Progress>>,
    cx: &mut App,
) {
    let replacing = choice == Some(Choice::Replace);
    let (api, restart_api) = if replacing {
        cx.global_mut::<Services>()
            .begin_locked_replace_write_consent();
        let held = services::hold_writes(cx);
        cx.global_mut::<Services>()
            .end_locked_replace_write_consent();
        held
    } else {
        (None, false)
    };
    let session = candidate.session;
    let library_id = candidate.library_id;
    let evicted = candidate.evicted;
    let same_token_as_locked = candidate.same_token_as_locked;
    let locked = cx.global::<Services>().locked_session.clone();
    let thread = seams.clone();
    let (internal_tx, _) = async_channel::unbounded();
    let tx = completion.unwrap_or(internal_tx);
    run(
        TaskKind::SignIn,
        Step::Saving,
        false,
        Some(tx.clone()),
        cx,
        move |_| {
            if let Some(api) = api
                && let Err(error) = api.stop()
            {
                eprintln!("kanso: the API server stopped with an error: {error:#}");
            }
            thread.promote(
                session,
                library_id,
                same_token_as_locked,
                choice,
                locked.as_deref(),
            )
        },
        move |result, cx| {
            if replacing {
                services::release_writes(restart_api, cx);
            }
            match result {
                Some(Ok((session, library_id))) => {
                    signed_in(session, library_id, true, cx);
                    if let Some(device) = evicted {
                        let notice = words::evicted_notice(&device);
                        notify_windows(move || Notification::info(notice.clone()), cx);
                    }
                    done(&tx, Ok(()));
                }
                Some(Err(failure)) => {
                    failed(&tx, Some(failure), cx);
                }
                None => {
                    failed(&tx, None, cx);
                }
            }
        },
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
        move |_| seams.sign_out(&session),
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
        move |step| seams.change_password(&session, &current, &new, step),
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
        move |step| seams.new_recovery_key(&session, &current, step),
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
        move |step| seams.delete_account(&session, &password, &typed, &library_id, step),
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
        move |_| seams.revoke_device(&session, &device_id),
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
    if let (Some(saved), Some(seams)) = (saved, get_seams(cx)) {
        cx.background_spawn(async move { seams.forget_saved_key(library_id, &saved.key) })
            .detach();
    }
    cx.refresh_windows();
}

// MARK: Servers

/// The server the forms use: `None` for the default.
pub fn set_server(server: Option<String>, cx: &mut App) {
    let server = server.filter(|server| !is_default_server(server));
    AppState::update_config(cx, |config| config.sync.server = server);
    {
        let services = cx.global_mut::<Services>();
        if let Some(wait) = services.provider_wait.take() {
            wait.cancel.store(true, Ordering::Release);
        }
        services.auth_generation = services.auth_generation.wrapping_add(1);
        services.auth = None;
        services.auth_asking = None;
        services.auth_attempted = None;
        services.signup = None;
        services.provider_signup = None;
    }
    check_signup(cx);
    check_auth(cx);
    cx.refresh_windows();
}

/// Requests cancellation of the active provider flow. Its completion callback
/// owns clearing the visible wait state.
pub fn cancel_provider_wait(cx: &mut App) {
    if let Some(wait) = cx.global::<Services>().provider_wait.as_ref() {
        wait.cancel.store(true, Ordering::Release);
    }
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
    let generation = cx.global::<Services>().auth_generation;
    run(
        TaskKind::CheckServer,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| seams.check_server(endpoint),
        move |result, cx| match result {
            Some(Ok((endpoint, signup))) => {
                if cx.global::<Services>().auth_generation != generation {
                    done(
                        &tx,
                        Err("The server changed while it was being checked. Try again.".into()),
                    );
                    return;
                }
                set_server(Some(endpoint.clone()), cx);
                {
                    let services = cx.global_mut::<Services>();
                    services.provider_signup = None;
                    services.signup = Some((endpoint.clone(), signup));
                }
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

/// Fetches the current server's provider advertisement while a signed-out
/// form is open. Answers are tagged with their server so an older request
/// cannot replace the methods for a newer server. An omitted `auth` is the
/// old-server password-only behavior; a health error likewise leaves the
/// password form available.
pub fn check_auth(cx: &mut App) {
    let server = form_server(cx);
    let services = cx.global_mut::<Services>();
    let generation = services.auth_generation;
    let request = (server.clone(), generation);
    if services.auth_asking.as_ref() == Some(&request)
        || services.auth_attempted.as_ref() == Some(&request)
    {
        return;
    }
    let Some(backend) = services.backend.clone() else {
        services.auth = Some((server, kanso_sync::ServerAuth::default()));
        services.auth_attempted = Some(request);
        return;
    };
    services.auth_asking = Some(request.clone());
    services.auth_attempted = Some(request.clone());
    let (tx, rx) = async_channel::bounded(1);
    let asked = server.clone();
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            tx.try_send(kanso_sync::server_auth(backend.as_ref(), &asked))
                .ok();
        });
    if spawned.is_err() {
        let services = cx.global_mut::<Services>();
        if services.auth_asking.as_ref() == Some(&request) {
            services.auth_asking = None;
        }
        return;
    }
    cx.spawn(async move |cx| {
        let answer = rx.recv().await.ok();
        cx.update(|cx| {
            let shown = form_server(cx);
            let visible = account_page_showing(cx) && !cx.global::<Services>().signed_in();
            let services = cx.global_mut::<Services>();
            if services.auth_asking.as_ref() == Some(&request) {
                services.auth_asking = None;
            }
            if shown == server
                && visible
                && auth_answer_is_current(
                    &server,
                    generation,
                    &shown,
                    services.auth_generation,
                    services.auth_attempted.as_ref(),
                    &request,
                )
            {
                let auth = match answer {
                    Some(Ok(auth)) => auth,
                    _ => kanso_sync::ServerAuth::default(),
                };
                services.auth = Some((server, auth));
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

/// Starts a fresh health request for a newly opened signed-out form.
pub fn refresh_auth(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    services.auth_generation = services.auth_generation.wrapping_add(1);
    services.auth = None;
    services.auth_asking = None;
    services.auth_attempted = None;
    services.signup = None;
    services.provider_signup = None;
    check_auth(cx);
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
            let details = match details_from(usage, devices) {
                Ok(details) => details,
                Err(reason) => {
                    services::end_session(reason, cx);
                    return;
                }
            };
            services.details = details;
            cx.refresh_windows();
            if stale {
                refresh_details(cx);
            }
        });
    })
    .detach();
}

/// Fetches the current account's ways in for Settings › Account. Every page
/// opening starts a new generation, so an older answer cannot replace it.
/// Only an unlocked `SignedIn` session is eligible; provider-only locked
/// sessions never reach this endpoint through the app.
pub fn refresh_account_methods(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    if !services.signed_in() || services.has_locked_session() {
        return;
    }
    let (Some(session), Some(backend)) = (services.session(), services.backend.clone()) else {
        return;
    };
    let generation = services.account_methods.begin_refresh();

    let (tx, rx) = async_channel::bounded(1);
    let asked = session.clone();
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            let methods = kanso_sync::account::methods(backend.as_ref(), &asked);
            let passkeys = kanso_sync::server_auth(backend.as_ref(), &asked.server)
                .ok()
                .and_then(|auth| auth.passkeys);
            tx.try_send((methods, passkeys)).ok();
        });
    if spawned.is_err() {
        cx.global_mut::<Services>()
            .account_methods
            .finish_failure(generation, SOMETHING_WRONG.to_owned());
        cx.refresh_windows();
        return;
    }
    cx.spawn(async move |cx| {
        let answer = rx.recv().await.ok();
        cx.update(|cx| {
            let services = cx.global_mut::<Services>();
            let current_session = services
                .session()
                .is_some_and(|now| Arc::ptr_eq(&now, &session));
            if !current_session {
                return;
            }
            let Some((answer, passkeys)) = answer else {
                services
                    .account_methods
                    .finish_failure(generation, SOMETHING_WRONG.to_owned());
                cx.refresh_windows();
                return;
            };
            match answer {
                Ok(methods) => {
                    if !services.account_methods.finish_success(generation, methods) {
                        return;
                    }
                    services.account_methods.passkeys = passkeys;
                    cx.refresh_windows();
                }
                Err(AccountError::Unauthorized { reason }) => {
                    if services.account_methods.generation != generation {
                        return;
                    }
                    services::end_session(reason, cx);
                }
                Err(err) => {
                    services.account_methods.finish_failure(
                        generation,
                        words::error_words(&err, ErrorContext::Account),
                    );
                    cx.refresh_windows();
                }
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
    let Some(seams) = get_seams(cx) else {
        return;
    };
    let account = cx.global::<AppState>().config.sync.account.clone();
    let asked = account.clone();
    let (tx, rx) = async_channel::bounded(1);
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            let restored =
                seams.restore(account.as_ref().map(|a| (a.origin.as_str(), a.id.as_str())));
            let library_id = restored
                .as_ref()
                .map(|restored| restored.library_id.clone());
            tx.try_send(restored).ok();
            if let Some(library_id) = library_id {
                // The session is in; old tokens can take their time.
                seams.flush_logouts(&library_id);
            }
        });
    if spawned.is_err() {
        return;
    }
    cx.global_mut::<Services>().restoring = true;
    cx.spawn(async move |cx| {
        let answer = rx.recv().await.ok().flatten();
        cx.update(|cx| {
            cx.global_mut::<Services>().restoring = false;
            let Some(Restored {
                library_id,
                loaded,
                kept,
                pending,
                local,
            }) = answer
            else {
                cx.refresh_windows();
                return;
            };
            let services = cx.global_mut::<Services>();
            services.library_id = Some(library_id.clone());
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
                    if !matches!(&pending, Ok(Some(_)))
                        && cx.global::<AppState>().config.sync.account.is_some()
                    {
                        AppState::update_config(cx, |config| config.sync.account = None);
                    }
                }
                // The Keychain wouldn't say (locked): signed out for now,
                // with the account kept for the next launch.
                Err(err) => eprintln!("kanso: reading the account from the Keychain: {err}"),
            }
            // A valid full session above wins. Otherwise resume the exact
            // provider token/approval persisted for this library, whether
            // or not the account config currently names it.
            if !cx.global::<Services>().signed_in() {
                match pending {
                    Ok(Some(locked)) => {
                        let services = cx.global_mut::<Services>();
                        services.locked_has_password = Some(locked.has_password);
                        services.locked_session = Some(Arc::new(locked.clone()));
                        services.account = AccountState::SignedInLocked {
                            username: locked.username.clone(),
                        };
                        let unlocked = candidate_from_local(&locked, &library_id, local);
                        if let Some(candidate) = unlocked {
                            finish_unlocked_candidate(candidate, None, None, cx);
                        } else if locked.approval_resume().is_some()
                            && let Some(seams) = get_seams(cx)
                            && let Ok(device) = seams.device()
                        {
                            let window = cx.active_window();
                            wait_for_device_approval(locked, device, window, cx);
                        }
                    }
                    Ok(None) => {}
                    Err(err) => {
                        eprintln!("kanso: reading locked pending session: {err}");
                        notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
                    }
                }
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

// MARK: Device approval

/// What the waiting worker's thread sends back: its steps, then its end.
enum WaitMessage {
    Update(WaitUpdate),
    Done(Box<Result<UnlockedCandidate, WaitEnd>>),
}

/// Invalid or ended locked credentials cannot remain available to a later
/// action. Clear the app state even if the Keychain refuses deletion; the
/// sync API still tries logout and queues it when possible.
fn end_invalid_locked_session(locked: LockedSession, failure: Failure, cx: &mut App) {
    let Some(seams) = get_seams(cx) else {
        clear_locked_account(cx);
        return;
    };
    let thread = seams.clone();
    run(
        TaskKind::SignOut,
        Step::Contacting,
        false,
        None,
        cx,
        move |_| thread.end_locked(&locked),
        move |result, cx| {
            let keychain_failed = failure.keychain_warning
                || matches!(
                    &result,
                    Some(Err(Failure {
                        keychain_warning: true,
                        ..
                    }))
                );
            if let Some(Err(error)) = result {
                eprintln!("kanso: ending invalid locked sign-in: {}", error.words);
            }
            clear_locked_account(cx);
            if keychain_failed {
                notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
            }
        },
    );
}

/// Requests or resumes approval. The worker only creates, polls, names and
/// unlocks; binding decisions and promotion return to the UI thread.
pub fn wait_for_device_approval(
    locked: LockedSession,
    device: DeviceIdentity,
    window: Option<AnyWindowHandle>,
    cx: &mut App,
) {
    if cx
        .try_global::<Services>()
        .is_some_and(|s| s.approval.waiting.is_some())
    {
        return;
    }
    let started = Clock::now_ms(cx);
    cx.global_mut::<Services>().approval.waiting =
        Some(crate::device_approval::WaitingState::requesting(started));
    let Some(seams) = get_seams(cx) else {
        cx.global_mut::<Services>().approval.waiting = None;
        return;
    };
    let (tx, rx) = async_channel::unbounded();
    let locked_for_cleanup = locked.clone();
    let cancel = cx
        .global::<Services>()
        .approval
        .waiting
        .as_ref()
        .map(|w| w.cancel.clone());
    let thread = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            let result = seams.wait_for_approval(
                &locked,
                &device,
                cancel.as_deref(),
                &|update| {
                    tx.try_send(WaitMessage::Update(update)).ok();
                },
                &std::thread::sleep,
            );
            tx.try_send(WaitMessage::Done(Box::new(result))).ok();
        });
    if thread.is_err() {
        cx.global_mut::<Services>().approval.waiting = None;
        return;
    }
    cx.spawn(async move |cx| {
        while let Ok(message) = rx.recv().await {
            let mut completed = None;
            cx.update(|cx| {
                let now = Clock::now_ms(cx);
                let services = cx.global_mut::<Services>();
                let Some(waiting) = services.approval.waiting.as_mut() else {
                    return;
                };
                match message {
                    WaitMessage::Update(update) => waiting.apply(update, now),
                    WaitMessage::Done(result) => completed = Some(*result),
                }
                cx.refresh_windows();
            });
            if let Some(result) = completed {
                match result {
                    Ok(candidate) => cx.update(|cx| {
                        cx.global_mut::<Services>().approval.waiting = None;
                        finish_unlocked_candidate(candidate, window, None, cx);
                    }),
                    Err(
                        end @ WaitEnd::Failed(Failure {
                            locked_cleanup: true,
                            ..
                        }),
                    ) => cx.update(|cx| {
                        if let WaitEnd::Failed(failure) = end {
                            end_invalid_locked_session(locked_for_cleanup, failure, cx);
                        }
                    }),
                    Err(end) => cx.update(|cx| {
                        if let Some(waiting) = cx.global_mut::<Services>().approval.waiting.as_mut()
                        {
                            waiting.end(&end);
                        }
                        cx.refresh_windows();
                    }),
                }
                break;
            }
        }
    })
    .detach();
    cx.refresh_windows();
}

pub fn restart_device_approval(window: Option<AnyWindowHandle>, cx: &mut App) {
    let Some(seams) = get_seams(cx) else {
        return;
    };
    let Ok(device) = seams.device() else {
        return;
    };
    let locked = match keychain::load_locked_pending(seams.secrets.as_ref(), &device.library_id) {
        Ok(Some(locked)) => locked,
        Ok(None) => {
            notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
            return;
        }
        Err(error) => {
            notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
            eprintln!("kanso: reloading locked pending session: {error}");
            return;
        }
    };
    {
        let services = cx.global_mut::<Services>();
        services.locked_has_password = Some(locked.has_password);
        services.locked_session = Some(Arc::new(locked.clone()));
        services.account = AccountState::SignedInLocked {
            username: locked.username.clone(),
        };
    }
    cx.global_mut::<Services>().approval.waiting = None;
    wait_for_device_approval(locked, device, window, cx);
}

/// Cancels a locked sign-in. `discard_locked_session` attempts both pending
/// deletion and logout even when deletion fails; the app state is cleared
/// either way so a Keychain error cannot leave the editor pretending to be
/// an active approval session.
pub fn cancel_locked_session(cx: &mut App) {
    let Some(locked) = cx.global::<Services>().locked_session.clone() else {
        return;
    };
    if let Some(waiting) = cx.global::<Services>().approval.waiting.as_ref() {
        waiting
            .cancel
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
    let (tx, _rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return;
    };
    let thread = seams.clone();
    run(
        TaskKind::SignOut,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| thread.cancel_locked(&locked),
        move |result, cx| {
            let keychain_failed = matches!(&result, Some(Err(AccountError::Keychain(_))));
            if let Some(Err(error)) = result.as_ref() {
                eprintln!("kanso: cancelling locked sign-in: {error}");
            }
            clear_locked_account(cx);
            if keychain_failed {
                notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
            }
            done(&tx, Ok(()));
        },
    );
}

/// Uses the persisted provider LoginDevice to replace the locked token with a
/// password-authenticated session, then returns it to UI-thread consent.
pub fn unlock_locked_password(
    password: Password,
    window: Option<AnyWindowHandle>,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(locked) = cx.global::<Services>().locked_session.clone() else {
        done(&tx, Err(SOMETHING_WRONG.into()));
        return rx;
    };
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let thread = seams.clone();
    run(
        TaskKind::SignIn,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |step| thread.unlock_with_password(&locked, &password, step),
        move |result, cx| match result {
            Some(Ok(candidate)) => finish_unlocked_candidate(candidate, window, Some(tx), cx),
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

/// Recovers a locked provider session using its own server and username.
/// Rotation uses the existing recovery transaction and exposes the new key
/// before binding consent is requested.
pub fn recover_locked(
    key: RecoveryKey,
    password: Password,
    window: Option<AnyWindowHandle>,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(locked) = cx.global::<Services>().locked_session.clone() else {
        done(&tx, Err(SOMETHING_WRONG.into()));
        return rx;
    };
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let thread = seams.clone();
    run(
        TaskKind::Recover,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |step| thread.recover_locked(&locked, &key, &password, step),
        move |result, cx| match result {
            Some(Ok((candidate, recovery_key))) => {
                show_key(&candidate.session, recovery_key, cx);
                finish_unlocked_candidate(candidate, window, Some(tx), cx);
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

#[cfg(test)]
mod passkey_end_tests {
    use super::{ErrorContext, PasskeyEnd, PasskeyFlowError, PinProblem, passkey_end, words};

    fn said(pin: Option<PinProblem>) -> Option<String> {
        let failed = PasskeyFlowError::Ceremony(kanso_sync::PasskeyCeremonyError::Failed);
        match passkey_end(failed, pin, true, ErrorContext::Passkey) {
            PasskeyEnd::Failed(failure) => Some(failure.words),
            _ => None,
        }
    }

    /// Each thing a key says about its PIN is its own line: a wrong PIN
    /// isn't "enter your PIN", and a locked one isn't "try again".
    #[test]
    fn each_pin_problem_says_what_to_do() {
        let lines = [
            said(Some(PinProblem::Required)),
            said(Some(PinProblem::Invalid { retries: Some(3) })),
            said(Some(PinProblem::Invalid { retries: Some(1) })),
            said(Some(PinProblem::Invalid { retries: None })),
            said(Some(PinProblem::AuthBlocked)),
            said(Some(PinProblem::Blocked)),
            said(None),
        ]
        .map(Option::unwrap);
        assert_eq!(lines[0], words::SECURITY_KEY_PIN);
        assert!(
            lines[1].contains("after 3 more wrong tries"),
            "{}",
            lines[1]
        );
        assert!(lines[2].contains("one more wrong try"), "{}", lines[2]);
        assert!(!lines[3].contains("more wrong"), "{}", lines[3]);
        assert_eq!(lines[4], words::SECURITY_KEY_PIN_AUTH_BLOCKED);
        assert_eq!(lines[5], words::SECURITY_KEY_PIN_BLOCKED);
        assert_eq!(lines[6], words::SECURITY_KEY_FAILED);
        let distinct: std::collections::HashSet<_> = lines.iter().collect();
        assert_eq!(distinct.len(), lines.len());
    }
}

#[cfg(test)]
mod provider_wait_tests {
    use super::{clear_provider_wait_if_owned, provider_request_matches};
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn provider_answer_rejects_a_new_server_generation_even_after_a_to_b_to_a() {
        assert!(provider_request_matches("a", 7, "a", 7));
        assert!(!provider_request_matches("b", 7, "a", 7));
        assert!(!provider_request_matches("a", 8, "a", 7));
    }

    #[test]
    fn stale_provider_callback_does_not_clear_a_newer_wait() {
        let old = Arc::new(AtomicBool::new(false));
        let new = Arc::new(AtomicBool::new(false));
        let mut wait = Some(crate::services::ProviderWait {
            provider: "GitHub",
            user_code: Some("CODE".into()),
            verification_uri: Some("https://github.com/login/device".into()),
            cancel: new.clone(),
        });

        clear_provider_wait_if_owned(&mut wait, &old);
        assert!(wait.is_some(), "the new flow's wait stays visible");

        clear_provider_wait_if_owned(&mut wait, &new);
        assert!(wait.is_none(), "the owning callback clears its wait");
    }
}
