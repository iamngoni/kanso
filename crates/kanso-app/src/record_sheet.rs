//! The Record sheet (issue #38, step 4): what to record, where it goes and
//! when it is transcribed. One sheet for every way in: the note toolbar's
//! Record button, Note › Record… (⇧⌘R), and Quick Capture.
//!
//! The system audio choices follow the platform (steps 8 and 9): where the
//! computer's sound can be recorded (Windows loopback, a Linux monitor
//! source) they can be picked and a row names the output to record; where
//! it can't, they are disabled and say why. Starting one still waits for
//! mixing it with the microphone (step 10). "While recording" is the same:
//! transcription starts the moment the recording stops.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{
    Disableable as _, Selectable as _, Sizable as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::*;
use kanso_audio::system_audio::{
    self, Availability, SystemAudioDevice, SystemAudioTarget, Unavailable,
};
use kanso_engine::recording_flow::{Destination, RecordPlan, TranscribeWhen, WhatToRecord};

use crate::recording;
use crate::theme::Colors;
use crate::zoom::px;

pub const WIDTH: f32 = 480.;

/// What the system audio choices say where the platform has no way to
/// record it at all (macOS until ScreenCaptureKit lands).
const NO_BACKEND: &str = "Recording a meeting app's audio comes in a later version.";

/// The sheet's choices.
pub struct RecordSheet {
    plan: RecordPlan,
    /// The note that is open, when there is one: "This note" is offered.
    open_note: Option<String>,
    /// Whether this computer can record its own sound, and from what.
    system: Availability,
    outputs: Vec<SystemAudioDevice>,
}

impl RecordSheet {
    fn new(open_note: Option<String>) -> Self {
        let (system, outputs) = match system_audio::platform_backend() {
            Some(backend) => (backend.availability(), backend.devices()),
            None => (
                Availability::Unavailable(Unavailable::Unsupported),
                Vec::new(),
            ),
        };
        Self::with_system(open_note, system, outputs)
    }

    fn with_system(
        open_note: Option<String>,
        system: Availability,
        outputs: Vec<SystemAudioDevice>,
    ) -> Self {
        let mut plan = RecordPlan::new();
        // Only the microphone, transcribed on stopping, can run today.
        plan.transcribe = TranscribeWhen::AfterStopping;
        RecordSheet {
            plan,
            open_note,
            system,
            outputs,
        }
    }

    pub fn plan(&self) -> &RecordPlan {
        &self.plan
    }

    /// Why the system audio choices can't be picked, or `None` when they can.
    fn system_disabled(&self) -> Option<String> {
        match &self.system {
            Availability::Unavailable(Unavailable::Unsupported) => Some(NO_BACKEND.into()),
            other => other.reason(),
        }
    }

    fn choose_destination(&mut self, destination: Destination, cx: &mut Context<Self>) {
        self.plan.destination = destination;
        cx.notify();
    }

    fn choose_what(&mut self, what: WhatToRecord, cx: &mut Context<Self>) {
        self.plan.what = what;
        cx.notify();
    }

    fn choose_output(&mut self, target: SystemAudioTarget, cx: &mut Context<Self>) {
        self.plan.system_output = match target {
            SystemAudioTarget::Device(id) => Some(id),
            _ => None,
        };
        cx.notify();
    }

    /// Whether a picker choice is the one the plan holds.
    fn output_selected(&self, target: &SystemAudioTarget) -> bool {
        match (&self.plan.system_output, target) {
            (None, SystemAudioTarget::Default) => true,
            (Some(id), SystemAudioTarget::Device(other)) => id == other,
            _ => false,
        }
    }
}

/// One option of a row: selected, or disabled with a reason.
fn option(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    selected: bool,
    disabled: Option<SharedString>,
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
        let system_off: Option<SharedString> = self.system_disabled().map(Into::into);
        let system_on = self.plan.what != WhatToRecord::Microphone;
        let mut sheet =
            v_flex().w_full().gap(px(14.)).child(row(
                "What to record",
                colors,
                h_flex()
                    .gap(px(6.))
                    .child(
                        option(
                            "rec-mic",
                            "Microphone",
                            self.plan.what == WhatToRecord::Microphone,
                            None,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.choose_what(WhatToRecord::Microphone, cx)
                        })),
                    )
                    .child(
                        option(
                            "rec-mic-system",
                            "Microphone + System Audio",
                            self.plan.what == WhatToRecord::MicrophoneAndSystem,
                            system_off.clone(),
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.choose_what(WhatToRecord::MicrophoneAndSystem, cx)
                        })),
                    )
                    .child(
                        option(
                            "rec-system",
                            "System Audio",
                            self.plan.what == WhatToRecord::SystemOnly,
                            system_off.clone(),
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.choose_what(WhatToRecord::SystemOnly, cx)
                        })),
                    ),
            ));
        // Which output, once system audio is on. One choice is not a choice:
        // the default alone needs no row.
        if system_on && self.outputs.len() > 1 {
            let mut choices = h_flex().gap(px(6.)).flex_wrap();
            for (ix, (target, label)) in system_audio::picker_labels(&self.outputs)
                .into_iter()
                .enumerate()
            {
                let selected = self.output_selected(&target);
                choices =
                    choices.child(option(("rec-output", ix), label, selected, None).on_click(
                        cx.listener(move |this, _, _, cx| this.choose_output(target.clone(), cx)),
                    ));
            }
            sheet = sheet.child(row("Sound to record", colors, choices));
        }
        sheet
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
                            open_note.is_none().then(|| "No note is open.".into()),
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
                        Some("Live transcription comes in a later version; the note is transcribed as soon as you stop.".into()),
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
    open_with(RecordSheet::new(open_note), window, cx);
}

/// Opens the sheet as it would look on a computer with `system` and
/// `outputs`, with `what` chosen, for the snapshots.
// no-prop-test: drawing and glue.
#[cfg(test)]
pub fn open_for_snapshot(
    system: Availability,
    outputs: Vec<SystemAudioDevice>,
    what: WhatToRecord,
    window: &mut Window,
    cx: &mut App,
) {
    let mut sheet = RecordSheet::with_system(None, system, outputs);
    sheet.plan.what = what;
    open_with(sheet, window, cx);
}

fn open_with(sheet: RecordSheet, window: &mut Window, cx: &mut App) {
    if recording::busy(cx) {
        window.push_notification(
            gpui_kit::component::notification::Notification::info(
                "A recording is already running. Stop it before starting another.",
            ),
            cx,
        );
        return;
    }
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
