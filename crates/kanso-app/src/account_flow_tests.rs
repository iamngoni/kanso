//! The account flows end to end, in real windows, against `MemoryServer` and
//! `MemorySecrets` (docs/ACCOUNTS.md › Account flows, › Library binding, ›
//! Services): sign-up and its recovery key, a second Mac signing in and the
//! question about its notes, another account's notes, signing out, a
//! password change, recovery, a new recovery key, deleting the account, and
//! a relaunch. Each test context is one Mac with its own library and
//! Keychain; the server is shared. Nothing here reaches a network or the
//! real Keychain. Signing in starts the real sync (`kanso_sync::sync` on its
//! own thread, as in the app), so the tests also check what reaches the
//! server and the other Macs, and that account tasks stop a running sync
//! first.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use async_channel::Receiver;
use gpui_kit::component::WindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Modifiers, TestAppContext, VisualTestContext};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use kanso_sync::api::LoginDevice;
use kanso_sync::{
    AccountError, Api, Backend, DeviceIdentity, KdfParams, Machine, MemorySecrets, MemoryServer,
    Password, Provider, ProviderStep, RecoveryKey, Route, SecretError, SecretStore, Session,
    SignOutReason, meta_keys,
};
use serde_json::json;

use crate::account_flow;
use crate::account_words as words;
use crate::config::AppConfig;
use crate::dialogs::Progress;
use crate::services::{self, AccountState, ProviderWait, Services, SyncState};
use crate::settings::SettingsPage;
use crate::workspace::{AppState, Workspace};

#[test]
fn stale_health_from_a_is_rejected_after_a_to_b_to_a() {
    let old_request = (SERVER.to_owned(), 10);
    let attempted_again = (SERVER.to_owned(), 12);
    assert!(!account_flow::auth_answer_is_current(
        SERVER,
        old_request.1,
        SERVER,
        12,
        Some(&attempted_again),
        &old_request,
    ));
    assert!(account_flow::auth_answer_is_current(
        SERVER,
        attempted_again.1,
        SERVER,
        12,
        Some(&attempted_again),
        &attempted_again,
    ));
}

#[test]
fn displayed_locked_state_holds_editor_writes_without_a_sync_session() {
    let mut services = Services::default();
    services.account = AccountState::SignedInLocked {
        username: "alice".into(),
    };

    assert!(services.has_locked_session());
    assert!(!services.signed_in());
    assert!(services.session().is_none());
}

#[gpui_kit::test]
fn cancel_provider_wait_requests_cancellation_without_hiding_the_active_wait(
    cx: &mut TestAppContext,
) {
    let cancel = Arc::new(AtomicBool::new(false));
    cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(Services::default());
        cx.global_mut::<Services>().provider_wait = Some(ProviderWait {
            provider: "GitHub",
            user_code: Some("CODE".into()),
            verification_uri: Some("https://github.com/login/device".into()),
            cancel: cancel.clone(),
        });
        account_flow::cancel_provider_wait(cx);
    });

    assert!(cancel.load(Ordering::Acquire));
    assert!(
        cx.read(|cx| cx.global::<Services>().provider_wait.is_some()),
        "the callback clears the wait after the worker observes cancellation"
    );
}

#[gpui_kit::test]
fn account_methods_refresh_fetches_for_unlocked_session_and_skips_locked_session(
    cx: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));

    key_saved(cx);
    let page = cx.read(|cx| a.workspace.read(cx).settings.read(cx).account.clone());
    cx.update_window(a.window, |_, window, cx| {
        page.update(cx, |page, cx| page.shown(cx));
        window.refresh();
    })
    .unwrap();
    wait_until(cx, "account sign-in methods", |cx| {
        cx.read(|cx| {
            let methods = &cx.global::<Services>().account_methods;
            !methods.loading && methods.last_successful.is_some()
        })
    });
    cx.read(|cx| {
        let state = &cx.global::<Services>().account_methods;
        assert!(state.last_successful.as_ref().unwrap().has_password);
        assert_eq!(state.failed, None);
    });

    cx.update(|cx| {
        let services = cx.global_mut::<Services>();
        services.clear_session(AccountState::SignedInLocked {
            username: "alice".into(),
        });
        services.account_methods = Default::default();
        account_flow::refresh_account_methods(cx);
    });
    cx.read(|cx| {
        assert_eq!(
            cx.global::<Services>().account_methods,
            Default::default(),
            "a locked session has no full Session and does not start a methods fetch"
        );
    });
}

/// Enters the same durable locked state as the provider success callback:
/// pending Keychain state is written before the UI exposes it.
fn enter_locked(
    cx: &mut TestAppContext,
    locked: kanso_sync::LockedSession,
    device: DeviceIdentity,
    window: AnyWindowHandle,
) {
    let (backend, secrets) = cx.read(|cx| {
        let services = cx.global::<Services>();
        (
            services.backend.clone().unwrap(),
            services.secrets.clone().unwrap(),
        )
    });
    account_flow::save_provider_pending(backend.as_ref(), secrets.as_ref(), &locked).unwrap();
    cx.update(|cx| account_flow::enter_locked(locked, device, Some(window), cx));
}

#[gpui_kit::test]
fn provider_signin_persists_a_locked_session_without_sync_or_account_events(
    cx: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let source = password_account(&server, "alice");
    let a = mac(cx, &server, "Studio", 1);
    let (login_device, provider_session) = provider_signin(&server, &source.account_id);
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    let locked = account_flow::locked_provider_session(
        SERVER.to_owned(),
        login_device.clone(),
        provider_session,
        &device.library_id,
    )
    .unwrap();

    enter_locked(cx, locked, device, a.window);
    // Read using the app's library ID: the pending item is independent of
    // config.sync.account and the local library has not been bound yet.
    let library_id = cx.read(|cx| cx.global::<Services>().library_id.clone().unwrap());
    let pending = kanso_sync::keychain::load_locked_pending(a.secrets.as_ref(), &library_id)
        .unwrap()
        .expect("provider success writes the locked item first");
    assert_eq!(pending.device_id, login_device.id);
    assert_eq!(pending.library_id(), Some(library_id.as_str()));
    assert!(cx.read(|cx| cx.global::<Services>().locked_has_password == Some(true)));
    assert_eq!(
        pending.wrapped_key, None,
        "providers never return a password wrap"
    );
    assert!(matches!(account(cx), AccountState::SignedInLocked { .. }));
    assert!(cx.read(|cx| cx.global::<Services>().session().is_none()));
    assert!(cx.read(|cx| !cx.global::<Services>().signed_in()));
    assert!(cx.read(|cx| cx.global::<Services>().sync == SyncState::Off));
    assert!(cx.read(|cx| cx.global::<Services>().approval.event_stop.is_none()));
    assert!(cx.update(|cx| cx.global_mut::<Services>().take_driver().is_none()));
    assert!(cx.read(|cx| cx.global::<AppState>().config.sync.account.is_none()));
    assert_eq!(a.bound_to(), None);
}

#[gpui_kit::test]
fn pending_provider_session_restores_without_a_config_account(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let source = password_account(&server, "alice");
    let a = mac(cx, &server, "Studio", 1);
    let (login_device, provider_session) = provider_signin(&server, &source.account_id);
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    let locked = account_flow::locked_provider_session(
        SERVER.to_owned(),
        login_device.clone(),
        provider_session,
        &device.library_id,
    )
    .unwrap();
    enter_locked(cx, locked, device, a.window);
    assert!(matches!(account(cx), AccountState::SignedInLocked { .. }));

    relaunched(cx, &server, &a);
    cx.update(account_flow::restore);
    restored(cx);

    assert!(matches!(account(cx), AccountState::SignedInLocked { .. }));
    assert!(cx.read(|cx| cx.global::<Services>().session().is_none()));
    assert!(cx.read(|cx| cx.global::<AppState>().config.sync.account.is_none()));
    assert!(cx.read(|cx| cx.global::<Services>().locked_has_password == Some(true)));
    let library_id = cx.read(|cx| cx.global::<Services>().library_id.clone().unwrap());
    let pending = kanso_sync::keychain::load_locked_pending(a.secrets.as_ref(), &library_id)
        .unwrap()
        .expect("restore retains the provider token");
    assert_eq!(pending.device_id, login_device.id);
    assert!(cx.read(|cx| cx.global::<Services>().approval.event_stop.is_none()));
    assert!(cx.update(|cx| cx.global_mut::<Services>().take_driver().is_none()));
}

#[gpui_kit::test]
fn expired_and_cancelled_approval_can_restart_without_losing_the_locked_token(
    cx: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let source = password_account(&server, "alice");
    let a = mac(cx, &server, "Studio", 0);
    let (login_device, provider_session) = provider_signin(&server, &source.account_id);
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    let library_id = device.library_id.clone();
    let locked = account_flow::locked_provider_session(
        SERVER.to_owned(),
        login_device,
        provider_session,
        &library_id,
    )
    .unwrap();
    enter_locked(cx, locked, device, a.window);

    let locked = cx.read(|cx| cx.global::<Services>().locked_session.clone().unwrap());
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    cx.update(|cx| {
        account_flow::wait_for_device_approval((*locked).clone(), device, Some(a.window), cx);
    });
    wait_until(cx, "the saved approval request", |_| {
        kanso_sync::keychain::load_locked_pending(a.secrets.as_ref(), &library_id)
            .unwrap()
            .is_some_and(|pending| pending.approval_resume().is_some())
    });

    let pending_item = kanso_sync::keychain::pending_item(&library_id);
    let original_raw = a.secrets.get(&pending_item).unwrap().unwrap();
    let request = kanso_sync::keychain::load_locked_pending(a.secrets.as_ref(), &library_id)
        .unwrap()
        .unwrap();
    let expired_at = request.approval_resume().unwrap().expires_at();
    server.set_now(expired_at.saturating_add(1));
    wait_until(cx, "the approval request to expire", |cx| {
        cx.read(|cx| {
            cx.global::<Services>()
                .approval
                .waiting
                .as_ref()
                .is_some_and(|waiting| waiting.phase == crate::device_approval::WaitPhase::Expired)
        })
    });

    let after_expiry = kanso_sync::keychain::load_locked_pending(a.secrets.as_ref(), &library_id)
        .unwrap()
        .expect("expiry keeps the provider session");
    assert!(after_expiry.approval_resume().is_none());
    assert!(matches!(account(cx), AccountState::SignedInLocked { .. }));
    assert!(cx.read(|cx| cx.global::<Services>().session().is_none()));
    let after_expiry_raw = a.secrets.get(&pending_item).unwrap().unwrap();
    let original_json: serde_json::Value = serde_json::from_str(&original_raw).unwrap();
    let expiry_json: serde_json::Value = serde_json::from_str(&after_expiry_raw).unwrap();
    assert_eq!(original_json["token"], expiry_json["token"]);

    cx.update(|cx| account_flow::restart_device_approval(Some(a.window), cx));
    wait_until(cx, "the replacement approval request", |_| {
        kanso_sync::keychain::load_locked_pending(a.secrets.as_ref(), &library_id)
            .unwrap()
            .is_some_and(|pending| {
                pending
                    .approval_resume()
                    .is_some_and(|resume| resume.expires_at() > expired_at)
            })
    });
    let resumed = kanso_sync::keychain::load_locked_pending(a.secrets.as_ref(), &library_id)
        .unwrap()
        .unwrap();
    let request_id = resumed.approval_resume().unwrap().request_id().to_owned();
    let listing = kanso_sync::pending_approvals(server.as_ref(), &source).unwrap();
    let listed = listing
        .iter()
        .find(|approval| approval.id == request_id)
        .expect("the replacement request is visible to the existing device");
    // Only the device that challenged may decline, so the existing device
    // answers first, as its sheet does on seeing the request.
    kanso_sync::challenge_request(
        server.as_ref(),
        &source,
        listed,
        &kanso_sync::ApprovalKeys::generate(),
    )
    .unwrap();
    kanso_sync::decline_request(server.as_ref(), &source, &listed.id).unwrap();
    wait_until(cx, "the replacement request to be cancelled", |cx| {
        cx.read(|cx| {
            cx.global::<Services>()
                .approval
                .waiting
                .as_ref()
                .is_some_and(|waiting| {
                    waiting.phase == crate::device_approval::WaitPhase::Cancelled
                })
        })
    });
    let after_cancel = kanso_sync::keychain::load_locked_pending(a.secrets.as_ref(), &library_id)
        .unwrap()
        .expect("remote cancellation keeps the provider session");
    assert!(after_cancel.approval_resume().is_none());
    let after_cancel_raw = a.secrets.get(&pending_item).unwrap().unwrap();
    let cancel_json: serde_json::Value = serde_json::from_str(&after_cancel_raw).unwrap();
    assert_eq!(original_json["token"], cancel_json["token"]);
    assert_eq!(a.bound_to(), None);
    assert!(cx.read(|cx| cx.global::<AppState>().config.sync.account.is_none()));
    assert!(cx.read(|cx| cx.global::<Services>().sync == SyncState::Off));

    cx.update(account_flow::cancel_locked_session);
    wait_until(cx, "locked sign-out after approval cancellation", |cx| {
        cx.read(|cx| {
            cx.global::<Services>().task.is_none()
                && cx.global::<Services>().account == AccountState::SignedOut
        })
    });
    assert!(
        kanso_sync::keychain::load_locked_pending(a.secrets.as_ref(), &library_id)
            .unwrap()
            .is_none()
    );
}

#[gpui_kit::test]
fn approval_with_no_other_devices_cancels_request_and_keeps_pending_token(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let source = password_account(&server, "alice");
    kanso_sync::sign_out(server.as_ref(), &MemorySecrets::new(), &source).unwrap();
    let a = mac(cx, &server, "Studio", 0);
    let (login_device, provider_session) = provider_signin(&server, &source.account_id);
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    let library_id = device.library_id.clone();
    let locked = account_flow::locked_provider_session(
        SERVER.to_owned(),
        login_device,
        provider_session,
        &library_id,
    )
    .unwrap();
    enter_locked(cx, locked, device, a.window);
    let pending_item = kanso_sync::keychain::pending_item(&library_id);
    let before: serde_json::Value =
        serde_json::from_str(&a.secrets.get(&pending_item).unwrap().unwrap()).unwrap();
    let token = before["token"].as_str().unwrap().to_owned();
    let locked = cx.read(|cx| cx.global::<Services>().locked_session.clone().unwrap());
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    let started = Instant::now();

    cx.update(|cx| {
        account_flow::wait_for_device_approval((*locked).clone(), device, Some(a.window), cx)
    });
    wait_until(cx, "the no-devices approval result", |cx| {
        cx.read(|cx| {
            cx.global::<Services>()
                .approval
                .waiting
                .as_ref()
                .is_some_and(|waiting| {
                    waiting.phase == crate::device_approval::WaitPhase::NoDevices
                })
        })
    });

    assert!(started.elapsed() < Duration::from_secs(10));
    assert!(matches!(account(cx), AccountState::SignedInLocked { .. }));
    let pending = kanso_sync::keychain::load_locked_pending(a.secrets.as_ref(), &library_id)
        .unwrap()
        .expect("no-devices result preserves the locked session");
    assert!(pending.approval_resume().is_none());
    let after: serde_json::Value =
        serde_json::from_str(&a.secrets.get(&pending_item).unwrap().unwrap()).unwrap();
    assert_eq!(after["token"].as_str(), Some(token.as_str()));
}

#[gpui_kit::test]
fn cancelling_locked_binding_consent_preserves_the_previous_account_and_notes(
    cx: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(ok(&synced(cx)));
    let library_id = library_id(cx);
    let account_item = kanso_sync::keychain::account_item(&library_id);
    let recovery_item = kanso_sync::keychain::recovery_item(&library_id);
    let previous_account_item = a.secrets.get(&account_item).unwrap().unwrap();
    let previous_recovery_item = a.secrets.get(&recovery_item).unwrap().unwrap();
    let previous_config = cx.read(|cx| cx.global::<AppState>().config.sync.account.clone());
    let previous_binding = a.bound_to();
    let previous_notes = a.notes();

    reset_runtime_without_signing_out(cx, &server, &a);
    let other = password_account(&server, "bob");
    let (login_device, provider_session) = provider_signin(&server, &other.account_id);
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    let locked = account_flow::locked_provider_session(
        SERVER.to_owned(),
        login_device,
        provider_session,
        &device.library_id,
    )
    .unwrap();
    enter_locked(cx, locked, device, a.window);
    let pending_item = kanso_sync::keychain::pending_item(&library_id);
    let provider_pending: serde_json::Value =
        serde_json::from_str(&a.secrets.get(&pending_item).unwrap().unwrap()).unwrap();
    let provider_token = provider_pending["token"].as_str().unwrap().to_owned();
    let unlock = cx.update(|cx| {
        account_flow::unlock_locked_password(password(PASSWORD, "bob"), Some(a.window), cx)
    });
    wait_until(cx, "the binding consent", |cx| a.asking(cx));
    let refreshed_pending: serde_json::Value =
        serde_json::from_str(&a.secrets.get(&pending_item).unwrap().unwrap()).unwrap();
    let refreshed_token = refreshed_pending["token"].as_str().unwrap();
    assert_ne!(refreshed_token, provider_token);
    assert!(matches!(account(cx), AccountState::SignedInLocked { .. }));
    assert!(cx.read(|cx| cx.global::<Services>().session().is_none()));
    assert!(cx.read(|cx| cx.global::<AppState>().config.sync.account == previous_config));

    a.answer(cx, "choice-cancel");
    assert_eq!(finish(cx, unlock), Err("Sign-in was cancelled.".to_owned()));
    assert_eq!(account(cx), AccountState::SignedOut);
    assert_eq!(a.bound_to(), previous_binding);
    assert_eq!(a.notes(), previous_notes);
    assert_eq!(
        a.secrets.get(&account_item).unwrap(),
        Some(previous_account_item)
    );
    assert_eq!(
        a.secrets.get(&recovery_item).unwrap(),
        Some(previous_recovery_item)
    );
    assert_eq!(
        cx.read(|cx| cx.global::<AppState>().config.sync.account.clone()),
        previous_config
    );
    assert!(
        kanso_sync::keychain::load_locked_pending(a.secrets.as_ref(), &library_id)
            .unwrap()
            .is_none(),
        "Cancel removes the refreshed pending session"
    );
    assert!(matches!(
        server
            .api(SERVER)
            .devices(&kanso_sync::SessionToken::new(refreshed_token.to_owned())),
        Err(kanso_sync::ApiError::Unauthorized { .. })
    ));
    assert_eq!(
        kanso_sync::devices(server.as_ref(), &other).unwrap().len(),
        1,
        "Cancel logs out the fresh password token, so it cannot be used"
    );
}

#[gpui_kit::test]
fn locked_bind_failure_rolls_back_the_previous_session_and_binding(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(ok(&synced(cx)));
    let library_id = library_id(cx);
    let account_item = kanso_sync::keychain::account_item(&library_id);
    let recovery_item = kanso_sync::keychain::recovery_item(&library_id);
    let previous_account_item = a.secrets.get(&account_item).unwrap().unwrap();
    let previous_recovery_item = a.secrets.get(&recovery_item).unwrap().unwrap();
    let previous_config = cx.read(|cx| cx.global::<AppState>().config.sync.account.clone());
    let previous_binding = a.bound_to();
    let previous_notes = a.notes();

    reset_runtime_without_signing_out(cx, &server, &a);
    let other = password_account(&server, "bob");
    let (login_device, provider_session) = provider_signin(&server, &other.account_id);
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    let locked = account_flow::locked_provider_session(
        SERVER.to_owned(),
        login_device,
        provider_session,
        &device.library_id,
    )
    .unwrap();
    enter_locked(cx, locked, device, a.window);
    let conn = refuse(
        &a,
        &format!(
            "CREATE TRIGGER refuse_locked_bind BEFORE INSERT ON meta WHEN NEW.key = '{}' \
             BEGIN SELECT RAISE(ABORT, 'refused'); END;",
            meta_keys::ACCOUNT_ID
        ),
    );
    let unlock = cx.update(|cx| {
        account_flow::unlock_locked_password(password(PASSWORD, "bob"), Some(a.window), cx)
    });
    wait_until(cx, "the binding consent", |cx| a.asking(cx));
    a.answer(cx, "choice-0");
    let result = finish(cx, unlock);
    conn.execute_batch("DROP TRIGGER refuse_locked_bind")
        .unwrap();

    assert!(result.is_err());
    assert_eq!(account(cx), AccountState::SignedOut);
    assert_eq!(a.bound_to(), previous_binding);
    assert_eq!(a.notes(), previous_notes);
    assert_eq!(
        a.secrets.get(&account_item).unwrap(),
        Some(previous_account_item)
    );
    assert_eq!(
        a.secrets.get(&recovery_item).unwrap(),
        Some(previous_recovery_item)
    );
    assert_eq!(
        cx.read(|cx| cx.global::<AppState>().config.sync.account.clone()),
        previous_config
    );
    assert!(
        kanso_sync::keychain::load_locked_pending(a.secrets.as_ref(), &library_id)
            .unwrap()
            .is_none(),
        "failed binding clears only the new pending sign-in"
    );
    assert_eq!(
        kanso_sync::devices(server.as_ref(), &other).unwrap().len(),
        1,
        "failed binding logs out both candidate tokens"
    );
}

#[gpui_kit::test]
fn pending_delete_failure_restores_the_previous_item_and_keeps_the_unlocked_candidate_retryable(
    cx: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(ok(&synced(cx)));
    let library_id = library_id(cx);
    let account_item = kanso_sync::keychain::account_item(&library_id);
    let previous_account_item = a.secrets.get(&account_item).unwrap().unwrap();
    let previous_config = cx.read(|cx| cx.global::<AppState>().config.sync.account.clone());
    let previous_binding = a.bound_to();
    let previous_notes = a.notes();

    reset_runtime_without_signing_out(cx, &server, &a);
    let other = password_account(&server, "bob");
    let (login_device, provider_session) = provider_signin(&server, &other.account_id);
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    let locked = account_flow::locked_provider_session(
        SERVER.to_owned(),
        login_device,
        provider_session,
        &device.library_id,
    )
    .unwrap();
    enter_locked(cx, locked, device, a.window);

    let unlock = cx.update(|cx| {
        account_flow::unlock_locked_password(password(PASSWORD, "bob"), Some(a.window), cx)
    });
    wait_until(cx, "the binding consent", |cx| a.asking(cx));
    let pending_item = kanso_sync::keychain::pending_item(&library_id);
    let pending: serde_json::Value =
        serde_json::from_str(&a.secrets.get(&pending_item).unwrap().unwrap()).unwrap();
    let refreshed_token = pending["token"].as_str().unwrap().to_owned();

    // The fresh token is durable before consent. Make only the later pending
    // delete fail, so promotion must restore the previous full item instead of
    // leaving a half-promoted account.
    a.secrets.fail_writes_to(Some("sync-pending:"));
    a.answer(cx, "choice-0");
    let result = finish(cx, unlock);
    a.secrets.fail_writes_to(None);

    assert!(result.is_err(), "{result:?}");
    assert!(matches!(account(cx), AccountState::SignedInLocked { .. }));
    assert_eq!(a.bound_to(), previous_binding);
    assert_eq!(a.notes(), previous_notes);
    assert_eq!(
        a.secrets.get(&account_item).unwrap(),
        Some(previous_account_item),
        "the previous full session item is restored"
    );
    assert_eq!(
        cx.read(|cx| cx.global::<AppState>().config.sync.account.clone()),
        previous_config
    );
    assert!(cx.read(|cx| cx.global::<Services>().session().is_none()));
    kanso_sync::keychain::load_locked_pending(a.secrets.as_ref(), &library_id)
        .unwrap()
        .expect("the refreshed unlocked candidate remains retryable");
    let kept: serde_json::Value =
        serde_json::from_str(&a.secrets.get(&pending_item).unwrap().unwrap()).unwrap();
    assert_eq!(kept["token"].as_str(), Some(refreshed_token.as_str()));
    assert!(
        server
            .api(SERVER)
            .devices(&kanso_sync::SessionToken::new(refreshed_token.clone()))
            .is_ok(),
        "the token kept in the pending item was not revoked"
    );
}

#[gpui_kit::test]
fn recovery_pending_write_failure_revokes_the_fresh_token_and_clears_the_dead_locked_session(
    cx: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let elsewhere = DeviceIdentity::from_parts(&format!("l_{:032x}", 8), "Elsewhere", [8; 32]);
    let bob = kanso_sync::sign_up(
        server.as_ref(),
        &MemorySecrets::new(),
        SERVER,
        "bob",
        &password(PASSWORD, "bob"),
        &elsewhere,
        &|_| {},
    )
    .unwrap();
    let a = mac(cx, &server, "Studio", 0);
    let (login_device, provider_session) = provider_signin(&server, &bob.session.account_id);
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    let locked = account_flow::locked_provider_session(
        SERVER.to_owned(),
        login_device,
        provider_session,
        &device.library_id,
    )
    .unwrap();
    enter_locked(cx, locked, device, a.window);
    let library_id = library_id(cx);
    let pending_item = kanso_sync::keychain::pending_item(&library_id);
    let failing = Arc::new(FailPendingSetOnce {
        inner: a.secrets.clone(),
        armed: AtomicBool::new(true),
    });
    let secrets: Arc<dyn SecretStore> = failing;
    cx.update(|cx| cx.global_mut::<Services>().secrets = Some(secrets));
    let recover = cx.update(|cx| {
        account_flow::recover_locked(
            bob.recovery_key.clone(),
            password(NEW_PASSWORD, "bob"),
            Some(a.window),
            cx,
        )
    });
    let result = finish(cx, recover);

    assert!(result.is_err(), "{result:?}");
    assert_eq!(account(cx), AccountState::SignedOut);
    assert!(cx.read(|cx| cx.global::<Services>().session().is_none()));
    assert!(cx.read(|cx| cx.global::<Services>().locked_session.is_none()));
    assert!(
        a.secrets.get(&pending_item).unwrap().is_none(),
        "the dead provider token is not left for a relaunch to restore"
    );
    let shown = pending_key(cx).expect("the rotated recovery key is shown");
    assert!(shown != bob.recovery_key);
    assert!(holds(&server, "bob", &shown));
}

#[gpui_kit::test]
fn an_agreeing_full_session_wins_over_a_pending_provider_session(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    let existing = session(cx);
    let library_id = library_id(cx);

    let (login_device, provider_session) = provider_signin(&server, &existing.account_id);
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    let locked = account_flow::locked_provider_session(
        SERVER.to_owned(),
        login_device,
        provider_session,
        &device.library_id,
    )
    .unwrap();
    kanso_sync::keychain::save_locked_pending(a.secrets.as_ref(), &library_id, &locked).unwrap();

    relaunched(cx, &server, &a);
    cx.update(account_flow::restore);
    restored(cx);

    assert_eq!(account(cx), signed_in_as("alice"));
    assert!(cx.read(|cx| cx.global::<Services>().session().is_some()));
    assert!(cx.read(|cx| cx.global::<Services>().locked_session.is_none()));
}

#[gpui_kit::test]
fn failed_pending_keychain_save_revokes_provider_token_and_stays_signed_out(
    cx: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let source = password_account(&server, "alice");
    let a = mac(cx, &server, "Studio", 0);
    let (login_device, provider_session) = provider_signin(&server, &source.account_id);
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    let locked = account_flow::locked_provider_session(
        SERVER.to_owned(),
        login_device,
        provider_session,
        &device.library_id,
    )
    .unwrap();
    a.secrets.fail_writes_to(Some("sync-pending:"));

    assert!(
        account_flow::save_provider_pending(server.as_ref(), a.secrets.as_ref(), &locked,).is_err()
    );
    a.secrets.heal();

    assert_eq!(account(cx), AccountState::SignedOut);
    assert!(cx.read(|cx| cx.global::<Services>().session().is_none()));
    assert!(cx.read(|cx| cx.global::<Services>().locked_session.is_none()));
    assert!(cx.read(|cx| cx.global::<AppState>().config.sync.account.is_none()));
    assert_eq!(
        kanso_sync::devices(server.as_ref(), &source).unwrap().len(),
        1,
        "the token that could not be saved was revoked"
    );
}

#[gpui_kit::test]
fn password_unlock_uses_provider_proof_and_waits_for_binding_consent(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let source = password_account(&server, "alice");
    let a = mac(cx, &server, "Studio", 1);
    let (login_device, provider_session) = provider_signin(&server, &source.account_id);
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    let library_device_id = device.device_id(&source.account_id);
    assert_ne!(login_device.id, library_device_id);
    let locked = account_flow::locked_provider_session(
        SERVER.to_owned(),
        login_device.clone(),
        provider_session,
        &device.library_id,
    )
    .unwrap();
    enter_locked(cx, locked, device, a.window);
    assert!(matches!(account(cx), AccountState::SignedInLocked { .. }));

    let wrong = cx.update(|cx| {
        account_flow::unlock_locked_password(
            password("not the password", "alice"),
            Some(a.window),
            cx,
        )
    });
    assert!(finish(cx, wrong).is_err());
    assert!(matches!(account(cx), AccountState::SignedInLocked { .. }));
    assert!(cx.read(|cx| cx.global::<Services>().session().is_none()));

    let right = cx.update(|cx| {
        account_flow::unlock_locked_password(password(PASSWORD, "alice"), Some(a.window), cx)
    });
    wait_until(cx, "the binding consent", |cx| a.asking(cx));
    assert!(matches!(account(cx), AccountState::SignedInLocked { .. }));
    assert!(cx.read(|cx| cx.global::<Services>().session().is_none()));
    assert!(cx.read(|cx| cx.global::<AppState>().config.sync.account.is_none()));

    // Add is the first choice. No full session, binding or sync is promoted
    // until this actual UI answer is selected.
    a.answer(cx, "choice-0");
    assert_eq!(finish(cx, right), Ok(()));
    assert_eq!(account(cx), signed_in_as("alice"));
    assert_eq!(session(cx).device_id, login_device.id);
    assert_ne!(session(cx).device_id, library_device_id);
}

#[gpui_kit::test]
fn local_key_reuse_reaches_consent_without_promoting_or_syncing(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let source = password_account(&server, "alice");
    let a = mac(cx, &server, "Studio", 1);
    let (login_device, provider_session) = provider_signin(&server, &source.account_id);
    let machine = cx.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let device = kanso_sync::device_identity(&a.store, &machine).unwrap();
    kanso_sync::keychain::save_session(a.secrets.as_ref(), &device.library_id, &source).unwrap();
    let devices_before = kanso_sync::devices(server.as_ref(), &source).unwrap();
    let locked = account_flow::locked_provider_session(
        SERVER.to_owned(),
        login_device,
        provider_session,
        &device.library_id,
    )
    .unwrap();

    enter_locked(cx, locked, device, a.window);

    assert!(
        a.asking(cx),
        "a local-key unlock reaches the binding question"
    );
    assert!(matches!(account(cx), AccountState::SignedInLocked { .. }));
    assert!(cx.read(|cx| cx.global::<Services>().session().is_none()));
    assert!(cx.read(|cx| cx.global::<Services>().sync == SyncState::Off));
    assert!(cx.read(|cx| cx.global::<AppState>().config.sync.account.is_none()));
    assert_eq!(
        kanso_sync::devices(server.as_ref(), &source).unwrap(),
        devices_before,
        "offline key reuse does not touch or add a server device"
    );
}

/// The forms' server; `MemoryServer` answers for any.
const SERVER: &str = kanso_sync::DEFAULT_SERVER;
const PASSWORD: &str = "correct horse battery staple";
const NEW_PASSWORD: &str = "a different long passphrase";

/// The shared server, resending a request whose answer was lost at once.
struct Quick(Arc<MemoryServer>);

impl Backend for Quick {
    fn api(&self, server: &str) -> Box<dyn Api + '_> {
        self.0.api(server)
    }
    fn kdf_floor(&self) -> KdfParams {
        self.0.kdf_floor()
    }
    fn kdf_default(&self) -> KdfParams {
        self.0.kdf_default()
    }
    fn retry_pause(&self, _: usize) -> Duration {
        Duration::ZERO
    }
}

/// One Mac: its library, its Keychain and its window.
struct Mac {
    dir: tempfile::TempDir,
    store: Arc<Store>,
    secrets: Arc<MemorySecrets>,
    window: AnyWindowHandle,
    workspace: gpui_kit::Entity<Workspace>,
}

/// A Mac called `name` with `notes` notes, using `server`.
fn mac(cx: &mut TestAppContext, server: &Arc<MemoryServer>, name: &str, notes: usize) -> Mac {
    // The account tasks run on threads of their own and wake the app from
    // there, as real I/O would.
    cx.executor().allow_parking();
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    if notes > 0 {
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Work"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        for n in 0..notes {
            store
                .put(
                    DocType::Note,
                    json!({"title": format!("{name} note {n}"), "body": "text", "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
    }
    let secrets = Arc::new(MemorySecrets::new());
    let backend: Arc<dyn Backend> = Arc::new(Quick(server.clone()));
    let (app_store, data_dir, keychain) =
        (store.clone(), dir.path().to_path_buf(), secrets.clone());
    let machine = Machine {
        hardware_uuid: Some(format!("UUID-{name}")),
        computer_name: name.to_owned(),
    };
    let (window, workspace) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store: app_store,
            data_dir,
            config: AppConfig::default(),
        });
        cx.set_global(services(backend, keychain, machine));
        crate::workspace::bind_keys(cx);
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    Mac {
        dir,
        store,
        secrets,
        window,
        workspace,
    }
}

/// Services as `main.rs` builds them, with the test seams.
fn services(
    backend: Arc<dyn Backend>,
    secrets: Arc<dyn SecretStore>,
    machine: Machine,
) -> Services {
    let mut services = Services::default();
    services.backend = Some(backend);
    services.secrets = Some(secrets);
    services.machine = Some(machine);
    services
}

fn password(text: &str, username: &str) -> Password {
    Password::new(text, username).unwrap()
}

fn password_account(server: &MemoryServer, username: &str) -> Session {
    let secrets = MemorySecrets::new();
    let device =
        DeviceIdentity::from_parts("l_00000000000000000000000000000001", "Other Mac", [1; 32]);
    kanso_sync::sign_up(
        server,
        &secrets,
        SERVER,
        username,
        &password(PASSWORD, username),
        &device,
        &|_| {},
    )
    .unwrap()
    .session
}

fn provider_signin(
    server: &MemoryServer,
    account_id: &str,
) -> (LoginDevice, kanso_sync::ProviderSession) {
    let login_device = LoginDevice::ephemeral();
    server.set_providers(vec![Provider::Apple]);
    server.set_provider_outcomes(Provider::Apple, [format!("session:{account_id}")]);
    let started =
        kanso_sync::start_provider(server, SERVER, &Provider::Apple, &login_device, None, None)
            .unwrap();
    let ProviderStep::Session(session) = kanso_sync::poll_provider(
        server,
        SERVER,
        &Provider::Apple,
        started.flow_id(),
        None,
        &login_device,
    )
    .unwrap() else {
        panic!("the scripted provider sign-in returns a session")
    };
    (login_device, session)
}

/// Runs the app until `rx` says the task is done.
fn finish(cx: &mut TestAppContext, rx: Receiver<Progress>) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        cx.run_until_parked();
        while let Ok(progress) = rx.try_recv() {
            if let Progress::Done(result) = progress {
                cx.run_until_parked();
                return result;
            }
        }
        assert!(Instant::now() < deadline, "the account task finished");
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Runs the app until `done` holds; the tasks run on their own threads.
fn wait_until(cx: &mut TestAppContext, what: &str, done: impl Fn(&mut TestAppContext) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        cx.run_until_parked();
        if done(cx) {
            return;
        }
        assert!(Instant::now() < deadline, "waited for {what}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Waits for the sync that is running (or about to start) to finish, and
/// returns its state.
fn synced(cx: &mut TestAppContext) -> SyncState {
    wait_until(cx, "the sync to finish", |cx| {
        cx.read(|cx| {
            let services = cx.global::<Services>();
            matches!(
                services.sync,
                SyncState::Synced { .. } | SyncState::Failed(_)
            ) && services.task.is_none()
        })
    });
    cx.read(|cx| cx.global::<Services>().sync.clone())
}

/// Syncs now and waits for it.
fn sync_now(cx: &mut TestAppContext) -> SyncState {
    cx.update(|cx| {
        // The last sync's state goes, so the wait sees this one's.
        cx.global_mut::<Services>().sync = SyncState::Syncing;
        services::sync_now(cx);
    });
    synced(cx)
}

fn ok(state: &SyncState) -> bool {
    matches!(state, SyncState::Synced { problem: None, .. })
}

fn account(cx: &mut TestAppContext) -> AccountState {
    cx.read(|cx| cx.global::<Services>().account.clone())
}

fn session(cx: &mut TestAppContext) -> Arc<Session> {
    cx.read(|cx| cx.global::<Services>().session())
        .expect("signed in")
}

/// The key screen's words over the key, if any.
fn key_note(cx: &mut TestAppContext) -> Option<&'static str> {
    cx.read(|cx| {
        cx.global::<Services>()
            .pending_key
            .as_ref()
            .and_then(|pending| pending.note)
    })
}

/// Waits for the one ask about a shown key Kanso couldn't tell the server
/// took (review-verify 6).
fn key_checked(cx: &mut TestAppContext) {
    wait_until(cx, "the key check", |cx| {
        cx.read(|cx| {
            let services = cx.global::<Services>();
            !services.key_check_due && services.task.is_none()
        })
    });
}

/// Whether `key` is the account's recovery key on `server` now.
fn holds(server: &MemoryServer, username: &str, key: &RecoveryKey) -> bool {
    holds_at(server, SERVER, username, key)
}

/// [`holds`] for an account on the server at `url`.
fn holds_at(server: &MemoryServer, url: &str, username: &str, key: &RecoveryKey) -> bool {
    let origin = kanso_sync::origin(url).unwrap();
    let auth = kanso_sync::crypto::recovery_keys(key, &origin, username).auth_key;
    server
        .api(url)
        .recover_start(&kanso_sync::api::RecoverStartRequest {
            username: username.into(),
            recovery_auth_key: auth.to_base64url(),
        })
        .is_ok()
}

/// "I've saved my recovery key", and its Keychain item's removal.
fn key_saved(cx: &mut TestAppContext) {
    cx.update(account_flow::key_saved);
    cx.run_until_parked();
}

fn pending_key(cx: &mut TestAppContext) -> Option<RecoveryKey> {
    cx.read(|cx| {
        cx.global::<Services>()
            .pending_key
            .as_ref()
            .map(|pending| pending.key.clone())
    })
}

fn library_id(cx: &mut TestAppContext) -> String {
    cx.read(|cx| cx.global::<Services>().library_id.clone())
        .expect("the library's ID is known")
}

fn signed_in_as(username: &str) -> AccountState {
    AccountState::SignedIn {
        username: username.to_owned(),
    }
}

impl Mac {
    fn notes(&self) -> usize {
        self.store
            .counts()
            .unwrap()
            .get(&DocType::Note)
            .copied()
            .unwrap_or(0)
    }

    fn bound_to(&self) -> Option<String> {
        self.store.meta(meta_keys::ACCOUNT_ID).unwrap()
    }

    fn items(&self) -> Vec<String> {
        self.secrets.items()
    }

    fn draw(&self, cx: &mut TestAppContext) {
        self.window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
    }

    /// The library's unsaved recovery key, as the Keychain holds it.
    fn kept_key(&self) -> Option<kanso_sync::keychain::PendingRecoveryKey> {
        let library = self
            .items()
            .iter()
            .find_map(|item| item.strip_prefix("sync-recovery-key:").map(str::to_owned))?;
        kanso_sync::keychain::load_any_recovery_key(self.secrets.as_ref(), &library).unwrap()
    }

    /// The first note's ID.
    fn first_note(&self) -> String {
        let conn = rusqlite::Connection::open(self.dir.path().join("kanso.db")).unwrap();
        conn.query_row(
            "SELECT id FROM docs WHERE id LIKE 'note:%' ORDER BY seq LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap()
    }

    /// Opens `id` in the editor and types `body` over it, not yet saved.
    fn type_into(&self, cx: &mut TestAppContext, id: &str, body: &str) {
        let library = cx.read(|cx| self.workspace.read(cx).library.clone());
        let id = id.to_owned();
        library.update(cx, |library, cx| library.select(Some(id), cx));
        cx.run_until_parked();
        let editor = cx.read(|cx| self.workspace.read(cx).editor.clone());
        self.window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| editor.edit_for_test(body, window, cx))
            })
            .unwrap();
    }

    fn notifications(&self, cx: &mut TestAppContext) -> usize {
        self.window
            .update(cx, |_, window, cx| window.notifications(cx).len())
            .unwrap()
    }

    /// Whether the window shows Settings › Account.
    fn showing_account(&self, cx: &mut TestAppContext) -> bool {
        cx.update(|cx| {
            let mut showing = false;
            crate::workspace::for_each_workspace(cx, |workspace, cx| {
                showing |= workspace.settings_open()
                    && workspace.settings.read(cx).page() == SettingsPage::Account;
            });
            showing
        })
    }

    /// Clicks what `selector` names in the window.
    fn click(&self, cx: &mut TestAppContext, selector: &'static str) {
        self.draw(cx);
        let mut vcx = VisualTestContext::from_window(self.window, cx);
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        vcx.simulate_click(bounds.center(), Modifiers::none());
        cx.run_until_parked();
    }

    fn asking(&self, cx: &mut TestAppContext) -> bool {
        self.window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap()
    }

    /// Waits for the binding question, then clicks `selector` in it.
    fn answer(&self, cx: &mut TestAppContext, selector: &'static str) {
        wait_until(cx, "the binding question", |cx| self.asking(cx));
        self.draw(cx);
        let mut vcx = VisualTestContext::from_window(self.window, cx);
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        vcx.simulate_click(bounds.center(), Modifiers::none());
        cx.run_until_parked();
    }

    fn sign_up(&self, cx: &mut TestAppContext, username: &str) -> Receiver<Progress> {
        let (window, username) = (self.window, username.to_owned());
        cx.update(|cx| {
            let password = password(PASSWORD, &username);
            account_flow::sign_up(SERVER.into(), username, password, window, cx)
        })
    }

    fn sign_in(&self, cx: &mut TestAppContext, username: &str, text: &str) -> Receiver<Progress> {
        let (window, username) = (self.window, username.to_owned());
        let password = password(text, &username);
        cx.update(|cx| account_flow::sign_in(SERVER.into(), username, password, window, cx))
    }

    fn recover(
        &self,
        cx: &mut TestAppContext,
        username: &str,
        key: RecoveryKey,
        text: &str,
    ) -> Receiver<Progress> {
        let (window, username) = (self.window, username.to_owned());
        let password = password(text, &username);
        cx.update(|cx| {
            account_flow::recover(SERVER.into(), username, key, password, Some(window), cx)
        })
    }
}

fn sign_out(cx: &mut TestAppContext) -> Result<(), String> {
    let rx = cx.update(account_flow::sign_out);
    finish(cx, rx)
}

/// Waits for the Account page's storage and devices after signing in.
fn details_loaded(cx: &mut TestAppContext) {
    wait_until(cx, "the account's details", |cx| {
        cx.read(|cx| {
            let details = &cx.global::<Services>().details;
            !details.loading && (details.devices.is_some() || details.failed.is_some())
        })
    });
}

/// Signing up creates the account, binds this library to it with its notes,
/// keeps the session and the new recovery key in the Keychain, and shows the
/// key until "I've saved my recovery key", which removes its item.
#[gpui_kit::test]
fn signing_up_binds_this_library_and_shows_the_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 3);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));

    assert_eq!(account(cx), signed_in_as("alice"));
    let session = session(cx);
    assert_eq!(a.bound_to(), Some(session.account_id.clone()));
    assert_eq!(
        a.store.meta(meta_keys::USERNAME).unwrap().as_deref(),
        Some("alice")
    );
    assert_eq!(a.notes(), 3, "the notes stay, to be uploaded");
    let library = library_id(cx);
    assert_eq!(
        a.items(),
        [
            format!("sync-account:{library}"),
            format!("sync-recovery-key:{library}")
        ]
    );
    let config = cx.read(|cx| cx.global::<AppState>().config.sync.clone());
    let saved = config.account.expect("the config names the account");
    assert_eq!(
        (saved.username.as_str(), saved.id.as_str()),
        ("alice", session.account_id.as_str())
    );
    assert!(config.enabled, "signing up turns sync on");
    assert!(pending_key(cx).is_some(), "the key screen shows");

    key_saved(cx);
    cx.run_until_parked();
    assert!(pending_key(cx).is_none());
    assert_eq!(a.items(), [format!("sync-account:{library}")]);

    details_loaded(cx);
    let devices = cx.read(|cx| cx.global::<Services>().details.devices.clone());
    let devices = devices.expect("the devices load");
    assert_eq!(devices.len(), 1);
    assert!(devices[0].current);

    // The first sync uploads the notes and their book, encrypted.
    let state = synced(cx);
    assert!(ok(&state), "{state:?}");
    let view = server.plain_view(&session);
    assert_eq!(view.docs().len(), 4);
    let local: Vec<String> = a
        .store
        .changes_since(0, None, false)
        .unwrap()
        .into_iter()
        .map(|doc| doc.id)
        .collect();
    assert!(
        view.wire_docs().iter().all(|doc| !local.contains(&doc.id)),
        "the server never sees an ID"
    );
}

/// Waits for the Account page's details with no fetch running or due.
fn details_settled(cx: &mut TestAppContext) -> String {
    wait_until(cx, "the account's details to settle", |cx| {
        cx.read(|cx| {
            let details = &cx.global::<Services>().details;
            !details.loading && !details.stale && details.usage.is_some()
        })
    });
    cx.read(|cx| crate::account_page::storage_detail(&cx.global::<Services>().details))
}

/// The Account page's storage line follows the syncs that change what the
/// server holds: after the first upload it counts the uploaded notes, not
/// the empty account the sign-up fetched. A sync that moves nothing asks the
/// server nothing more (each account has 120 requests a minute).
#[gpui_kit::test]
fn the_storage_line_follows_the_uploads(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 3);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);

    let state = synced(cx);
    assert!(ok(&state), "{state:?}");
    let line = details_settled(cx);
    assert!(line.starts_with("4 items · "), "{line}");

    let asked = server.stats().count(Route::Account);
    let state = sync_now(cx);
    assert!(ok(&state), "{state:?}");
    let line = details_settled(cx);
    assert!(line.starts_with("4 items · "), "{line}");
    assert_eq!(
        server.stats().count(Route::Account),
        asked,
        "an idle sync asks nothing"
    );

    let book = a
        .store
        .changes_since(0, None, false)
        .unwrap()
        .into_iter()
        .find(|doc| doc.id.starts_with("book:"))
        .unwrap();
    a.store
        .put(
            DocType::Note,
            json!({"title": "One more", "body": "text", "bookId": book.id}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    let state = sync_now(cx);
    assert!(ok(&state), "{state:?}");
    let line = details_settled(cx);
    assert!(line.starts_with("5 items · "), "{line}");
    assert_eq!(server.stats().count(Route::Account), asked + 1);
}

/// A second Mac with notes of its own signs in: it is asked about them, and
/// adding them binds it to the same account with its notes kept. An empty
/// Mac signs in without a question.
#[gpui_kit::test]
fn a_second_mac_is_asked_about_its_notes(
    cx: &mut TestAppContext,
    cx_b: &mut TestAppContext,
    cx_c: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let account_id = a.bound_to().unwrap();
    assert!(ok(&synced(cx)));

    let b = mac(cx_b, &server, "Laptop", 4);
    let rx = b.sign_in(cx_b, "alice", PASSWORD);
    b.answer(cx_b, "choice-0");
    assert_eq!(finish(cx_b, rx), Ok(()));
    assert_eq!(account(cx_b), signed_in_as("alice"));
    assert_eq!(b.bound_to(), Some(account_id.clone()));
    assert!(pending_key(cx_b).is_none(), "signing in makes no new key");
    let config = cx_b.read(|cx| cx.global::<AppState>().config.sync.clone());
    assert!(config.enabled, "signing in turns sync on");
    details_loaded(cx_b);
    let devices = cx_b.read(|cx| cx.global::<Services>().details.devices.clone());
    assert_eq!(devices.map(|d| d.len()), Some(2), "both Macs are devices");
    // Add keeps this Mac's notes; its first sync brings alice's and sends
    // its own.
    assert!(ok(&synced(cx_b)));
    assert_eq!(b.notes(), 6);
    assert!(ok(&sync_now(cx)));
    assert_eq!(a.notes(), 6, "the first Mac gets the second's notes");

    let c = mac(cx_c, &server, "Mini", 0);
    let rx = c.sign_in(cx_c, "alice", PASSWORD);
    assert_eq!(finish(cx_c, rx), Ok(()));
    assert!(!c.asking(cx_c), "an empty library needs no question");
    assert_eq!(c.bound_to(), Some(account_id));
    assert!(ok(&synced(cx_c)));
    assert_eq!(c.notes(), 6);
}

/// Replace saves a backup of the library first, then empties it for the
/// account's notes.
#[gpui_kit::test]
fn replacing_the_notes_keeps_a_backup_first(cx: &mut TestAppContext, cx_b: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(ok(&synced(cx)));

    let b = mac(cx_b, &server, "Laptop", 3);
    let rx = b.sign_in(cx_b, "alice", PASSWORD);
    b.answer(cx_b, "choice-1");
    assert_eq!(finish(cx_b, rx), Ok(()));
    assert_eq!(account(cx_b), signed_in_as("alice"));
    assert_eq!(b.bound_to(), a.bound_to());
    let backups: Vec<_> = std::fs::read_dir(crate::paths::backups_dir(b.dir.path()))
        .expect("a backups folder")
        .collect();
    assert_eq!(backups.len(), 1, "one backup, made first");
    assert!(ok(&synced(cx_b)));
    assert_eq!(b.notes(), 1, "the account's notes, and only those");
    assert!(ok(&sync_now(cx)));
    assert_eq!(a.notes(), 1, "the replaced notes never reach the account");
}

/// An edit not saved yet when Replace is chosen is saved first, so the
/// backup has it; the Replace then empties the library with it (A10).
#[gpui_kit::test]
fn an_edit_typed_before_a_replace_is_in_its_backup(
    cx: &mut TestAppContext,
    cx_b: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(ok(&synced(cx)));

    let b = mac(cx_b, &server, "Laptop", 2);
    let id = b.first_note();
    let rx = b.sign_in(cx_b, "alice", PASSWORD);
    wait_until(cx_b, "the binding question", |cx| b.asking(cx));
    b.type_into(cx_b, &id, "typed just before Replace");
    assert_eq!(
        b.store.note(&id).unwrap().unwrap().body,
        "text",
        "not saved yet"
    );
    b.answer(cx_b, "choice-1");
    assert_eq!(finish(cx_b, rx), Ok(()));

    let backups: Vec<_> = std::fs::read_dir(crate::paths::backups_dir(b.dir.path()))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(backups.len(), 1);
    let backup = Store::open(&backups[0]).unwrap();
    assert_eq!(
        backup.note(&id).unwrap().map(|note| note.body),
        Some("typed just before Replace".into()),
        "the backup has the edit"
    );
    assert!(
        b.store.note(&id).unwrap().is_none(),
        "the library was replaced"
    );
    let writes_held = cx_b.read(|cx| cx.global::<Services>().writes_held);
    assert!(!writes_held, "saves go on after the Replace");
}

/// Another Mac changes the password, which replaces the recovery key: this
/// Mac's unsaved key from sign-up no longer works, so it goes when this
/// Mac hears its session ended (sec-client 3). An unconfirmed key stays
/// (`a_lost_password_change_still_shows_the_key`).
#[gpui_kit::test]
fn a_key_replaced_on_another_mac_is_dropped(cx: &mut TestAppContext, cx_b: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let unsaved = pending_key(cx).expect("the sign-up key shows");
    assert!(a.kept_key().is_some_and(|kept| !kept.unconfirmed));

    let b = mac(cx_b, &server, "Laptop", 0);
    let rx = b.sign_in(cx_b, "alice", PASSWORD);
    assert_eq!(finish(cx_b, rx), Ok(()));
    let rx = cx_b.update(|cx| {
        account_flow::change_password(
            password(PASSWORD, "alice"),
            password(NEW_PASSWORD, "alice"),
            cx,
        )
    });
    assert_eq!(finish(cx_b, rx), Ok(()));
    assert!(!holds(&server, "alice", &unsaved), "replaced");

    cx.update(account_flow::refresh_details);
    wait_until(cx, "this Mac to hear", |cx| {
        matches!(account(cx), AccountState::SessionEnded { .. })
    });
    wait_until(cx, "the stale key to go", |cx| pending_key(cx).is_none());
    assert!(a.kept_key().is_none(), "and its Keychain item");
}

/// A Keychain whose next save of a session item, once saved, waits: it
/// says so on the first channel, then waits for the second.
struct Gated {
    inner: Arc<MemorySecrets>,
    gate: std::sync::Mutex<Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>>,
}

/// Fails one `sync-pending` set but lets cleanup deletes through, as a
/// transient Keychain write failure would.
struct FailPendingSetOnce {
    inner: Arc<MemorySecrets>,
    armed: AtomicBool,
}

impl SecretStore for FailPendingSetOnce {
    fn get(&self, item: &str) -> Result<Option<zeroize::Zeroizing<String>>, SecretError> {
        self.inner.get(item)
    }

    fn set(&self, item: &str, value: &str) -> Result<(), SecretError> {
        if item.starts_with("sync-pending:") && self.armed.swap(false, Ordering::AcqRel) {
            return Err(SecretError("the Keychain refused once (simulated)".into()));
        }
        self.inner.set(item, value)
    }

    fn delete(&self, item: &str) -> Result<(), SecretError> {
        self.inner.delete(item)
    }
}

impl SecretStore for Gated {
    fn get(&self, item: &str) -> Result<Option<zeroize::Zeroizing<String>>, SecretError> {
        self.inner.get(item)
    }
    fn set(&self, item: &str, value: &str) -> Result<(), SecretError> {
        self.inner.set(item, value)?;
        if item.starts_with("sync-account:") {
            let gate = self.gate.lock().unwrap().take();
            if let Some((entered, release)) = gate {
                entered.send(()).unwrap();
                release.recv().unwrap();
            }
        }
        Ok(())
    }
    fn delete(&self, item: &str) -> Result<(), SecretError> {
        self.inner.delete(item)
    }
}

/// Review-verify 3: "Forgot your password?" while signed in ends this Mac's
/// own session too. The Account page's details, asked for on that session,
/// get a 401 after the recovery took the new key and saved the new
/// session's item, before it signed in again. That 401 ends the old
/// session, but never drops the recovery's new key, and never removes the
/// new session's item: a relaunch signs in with it.
#[gpui_kit::test]
fn a_401_from_this_macs_own_recovery_keeps_its_new_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let first = pending_key(cx).unwrap();
    key_saved(cx);
    details_loaded(cx);
    let old = session(cx);
    let library = library_id(cx);
    let (entered_tx, entered) = mpsc::channel();
    let (release, release_rx) = mpsc::channel();
    let gated: Arc<dyn SecretStore> = Arc::new(Gated {
        inner: a.secrets.clone(),
        gate: std::sync::Mutex::new(Some((entered_tx, release_rx))),
    });
    cx.update(|cx| cx.global_mut::<Services>().secrets = Some(gated));

    let rx = cx.update(|cx| {
        account_flow::recover_signed_in(first.clone(), password(NEW_PASSWORD, "alice"), cx)
    });
    wait_until(cx, "the recovery to save its session", |_| {
        entered.try_recv().is_ok()
    });
    let new_key = pending_key(cx).expect("the new key shows");
    assert!(new_key != first);
    assert!(
        a.kept_key()
            .is_some_and(|kept| kept.recovery_key == new_key && !kept.unconfirmed),
        "the server took it"
    );
    // Not signed in again yet. A background sync on the old session may
    // already have had its 401 and ended it; that is the same 401 the
    // refresh below asks for, just sooner.
    let current = cx.read(|cx| cx.global::<Services>().session());
    assert!(
        current.is_none_or(|current| Arc::ptr_eq(&current, &old)),
        "not signed in again yet"
    );

    cx.update(account_flow::refresh_details);
    wait_until(cx, "the old session's 401", |cx| {
        matches!(account(cx), AccountState::SessionEnded { .. })
    });
    for _ in 0..10 {
        cx.run_until_parked();
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(pending_key(cx), Some(new_key.clone()), "still on screen");
    assert!(
        a.kept_key()
            .is_some_and(|kept| kept.recovery_key == new_key),
        "still in the Keychain"
    );
    assert!(
        a.items().contains(&format!("sync-account:{library}")),
        "the new session's item stays"
    );

    release.send(()).unwrap();
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), signed_in_as("alice"));
    assert_eq!(pending_key(cx), Some(new_key.clone()));
    assert!(
        a.kept_key()
            .is_some_and(|kept| kept.recovery_key == new_key)
    );
    assert!(holds(&server, "alice", &new_key));
    let now = session(cx);
    let saved = kanso_sync::keychain::load_session(
        a.secrets.as_ref(),
        &library,
        &now.origin,
        &now.account_id,
    )
    .unwrap()
    .expect("a relaunch finds a session");
    assert!(
        kanso_sync::usage(server.as_ref(), &saved).is_ok(),
        "the new one, which the server accepts"
    );
}

/// Signing in to another account on a Mac bound to the first asks about
/// that account's notes; Cancel signs the new session out and changes
/// nothing here.
#[gpui_kit::test]
fn another_accounts_notes_are_asked_about(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let elsewhere = DeviceIdentity::from_parts(&format!("l_{:032x}", 9), "Elsewhere", [9; 32]);
    let bob = kanso_sync::sign_up(
        server.as_ref(),
        &MemorySecrets::new(),
        SERVER,
        "bob",
        &password(PASSWORD, "bob"),
        &elsewhere,
        &|_| {},
    )
    .unwrap();

    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    let alice = a.bound_to();
    assert_eq!(sign_out(cx), Ok(()));

    let rx = a.sign_in(cx, "bob", PASSWORD);
    a.answer(cx, "choice-cancel");
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), AccountState::SignedOut);
    assert_eq!(a.bound_to(), alice, "still alice's library");
    assert_eq!(a.notes(), 2);
    assert!(a.items().is_empty(), "no session is kept");
    let devices = kanso_sync::devices(server.as_ref(), &bob.session).unwrap();
    assert_eq!(devices.len(), 1, "the cancelled session is gone");
}

/// Cancelling the question about another account while signed in leaves this
/// one as it was: its Keychain item, its session on the server, and its
/// syncing, which starts again.
#[gpui_kit::test]
fn cancelling_another_account_keeps_this_one(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let elsewhere = DeviceIdentity::from_parts(&format!("l_{:032x}", 9), "Elsewhere", [9; 32]);
    let bob = kanso_sync::sign_up(
        server.as_ref(),
        &MemorySecrets::new(),
        SERVER,
        "bob",
        &password(PASSWORD, "bob"),
        &elsewhere,
        &|_| {},
    )
    .unwrap();
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(ok(&synced(cx)));
    let alice = session(cx);
    let library = library_id(cx);
    let items = a.items();

    // Signing in stops the running sync first.
    let window = a.window;
    let rx = cx.update(|cx| {
        services::sync_now(cx);
        account_flow::sign_in(
            SERVER.into(),
            "bob".into(),
            password(PASSWORD, "bob"),
            window,
            cx,
        )
    });
    a.answer(cx, "choice-cancel");
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), signed_in_as("alice"));
    assert!(Arc::ptr_eq(&session(cx), &alice));
    assert_eq!(a.items(), items, "alice's items stay");
    let kept = kanso_sync::keychain::load_session(
        a.secrets.as_ref(),
        &library,
        &alice.origin,
        &alice.account_id,
    )
    .unwrap();
    assert!(kept.is_some(), "a relaunch still finds alice");
    assert!(kanso_sync::usage(server.as_ref(), &alice).is_ok());
    let devices = kanso_sync::devices(server.as_ref(), &bob.session).unwrap();
    assert_eq!(devices.len(), 1, "bob's cancelled session is gone");
    assert!(ok(&synced(cx)), "alice syncs again");
}

/// A recovery whose binding question is cancelled keeps the new recovery
/// key on screen and in the Keychain: the used one no longer works.
#[gpui_kit::test]
fn a_cancelled_recovery_keeps_the_new_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let elsewhere = DeviceIdentity::from_parts(&format!("l_{:032x}", 9), "Elsewhere", [9; 32]);
    let bob = kanso_sync::sign_up(
        server.as_ref(),
        &MemorySecrets::new(),
        SERVER,
        "bob",
        &password(PASSWORD, "bob"),
        &elsewhere,
        &|_| {},
    )
    .unwrap();
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    assert_eq!(sign_out(cx), Ok(()));
    let library = library_id(cx);

    let rx = a.recover(cx, "bob", bob.recovery_key.clone(), NEW_PASSWORD);
    a.answer(cx, "choice-cancel");
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), AccountState::SignedOut);
    let shown = pending_key(cx).expect("the new key shows");
    assert!(shown != bob.recovery_key);
    assert!(holds(&server, "bob", &shown));
    assert_eq!(a.items(), [format!("sync-recovery-key:{library}")]);
    let devices = kanso_sync::devices(server.as_ref(), &bob.session);
    assert!(devices.is_err(), "recovery signed bob's other session out");

    // A relaunch, signed out, shows it again.
    let backend: Arc<dyn Backend> = Arc::new(Quick(server.clone()));
    let secrets: Arc<dyn SecretStore> = a.secrets.clone();
    cx.update(|cx| {
        let machine = cx.global::<Services>().machine.clone().unwrap();
        cx.set_global(services(backend, secrets, machine));
        account_flow::restore(cx);
    });
    wait_until(cx, "the launch check", |cx| {
        cx.read(|cx| cx.global::<Services>().library_id.is_some())
    });
    assert_eq!(account(cx), AccountState::SignedOut);
    assert_eq!(pending_key(cx), Some(shown));
    assert_eq!(key_note(cx), None, "the server took it");
}

/// Signing out while a sync runs stops it first ("Stopping sync…"), and its
/// result changes nothing afterwards.
#[gpui_kit::test]
fn signing_out_during_a_sync_stops_it_first(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 40);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(ok(&synced(cx)));
    let old = session(cx);

    let rx = cx.update(|cx| {
        services::sync_now(cx);
        account_flow::sign_out(cx)
    });
    assert_eq!(
        rx.try_recv().ok(),
        Some(Progress::Step(kanso_sync::Step::StoppingSync))
    );
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), AccountState::SignedOut);
    for _ in 0..20 {
        cx.run_until_parked();
        std::thread::sleep(Duration::from_millis(2));
    }
    let state = cx.read(|cx| cx.global::<Services>().sync.clone());
    assert_eq!(state, SyncState::Off, "the stopped sync's news is dropped");
    assert!(a.items().is_empty());
    assert!(matches!(
        kanso_sync::usage(server.as_ref(), &old),
        Err(AccountError::Unauthorized { .. })
    ));
}

/// Signing out ends the session on the server, forgets it here, and keeps
/// the notes and the binding: signing in again resumes without a question.
#[gpui_kit::test]
fn signing_out_keeps_the_notes(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let old = session(cx);
    let bound = a.bound_to();

    assert_eq!(sign_out(cx), Ok(()));
    assert_eq!(account(cx), AccountState::SignedOut);
    assert!(pending_key(cx).is_none());
    assert!(a.items().is_empty(), "the session and key items go");
    let config = cx.read(|cx| cx.global::<AppState>().config.sync.account.clone());
    assert!(config.is_none());
    assert_eq!(a.notes(), 2);
    assert_eq!(a.bound_to(), bound);
    assert!(matches!(
        kanso_sync::usage(server.as_ref(), &old),
        Err(AccountError::Unauthorized { .. })
    ));

    let rx = a.sign_in(cx, "alice", PASSWORD);
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(!a.asking(cx), "the same account resumes");
    assert_eq!(account(cx), signed_in_as("alice"));
    assert_eq!(a.bound_to(), bound);
}

/// Changing the password keeps this Mac signed in and signs the others out
/// with the reason; they sign in again with the new password only.
#[gpui_kit::test]
fn changing_the_password_signs_the_other_macs_out(
    cx: &mut TestAppContext,
    cx_b: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let b = mac(cx_b, &server, "Laptop", 0);
    let rx = b.sign_in(cx_b, "alice", PASSWORD);
    assert_eq!(finish(cx_b, rx), Ok(()));
    details_loaded(cx_b);

    let rx = cx.update(|cx| {
        account_flow::change_password(
            password("not the password at all", "alice"),
            password(NEW_PASSWORD, "alice"),
            cx,
        )
    });
    assert_eq!(
        finish(cx, rx),
        Err("That isn't your current password.".to_owned())
    );
    let first = pending_key(cx).unwrap();
    key_saved(cx);
    let rx = cx.update(|cx| {
        account_flow::change_password(
            password(PASSWORD, "alice"),
            password(NEW_PASSWORD, "alice"),
            cx,
        )
    });
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), signed_in_as("alice"));
    // The recovery key changed with the password.
    let second = pending_key(cx).expect("the new key shows");
    assert_eq!(key_note(cx), Some(words::KEY_AFTER_PASSWORD));
    assert!(holds(&server, "alice", &second));
    assert!(
        !holds(&server, "alice", &first),
        "the saved key stops working"
    );

    cx_b.update(account_flow::refresh_details);
    wait_until(cx_b, "the other Mac to hear", |cx| {
        matches!(account(cx), AccountState::SessionEnded { .. })
    });
    assert_eq!(
        account(cx_b),
        AccountState::SessionEnded {
            username: "alice".into(),
            reason: SignOutReason::PasswordChanged,
        }
    );
    assert!(
        b.items().is_empty(),
        "the ended session leaves the Keychain"
    );

    let rx = b.sign_in(cx_b, "alice", PASSWORD);
    assert_eq!(
        finish(cx_b, rx),
        Err("That username and password don't match.".to_owned())
    );
    let rx = b.sign_in(cx_b, "alice", NEW_PASSWORD);
    assert_eq!(finish(cx_b, rx), Ok(()));
    assert_eq!(account(cx_b), signed_in_as("alice"));
}

/// The recovery key sets a new password and signs this Mac in; Kanso shows
/// a new key, and the used one stops working. A new recovery key replaces
/// the current one the same way.
#[gpui_kit::test]
fn recovery_replaces_the_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let first = pending_key(cx).unwrap();
    key_saved(cx);
    assert_eq!(sign_out(cx), Ok(()));

    let rx = a.recover(cx, "alice", first.clone(), NEW_PASSWORD);
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), signed_in_as("alice"));
    let second = pending_key(cx).expect("a new key shows");
    assert!(second != first);
    let library = library_id(cx);
    assert!(a.items().contains(&format!("sync-recovery-key:{library}")));
    key_saved(cx);

    let rx = cx.update(|cx| account_flow::new_recovery_key(password(NEW_PASSWORD, "alice"), cx));
    assert_eq!(finish(cx, rx), Ok(()));
    let third = pending_key(cx).expect("the new key shows");
    assert!(third != second);
    key_saved(cx);

    assert_eq!(sign_out(cx), Ok(()));
    for used in [first, second] {
        let rx = a.recover(cx, "alice", used, "yet another long passphrase");
        assert_eq!(
            finish(cx, rx),
            Err("That recovery key doesn't match this account.".to_owned())
        );
    }
    let rx = a.sign_in(cx, "alice", PASSWORD);
    assert_eq!(
        finish(cx, rx),
        Err("That username and password don't match.".to_owned())
    );
    let rx = a.recover(cx, "alice", third, "yet another long passphrase");
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), signed_in_as("alice"));
}

/// Deleting the account needs its name and password, unbinds the library
/// and keeps the notes; the account can't be signed in to afterwards.
#[gpui_kit::test]
fn deleting_the_account_keeps_the_notes(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));

    let rx = cx
        .update(|cx| account_flow::delete_account(password(PASSWORD, "alice"), "alice".into(), cx));
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), AccountState::SignedOut);
    assert_eq!(a.bound_to(), None);
    assert_eq!(a.notes(), 2);
    assert!(a.items().is_empty());
    let config = cx.read(|cx| cx.global::<AppState>().config.sync.account.clone());
    assert!(config.is_none());

    let rx = a.sign_in(cx, "alice", PASSWORD);
    assert_eq!(
        finish(cx, rx),
        Err("That username and password don't match.".to_owned())
    );
}

/// At launch the session comes back when the config, the binding and the
/// Keychain agree, with the key screen if the key wasn't saved yet; without
/// the Keychain item, the Mac starts signed out and the config forgets the
/// account.
#[gpui_kit::test]
fn relaunching_restores_the_session(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let first = session(cx);
    let library = library_id(cx);

    let relaunch = |cx: &mut TestAppContext| {
        let backend: Arc<dyn Backend> = Arc::new(Quick(server.clone()));
        let secrets: Arc<dyn SecretStore> = a.secrets.clone();
        cx.update(|cx| {
            let machine = cx.global::<Services>().machine.clone().unwrap();
            cx.set_global(services(backend, secrets, machine));
            account_flow::restore(cx);
        });
        wait_until(cx, "the launch check", |cx| {
            cx.read(|cx| cx.global::<Services>().library_id.is_some())
        });
    };

    relaunch(cx);
    assert_eq!(account(cx), signed_in_as("alice"));
    assert_eq!(session(cx).account_id, first.account_id);
    assert!(pending_key(cx).is_some(), "the unsaved key shows again");

    a.secrets
        .delete(&format!("sync-account:{library}"))
        .unwrap();
    relaunch(cx);
    assert_eq!(account(cx), AccountState::SignedOut);
    let config = cx.read(|cx| cx.global::<AppState>().config.sync.account.clone());
    assert!(config.is_none());
    assert_eq!(a.notes(), 1);
}

// MARK: New recovery keys are never lost

/// Sign-up's answer is lost three times: the account may exist, so its key
/// shows anyway, stays in the Keychain, and is the key the server holds.
/// The server is asked once whether it holds it, and then the words say the
/// account was created and the item is confirmed. The Account page isn't
/// showing, so a notification's "Show key" leads there.
#[gpui_kit::test]
fn a_lost_sign_up_answer_still_shows_the_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    // No notes: signing in later asks nothing.
    let a = mac(cx, &server, "Studio", 0);
    let before = a.notifications(cx);
    server.drop_next_responses(Route::Signup, 3);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Err(words::CANT_REACH.to_owned()));
    assert_eq!(account(cx), AccountState::SignedOut);
    let shown = pending_key(cx).expect("the key shows");
    key_checked(cx);
    assert_eq!(key_note(cx), Some(words::KEY_TAKEN_SIGN_UP));
    assert_eq!(server.stats().count(Route::RecoverStart), 1, "asked once");
    let kept = a.kept_key().expect("the Keychain keeps it");
    assert!(!kept.unconfirmed, "confirmed once the server said so");
    assert_eq!(kept.recovery_key, shown);
    assert!(holds(&server, "alice", &shown), "the account was created");

    assert_eq!(a.notifications(cx), before + 1, "a notification says so");
    assert!(!a.showing_account(cx));
    a.click(cx, "show-recovery-key");
    assert!(a.showing_account(cx), "Show key opens the Account page");

    // On the Account page, the next key needs no notification.
    key_saved(cx);
    let rx = a.sign_in(cx, "alice", PASSWORD);
    assert_eq!(finish(cx, rx), Ok(()));
    a.window
        .update(cx, |_, window, cx| window.clear_notifications(cx))
        .unwrap();
    cx.executor().advance_clock(Duration::from_secs(2));
    cx.run_until_parked();
    assert_eq!(a.notifications(cx), 0);
    let rx = cx.update(|cx| account_flow::new_recovery_key(password(PASSWORD, "alice"), cx));
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(pending_key(cx).is_some());
    assert!(a.showing_account(cx));
    assert_eq!(a.notifications(cx), 0);
}

/// The account is created but this Mac can't keep its session: the key
/// still shows and stays in the Keychain, and the words say the account
/// exists.
#[gpui_kit::test]
fn a_sign_up_that_cant_finish_still_shows_the_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    a.secrets.fail_writes_to(Some("sync-account:"));
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Err(words::CREATED_NOT_SIGNED_IN.to_owned()));
    a.secrets.heal();
    assert_eq!(account(cx), AccountState::SignedOut);
    let shown = pending_key(cx).expect("the key shows");
    assert_eq!(key_note(cx), Some(words::CREATED_NOT_SIGNED_IN));
    assert_eq!(
        a.kept_key().map(|kept| kept.recovery_key),
        Some(shown.clone())
    );
    assert!(holds(&server, "alice", &shown));
    assert_eq!(a.bound_to(), None, "never bound without its Keychain item");
    let rx = a.sign_in(cx, "alice", PASSWORD);
    a.answer(cx, "choice-0");
    assert_eq!(finish(cx, rx), Ok(()), "the account is there to sign in to");
}

/// A recovery that changed the password but couldn't finish signing in says
/// so, and its new key stays on screen and in the Keychain.
#[gpui_kit::test]
fn a_recovery_that_cant_finish_says_the_password_changed(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let first = pending_key(cx).unwrap();
    key_saved(cx);
    assert_eq!(sign_out(cx), Ok(()));

    a.secrets.fail_writes_to(Some("sync-account:"));
    let rx = a.recover(cx, "alice", first.clone(), NEW_PASSWORD);
    assert_eq!(finish(cx, rx), Err(words::CHANGED_NOT_SIGNED_IN.to_owned()));
    a.secrets.heal();
    assert_eq!(account(cx), AccountState::SignedOut);
    let second = pending_key(cx).expect("the new key shows");
    assert!(second != first);
    assert_eq!(key_note(cx), Some(words::CHANGED_NOT_SIGNED_IN));
    assert_eq!(
        a.kept_key().map(|kept| kept.recovery_key),
        Some(second.clone())
    );
    assert!(holds(&server, "alice", &second));
    let rx = a.sign_in(cx, "alice", NEW_PASSWORD);
    assert_eq!(finish(cx, rx), Ok(()), "the new password works");
}

/// "New recovery key" whose answer is lost shows the key, and once the
/// server says it holds it, says it took; a Keychain that refuses the new
/// key sends nothing.
#[gpui_kit::test]
fn a_new_key_is_kept_before_it_is_sent(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let first = pending_key(cx).unwrap();
    key_saved(cx);

    a.secrets.fail_writes_to(Some("sync-recovery-key:"));
    let rx = cx.update(|cx| account_flow::new_recovery_key(password(PASSWORD, "alice"), cx));
    let refused = finish(cx, rx);
    a.secrets.heal();
    assert_eq!(
        refused,
        Err(words::error_words(
            &AccountError::Keychain(kanso_sync::SecretError(String::new())),
            words::ErrorContext::CurrentPassword
        ))
    );
    assert_eq!(
        server.stats().count(Route::RecoveryKey),
        0,
        "nothing was sent"
    );
    assert!(pending_key(cx).is_none());
    assert!(holds(&server, "alice", &first));

    server.drop_next_responses(Route::RecoveryKey, 3);
    let asked = server.stats().count(Route::RecoverStart);
    let rx = cx.update(|cx| account_flow::new_recovery_key(password(PASSWORD, "alice"), cx));
    assert_eq!(finish(cx, rx), Err(words::CANT_REACH.to_owned()));
    let shown = pending_key(cx).expect("the key shows");
    key_checked(cx);
    assert_eq!(key_note(cx), Some(words::KEY_TAKEN));
    assert_eq!(
        server.stats().count(Route::RecoverStart),
        asked + 1,
        "asked once"
    );
    assert!(a.kept_key().is_some_and(|kept| !kept.unconfirmed));
    assert!(holds(&server, "alice", &shown), "the server took it");
    assert_eq!(account(cx), signed_in_as("alice"));
}

/// A password change whose answer is lost shows the new key anyway, and
/// once the server says it holds it, says the password changed.
#[gpui_kit::test]
fn a_lost_password_change_still_shows_the_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);

    server.drop_next_responses(Route::Password, 3);
    let rx = cx.update(|cx| {
        account_flow::change_password(
            password(PASSWORD, "alice"),
            password(NEW_PASSWORD, "alice"),
            cx,
        )
    });
    assert_eq!(finish(cx, rx), Err(words::CANT_REACH.to_owned()));
    let shown = pending_key(cx).expect("the key shows");
    key_checked(cx);
    assert_eq!(key_note(cx), Some(words::KEY_TAKEN_PASSWORD));
    assert!(holds(&server, "alice", &shown), "the change went through");
    assert!(a.kept_key().is_some_and(|kept| !kept.unconfirmed));
}

/// A new key whose request timed out before the server carried it out: the
/// server is asked once, says it doesn't hold it, and the key goes (screen
/// and Keychain item) with a notification, in place of "Show key", saying
/// nothing changed. The old key still works (review-verify 6).
#[gpui_kit::test]
fn a_key_the_server_never_took_goes_and_says_nothing_changed(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let first = pending_key(cx).unwrap();
    key_saved(cx);
    a.window
        .update(cx, |_, window, cx| window.clear_notifications(cx))
        .unwrap();
    cx.executor().advance_clock(Duration::from_secs(2));
    cx.run_until_parked();
    assert_eq!(a.notifications(cx), 0);

    server.lose_next_requests(Route::RecoveryKey, 3);
    let rx = cx.update(|cx| account_flow::new_recovery_key(password(PASSWORD, "alice"), cx));
    assert_eq!(finish(cx, rx), Err(words::CANT_REACH.to_owned()));
    key_checked(cx);
    assert_eq!(server.stats().count(Route::RecoverStart), 1, "asked once");
    assert!(
        pending_key(cx).is_none(),
        "the key the server never took goes"
    );
    assert!(a.kept_key().is_none(), "and so does its Keychain item");
    assert_eq!(a.notifications(cx), 1, "one notification says so");
    assert!(holds(&server, "alice", &first), "the old key still works");
    assert_eq!(account(cx), signed_in_as("alice"));

    // A lost recovery's key: the password didn't change either.
    assert_eq!(sign_out(cx), Ok(()));
    server.lose_next_requests(Route::RecoverFinish, 3);
    let rx = a.recover(cx, "alice", first.clone(), NEW_PASSWORD);
    assert_eq!(finish(cx, rx), Err(words::CANT_REACH.to_owned()));
    key_checked(cx);
    assert!(pending_key(cx).is_none());
    assert!(a.kept_key().is_none());
    assert!(holds(&server, "alice", &first));
    let rx = a.sign_in(cx, "alice", PASSWORD);
    assert_eq!(finish(cx, rx), Ok(()), "the old password still works");
}

/// Until the server has answered, the words over the key say to keep the
/// old key too, and the ask isn't repeated by later tasks: the next launch
/// asks again, once, and the item is confirmed (review-verify 6).
#[gpui_kit::test]
fn an_unanswered_key_check_waits_for_the_next_launch(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let first = pending_key(cx).unwrap();
    key_saved(cx);

    server.drop_next_responses(Route::RecoveryKey, 3);
    server.lose_next_requests(Route::RecoverStart, 1);
    let rx = cx.update(|cx| account_flow::new_recovery_key(password(PASSWORD, "alice"), cx));
    assert_eq!(finish(cx, rx), Err(words::CANT_REACH.to_owned()));
    key_checked(cx);
    let shown = pending_key(cx).expect("the key still shows");
    assert_eq!(key_note(cx), Some(words::KEY_UNCONFIRMED));
    assert!(words::KEY_UNCONFIRMED.contains("keep your old recovery key too"));
    assert!(a.kept_key().is_some_and(|kept| kept.unconfirmed));

    // Other tasks don't ask again.
    let rx = cx.update(|cx| account_flow::use_server(SERVER.into(), cx));
    assert_eq!(finish(cx, rx), Ok(()));
    key_checked(cx);
    assert_eq!(
        server.stats().count(Route::RecoverStart),
        0,
        "not asked again"
    );
    assert_eq!(key_note(cx), Some(words::KEY_UNCONFIRMED));

    relaunched(cx, &server, &a);
    cx.update(account_flow::restore);
    restored(cx);
    key_checked(cx);
    assert_eq!(pending_key(cx), Some(shown.clone()));
    assert_eq!(key_note(cx), Some(words::KEY_TAKEN));
    assert_eq!(server.stats().count(Route::RecoverStart), 1, "asked once");
    assert!(a.kept_key().is_some_and(|kept| !kept.unconfirmed));
    assert!(holds(&server, "alice", &shown));
    assert!(!holds(&server, "alice", &first));
}

/// A Keychain that won't delete the session's item still signs this Mac out
/// and ends the session on the server, and says so (A14).
#[gpui_kit::test]
fn signing_out_with_a_stuck_keychain_still_logs_out(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    let old = session(cx);
    let before = a.notifications(cx);

    a.secrets.fail_writes_to(Some("sync-account:"));
    assert_eq!(sign_out(cx), Ok(()));
    a.secrets.heal();
    assert_eq!(account(cx), AccountState::SignedOut);
    assert!(matches!(
        kanso_sync::usage(server.as_ref(), &old),
        Err(AccountError::Unauthorized { .. })
    ));
    assert_eq!(
        a.notifications(cx),
        before + 1,
        "the Keychain trouble is told"
    );
    let config = cx.read(|cx| cx.global::<AppState>().config.sync.account.clone());
    assert!(config.is_none(), "the leftover item is never loaded");
}

// MARK: Saving, then binding

/// Makes the library refuse `statement`'s writes until the trigger is
/// dropped, as a full disk or a damaged file would.
fn refuse(mac: &Mac, trigger: &str) -> rusqlite::Connection {
    let conn = rusqlite::Connection::open(mac.dir.path().join("kanso.db")).unwrap();
    conn.execute_batch(trigger).unwrap();
    conn
}

/// A Replace that fails after its backup leaves this Mac as it was: still
/// signed in to the first account with its Keychain item, its notes and its
/// binding, and syncing again; the backup made first is said all the same
/// (A1, sync 7).
#[gpui_kit::test]
fn a_failed_replace_keeps_this_account_and_says_where_the_backup_is(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let elsewhere = DeviceIdentity::from_parts(&format!("l_{:032x}", 9), "Elsewhere", [9; 32]);
    let bob = kanso_sync::sign_up(
        server.as_ref(),
        &MemorySecrets::new(),
        SERVER,
        "bob",
        &password(PASSWORD, "bob"),
        &elsewhere,
        &|_| {},
    )
    .unwrap();
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    assert!(ok(&synced(cx)));
    let alice = session(cx);
    let library = library_id(cx);
    let item = kanso_sync::keychain::account_item(&library);
    let kept = a.secrets.get(&item).unwrap().expect("alice's item");
    let before = a.notifications(cx);

    let conn = refuse(
        &a,
        "CREATE TRIGGER refuse BEFORE DELETE ON docs BEGIN SELECT RAISE(ABORT, 'refused'); END;",
    );
    let rx = a.sign_in(cx, "bob", PASSWORD);
    a.answer(cx, "choice-1");
    let failed = finish(cx, rx);
    conn.execute_batch("DROP TRIGGER refuse").unwrap();
    assert!(failed.is_err(), "{failed:?}");

    assert_eq!(account(cx), signed_in_as("alice"));
    assert!(Arc::ptr_eq(&session(cx), &alice));
    assert_eq!(
        a.secrets.get(&item).unwrap().map(|now| now.to_string()),
        Some(kept.to_string()),
        "alice's item is back"
    );
    assert_eq!(a.bound_to(), Some(alice.account_id.clone()));
    assert_eq!(a.notes(), 2);
    let backups: Vec<_> = std::fs::read_dir(crate::paths::backups_dir(a.dir.path()))
        .expect("a backups folder")
        .collect();
    assert_eq!(backups.len(), 1, "the backup was made first");
    assert_eq!(a.notifications(cx), before + 1, "and is said");
    let devices = kanso_sync::devices(server.as_ref(), &bob.session).unwrap();
    assert_eq!(devices.len(), 1, "bob's new session is logged out");
    assert!(ok(&sync_now(cx)), "alice syncs");
}

/// A sign-up whose binding fails deletes the session's Keychain item again:
/// a library is never left with an item for an account it isn't bound to.
#[gpui_kit::test]
fn a_sign_up_whose_binding_fails_leaves_no_session_item(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    // The library has its ID; binding writes the account's.
    cx.update(|cx| {
        let machine = cx.global::<Services>().machine.clone().unwrap();
        kanso_sync::device_identity(&a.store, &machine).unwrap();
    });
    let conn = refuse(
        &a,
        &format!(
            "CREATE TRIGGER refuse BEFORE INSERT ON meta WHEN NEW.key = '{}' \
             BEGIN SELECT RAISE(ABORT, 'refused'); END;",
            meta_keys::ACCOUNT_ID
        ),
    );
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Err(words::CREATED_NOT_SIGNED_IN.to_owned()));
    conn.execute_batch("DROP TRIGGER refuse").unwrap();
    assert_eq!(account(cx), AccountState::SignedOut);
    assert_eq!(a.bound_to(), None);
    assert!(
        a.items()
            .iter()
            .all(|item| !item.starts_with("sync-account:")),
        "{:?}",
        a.items()
    );
    assert!(pending_key(cx).is_some(), "the key still shows");
}

// MARK: The launch check

/// Fresh services over the same Keychain and server, as at launch.
fn relaunched(cx: &mut TestAppContext, server: &Arc<MemoryServer>, a: &Mac) {
    let backend: Arc<dyn Backend> = Arc::new(Quick(server.clone()));
    let secrets: Arc<dyn SecretStore> = a.secrets.clone();
    cx.update(|cx| {
        let machine = cx.global::<Services>().machine.clone().unwrap();
        cx.set_global(services(backend, secrets, machine));
    });
}

/// Drops the in-memory signed-in runtime without touching the persisted
/// account, recovery key, config or binding, as when testing a later sign-in
/// against this library's previous account.
fn reset_runtime_without_signing_out(cx: &mut TestAppContext, server: &Arc<MemoryServer>, a: &Mac) {
    cx.update(|cx| {
        crate::device_approval::stop_events(cx);
        if let Some(driver) = cx.global_mut::<Services>().take_driver() {
            driver.stop();
        }
    });
    relaunched(cx, server, a);
}

fn restored(cx: &mut TestAppContext) {
    wait_until(cx, "the launch check", |cx| {
        cx.read(|cx| !cx.global::<Services>().restoring)
    });
}

/// While the launch check reads the Keychain, account tasks wait: they are
/// refused as busy, and work once it is done (A4).
#[gpui_kit::test]
fn account_tasks_wait_for_the_launch_check(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    assert_eq!(sign_out(cx), Ok(()));

    relaunched(cx, &server, &a);
    let (window, pw) = (a.window, password(PASSWORD, "alice"));
    let rx = cx.update(|cx| {
        account_flow::restore(cx);
        account_flow::sign_in(SERVER.into(), "alice".into(), pw, window, cx)
    });
    assert_eq!(
        finish(cx, rx),
        Err("Kanso is still busy with your account. Try again in a moment.".to_owned())
    );
    restored(cx);
    let rx = a.sign_in(cx, "alice", PASSWORD);
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), signed_in_as("alice"));
}

/// The launch check's answer never replaces a session that appeared, or a
/// config account that changed, while it read (A4).
#[gpui_kit::test]
fn the_launch_check_never_overwrites_a_newer_state(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    let newer = session(cx);

    // A session set while the check reads stays.
    relaunched(cx, &server, &a);
    let kept = newer.clone();
    cx.update(|cx| {
        account_flow::restore(cx);
        cx.global_mut::<Services>().set_signed_in(kept);
    });
    restored(cx);
    assert!(Arc::ptr_eq(&session(cx), &newer), "the newer session stays");

    // The config's account forgotten while the check reads: not signed in
    // again, and the config stays as it was made.
    relaunched(cx, &server, &a);
    cx.update(|cx| {
        account_flow::restore(cx);
        AppState::update_config(cx, |config| config.sync.account = None);
    });
    restored(cx);
    assert_eq!(account(cx), AccountState::SignedOut);
    let config = cx.read(|cx| cx.global::<AppState>().config.sync.account.clone());
    assert!(config.is_none());

    // Unchanged, the check signs in as before.
    let saved = crate::config::AccountRef {
        origin: newer.origin.clone(),
        id: newer.account_id.clone(),
        username: newer.username.clone(),
    };
    cx.update(|cx| AppState::update_config(cx, |config| config.sync.account = Some(saved)));
    relaunched(cx, &server, &a);
    cx.update(account_flow::restore);
    restored(cx);
    assert_eq!(account(cx), signed_in_as("alice"));
    assert!(
        !Arc::ptr_eq(&session(cx), &newer),
        "loaded from the Keychain"
    );
}

// MARK: Other devices

/// Signing another Mac out says so and refreshes the list; one the server
/// no longer knows counts as signed out already; a failure says why (A5).
#[gpui_kit::test]
fn signing_another_mac_out_says_how_it_went(cx: &mut TestAppContext, cx_b: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    let b = mac(cx_b, &server, "Laptop", 0);
    let rx = b.sign_in(cx_b, "alice", PASSWORD);
    assert_eq!(finish(cx_b, rx), Ok(()));
    let laptop = session(cx_b);
    let devices = kanso_sync::devices(server.as_ref(), &session(cx)).unwrap();
    let other = devices.iter().find(|d| !d.current).unwrap().id.clone();

    let before = a.notifications(cx);
    let id = other.clone();
    let rx = cx.update(|cx| account_flow::revoke_device(id, "Laptop".into(), cx));
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(a.notifications(cx), before + 1, "Signed out Laptop.");
    assert!(matches!(
        kanso_sync::usage(server.as_ref(), &laptop),
        Err(AccountError::Unauthorized { .. })
    ));
    wait_until(cx, "the list without it", |cx| {
        cx.read(|cx| {
            let details = &cx.global::<Services>().details;
            !details.loading && details.devices.as_ref().is_some_and(|d| d.len() == 1)
        })
    });

    // Again, from a list that hadn't caught up: already done.
    let id = other.clone();
    let rx = cx.update(|cx| account_flow::revoke_device(id, "Laptop".into(), cx));
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(a.notifications(cx), before + 2);

    // An answer that never came says why.
    server.drop_next_responses(Route::RevokeDevice, 1);
    let rx = cx.update(|cx| account_flow::revoke_device(other, "Laptop".into(), cx));
    assert_eq!(finish(cx, rx), Err(words::CANT_REACH.to_owned()));
    assert_eq!(a.notifications(cx), before + 3);
    assert_eq!(account(cx), signed_in_as("alice"));
}

// MARK: New accounts

fn signup_answer(cx: &mut TestAppContext) -> Option<(String, kanso_sync::SignupState)> {
    cx.read(|cx| cx.global::<Services>().signup.clone())
}

/// Each time the Create account form opens, its server is asked again
/// whether it takes new accounts; an answer about a server the form no
/// longer shows is dropped (A6).
#[gpui_kit::test]
fn the_sign_up_answer_is_asked_again_each_time(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let _a = mac(cx, &server, "Studio", 0);
    let asked = |cx: &mut TestAppContext| {
        cx.update(account_flow::check_signup);
        wait_until(cx, "the answer", |cx| {
            cx.read(|cx| cx.global::<Services>().signup_asking.is_none())
        });
    };
    asked(cx);
    assert_eq!(
        signup_answer(cx),
        Some((SERVER.to_owned(), kanso_sync::SignupState::Open))
    );
    server.set_signup(kanso_sync::Signup::Closed);
    asked(cx);
    assert_eq!(
        signup_answer(cx),
        Some((SERVER.to_owned(), kanso_sync::SignupState::Closed)),
        "the newer answer"
    );

    // The form moves to another server before the answer comes.
    server.set_signup(kanso_sync::Signup::Open);
    cx.update(|cx| {
        account_flow::check_signup(cx);
        AppState::update_config(cx, |config| {
            config.sync.server = Some("https://other.example".into())
        });
    });
    wait_until(cx, "the answer", |cx| {
        cx.read(|cx| cx.global::<Services>().signup_asking.is_none())
    });
    assert_eq!(
        signup_answer(cx),
        Some((SERVER.to_owned(), kanso_sync::SignupState::Closed)),
        "an answer about a server no longer shown is dropped"
    );
}

#[gpui_kit::test]
fn server_round_trip_clears_provider_ticket_and_refreshes_health_generation(
    cx: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let _mac = mac(cx, &server, "Studio", 0);
    cx.update(|cx| {
        cx.global_mut::<Services>().provider_signup = Some(services::ProviderSignup {
            server: SERVER.to_owned(),
            provider: kanso_sync::Provider::Google,
            ticket: "ticket-a".into(),
            suggested_username: "alice".into(),
            login_device: kanso_sync::api::LoginDevice::ephemeral(),
        });
        account_flow::check_auth(cx);
    });
    let generation_a = cx.read(|cx| cx.global::<Services>().auth_generation);
    cx.update(|cx| account_flow::set_server(Some("https://server-b.example".into()), cx));
    let generation_b = cx.read(|cx| cx.global::<Services>().auth_generation);
    cx.update(|cx| account_flow::set_server(None, cx));
    cx.read(|cx| {
        let services = cx.global::<Services>();
        assert!(generation_b > generation_a);
        assert!(services.auth_generation > generation_b);
        assert!(services.provider_signup.is_none());
        assert_eq!(account_flow::form_server(cx), SERVER);
        assert!(
            services
                .auth
                .as_ref()
                .is_none_or(|(server, _)| server == SERVER)
        );
    });
}

// MARK: Sync around account tasks

/// A sign-in to another account that stops the sync, then fails in the
/// task it chained without a question (saving bob's session into an empty
/// library), starts alice's stopped sync again at once (A8).
#[gpui_kit::test]
fn a_chained_task_that_fails_starts_the_stopped_sync_again(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let elsewhere = DeviceIdentity::from_parts(&format!("l_{:032x}", 9), "Elsewhere", [9; 32]);
    kanso_sync::sign_up(
        server.as_ref(),
        &MemorySecrets::new(),
        SERVER,
        "bob",
        &password(PASSWORD, "bob"),
        &elsewhere,
        &|_| {},
    )
    .unwrap();
    // No notes: signing in to bob asks nothing and chains the binding.
    let a = mac(cx, &server, "Studio", 0);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    assert!(ok(&synced(cx)));
    let alice = session(cx);

    a.secrets.fail_writes_to(Some("sync-account:"));
    let (window, pw) = (a.window, password(PASSWORD, "bob"));
    let (rx, started) = cx.update(|cx| {
        services::sync_now(cx);
        let started = cx.global::<Services>().sync_starts();
        let rx = account_flow::sign_in(SERVER.into(), "bob".into(), pw, window, cx);
        (rx, started)
    });
    assert!(finish(cx, rx).is_err());
    a.secrets.heal();
    assert!(!a.asking(cx), "no question was asked");
    assert!(Arc::ptr_eq(&session(cx), &alice), "still alice's session");
    assert_eq!(
        cx.read(|cx| cx.global::<Services>().sync_starts()),
        started + 1,
        "the stopped sync started again"
    );
    assert!(ok(&synced(cx)));
}

/// An automatic sync asked for while one runs stays automatic: when the
/// running sync ends on a problem that holds automatic syncs (a disabled
/// account), the queued one waits too, instead of running as if Sync now had
/// been pressed (sync 8). A queued Sync now still runs.
#[gpui_kit::test]
fn a_queued_automatic_sync_respects_the_hold(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    assert!(ok(&synced(cx)));
    let tries = || server.stats().count(Route::Changes) + server.stats().count(Route::Push);

    server.disable("alice");
    let before = tries();
    cx.update(|cx| {
        cx.global_mut::<Services>().sync = SyncState::Syncing;
        services::resume_sync(cx);
        // Asked for while the first runs: queued, automatic.
        services::resume_sync(cx);
    });
    let state = synced(cx);
    for _ in 0..20 {
        cx.run_until_parked();
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(matches!(state, SyncState::Failed(_)), "{state:?}");
    assert_eq!(tries() - before, 1, "the queued sync waits for Sync now");
    assert_eq!(
        cx.read(|cx| cx.global::<Services>().hold()),
        Some(services::Hold::UntilSyncNow)
    );

    // Sync now queued behind an automatic sync runs.
    let before = tries();
    cx.update(|cx| {
        cx.global_mut::<Services>().set_hold(None);
        services::resume_sync(cx);
        services::sync_now(cx);
    });
    synced(cx);
    wait_until(cx, "the queued Sync now", |_| tries() - before == 2);
}

/// A sync asked for while a stale one (of a session since replaced) still
/// runs is run for the new session once the stale one ends (sync 8).
#[gpui_kit::test]
fn a_sync_asked_for_during_a_stale_one_still_runs(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    assert!(ok(&synced(cx)));
    let elsewhere = DeviceIdentity::from_parts(&library_id(cx), "Studio", [7; 32]);
    let newer = kanso_sync::sign_in(
        server.as_ref(),
        SERVER,
        "alice",
        &password(PASSWORD, "alice"),
        &elsewhere,
        &|_| {},
    )
    .unwrap()
    .session;

    let started = cx.update(|cx| {
        services::resume_sync(cx);
        let started = cx.global::<Services>().sync_starts();
        // Signed in again while it runs, and a sync asked for at once.
        cx.global_mut::<Services>().set_signed_in(Arc::new(newer));
        services::resume_sync(cx);
        started
    });
    wait_until(cx, "the queued sync", |cx| {
        cx.read(|cx| cx.global::<Services>().sync_starts()) == started + 1
    });
    assert!(ok(&synced(cx)), "the new session syncs");
}

// MARK: Passkeys and security keys

impl Mac {
    /// The page's passkey ceremony from now on: `key` stands in for the
    /// platform sheet (or a security key).
    fn use_passkey(&self, cx: &mut TestAppContext, key: Arc<kanso_sync::TestAuthenticator>) {
        cx.update(|cx| cx.global_mut::<Services>().test_ceremony = Some(key));
    }

    fn passkey_sign_up(&self, cx: &mut TestAppContext, username: &str) -> Receiver<Progress> {
        let (window, username) = (self.window, username.to_owned());
        cx.update(|cx| {
            account_flow::passkey_sign_up(
                kanso_sync::MEMORY_PASSKEY_SERVER.into(),
                account_flow::PasskeyMethod::Platform { window: 0 },
                username,
                window,
                cx,
            )
        })
    }

    fn passkey_sign_in(&self, cx: &mut TestAppContext) -> Receiver<Progress> {
        let window = self.window;
        cx.update(|cx| {
            account_flow::passkey_sign_in(
                kanso_sync::MEMORY_PASSKEY_SERVER.into(),
                account_flow::PasskeyMethod::Platform { window: 0 },
                None,
                window,
                cx,
            )
        })
    }
}

/// A ceremony that ends one way, as a dismissed or missing sheet does.
struct Ends(kanso_sync::PasskeyCeremonyError);

impl kanso_sync::PasskeyCeremony for Ends {
    fn register(
        &self,
        _: &kanso_sync::PasskeyRegistrationChallenge,
    ) -> Result<kanso_sync::PasskeyRegistration, kanso_sync::PasskeyCeremonyError> {
        Err(self.0)
    }

    fn assert(
        &self,
        _: &kanso_sync::PasskeyAssertionChallenge,
    ) -> Result<kanso_sync::PasskeyAssertion, kanso_sync::PasskeyCeremonyError> {
        Err(self.0)
    }
}

/// Create with a passkey: the account, its recovery key and a signed-in
/// Mac; then another Mac signs in with the same (synced) passkey and its
/// PRF opens the key, so it is signed in without approval.
#[gpui_kit::test]
fn a_passkey_creates_the_account_and_unlocks_a_second_mac(
    cx: &mut TestAppContext,
    cx_b: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let passkey = Arc::new(kanso_sync::TestAuthenticator::new(21));
    let a = mac(cx, &server, "Studio", 1);
    a.use_passkey(cx, passkey.clone());
    let rx = a.passkey_sign_up(cx, "quiet-otter-4821");
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), signed_in_as("quiet-otter-4821"));
    assert!(pending_key(cx).is_some(), "the recovery key shows");
    assert!(a.bound_to().is_some());
    assert!(ok(&synced(cx)));

    let b = mac(cx_b, &server, "Laptop", 0);
    b.use_passkey(cx_b, passkey);
    let rx = b.passkey_sign_in(cx_b);
    assert_eq!(finish(cx_b, rx), Ok(()));
    assert_eq!(account(cx_b), signed_in_as("quiet-otter-4821"));
    assert_eq!(b.bound_to(), a.bound_to());
    assert!(ok(&synced(cx_b)));
    assert_eq!(b.notes(), 1, "the first Mac's note arrives");
}

/// A passkey without PRF signs in, but the second Mac waits locked for
/// approval or the recovery key.
#[gpui_kit::test]
fn a_passkey_without_prf_signs_in_locked(cx: &mut TestAppContext, cx_b: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let passkey = Arc::new(kanso_sync::TestAuthenticator::new(22).without_prf());
    let a = mac(cx, &server, "Studio", 0);
    a.use_passkey(cx, passkey.clone());
    let rx = a.passkey_sign_up(cx, "quiet-otter-4821");
    assert_eq!(finish(cx, rx), Ok(()));

    let b = mac(cx_b, &server, "Laptop", 0);
    b.use_passkey(cx_b, passkey);
    let rx = b.passkey_sign_in(cx_b);
    assert_eq!(finish(cx_b, rx), Ok(()));
    assert_eq!(
        account(cx_b),
        AccountState::SignedInLocked {
            username: "quiet-otter-4821".into()
        }
    );
    assert!(cx_b.read(|cx| cx.global::<Services>().has_locked_session()));
}

/// A dismissed sheet leaves the page as it was; a sheet that can't run
/// says so once and the passkey buttons go.
#[gpui_kit::test]
fn a_dismissed_or_missing_sheet_signs_nothing_in(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    cx.update(|cx| {
        cx.global_mut::<Services>().test_ceremony =
            Some(Arc::new(Ends(kanso_sync::PasskeyCeremonyError::Cancelled)))
    });
    let rx = a.passkey_sign_in(cx);
    assert_eq!(finish(cx, rx), Ok(()));
    let rx = a.passkey_sign_up(cx, "quiet-otter-4821");
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), AccountState::SignedOut);
    assert!(pending_key(cx).is_none());

    cx.update(|cx| {
        cx.global_mut::<Services>().test_ceremony = Some(Arc::new(Ends(
            kanso_sync::PasskeyCeremonyError::Unavailable,
        )))
    });
    let rx = a.passkey_sign_in(cx);
    assert_eq!(finish(cx, rx), Err(words::PASSKEY_UNAVAILABLE.to_owned()));
    assert!(cx.read(|cx| cx.global::<Services>().platform_passkeys_failed));

    cx.update(|cx| {
        cx.global_mut::<Services>().test_ceremony =
            Some(Arc::new(Ends(kanso_sync::PasskeyCeremonyError::Failed)))
    });
    let rx = a.passkey_sign_in(cx);
    assert_eq!(finish(cx, rx), Err(words::PASSKEY_FAILED.to_owned()));
    assert_eq!(account(cx), AccountState::SignedOut);
}

/// A passkey the server doesn't know is refused in a passkey's words.
#[gpui_kit::test]
fn an_unknown_passkey_is_refused(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    a.use_passkey(cx, Arc::new(kanso_sync::TestAuthenticator::new(23)));
    let rx = a.passkey_sign_in(cx);
    assert_eq!(finish(cx, rx), Err(words::PASSKEY_UNKNOWN.to_owned()));
    assert_eq!(account(cx), AccountState::SignedOut);
}

// MARK: Sign-in methods

const PASSKEY_SERVER: &str = kanso_sync::MEMORY_PASSKEY_SERVER;

impl Mac {
    /// A password sign-up on the server that runs passkeys.
    fn sign_up_with_passkeys(&self, cx: &mut TestAppContext, username: &str) {
        let (window, username) = (self.window, username.to_owned());
        let rx = cx.update(|cx| {
            let password = password(PASSWORD, &username);
            account_flow::sign_up(PASSKEY_SERVER.into(), username, password, window, cx)
        });
        assert_eq!(finish(cx, rx), Ok(()));
        key_saved(cx);
    }

    fn change(
        &self,
        cx: &mut TestAppContext,
        change: account_flow::MethodChange,
        proof: account_flow::ProofWith,
        recovery: Option<RecoveryKey>,
    ) -> Result<(), String> {
        let passkey = matches!(
            (&change, &proof),
            (account_flow::MethodChange::AddPasskey, _) | (_, account_flow::ProofWith::Passkey)
        )
        .then_some(account_flow::PasskeyMethod::Platform { window: 0 });
        let rx = cx.update(|cx| {
            account_flow::change_method(
                account_flow::ChangeRequest {
                    change,
                    proof,
                    recovery,
                    passkey,
                },
                cx,
            )
        });
        finish(cx, rx)
    }
}

/// The account's ways in, as the server holds them now.
fn ways_in(server: &MemoryServer, cx: &mut TestAppContext) -> kanso_sync::AccountMethods {
    kanso_sync::methods(server, &session(cx)).unwrap()
}

fn password_proof(username: &str, text: &str) -> account_flow::ProofWith {
    account_flow::ProofWith::Password(password(text, username))
}

/// A passkey ceremony that lets the proof's five minutes pass before it
/// answers.
struct Late {
    key: Arc<kanso_sync::TestAuthenticator>,
    server: Arc<MemoryServer>,
}

impl kanso_sync::PasskeyCeremony for Late {
    fn register(
        &self,
        challenge: &kanso_sync::PasskeyRegistrationChallenge,
    ) -> Result<kanso_sync::PasskeyRegistration, kanso_sync::PasskeyCeremonyError> {
        self.key.register(challenge)
    }

    fn assert(
        &self,
        challenge: &kanso_sync::PasskeyAssertionChallenge,
    ) -> Result<kanso_sync::PasskeyAssertion, kanso_sync::PasskeyCeremonyError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        self.server.set_now(now + 6 * 60 * 1000);
        self.key.assert(challenge)
    }
}

/// [Add a passkey…] with the password as proof, and [Remove] with the new
/// passkey as proof. A wrong password changes nothing and says so.
#[gpui_kit::test]
fn a_password_account_adds_and_removes_a_passkey(cx: &mut TestAppContext) {
    use account_flow::{MethodChange, ProofWith};
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    a.sign_up_with_passkeys(cx, "alice");
    a.use_passkey(cx, Arc::new(kanso_sync::TestAuthenticator::new(31)));

    assert_eq!(
        a.change(
            cx,
            MethodChange::AddPasskey,
            password_proof("alice", "not the password at all"),
            None
        ),
        Err("That isn't your current password.".to_owned())
    );
    assert!(ways_in(&server, cx).passkeys.is_empty());

    assert_eq!(
        a.change(
            cx,
            MethodChange::AddPasskey,
            password_proof("alice", PASSWORD),
            None
        ),
        Ok(())
    );
    let passkeys = ways_in(&server, cx).passkeys;
    assert_eq!(passkeys.len(), 1);
    assert!(passkeys[0].prf, "the PRF wrap went with it: it can unlock");
    wait_until(cx, "the method list", |cx| {
        cx.read(|cx| {
            cx.global::<Services>()
                .account_methods
                .last_successful
                .as_ref()
                .is_some_and(|methods| methods.passkeys.len() == 1)
        })
    });

    let credential_id = passkeys[0].credential_id.clone();
    assert_eq!(
        a.change(
            cx,
            MethodChange::RemovePasskey { credential_id },
            ProofWith::Passkey,
            None
        ),
        Ok(())
    );
    assert!(ways_in(&server, cx).passkeys.is_empty());
    assert_eq!(account(cx), signed_in_as("alice"));
}

/// Removing the only way in is refused without the recovery key, and done
/// with it. A proof whose five minutes ran out says it took too long.
#[gpui_kit::test]
fn the_last_passkey_needs_the_recovery_key(cx: &mut TestAppContext) {
    use account_flow::{MethodChange, ProofWith};
    let server = Arc::new(MemoryServer::new());
    let passkey = Arc::new(kanso_sync::TestAuthenticator::new(32));
    let a = mac(cx, &server, "Studio", 0);
    a.use_passkey(cx, passkey.clone());
    let rx = a.passkey_sign_up(cx, "quiet-otter-4821");
    assert_eq!(finish(cx, rx), Ok(()));
    let key = pending_key(cx).expect("the sign-up's recovery key");
    key_saved(cx);
    let credential_id = ways_in(&server, cx).passkeys[0].credential_id.clone();
    let remove = || MethodChange::RemovePasskey {
        credential_id: credential_id.clone(),
    };

    assert_eq!(
        a.change(cx, remove(), ProofWith::Passkey, None),
        Err(words::LAST_METHOD.to_owned())
    );
    assert_eq!(ways_in(&server, cx).passkeys.len(), 1);

    cx.update(|cx| {
        cx.global_mut::<Services>().test_ceremony = Some(Arc::new(Late {
            key: passkey.clone(),
            server: server.clone(),
        }))
    });
    assert_eq!(
        a.change(cx, remove(), ProofWith::Passkey, Some(key.clone())),
        Err(words::FLOW_EXPIRED.to_owned())
    );
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    server.set_now(now + 7 * 60 * 1000);
    a.use_passkey(cx, passkey);
    assert_eq!(ways_in(&server, cx).passkeys.len(), 1);

    assert_eq!(
        a.change(cx, remove(), ProofWith::Passkey, Some(key)),
        Ok(())
    );
    assert!(ways_in(&server, cx).passkeys.is_empty());
}

/// A passkey account's [Set a password…]: the first password, a new
/// recovery key on the key screen, and the password row's state. The
/// password routes say when the page was out of date.
#[gpui_kit::test]
fn a_passkey_account_sets_its_first_password(cx: &mut TestAppContext) {
    use account_flow::{MethodChange, ProofWith};
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    a.use_passkey(cx, Arc::new(kanso_sync::TestAuthenticator::new(33)));
    let rx = a.passkey_sign_up(cx, "quiet-otter-4821");
    assert_eq!(finish(cx, rx), Ok(()));
    let first_key = pending_key(cx).unwrap();
    key_saved(cx);

    // 409 no_password, in words.
    let rx = cx.update(|cx| {
        account_flow::change_password(
            password("any old guess here", "quiet-otter-4821"),
            password(NEW_PASSWORD, "quiet-otter-4821"),
            cx,
        )
    });
    assert_eq!(
        finish(cx, rx),
        Err(words::PASSWORD_STATE_CHANGED.to_owned())
    );

    assert_eq!(
        a.change(
            cx,
            MethodChange::SetPassword(password(NEW_PASSWORD, "quiet-otter-4821")),
            ProofWith::Passkey,
            None
        ),
        Ok(())
    );
    assert!(ways_in(&server, cx).has_password);
    assert_eq!(key_note(cx), Some(words::KEY_AFTER_PASSWORD));
    let new_key = pending_key(cx).unwrap();
    assert!(holds_at(
        &server,
        PASSKEY_SERVER,
        "quiet-otter-4821",
        &new_key
    ));
    assert!(!holds_at(
        &server,
        PASSKEY_SERVER,
        "quiet-otter-4821",
        &first_key
    ));
    key_saved(cx);

    // 409 already_has_password, in words.
    assert_eq!(
        a.change(
            cx,
            MethodChange::SetPassword(password(NEW_PASSWORD, "quiet-otter-4821")),
            ProofWith::Passkey,
            None
        ),
        Err(words::PASSWORD_STATE_CHANGED.to_owned())
    );
}

/// A passwordless account makes a new recovery key and deletes itself with
/// a passkey proof; a mistyped name deletes nothing.
#[gpui_kit::test]
fn a_passwordless_account_replaces_its_key_and_deletes_with_a_passkey(cx: &mut TestAppContext) {
    use account_flow::{MethodChange, ProofWith};
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    a.use_passkey(cx, Arc::new(kanso_sync::TestAuthenticator::new(34)));
    let rx = a.passkey_sign_up(cx, "quiet-otter-4821");
    assert_eq!(finish(cx, rx), Ok(()));
    let first_key = pending_key(cx).unwrap();
    key_saved(cx);

    assert_eq!(
        a.change(cx, MethodChange::NewRecoveryKey, ProofWith::Passkey, None),
        Ok(())
    );
    let new_key = pending_key(cx).expect("the new key shows");
    assert_ne!(new_key.display(), first_key.display());
    assert!(holds_at(
        &server,
        PASSKEY_SERVER,
        "quiet-otter-4821",
        &new_key
    ));
    key_saved(cx);

    assert_eq!(
        a.change(
            cx,
            MethodChange::DeleteAccount {
                typed: "someone-else".into()
            },
            ProofWith::Passkey,
            None
        ),
        Err("That isn't this account's username.".to_owned())
    );
    assert_eq!(account(cx), signed_in_as("quiet-otter-4821"));

    assert_eq!(
        a.change(
            cx,
            MethodChange::DeleteAccount {
                typed: "quiet-otter-4821".into()
            },
            ProofWith::Passkey,
            None
        ),
        Ok(())
    );
    assert_eq!(account(cx), AccountState::SignedOut);
    assert_eq!(a.bound_to(), None);
    assert_eq!(a.notes(), 1, "the notes on this device stay");
}

/// [Link a provider…] after a password proof, then [Unlink] with a fresh
/// sign-in at that provider as the proof. Nothing opens a browser in tests.
#[gpui_kit::test]
fn linking_and_unlinking_github_take_their_proofs(cx: &mut TestAppContext) {
    use account_flow::{MethodChange, ProofWith};
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    a.sign_up_with_passkeys(cx, "alice");
    let account_id = session(cx).account_id.clone();
    server.set_providers(vec![Provider::Github]);

    server.set_provider_outcomes(Provider::Github, [format!("session:{account_id}")]);
    assert_eq!(
        a.change(
            cx,
            MethodChange::Link(Provider::Github),
            password_proof("alice", PASSWORD),
            None
        ),
        Ok(())
    );
    let linked = ways_in(&server, cx).linked;
    assert_eq!(linked.len(), 1);
    assert_eq!(linked[0].provider, Provider::Github);
    assert!(cx.read(|cx| cx.global::<Services>().provider_wait.is_none()));
    // GitHub's code went to the clipboard, ready to paste on its page.
    let copied = cx.update(|cx| cx.read_from_clipboard().and_then(|item| item.text()));
    assert_eq!(copied.as_deref(), Some("MEMORY-CODE"));

    // A provider can't prove a link: that needs the password or a passkey.
    assert_eq!(
        a.change(
            cx,
            MethodChange::Link(Provider::Github),
            ProofWith::Provider(Provider::Github),
            None
        ),
        Err(words::error_words(
            &AccountError::ReauthRequired,
            words::ErrorContext::Account
        ))
    );

    server.set_provider_outcomes(Provider::Github, [format!("session:{account_id}")]);
    assert_eq!(
        a.change(
            cx,
            MethodChange::Unlink(Provider::Github),
            ProofWith::Provider(Provider::Github),
            None
        ),
        Ok(())
    );
    assert!(ways_in(&server, cx).linked.is_empty());
    assert_eq!(account(cx), signed_in_as("alice"));
}

/// Links GitHub with a passkey proof, then unlinks it with a GitHub proof,
/// from whatever session `mac` holds now.
fn link_and_unlink_github(mac: &Mac, cx: &mut TestAppContext, server: &MemoryServer) {
    use account_flow::{MethodChange, ProofWith};
    let account_id = session(cx).account_id.clone();
    server.set_providers(vec![Provider::Github]);
    server.set_provider_outcomes(Provider::Github, [format!("session:{account_id}")]);
    assert_eq!(
        mac.change(
            cx,
            MethodChange::Link(Provider::Github),
            ProofWith::Passkey,
            None
        ),
        Ok(())
    );
    assert_eq!(ways_in(server, cx).linked.len(), 1);
    server.set_provider_outcomes(Provider::Github, [format!("session:{account_id}")]);
    assert_eq!(
        mac.change(
            cx,
            MethodChange::Unlink(Provider::Github),
            ProofWith::Provider(Provider::Github),
            None
        ),
        Ok(())
    );
    assert!(ways_in(server, cx).linked.is_empty());
}

/// A device that signed in with a synced passkey is bound to that sign-in's
/// own device, not the library's: its link and provider proof starts carry
/// that device's proof, before and after a relaunch.
#[gpui_kit::test]
fn a_passkey_signed_in_device_links_and_proves_with_its_own_device(
    cx: &mut TestAppContext,
    cx_b: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let passkey = Arc::new(kanso_sync::TestAuthenticator::new(41));
    let a = mac(cx, &server, "Studio", 0);
    a.use_passkey(cx, passkey.clone());
    let rx = a.passkey_sign_up(cx, "quiet-otter-4821");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    link_and_unlink_github(&a, cx, &server);

    let b = mac(cx_b, &server, "Laptop", 0);
    b.use_passkey(cx_b, passkey.clone());
    let rx = b.passkey_sign_in(cx_b);
    assert_eq!(finish(cx_b, rx), Ok(()));
    assert_eq!(account(cx_b), signed_in_as("quiet-otter-4821"));
    let machine = cx_b.read(|cx| cx.global::<Services>().machine.clone().unwrap());
    let own = kanso_sync::device_identity(&b.store, &machine)
        .unwrap()
        .device_id(&session(cx_b).account_id);
    assert_ne!(
        session(cx_b).device_id,
        own,
        "a first passkey sign-in uses the flow's device"
    );
    link_and_unlink_github(&b, cx_b, &server);

    // The proof survives a relaunch in the Keychain item.
    relaunched(cx_b, &server, &b);
    cx_b.update(account_flow::restore);
    restored(cx_b);
    assert_eq!(account(cx_b), signed_in_as("quiet-otter-4821"));
    b.use_passkey(cx_b, passkey);
    link_and_unlink_github(&b, cx_b, &server);
}

/// An account made with GitHub is bound to the provider flow's device; its
/// link and provider proof starts carry that device's proof.
#[gpui_kit::test]
fn a_provider_created_session_links_and_proves_with_its_own_device(cx: &mut TestAppContext) {
    use account_flow::{MethodChange, ProofWith};
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    let (login_device, key) = github_account(cx, &server, &a);
    assert_eq!(session(cx).device_id, login_device.id);
    let account_id = session(cx).account_id.clone();

    // A GitHub proof, with the recovery key, makes a new recovery key.
    server.set_provider_outcomes(Provider::Github, [format!("session:{account_id}")]);
    assert_eq!(
        a.change(
            cx,
            MethodChange::NewRecoveryKey,
            ProofWith::Provider(Provider::Github),
            Some(key)
        ),
        Ok(())
    );
    let key = pending_key(cx).expect("the new key");
    key_saved(cx);

    // And it starts an unlink: the server then refuses the last way in
    // without the recovery key, and takes it with the key.
    server.set_provider_outcomes(Provider::Github, [format!("session:{account_id}")]);
    assert_eq!(
        a.change(
            cx,
            MethodChange::Unlink(Provider::Github),
            ProofWith::Provider(Provider::Github),
            None
        ),
        Err(words::LAST_METHOD.to_owned())
    );
    assert_eq!(ways_in(&server, cx).linked.len(), 1);
    server.set_provider_outcomes(Provider::Github, [format!("session:{account_id}")]);
    assert_eq!(
        a.change(
            cx,
            MethodChange::Unlink(Provider::Github),
            ProofWith::Provider(Provider::Github),
            Some(key)
        ),
        Ok(())
    );
    assert!(ways_in(&server, cx).linked.is_empty());
}

/// An account made with GitHub on `a`, signed in: the flow's device and the
/// sign-up's recovery key (saved).
fn github_account(
    cx: &mut TestAppContext,
    server: &Arc<MemoryServer>,
    a: &Mac,
) -> (LoginDevice, RecoveryKey) {
    server.set_providers(vec![Provider::Github]);
    server.set_provider_outcomes(Provider::Github, ["new_account:quiet-otter-4821"]);
    let login_device = LoginDevice::ephemeral();
    let started = kanso_sync::start_provider(
        server.as_ref(),
        PASSKEY_SERVER,
        &Provider::Github,
        &login_device,
        None,
        None,
    )
    .unwrap();
    let ProviderStep::NewAccount(ticket) = kanso_sync::poll_provider(
        server.as_ref(),
        PASSKEY_SERVER,
        &Provider::Github,
        started.flow_id(),
        None,
        &login_device,
    )
    .unwrap() else {
        panic!("the scripted GitHub sign-in is a new account")
    };
    let pending = crate::services::ProviderSignup {
        server: PASSKEY_SERVER.into(),
        provider: Provider::Github,
        ticket: ticket.ticket,
        suggested_username: ticket.suggested_username,
        login_device: login_device.clone(),
    };
    let window = a.window;
    let rx = cx.update(|cx| {
        account_flow::provider_sign_up(pending, "quiet-otter-4821".into(), window, cx)
    });
    assert_eq!(finish(cx, rx), Ok(()));
    let key = pending_key(cx).expect("the sign-up's recovery key");
    key_saved(cx);
    (login_device, key)
}

/// When the page's list was out of date and the server answers 409
/// last_method, the form opens again with the recovery key's field and the
/// reason, instead of a dead end.
#[gpui_kit::test]
fn a_last_method_refusal_asks_again_with_the_recovery_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    let (_, key) = github_account(cx, &server, &a);
    let account_id = session(cx).account_id.clone();
    // Out of date: the page still thinks the account has a password.
    let mut stale = ways_in(&server, cx);
    stale.has_password = true;
    cx.update(|cx| {
        cx.global_mut::<Services>().account_methods = crate::services::AccountMethodsState {
            last_successful: Some(stale),
            ..Default::default()
        }
    });
    let window = a.window;
    cx.update(|cx| {
        window
            .update(cx, |_, window, cx| {
                crate::proof_sheet::open(
                    crate::proof_sheet::Action::Unlink(Provider::Github),
                    window,
                    cx,
                )
            })
            .unwrap()
    });
    // "Sign in with GitHub", after "Use your password".
    a.click(cx, "choice-1");
    server.set_provider_outcomes(Provider::Github, [format!("session:{account_id}")]);
    a.click(cx, "form-submit");
    wait_until(cx, "the form with the recovery key", |cx| {
        a.draw(cx);
        let mut vcx = VisualTestContext::from_window(a.window, cx);
        vcx.debug_bounds("form-field-0").is_some()
    });
    assert_eq!(ways_in(&server, cx).linked.len(), 1);
    let form = cx.read(crate::proof_sheet::shown_form).expect("the form");
    cx.read(|cx| {
        assert_eq!(form.read(cx).error_text(), Some(words::LAST_METHOD));
    });
    // With the key it goes through.
    server.set_provider_outcomes(Provider::Github, [format!("session:{account_id}")]);
    a.window
        .update(cx, |_, window, cx| {
            form.update(cx, |form, cx| {
                form.fill(&[&key.display()], window, cx);
                form.submit(window, cx);
            })
        })
        .unwrap();
    wait_until(cx, "the unlink", |cx| {
        cx.read(|cx| cx.global::<Services>().task.is_none())
            && kanso_sync::methods(server.as_ref(), &session(cx))
                .unwrap()
                .linked
                .is_empty()
    });
}

/// A wrong password with a right recovery key says the password or the key
/// didn't match, never only "the recovery key"; a provider proof's change
/// that fails on the key blames the key.
#[gpui_kit::test]
fn a_failed_proof_is_not_blamed_on_the_recovery_key(cx: &mut TestAppContext) {
    use account_flow::{MethodChange, ProofWith};
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 0);
    let (_, key) = github_account(cx, &server, &a);
    let account_id = session(cx).account_id.clone();
    a.use_passkey(cx, Arc::new(kanso_sync::TestAuthenticator::new(51)));

    // A wrong password goes with the change, beside a right key: the words
    // name both, never only the key.
    assert_eq!(
        a.change(
            cx,
            MethodChange::NewRecoveryKey,
            password_proof("quiet-otter-4821", "not the password at all"),
            Some(key.clone())
        ),
        Err(words::PASSWORD_OR_RECOVERY.to_owned())
    );
    // A passkey this account doesn't have.
    let said = a.change(
        cx,
        MethodChange::NewRecoveryKey,
        ProofWith::Passkey,
        Some(key.clone()),
    );
    assert!(
        said == Err(words::PASSKEY_UNKNOWN.to_owned())
            || said == Err(words::PASSKEY_OR_RECOVERY.to_owned()),
        "{said:?}"
    );

    // GitHub proved it; the recovery key is the one that's wrong.
    let wrong = RecoveryKey::parse(&kanso_sync::RecoveryKey::generate().display()).unwrap();
    server.set_provider_outcomes(Provider::Github, [format!("session:{account_id}")]);
    assert_eq!(
        a.change(
            cx,
            MethodChange::NewRecoveryKey,
            ProofWith::Provider(Provider::Github),
            Some(wrong)
        ),
        Err("That recovery key doesn't match this account.".to_owned())
    );
}
