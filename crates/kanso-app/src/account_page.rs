//! Settings › Account (docs/ACCOUNTS.md › Account page): signed out, the
//! Create account, Sign in and recovery forms; after sign-up, recovery or a
//! new key, the recovery key screen; signed in, the account, its storage,
//! sync, devices and security, and signing out or deleting the account.
//!
//! The page draws from [`Services`] alone (the account's name, the pending
//! key, the fetched details), so it shows the same with or without a
//! session loaded. The tasks themselves are in `account_flow.rs`.

use std::path::Path;

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
use crate::services::{
    self, AccountDetails, AccountState, PendingKey, Services, SyncState, TaskKind,
};
use crate::settings::{Row, SYNC_INTERVALS, interval_label, section, status_row, sync_status_line};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// How wide the forms' fields are, beside their labels.
const FIELD_WIDTH: f32 = 280.;

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
        AccountState::SignedIn { .. } => SignedOutForm::None,
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
        let username = input(false, "ngoni", cx);
        let password = input(true, "", cx);
        let confirm = input(true, "", cx);
        let recovery_key = input(false, "XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX", cx);
        let subscriptions = [&username, &password, &confirm, &recovery_key]
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
            error: None,
            waiting: false,
            notes: 0,
            key_saved: false,
            prefilled: None,
            _subscriptions: subscriptions,
            _listening: None,
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
        }
        if self.form == SignedOutForm::CreateAccount {
            account_flow::check_signup(cx);
        }
    }

    #[cfg(test)]
    pub fn form(&self) -> SignedOutForm {
        self.form
    }

    /// Shows `form` from empty, the keyboard in its first field.
    pub fn open_form(&mut self, form: SignedOutForm, window: &mut Window, cx: &mut Context<Self>) {
        if self.waiting {
            return;
        }
        self.form = form;
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
        if form != SignedOutForm::None {
            self.username
                .update(cx, |input, cx| input.focus(window, cx));
        }
        cx.notify();
    }

    fn clear_secrets(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for input in [&self.password, &self.confirm, &self.recovery_key] {
            input.update(cx, |input, cx| input.set_value("", window, cx));
        }
    }

    fn value(input: &Entity<InputState>, cx: &App) -> String {
        input.read(cx).value().to_string()
    }

    /// Presses the form's button.
    pub fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let services = cx.global::<Services>();
        if self.waiting || services.task.is_some() {
            return;
        }
        let form = form_for(&services.account, self.form);
        let username = Self::value(&self.username, cx);
        let password = Self::value(&self.password, cx);
        let confirm = Self::value(&self.confirm, cx);
        let key = Self::value(&self.recovery_key, cx);
        let server = account_flow::form_server(cx);
        let handle = window.window_handle();
        let started = match form {
            SignedOutForm::None => return,
            SignedOutForm::CreateAccount => check_sign_up(&username, &password, &confirm)
                .map(|(name, password)| account_flow::sign_up(server, name, password, handle, cx)),
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
                        if cx.global::<Services>().signed_in() {
                            this.form = SignedOutForm::None;
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
            (None, None, _) => self.signed_out_page(&account, cx),
        };
        v_flex().w_full().children(body)
    }
}

impl AccountPage {
    fn tooltip(cx: &App) -> String {
        cx.try_global::<Services>()
            .map(|services| services.sync_status(cx))
            .unwrap_or_default()
    }

    /// One of a form's rows: a label, a hint, and the field on the right.
    fn field_row(
        &self,
        icon: Lucide,
        label: &'static str,
        hint: Option<&'static str>,
        input: &Entity<InputState>,
        selector: &'static str,
        disabled: bool,
    ) -> Row {
        let row = Row::new(icon, label);
        let row = match hint {
            Some(hint) => row.detail(hint),
            None => row,
        };
        row.trailing(
            div()
                .w(px(FIELD_WIDTH))
                .debug_selector(move || selector.into())
                .child(
                    Input::new(input)
                        .disabled(disabled)
                        .when(self.is_password(input), |input| input.mask_toggle()),
                ),
        )
    }

    /// Whether `input` is one of the password fields, which can be shown.
    fn is_password(&self, input: &Entity<InputState>) -> bool {
        [&self.password, &self.confirm]
            .iter()
            .any(|field| field.entity_id() == input.entity_id())
    }

    /// The Server row: the forms' server, and the way to change it.
    fn server_row(&self, disabled: bool, cx: &App) -> Row {
        let server = cx.global::<AppState>().config.sync.server.clone();
        let custom = server.is_some();
        let mut controls = vec![
            Button::new("use-different-server")
                .outline()
                .small()
                .label("Use a different server…")
                .disabled(disabled)
                .debug_selector(|| "use-different-server".into())
                .on_click(|_, window, cx| {
                    server_dialog(window, cx);
                })
                .into_any_element(),
        ];
        if custom {
            controls.push(
                Button::new("use-default-server")
                    .ghost()
                    .small()
                    .label("Use the default server")
                    .disabled(disabled)
                    .on_click(|_, _, cx| account_flow::set_server(None, cx))
                    .into_any_element(),
            );
        }
        Row::new(Lucide::Server, "Server")
            .value(server_label(server.as_deref()))
            .trailing_all(controls)
    }

    fn signed_out_page(
        &mut self,
        account: &AccountState,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let colors = Colors::get(cx);
        let services = cx.global::<Services>();
        let task = services.task;
        let signup = services.signup.clone();
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
        let mut page = vec![status_row(&status, Self::tooltip(cx), None, colors)];
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
        let server = account_flow::form_server(cx);
        let signup = signup
            .filter(|(asked, _)| *asked == server)
            .map(|(_, signup)| signup);
        let (title, rows, submit) = match form {
            SignedOutForm::CreateAccount => (
                "Create account",
                vec![
                    self.field_row(
                        Lucide::User,
                        "Username",
                        Some("3–32 lowercase letters, digits, - or _"),
                        &self.username,
                        "account-username",
                        busy,
                    ),
                    self.field_row(
                        Lucide::KeyRound,
                        "Password",
                        Some("At least 12 characters. A few random words work well."),
                        &self.password,
                        "account-password",
                        busy,
                    ),
                    self.field_row(
                        Lucide::KeyRound,
                        "Confirm password",
                        None,
                        &self.confirm,
                        "account-confirm",
                        busy,
                    ),
                    self.server_row(busy, cx),
                ],
                "Create account",
            ),
            SignedOutForm::SignIn => (
                "Sign in",
                vec![
                    self.field_row(
                        Lucide::User,
                        "Username",
                        None,
                        &self.username,
                        "account-username",
                        busy,
                    ),
                    self.field_row(
                        Lucide::KeyRound,
                        "Password",
                        None,
                        &self.password,
                        "account-password",
                        busy,
                    ),
                    self.server_row(busy, cx),
                ],
                "Sign in",
            ),
            SignedOutForm::Recover | SignedOutForm::None => (
                "Forgot your password?",
                vec![
                    self.field_row(
                        Lucide::User,
                        "Username",
                        None,
                        &self.username,
                        "account-username",
                        busy,
                    ),
                    self.field_row(
                        Lucide::FileKey,
                        "Recovery key",
                        Some("The key Kanso gave you when you made the account."),
                        &self.recovery_key,
                        "account-recovery-key",
                        busy,
                    ),
                    self.field_row(
                        Lucide::KeyRound,
                        "New password",
                        Some("At least 12 characters. A few random words work well."),
                        &self.password,
                        "account-password",
                        busy,
                    ),
                    self.field_row(
                        Lucide::KeyRound,
                        "Confirm",
                        None,
                        &self.confirm,
                        "account-confirm",
                        busy,
                    ),
                    self.server_row(busy, cx),
                ],
                "Set new password",
            ),
        };
        page.push(section(title, rows, colors));

        // Under the form: what will happen, the error, and the buttons.
        // Only a full server blocks the form: "closed" may hide a reservation.
        let closed = matches!(signup, Some(SignupState::Full));
        let line = match form {
            SignedOutForm::CreateAccount => Some(sign_up_line(self.notes, signup)),
            SignedOutForm::Recover => Some(
                "The recovery key you use stops working; Kanso gives you a new one.".to_owned(),
            ),
            _ => None,
        };
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
        let footer = h_flex()
            .mt(px(14.))
            .gap_2()
            .items_center()
            .child(
                // A row, so the link keeps its own width at the left.
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .when(form == SignedOutForm::SignIn, |el| {
                        el.child(
                            Button::new("account-forgot")
                                .ghost()
                                .small()
                                .label("Forgot your password?")
                                .disabled(busy)
                                .debug_selector(|| "account-forgot".into())
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.open_form(SignedOutForm::Recover, window, cx)
                                })),
                        )
                    }),
            )
            .when(!(ended && form == SignedOutForm::SignIn), |el| {
                el.child(
                    Button::new("account-cancel")
                        .label("Cancel")
                        .disabled(busy)
                        .debug_selector(|| "account-cancel".into())
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.open_form(SignedOutForm::None, window, cx)
                        })),
                )
            })
            .child(
                Button::new("account-submit")
                    .primary()
                    .label(label)
                    // The spinner takes the icon's place; a loading button
                    // ignores clicks, and keeps its colors so the step reads.
                    .when(spinning, |button| button.icon(Lucide::RefreshCcw))
                    .loading(spinning)
                    .disabled(!ready && !spinning)
                    .debug_selector(|| "account-submit".into())
                    .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
            );
        page.push(
            v_flex()
                .mt(px(14.))
                .gap(px(8.))
                .when_some(line, |el, line| {
                    el.child(
                        div()
                            .text_size(px(13.))
                            .line_height(px(18.))
                            .text_color(colors.text_secondary)
                            .child(line),
                    )
                })
                .when_some(self.error.clone(), |el, error| {
                    el.child(
                        div()
                            .debug_selector(|| "account-error".into())
                            .text_size(px(13.))
                            .line_height(px(18.))
                            .text_color(colors.danger)
                            .child(error),
                    )
                })
                .child(footer)
                .into_any_element(),
        );
        page
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
            section("Devices", devices, colors),
            section(
                "Security",
                vec![
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
                        ),
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
                        .detail("Set a new one with your recovery key.")
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
                ],
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

/// The Storage row's line, or why it isn't there yet.
pub fn storage_detail(details: &AccountDetails) -> String {
    match (&details.usage, &details.failed) {
        (Some(usage), _) => words::storage_line(usage),
        (None, Some(failed)) => failed.clone(),
        (None, None) => "Checking…".to_owned(),
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
    if devices.is_empty() {
        return vec![Row::new(Lucide::Laptop, "No devices").detail("None is signed in.")];
    }
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

pub fn change_password_dialog(window: &mut Window, cx: &mut App) -> Option<Entity<FormView>> {
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
        move |values, _, cx| match recovery_values(&values, &username) {
            Ok((key, password)) => {
                Submit::Running(account_flow::recover_signed_in(key, password, cx))
            }
            Err(words) => Submit::Invalid(words),
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
        FINGERPRINT_SENTENCE, SignedOutForm, device_detail, fingerprint_lines, form_for,
        storage_detail, write_key_file,
    };
    use crate::services::{AccountDetails, AccountState, strategies};
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
                AccountState::SignedIn { .. } => prop_assert_eq!(form, SignedOutForm::None),
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
