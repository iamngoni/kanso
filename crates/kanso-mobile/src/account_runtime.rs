//! What the account tasks of one library share: the one-task-at-a-time
//! flag, the question a sign-in is waiting on, the provider flow that is
//! running, the passkey requests' listeners, and where the account stands.
//!
//! The account's state is not kept here. It is read from what is durable (the
//! library's binding, the session in the secret store, a locked sign-in's
//! pending item), so a restart finds the phone where it was; only a question
//! that is waiting for an answer lives in memory.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use kanso_engine::account::provider::ProviderSignup;
use kanso_engine::account::words::{self, ErrorContext, Question};
use kanso_engine::account::{Authenticated, BUSY, Failure, LIBRARY_TROUBLE, UnlockedCandidate};
use kanso_sync::{LockedSession, RecoveryKey, keychain, meta_keys};

use crate::account_model::{
    AccountState, BindingAnswer, BindingQuestion, LockedInfo, PasskeyRequest, RecoveryKeyText,
    SignedUp,
};
use crate::boundary::held;
use crate::errors::KansoError;
use crate::library::Inner;
use crate::stream::{Listeners, Sink};
use crate::syncing::SyncHandle;

/// A sign-up that waits for the person to say the notes may be uploaded.
/// Takes the busy flag it will run under.
pub(crate) type SignUpLater =
    Box<dyn FnOnce(Busy) -> Result<SignedUp, KansoError> + Send + 'static>;

/// What a question is waiting to do once it is answered.
pub(crate) enum Held {
    /// A sign-in or recovery that proved the account: the new session, not
    /// saved yet, and the library it would bind.
    Auth {
        auth: Box<Authenticated>,
        question: BindingQuestion,
    },
    /// An unlock (approval, password, recovery key or a saved key) whose
    /// candidate waits to be promoted.
    Unlock {
        candidate: Box<UnlockedCandidate>,
        question: BindingQuestion,
    },
    /// A sign-up that would upload notes from another account. Nothing has
    /// been sent to the server yet.
    SignUp {
        later: SignUpLater,
        question: BindingQuestion,
    },
}

impl Held {
    pub fn question(&self) -> &BindingQuestion {
        match self {
            Held::Auth { question, .. }
            | Held::Unlock { question, .. }
            | Held::SignUp { question, .. } => question,
        }
    }
}

/// The provider flow that is running: its cancel flag, and where the end of
/// Apple's sign-in session is handed in.
pub(crate) struct ProviderRun {
    pub cancel: Arc<AtomicBool>,
    pub apple: Option<std::sync::mpsc::Sender<String>>,
}

/// The shared state of one library's account tasks.
#[derive(Default)]
pub(crate) struct Runtime {
    /// One account task runs at a time.
    busy: AtomicBool,
    /// Whether the launch check is running; tasks wait for it.
    restoring: Mutex<bool>,
    restored_cv: Condvar,
    /// Whether the launch check has started.
    pub restored: AtomicBool,
    pub listeners: Listeners<AccountState>,
    /// The state last sent to the listeners.
    pub published: Mutex<Option<AccountState>>,
    pub held: Mutex<Option<Held>>,
    pub provider: Mutex<Option<ProviderRun>>,
    /// A provider identity no account has, waiting for a username.
    pub signup: Mutex<Option<ProviderSignup>>,
    /// Set to end the device approval that is being waited for.
    pub approval: Mutex<Option<Arc<AtomicBool>>>,
    pub passkey_sinks: Listeners<PasskeyRequest>,
    /// Ends a passkey ceremony that is waiting for the sheet.
    pub ceremony_cancel: AtomicBool,
    /// The recovery key shown last, so "I've saved it" settles that one.
    pub shown_key: Mutex<Option<RecoveryKey>>,
}

impl Runtime {
    /// Marks the launch check as running. False if it already ran.
    pub fn begin_restore(&self) -> bool {
        if self.restored.swap(true, Ordering::AcqRel) {
            return false;
        }
        *held(&self.restoring) = true;
        true
    }

    pub fn end_restore(&self) {
        *held(&self.restoring) = false;
        self.restored_cv.notify_all();
    }
}

/// The right to run an account task: released when dropped.
pub(crate) struct Busy {
    inner: Arc<Inner>,
}

impl Drop for Busy {
    fn drop(&mut self) {
        self.inner.account.busy.store(false, Ordering::Release);
    }
}

/// Takes the one account task slot, once the launch check is done. An error
/// when another task has it.
pub(crate) fn begin(inner: &Arc<Inner>) -> Result<Busy, KansoError> {
    {
        let mut restoring = held(&inner.account.restoring);
        while *restoring {
            restoring = inner
                .account
                .restored_cv
                .wait(restoring)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }
    inner
        .account
        .busy
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .map_err(|_| KansoError::invalid(BUSY))?;
    Ok(Busy {
        inner: inner.clone(),
    })
}

/// [`begin`] for a task that starts something new: refused while a question
/// waits for its answer.
pub(crate) fn begin_new(inner: &Arc<Inner>) -> Result<Busy, KansoError> {
    let busy = begin(inner)?;
    if held(&inner.account.held).is_some() {
        return Err(KansoError::invalid(
            "Answer the question on screen first, or cancel it.",
        ));
    }
    Ok(busy)
}

/// A stream's sink shared with the thread that feeds it, and whether the
/// listener is still there.
pub(crate) struct Out<T> {
    sink: Mutex<Sink<T>>,
    gone: AtomicBool,
}

impl<T> Out<T> {
    pub fn new(sink: impl FnMut(T) -> bool + Send + 'static) -> Out<T> {
        Out {
            sink: Mutex::new(Box::new(sink)),
            gone: AtomicBool::new(false),
        }
    }

    /// Sends `value`. False once the listener has said it is gone (or its
    /// sink panicked), and then nothing more is sent.
    pub fn emit(&self, value: T) -> bool {
        if self.gone.load(Ordering::Acquire) {
            return false;
        }
        let mut sink = held(&self.sink);
        let alive =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sink(value))).unwrap_or(false);
        if !alive {
            self.gone.store(true, Ordering::Release);
        }
        alive
    }
}

// ----------------------------------------------------------------- state

/// Where the account stands, read from what is durable and the question that
/// waits.
pub(crate) fn current(inner: &Arc<Inner>) -> Result<AccountState, KansoError> {
    if let Some(waiting) = held(&inner.account.held).as_ref() {
        return Ok(AccountState::Question(waiting.question().clone()));
    }
    let sync = SyncHandle {
        inner: inner.clone(),
    };
    if let Some(session) = sync.load_session()? {
        return Ok(AccountState::SignedIn {
            username: session.username.clone(),
            server: session.server.clone(),
        });
    }
    Ok(match locked_session(inner)? {
        Some(locked) => AccountState::Locked(locked_info(&locked)),
        None => AccountState::SignedOut,
    })
}

/// The locked sign-in this library holds, if any.
pub(crate) fn locked_session(inner: &Arc<Inner>) -> Result<Option<LockedSession>, KansoError> {
    let Some(library_id) = inner.store.meta(meta_keys::LIBRARY_ID)? else {
        return Ok(None);
    };
    Ok(keychain::load_locked_pending(
        inner.secrets.as_ref(),
        &library_id,
    )?)
}

pub(crate) fn locked_info(locked: &LockedSession) -> LockedInfo {
    LockedInfo {
        username: locked.username.clone(),
        server: locked.server.clone(),
        has_password: locked.has_password,
        approval_pending: locked.approval_resume().is_some(),
    }
}

/// Tells the listeners where the account stands now, if that changed.
pub(crate) fn publish(inner: &Arc<Inner>) {
    if inner.account.listeners.is_empty() {
        return;
    }
    let Ok(state) = current(inner) else {
        return;
    };
    let mut last = held(&inner.account.published);
    if last.as_ref() == Some(&state) {
        return;
    }
    *last = Some(state.clone());
    inner.account.listeners.send(state);
}

/// The question as the shell asks it: Add, then Replace; Cancel is always
/// offered by the shell.
pub(crate) fn binding_question(question: Question) -> BindingQuestion {
    let answers = question
        .answers
        .iter()
        .filter_map(|answer| match answer.choice {
            kanso_sync::Choice::Add => Some(BindingAnswer::Add),
            kanso_sync::Choice::Replace => Some(BindingAnswer::Replace),
            kanso_sync::Choice::Cancel => None,
        })
        .collect();
    BindingQuestion {
        title: question.title,
        body: question.body,
        answers,
    }
}

/// A recovery key as the person writes it down.
pub(crate) fn key_text(key: &RecoveryKey) -> RecoveryKeyText {
    RecoveryKeyText { key: key.display() }
}

// ---------------------------------------------------------------- failures

/// What a failed account task is to the shell. The task's words are the
/// engine's; the kind is told from them, because the engine's [`Failure`]
/// keeps the words and not the error they came from.
pub(crate) fn failure_error(failure: &Failure, context: ErrorContext) -> KansoError {
    use kanso_sync::AccountError as E;
    let message = failure.words.clone();
    if failure.signed_out.is_some() {
        return KansoError::SignedOut;
    }
    if failure.keychain_warning {
        return KansoError::Auth { message };
    }
    let said = |err: E| words::error_words(&err, context) == message;
    if said(E::InvalidCredentials)
        || said(E::AccountDisabled)
        || said(E::UsernameMismatch)
        || said(E::KeyMismatch)
    {
        return KansoError::Auth { message };
    }
    if said(E::Transport(String::new()))
        || said(E::Internal)
        || said(E::Misconfigured)
        || message.starts_with("Too many attempts.")
    {
        return KansoError::Network { message };
    }
    if message == LIBRARY_TROUBLE
        || message == words::CREATED_NOT_SIGNED_IN
        || message == words::CREATED_NOT_SIGNED_IN_PASSKEY
        || message == words::CHANGED_NOT_SIGNED_IN
    {
        return KansoError::Storage { message };
    }
    KansoError::Invalid { message }
}

#[cfg(test)]
mod tests {
    use kanso_engine::account::words::{ErrorContext, error_words};
    use kanso_sync::AccountError;
    use proptest::prelude::*;

    use super::*;

    fn failure(words: String) -> Failure {
        Failure::words(&words)
    }

    // no-prop-test: a table of the failures whose kind the shell tells apart.
    #[test]
    fn failures_are_told_apart_by_their_words() {
        let sign_in = ErrorContext::SignIn;
        let of = |err: AccountError, context| {
            failure_error(&failure(error_words(&err, context)), context)
        };
        assert!(matches!(
            of(AccountError::InvalidCredentials, sign_in),
            KansoError::Auth { .. }
        ));
        assert!(matches!(
            of(AccountError::InvalidCredentials, ErrorContext::Recovery),
            KansoError::Auth { .. }
        ));
        assert!(matches!(
            of(AccountError::AccountDisabled, sign_in),
            KansoError::Auth { .. }
        ));
        assert!(matches!(
            of(AccountError::KeyMismatch, sign_in),
            KansoError::Auth { .. }
        ));
        assert!(matches!(
            of(AccountError::Transport("no route".into()), sign_in),
            KansoError::Network { .. }
        ));
        assert!(matches!(
            of(AccountError::RateLimited { retry_after: 90 }, sign_in),
            KansoError::Network { .. }
        ));
        assert!(matches!(
            of(AccountError::UsernameTaken, ErrorContext::SignUp),
            KansoError::Invalid { .. }
        ));
        assert!(matches!(
            failure_error(&Failure::words(LIBRARY_TROUBLE), sign_in),
            KansoError::Storage { .. }
        ));
        assert!(matches!(
            failure_error(
                &Failure::keychain(kanso_sync::SecretError("locked".into())),
                sign_in
            ),
            KansoError::Auth { .. }
        ));
        let mut ended = Failure::words("x");
        ended.signed_out = Some(kanso_sync::SignOutReason::Revoked);
        assert_eq!(failure_error(&ended, sign_in), KansoError::SignedOut);
    }

    proptest! {
        /// A rate limit is a network problem for any wait, and its words say
        /// how long.
        #[test]
        fn prop_a_rate_limit_is_a_network_error_whatever_the_wait(wait in 0u64..100_000) {
            let words = error_words(&AccountError::RateLimited { retry_after: wait }, ErrorContext::SignIn);
            let err = failure_error(&failure(words.clone()), ErrorContext::SignIn);
            prop_assert_eq!(err, KansoError::Network { message: words });
        }

        /// The kind never depends on the context for a transport failure.
        #[test]
        fn prop_no_context_makes_the_network_a_wrong_password(ix in 0usize..9) {
            let contexts = [
                ErrorContext::SignUp, ErrorContext::SignIn, ErrorContext::Recovery,
                ErrorContext::CurrentPassword, ErrorContext::Server, ErrorContext::Account,
                ErrorContext::Passkey, ErrorContext::PasswordOrRecovery, ErrorContext::PasskeyOrRecovery,
            ];
            let context = contexts[ix];
            let words = error_words(&AccountError::Transport("down".into()), context);
            let is_network = matches!(
                failure_error(&failure(words), context),
                KansoError::Network { .. }
            );
            prop_assert!(is_network);
        }
    }

    // no-prop-test: the shell is told Add then Replace, and never Cancel.
    #[test]
    fn a_question_offers_add_then_replace() {
        let plan = kanso_sync::Plan::Ask {
            from: None,
            notes: 3,
            unsynced: 1,
        };
        let question = words::binding_question(&plan, "ada").unwrap();
        let shown = binding_question(question);
        assert_eq!(
            shown.answers,
            vec![BindingAnswer::Add, BindingAnswer::Replace]
        );
        assert!(shown.body.contains("3 notes"));
    }

    // no-prop-test: two sinks, one that leaves.
    #[test]
    fn an_output_stops_sending_once_its_listener_is_gone() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        let out = Out::new(move |n: u8| {
            log.lock().unwrap().push(n);
            n < 2
        });
        assert!(out.emit(1));
        assert!(!out.emit(2));
        assert!(!out.emit(3));
        assert_eq!(*seen.lock().unwrap(), vec![1, 2]);
        let panics = Out::new(|_: u8| panic!("a bad listener"));
        assert!(!panics.emit(1));
    }
}
