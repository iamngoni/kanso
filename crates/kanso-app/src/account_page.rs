//! Settings › Account (docs/ACCOUNTS.md › Account page): signed out, the
//! Create account, Sign in and recovery forms; after sign-up, recovery or a
//! new key, the recovery key screen; signed in, the account, its storage,
//! sync, devices and security, and signing out or deleting the account.
//!
//! The page draws from [`Services`] alone (the account's name, the pending
//! key, the fetched details), so it shows the same with or without a
//! session loaded. The tasks themselves are in `account_flow.rs`.

use std::path::Path;
use std::sync::Arc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _, StyledExt as _, WindowExt as _, h_flex,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::DocType;
use kanso_sync::{DeviceInfo, RecoveryKey, SignupState};

use crate::account_flow;
use crate::account_words::{
    self as words, PASSWORDS_DIFFER, check_current_password, check_new_password, check_recovery,
    check_sign_in, check_sign_up, server_label, sign_up_line, step_label,
};
use crate::clock::Clock;
use crate::dialogs::{self, Field, FormSpec, FormView, Progress, Submit};
use crate::library::Library;
use crate::passkey_ceremony::{self, Platform};
use crate::proof_sheet;
use crate::security_key_flow;
use crate::services::{
    self, AccountDetails, AccountMethodsState, AccountState, PendingKey, Services, SyncState,
    TaskKind,
};
use crate::settings::{Row, SYNC_INTERVALS, interval_label, section, status_row, sync_status_line};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// Which form the signed-out page shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SignedOutForm {
    /// Just the two ways in.
    #[default]
    None,
    CreateAccount,
    SignIn,
    /// "Forgot your password?": the recovery key and a new password.
    Recover,
}

fn runnable_methods(auth: Option<&kanso_sync::ServerAuth>) -> (bool, Vec<kanso_sync::Provider>) {
    match auth {
        None => (true, Vec::new()),
        Some(auth) => (
            auth.password,
            auth.providers
                .iter()
                // Apple, Google and GitHub run on every platform: Apple's
                // sheet on macOS, the system browser and the `kanso://`
                // handler elsewhere (`url_callback.rs`).
                .filter(|provider| {
                    matches!(
                        provider,
                        kanso_sync::Provider::Apple
                            | kanso_sync::Provider::Google
                            | kanso_sync::Provider::Github
                    )
                })
                .cloned()
                .collect(),
        ),
    }
}

/// The passkey or security-key form the signed-out page shows in place of
/// the username form: a passkey sign-up's username step, or a security
/// key's username-first sign-in or sign-up. A platform sign-in has no form:
/// its button opens the system sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PasskeyForm {
    pub creating: bool,
    pub security_key: bool,
}

/// Whether the security-key button shows, and whether a key is attached.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SecurityKeyOffer {
    #[default]
    None,
    /// Advertised, but no key is attached: "Connect a FIDO2 security key to
    /// use one."
    Connect,
    Ready,
}

/// The passkey methods the signed-out page offers, before the providers
/// (docs/ACCOUNTS.md › Account page › Signed out).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PasskeyOffer {
    /// [Continue with a passkey] / [Create with a passkey].
    pub platform: bool,
    /// [Continue with a security key] / [Create with a security key].
    pub security_key: SecurityKeyOffer,
}

/// What `platform` offers for the server's health: a platform passkey when
/// its ceremony can run (and, on macOS, the relying party is the
/// entitled one), a security key on Linux for the server's own relying
/// party.
pub fn passkey_offer(
    platform: Platform,
    auth: Option<&kanso_sync::ServerAuth>,
    server: &str,
    ceremony_available: bool,
    keys_attached: bool,
) -> PasskeyOffer {
    let passkeys = auth.and_then(|auth| auth.passkeys.as_ref());
    let security_key = if !security_key_flow::offers_security_keys(platform, passkeys, server) {
        SecurityKeyOffer::None
    } else if keys_attached {
        SecurityKeyOffer::Ready
    } else {
        SecurityKeyOffer::Connect
    };
    PasskeyOffer {
        platform: passkey_ceremony::offers_platform_passkeys(
            platform,
            passkeys,
            server,
            ceremony_available,
        ),
        security_key,
    }
}

fn completed_form(current: SignedOutForm, has_provider_ticket: bool) -> SignedOutForm {
    if has_provider_ticket {
        SignedOutForm::CreateAccount
    } else {
        current
    }
}

impl SignedOutForm {
    /// The task each form starts, so its button is the one that spins.
    fn task(self) -> Option<TaskKind> {
        match self {
            SignedOutForm::None => None,
            SignedOutForm::CreateAccount => Some(TaskKind::SignUp),
            SignedOutForm::SignIn => Some(TaskKind::SignIn),
            SignedOutForm::Recover => Some(TaskKind::Recover),
        }
    }
}

/// The form the page shows for `account`: an ended session shows Sign in
/// (or the recovery form, when asked for) until signed in again.
pub fn form_for(account: &AccountState, chosen: SignedOutForm) -> SignedOutForm {
    match account {
        AccountState::SignedIn { .. } | AccountState::SignedInLocked { .. } => SignedOutForm::None,
        AccountState::SessionEnded { .. } if chosen != SignedOutForm::Recover => {
            SignedOutForm::SignIn
        }
        _ => chosen,
    }
}

pub struct AccountPage {
    library: Entity<Library>,
    form: SignedOutForm,
    username: Entity<InputState>,
    password: Entity<InputState>,
    confirm: Entity<InputState>,
    recovery_key: Entity<InputState>,
    /// A security key's PIN, when it has one.
    pin: Entity<InputState>,
    /// The passkey form shown in place of the username form.
    passkey_form: Option<PasskeyForm>,
    /// Create account: [Create with a username] was chosen from the
    /// methods, so the password form shows in their place.
    password_form: bool,
    error: Option<SharedString>,
    /// A task started from these forms runs: they stay disabled until it
    /// says it is done, through any question it asks on the way.
    waiting: bool,
    /// The live notes, for the Create account form's line.
    notes: usize,
    /// "I've saved my recovery key".
    key_saved: bool,
    /// The username an ended session filled in.
    prefilled: Option<String>,
    _subscriptions: Vec<Subscription>,
    _listening: Option<Task<()>>,
    /// Linux: security keys coming and going while Sign in or Create
    /// account is open.
    _key_watch: Option<kanso_fido::watch::Watch>,
    _key_events: Option<Task<()>>,
}

impl AccountPage {
    pub fn new(library: Entity<Library>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut input = |masked: bool, placeholder: &'static str, cx: &mut Context<Self>| {
            cx.new(|cx| {
                InputState::new(window, cx)
                    .masked(masked)
                    .placeholder(placeholder)
            })
        };
        let username = input(false, "", cx);
        let password = input(true, "", cx);
        let confirm = input(true, "", cx);
        let recovery_key = input(false, "XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX", cx);
        let pin = input(true, "", cx);
        let subscriptions = [&username, &password, &confirm, &recovery_key, &pin]
            .into_iter()
            .map(|input| {
                cx.subscribe_in(
                    input,
                    window,
                    |this: &mut AccountPage, _, event: &InputEvent, window, cx| match event {
                        InputEvent::PressEnter { .. } => this.submit(window, cx),
                        InputEvent::Change => cx.notify(),
                        _ => {}
                    },
                )
            })
            .collect();
        AccountPage {
            library,
            form: SignedOutForm::None,
            username,
            password,
            confirm,
            recovery_key,
            pin,
            passkey_form: None,
            password_form: false,
            error: None,
            waiting: false,
            notes: 0,
            key_saved: false,
            prefilled: None,
            _subscriptions: subscriptions,
            _listening: None,
            _key_watch: None,
            _key_events: None,
        }
    }

    /// The page came into view: storage and devices are fetched again, and
    /// the Create account form asks its server about new accounts.
    pub fn shown(&mut self, cx: &mut Context<Self>) {
        if cx.try_global::<Services>().is_some_and(Services::signed_in) {
            crate::device_approval::refresh(cx);
        }
        if cx.global::<Services>().signed_in() {
            account_flow::refresh_details(cx);
            account_flow::refresh_account_methods(cx);
        }
        if self.form == SignedOutForm::CreateAccount {
            account_flow::check_signup(cx);
        }
        if self.form != SignedOutForm::None
            && !cx.global::<Services>().signed_in()
            && !cx.global::<Services>().has_locked_session()
        {
            account_flow::check_auth(cx);
        }
    }

    #[cfg(test)]
    pub fn form(&self) -> SignedOutForm {
        self.form
    }

    /// Shows `form` from empty, the keyboard in its first field.
    pub fn open_form(&mut self, form: SignedOutForm, window: &mut Window, cx: &mut Context<Self>) {
        if self.waiting || cx.global::<Services>().has_locked_session() {
            return;
        }
        self.form = form;
        self.passkey_form = None;
        self.password_form = false;
        if matches!(form, SignedOutForm::CreateAccount | SignedOutForm::SignIn) {
            self.watch_security_keys(cx);
        } else {
            self._key_watch = None;
            self._key_events = None;
        }
        if form != SignedOutForm::CreateAccount {
            cx.global_mut::<Services>().provider_signup = None;
        }
        self.error = None;
        self.clear_secrets(window, cx);
        if form == SignedOutForm::CreateAccount {
            self.notes = cx
                .global::<AppState>()
                .store
                .counts()
                .ok()
                .and_then(|counts| counts.get(&DocType::Note).copied())
                .unwrap_or(0);
            account_flow::check_signup(cx);
        }
        if matches!(form, SignedOutForm::CreateAccount | SignedOutForm::SignIn) {
            account_flow::refresh_auth(cx);
        }
        if form != SignedOutForm::None {
            self.username
                .update(cx, |input, cx| input.focus(window, cx));
        }
        cx.notify();
    }

    /// Create account: [Create with a username] shows the password form in
    /// place of the methods.
    pub fn choose_password_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.password_form = true;
        self.error = None;
        self.username
            .update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }

    /// Shows `form`: a passkey sign-up suggests a username, which the user
    /// may change; a security key's sign-in keeps the name typed so far.
    pub fn open_passkey_form(
        &mut self,
        form: PasskeyForm,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.waiting || cx.global::<Services>().has_locked_session() {
            return;
        }
        self.passkey_form = Some(form);
        self.error = None;
        self.clear_secrets(window, cx);
        if form.creating {
            let suggestion = crate::passkey_accounts::suggest_username(fastrand::u64(..));
            self.username
                .update(cx, |input, cx| input.set_value(suggestion, window, cx));
        }
        self.username
            .update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }

    /// Back from a passkey form to the list of ways in.
    fn close_passkey_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.passkey_form = None;
        self.error = None;
        self.clear_secrets(window, cx);
        cx.notify();
    }

    /// Enumerates the security keys now, and again whenever the kernel says
    /// one came or went, while the form is open (Linux; nothing elsewhere).
    fn watch_security_keys(&mut self, cx: &mut Context<Self>) {
        if Platform::current() != Platform::Linux {
            return;
        }
        let (tx, rx) = async_channel::unbounded::<()>();
        tx.try_send(()).ok();
        // The watch's thread holds the only other sender: without a watch,
        // the first enumeration is the last.
        self._key_watch = {
            let tx = tx.clone();
            kanso_fido::watch::watch(move || {
                tx.try_send(()).ok();
            })
        };
        drop(tx);
        self._key_events = Some(cx.spawn(async move |this, cx| {
            while rx.recv().await.is_ok() {
                let keys = cx
                    .background_executor()
                    .spawn(async { security_key_flow::enumerate() })
                    .await;
                let shown = this.update(cx, |_, cx| {
                    cx.global_mut::<Services>().security_keys = keys;
                    cx.notify();
                });
                if shown.is_err() {
                    break;
                }
            }
        }));
    }

    /// Whether this platform's passkey sheet can run now.
    fn platform_passkeys(cx: &App) -> bool {
        let services = cx.global::<Services>();
        !services.platform_passkeys_failed
            && services
                .platform_passkeys
                .unwrap_or_else(passkey_ceremony::platform_available)
    }

    /// [Continue with a passkey]: the system sheet, with its own picker.
    fn platform_sign_in(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let services = cx.global::<Services>();
        if self.waiting || services.task.is_some() || services.has_locked_session() {
            return;
        }
        let server = account_flow::form_server(cx);
        let progress = account_flow::passkey_sign_in(
            server,
            account_flow::PasskeyMethod::Platform {
                window: passkey_ceremony::native_window(window),
            },
            None,
            window.window_handle(),
            cx,
        );
        self.listen(progress, window, cx);
    }

    /// The passkey form's button: a security key's username-first sign-in,
    /// or a passkey's or security key's sign-up with the chosen name.
    fn submit_passkey(&mut self, form: PasskeyForm, window: &mut Window, cx: &mut Context<Self>) {
        let username = Self::value(&self.username, cx);
        let server = account_flow::form_server(cx);
        let handle = window.window_handle();
        let method = if form.security_key {
            let Some(device) =
                security_key_flow::pick(&cx.global::<Services>().security_keys).cloned()
            else {
                self.error = Some(words::CONNECT_SECURITY_KEY.into());
                cx.notify();
                return;
            };
            let pin = zeroize::Zeroizing::new(Self::value(&self.pin, cx));
            account_flow::PasskeyMethod::SecurityKey {
                device,
                pin: Some(pin),
            }
        } else {
            account_flow::PasskeyMethod::Platform {
                window: passkey_ceremony::native_window(window),
            }
        };
        let started = if form.creating {
            kanso_sync::check_new_username(&username)
                .map_err(|err| words::username_words(err).to_owned())
                .map(|name| account_flow::passkey_sign_up(server, method, name, handle, cx))
        } else {
            kanso_sync::normalize_username(&username)
                .map_err(|err| words::username_words(err).to_owned())
                .map(|name| account_flow::passkey_sign_in(server, method, Some(name), handle, cx))
        };
        match started {
            Err(words) => {
                self.error = Some(words.into());
                cx.notify();
            }
            Ok(progress) => {
                // The PIN doesn't stay on screen while the key runs.
                self.pin
                    .update(cx, |input, cx| input.set_value("", window, cx));
                self.listen(progress, window, cx);
            }
        }
    }

    fn clear_secrets(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for input in [&self.password, &self.confirm, &self.recovery_key, &self.pin] {
            input.update(cx, |input, cx| input.set_value("", window, cx));
        }
    }

    fn value(input: &Entity<InputState>, cx: &App) -> String {
        input.read(cx).value().to_string()
    }

    /// Presses the form's button.
    pub fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let services = cx.global::<Services>();
        if self.waiting || services.task.is_some() || services.has_locked_session() {
            return;
        }
        let form = form_for(&services.account, self.form);
        if let Some(passkey_form) = self.passkey_form
            && matches!(form, SignedOutForm::SignIn | SignedOutForm::CreateAccount)
        {
            return self.submit_passkey(passkey_form, window, cx);
        }
        let username = Self::value(&self.username, cx);
        let password = Self::value(&self.password, cx);
        let confirm = Self::value(&self.confirm, cx);
        let key = Self::value(&self.recovery_key, cx);
        let server = account_flow::form_server(cx);
        let handle = window.window_handle();
        let started = match form {
            SignedOutForm::None => return,
            SignedOutForm::CreateAccount => {
                if let Some(provider_signup) = cx.global::<Services>().provider_signup.clone() {
                    if provider_signup.server != server {
                        cx.global_mut::<Services>().provider_signup = None;
                        self.error =
                            Some("The server changed. Start provider sign up again.".into());
                        cx.notify();
                        return;
                    }
                    match kanso_sync::check_new_username(&username) {
                        Ok(name) => Ok(account_flow::provider_sign_up(
                            provider_signup,
                            name,
                            handle,
                            cx,
                        )),
                        Err(err) => Err(words::username_words(err).to_owned()),
                    }
                } else {
                    check_sign_up(&username, &password, &confirm).map(|(name, password)| {
                        account_flow::sign_up(server, name, password, handle, cx)
                    })
                }
            }
            SignedOutForm::SignIn => check_sign_in(&username, &password)
                .map(|(name, password)| account_flow::sign_in(server, name, password, handle, cx)),
            SignedOutForm::Recover => {
                check_recovery(&username, &key, &password, &confirm).map(|(name, key, password)| {
                    account_flow::recover(server, name, key, password, Some(handle), cx)
                })
            }
        };
        match started {
            Err(words) => {
                self.error = Some(words.into());
                cx.notify();
            }
            Ok(progress) => {
                // Secrets don't stay on screen while the task runs.
                for input in [&self.password, &self.confirm] {
                    input.update(cx, |input, cx| input.set_value("", window, cx));
                }
                self.listen(progress, window, cx);
            }
        }
    }

    fn listen(
        &mut self,
        progress: async_channel::Receiver<Progress>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.error = None;
        self.waiting = true;
        cx.notify();
        self._listening = Some(cx.spawn_in(window, async move |this, cx| {
            let mut result = Err("Something went wrong. Try again.".to_owned());
            while let Ok(event) = progress.recv().await {
                match event {
                    Progress::Step(_) => {
                        if this.update(cx, |_, cx| cx.notify()).is_err() {
                            return;
                        }
                    }
                    Progress::Done(done) => {
                        result = done;
                        break;
                    }
                }
            }
            this.update_in(cx, |this, window, cx| {
                this.waiting = false;
                match result {
                    Ok(()) => {
                        this.error = None;
                        this.clear_secrets(window, cx);
                        let ticket = cx.global::<Services>().provider_signup.clone();
                        this.form = completed_form(this.form, ticket.is_some());
                        if let Some(ticket) = ticket {
                            this.username.update(cx, |input, cx| {
                                input.set_value(ticket.suggested_username, window, cx);
                            });
                        }
                        if cx.global::<Services>().signed_in() {
                            this.form = SignedOutForm::None;
                            cx.global_mut::<Services>().provider_signup = None;
                        }
                        let services = cx.global::<Services>();
                        if services.signed_in() || services.has_locked_session() {
                            this.passkey_form = None;
                            this._key_watch = None;
                            this._key_events = None;
                        }
                    }
                    Err(words) => this.error = Some(words.into()),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// Fills the form as if typed (tests and snapshots).
    #[cfg(test)]
    pub fn fill(&mut self, values: [&str; 4], window: &mut Window, cx: &mut Context<Self>) {
        let inputs = [
            &self.username,
            &self.password,
            &self.confirm,
            &self.recovery_key,
        ];
        for (input, value) in inputs.into_iter().zip(values) {
            let value = value.to_owned();
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        cx.notify();
    }

    /// Shows `error` under the form, as a failed task does (snapshots).
    #[cfg(test)]
    pub fn show_error(&mut self, error: &str, cx: &mut Context<Self>) {
        self.error = Some(error.to_owned().into());
        cx.notify();
    }

    #[cfg(test)]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    #[cfg(test)]
    pub fn set_key_saved(&mut self, saved: bool, cx: &mut Context<Self>) {
        self.key_saved = saved;
        cx.notify();
    }

    #[cfg(test)]
    pub fn set_notes(&mut self, notes: usize) {
        self.notes = notes;
    }
}

// MARK: Drawing

impl Render for AccountPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let candidate = {
            let services = cx.global_mut::<Services>();
            if services.task.is_none()
                && services.pending_key.is_none()
                && services.approval.waiting.is_none()
            {
                services.pending_unlocked_candidate.take()
            } else {
                None
            }
        };
        if let Some(candidate) = candidate {
            let handle = window.window_handle();
            cx.defer(move |cx| {
                account_flow::finish_unlocked_candidate(candidate, Some(handle), None, cx)
            });
        }
        let services = cx.global::<Services>();
        let account = services.account.clone();
        let pending = services.pending_key.clone();
        let waiting = services.approval.waiting.clone();
        // An ended session fills in its name once, for Sign in.
        if let AccountState::SessionEnded { username, .. } = &account
            && self.prefilled.as_ref() != Some(username)
        {
            let name = username.clone();
            self.prefilled = Some(name.clone());
            self.username
                .update(cx, |input, cx| input.set_value(name, window, cx));
        }
        let body = match (&pending, &waiting, &account) {
            (Some(key), _, _) => self.key_screen(key, cx),
            (None, Some(waiting), _) => {
                vec![crate::device_approval::waiting_page(waiting, window, cx)]
            }
            (None, None, AccountState::SignedIn { username }) => self.signed_in_page(username, cx),
            (None, None, AccountState::SignedInLocked { username }) => {
                self.locked_page(username, window, cx)
            }
            (None, None, _) => self.signed_out_page(&account, window, cx),
        };
        v_flex().w_full().children(body)
    }
}

impl AccountPage {
    fn locked_page(
        &mut self,
        _username: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let colors = Colors::get(cx);
        let has_password = cx.global::<Services>().locked_has_password.unwrap_or(false);
        let start_again = cx
            .global::<Services>()
            .approval
            .waiting
            .as_ref()
            .is_some_and(|waiting| {
                matches!(
                    waiting.phase,
                    crate::device_approval::WaitPhase::Expired
                        | crate::device_approval::WaitPhase::Cancelled
                )
            });
        let busy = self.waiting || cx.global::<Services>().task.is_some();
        let approval_window = window.window_handle();
        let mut page = vec![
            div()
                .text_size(px(17.))
                .line_height(px(24.))
                .font_semibold()
                .text_color(colors.text)
                .child("Approve this device")
                .into_any_element(),
            div()
                .text_size(px(13.5))
                .line_height(px(20.))
                .text_color(colors.text_secondary)
                .child("Open Kanso on a device that's already signed in and approve this one.")
                .into_any_element(),
        ];
        page.push(
            h_flex()
                .mt(px(10.))
                .gap_2()
                .child(
                    Button::new("locked-start-approval")
                        .primary()
                        .label(if start_again {
                            "Start again"
                        } else {
                            "Ask another device"
                        })
                        .disabled(busy)
                        .on_click(move |_, _, cx| {
                            account_flow::restart_device_approval(Some(approval_window), cx)
                        }),
                )
                .child(
                    Button::new("locked-cancel")
                        .outline()
                        .label("Cancel")
                        .disabled(busy)
                        .on_click(|_, _, cx| account_flow::cancel_locked_session(cx)),
                )
                .into_any_element(),
        );
        if has_password {
            page.push(
                Button::new("locked-password")
                    .ghost()
                    .small()
                    .label("Unlock with password")
                    .disabled(busy)
                    .on_click(|_, window, cx| {
                        unlock_password_dialog(window, cx);
                    })
                    .into_any_element(),
            );
        }
        page.push(
            Button::new("locked-recovery")
                .ghost()
                .small()
                .label("Use your recovery key")
                .disabled(busy)
                .on_click(|_, window, cx| {
                    recover_dialog(window, cx);
                })
                .into_any_element(),
        );
        page
    }

    fn tooltip(cx: &App) -> String {
        cx.try_global::<Services>()
            .map(|services| services.sync_status(cx))
            .unwrap_or_default()
    }

    /// A field in the sign-in column: its label above (with `aside` at the
    /// label's right, such as "Forgot your password?"), its hint below.
    #[allow(clippy::too_many_arguments, reason = "one field's whole description")]
    fn stacked_field(
        &self,
        label: &'static str,
        hint: Option<&'static str>,
        input: &Entity<InputState>,
        selector: &'static str,
        disabled: bool,
        aside: Option<AnyElement>,
        colors: &Colors,
    ) -> AnyElement {
        v_flex()
            .gap(px(6.))
            .child(
                h_flex()
                    .min_h(px(22.))
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.text)
                            .child(label),
                    )
                    .children(aside),
            )
            .child(
                div()
                    .w_full()
                    .debug_selector(move || selector.into())
                    .child(
                        Input::new(input)
                            .disabled(disabled)
                            .when(self.is_password(input), |input| input.mask_toggle()),
                    ),
            )
            .when_some(hint, |el, hint| {
                el.child(
                    div()
                        .text_size(px(12.))
                        .line_height(px(16.))
                        .text_color(colors.text_secondary)
                        .child(hint),
                )
            })
            .into_any_element()
    }

    /// Whether `input` is one of the password fields, which can be shown.
    fn is_password(&self, input: &Entity<InputState>) -> bool {
        [&self.password, &self.confirm]
            .iter()
            .any(|field| field.entity_id() == input.entity_id())
    }

    /// The forms' server, quietly at the column's foot, and the way to
    /// change it.
    fn server_line(&self, disabled: bool, colors: &Colors, cx: &App) -> AnyElement {
        let server = cx.global::<AppState>().config.sync.server.clone();
        let custom = server.is_some();
        h_flex()
            .pt(px(12.))
            .border_t_1()
            .border_color(colors.divider)
            .gap(px(6.))
            .items_center()
            .child(
                Icon::new(Lucide::Server)
                    .size(px(13.))
                    .text_color(colors.text_muted),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(px(12.))
                    .text_color(colors.text_secondary)
                    .debug_selector(|| "account-server".into())
                    .child(server_label(server.as_deref())),
            )
            .child(
                Button::new("use-different-server")
                    .ghost()
                    .xsmall()
                    .label("Change…")
                    .disabled(disabled)
                    .debug_selector(|| "use-different-server".into())
                    .on_click(|_, window, cx| {
                        server_dialog(window, cx);
                    }),
            )
            .when(custom, |el| {
                el.child(
                    Button::new("use-default-server")
                        .ghost()
                        .xsmall()
                        .label("Use the default")
                        .disabled(disabled)
                        .on_click(|_, _, cx| account_flow::set_server(None, cx)),
                )
            })
            .into_any_element()
    }

    fn signed_out_page(
        &mut self,
        account: &AccountState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let colors = Colors::get(cx);
        let services = cx.global::<Services>();
        let task = services.task;
        let signup = services.signup.clone();
        let server = account_flow::form_server(cx);
        let provider_signup = services
            .provider_signup
            .clone()
            .filter(|ticket| ticket.server == server);
        let current_auth = services
            .auth
            .as_ref()
            .filter(|(asked, _)| *asked == account_flow::form_server(cx))
            .map(|(_, auth)| auth);
        let (password_available, advertised) = runnable_methods(current_auth);
        let offer = passkey_offer(
            Platform::current(),
            current_auth,
            &server,
            Self::platform_passkeys(cx),
            !services.security_keys.is_empty(),
        );
        let form = form_for(account, self.form);
        let busy = self.waiting || task.is_some();
        let status = sync_status_line(
            &SyncState::Off,
            account,
            &cx.global::<AppState>().config.sync,
            |_| String::new(),
            |_| String::new(),
            None,
        );
        // The forms carry their own heading; the landing page and an ended
        // session (whose reason it says) keep the status row.
        let mut page = Vec::new();
        if form == SignedOutForm::None || matches!(account, AccountState::SessionEnded { .. }) {
            page.push(status_row(&status, Self::tooltip(cx), None, colors));
        }
        if let Some(wait) = services.provider_wait.as_ref() {
            page.push(provider_wait_view(wait, colors, cx));
        }
        if form == SignedOutForm::None {
            // What an account gives, before what it asks.
            let benefit = |icon: Lucide, text: &'static str| {
                h_flex()
                    .gap(px(10.))
                    .child(
                        Icon::new(icon)
                            .size(px(16.))
                            .text_color(colors.text_secondary),
                    )
                    .child(
                        div()
                            .text_size(px(13.5))
                            .line_height(px(20.))
                            .text_color(colors.text)
                            .child(text),
                    )
            };
            page.push(
                v_flex()
                    .mt(px(18.))
                    .gap(px(8.))
                    .child(benefit(
                        Lucide::MonitorSmartphone,
                        "Your notes on all your devices",
                    ))
                    .child(benefit(Lucide::CloudUpload, "A backup off this computer"))
                    .child(benefit(Lucide::Lock, "Locked so only you can read them"))
                    .child(benefit(
                        Lucide::Gift,
                        "Free, and your notes stay here if you never sign up",
                    ))
                    .into_any_element(),
            );
            page.push(
                h_flex()
                    .mt(px(18.))
                    .gap_2()
                    .child(
                        Button::new("account-create")
                            .primary()
                            .icon(Lucide::UserPlus)
                            .label("Create account")
                            .debug_selector(|| "account-create".into())
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_form(SignedOutForm::CreateAccount, window, cx)
                            })),
                    )
                    .child(
                        Button::new("account-sign-in")
                            .outline()
                            .icon(Lucide::LogIn)
                            .label("Sign in")
                            .debug_selector(|| "account-sign-in".into())
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_form(SignedOutForm::SignIn, window, cx)
                            })),
                    )
                    .into_any_element(),
            );
            return page;
        }
        let signup = signup
            .filter(|(asked, _)| *asked == server)
            .map(|(_, signup)| signup);
        if let Some(passkey_form) = self.passkey_form
            && matches!(form, SignedOutForm::SignIn | SignedOutForm::CreateAccount)
        {
            page.extend(self.passkey_form_page(passkey_form, signup, busy, task, cx));
            return page;
        }
        let pw = |hint: &'static str| Some(hint);
        let (heading, fields, submit): (&'static str, Vec<AnyElement>, &'static str) = match form {
            SignedOutForm::CreateAccount => (
                words::CREATE_HEADING,
                vec![
                    self.stacked_field(
                        "Username",
                        pw("3–32 lowercase letters, digits, - or _"),
                        &self.username,
                        "account-username",
                        busy,
                        None,
                        colors,
                    ),
                    self.stacked_field(
                        "Password",
                        pw("At least 12 characters. A few random words work well."),
                        &self.password,
                        "account-password",
                        busy,
                        None,
                        colors,
                    ),
                    self.stacked_field(
                        "Confirm password",
                        None,
                        &self.confirm,
                        "account-confirm",
                        busy,
                        None,
                        colors,
                    ),
                ],
                "Create account",
            ),
            SignedOutForm::SignIn => (
                words::SIGN_IN_HEADING,
                vec![
                    self.stacked_field(
                        "Username",
                        None,
                        &self.username,
                        "account-username",
                        busy,
                        None,
                        colors,
                    ),
                    self.stacked_field(
                        "Password",
                        None,
                        &self.password,
                        "account-password",
                        busy,
                        password_available.then(|| {
                            Button::new("account-forgot")
                                .ghost()
                                .xsmall()
                                .label("Forgot your password?")
                                .disabled(busy)
                                .debug_selector(|| "account-forgot".into())
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.open_form(SignedOutForm::Recover, window, cx)
                                }))
                                .into_any_element()
                        }),
                        colors,
                    ),
                ],
                "Sign in",
            ),
            SignedOutForm::Recover | SignedOutForm::None => (
                words::RECOVER_HEADING,
                vec![
                    self.stacked_field(
                        "Username",
                        None,
                        &self.username,
                        "account-username",
                        busy,
                        None,
                        colors,
                    ),
                    self.stacked_field(
                        "Recovery key",
                        pw("The key Kanso gave you when you made the account."),
                        &self.recovery_key,
                        "account-recovery-key",
                        busy,
                        None,
                        colors,
                    ),
                    self.stacked_field(
                        "New password",
                        pw("At least 12 characters. A few random words work well."),
                        &self.password,
                        "account-password",
                        busy,
                        None,
                        colors,
                    ),
                    self.stacked_field(
                        "Confirm",
                        None,
                        &self.confirm,
                        "account-confirm",
                        busy,
                        None,
                        colors,
                    ),
                ],
                "Set new password",
            ),
        };
        let has_methods = !advertised.is_empty()
            || offer.platform
            || offer.security_key != SecurityKeyOffer::None;
        // Create account lists its methods, the username one among them; the
        // password form replaces the list once that one is chosen.
        let choosing = form == SignedOutForm::CreateAccount
            && provider_signup.is_none()
            && has_methods
            && !self.password_form;
        let creating = form == SignedOutForm::CreateAccount;
        let closed_to_new = creating && !matches!(signup.as_ref(), Some(SignupState::Open));
        let mut methods: Vec<AnyElement> = Vec::new();
        if matches!(form, SignedOutForm::SignIn | SignedOutForm::CreateAccount)
            && !(creating && (provider_signup.is_some() || self.password_form))
        {
            if offer.platform {
                let (label, selector) = if creating {
                    (words::CREATE_WITH_PASSKEY, "account-passkey-create")
                } else {
                    (words::CONTINUE_WITH_PASSKEY, "account-passkey")
                };
                methods.push(
                    method_button(selector, Lucide::FingerprintPattern, label)
                        .disabled(busy || closed_to_new)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            if creating {
                                let form = PasskeyForm {
                                    creating: true,
                                    security_key: false,
                                };
                                this.open_passkey_form(form, window, cx);
                            } else {
                                this.platform_sign_in(window, cx);
                            }
                        }))
                        .into_any_element(),
                );
            }
            match offer.security_key {
                SecurityKeyOffer::Ready => {
                    let (label, selector) = if creating {
                        (
                            words::CREATE_WITH_SECURITY_KEY,
                            "account-security-key-create",
                        )
                    } else {
                        (words::CONTINUE_WITH_SECURITY_KEY, "account-security-key")
                    };
                    methods.push(
                        method_button(selector, Lucide::Usb, label)
                            .disabled(busy || closed_to_new)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                let form = PasskeyForm {
                                    creating,
                                    security_key: true,
                                };
                                this.open_passkey_form(form, window, cx);
                            }))
                            .into_any_element(),
                    );
                }
                SecurityKeyOffer::Connect => methods.push(
                    h_flex()
                        .gap(px(8.))
                        .debug_selector(|| "account-security-key-connect".into())
                        .child(
                            Icon::new(Lucide::Usb)
                                .size(px(16.))
                                .text_color(colors.text_secondary),
                        )
                        .child(quiet_text(words::CONNECT_SECURITY_KEY, colors))
                        .into_any_element(),
                ),
                SecurityKeyOffer::None => {}
            }
            for provider in &advertised {
                let (sign_in_label, create_label, selector, icon) = match provider {
                    kanso_sync::Provider::Apple => (
                        "Continue with Apple",
                        "Create with Apple",
                        "account-provider-apple",
                        Lucide::Apple,
                    ),
                    kanso_sync::Provider::Google => (
                        "Continue with Google",
                        "Create with Google",
                        "account-provider-google",
                        Lucide::Globe,
                    ),
                    kanso_sync::Provider::Github => (
                        "Continue with GitHub",
                        "Create with GitHub",
                        "account-provider-github",
                        Lucide::Github,
                    ),
                    _ => continue,
                };
                let label = if creating {
                    create_label
                } else {
                    sign_in_label
                };
                let provider = provider.clone();
                methods.push(
                    method_button(selector, icon, label)
                        .disabled(busy || closed_to_new)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            let progress = account_flow::start_provider(
                                provider.clone(),
                                creating,
                                window.window_handle(),
                                cx,
                            );
                            this.listen(progress, window, cx);
                        }))
                        .into_any_element(),
                );
            }
            if creating && password_available {
                methods.push(
                    method_button(
                        "account-username-create",
                        Lucide::User,
                        words::CREATE_WITH_USERNAME,
                    )
                    .disabled(busy || closed_to_new)
                    .on_click(
                        cx.listener(|this, _, window, cx| this.choose_password_form(window, cx)),
                    )
                    .into_any_element(),
                );
            }
        }

        let error = self.error.clone().map(|error| error_text(error, colors));
        if choosing {
            let mut column = vec![
                auth_heading(words::CREATE_HEADING, colors),
                quiet_text(sign_up_line(self.notes, signup), colors),
            ];
            column.push(v_flex().gap(px(8.)).children(methods).into_any_element());
            column.extend(error);
            column.push(
                h_flex()
                    .child(
                        Button::new("account-cancel")
                            .ghost()
                            .small()
                            .label("Cancel")
                            .disabled(busy)
                            .debug_selector(|| "account-cancel".into())
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_form(SignedOutForm::None, window, cx)
                            })),
                    )
                    .into_any_element(),
            );
            column.push(self.server_line(busy, colors, cx));
            page.push(auth_column(column));
            return page;
        }

        let mut column = Vec::new();
        let fields = if form == SignedOutForm::CreateAccount
            && let Some(provider_signup) = &provider_signup
        {
            self.username.update(cx, |input, cx| {
                if input.value().is_empty() {
                    input.set_value(provider_signup.suggested_username.clone(), window, cx);
                }
            });
            column.push(auth_heading(words::PICK_USERNAME_HEADING, colors));
            column.push(quiet_text(
                "You can change this before creating the account.",
                colors,
            ));
            vec![self.stacked_field(
                "Username",
                pw("3–32 lowercase letters, digits, - or _"),
                &self.username,
                "account-username",
                busy,
                None,
                colors,
            )]
        } else {
            column.push(auth_heading(heading, colors));
            if form == SignedOutForm::Recover {
                column.push(quiet_text(
                    "The recovery key you use stops working; Kanso gives you a new one.",
                    colors,
                ));
            }
            if password_available || form == SignedOutForm::Recover {
                fields
            } else {
                Vec::new()
            }
        };
        let has_fields = !fields.is_empty();
        if !methods.is_empty() {
            column.push(v_flex().gap(px(8.)).children(methods).into_any_element());
            if has_fields {
                column.push(or_divider(colors));
            }
        }
        if has_fields {
            column.push(v_flex().gap(px(14.)).children(fields).into_any_element());
        }

        // Under the form: what will happen, the error, and the buttons.
        // Only a full server blocks the form: "closed" may hide a reservation.
        let closed = matches!(signup, Some(SignupState::Full));
        if form == SignedOutForm::CreateAccount {
            column.push(quiet_text(sign_up_line(self.notes, signup), colors));
        }
        column.extend(error);
        let running_here = task.is_some_and(|task| Some(task.kind) == form.task());
        let spinning = busy && (running_here || self.waiting);
        let label = match task {
            Some(task) if running_here => SharedString::from(step_label(task.step)),
            _ => SharedString::from(submit),
        };
        let ready = !busy
            && !Self::value(&self.username, cx).trim().is_empty()
            && !(form == SignedOutForm::CreateAccount && closed);
        let ended = matches!(account, AccountState::SessionEnded { .. });
        let back_to_methods = form == SignedOutForm::CreateAccount
            && self.password_form
            && has_methods
            && provider_signup.is_none();
        if password_available || form == SignedOutForm::Recover || provider_signup.is_some() {
            column.push(
                Button::new("account-submit")
                    .primary()
                    .w_full()
                    .label(label)
                    // The spinner takes the icon's place; a loading button
                    // ignores clicks, and keeps its colors so the step reads.
                    .when(spinning, |button| button.icon(Lucide::RefreshCcw))
                    .loading(spinning)
                    .disabled(!ready && !spinning)
                    .debug_selector(|| "account-submit".into())
                    .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx)))
                    .into_any_element(),
            );
        }
        if !(ended && form == SignedOutForm::SignIn) {
            let cancel = if back_to_methods {
                "Other ways to create an account"
            } else if form == SignedOutForm::Recover {
                "Back to sign in"
            } else {
                "Cancel"
            };
            column.push(
                h_flex()
                    .justify_center()
                    .child(
                        Button::new("account-cancel")
                            .ghost()
                            .small()
                            .label(cancel)
                            .disabled(busy)
                            .debug_selector(|| "account-cancel".into())
                            .on_click(cx.listener(move |this, _, window, cx| {
                                // From the password form, back to the methods.
                                if back_to_methods {
                                    this.password_form = false;
                                    this.error = None;
                                    this.clear_secrets(window, cx);
                                    cx.notify();
                                } else if form == SignedOutForm::Recover {
                                    this.open_form(SignedOutForm::SignIn, window, cx)
                                } else {
                                    this.open_form(SignedOutForm::None, window, cx)
                                }
                            })),
                    )
                    .into_any_element(),
            );
        }
        column.push(self.server_line(busy, colors, cx));
        page.push(auth_column(column));
        page
    }

    /// A passkey sign-up's username step, or a security key's form: the
    /// username, the PIN for a security key, the server, and the button.
    fn passkey_form_page(
        &mut self,
        form: PasskeyForm,
        signup: Option<SignupState>,
        busy: bool,
        task: Option<services::RunningTask>,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let colors = Colors::get(cx);
        let title = match (form.creating, form.security_key) {
            (true, false) => words::CREATE_WITH_PASSKEY,
            (true, true) => words::CREATE_WITH_SECURITY_KEY,
            (false, true) => words::CONTINUE_WITH_SECURITY_KEY,
            (false, false) => words::CONTINUE_WITH_PASSKEY,
        };
        let no_key = form.security_key && cx.global::<Services>().security_keys.is_empty();
        let mut fields = vec![
            self.stacked_field(
                "Username",
                form.creating
                    .then_some("3–32 lowercase letters, digits, - or _"),
                &self.username,
                "account-username",
                busy,
                None,
                colors,
            ),
        ];
        if form.security_key {
            fields.push(self.stacked_field(
                "PIN",
                Some(words::SECURITY_KEY_PIN_HINT),
                &self.pin,
                "account-security-key-pin",
                busy,
                None,
                colors,
            ));
        }
        let note = match (form.creating, form.security_key) {
            (true, _) => words::PASSKEY_USERNAME_NOTE,
            (false, _) => words::SECURITY_KEY_SIGN_IN_NOTE,
        };
        let mut column = vec![
            auth_heading(title, colors),
            quiet_text(note, colors),
            v_flex().gap(px(14.)).children(fields).into_any_element(),
        ];
        let closed = matches!(signup, Some(SignupState::Full));
        let lines: Vec<SharedString> = [
            form.creating
                .then(|| words::passkey_sign_up_line(self.notes, signup).into()),
            (form.creating && form.security_key).then_some(words::SECURITY_KEY_SIGN_UP_NOTE.into()),
            no_key.then_some(words::CONNECT_SECURITY_KEY.into()),
        ]
        .into_iter()
        .flatten()
        .collect();
        column.extend(lines.into_iter().map(|line| quiet_text(line, colors)));
        column.extend(self.error.clone().map(|error| error_text(error, colors)));
        let kind = if form.creating {
            TaskKind::SignUp
        } else {
            TaskKind::SignIn
        };
        let running_here = task.is_some_and(|task| task.kind == kind);
        let spinning = busy && (running_here || self.waiting);
        let label = match task {
            Some(task) if running_here => SharedString::from(step_label(task.step)),
            _ if form.creating => SharedString::from("Create account"),
            _ => SharedString::from("Sign in"),
        };
        let ready = !busy
            && !no_key
            && !Self::value(&self.username, cx).trim().is_empty()
            && !(form.creating && closed);
        column.push(
            Button::new("account-passkey-submit")
                .primary()
                .w_full()
                .label(label)
                .when(spinning, |button| button.icon(Lucide::RefreshCcw))
                .loading(spinning)
                .disabled(!ready && !spinning)
                .debug_selector(|| "account-passkey-submit".into())
                .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx)))
                .into_any_element(),
        );
        column.push(
            h_flex()
                .justify_center()
                .child(
                    Button::new("account-passkey-back")
                        .ghost()
                        .small()
                        .label("Cancel")
                        .disabled(busy)
                        .debug_selector(|| "account-passkey-back".into())
                        .on_click(
                            cx.listener(|this, _, window, cx| this.close_passkey_form(window, cx)),
                        ),
                )
                .into_any_element(),
        );
        column.push(self.server_line(busy, colors, cx));
        vec![auth_column(column)]
    }

    fn key_screen(&mut self, pending: &PendingKey, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let colors = Colors::get(cx);
        let mono = cx.theme().mono_font_family.clone();
        let shown = pending.key.display();
        let copy_key = pending.key.clone();
        let save = pending.clone();
        let saved = self.key_saved;
        vec![
            v_flex()
                .mt(px(18.))
                .gap(px(14.))
                .child(
                    div()
                        .text_size(px(17.))
                        .line_height(px(24.))
                        .font_semibold()
                        .text_color(colors.text)
                        .child("Save your recovery key"),
                )
                .when_some(pending.note, |el, note| {
                    el.child(
                        div()
                            .debug_selector(|| "recovery-key-note".into())
                            .text_size(px(13.5))
                            .line_height(px(20.))
                            .font_medium()
                            .text_color(colors.warning)
                            .child(note),
                    )
                })
                .child(
                    div()
                        .text_size(px(13.5))
                        .line_height(px(20.))
                        .text_color(colors.text_secondary)
                        .child(words::RECOVERY_KEY_TEXT),
                )
                .child(
                    v_flex()
                        .debug_selector(|| "recovery-key".into())
                        .items_center()
                        .gap(px(6.))
                        .py(px(22.))
                        .px(px(16.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(colors.divider)
                        .bg(colors.list)
                        .child(
                            div()
                                .font_family(mono)
                                .text_size(px(22.))
                                .line_height(px(30.))
                                .text_color(colors.text)
                                .child(shown),
                        )
                        .child(
                            div()
                                .text_size(px(12.5))
                                .text_color(colors.text_muted)
                                .child(format!(
                                    "For {} on {}",
                                    pending.username,
                                    words::host_of(&pending.server)
                                )),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("copy-recovery-key")
                                .outline()
                                .icon(Lucide::Copy)
                                .label("Copy")
                                .debug_selector(|| "copy-recovery-key".into())
                                .on_click(move |_, window, cx| {
                                    copy_key_to_clipboard(&copy_key, window, cx)
                                }),
                        )
                        .child(
                            Button::new("save-recovery-key")
                                .outline()
                                .icon(Lucide::Save)
                                .label("Save as file…")
                                .debug_selector(|| "save-recovery-key".into())
                                .on_click(move |_, window, cx| {
                                    save_key_file(save.clone(), window, cx)
                                }),
                        ),
                )
                .child(
                    h_flex()
                        .mt(px(6.))
                        .gap_2()
                        .items_center()
                        .child(
                            div().flex_1().child(
                                Checkbox::new("recovery-key-saved")
                                    .label("I've saved my recovery key")
                                    .checked(saved)
                                    .debug_selector(|| "recovery-key-saved".into())
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.key_saved = *checked;
                                        cx.notify();
                                    })),
                            ),
                        )
                        .child(
                            Button::new("recovery-key-done")
                                .primary()
                                .label("Done")
                                .disabled(!saved)
                                .debug_selector(|| "recovery-key-done".into())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.key_saved = false;
                                    account_flow::key_saved(cx);
                                    cx.notify();
                                })),
                        ),
                )
                .into_any_element(),
        ]
    }

    fn signed_in_page(&mut self, username: &str, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let colors = Colors::get(cx);
        let services = cx.global::<Services>();
        let state = services.sync.clone();
        let account = services.account_state();
        let details = services.details.clone();
        let account_methods = services.account_methods.clone();
        let busy = services.task.is_some();
        let server = services
            .session()
            .map(|session| session.server.clone())
            .or_else(|| cx.global::<AppState>().config.sync.server.clone());
        let settings = cx.global::<AppState>().config.sync.clone();
        let library = self.library.clone();

        // The first conflicted note that is still there, to name and open.
        let conflicted = match &state {
            SyncState::Synced { conflicted, .. } => {
                let store = &cx.global::<AppState>().store;
                conflicted
                    .iter()
                    .filter(|id| id.starts_with("note:"))
                    .find_map(|id| store.note(id).ok().flatten())
            }
            _ => None,
        };
        let conflicted_title = conflicted.as_ref().map(|note| {
            if note.title.trim().is_empty() {
                "Untitled".to_owned()
            } else {
                note.title.clone()
            }
        });
        let status = sync_status_line(
            &state,
            &account,
            &settings,
            |at| Clock::edited_label(cx, at),
            |at| {
                words::resume_time(
                    Clock::local_ms(cx, at),
                    Clock::local_ms(cx, Clock::now_ms(cx)),
                )
            },
            conflicted_title.as_deref(),
        );
        let open_note = conflicted.map(|note| {
            let id = note.id;
            Button::new("open-conflicted-note")
                .outline()
                .label("Open note")
                .debug_selector(|| "open-conflicted-note".into())
                .on_click(move |_, _, cx| {
                    let id = id.clone();
                    library.update(cx, |library, cx| library.select(Some(id), cx));
                })
        });
        let sync_now = Button::new("sync-now")
            .outline()
            .icon(Lucide::RefreshCcw)
            .label("Sync now")
            .loading(state == SyncState::Syncing)
            .disabled(busy)
            .debug_selector(|| "sync-now".into())
            .on_click(|_, _, cx| services::sync_now(cx));
        let actions = h_flex()
            .flex_none()
            .gap_2()
            .children(open_note)
            .child(sync_now);

        let minutes = (settings.interval_secs / 60).max(1);
        let interval = Button::new("sync-interval")
            .outline()
            .small()
            .label(interval_label(minutes))
            .dropdown_caret(true)
            .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, _, _| {
                SYNC_INTERVALS.iter().fold(menu, |menu, &option| {
                    menu.item(
                        PopupMenuItem::new(interval_label(option))
                            .checked(option == minutes)
                            .on_click(move |_, _, cx| {
                                AppState::update_config(cx, |config| {
                                    config.sync.interval_secs = option * 60
                                });
                                services::apply_sync_settings(cx);
                            }),
                    )
                })
            });

        let storage = storage_detail(&details);
        let devices = device_rows(&details, busy, cx);
        let sign_in_methods =
            sign_in_methods_rows(&account_methods, |at| Clock::edited_label(cx, at), busy, cx);
        let has_password = account_methods
            .last_successful
            .as_ref()
            .is_none_or(|methods| methods.has_password);
        let provider_wait = services
            .provider_wait
            .as_ref()
            .map(|wait| provider_wait_view(wait, colors, cx));

        vec![
            status_row(&status, Self::tooltip(cx), Some(actions.into_any_element()), colors),
            section(
                "Account",
                vec![
                    Row::new(Lucide::CircleUser, username.to_owned())
                        .value(server_label(server.as_deref())),
                    Row::new(Lucide::HardDrive, "Storage").detail(storage),
                ],
                colors,
            ),
            section(
                "Sync",
                vec![
                    Row::new(Lucide::RefreshCcw, "Sync automatically")
                        .detail("In the background, and soon after you edit.")
                        .trailing(
                            Switch::new("sync-enabled")
                                .checked(settings.enabled)
                                .on_click(|on, _, cx| {
                                    let on = *on;
                                    AppState::update_config(cx, |config| config.sync.enabled = on);
                                    services::apply_sync_settings(cx);
                                }),
                        ),
                    Row::new(Lucide::Clock, "Sync every").trailing(interval),
                ],
                colors,
            ),
            div()
                .debug_selector(|| "account-sign-in-methods".into())
                .child(section("Sign-in methods", sign_in_methods, colors))
                .children(provider_wait)
                .into_any_element(),
            section("Devices", devices, colors),
            section(
                "Security",
                has_password.then(|| {
                    Row::new(Lucide::KeyRound, "Password")
                        .detail("Your other devices ask for the new one, and you get a new recovery key.")
                        .trailing(
                            Button::new("change-account-password")
                                .outline()
                                .small()
                                .label("Change password…")
                                .disabled(busy)
                                .debug_selector(|| "change-account-password".into())
                                .on_click(|_, window, cx| {
                                    change_password_dialog(window, cx);
                                }),
                        )
                })
                .into_iter()
                .chain([
                    Row::new(Lucide::FileKey, "Recovery key")
                        .detail("A new key replaces the one you saved, which stops working.")
                        .trailing(
                            Button::new("new-recovery-key")
                                .outline()
                                .small()
                                .label("New recovery key…")
                                .disabled(busy)
                                .debug_selector(|| "new-recovery-key".into())
                                .on_click(|_, window, cx| {
                                    new_recovery_key_dialog(window, cx);
                                }),
                        ),
                fingerprint_row(cx),
                    Row::new(Lucide::LockKeyhole, "Forgot your password?")
                        .detail(if has_password {
                            "Set a new one with your recovery key."
                        } else {
                            "Set one with your recovery key. It removes every passkey."
                        })
                        .trailing(
                            Button::new("recover-signed-in")
                                .outline()
                                .small()
                                .label("Use recovery key…")
                                .disabled(busy)
                                .debug_selector(|| "recover-signed-in".into())
                                .on_click(|_, window, cx| {
                                    recover_dialog(window, cx);
                                }),
                        ),
                ])
                .collect(),
                colors,
            ),
            section(
                "Conflict handling",
                vec![
                    Row::new(Lucide::Split, "Keep both versions")
                        .detail("When a note changes on two devices at once, the newer version shows and the other is kept in the note's history."),
                ],
                colors,
            ),
            h_flex()
                .mt(px(26.))
                .gap_2()
                .child(
                    Button::new("sign-out")
                        .outline()
                        .icon(Lucide::LogOut)
                        .label("Sign out")
                        .disabled(busy)
                        .debug_selector(|| "sign-out".into())
                        .on_click(|_, window, cx| sign_out_dialog(window, cx)),
                )
                .child(div().flex_1())
                .child(
                    Button::new("delete-account")
                        .danger()
                        .outline()
                        .label("Delete account…")
                        .disabled(busy)
                        .debug_selector(|| "delete-account".into())
                        .on_click(move |_, window, cx| {
                            delete_account_dialog(window, cx);
                        }),
                )
                .into_any_element(),
        ]
    }
}

/// The wait for a provider: GitHub's code, or "Waiting for Apple sign-in…",
/// with [Cancel]. Shown on the signed-out page and over Sign-in methods.
/// How wide the sign-in column is.
const AUTH_COLUMN: f32 = 360.;

/// The signed-out forms' one column: heading, ways in, fields, button.
fn auth_column(children: Vec<AnyElement>) -> AnyElement {
    v_flex()
        .mt(px(20.))
        .w(px(AUTH_COLUMN))
        .max_w_full()
        .gap(px(16.))
        .children(children)
        .into_any_element()
}

fn auth_heading(text: &'static str, colors: &Colors) -> AnyElement {
    div()
        .text_size(px(20.))
        .line_height(px(26.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(colors.text)
        .debug_selector(|| "account-form-heading".into())
        .child(text)
        .into_any_element()
}

fn quiet_text(text: impl Into<SharedString>, colors: &Colors) -> AnyElement {
    div()
        .text_size(px(13.))
        .line_height(px(18.))
        .text_color(colors.text_secondary)
        .child(text.into())
        .into_any_element()
}

fn error_text(text: impl Into<SharedString>, colors: &Colors) -> AnyElement {
    div()
        .debug_selector(|| "account-error".into())
        .text_size(px(13.))
        .line_height(px(18.))
        .text_color(colors.danger)
        .child(text.into())
        .into_any_element()
}

/// One way in: a full-width outlined button with its mark at the left.
fn method_button(selector: &'static str, icon: Lucide, label: &'static str) -> Button {
    Button::new(SharedString::from(selector))
        .outline()
        .w_full()
        .icon(icon)
        .label(label)
        .debug_selector(move || selector.into())
}

/// The rule between the ways in and the username form.
fn or_divider(colors: &Colors) -> AnyElement {
    let rule = || div().flex_1().h(px(1.)).bg(colors.divider);
    h_flex()
        .gap(px(10.))
        .items_center()
        .child(rule())
        .child(
            div()
                .text_size(px(12.))
                .text_color(colors.text_muted)
                .child("or"),
        )
        .child(rule())
        .into_any_element()
}

fn provider_wait_view(
    wait: &crate::services::ProviderWait,
    colors: &Colors,
    cx: &App,
) -> AnyElement {
    let mut waiting = h_flex().mt(px(14.)).gap(px(8.));
    waiting = if let (Some(uri), Some(code)) =
        (wait.verification_uri.as_ref(), wait.user_code.as_ref())
    {
        waiting.child(
            div()
                .text_size(px(13.))
                .text_color(colors.text_secondary)
                .child(format!("Paste this code at {uri}: {code}")),
        )
    } else {
        waiting.child(
            div()
                .text_size(px(13.))
                .text_color(colors.text_secondary)
                .child(format!("Waiting for {0} sign-in…", wait.provider)),
        )
    };
    waiting = waiting.child(
        Button::new("cancel-provider-wait")
            .ghost()
            .small()
            .label("Cancel")
            .debug_selector(|| "cancel-provider-wait".into())
            .on_click(|_, _, cx| account_flow::cancel_provider_wait(cx))
            .into_any_element(),
    );
    v_flex()
        .mt(px(14.))
        .gap(px(6.))
        .child(waiting)
        .children(wait.user_code.as_ref().map(|code| {
            div()
                .px(px(12.))
                .py(px(8.))
                .rounded(px(6.))
                .bg(colors.list)
                .font_family(cx.theme().mono_font_family.clone())
                .text_size(px(20.))
                .text_color(colors.text)
                .child(code.clone())
        }))
        .when(wait.user_code.is_some(), |view| {
            view.child(
                div()
                    .text_size(px(12.))
                    .text_color(colors.text_secondary)
                    .child("It's copied, so you can paste it on GitHub's page. It stays here while Kanso checks for approval."),
            )
        })
        .when(wait.provider == "Apple", |view| {
            view.child(
                div()
                    .text_size(px(12.))
                    .text_color(colors.text_secondary)
                    .child(words::APPLE_WAIT_NOTE),
            )
        })
        .into_any_element()
}

/// The Storage row's line, or why it isn't there yet.
pub fn storage_detail(details: &AccountDetails) -> String {
    match (&details.usage, &details.failed) {
        (Some(usage), _) => words::storage_line(usage),
        (None, Some(failed)) => failed.clone(),
        (None, None) => "Checking…".to_owned(),
    }
}

/// Sign-in method rows, in display order. While a refresh is pending or has
/// failed, retained methods are deliberately hidden so old data is not shown
/// as current.
fn sign_in_methods_rows(
    state: &AccountMethodsState,
    when: impl Fn(i64) -> String,
    busy: bool,
    cx: &App,
) -> Vec<Row> {
    let mut rows: Vec<Row> = sign_in_method_lines(state, when)
        .into_iter()
        .map(|(icon, title, detail)| Row::new(icon, title).detail(detail))
        .collect();
    let (false, None, Some(methods)) = (state.loading, &state.failed, &state.last_successful)
    else {
        return rows;
    };
    let small = |id: SharedString, label: &'static str| {
        Button::new(id.clone())
            .outline()
            .small()
            .label(label)
            .disabled(busy)
            .debug_selector(move || id.to_string())
    };
    // The rows' order is `sign_in_method_lines`': the password, the linked
    // providers, the passkeys, then the providers left to link.
    let mut rows_out = Vec::with_capacity(rows.len() + 1);
    let mut rest = rows.drain(..);
    if let Some(password) = rest.next() {
        let button = if methods.has_password {
            small("change-password-method".into(), "Change password…").on_click(|_, window, cx| {
                change_password_dialog(window, cx);
            })
        } else {
            small("set-password".into(), "Set a password…").on_click(|_, window, cx| {
                proof_sheet::open(proof_sheet::Action::SetPassword, window, cx)
            })
        };
        rows_out.push(password.trailing(button));
    }
    for linked in &methods.linked {
        let Some(row) = rest.next() else { break };
        let provider = linked.provider.clone();
        rows_out.push(row.trailing(
            small(format!("unlink-{}", provider.as_str()).into(), "Unlink").on_click(
                move |_, window, cx| {
                    proof_sheet::open(proof_sheet::Action::Unlink(provider.clone()), window, cx)
                },
            ),
        ));
    }
    for (ix, passkey) in methods.passkeys.iter().enumerate() {
        let Some(row) = rest.next() else { break };
        let id = passkey.credential_id.clone();
        rows_out.push(row.trailing(
            small(format!("remove-passkey-{ix}").into(), "Remove").on_click(
                move |_, window, cx| {
                    proof_sheet::open(proof_sheet::Action::RemovePasskey(id.clone()), window, cx)
                },
            ),
        ));
    }
    if methods.rp_id.is_some() && !passkey_already_here(Platform::current(), methods) {
        let ceremony = proof_sheet::ceremony(cx);
        let (title, detail) = match ceremony {
            Some(true) => ("Add a security key", "Sign in with a FIDO2 security key."),
            _ => (
                "Add a passkey",
                "Sign in with this device's passkey, without a password.",
            ),
        };
        let detail = if ceremony.is_none() {
            words::PASSKEYS_NOT_HERE
        } else {
            detail
        };
        rows_out.push(
            Row::new(Lucide::FingerprintPattern, title)
                .detail(detail)
                .trailing(
                    small("add-passkey".into(), "Add a passkey…")
                        .disabled(busy || ceremony.is_none())
                        .on_click(|_, window, cx| {
                            proof_sheet::open(proof_sheet::Action::AddPasskey, window, cx)
                        }),
                ),
        );
    }
    let linkable: Vec<kanso_sync::Provider> = methods
        .unlinked()
        .into_iter()
        .filter(|provider| {
            matches!(
                provider,
                kanso_sync::Provider::Apple
                    | kanso_sync::Provider::Google
                    | kanso_sync::Provider::Github
            )
        })
        .collect();
    if !methods.unlinked().is_empty()
        && let Some(row) = rest.next()
    {
        let row = if linkable.is_empty() {
            row
        } else {
            row.trailing(
                small("link-provider".into(), "Link a provider…")
                    .dropdown_caret(true)
                    .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, _, _| {
                        linkable.iter().fold(menu, |menu, provider| {
                            let chosen = provider.clone();
                            menu.item(
                                PopupMenuItem::new(proof_sheet::provider_name(provider)).on_click(
                                    move |_, window, cx| {
                                        proof_sheet::open(
                                            proof_sheet::Action::Link(chosen.clone()),
                                            window,
                                            cx,
                                        )
                                    },
                                ),
                            )
                        })
                    }),
            )
        };
        rows_out.push(row);
    }
    rows_out.extend(rest);
    rows_out
}

fn sign_in_method_lines(
    state: &AccountMethodsState,
    when: impl Fn(i64) -> String,
) -> Vec<(Lucide, String, String)> {
    if state.loading {
        return vec![(
            Lucide::KeyRound,
            "Sign-in methods".into(),
            if state.last_successful.is_some() {
                "Refreshing…"
            } else {
                "Checking…"
            }
            .into(),
        )];
    }
    if let Some(failed) = &state.failed {
        return vec![(
            Lucide::KeyRound,
            "Sign-in methods".into(),
            format!("Couldn't refresh. {failed}"),
        )];
    }
    let Some(methods) = &state.last_successful else {
        return vec![(
            Lucide::KeyRound,
            "Sign-in methods".into(),
            "Checking…".into(),
        )];
    };

    let mut rows = vec![(
        Lucide::KeyRound,
        "Password".into(),
        if methods.has_password {
            "Password set"
        } else {
            "No password"
        }
        .into(),
    )];
    rows.extend(methods.linked.iter().map(|linked| {
        (
            provider_icon(&linked.provider),
            provider_name(&linked.provider),
            format!("linked {}", when(linked.created_at)),
        )
    }));
    rows.extend(methods.passkeys.iter().map(|passkey| {
        let mut detail = format!("added {}", when(passkey.created_at));
        if let Some(last_used_at) = passkey.last_used_at {
            detail.push_str(&format!(" · last used {}", when(last_used_at)));
        }
        detail.push_str(if passkey.prf {
            " · can unlock"
        } else {
            " · signs in only"
        });
        let title = match passkey.manager() {
            Some(manager) => format!("Passkey in {manager}"),
            None => "Passkey".to_owned(),
        };
        (Lucide::FingerprintPattern, title, detail)
    }));
    let unlinked = methods.unlinked();
    if !unlinked.is_empty() {
        let names = unlinked.iter().map(provider_name).collect::<Vec<_>>();
        let available = match names.as_slice() {
            [only] => format!("{only} is available to link."),
            [first, second] => format!("{first} and {second} are available to link."),
            _ => format!("{} are available to link.", names.join(", ")),
        };
        rows.push((Lucide::Globe, "Available providers".into(), available));
    }
    rows.push((
        Lucide::FileKey,
        "Recovery key".into(),
        "Your recovery key still works if you lose these.".into(),
    ));
    if sign_in_method_count(methods) == 1 {
        rows.push((
            Lucide::Info,
            "Sign-in".into(),
            "This is your last way to sign in.".into(),
        ));
    }
    rows
}

fn sign_in_method_count(methods: &kanso_sync::AccountMethods) -> usize {
    usize::from(methods.has_password) + methods.linked.len() + methods.passkeys.len()
}

/// A Mac already has a passkey the account holds in iCloud Keychain, so
/// [Add a passkey…] would only meet the platform's "already registered".
fn passkey_already_here(platform: Platform, methods: &kanso_sync::AccountMethods) -> bool {
    platform == Platform::MacOs
        && methods
            .passkeys
            .iter()
            .any(kanso_sync::provider::PasskeyInfo::in_icloud_keychain)
}

fn provider_name(provider: &kanso_sync::Provider) -> String {
    match provider {
        kanso_sync::Provider::Apple => "Apple".to_owned(),
        kanso_sync::Provider::Google => "Google".to_owned(),
        kanso_sync::Provider::Github => "GitHub".to_owned(),
        kanso_sync::Provider::Other(name) => name.clone(),
    }
}

fn provider_icon(provider: &kanso_sync::Provider) -> Lucide {
    match provider {
        kanso_sync::Provider::Apple => Lucide::Apple,
        kanso_sync::Provider::Github => Lucide::Github,
        kanso_sync::Provider::Google | kanso_sync::Provider::Other(_) => Lucide::Globe,
    }
}

/// What a device's row says under its name: this Mac, or when it was last
/// active, named as the note list names a time (`when`).
pub fn device_detail(device: &DeviceInfo, when: impl FnOnce(i64) -> String) -> String {
    if device.current {
        "This device".to_owned()
    } else {
        format!("Last active {}", when(device.last_seen_at))
    }
}

fn device_rows(details: &AccountDetails, busy: bool, cx: &App) -> Vec<Row> {
    let Some(devices) = &details.devices else {
        let words = details
            .failed
            .clone()
            .unwrap_or_else(|| "Checking…".to_owned());
        return vec![Row::new(Lucide::Laptop, "Devices").detail(words)];
    };
    let mut rows: Vec<Row> = if devices.is_empty() {
        vec![Row::new(Lucide::Laptop, "No devices").detail("None is signed in.")]
    } else {
        devices
            .iter()
            .map(|device| {
                let title = words::device_title(device);
                let row = Row::new(Lucide::Laptop, title.clone())
                    .detail(device_detail(device, |at| Clock::edited_label(cx, at)));
                if device.current {
                    row
                } else {
                    let id = device.id.clone();
                    row.trailing(
                        Button::new(SharedString::from(format!("revoke-{}", device.id)))
                            .outline()
                            .small()
                            .label("Sign out")
                            .disabled(busy)
                            .on_click(move |_, window, cx| {
                                revoke_dialog(title.clone(), id.clone(), window, cx)
                            }),
                    )
                }
            })
            .collect()
    };
    if cx
        .try_global::<Services>()
        .is_some_and(|services| services.session().is_some() && services.backend.is_some())
    {
        rows.push(
            Row::new(Lucide::Laptop, "Add a computer without a screen…")
                .detail("Make a one-time code for `kanso account sign-in --pair`.")
                .trailing(
                    Button::new("add-screenless-computer")
                        .outline()
                        .small()
                        .label("Add computer…")
                        .disabled(busy)
                        .debug_selector(|| "add-screenless-computer".into())
                        .on_click(|_, window, cx| start_screenless_pairing(window, cx)),
                ),
        );
    }
    rows
}

/// Creates a ten-minute pairing code, shows the command to run on the new
/// computer, and watches for its use while the existing approval listener
/// opens the normal six-digit approval sheet.
fn start_screenless_pairing(window: &mut Window, cx: &mut App) {
    let Some(services) = cx.try_global::<Services>() else {
        return;
    };
    let (Some(session), Some(backend)) = (services.session(), services.backend.clone()) else {
        return;
    };
    let store = cx.global::<AppState>().store.clone();
    let window_handle = window.window_handle();
    let poll_backend = backend.clone();
    let poll_session = session.clone();
    let cancel_pairing = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let close_signal = cancel_pairing.clone();
    cx.spawn(async move |cx| {
        let made = cx
            .background_executor()
            .spawn(async move {
                let identity = kanso_sync::device_identity(
                    &store,
                    &kanso_sync::Machine::this_computer(),
                )
                .map_err(|error| error.to_string())?;
                kanso_sync::create_pairing(backend.as_ref(), &session, &identity)
                    .map_err(|error| error.to_string())
            })
            .await;
        let pairing = match made {
            Ok(pairing) => pairing,
            Err(error) => {
                let message = format!("Couldn't make a pairing code: {error}");
                cx.update_window(window_handle, |_, window, cx| {
                    window.push_notification(Notification::error(message), cx);
                })
                .ok();
                return;
            }
        };
        let command = format!("kanso account sign-in --pair {}", pairing.code);
        cx.update_window(window_handle, |_, window, cx| {
            let description = format!(
                "Run this on the computer you're adding. This code works once, for 10 minutes.\n\n{}\n\nWaiting for a computer to use this code…",
                command
            );
            let copy = command.clone();
            let close_signal = close_signal.clone();
            window.open_alert_dialog(cx, move |alert, _, _| {
                let copy = copy.clone();
                let close_signal = close_signal.clone();
                alert
                    .title("Add a computer without a screen")
                    .description(description.clone())
                    .show_cancel(true)
                    .cancel_text("Done")
                    .ok_text("Copy command")
                    .on_ok(move |_, _, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(copy.clone()));
                        false
                    })
                    .on_close(move |_, _, _| {
                        close_signal.store(true, std::sync::atomic::Ordering::SeqCst);
                    })
            });
        })
        .ok();

        let pairing_id = pairing.id;
        let deadline = pairing.expires_at;
        while kanso_core::now_ms() < deadline
            && !cancel_pairing.load(std::sync::atomic::Ordering::SeqCst)
        {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(2))
                .await;
            let backend = poll_backend.clone();
            let session = poll_session.clone();
            let id = pairing_id.clone();
            let status = cx
                .background_executor()
                .spawn(async move {
                    kanso_sync::pairing_status(backend.as_ref(), &session, &id)
                })
                .await;
            match status {
                Ok(status) if status.status == "used" => {
                    cx.update_window(window_handle, |_, window, cx| {
                        window.push_notification(Notification::info(
                            "The computer used this code. Check the device approval sheet and compare its code.",
                        ), cx);
                    })
                    .ok();
                    break;
                }
                Ok(status) if status.status == "waiting" => {}
                _ => break,
            }
        }
        if cancel_pairing.load(std::sync::atomic::Ordering::SeqCst) {
            let backend = poll_backend;
            let session = poll_session;
            let id = pairing_id;
            let _ = cx
                .background_executor()
                .spawn(async move { kanso_sync::cancel_pairing(backend.as_ref(), &session, &id) })
                .await;
        }
    })
    .detach();
}

/// This account's fingerprint, for the people it shares notes with to
/// compare: six groups in two lines of the monospaced face.
// no-prop-test: drawing; `fingerprint_lines` has the properties.
fn fingerprint_row(cx: &App) -> Row {
    let row = Row::new(Lucide::FingerprintPattern, "Your fingerprint");
    let Some(fingerprint) = cx
        .try_global::<crate::collab_model::Collab>()
        .and_then(|c| c.fingerprint.clone())
    else {
        return row.detail("Shows once your keys for shared notes are made, at the next sync.");
    };
    let colors = Colors::get(cx);
    row.detail(FINGERPRINT_SENTENCE).trailing(
        v_flex()
            .debug_selector(|| "account-fingerprint".into())
            .px(px(10.))
            .py(px(6.))
            .gap(px(2.))
            .rounded(px(6.))
            .bg(colors.fill)
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(px(13.))
            .text_color(colors.text)
            .children(
                fingerprint_lines(&fingerprint)
                    .into_iter()
                    .map(|line| div().child(line)),
            ),
    )
}

/// What the fingerprint row says about it.
pub const FINGERPRINT_SENTENCE: &str = "Compare these numbers with what the people you share with see for you, in person or on a call.";

/// A fingerprint's groups, three to a line. Pure.
pub fn fingerprint_lines(fingerprint: &str) -> Vec<String> {
    crate::collab_text::fingerprint_groups(fingerprint)
        .chunks(3)
        .map(|groups| groups.join(" "))
        .collect()
}

// MARK: Dialogs, also run from the palette

/// The signed-in account's name, for the dialogs that ask for it.
fn account_name(cx: &App) -> Option<String> {
    match &cx.try_global::<Services>()?.account {
        AccountState::SignedIn { username } => Some(username.clone()),
        AccountState::SignedInLocked { username } => Some(username.clone()),
        _ => None,
    }
}

/// "Use a different server…": checked by `check_endpoint`, then asked
/// whether it is a Kanso server.
pub fn server_dialog(window: &mut Window, cx: &mut App) -> Entity<FormView> {
    let current = cx
        .global::<AppState>()
        .config
        .sync
        .server
        .clone()
        .unwrap_or_default();
    dialogs::form(
        FormSpec::new("Use a different server", "Use server")
            .description("Kanso checks that the server runs Kanso sync before using it.")
            .field(
                Field::text("Server address")
                    .hint("Such as https://sync.example.com")
                    .initial(current),
            ),
        window,
        cx,
        |values, _, cx| Submit::Running(account_flow::use_server(values[0].clone(), cx)),
    )
}

/// Whether the account is known to have no password: its changes then take
/// a proof through the proof sheet.
fn passwordless(cx: &App) -> bool {
    cx.try_global::<Services>()
        .and_then(|services| services.account_methods.last_successful.as_ref())
        .is_some_and(|methods| !methods.has_password)
}

pub fn change_password_dialog(window: &mut Window, cx: &mut App) -> Option<Entity<FormView>> {
    if passwordless(cx) {
        proof_sheet::open(proof_sheet::Action::SetPassword, window, cx);
        return None;
    }
    let username = account_name(cx)?;
    Some(dialogs::form(
        FormSpec::new("Change password", "Change password")
            .description(words::CHANGE_PASSWORD_TEXT)
            .field(Field::secret("Current password"))
            .field(
                Field::secret("New password")
                    .hint("At least 12 characters. A few random words work well."),
            )
            .field(Field::secret("Confirm new password")),
        window,
        cx,
        move |values, _, cx| {
            let current = match check_current_password(&values[0], &username) {
                Ok(current) => current,
                Err(words) => return Submit::Invalid(words),
            };
            match check_new_password(&values[1], &values[2], &username) {
                Ok(new) => Submit::Running(account_flow::change_password(current, new, cx)),
                Err(words) => Submit::Invalid(words),
            }
        },
    ))
}

pub fn new_recovery_key_dialog(window: &mut Window, cx: &mut App) -> Option<Entity<FormView>> {
    if passwordless(cx) {
        proof_sheet::open(proof_sheet::Action::NewRecoveryKey, window, cx);
        return None;
    }
    let username = account_name(cx)?;
    Some(dialogs::form(
        FormSpec::new("New recovery key", "Make a new key")
            .description(
                "The key you saved stops working, and Kanso shows the new one to save instead.",
            )
            .field(Field::secret("Current password")),
        window,
        cx,
        move |values, _, cx| match check_current_password(&values[0], &username) {
            Ok(current) => Submit::Running(account_flow::new_recovery_key(current, cx)),
            Err(words) => Submit::Invalid(words),
        },
    ))
}

/// "Forgot your password?" while signed in: the recovery key and a new
/// password.
pub fn recover_dialog(window: &mut Window, cx: &mut App) -> Option<Entity<FormView>> {
    let username = account_name(cx)?;
    Some(dialogs::form(
        FormSpec::new("Forgot your password?", "Set new password")
            .description(
                "The recovery key you use stops working; Kanso gives you a new one to save.",
            )
            .field(Field::text("Recovery key"))
            .field(
                Field::secret("New password")
                    .hint("At least 12 characters. A few random words work well."),
            )
            .field(Field::secret("Confirm new password")),
        window,
        cx,
        move |values, window, cx| match recovery_values(&values, &username) {
            Ok((key, password)) => {
                let progress = if cx.global::<Services>().has_locked_session() {
                    account_flow::recover_locked(key, password, Some(window.window_handle()), cx)
                } else {
                    account_flow::recover_signed_in(key, password, cx)
                };
                Submit::Running(progress)
            }
            Err(words) => Submit::Invalid(words),
        },
    ))
}

/// Password unlock for a provider session; the data key is returned only
/// after the server authenticates the saved provider LoginDevice.
pub fn unlock_password_dialog(window: &mut Window, cx: &mut App) -> Option<Entity<FormView>> {
    let username = account_name(cx)?;
    Some(dialogs::form(
        FormSpec::new("Unlock this device", "Unlock")
            .description("Your password unlocks this device's local copy of your notes.")
            .field(Field::secret("Password")),
        window,
        cx,
        move |values, window, cx| {
            let [password] = values.as_slice() else {
                return Submit::Invalid("Enter your password.".into());
            };
            match check_sign_in(&username, password) {
                Ok((_, password)) => Submit::Running(account_flow::unlock_locked_password(
                    password,
                    Some(window.window_handle()),
                    cx,
                )),
                Err(error) => Submit::Invalid(error),
            }
        },
    ))
}

/// The signed-in recovery form's key and new password.
fn recovery_values(
    values: &[String],
    username: &str,
) -> Result<(RecoveryKey, kanso_sync::Password), String> {
    let [key, password, confirm] = values else {
        return Err(PASSWORDS_DIFFER.to_owned());
    };
    let (_, key, password) = check_recovery(username, key, password, confirm)?;
    Ok((key, password))
}

pub fn sign_out_dialog(window: &mut Window, cx: &mut App) {
    if let Some(username) = account_name(cx) {
        sign_out_dialog_for(&username, window, cx);
    }
}

fn sign_out_dialog_for(username: &str, window: &mut Window, cx: &mut App) {
    let unsaved_key = cx.global::<Services>().pending_key.is_some();
    let (title, body) = words::sign_out_question(username, unsaved_key);
    dialogs::confirm(title, body, "Sign out", window, cx, |_, cx| {
        account_flow::sign_out(cx);
    });
}

pub fn delete_account_dialog(window: &mut Window, cx: &mut App) -> Option<Entity<FormView>> {
    if passwordless(cx) {
        proof_sheet::open(proof_sheet::Action::DeleteAccount, window, cx);
        return None;
    }
    let username = account_name(cx)?;
    Some(delete_account_dialog_for(&username, window, cx))
}

fn delete_account_dialog_for(
    username: &str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<FormView> {
    let name = username.to_owned();
    dialogs::confirm_typed(
        FormSpec::new("Delete account?", "Delete account")
            .description(words::delete_account_words(username))
            .field(Field::secret("Password"))
            .danger(),
        username,
        window,
        cx,
        move |values, _, cx| match check_current_password(&values[1], &name) {
            Ok(password) => Submit::Running(account_flow::delete_account(
                password,
                values[0].clone(),
                cx,
            )),
            Err(words) => Submit::Invalid(words),
        },
    )
}

fn revoke_dialog(name: String, device_id: String, window: &mut Window, cx: &mut App) {
    let (title, body) = words::revoke_question(&name);
    dialogs::confirm(title, body, "Sign out", window, cx, move |_, cx| {
        // The task says how it went.
        account_flow::revoke_device(device_id.clone(), name.clone(), cx);
    });
}

// MARK: The recovery key's copy and file

fn copy_key_to_clipboard(key: &RecoveryKey, window: &mut Window, cx: &mut App) {
    let notification = if crate::clipboard::copy_secret(&key.display(), cx) {
        Notification::info(words::COPIED_KEY)
    } else {
        Notification::error("Couldn't copy the key. Write it down or save it as a file.")
    };
    window.push_notification(notification, cx);
}

/// Writes the key's file, readable only by this user. The save panel has
/// asked before replacing a file already there. (On Windows the file keeps
/// its folder's permissions.)
pub fn write_key_file(path: &Path, text: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(path)?;
    // A file that was there keeps its old mode unless it is set again.
    #[cfg(unix)]
    std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
    file.write_all(text.as_bytes())?;
    file.sync_all()
}

/// "Save as file…": a save panel, a question first for a folder that syncs
/// to iCloud, then the 0600 file.
fn save_key_file(pending: PendingKey, window: &mut Window, cx: &mut App) {
    let home = crate::paths::home();
    let start = home.join("Documents");
    let picked = cx.prompt_for_new_path(
        if start.is_dir() { &start } else { &home },
        Some(words::RECOVERY_KEY_FILE_NAME),
    );
    let date = words::file_date(Clock::local_ms(cx, Clock::now_ms(cx)));
    let text = words::recovery_file(
        &pending.key.display(),
        &pending.username,
        &pending.server,
        &date,
    );
    window
        .spawn(cx, async move |cx| {
            let Ok(Ok(Some(path))) = picked.await else {
                return;
            };
            let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
            cx.update(|window, cx| {
                if crate::clipboard::syncs_to_icloud(&folder, &home) {
                    dialogs::confirm(
                        "Save to iCloud?",
                        words::ICLOUD_WARNING,
                        "Save anyway",
                        window,
                        cx,
                        move |window, cx| write_and_tell(&path, &text, window, cx),
                    );
                } else {
                    write_and_tell(&path, &text, window, cx);
                }
            })
            .ok();
        })
        .detach();
}

fn write_and_tell(path: &Path, text: &str, window: &mut Window, cx: &mut App) {
    let notification = match write_key_file(path, text) {
        Ok(()) => Notification::success("Saved the recovery key."),
        Err(err) => {
            eprintln!(
                "kanso: saving the recovery key to {}: {err}",
                path.display()
            );
            Notification::error("Couldn't save the file. Choose another folder, or copy the key.")
        }
    };
    window.push_notification(notification, cx);
}

#[cfg(test)]
mod tests {
    use super::{
        FINGERPRINT_SENTENCE, PasskeyOffer, SecurityKeyOffer, SignedOutForm, device_detail,
        fingerprint_lines, form_for, passkey_already_here, passkey_offer, runnable_methods,
        sign_in_method_count, sign_in_method_lines, storage_detail, write_key_file,
    };
    use crate::passkey_ceremony::Platform;
    use crate::services::{AccountDetails, AccountMethodsState, AccountState, strategies};
    use kanso_sync::{DeviceInfo, Quota, Usage};
    use proptest::prelude::*;

    fn device(current: bool, last_seen_at: i64) -> DeviceInfo {
        DeviceInfo {
            id: "d_1".into(),
            name: Some("Studio".into()),
            created_at: 0,
            last_seen_at,
            current,
        }
    }

    #[test]
    fn examples() {
        let ended = AccountState::SessionEnded {
            username: "ngoni".into(),
            reason: kanso_sync::SignOutReason::Revoked,
        };
        assert_eq!(form_for(&ended, SignedOutForm::None), SignedOutForm::SignIn);
        assert_eq!(
            form_for(&ended, SignedOutForm::Recover),
            SignedOutForm::Recover
        );
        assert_eq!(
            form_for(&AccountState::SignedOut, SignedOutForm::CreateAccount),
            SignedOutForm::CreateAccount
        );
        assert_eq!(storage_detail(&AccountDetails::default()), "Checking…");
        let details = AccountDetails {
            usage: Some(Usage {
                docs: 850,
                doc_bytes: 3_250_586,
                files: 15,
                file_bytes: 14 * 1024 * 1024,
                writes_today: 0,
                quota: Quota {
                    docs: 100_000,
                    doc_bytes: 100 * 1024 * 1024,
                    files: 10_000,
                    file_bytes: 500 * 1024 * 1024,
                    writes_per_day: 10_000,
                },
            }),
            ..AccountDetails::default()
        };
        assert_eq!(
            storage_detail(&details),
            "850 items · 3.1 MB of 100 MB · attachments 14 MB of 500 MB (15 of 10,000)"
        );
        assert_eq!(
            device_detail(&device(true, 0), |_| "now".into()),
            "This device"
        );
        assert_eq!(
            device_detail(&device(false, 0), |_| "2 h ago".into()),
            "Last active 2 h ago"
        );
    }

    #[test]
    fn an_icloud_keychain_passkey_is_named_and_needs_no_second_one_on_a_mac() {
        let passkey = |aaguid: Option<&str>| kanso_sync::provider::PasskeyInfo {
            credential_id: vec![1],
            created_at: 2,
            last_used_at: None,
            prf: true,
            transports: vec!["internal".into()],
            aaguid: aaguid.map(str::to_owned),
        };
        let methods = |passkeys| kanso_sync::AccountMethods {
            has_password: false,
            rp_id: Some("sync.kansoapp.app".into()),
            available: vec![],
            linked: vec![],
            passkeys,
        };
        let icloud = methods(vec![passkey(Some(
            kanso_sync::provider::ICLOUD_KEYCHAIN_AAGUIDS[0],
        ))]);
        assert!(passkey_already_here(Platform::MacOs, &icloud));
        // Elsewhere, or kept by another manager, another passkey can be added.
        assert!(!passkey_already_here(Platform::Windows, &icloud));
        assert!(!passkey_already_here(Platform::Linux, &icloud));
        let other = methods(vec![passkey(Some("bada5566a7aa401fbd9645619a55120d"))]);
        assert!(!passkey_already_here(Platform::MacOs, &other));
        assert!(!passkey_already_here(
            Platform::MacOs,
            &methods(vec![passkey(None)])
        ));
        assert!(!passkey_already_here(Platform::MacOs, &methods(vec![])));

        let state = AccountMethodsState {
            last_successful: Some(icloud),
            ..AccountMethodsState::default()
        };
        let titles: Vec<String> = sign_in_method_lines(&state, |at| format!("date-{at}"))
            .into_iter()
            .map(|(_, title, _)| title)
            .collect();
        assert!(
            titles.contains(&"Passkey in iCloud Keychain".to_owned()),
            "{titles:?}"
        );
    }

    #[test]
    fn signed_in_methods_show_current_text_and_last_way_only_for_one_method() {
        let mut methods = kanso_sync::AccountMethods {
            has_password: false,
            rp_id: None,
            available: vec![kanso_sync::Provider::Apple, kanso_sync::Provider::Google],
            linked: vec![kanso_sync::provider::LinkedProvider {
                provider: kanso_sync::Provider::Apple,
                created_at: 1,
            }],
            passkeys: vec![kanso_sync::provider::PasskeyInfo {
                credential_id: vec![1],
                created_at: 2,
                last_used_at: Some(3),
                prf: true,
                transports: vec![],
                aaguid: None,
            }],
        };
        let state = AccountMethodsState {
            last_successful: Some(methods.clone()),
            ..AccountMethodsState::default()
        };
        let lines = sign_in_method_lines(&state, |at| format!("date-{at}"));
        let text = lines
            .iter()
            .map(|(_, title, detail)| format!("{title} · {detail}"))
            .collect::<Vec<_>>();
        assert_eq!(text[0], "Password · No password");
        assert_eq!(text[1], "Apple · linked date-1");
        assert_eq!(
            text[2],
            "Passkey · added date-2 · last used date-3 · can unlock"
        );
        assert!(text.contains(&"Available providers · Google is available to link.".to_owned()));
        assert!(text.contains(
            &"Recovery key · Your recovery key still works if you lose these.".to_owned()
        ));
        assert!(!text.iter().any(|line| line.contains("last way")));
        assert_eq!(sign_in_method_count(&methods), 2);

        methods.has_password = true;
        methods.linked.clear();
        methods.passkeys.clear();
        assert_eq!(sign_in_method_count(&methods), 1);
        let state = AccountMethodsState {
            last_successful: Some(methods),
            ..AccountMethodsState::default()
        };
        let text = sign_in_method_lines(&state, |_| "unused".into());
        assert!(text.iter().any(|(_, _, detail)| detail == "Password set"));
        assert!(
            text.iter()
                .any(|(_, _, detail)| detail == "This is your last way to sign in.")
        );
    }

    #[test]
    fn methods_refresh_states_do_not_present_retained_data_as_current() {
        let last_successful = kanso_sync::AccountMethods {
            has_password: true,
            rp_id: None,
            available: vec![],
            linked: vec![],
            passkeys: vec![],
        };
        let loading = AccountMethodsState {
            last_successful: Some(last_successful.clone()),
            loading: true,
            ..AccountMethodsState::default()
        };
        let failed = AccountMethodsState {
            last_successful: Some(last_successful),
            failed: Some("Offline".into()),
            ..AccountMethodsState::default()
        };
        for (state, status) in [
            (loading, "Refreshing…"),
            (failed, "Couldn't refresh. Offline"),
        ] {
            let lines = sign_in_method_lines(&state, |_| "unused".into());
            assert_eq!(lines.len(), 1);
            assert_eq!(lines[0].2, status);
            assert!(!lines.iter().any(|(_, title, _)| title == "Password"));
            assert!(
                !lines
                    .iter()
                    .any(|(_, _, detail)| detail.contains("last way"))
            );
        }
    }

    #[test]
    fn signed_out_methods_match_runnable_health_and_old_server_fallback() {
        let (password, providers) = runnable_methods(None);
        assert!(password, "old servers retain password sign in");
        assert!(providers.is_empty());

        let mut auth = kanso_sync::ServerAuth {
            password: false,
            ..Default::default()
        };
        assert!(!runnable_methods(Some(&auth)).0, "passwordless server");
        auth.providers = vec![
            kanso_sync::Provider::Github,
            kanso_sync::Provider::Apple,
            kanso_sync::Provider::Google,
        ];
        let methods = runnable_methods(Some(&auth)).1;
        // Apple too, on every platform, in the server's order.
        assert_eq!(methods, auth.providers, "preserve server method ordering");
    }

    #[test]
    fn passkey_methods_follow_the_platform_the_health_and_the_attached_keys() {
        let official = "https://sync.kansoapp.app";
        let auth = |rp_id: &str, security_key: bool| kanso_sync::ServerAuth {
            password: true,
            passkeys: Some(kanso_sync::PasskeyMethods {
                rp_id: rp_id.into(),
                security_key,
            }),
            ..Default::default()
        };
        let health = auth("sync.kansoapp.app", true);
        // macOS and Windows: the sheet, when it can run; never a security key.
        for platform in [Platform::MacOs, Platform::Windows] {
            let offer = passkey_offer(platform, Some(&health), official, true, true);
            assert!(offer.platform);
            assert_eq!(offer.security_key, SecurityKeyOffer::None);
            assert!(!passkey_offer(platform, Some(&health), official, false, true).platform);
        }
        // A self-hosted domain isn't in the Mac app's entitlement.
        let own = auth("sync.example.com", true);
        let own_server = "https://sync.example.com";
        assert!(!passkey_offer(Platform::MacOs, Some(&own), own_server, true, false).platform);
        assert!(passkey_offer(Platform::Windows, Some(&own), own_server, true, false).platform);
        // Linux: a security key, or the line asking for one.
        let linux = passkey_offer(Platform::Linux, Some(&own), own_server, true, true);
        assert!(!linux.platform);
        assert_eq!(linux.security_key, SecurityKeyOffer::Ready);
        assert_eq!(
            passkey_offer(Platform::Linux, Some(&own), own_server, true, false).security_key,
            SecurityKeyOffer::Connect
        );
        assert_eq!(
            passkey_offer(
                Platform::Linux,
                Some(&auth("sync.example.com", false)),
                own_server,
                true,
                true
            )
            .security_key,
            SecurityKeyOffer::None
        );
        // Nothing advertised, or an old server: no passkey methods.
        let none = kanso_sync::ServerAuth::default();
        for platform in [Platform::MacOs, Platform::Windows, Platform::Linux] {
            assert_eq!(
                passkey_offer(platform, Some(&none), official, true, true),
                PasskeyOffer::default()
            );
            assert_eq!(
                passkey_offer(platform, None, official, true, true),
                PasskeyOffer::default()
            );
        }
    }

    #[test]
    fn provider_ticket_completion_selects_create_form_from_signin_or_signup() {
        for entry in [SignedOutForm::SignIn, SignedOutForm::CreateAccount] {
            let resulting = super::completed_form(entry, true);
            assert_eq!(resulting, SignedOutForm::CreateAccount);
        }
    }

    /// The key's file is written whole and readable only by its owner,
    /// even where a file with looser permissions was.
    #[test]
    fn the_key_file_is_private() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Kanso recovery key.txt");
        std::fs::write(&path, "old and longer than the new text").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        write_key_file(&path, "new").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Signed in, no signed-out form shows; an ended session always
        /// shows Sign in or the recovery form; otherwise the chosen one.
        #[test]
        fn prop_forms_follow_the_account(account in strategies::account(), chosen in prop::sample::select(vec![
            SignedOutForm::None, SignedOutForm::CreateAccount, SignedOutForm::SignIn, SignedOutForm::Recover,
        ])) {
            let form = form_for(&account, chosen);
            match account {
                AccountState::SignedIn { .. } | AccountState::SignedInLocked { .. } => {
                    prop_assert_eq!(form, SignedOutForm::None)
                },
                AccountState::SessionEnded { .. } => {
                    prop_assert!(matches!(form, SignedOutForm::SignIn | SignedOutForm::Recover));
                    prop_assert_eq!(form == SignedOutForm::Recover, chosen == SignedOutForm::Recover);
                }
                _ => prop_assert_eq!(form, chosen),
            }
        }

        /// A device's line names this Mac, or when the other was active.
        #[test]
        fn prop_device_lines_name_the_mac_or_the_time(current: bool, at in 0i64..1_000_000) {
            let line = device_detail(&device(current, at), |at| format!("t{at}"));
            if current {
                prop_assert_eq!(line, "This device");
            } else {
                prop_assert_eq!(line, format!("Last active t{at}"));
            }
        }

        /// The fingerprint's lines keep every group, in order, at most
        /// three to a line.
        #[test]
        fn prop_fingerprint_lines_keep_every_group(groups in prop::collection::vec("[0-9]{5}", 0..9)) {
            let fingerprint = groups.join(" ");
            let lines = fingerprint_lines(&fingerprint);
            prop_assert!(lines.iter().all(|l| l.split(' ').count() <= 3));
            prop_assert_eq!(lines.join(" "), fingerprint);
        }
    }

    #[test]
    fn the_fingerprint_shows_in_two_lines_of_three() {
        assert_eq!(
            fingerprint_lines("01234 56789 00000 11111 22222 33333"),
            ["01234 56789 00000", "11111 22222 33333"]
        );
        // One plain sentence.
        assert_eq!(FINGERPRINT_SENTENCE.matches(". ").count(), 0);
        assert!(FINGERPRINT_SENTENCE.ends_with('.'));
    }
}
