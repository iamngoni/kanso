//! The Record sheet (issue #38, steps 4 and 7): what to record, which app,
//! where it goes and when it is transcribed. One sheet for every way in:
//! the note toolbar's Record button, Note › Record… (⇧⌘R), and Quick
//! Capture.
//!
//! The system audio choices follow the platform. On a Mac, turning system
//! audio on is where macOS's Screen Recording permission is asked for, and
//! the only place: a permission row says where it stands and how to turn it
//! on, and an app picker lists what is running. With the permission off,
//! Start still works for the microphone and says why the app is left out.
//! Where the computer's sound is a device (Windows loopback, a Linux monitor
//! source) a row names the output to record; where nothing can be recorded
//! the choices are disabled and say why. "While recording" is not offered
//! yet: transcription starts the moment the recording stops.

use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{
    Disableable as _, Selectable as _, Sizable as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_audio::system_audio::{
    self, Availability, SystemAudioDevice, SystemAudioTarget, Unavailable,
};
use kanso_audio::{CaptureApp, MEETING_APPS, ScreenAccess};
use kanso_engine::recording_flow::{
    Destination, KeepAudio, RecordPlan, SystemSource, TranscribeWhen, WhatToRecord,
};
use kanso_engine::recording_privacy::BACKUP_WARNING;

use crate::recording;
use crate::theme::Colors;
use crate::zoom::px;

pub const WIDTH: f32 = 480.;
/// How often the permission row looks again while it waits for a yes.
const RECHECK: Duration = Duration::from_secs(1);

/// What the sheet knows about the machine. Real in the app; fixed in the
/// snapshots, which must not read the machine they run on.
#[derive(Clone, Debug)]
pub struct Machine {
    pub apps: Vec<CaptureApp>,
    pub access: ScreenAccess,
    /// The person has been asked for the permission in this sheet.
    pub asked: bool,
}

/// What the snapshots show the sheet over, instead of the machine they run
/// on.
#[derive(Clone, Debug)]
pub struct Fixed {
    /// The Mac's apps and permission, when the sheet is the Mac's.
    pub machine: Option<Machine>,
    pub what: WhatToRecord,
    /// The app picked, when one is.
    pub system: Option<SystemSource>,
    /// Whether system audio can be recorded, and the outputs to pick from.
    pub backend: Option<(Availability, Vec<SystemAudioDevice>)>,
}

/// What the system audio choices say where the platform has no way to
/// record it at all.
const NO_BACKEND: &str = "Recording a meeting app's audio comes in a later version.";

/// The sheet's choices.
pub struct RecordSheet {
    plan: RecordPlan,
    /// The note that is open, when there is one: "This note" is offered.
    open_note: Option<String>,
    machine: Machine,
    /// Whether this is the Mac's sheet: one app's audio, behind a
    /// permission, instead of an output device.
    app_mode: bool,
    /// Whether this computer can record its own sound, and from what.
    system: Availability,
    outputs: Vec<SystemAudioDevice>,
    /// Reads the machine (the apps, the permission) instead of using what
    /// it was given.
    live: bool,
    _recheck: Option<Task<()>>,
}

impl RecordSheet {
    fn new(open_note: Option<String>, fixed: Option<Fixed>, cx: &mut Context<Self>) -> Self {
        let live = fixed.is_none();
        let none = Machine {
            apps: Vec::new(),
            access: ScreenAccess::NotAllowed,
            asked: false,
        };
        let (machine, app_mode, what, system, backend) = match fixed {
            Some(fixed) => (
                fixed.machine.clone().unwrap_or(none),
                fixed.machine.is_some(),
                fixed.what,
                fixed.system,
                fixed.backend,
            ),
            None => (
                none,
                cfg!(target_os = "macos"),
                WhatToRecord::Microphone,
                None,
                None,
            ),
        };
        let (availability, outputs) =
            backend.unwrap_or_else(|| match system_audio::platform_backend() {
                Some(backend) => (backend.availability(), backend.devices()),
                None => (
                    Availability::Unavailable(Unavailable::Unsupported),
                    Vec::new(),
                ),
            });
        let mut sheet = RecordSheet {
            plan: RecordPlan::new(),
            open_note,
            machine,
            app_mode,
            system: availability,
            outputs,
            live,
            _recheck: None,
        };
        sheet.plan.what = what;
        sheet.plan.system = system;
        if !sheet.app_mode && what.uses_system() {
            sheet.plan.system.get_or_insert(SystemSource::Everything);
        }
        sheet.plan.transcribe = TranscribeWhen::AfterStopping;
        if live && app_mode {
            sheet._recheck = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor().timer(RECHECK).await;
                    if this.update(cx, |this, cx| this.recheck(cx)).is_err() {
                        break;
                    }
                }
            }));
        }
        sheet
    }

    pub fn plan(&self) -> &RecordPlan {
        &self.plan
    }

    /// Why the system audio choices can't be picked, or `None` when they can.
    /// A Mac that hasn't been allowed yet can still pick them: choosing is
    /// where the permission is asked for.
    fn system_disabled(&self) -> Option<String> {
        match &self.system {
            Availability::Unavailable(Unavailable::Unsupported) => Some(NO_BACKEND.into()),
            Availability::Unavailable(Unavailable::Denied(_)) if self.app_mode => None,
            other => other.reason(),
        }
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

    fn choose_destination(&mut self, destination: Destination, cx: &mut Context<Self>) {
        self.plan.destination = destination;
        cx.notify();
    }

    /// Picks what becomes of the audio once it is transcribed.
    pub(crate) fn choose_keep(&mut self, keep: KeepAudio, cx: &mut Context<Self>) {
        self.plan.keep = keep;
        cx.notify();
    }

    /// Picks what to record. Turning system audio on lists the running apps
    /// and, the first time, asks macOS for Screen Recording; the microphone
    /// alone never does.
    fn choose_what(&mut self, what: WhatToRecord, cx: &mut Context<Self>) {
        self.plan.what = what;
        if what.uses_system() && !self.app_mode {
            // A computer that records a device has no app to pick and no
            // Screen Recording to ask for: it records what it plays (the
            // output row picks which device).
            self.plan.system.get_or_insert(SystemSource::Everything);
        } else if what.uses_system() {
            if self.live {
                self.machine.apps = kanso_audio::running_apps();
                self.machine.access = kanso_audio::screen_access();
            }
            if self.machine.access != ScreenAccess::Allowed && !self.machine.asked {
                self.ask(cx);
            }
            self.keep_choice_listed();
        }
        cx.notify();
    }

    /// Asks the system for Screen Recording. It answers at once with the
    /// present state; the person's yes arrives later and `recheck` sees it.
    fn ask(&mut self, cx: &mut Context<Self>) {
        self.machine.asked = true;
        if self.live {
            self.machine.access = kanso_audio::request_screen_access();
        }
        cx.notify();
    }

    /// Looks at the permission and the apps again.
    fn recheck(&mut self, cx: &mut Context<Self>) {
        if !self.live || !self.plan.what.uses_system() {
            return;
        }
        let access = kanso_audio::screen_access();
        let apps = kanso_audio::running_apps();
        if access != self.machine.access || apps != self.machine.apps {
            self.machine.access = access;
            self.machine.apps = apps;
            self.keep_choice_listed();
            cx.notify();
        }
    }

    /// An app that was chosen and has quit is no longer chosen; with nothing
    /// chosen, a meeting app that is running is offered first.
    fn keep_choice_listed(&mut self) {
        let listed = |id: &str| self.machine.apps.iter().any(|a| a.bundle_id == id);
        if let Some(SystemSource::App { bundle_id, .. }) = &self.plan.system
            && !listed(bundle_id)
        {
            self.plan.system = None;
        }
        if self.plan.system.is_none()
            && let Some(app) = self
                .machine
                .apps
                .first()
                .filter(|a| MEETING_APPS.contains(&a.bundle_id.as_str()))
        {
            self.plan.system = Some(SystemSource::App {
                bundle_id: app.bundle_id.clone(),
                name: app.name.clone(),
            });
        }
    }

    fn choose_app(&mut self, source: SystemSource, cx: &mut Context<Self>) {
        self.plan.system = Some(source);
        cx.notify();
    }
}

/// The reminder under a choice that records system audio: whose sound is
/// recorded (the app on a Mac, else what the computer plays), with the
/// microphone named only when it records too. Pure.
fn consent_words(what: WhatToRecord, app_mode: bool, system: Option<&SystemSource>) -> String {
    let mine = match (what.uses_microphone(), app_mode) {
        (false, _) => "",
        (true, true) => "this Mac's microphone and ",
        (true, false) => "the microphone and ",
    };
    let whose = match (system, app_mode) {
        (_, false) => "what the computer plays".to_owned(),
        (Some(SystemSource::App { name, .. }), true) => format!("{name}'s audio"),
        (Some(SystemSource::Everything), true) => "everything this Mac plays".to_owned(),
        (None, true) => "the app's audio".to_owned(),
    };
    format!(
        "Kanso records {mine}{whose}. Make sure everyone in the meeting knows. Recording rules differ by place."
    )
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

fn note(text: impl Into<SharedString>, colors: &Colors) -> Div {
    div()
        .text_size(px(12.))
        .line_height(relative(1.4))
        .text_color(colors.text_muted)
        .child(text.into())
}

/// Where Screen Recording is turned on.
const SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture";

// no-prop-test: opens a system window.
fn open_settings() {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(SETTINGS_URL).spawn();
    }
}

impl RecordSheet {
    /// The permission row: a dot, what it means in plain words, and the
    /// button that fits where it stands.
    fn permission_row(&self, colors: &Colors, cx: &mut Context<Self>) -> impl IntoElement {
        let allowed = self.machine.access == ScreenAccess::Allowed;
        let dot = if allowed {
            Hsla::from(colors.success)
        } else {
            Hsla::from(colors.warning)
        };
        let words = kanso_audio::access_words(self.machine.access, self.machine.asked);
        let buttons = h_flex().gap(px(6.)).when(!allowed, |row| {
            if self.machine.asked {
                row.child(
                    Button::new("rec-open-settings")
                        .debug_selector(|| "rec-open-settings".into())
                        .small()
                        .label("Open System Settings")
                        .on_click(|_, _, _| open_settings()),
                )
                .child(
                    Button::new("rec-check-again")
                        .small()
                        .label("Check again")
                        .on_click(cx.listener(|this, _, _, cx| {
                            if this.live {
                                this.machine.access = kanso_audio::screen_access();
                            }
                            cx.notify();
                        })),
                )
            } else {
                row.child(
                    Button::new("rec-allow")
                        .debug_selector(|| "rec-allow".into())
                        .small()
                        .label("Allow Screen Recording")
                        .on_click(cx.listener(|this, _, _, cx| this.ask(cx))),
                )
            }
        });
        v_flex()
            .id("rec-permission")
            .debug_selector(|| "rec-permission".into())
            .gap(px(8.))
            .p(px(10.))
            .rounded(px(8.))
            .border_1()
            .border_color(colors.divider)
            .child(
                h_flex()
                    .gap(px(8.))
                    .items_start()
                    .child(
                        div()
                            .mt(px(4.))
                            .flex_none()
                            .size(px(8.))
                            .rounded_full()
                            .bg(dot),
                    )
                    .child(div().flex_1().min_w(px(0.)).child(note(words, colors))),
            )
            .child(buttons)
    }

    /// The app picker: the running apps with meeting apps first, and the
    /// whole system as the last choice.
    fn app_picker(&self, colors: &Colors, cx: &mut Context<Self>) -> impl IntoElement {
        let chosen = self.plan.system.clone();
        let mut apps = h_flex().id("rec-apps").gap(px(6.)).flex_wrap();
        for (ix, app) in self.machine.apps.iter().enumerate() {
            let source = SystemSource::App {
                bundle_id: app.bundle_id.clone(),
                name: app.name.clone(),
            };
            let selected = chosen.as_ref() == Some(&source);
            apps = apps.child(
                option(("rec-app", ix), app.name.clone(), selected, None).on_click(
                    cx.listener(move |this, _, _, cx| this.choose_app(source.clone(), cx)),
                ),
            );
        }
        apps = apps.child(
            option(
                "rec-app-all",
                "All system audio",
                chosen == Some(SystemSource::Everything),
                None,
            )
            .on_click(cx.listener(|this, _, _, cx| {
                this.choose_app(SystemSource::Everything, cx);
            })),
        );
        let hint = if self.machine.apps.is_empty() {
            "No other app is open. Open your meeting app, then pick it here."
        } else {
            "Kanso hears one app, so notification sounds from others stay out. Audio is per app, not per window: a browser records every tab."
        };
        v_flex()
            .id("rec-app-picker")
            .debug_selector(|| "rec-app-picker".into())
            .gap(px(6.))
            .child(row("Which app", colors, apps))
            .child(note(hint, colors))
    }
}

impl Render for RecordSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let new_note = matches!(self.plan.destination, Destination::NewNote { .. });
        let open_note = self.open_note.clone();
        let this_note = open_note.clone();
        let what = self.plan.what;
        let keep = self.plan.keep;
        let system_off: Option<SharedString> = self.system_disabled().map(Into::into);
        let consent = what
            .uses_system()
            .then(|| consent_words(what, self.app_mode, self.plan.system.as_ref()));
        let problem = self.plan.problem();
        v_flex()
            .w_full()
            .gap(px(14.))
            .child(row(
                "What to record",
                colors,
                h_flex()
                    .gap(px(6.))
                    .flex_wrap()
                    .child(
                        option(
                            "rec-mic",
                            "Microphone",
                            what == WhatToRecord::Microphone,
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
                            what == WhatToRecord::MicrophoneAndSystem,
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
                            what == WhatToRecord::SystemOnly,
                            system_off.clone(),
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.choose_what(WhatToRecord::SystemOnly, cx)
                        })),
                    ),
            ))
            .when(what.uses_system() && self.app_mode, |sheet| {
                sheet
                    .child(self.permission_row(colors, cx))
                    .child(self.app_picker(colors, cx))
            })
            // Which output, on a computer that records a device. One choice
            // is not a choice: the default alone needs no row.
            .when(
                what.uses_system() && !self.app_mode && self.outputs.len() > 1,
                |sheet| {
                    let mut choices = h_flex().gap(px(6.)).flex_wrap();
                    for (ix, (target, label)) in system_audio::picker_labels(&self.outputs)
                        .into_iter()
                        .enumerate()
                    {
                        let selected = self.output_selected(&target);
                        choices = choices.child(
                            option(("rec-output", ix), label, selected, None).on_click(
                                cx.listener(move |this, _, _, cx| {
                                    this.choose_output(target.clone(), cx)
                                }),
                            ),
                        );
                    }
                    sheet.child(row("Sound to record", colors, choices))
                },
            )
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
            .child(row(
                "After transcribing",
                colors,
                h_flex()
                    .gap(px(6.))
                    .child(
                        option(
                            "rec-keep-audio",
                            "Keep audio",
                            keep == KeepAudio::Keep,
                            None,
                        )
                        .on_click(
                            cx.listener(|this, _, _, cx| this.choose_keep(KeepAudio::Keep, cx)),
                        ),
                    )
                    .child(
                        option(
                            "rec-keep-transcript",
                            "Keep transcript only",
                            keep == KeepAudio::TranscriptOnly,
                            None,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.choose_keep(KeepAudio::TranscriptOnly, cx)
                        })),
                    ),
            ))
            .when(keep == KeepAudio::TranscriptOnly, |sheet| {
                sheet.child(
                    div()
                        .debug_selector(|| "rec-keep-warning".into())
                        .child(note(
                            format!(
                                "The audio is deleted once the transcript is in the note. If the transcript fails, the audio stays. {BACKUP_WARNING}"
                            ),
                            colors,
                        )),
                )
            })
            .when_some(problem, |sheet, problem| {
                sheet.child(
                    div()
                        .debug_selector(|| "rec-problem".into())
                        .text_size(px(12.))
                        .text_color(colors.warning)
                        .child(problem),
                )
            })
            .child(note(
                consent.unwrap_or_else(|| {
                    "Audio stays on this device until you sync, and the words are written on this device too. Tell everyone nearby you are recording; the rules differ by place.".to_owned()
                }),
                colors,
            ))
            .child(note(
                "Headphones keep the other side out of your microphone.",
                colors,
            ))
    }
}

/// Opens the sheet. `open_note` is the note on screen, if any.
// no-prop-test: drawing and glue.
pub fn open(open_note: Option<String>, window: &mut Window, cx: &mut App) {
    open_with(open_note, None, window, cx);
}

/// Opens the sheet as it would look on a computer with `system` and
/// `outputs`, with `what` chosen, for the snapshots.
// no-prop-test: drawing and glue.
#[cfg(test)]
pub fn open_for_snapshot(
    system: Availability,
    outputs: Vec<SystemAudioDevice>,
    what: WhatToRecord,
    keep: KeepAudio,
    window: &mut Window,
    cx: &mut App,
) {
    let fixed = Fixed {
        machine: None,
        what,
        system: None,
        backend: Some((system, outputs)),
    };
    let sheet = open_with(None, Some(fixed), window, cx);
    if let Some(sheet) = sheet {
        sheet.update(cx, |sheet, cx| sheet.choose_keep(keep, cx));
    }
}

/// Opens the sheet over a fixed machine and choice (the snapshots), or the
/// real machine.
pub fn open_with(
    open_note: Option<String>,
    fixed: Option<Fixed>,
    window: &mut Window,
    cx: &mut App,
) -> Option<Entity<RecordSheet>> {
    if recording::busy(cx) {
        window.push_notification(
            gpui_kit::component::notification::Notification::info(
                "A recording is already running. Stop it before starting another.",
            ),
            cx,
        );
        return None;
    }
    let sheet = cx.new(|cx| RecordSheet::new(open_note, fixed, cx));
    let body = sheet.clone();
    let opened = sheet.clone();
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
                                // Say what is missing and keep the sheet.
                                if let Some(problem) = plan.problem() {
                                    window.push_notification(
                                        gpui_kit::component::notification::Notification::info(
                                            problem,
                                        ),
                                        cx,
                                    );
                                    return;
                                }
                                window.close_dialog(cx);
                                recording::start(plan, cx);
                            }),
                    ),
            )
    });
    Some(opened)
}

#[cfg(test)]
mod tests {
    // Not `super::*`: it brings `gpui_kit::*`'s `test`, which `#[test]`
    // inside `gpui_kit::test` would then expand into forever.
    use super::{Fixed, Machine, RecordSheet, consent_words};
    use gpui_kit::{AppContext as _, TestAppContext};
    use kanso_audio::ScreenAccess;
    use kanso_audio::system_audio::{Availability, SystemAudioDevice};
    use kanso_engine::recording_flow::{SystemSource, WhatToRecord};
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    fn device_machine(what: WhatToRecord) -> Fixed {
        Fixed {
            machine: None,
            what,
            system: None,
            backend: Some((
                Availability::Available,
                vec![SystemAudioDevice {
                    id: "out-1".into(),
                    name: "Speakers".into(),
                    is_default: true,
                }],
            )),
        }
    }

    /// Where system audio is a device (Windows, Linux) there is no app
    /// picker, so the sheet must never ask for an app it can't offer.
    #[gpui_kit::test]
    fn a_device_machine_never_asks_for_an_app(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let opened = cx.new(|cx| {
            RecordSheet::new(
                None,
                Some(device_machine(WhatToRecord::MicrophoneAndSystem)),
                cx,
            )
        });
        cx.read(|cx| assert_eq!(opened.read(cx).plan().problem(), None));
        let chosen =
            cx.new(|cx| RecordSheet::new(None, Some(device_machine(WhatToRecord::Microphone)), cx));
        chosen.update(cx, |sheet, cx| {
            sheet.choose_what(WhatToRecord::SystemOnly, cx);
            assert_eq!(sheet.plan().problem(), None);
            assert_eq!(sheet.plan().system, Some(SystemSource::Everything));
            // Nothing was asked of the machine.
            assert!(!sheet.machine.asked);
        });
    }

    /// On a Mac, system audio still needs an app (or "All system audio").
    #[gpui_kit::test]
    fn a_mac_needs_an_app_picked(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let sheet = cx.new(|cx| {
            RecordSheet::new(
                None,
                Some(Fixed {
                    machine: Some(Machine {
                        apps: Vec::new(),
                        access: ScreenAccess::Allowed,
                        asked: true,
                    }),
                    what: WhatToRecord::MicrophoneAndSystem,
                    system: None,
                    backend: Some((Availability::Available, Vec::new())),
                }),
                cx,
            )
        });
        cx.read(|cx| assert!(sheet.read(cx).plan().problem().is_some()));
    }

    fn what() -> impl Strategy<Value = WhatToRecord> {
        prop_oneof![
            Just(WhatToRecord::Microphone),
            Just(WhatToRecord::MicrophoneAndSystem),
            Just(WhatToRecord::SystemOnly),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// The reminder names the microphone exactly when it records (System
        /// Audio alone never claims it), and names an app only on a Mac.
        #[test]
        fn prop_consent_names_what_is_recorded(
            what in what(),
            app_mode in any::<bool>(),
            app in proptest::option::of("[A-Z][a-z]{2,8}"),
        ) {
            let source = app.clone().map(|name| SystemSource::App {
                bundle_id: format!("com.{name}"),
                name,
            });
            let words = consent_words(what, app_mode, source.as_ref());
            prop_assert_eq!(words.contains("microphone"), what.uses_microphone());
            prop_assert_eq!(words.contains("what the computer plays"), !app_mode);
            if let Some(name) = &app {
                prop_assert_eq!(words.contains(&format!("{name}'s audio")), app_mode);
            }
        }
    }
}
