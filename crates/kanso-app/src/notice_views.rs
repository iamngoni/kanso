//! What update notices look like on the desktop (issue #243):
//!
//! ```text
//! ⚠  Your AI agents and scripts need new keys    1 of 2   [How to fix] [Open Settings] [Dismiss]
//! ```
//!
//! The banner runs across the top of the main window until Dismiss. How to
//! fix opens the whole notice: what changed and numbered steps. What's new
//! lists every notice for its release under "Needs your attention", and the
//! palette's "Update notices" lists those that apply. The rules are in
//! `kanso_core::notices` and the state in `notices`.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::notices::{Notice, NoticeState, count_label};

use crate::notices;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

actions!(kanso, [ShowUpdateNotices]);

/// The sheet's width.
pub const SHEET_WIDTH: f32 = 560.;

fn state(cx: &App) -> NoticeState {
    cx.try_global::<AppState>()
        .map(|state| state.config.notices.clone())
        .unwrap_or_default()
}

/// The banner across the top of the main window, or nothing.
// no-prop-test: drawing and clicks; `kanso_core::notices::banner` decides
// what shows, and the snapshots show the banner.
pub fn strip(cx: &App) -> Option<AnyElement> {
    let pick = notices::banner(cx)?;
    let notice = pick.notice;
    let colors = Colors::get(cx);
    let under_traffic_lights = cfg!(target_os = "macos") && crate::recording::notice(cx).is_none();
    let id = notice.id;
    Some(
        h_flex()
            .id("notice-banner")
            .debug_selector(|| "notice-banner".into())
            .flex_none()
            .w_full()
            .h(px(36.))
            .pl(px(if under_traffic_lights { 84. } else { 12. }))
            .pr(px(12.))
            .gap(px(10.))
            .items_center()
            .bg(colors.fill)
            .border_b_1()
            .border_color(colors.divider)
            .text_size(px(12.5))
            .child(
                Icon::new(Lucide::TriangleAlert)
                    .size(px(14.))
                    .flex_none()
                    .text_color(colors.warning),
            )
            .child(
                div()
                    .debug_selector(|| "notice-title".into())
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .font_medium()
                    .text_color(colors.text)
                    .child(notice.title),
            )
            .children(count_label(pick.total).map(|label| {
                div()
                    .debug_selector(|| "notice-count".into())
                    .flex_none()
                    .text_color(colors.text_muted)
                    .child(label)
            }))
            .child(
                Button::new("notice-how-to-fix")
                    .debug_selector(|| "notice-how-to-fix".into())
                    .flex_none()
                    .xsmall()
                    .label("How to fix")
                    .on_click(move |_, window, cx| open_by_id(id, window, cx)),
            )
            .children(notice.action.map(|action| {
                Button::new("notice-action")
                    .debug_selector(|| "notice-action".into())
                    .flex_none()
                    .xsmall()
                    .label(notices::action_label(action))
                    .on_click(move |_, window, cx| notices::run_action(action, window, cx))
            }))
            .child(
                Button::new("notice-dismiss")
                    .debug_selector(|| "notice-dismiss".into())
                    .flex_none()
                    .ghost()
                    .xsmall()
                    .label("Dismiss")
                    .tooltip("Hide this on this computer. It stays in What's new.")
                    .on_click(move |_, _, cx| notices::dismiss(id, cx)),
            )
            .into_any_element(),
    )
}

fn open_by_id(id: &str, window: &mut Window, cx: &mut App) {
    if let Some(notice) = notices::find(id, cx) {
        open_how_to_fix(notice, window, cx);
    }
}

/// The full notice: what changed, why, and the steps.
// no-prop-test: a sheet; the snapshots show it.
pub fn open_how_to_fix(notice: &'static Notice, window: &mut Window, cx: &mut App) {
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let colors = Colors::get(cx);
        let state = state(cx);
        let fixed = state.is_fixed(notice.id);
        let hidden = !state.banner_allowed(notice.id);
        let steps = notice.steps.iter().enumerate().map(|(ix, step)| {
            h_flex()
                .gap(px(10.))
                .items_start()
                .child(
                    div()
                        .flex_none()
                        .w(px(20.))
                        .h(px(20.))
                        .rounded_full()
                        .bg(colors.fill)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(11.5))
                        .font_medium()
                        .text_color(colors.text_secondary)
                        .child((ix + 1).to_string()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .pt(px(1.))
                        .text_size(px(13.5))
                        .line_height(px(19.))
                        .text_color(colors.text)
                        .child(*step),
                )
        });
        let content = v_flex()
            .gap(px(14.))
            .child(
                div()
                    .debug_selector(|| "notice-what-changed".into())
                    .text_size(px(13.5))
                    .line_height(px(20.))
                    .text_color(colors.text_secondary)
                    .child(notice.what_changed),
            )
            .child(
                v_flex()
                    .debug_selector(|| "notice-steps".into())
                    .gap(px(10.))
                    .children(steps),
            )
            .when(fixed, |el| {
                el.child(
                    h_flex()
                        .gap(px(6.))
                        .items_center()
                        .text_size(px(12.5))
                        .text_color(colors.success)
                        .child(Icon::new(Lucide::Check).size(px(13.)))
                        .child("Marked as fixed"),
                )
            });
        let footer = h_flex()
            .w_full()
            .justify_end()
            .gap(px(8.))
            .when(!hidden, |el| {
                el.child(
                    Button::new("notice-sheet-dismiss")
                        .label("Dismiss")
                        .ghost()
                        .on_click(move |_, window, cx| {
                            window.close_dialog(cx);
                            notices::dismiss(notice.id, cx);
                        }),
                )
            })
            .when(!fixed, |el| {
                el.child(
                    Button::new("notice-sheet-fixed")
                        .label("Mark as fixed")
                        .on_click(move |_, window, cx| {
                            window.close_dialog(cx);
                            notices::mark_fixed(notice.id, cx);
                        }),
                )
            })
            .children(notice.action.map(|action| {
                let primary = Button::new("notice-sheet-action")
                    .label(notices::action_label(action))
                    .primary();
                primary.on_click(move |_, window, cx| {
                    window.close_dialog(cx);
                    notices::run_action(action, window, cx);
                })
            }))
            .when(notice.action.is_none() && fixed, |el| {
                el.child(
                    Button::new("notice-sheet-done")
                        .label("Done")
                        .primary()
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                )
            });
        alert
            .title(notice.title)
            .width(px(SHEET_WIDTH))
            .child(content)
            .footer(footer)
    });
}

/// "Needs your attention" for the What's new sheet: a row for each notice
/// of this release, dismissed ones included. Nothing when there are none.
// no-prop-test: drawing; `kanso_core::notices::for_whats_new` decides.
pub fn attention_section(cx: &App) -> Option<AnyElement> {
    let found = notices::for_whats_new(cx);
    if found.is_empty() {
        return None;
    }
    let colors = Colors::get(cx);
    let state = state(cx);
    Some(
        v_flex()
            .id("needs-attention")
            .debug_selector(|| "needs-attention".into())
            .gap(px(8.))
            .child(
                h_flex()
                    .gap(px(6.))
                    .items_center()
                    .child(
                        Icon::new(Lucide::TriangleAlert)
                            .size(px(14.))
                            .text_color(colors.warning),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_semibold()
                            .text_color(colors.text)
                            .child("Needs your attention"),
                    ),
            )
            .children(found.into_iter().map(|notice| row(notice, &state, colors)))
            .into_any_element(),
    )
}

/// One notice as a row: its title, its state, and How to fix.
fn row(notice: &'static Notice, state: &NoticeState, colors: &Colors) -> impl IntoElement {
    let status = if state.is_fixed(notice.id) {
        Some("Fixed")
    } else if state.is_dismissed(notice.id) {
        Some("Dismissed")
    } else {
        None
    };
    h_flex()
        .id(SharedString::from(format!("notice-row-{}", notice.id)))
        .debug_selector(|| "notice-row".into())
        .gap(px(10.))
        .items_start()
        .p(px(10.))
        .rounded(px(8.))
        .border_1()
        .border_color(colors.divider)
        .bg(colors.editor)
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(px(2.))
                .child(
                    h_flex()
                        .gap(px(8.))
                        .items_center()
                        .child(
                            div()
                                .text_size(px(13.5))
                                .font_medium()
                                .text_color(colors.text)
                                .child(notice.title),
                        )
                        .children(status.map(|status| {
                            div()
                                .px(px(6.))
                                .rounded(px(4.))
                                .bg(colors.fill)
                                .text_size(px(11.))
                                .text_color(colors.text_muted)
                                .child(status)
                        })),
                )
                .child(
                    div()
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text_secondary)
                        .child(notice.what_changed),
                ),
        )
        .child(
            Button::new(SharedString::from(format!("notice-fix-{}", notice.id)))
                .flex_none()
                .xsmall()
                .label("How to fix")
                .on_click(move |_, window, cx| {
                    window.close_dialog(cx);
                    open_how_to_fix(notice, window, cx);
                }),
        )
}

/// The palette's "Update notices": every notice that applies to this
/// library, except those marked fixed.
// no-prop-test: a sheet; `kanso_core::notices::listed` decides.
pub fn open_list(window: &mut Window, cx: &mut App) {
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let colors = Colors::get(cx);
        let state = state(cx);
        let found = notices::listed(cx);
        let body = if found.is_empty() {
            div()
                .text_size(px(13.5))
                .text_color(colors.text_secondary)
                .child("Nothing needs your attention.")
                .into_any_element()
        } else {
            v_flex()
                .gap(px(8.))
                .children(found.into_iter().map(|notice| row(notice, &state, colors)))
                .into_any_element()
        };
        alert
            .title("Update notices")
            .width(px(SHEET_WIDTH))
            .show_cancel(false)
            .ok_text("Done")
            .child(body)
    });
}

/// The workspace's handler for the palette entry.
// no-prop-test: wiring; the palette's tests send each command's action.
pub fn on_actions(el: Div, cx: &mut Context<crate::workspace::Workspace>) -> Div {
    el.on_action(cx.listener(|_, _: &ShowUpdateNotices, window, cx| open_list(window, cx)))
}
