//! My feedback (issue #232): the reports this person sent, newest first,
//! with where each stands: Waiting to send, Received, Filed as #241,
//! Building, Fix proposed, Merged, Released in 0.9.2, Not filed, Same as
//! #198. Open on GitHub (once filed) and Delete.
//!
//! Opening the sheet asks the server once; nothing polls. Deleting asks
//! first, then removes the report from Kanso's server; a delete that can't
//! reach it shows "Deleting…" until it goes through.

use gpui_component::scroll::ScrollableElement as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{
    Disableable as _, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_engine::feedback::{self, Chip, ReportView};
use kanso_engine::feedback_flow;

use crate::feedback_app::{self, Env};
use crate::theme::Colors;
use crate::zoom::px;

/// The sheet's width.
pub const SHEET_WIDTH: f32 = 560.;

/// "3 Nov", from a time in milliseconds (UTC). Pure.
pub fn short_date(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|at| at.format("%-d %b").to_string())
        .unwrap_or_default()
}

pub struct MyFeedbackSheet {
    env: Option<Env>,
    reports: Vec<ReportView>,
    asking: bool,
    confirming: Option<String>,
}

impl MyFeedbackSheet {
    fn new(cx: &mut Context<Self>) -> MyFeedbackSheet {
        let env = Env::capture(cx);
        let reports = env
            .as_ref()
            .and_then(|env| feedback_flow::views(&env.store).ok())
            .unwrap_or_default();
        let mut sheet = MyFeedbackSheet {
            env,
            reports,
            asking: false,
            confirming: None,
        };
        sheet.refresh(cx);
        sheet
    }

    /// Asks the server how the reports stand (and sends what is waiting).
    fn refresh(&mut self, cx: &mut Context<Self>) {
        let Some(env) = self.env.clone() else {
            return;
        };
        self.asking = true;
        let (tx, rx) = async_channel::bounded::<Vec<ReportView>>(1);
        std::thread::Builder::new()
            .name("kanso-feedback-list".into())
            .spawn(move || {
                let now = kanso_core::now_ms();
                let reporters = env.reporters();
                let client = env.client();
                feedback_flow::flush_as(&env.store, &client, &reporters, now).ok();
                let views = feedback_flow::refresh_as(&env.store, &client, &reporters, now)
                    .unwrap_or_default();
                let _ = tx.send_blocking(views);
            })
            .ok();
        cx.spawn(async move |this, cx| {
            let views = rx.recv().await.ok();
            this.update(cx, |this, cx| {
                this.asking = false;
                if let Some(views) = views {
                    this.reports = views;
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Deletes a report: gone at once from the list when the server never had
    /// it, else "Deleting…" until the server confirms.
    fn delete(&mut self, id: String, cx: &mut Context<Self>) {
        self.confirming = None;
        let Some(env) = self.env.clone() else {
            return;
        };
        feedback_flow::delete(&env.store, &id).ok();
        self.reports = feedback_flow::views(&env.store).unwrap_or_default();
        cx.notify();
        let (tx, rx) = async_channel::bounded::<Vec<ReportView>>(1);
        std::thread::Builder::new()
            .name("kanso-feedback-delete".into())
            .spawn(move || {
                feedback_flow::flush_as(
                    &env.store,
                    &env.client(),
                    &env.reporters(),
                    kanso_core::now_ms(),
                )
                .ok();
                let _ = tx.send_blocking(feedback_flow::views(&env.store).unwrap_or_default());
            })
            .ok();
        cx.spawn(async move |this, cx| {
            if let Ok(views) = rx.recv().await {
                this.update(cx, |this, cx| {
                    this.reports = views;
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }
}

impl Render for MyFeedbackSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let body = if self.reports.is_empty() {
            v_flex()
                .py(px(16.))
                .child(
                    div()
                        .text_size(px(13.5))
                        .text_color(colors.text_secondary)
                        .child(if self.asking {
                            "Looking for your reports…"
                        } else {
                            "You haven't sent any feedback yet. Help › Send Feedback… is the place."
                        }),
                )
                .into_any_element()
        } else {
            let rows = self.reports.iter().enumerate().map(|(ix, report)| {
                let chip = feedback::chip(report);
                let id = report.id.clone();
                let confirming = self.confirming.as_deref() == Some(report.id.as_str());
                let url = feedback::issue_url(feedback::REPO, report);
                let detail = feedback_app::chip_detail(&chip);
                let released = matches!(chip, Chip::ReleasedIn(_));
                v_flex()
                    .gap(px(6.))
                    .py(px(10.))
                    .border_b_1()
                    .border_color(colors.divider)
                    .child(
                        h_flex()
                            .gap_3()
                            .items_center()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(13.5))
                                    .when(released, |el| el.font_medium())
                                    .text_color(colors.text)
                                    .child(if report.preview.is_empty() {
                                        "(no text)".to_owned()
                                    } else {
                                        report.preview.clone()
                                    }),
                            )
                            .child(
                                div()
                                    .px(px(8.))
                                    .py(px(2.))
                                    .rounded(px(10.))
                                    .bg(colors.sidebar_hover)
                                    .text_size(px(12.))
                                    .text_color(colors.text_secondary)
                                    .debug_selector(move || format!("feedback-chip-{ix}"))
                                    .child(feedback_app::chip_text(&chip)),
                            )
                            .child(
                                div()
                                    .w(px(48.))
                                    .text_size(px(12.5))
                                    .text_color(colors.text_secondary)
                                    .child(short_date(report.created_ms)),
                            ),
                    )
                    .when_some(detail, |el, detail| {
                        el.child(
                            div()
                                .text_size(px(12.5))
                                .line_height(px(18.))
                                .text_color(colors.text_secondary)
                                .child(detail),
                        )
                    })
                    .child(if confirming {
                        let yes = id.clone();
                        v_flex()
                            .gap(px(8.))
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .line_height(px(18.))
                                    .text_color(colors.text)
                                    .child(feedback_app::DELETE_PROMPT),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        Button::new(("feedback-delete-yes", ix))
                                            .small()
                                            .danger()
                                            .label("Delete")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.delete(yes.clone(), cx)
                                            })),
                                    )
                                    .child(
                                        Button::new(("feedback-delete-no", ix))
                                            .small()
                                            .label("Cancel")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.confirming = None;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            .into_any_element()
                    } else {
                        let ask = id.clone();
                        h_flex()
                            .gap_2()
                            .when_some(url, |el, url| {
                                el.child(
                                    Button::new(("feedback-github", ix))
                                        .small()
                                        .ghost()
                                        .label("Open on GitHub")
                                        .on_click(move |_, _, cx| cx.open_url(&url)),
                                )
                            })
                            .child(
                                Button::new(("feedback-delete", ix))
                                    .small()
                                    .ghost()
                                    .label("Delete")
                                    .disabled(report.deleting)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.confirming = Some(ask.clone());
                                        cx.notify();
                                    })),
                            )
                            .into_any_element()
                    })
            });
            v_flex()
                .id("my-feedback-list")
                .max_h(px(420.))
                .overflow_y_scrollbar()
                .children(rows)
                .into_any_element()
        };
        v_flex().debug_selector(|| "my-feedback".into()).child(body)
    }
}

/// Opens My feedback.
// no-prop-test: opens a dialog; `chip`, `issue_url` and the flow are tested in the engine.
pub fn open(window: &mut Window, cx: &mut App) -> Entity<MyFeedbackSheet> {
    let sheet = cx.new(MyFeedbackSheet::new);
    let body = sheet.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title("My feedback")
            .width(px(SHEET_WIDTH))
            .child(body.clone())
    });
    sheet
}

#[cfg(test)]
mod tests {
    use super::short_date;

    #[test]
    fn dates_read_short() {
        assert_eq!(short_date(1_730_650_000_000), "3 Nov");
        assert_eq!(short_date(i64::MAX), "");
    }
}
