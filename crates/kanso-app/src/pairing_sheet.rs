//! The pairing sheet (issue #390, step 13): an app on this Mac asked the
//! local API for a key, so Kanso comes to the front and asks a person.
//!
//! The server holds one request at a time (`kanso_server::Pairing`) and
//! tells the app when one is accepted. The sheet shows what the app calls
//! itself and the code to compare with the one the app shows, and answers
//! Allow or Don't Allow. It closes by itself when the request expires or is
//! answered some other way. Escape closes it without answering: the request
//! then waits out its two minutes and is refused.

use std::time::Duration;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_server::{PairError, Pairing, PendingPair};

use crate::pairing_model::{
    CAN, CANNOT, CONTESTED, LOCKED, code_label, compare, deny_is_default, intro, notebooks_line,
    title,
};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// The sheet's width.
pub const SHEET_WIDTH: f32 = 520.;

/// How often an open sheet looks at whether its request is still waiting.
const WATCH_EVERY: Duration = Duration::from_secs(1);

/// The sheet that is open, if any, so a second request is shown in it
/// rather than under it.
#[derive(Default)]
struct Open(Option<WeakEntity<PairSheet>>);

impl Global for Open {}

/// What the sheet shows, and what it answers.
pub struct PairSheet {
    pairing: Pairing,
    pending: PendingPair,
    /// Notebook names, for the notebooks it asked to read.
    book_names: std::collections::HashMap<String, String>,
    error: Option<String>,
    _watch: Option<Task<()>>,
}

impl PairSheet {
    fn new(
        pairing: Pairing,
        pending: PendingPair,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let book_names = cx
            .global::<AppState>()
            .store
            .books()
            .map(|books| books.into_iter().map(|b| (b.id, b.name)).collect())
            .unwrap_or_default();
        // A test moves the keys' clock and calls `sync` itself.
        let watch = cfg!(not(test)).then(|| {
            cx.spawn_in(window, async move |this, cx| {
                loop {
                    cx.background_executor().timer(WATCH_EVERY).await;
                    if this
                        .update_in(cx, |this, window, cx| this.sync(window, cx))
                        .is_err()
                    {
                        break;
                    }
                }
            })
        });
        PairSheet {
            pairing,
            pending,
            book_names,
            error: None,
            _watch: watch,
        }
    }

    /// Looks at the request again: the sheet closes when nothing waits,
    /// and shows the new request when another took the place of its own.
    pub fn sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.pairing.pending() {
            None => window.close_dialog(cx),
            Some(pending) => {
                if pending != self.pending {
                    self.pending = pending;
                    self.error = None;
                    cx.notify();
                }
            }
        }
    }

    /// Allow: makes the key, which the app that asked fetches.
    fn allow(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.pairing.approve(&self.pending.request) {
            Ok(_) => window.close_dialog(cx),
            // It ended while the sheet was open: nothing is left to answer.
            Err(PairError::NotPending) => window.close_dialog(cx),
            Err(err) => {
                self.error = Some(err.to_string());
                cx.notify();
            }
        }
    }

    /// Don't Allow: the app that asked is told no.
    fn deny(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.pairing.deny(&self.pending.request);
        window.close_dialog(cx);
    }

    /// What the sheet is for, for tests.
    #[cfg(test)]
    pub(crate) fn deny_is_main(&self) -> bool {
        deny_is_default(&self.pending)
    }

    #[cfg(test)]
    pub(crate) fn request(&self) -> &PendingPair {
        &self.pending
    }
}

// no-prop-test: drawing; `pairing_model` is tested and the sheet's
// buttons are driven by a window test.
impl Render for PairSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let mono = crate::theme::mono_font(cx);
        let line = |text: String| {
            div()
                .text_size(px(13.))
                .line_height(px(19.))
                .text_color(colors.text)
                .child(text)
        };
        let notebooks = notebooks_line(&self.pending, |id| self.book_names.get(id).cloned());
        v_flex()
            .gap(px(12.))
            .child(line(intro(&self.pending)))
            .child(
                v_flex()
                    .gap(px(4.))
                    .items_start()
                    .p(px(12.))
                    .rounded(px(8.))
                    .bg(colors.fill)
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(colors.text_secondary)
                            .child("Code"),
                    )
                    .child(
                        div()
                            .id("pair-code")
                            .debug_selector(|| "pair-code".into())
                            .text_size(px(30.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors.text)
                            .when_some(mono, |el, family| el.font_family(family))
                            .child(code_label(&self.pending.code)),
                    )
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(colors.text_secondary)
                            .child(compare(&self.pending)),
                    ),
            )
            .child(
                v_flex()
                    .gap(px(2.))
                    .child(line(CAN.to_owned()))
                    .child(line(CANNOT.to_owned())),
            )
            .child(
                v_flex().gap(px(2.)).child(line(notebooks)).child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(LOCKED),
                ),
            )
            .when(self.pending.contested, |el| {
                el.child(
                    div()
                        .id("pair-contested")
                        .debug_selector(|| "pair-contested".into())
                        .p(px(10.))
                        .rounded(px(8.))
                        .bg(colors.fill)
                        .text_size(px(13.))
                        .line_height(px(19.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors.danger)
                        .child(CONTESTED),
                )
            })
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.danger)
                        .child(error),
                )
            })
    }
}

/// Don't Allow and Allow.
struct PairFooter {
    sheet: Entity<PairSheet>,
}

// no-prop-test: drawing.
impl Render for PairFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (deny, allow) = (self.sheet.clone(), self.sheet.clone());
        let deny_first = deny_is_default(&self.sheet.read(cx).pending);
        let deny_button = Button::new("pair-deny")
            .debug_selector(|| "pair-deny".into())
            .label("Don't Allow")
            .on_click(move |_, window, cx| deny.update(cx, |sheet, cx| sheet.deny(window, cx)));
        let allow_button = Button::new("pair-allow")
            .debug_selector(|| "pair-allow".into())
            .label("Allow")
            .on_click(move |_, window, cx| allow.update(cx, |sheet, cx| sheet.allow(window, cx)));
        h_flex()
            .w_full()
            .gap_2()
            .justify_end()
            .child(if deny_first {
                deny_button.primary()
            } else {
                deny_button
            })
            .child(if deny_first {
                allow_button
            } else {
                allow_button.primary()
            })
    }
}

/// Opens the sheet for `pending` in `window`. Returns it, held weakly: the
/// dialog owns it.
pub fn open_sheet(
    pairing: Pairing,
    pending: PendingPair,
    window: &mut Window,
    cx: &mut App,
) -> WeakEntity<PairSheet> {
    let sheet = cx.new(|cx| PairSheet::new(pairing, pending, window, cx));
    let weak = sheet.downgrade();
    let footer = cx.new(|_| PairFooter {
        sheet: sheet.clone(),
    });
    // The title follows the sheet's request, which can change while it is
    // open, so it is read as each frame draws.
    let titled = sheet.clone();
    window.open_alert_dialog(cx, move |alert, _, cx| {
        alert
            .title(title(titled.read(cx).pending_ref()))
            .width(px(SHEET_WIDTH))
            .child(sheet.clone())
            .footer(footer.clone())
    });
    cx.default_global::<Open>().0 = Some(weak.clone());
    weak
}

impl PairSheet {
    fn pending_ref(&self) -> &PendingPair {
        &self.pending
    }
}

/// A request was accepted: brings Kanso forward and shows the sheet. A
/// request that is no longer waiting, or a Kanso with no window open, shows
/// nothing; the request then ends on its own.
pub fn show(pairing: &Pairing, cx: &mut App) {
    let Some(pending) = pairing.pending() else {
        return;
    };
    cx.activate(true);
    if let Some(open) = cx.default_global::<Open>().0.clone()
        && let Some(sheet) = open.upgrade()
    {
        sheet.update(cx, |sheet, cx| {
            sheet.pending = pending;
            sheet.error = None;
            cx.notify();
        });
        return;
    }
    let Some((window, _)) = crate::workspace::main_workspace(cx) else {
        return;
    };
    let pairing = pairing.clone();
    window
        .update(cx, move |_, window, cx| {
            window.activate_window();
            open_sheet(pairing, pending, window, cx);
        })
        .ok();
}

/// Shows the sheet each time the server accepts a request. The task ends
/// with the `Pairing`'s keys, and is dropped to stop it.
pub fn watch(pairing: Pairing, cx: &mut App) -> Task<()> {
    let events = pairing.events();
    cx.spawn(async move |cx| {
        while events.recv().await.is_ok() {
            cx.update(|cx| show(&pairing, cx));
        }
    })
}

/// The open sheet, for tests.
#[cfg(test)]
pub(crate) fn open_for_test(cx: &App) -> Option<Entity<PairSheet>> {
    cx.try_global::<Open>()
        .and_then(|open| open.0.as_ref())
        .and_then(WeakEntity::upgrade)
}

/// The sheet for a request the server does not hold, for the snapshots.
#[cfg(test)]
pub(crate) fn open_sheet_for_snapshot(
    pairing: Pairing,
    pending: PendingPair,
    window: &mut Window,
    cx: &mut App,
) {
    open_sheet(pairing, pending, window, cx);
}
