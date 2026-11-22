//! Settings › Account › Sign-in methods (docs/ACCOUNTS.md › Account page ›
//! Signed in, › Account flows › Link a provider … Set a password, › Changing
//! a way in needs a fresh proof). Every change takes a fresh proof first:
//! the current password, a passkey assertion over a proof challenge, or a
//! provider sign-in. The proof is made on this device's account thread and
//! sent with the one change it authorizes; the PRF output of a passkey proof
//! is never asked for or sent.
//!
//! A change that needs a provider (a provider proof, or a link) runs in two
//! tasks around the browser, as a provider sign-in does: the first starts
//! the flow with the session's route, the main thread shows Apple's sheet
//! or the wait, and the second finishes it and makes the change.

use kanso_sync::passkey::PasskeyCeremony;
use kanso_sync::provider::{IdentityIntent, LinkOutcome};
use kanso_sync::{PasswordChanged, Proof};

use super::*;

/// How the person proves it's them for one change.
pub enum ProofWith {
    Password(Password),
    /// A passkey assertion, through the request's [`PasskeyMethod`].
    Passkey,
    /// A fresh sign-in at a provider already linked to the account.
    Provider(Provider),
}

/// A change to the account's ways in, or one of the two account actions
/// that a passwordless account makes with a proof.
pub enum MethodChange {
    AddPasskey,
    RemovePasskey {
        credential_id: Vec<u8>,
    },
    Link(Provider),
    Unlink(Provider),
    /// A first password, on an account made with a passkey or a provider.
    SetPassword(Password),
    NewRecoveryKey,
    DeleteAccount {
        typed: String,
    },
}

impl MethodChange {
    fn kind(&self) -> TaskKind {
        match self {
            MethodChange::SetPassword(_) => TaskKind::ChangePassword,
            MethodChange::NewRecoveryKey => TaskKind::NewRecoveryKey,
            MethodChange::DeleteAccount { .. } => TaskKind::DeleteAccount,
            _ => TaskKind::ChangeMethod,
        }
    }
}

/// One change and what it takes.
pub struct ChangeRequest {
    pub change: MethodChange,
    pub proof: ProofWith,
    /// The recovery key: required with a provider proof for a password, a
    /// new key or a deletion, and for removing the last way in.
    pub recovery: Option<RecoveryKey>,
    /// The ceremony for a passkey proof and for a new passkey: the
    /// platform's sheet, or a Linux security key.
    pub passkey: Option<PasskeyMethod>,
}

/// What a change came to.
enum Changed {
    /// A notification's words.
    Said(&'static str),
    PasskeyAdded {
        can_unlock: bool,
    },
    PasswordSet(PasswordChanged),
    NewKey(RecoveryKey),
    Deleted,
}

/// How a change failed on its thread, before the ceremony's PIN flag is
/// known.
enum Fail {
    Ceremony(PasskeyFlowError),
    Failed(Failure),
}

impl From<Failure> for Fail {
    fn from(failure: Failure) -> Self {
        Fail::Failed(failure)
    }
}

/// The words for an error while making the proof: only the proof's secret
/// was checked.
fn proof_context(proof: &ProofWith) -> ErrorContext {
    match proof {
        ProofWith::Password(_) => ErrorContext::CurrentPassword,
        ProofWith::Passkey => ErrorContext::Passkey,
        ProofWith::Provider(_) => ErrorContext::Account,
    }
}

/// The words for an error from the change itself. A password or passkey
/// proof is checked with the change, beside the recovery key when one goes
/// too, and the server doesn't say which failed; a provider proof was
/// checked by its flow already, so then it is the key.
fn change_context(proof: &ProofWith, recovery: bool) -> ErrorContext {
    match (proof, recovery) {
        (ProofWith::Password(_), true) => ErrorContext::PasswordOrRecovery,
        (ProofWith::Passkey, true) => ErrorContext::PasskeyOrRecovery,
        (ProofWith::Provider(_), true) => ErrorContext::Recovery,
        (proof, false) => proof_context(proof),
    }
}

/// Makes `request`'s change after its proof. Steps and the result reach the
/// form that started it; the method list is fetched again afterwards.
pub fn change_method(request: ChangeRequest, cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some((seams, session, library_id)) = begin_signed_in(&tx, cx) else {
        return rx;
    };
    if matches!(request.proof, ProofWith::Provider(_))
        || matches!(request.change, MethodChange::Link(_))
    {
        provider_change(seams, session, library_id, request, tx, cx);
        return rx;
    }
    let ChangeRequest {
        change,
        proof,
        recovery,
        passkey,
    } = request;
    let security_key = passkey.as_ref().is_some_and(PasskeyMethod::is_security_key);
    let (runner, serving) = match passkey {
        Some(method) => {
            let (runner, serving) = Runner::new(method, cx);
            (Some(runner), serving)
        }
        None => (None, None),
    };
    let kind = change.kind();
    let deleting = matches!(change, MethodChange::DeleteAccount { .. });
    let shown = session.clone();
    run(
        kind,
        Step::Contacting,
        deleting,
        Some(tx.clone()),
        cx,
        move |step| {
            let rule = runner.as_ref().map(Runner::rule);
            let proving = proof_context(&proof);
            let words = change_context(&proof, recovery.is_some());
            let (result, pin) = with_runner(runner, |ceremony| {
                let auth_key;
                let proof = match proof {
                    ProofWith::Password(password) => {
                        auth_key = kanso_sync::password_proof(
                            seams.backend.as_ref(),
                            &session,
                            &password,
                            step,
                        )
                        .map_err(|err| Failure::account(err, proving))?;
                        Proof::Password(&auth_key)
                    }
                    ProofWith::Passkey => {
                        passkey_proof(seams.backend.as_ref(), &session, ceremony, rule, step)?
                    }
                    ProofWith::Provider(_) => {
                        return Err(Failure::words(SOMETHING_WRONG).into());
                    }
                };
                apply(
                    &seams,
                    &session,
                    &library_id,
                    change,
                    &proof,
                    recovery.as_ref(),
                    ceremony,
                    rule,
                    words,
                    step,
                )
            });
            result.map_err(|fail| match fail {
                Fail::Ceremony(err) => passkey_end(err, pin, security_key, proving),
                Fail::Failed(failure) => PasskeyEnd::Failed(failure),
            })
        },
        move |result, cx| {
            drop(serving);
            finished(result, &shown, security_key, tx, cx);
        },
    );
    rx
}

/// Runs `work` with the ceremony when there is one.
fn with_runner<T>(
    runner: Option<Runner>,
    work: impl FnOnce(Option<&dyn PasskeyCeremony>) -> T,
) -> (T, Option<PinProblem>) {
    match runner {
        Some(runner) => runner.run(|ceremony| work(Some(ceremony))),
        None => (work(None), None),
    }
}

/// The relying party the server named is one this ceremony may be run for.
fn rp_allowed(rule: RpRule, server: &str, rp_id: &str, origin: &str) -> bool {
    rule.allows(rp_id, server) && kanso_sync::passkey_origin_allowed(origin, rp_id)
}

/// A passkey proof: a proof challenge, then the ceremony's assertion over it.
fn passkey_proof(
    backend: &dyn Backend,
    session: &Session,
    ceremony: Option<&dyn PasskeyCeremony>,
    rule: Option<RpRule>,
    step: &dyn Fn(Step),
) -> Result<Proof<'static>, Fail> {
    let (Some(ceremony), Some(rule)) = (ceremony, rule) else {
        return Err(Failure::words(SOMETHING_WRONG).into());
    };
    step(Step::Contacting);
    let start = kanso_sync::start_passkey_proof(backend, session, None)
        .map_err(|err| Failure::account(err, ErrorContext::Passkey))?;
    if !rp_allowed(
        rule,
        &session.server,
        &start.challenge.rp_id,
        &start.challenge.origin,
    ) {
        return Err(Fail::Ceremony(PasskeyFlowError::WrongRelyingParty));
    }
    let assertion = ceremony
        .assert(&start.challenge)
        .map_err(|err| Fail::Ceremony(PasskeyFlowError::Ceremony(err)))?;
    Ok(Proof::passkey(&start, assertion))
}

/// The change itself, with its proof.
#[allow(clippy::too_many_arguments, reason = "the change's whole context")]
fn apply(
    s: &Seams,
    session: &Session,
    library_id: &str,
    change: MethodChange,
    proof: &Proof,
    recovery: Option<&RecoveryKey>,
    ceremony: Option<&dyn PasskeyCeremony>,
    rule: Option<RpRule>,
    words: ErrorContext,
    step: &dyn Fn(Step),
) -> Result<Changed, Fail> {
    let b = s.backend.as_ref();
    let account = |err: AccountError| Fail::Failed(Failure::account(err, words));
    step(Step::Contacting);
    match change {
        MethodChange::AddPasskey => {
            let (Some(ceremony), Some(rule)) = (ceremony, rule) else {
                return Err(Failure::words(SOMETHING_WRONG).into());
            };
            let start = kanso_sync::start_add_passkey(b, session, proof).map_err(account)?;
            if !rp_allowed(
                rule,
                &session.server,
                &start.challenge.rp_id,
                &start.challenge.origin,
            ) {
                return Err(Fail::Ceremony(PasskeyFlowError::WrongRelyingParty));
            }
            let registration = ceremony
                .register(&start.challenge)
                .map_err(|err| Fail::Ceremony(PasskeyFlowError::Ceremony(err)))?;
            let added =
                kanso_sync::add_passkey(b, session, &start, registration).map_err(account)?;
            Ok(Changed::PasskeyAdded {
                can_unlock: added.prf,
            })
        }
        MethodChange::RemovePasskey { credential_id } => {
            kanso_sync::remove_passkey(b, session, &credential_id, proof, recovery)
                .map_err(account)?;
            Ok(Changed::Said(words::PASSKEY_REMOVED))
        }
        MethodChange::Unlink(provider) => {
            kanso_sync::unlink_identity(b, session, &provider, proof.to_wire(), recovery)
                .map_err(account)?;
            Ok(Changed::Said(words::PROVIDER_UNLINKED))
        }
        MethodChange::Link(_) => Err(Failure::words(SOMETHING_WRONG).into()),
        MethodChange::SetPassword(password) => kanso_sync::set_password(
            b,
            s.secrets.as_ref(),
            session,
            &password,
            proof,
            recovery,
            step,
        )
        .map(Changed::PasswordSet)
        .map_err(|err| Failure::rotation(err, words, words::KEY_UNCONFIRMED_PASSWORD).into()),
        MethodChange::NewRecoveryKey => kanso_sync::new_recovery_key_with_proof(
            b,
            s.secrets.as_ref(),
            session,
            proof,
            recovery,
            step,
        )
        .map(Changed::NewKey)
        .map_err(|err| Failure::rotation(err, words, words::KEY_UNCONFIRMED).into()),
        MethodChange::DeleteAccount { typed } => {
            kanso_sync::delete_account_with_proof(b, session, proof, recovery, &typed, step)
                .map_err(account)?;
            step(Step::Saving);
            kanso_sync::unbind(&s.store, s.secrets.as_ref()).map_err(Failure::bind)?;
            keychain::delete_recovery_key(s.secrets.as_ref(), library_id).ok();
            Ok(Changed::Deleted)
        }
    }
}

/// Shows how a change went, and fetches the method list again.
fn finished(
    result: Option<Result<Changed, PasskeyEnd>>,
    session: &Session,
    security_key: bool,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    match result {
        Some(Ok(Changed::Said(notice))) => {
            notify_windows(move || Notification::success(notice), cx);
            done(&tx, Ok(()));
        }
        Some(Ok(Changed::PasskeyAdded { can_unlock })) => {
            let notice = words::passkey_added(security_key, can_unlock);
            if can_unlock {
                notify_windows(move || Notification::success(notice), cx);
            } else {
                notify_windows(move || Notification::warning(notice), cx);
            }
            done(&tx, Ok(()));
        }
        Some(Ok(Changed::PasswordSet(changed))) => {
            show_pending(
                PendingKey {
                    key: changed.recovery_key,
                    username: session.username.clone(),
                    server: session.server.clone(),
                    note: Some(words::KEY_AFTER_PASSWORD),
                },
                cx,
            );
            notify_windows(|| Notification::success(words::PASSWORD_SET), cx);
            done(&tx, Ok(()));
        }
        Some(Ok(Changed::NewKey(key))) => {
            show_key(session, key, cx);
            done(&tx, Ok(()));
        }
        Some(Ok(Changed::Deleted)) => {
            let username = session.username.clone();
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
            return;
        }
        Some(Err(end)) => passkey_ended(end, &tx, cx),
        None => failed(&tx, None, cx),
    }
    refresh_account_methods(cx);
}

// MARK: Through a provider

/// The device a session's provider flow is bound to: the one the session
/// was created with (a passkey or provider sign-in's flow device), or this
/// library's own. The server checks it against the session's.
fn session_device(s: &Seams, session: &Session) -> Result<LoginDevice, Failure> {
    Ok(session.login_device(&s.device()?))
}

/// A link (after a password or passkey proof) or a provider proof: starts
/// the session's provider flow, shows the browser or Apple's sheet, then
/// finishes it and makes the change.
fn provider_change(
    seams: Seams,
    session: Arc<Session>,
    library_id: String,
    request: ChangeRequest,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let ChangeRequest {
        change,
        proof,
        recovery,
        passkey,
    } = request;
    let provider = match (&change, &proof) {
        (MethodChange::Link(provider), _) | (_, ProofWith::Provider(provider)) => provider.clone(),
        _ => unreachable!("only a link or a provider proof comes here"),
    };
    let security_key = passkey.as_ref().is_some_and(PasskeyMethod::is_security_key);
    let (runner, serving) = match passkey {
        Some(method) => {
            let (runner, serving) = Runner::new(method, cx);
            (Some(runner), serving)
        }
        None => (None, None),
    };
    let kind = change.kind();
    let thread = seams.clone();
    let start_session = session.clone();
    let start_provider = provider.clone();
    let linking = matches!(change, MethodChange::Link(_));
    let proving = proof_context(&proof);
    let words = change_context(&proof, recovery.is_some());
    run(
        kind,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |step| -> Result<(StartedProviderFlow, LoginDevice, ProviderRoute), PasskeyEnd> {
            let device = session_device(&thread, &start_session)?;
            // A link adds a way in: its start carries a password or
            // passkey proof. A provider proof needs only the session.
            let rule = runner.as_ref().map(Runner::rule);
            let (wire, pin) = with_runner(runner, |ceremony| -> Result<_, Fail> {
                if !linking {
                    return Ok(None);
                }
                let wire = match &proof {
                    ProofWith::Password(password) => {
                        let auth_key = kanso_sync::password_proof(
                            thread.backend.as_ref(),
                            &start_session,
                            password,
                            step,
                        )
                        .map_err(|err| Failure::account(err, proving))?;
                        Proof::Password(&auth_key).to_wire()
                    }
                    ProofWith::Passkey => passkey_proof(
                        thread.backend.as_ref(),
                        &start_session,
                        ceremony,
                        rule,
                        step,
                    )?
                    .to_wire(),
                    ProofWith::Provider(_) => {
                        return Err(Failure::account(AccountError::ReauthRequired, proving).into());
                    }
                };
                Ok(Some(wire))
            });
            let wire = wire.map_err(|fail| match fail {
                Fail::Ceremony(err) => passkey_end(err, pin, security_key, proving),
                Fail::Failed(failure) => PasskeyEnd::Failed(failure),
            })?;
            let route = ProviderRoute::Session {
                session: start_session,
                intent: if linking {
                    IdentityIntent::Link
                } else {
                    IdentityIntent::Proof
                },
                proof: wire,
            };
            let started = provider_begin(
                thread.backend.as_ref(),
                &Desktop,
                &route,
                &start_provider,
                &device,
                step,
                Some(crate::flavour::CURRENT.scheme),
            )
            .map_err(|err| Failure::account(err, ErrorContext::Account))?;
            Ok((started, device, route))
        },
        move |result, cx| {
            drop(serving);
            let (started, device, route) = match result {
                Some(Ok(started)) => started,
                Some(Err(end)) => {
                    passkey_ended(end, &tx, cx);
                    return;
                }
                None => {
                    failed(&tx, None, cx);
                    return;
                }
            };
            let Some((apple_callback, cancel)) = provider_wait(&started, &provider, &tx, cx) else {
                return;
            };
            finish_provider_change(
                seams,
                session,
                library_id,
                change,
                recovery,
                ProviderFlow {
                    provider,
                    device,
                    route,
                    started,
                    apple_callback,
                    cancel,
                },
                words,
                tx,
                cx,
            );
        },
    );
}

/// A started provider flow, on its way to its finish.
struct ProviderFlow {
    provider: Provider,
    device: LoginDevice,
    route: ProviderRoute,
    started: StartedProviderFlow,
    apple_callback: Option<Box<dyn AppleWait>>,
    cancel: Arc<AtomicBool>,
}

/// On the main thread: Apple's sheet, and the wait the page shows (GitHub's
/// code). `None` when the sheet couldn't open; the task has ended then.
#[allow(
    clippy::type_complexity,
    reason = "the sheet's receiver and the wait's flag"
)]
fn provider_wait(
    started: &StartedProviderFlow,
    provider: &Provider,
    tx: &Sender<Progress>,
    cx: &mut App,
) -> Option<(Option<Box<dyn AppleWait>>, Arc<AtomicBool>)> {
    let apple_callback = match (&started.start, provider) {
        (ProviderStart::Apple(apple), Provider::Apple) => {
            match crate::provider_browser::apple_sign_in(apple) {
                Ok(callback) => Some(Box::new(crate::provider_browser::AppleReceiver(callback))
                    as Box<dyn AppleWait>),
                Err(_) => {
                    failed(
                        tx,
                        Some(Failure::account(
                            AccountError::ProviderUnavailable,
                            ErrorContext::Account,
                        )),
                        cx,
                    );
                    return None;
                }
            }
        }
        _ => None,
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let (label, user_code, verification_uri) = match &started.start {
        ProviderStart::Apple(_) => ("Apple", None, None),
        ProviderStart::Google(_) => ("Google", None, None),
        ProviderStart::Github(start) => (
            "GitHub",
            Some(start.user_code.clone()),
            Some(start.verification_uri.clone()),
        ),
    };
    super::copy_device_code(user_code.as_deref(), cx);
    cx.global_mut::<Services>().provider_wait = Some(crate::services::ProviderWait {
        provider: label,
        user_code,
        verification_uri,
        cancel: cancel.clone(),
    });
    Some((apple_callback, cancel))
}

#[allow(clippy::too_many_arguments, reason = "the change's whole context")]
fn finish_provider_change(
    seams: Seams,
    session: Arc<Session>,
    library_id: String,
    change: MethodChange,
    recovery: Option<RecoveryKey>,
    flow: ProviderFlow,
    words: ErrorContext,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let kind = change.kind();
    let deleting = matches!(change, MethodChange::DeleteAccount { .. });
    let wait_cancel = flow.cancel.clone();
    let shown = session.clone();
    run(
        kind,
        Step::Contacting,
        deleting,
        Some(tx.clone()),
        cx,
        move |step| -> Result<Option<Changed>, PasskeyEnd> {
            let ProviderFlow {
                provider,
                device,
                route,
                started,
                apple_callback,
                cancel,
            } = flow;
            let flow_id = started.start.flow_id().to_owned();
            let answer = provider_finish(
                seams.backend.as_ref(),
                &Desktop,
                &route,
                &provider,
                &device,
                started,
                apple_callback,
                cancel,
                step,
            )
            .map_err(|err| Failure::account(err, ErrorContext::Account))?;
            let outcome = match answer {
                None => return Ok(None),
                Some(ProviderDone::Link(outcome)) => outcome,
                Some(ProviderDone::SignIn(_)) => {
                    return Err(Failure::words(SOMETHING_WRONG).into());
                }
            };
            match (change, outcome) {
                (MethodChange::Link(_), LinkOutcome::Linked { .. }) => {
                    Ok(Some(Changed::Said(words::PROVIDER_LINKED)))
                }
                (change, LinkOutcome::Proof { .. }) => {
                    let proof = Proof::Provider { flow: flow_id };
                    apply(
                        &seams,
                        &session,
                        &library_id,
                        change,
                        &proof,
                        recovery.as_ref(),
                        None,
                        None,
                        words,
                        step,
                    )
                    .map(Some)
                    .map_err(|fail| match fail {
                        Fail::Ceremony(err) => passkey_end(err, None, false, words),
                        Fail::Failed(failure) => PasskeyEnd::Failed(failure),
                    })
                }
                _ => Err(Failure::account(
                    AccountError::Protocol("a provider answer for another change".into()),
                    words,
                )
                .into()),
            }
        },
        move |result, cx| {
            clear_provider_wait_if_owned(
                &mut cx.global_mut::<Services>().provider_wait,
                &wait_cancel,
            );
            match result {
                Some(Ok(None)) => done(&tx, Ok(())),
                Some(Ok(Some(changed))) => finished(Some(Ok(changed)), &shown, false, tx, cx),
                Some(Err(end)) => finished(Some(Err(end)), &shown, false, tx, cx),
                None => finished(None, &shown, false, tx, cx),
            }
        },
    );
}
