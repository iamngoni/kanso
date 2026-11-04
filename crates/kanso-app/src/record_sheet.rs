//! The Record sheet (issue #38, step 4): what to record, where it goes and
//! when it is transcribed. One sheet for every way in: the note toolbar's
//! Record button, Note › Record… (⇧⌘R), and Quick Capture.
//!
//! Only the microphone can be chosen today; the system audio choices are
//! there and say why they can't be picked. "While recording" is the same:
//! transcription starts the moment the recording stops.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{
    Disableable as _, Selectable as _, Sizable as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::*;
use kanso_engine::recording_flow::{Destination, RecordPlan, TranscribeWhen, WhatToRecord};

use crate::recording;
use crate::theme::Colors;
use crate::zoom::px;

const WIDTH: f32 = 480.;

/// The sheet's choices.
pub struct RecordSheet {
    plan: RecordPlan,
    /// The note that is open, when there is one: "This note" is offered.
    open_note: Option<String>,
}

impl RecordSheet {
    fn new(open_note: Option<String>) -> Self {
        RecordSheet {
            plan: RecordPlan::new(),
            open_note,
        }
    }

    pub fn plan(&self) -> &RecordPlan {
        &self.plan
    }

    fn choose_destination(&mut self, destination: Destination, cx: &mut Context<Self>) {
        self.plan.destination = destination;
        cx.notify();
    }
}

/// One option of a row: selected, or disabled with a reason.
fn option(
    id: impl Into<ElementId>,
    label: &'static str,
    selected: bool,
    disabled: Option<&'static str>,
) -> Button {
    let button = Button::new(id)
        .small()
        .label(label)
        .selected(selected)
        .disabled(disabled.is_some());
    match disabled {
        Some(why) => button.tooltip(why),
        None => button,
    }
}

fn row(title: &'static str, colors: &Colors, options: impl IntoElement) -> impl IntoElement {
    v_flex()
        .gap(px(6.))
        .child(
            div()
                .text_size(px(12.))
                .text_color(colors.text_muted)
                .child(title),
        )
        .child(options)
}

impl Render for RecordSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let new_note = matches!(self.plan.destination, Destination::NewNote { .. });
        let open_note = self.open_note.clone();
        let this_note = open_note.clone();
        v_flex()
            .w_full()
            .gap(px(14.))
            .child(row(
                "What to record",
                colors,
                h_flex()
                    .gap(px(6.))
                    .child(option(
                        "rec-mic",
                        "Microphone",
                        self.plan.what == WhatToRecord::Microphone,
                        None,
                    ))
                    .child(option(
                        "rec-mic-system",
                        "Microphone + System Audio",
                        false,
                        Some("Recording a meeting app's audio comes in a later version."),
                    ))
                    .child(option(
                        "rec-system",
                        "System Audio",
                        false,
                        Some("Recording a meeting app's audio comes in a later version."),
                    )),
            ))
            .child(row(
                "Put it in",
                colors,
                h_flex()
                    .gap(px(6.))
                    .child(
                        option("rec-new-note", "New note", new_note, None).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.choose_destination(
                                    Destination::NewNote { book_id: None },
                                    cx,
                                )
                            },
                        )),
                    )
                    .child(
                        option(
                            "rec-this-note",
                            "This note",
                            !new_note,
                            open_note.is_none().then_some("No note is open."),
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(note_id) = this_note.clone() {
                                this.choose_destination(Destination::ThisNote { note_id }, cx);
                            }
                        })),
                    ),
            ))
            .child(row(
                "Transcribe",
                colors,
                h_flex()
                    .gap(px(6.))
                    .child(option(
                        "rec-live",
                        "While recording",
                        false,
                        Some("Live transcription comes in a later version; the note is transcribed as soon as you stop."),
                    ))
                    .child(option(
                        "rec-after",
                        "After stopping",
                        self.plan.transcribe == TranscribeWhen::AfterStopping,
                        None,
                    )),
            ))
            .child(
                div()
                    .text_size(px(12.))
                    .line_height(relative(1.4))
                    .text_color(colors.text_muted)
                    .child("Audio stays on this device until you sync, and the words are written on this device too. Tell everyone nearby you are recording; the rules differ by place. Headphones keep the other side out of your microphone."),
            )
    }
}

/// Opens the sheet. `open_note` is the note on screen, if any.
// no-prop-test: drawing and glue.
pub fn open(open_note: Option<String>, window: &mut Window, cx: &mut App) {
    if recording::busy(cx) {
        window.push_notification(
            gpui_kit::component::notification::Notification::info(
                "A recording is already running. Stop it before starting another.",
            ),
            cx,
        );
        return;
    }
    let mut sheet = RecordSheet::new(open_note);
    // Only the microphone, transcribed on stopping, can run today.
    sheet.plan.transcribe = TranscribeWhen::AfterStopping;
    let sheet = cx.new(|_| sheet);
    let body = sheet.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        let start = body.clone();
        alert
            .title("Record")
            .width(px(WIDTH))
            .child(body.clone())
            .footer(
                h_flex()
                    .w_full()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("record-cancel")
                            .small()
                            .label("Cancel")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("record-start")
                            .debug_selector(|| "record-start".into())
                            .small()
                            .primary()
                            .label("Start Recording")
                            .on_click(move |_, window, cx| {
                                let plan = start.read(cx).plan().clone();
                                window.close_dialog(cx);
                                recording::start(plan, cx);
                            }),
                    ),
            )
    });
}
