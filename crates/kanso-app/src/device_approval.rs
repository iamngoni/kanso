//! Approving a new device from one already signed in
//! (docs/ACCOUNTS.md › Unlocking a new device, › App: Device approval).
//!
//! An unlocked device keeps the account event socket open, so a request
//! shows without a timer: an event is only a hint, and every connect, event,
//! launch and Settings open does one approvals GET. The sheet shows the
//! requesting device's name, when it asked, and the six-digit code both
//! screens compare. The approver checks the revealed key against the
//! commitment itself before it shows anything, because it doesn't trust the
//! server.
//!
//! The new device's waiting page shows its own code and offers the recovery
//! key after two minutes, or at once when no other device is signed in. The
//! account flow that polls and unlocks lives in `account_flow`.

// Step 2's locked sign-in is the caller of the waiting flow; until its branch
// merges, the page and flow are reachable but have no producer of a
// `LockedSession`. Keep them compiled and tested here.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_sync::{
    AccountError, ApprovalKeys, Machine, OpenedRequest, PendingApproval, Session,
    account_events_path, challenge_request, decline_request,
    events::{self, EventError, EventSource, Signal},
    finish_approval, open_request, pending_approvals,
};

use crate::account_words as words;
use crate::clock::Clock;
use crate::services::Services;
use crate::theme::Colors;
use crate::zoom::px;

/// The sheet's width.
pub const SHEET_WIDTH: f32 = 520.;
/// How long the new device waits before it offers the recovery key.
pub const RECOVERY_AFTER_MS: i64 = 120_000;
/// How often the waiting device asks its request.
pub const POLL_EVERY: Duration = Duration::from_secs(2);

/// Where the new device's request is, for its waiting page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaitPhase {
    /// Asking the account's other devices.
    Requesting,
    /// The request is waiting for a device to answer.
    Waiting,
    /// A device answered: show the code and post the sealed name.
    Compare {
        code: String,
    },
    /// The data key is on its way.
    Unlocking,
    /// A device approved: the key arrived.
    Approved,
    /// No other device is signed in; the recovery key is the only way.
    NoDevices,
    Cancelled,
    Expired,
    Failed(String),
}

/// The waiting page's state. `started_ms` is when the request was made.
#[derive(Clone, Debug)]
pub struct WaitingState {
    pub phase: WaitPhase,
    pub code: Option<String>,
    pub started_ms: i64,
    pub other_devices: usize,
    /// False until the create answer says how many other devices could answer.
    pub count_known: bool,
    pub recovery_offered: bool,
    /// The server's request ID, once it exists.
    pub request_id: Option<String>,
    /// Set by the waiting page's Cancel; the polling thread asks the server
    /// to cancel and stops.
    pub cancel: Arc<AtomicBool>,
}

impl WaitingState {
    pub fn new(other_devices: usize, started_ms: i64) -> WaitingState {
        WaitingState {
            phase: WaitPhase::Requesting,
            code: None,
            started_ms,
            other_devices,
            count_known: true,
            recovery_offered: other_devices == 0,
            request_id: None,
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    /// The state while the create request is in flight. The count is not
    /// known yet, so recovery is never offered early; `know_devices` settles
    /// it when the create answer arrives.
    pub fn requesting(started_ms: i64) -> WaitingState {
        let mut state = WaitingState::new(0, started_ms);
        state.count_known = false;
        state.recovery_offered = false;
        state
    }

    /// The create answer's count. With no other device the recovery key is
    /// offered at once; otherwise only after two minutes.
    pub fn know_devices(&mut self, other_devices: usize, now_ms: i64) {
        self.other_devices = other_devices;
        self.count_known = true;
        let elapsed = now_ms.saturating_sub(self.started_ms);
        self.recovery_offered = other_devices == 0 || elapsed >= RECOVERY_AFTER_MS;
    }

    /// The recovery key is offered at once with no other device, otherwise
    /// after two minutes. Before the count is known, nothing is offered.
    pub fn offer_recovery(&mut self, now_ms: i64) {
        if !self.count_known {
            return;
        }
        let elapsed = now_ms.saturating_sub(self.started_ms);
        self.recovery_offered |= self.other_devices == 0 || elapsed >= RECOVERY_AFTER_MS;
    }
}

/// What the waiting page says above the code.
pub fn waiting_line(state: &WaitingState) -> &'static str {
    match &state.phase {
        WaitPhase::Requesting => "Asking your other devices…",
        WaitPhase::Waiting => "Waiting for a device you're signed in on to approve this one.",
        WaitPhase::Compare { .. } => {
            "Does the other device show this code? Only approve there if it does."
        }
        WaitPhase::Unlocking => "Approved. Unlocking this Mac…",
        WaitPhase::Approved => "This Mac is unlocked.",
        WaitPhase::NoDevices => {
            "No other device is signed in. Use your recovery key to unlock this Mac."
        }
        WaitPhase::Cancelled => "That request was cancelled. Ask again, or use your recovery key.",
        WaitPhase::Expired => "That request expired. Ask again, or use your recovery key.",
        WaitPhase::Failed(_) => words::CANT_REACH,
    }
}

/// How a request's name reads before the sealed name arrives.
pub fn request_title(_pending: &PendingApproval, opened_name: Option<&str>) -> String {
    match opened_name {
        Some(name) if !name.trim().is_empty() => name.to_owned(),
        _ => "A device".to_owned(),
    }
}

// ---------------------------------------------------------------------------
// The approver's sheet
// ---------------------------------------------------------------------------

/// One open approver sheet: the account's pending requests, this device's
/// ephemeral key per request, and the opened code and name once the
/// requester has revealed its key.
pub struct ApprovalSheet {
    session: Arc<Session>,
    pub(crate) pending: Vec<PendingApproval>,
    keys: BTreeMap<String, ApprovalKeys>,
    opened: BTreeMap<String, OpenedRequest>,
    /// Requests this sheet has already challenged, so a reload doesn't ask
    /// again.
    challenged: BTreeSet<String>,
    pub(crate) busy: bool,
    pub(crate) error: Option<SharedString>,
}

impl ApprovalSheet {
    pub fn new(session: Arc<Session>, cx: &mut Context<Self>) -> ApprovalSheet {
        let mut sheet = ApprovalSheet {
            session,
            pending: Vec::new(),
            keys: BTreeMap::new(),
            opened: BTreeMap::new(),
            challenged: BTreeSet::new(),
            busy: false,
            error: None,
        };
        sheet.reload(cx);
        sheet
    }

    pub fn is_busy(&self) -> bool {
        self.busy
    }

    /// Reads the pending requests again. It runs on launch, when Settings ›
    /// Account opens, on foreground, after every socket connect and after
    /// every event; never on a timer.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let session = self.session.clone();
        let Some(backend) = cx
            .try_global::<Services>()
            .and_then(|services| services.backend.clone())
        else {
            return;
        };
        self.busy = true;
        let task = cx
            .background_executor()
            .spawn(async move { pending_approvals(backend.as_ref(), &session) });
        cx.spawn(async move |this, cx| {
            let answer = task.await;
            this.update(cx, |this, cx| this.finish_reload(answer, cx))
                .ok();
        })
        .detach();
        cx.notify();
    }

    fn finish_reload(
        &mut self,
        answer: Result<Vec<PendingApproval>, AccountError>,
        cx: &mut Context<Self>,
    ) {
        self.busy = false;
        match answer {
            Ok(pending) => {
                self.error = None;
                self.pending = pending;
                let ids: Vec<String> = self.pending.iter().map(|p| p.id.clone()).collect();
                self.keys.retain(|id, _| ids.contains(id));
                self.opened.retain(|id, _| ids.contains(id));
                self.challenged.retain(|id| ids.contains(id));
                let mut opened_now = Vec::new();
                let mut error = None;
                // Only the oldest request is on screen. A second one queues
                // behind it; its key is generated when it reaches the front.
                if let Some(pending) = self.pending.first().cloned() {
                    let keys = self
                        .keys
                        .entry(pending.id.clone())
                        .or_insert_with(ApprovalKeys::generate);
                    // If this device already challenged, its key is the one
                    // stored; otherwise the name hasn't arrived yet.
                    let (Some(_epk), Some(sealed)) = (pending.epk, pending.sealed_name.as_deref())
                    else {
                        if pending.approver_epk.is_none() && !self.challenged.contains(&pending.id)
                        {
                            self.challenged.insert(pending.id.clone());
                            self.challenge(pending.id.clone(), cx);
                        }
                        cx.notify();
                        return;
                    };
                    if !self.opened.contains_key(&pending.id) {
                        if pending
                            .approver_epk
                            .is_some_and(|stored| stored != keys.epk())
                        {
                            error = Some(words::APPROVAL_CHANGED.into());
                        } else {
                            match open_request(&self.session, &pending, keys) {
                                Ok(Some(opened)) => {
                                    if opened.name != sealed
                                        || pending.epk != Some(opened.approval.epk_new())
                                    {
                                        error = Some(words::APPROVAL_CHANGED.into());
                                    } else {
                                        opened_now.push((pending.id.clone(), opened));
                                    }
                                }
                                Ok(None) => {}
                                Err(err) => {
                                    error = Some(
                                        words::error_words(&err, words::ErrorContext::Account)
                                            .into(),
                                    )
                                }
                            }
                        }
                    }
                }
                for (id, opened) in opened_now {
                    self.opened.insert(id, opened);
                }
                if error.is_some() {
                    self.error = error;
                }
                // Seeing a request is the approver's step 2: post this
                // device's key once, then wait for the requester's name.
                let to_challenge: Vec<String> = self
                    .pending
                    .first()
                    .filter(|p| p.approver_epk.is_none() && !self.challenged.contains(&p.id))
                    .map(|p| p.id.clone())
                    .into_iter()
                    .collect();
                for id in to_challenge {
                    self.challenged.insert(id.clone());
                    self.challenge(id, cx);
                }
            }
            Err(err) => {
                self.error = Some(words::error_words(&err, words::ErrorContext::Account).into())
            }
        }
        cx.notify();
    }

    /// Posts this device's key for the request, then reads the name when it
    /// arrives. A repeat is harmless (the server answers 200).
    pub fn challenge(&mut self, id: String, cx: &mut Context<Self>) {
        let Some(pending) = self.pending.iter().find(|p| p.id == id).cloned() else {
            return;
        };
        let keys = self.keys.remove(&id).unwrap_or_else(ApprovalKeys::generate);
        let Some(backend) = cx
            .try_global::<Services>()
            .and_then(|services| services.backend.clone())
        else {
            return;
        };
        let session = self.session.clone();
        self.busy = true;
        let task = cx.background_executor().spawn(async move {
            let answer = challenge_request(backend.as_ref(), &session, &pending, &keys);
            (answer, id, keys)
        });
        cx.spawn(async move |this, cx| {
            let (answer, id, keys) = task.await;
            this.update(cx, |this, cx| {
                this.busy = false;
                this.keys.insert(id, keys);
                if let Err(err) = answer {
                    this.error =
                        Some(words::error_words(&err, words::ErrorContext::Account).into());
                }
                this.reload(cx);
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// Sends the data key sealed for the request.
    pub fn approve(&mut self, id: String, cx: &mut Context<Self>) {
        let Some(pending) = self.pending.iter().find(|p| p.id == id).cloned() else {
            return;
        };
        let Some(opened) = self.opened.remove(&id) else {
            self.challenge(id, cx);
            return;
        };
        let Some(backend) = cx
            .try_global::<Services>()
            .and_then(|services| services.backend.clone())
        else {
            return;
        };
        let session = self.session.clone();
        self.busy = true;
        let task = cx.background_executor().spawn(async move {
            let answer = finish_approval(backend.as_ref(), &session, &pending, &opened.approval);
            (answer, opened)
        });
        cx.spawn(async move |this, cx| {
            let (answer, opened) = task.await;
            this.update(cx, |this, cx| {
                this.busy = false;
                match answer {
                    Ok(()) => {
                        this.pending.retain(|p| p.id != id);
                        this.keys.remove(&id);
                        this.challenged.remove(&id);
                        // The next queued request becomes the sheet's front.
                        this.reload(cx);
                    }
                    Err(err) => {
                        this.opened.insert(id, opened);
                        this.error =
                            Some(words::error_words(&err, words::ErrorContext::Account).into());
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// Declines the request; one that is already gone counts as declined.
    pub fn decline(&mut self, id: String, cx: &mut Context<Self>) {
        let Some(backend) = cx
            .try_global::<Services>()
            .and_then(|services| services.backend.clone())
        else {
            return;
        };
        let session = self.session.clone();
        self.busy = true;
        let asked = id.clone();
        let task = cx
            .background_executor()
            .spawn(async move { decline_request(backend.as_ref(), &session, &asked) });
        cx.spawn(async move |this, cx| {
            let answer = task.await;
            this.update(cx, |this, cx| {
                this.busy = false;
                match answer {
                    Ok(()) => {
                        this.pending.retain(|p| p.id != id);
                        this.opened.remove(&id);
                        this.keys.remove(&id);
                        this.challenged.remove(&id);
                        // The next queued request becomes the sheet's front.
                        this.reload(cx);
                    }
                    Err(err) => {
                        this.error =
                            Some(words::error_words(&err, words::ErrorContext::Account).into());
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    fn render_request(
        &self,
        ix: usize,
        pending: &PendingApproval,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let opened = self.opened.get(&pending.id);
        let name = request_title(pending, opened.map(|o| o.name.as_str()));
        let asked = Clock::edited_label(cx, pending.created_at);
        let code = opened.map(|o| o.approval.code());
        let ready = code.is_some() && !self.busy;
        let id = pending.id.clone();
        let decline = pending.id.clone();
        v_flex()
            .id(("approval", ix))
            .gap(px(8.))
            .py(px(10.))
            .px(px(12.))
            .rounded(px(8.))
            .border_1()
            .border_color(colors.divider)
            .child(
                h_flex()
                    .gap(px(10.))
                    .child(
                        Icon::new(Lucide::MonitorSmartphone)
                            .size(px(18.))
                            .text_color(colors.text_secondary),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(colors.text)
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .text_color(colors.text_secondary)
                                    .child(format!("A device wants to join. Asked at {asked}.")),
                            ),
                    ),
            )
            .when_some(code, |el, code| {
                el.child(
                    div()
                        .text_size(px(28.))
                        .font_family(cx.theme().mono_font_family.clone())
                        .text_color(colors.text)
                        .child(code),
                )
            })
            .child(
                h_flex()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new(("approval-decline", ix))
                            .ghost()
                            .small()
                            .label("Cancel")
                            .disabled(self.busy)
                            .debug_selector(move || format!("approval-decline-{ix}"))
                            .on_click(
                                cx.listener(move |this, _, _, cx| {
                                    this.decline(decline.clone(), cx)
                                }),
                            ),
                    )
                    .child(
                        Button::new(("approval-approve", ix))
                            .primary()
                            .small()
                            .label("Approve")
                            .disabled(!ready)
                            .debug_selector(move || format!("approval-approve-{ix}"))
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.approve(id.clone(), cx)),
                            ),
                    ),
            )
            .into_any_element()
    }
}

impl Render for ApprovalSheet {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        // One request is open at a time. The rest queue behind it, and the
        // sheet advances only after the front is approved or declined.
        let rows: Vec<AnyElement> = self
            .pending
            .first()
            .map(|pending| self.render_request(0, pending, cx))
            .into_iter()
            .collect();
        let queued = self.pending.len().saturating_sub(1);
        v_flex()
            .w_full()
            .gap(px(8.))
            .when(queued > 0, |el| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_secondary)
                        .child(if queued == 1 {
                            "1 more request is waiting.".to_owned()
                        } else {
                            format!("{queued} more requests are waiting.")
                        }),
                )
            })
            .when(rows.is_empty(), |el| {
                el.child(
                    div()
                        .text_size(px(13.))
                        .text_color(colors.text_secondary)
                        .child("No device is waiting. An unlocked device gets a request here when it asks to join."),
                )
            })
            .children(rows)
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .debug_selector(|| "approval-error".into())
                        .text_size(px(13.))
                        .text_color(colors.danger)
                        .child(error),
                )
            })
    }
}

/// Opens the approver sheet, or returns the one already open.
pub fn open_sheet(cx: &mut App) -> Option<Entity<ApprovalSheet>> {
    let services = cx.try_global::<Services>()?;
    if let Some(sheet) = services.approval.sheet.clone() {
        sheet.update(cx, |sheet, cx| {
            sheet.reload(cx);
            cx.notify();
        });
        return Some(sheet);
    }
    let session = services.session()?;
    let sheet = cx.new(|cx| ApprovalSheet::new(session, cx));
    cx.global_mut::<Services>().approval.sheet = Some(sheet.clone());
    let dialog_view = sheet.clone();
    if let Some(window) = cx.active_window() {
        window
            .update(cx, move |_, window, cx| {
                let running = dialog_view.clone();
                window.open_alert_dialog(cx, move |alert, _, _| {
                    let running = running.clone();
                    alert
                        .title("Approve a device")
                        .description("Only approve a device you asked to add. Compare the code on both screens.")
                        .width(px(SHEET_WIDTH))
                        .child(dialog_view.clone())
                        .on_cancel(move |_, _, cx| {
                            if running.read(cx).is_busy() {
                                return false;
                            }
                            cx.global_mut::<Services>().approval.sheet = None;
                            true
                        })
                });
            })
            .ok();
    }
    Some(sheet)
}

/// The new device's waiting page: the six-digit code, and the recovery key
/// once it is offered. Rendered in place of the Account page's usual body.
pub fn waiting_page(state: &WaitingState, _window: &mut Window, cx: &mut App) -> AnyElement {
    let colors = Colors::get(cx);
    let mono = cx.theme().mono_font_family.clone();
    let code = state.code.clone();
    let recovery = state.recovery_offered;
    let failed = matches!(state.phase, WaitPhase::Failed(_));
    let ended = matches!(
        state.phase,
        WaitPhase::Cancelled | WaitPhase::Expired | WaitPhase::NoDevices | WaitPhase::Failed(_)
    );
    let mut page = v_flex()
        .w_full()
        .gap(px(10.))
        .child(
            div()
                .text_size(px(16.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors.text)
                .child("Approve this Mac"),
        )
        .child(
            div()
                .text_size(px(13.5))
                .line_height(px(20.))
                .text_color(colors.text_secondary)
                .child(waiting_line(state)),
        );
    if let Some(code) = code {
        page = page
            .child(
                div()
                    .mt(px(8.))
                    .text_size(px(40.))
                    .font_family(mono)
                    .text_color(colors.text)
                    .debug_selector(|| "approval-code".into())
                    .child(code),
            )
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(colors.text_secondary)
                    .child(
                        "On the other device, open Settings › Account and approve this request.",
                    ),
            );
    }
    page = page.child(
        h_flex()
            .mt(px(10.))
            .gap_2()
            .when(recovery, |el| {
                el.child(
                    Button::new("approval-recover")
                        .primary()
                        .label("Use recovery key")
                        .debug_selector(|| "approval-recover".into())
                        .on_click(move |_, window, cx| {
                            crate::account_page::recover_dialog(window, cx);
                        }),
                )
            })
            .when(ended, |el| {
                el.child(
                    Button::new("approval-close")
                        .outline()
                        .label("Close")
                        .on_click(|_, _, cx| crate::account_flow::dismiss_device_approval(cx)),
                )
            })
            .when(!ended, |el| {
                el.child(
                    Button::new("approval-cancel-wait")
                        .outline()
                        .label("Cancel")
                        .debug_selector(|| "approval-cancel".into())
                        .on_click(|_, _, cx| crate::account_flow::cancel_device_approval(cx)),
                )
            }),
    );
    if failed {
        page = page.child(
            div()
                .text_size(px(13.))
                .text_color(colors.danger)
                .child("The request couldn't be made. Try again, or use your recovery key."),
        );
    }
    page.into_any_element()
}

// ---------------------------------------------------------------------------
// Reading requests and the live socket
// ---------------------------------------------------------------------------

/// Reads the account's pending requests once and opens the sheet when there
/// is one. Runs at launch, Settings open, foreground, every connect and
/// after every event (never on a timer).
pub fn refresh(cx: &mut App) {
    let Some(session) = cx.try_global::<Services>().and_then(Services::session) else {
        return;
    };
    let Some(backend) = cx.try_global::<Services>().and_then(|s| s.backend.clone()) else {
        return;
    };
    let asked = session.clone();
    let task = cx
        .background_executor()
        .spawn(async move { pending_approvals(backend.as_ref(), &asked) });
    cx.spawn(async move |cx| {
        let answer = task.await;
        cx.update(|cx| {
            match answer {
                Ok(pending) => {
                    cx.global_mut::<Services>().approval.pending = pending.clone();
                    if !pending.is_empty() {
                        open_sheet(cx);
                    }
                }
                Err(err) => {
                    if let AccountError::Unauthorized { .. } = err {
                        // The next sync or Account refresh says why; the
                        // event loop stops on its own 401.
                    }
                }
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

/// Opens the account event socket and keeps it open. One socket per device;
/// the server replaces an older one. A connect or an event asks for one
/// approvals GET; a lost frame is caught up by that GET.
pub fn start_events(cx: &mut App) {
    stop_events(cx);
    let store = cx.global::<crate::workspace::AppState>().store.clone();
    let services = cx.global_mut::<Services>();
    let (Some(session), Some(backend)) = (services.session(), services.backend.clone()) else {
        return;
    };
    let machine = services.machine.clone();
    let stop = Arc::new(AtomicBool::new(false));
    services.approval.event_stop = Some(stop.clone());
    let (tx, rx) = async_channel::unbounded::<Signal>();
    let thread_stop = stop.clone();
    let event_session = session.clone();
    cx.background_executor()
        .spawn(async move {
            let machine = machine.unwrap_or_else(Machine::this_mac);
            let Ok(device) = kanso_sync::device_identity(&store, &machine) else {
                return;
            };
            let path = account_events_path(backend.as_ref(), &event_session.server)
                .ok()
                .flatten()
                .unwrap_or_else(|| events::ACCOUNT_EVENTS_PATH.to_owned());
            let mut connect = || -> Result<Box<dyn EventSource>, EventError> {
                backend.events(&event_session, &device, &path)
            };
            let sender = tx.clone();
            let mut on = |signal: Signal| {
                sender.try_send(signal).ok();
            };
            let sleep = |wait: Duration| events::sleep_unless_stopped(&thread_stop, wait);
            events::run(&mut connect, &thread_stop, &mut on, &sleep);
        })
        .detach();
    cx.spawn(async move |cx| {
        while let Ok(signal) = rx.recv().await {
            match signal {
                Signal::Connected | Signal::Event(_) => {
                    cx.update(refresh);
                }
                Signal::Ended(err) => {
                    cx.update(|cx| {
                        cx.global_mut::<Services>().approval.event_stop = None;
                        eprintln!("kanso: the account event socket ended: {err:?}");
                    });
                    break;
                }
            }
        }
    })
    .detach();
}

/// Stops the event socket (sign-out, a new session, or shutdown).
pub fn stop_events(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    if let Some(stop) = services.approval.event_stop.take() {
        stop.store(true, Ordering::SeqCst);
    }
    services.approval.sheet = None;
    services.approval.pending.clear();
}

#[cfg(test)]
mod tests {
    use super::{RECOVERY_AFTER_MS, WaitPhase, WaitingState, waiting_line};
    use crate::account_words as words;

    #[test]
    fn recovery_is_offered_at_once_with_no_device_and_after_two_minutes() {
        let mut state = WaitingState::new(0, 1_000);
        assert!(state.recovery_offered);
        state.offer_recovery(1_000);
        assert!(state.recovery_offered);

        let mut state = WaitingState::new(1, 1_000);
        state.offer_recovery(1_000 + RECOVERY_AFTER_MS - 1);
        assert!(!state.recovery_offered);
        state.offer_recovery(1_000 + RECOVERY_AFTER_MS);
        assert!(state.recovery_offered);
    }

    #[test]
    fn recovery_waits_for_the_create_answer_before_it_can_be_offered() {
        let mut state = WaitingState::requesting(1_000);
        assert!(!state.count_known);
        assert!(!state.recovery_offered);
        state.offer_recovery(1_000 + RECOVERY_AFTER_MS);
        assert!(!state.recovery_offered);
        state.know_devices(1, 1_000);
        assert!(state.count_known);
        assert!(!state.recovery_offered);
        state.offer_recovery(1_000 + RECOVERY_AFTER_MS);
        assert!(state.recovery_offered);

        let mut state = WaitingState::requesting(1_000);
        state.know_devices(0, 1_000);
        assert!(state.recovery_offered);
    }

    #[test]
    fn the_waiting_page_says_what_is_happening() {
        let mut state = WaitingState::new(1, 0);
        assert_eq!(waiting_line(&state), "Asking your other devices…");
        state.phase = WaitPhase::Compare {
            code: "123 456".into(),
        };
        assert!(waiting_line(&state).starts_with("Does the other device"));
        state.phase = WaitPhase::NoDevices;
        assert!(waiting_line(&state).contains("recovery key"));
        state.phase = WaitPhase::Failed("x".into());
        assert_eq!(waiting_line(&state), words::CANT_REACH);
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(64))]

        /// The recovery fallback depends only on the count and elapsed time.
        #[test]
        fn prop_recovery_offer_follows_time_and_count(
            other in 0usize..=9,
            started in 0i64..2_000_000,
            elapsed in -10_000i64..400_000,
        ) {
            let mut state = WaitingState::new(other, started);
            state.recovery_offered = other == 0;
            state.offer_recovery(started + elapsed);
            let expected = other == 0 || elapsed >= RECOVERY_AFTER_MS;
            proptest::prop_assert_eq!(state.recovery_offered, expected);
        }
    }
}
