//! Voice in the Quick Capture window (issue #193): the mic button, hold ⌥ to
//! talk, the optional "Voice capture" shortcut that opens the window already
//! listening, and the Tidy button.
//!
//! It reuses the editor mic's parts (`kanso-voice`'s [`Dictation`] with the
//! downloaded Parakeet model, the rules in `voice_state::Phase`, and the
//! spoken commands) but its words go into the capture's own text area. A
//! capture made by voice remembers that, and saves through
//! `kanso_engine::voice_capture::save_voice_text` so the note records
//! `kansoCapture` and lands in the Inbox like any capture.
//!
//! Tidy sends the words to the person's own AI and puts the tidy version
//! (with the transcript folded under it) in the text area. **Undo tidy**
//! brings the words back for ten seconds. Nothing is ever half written: a
//! failed tidy leaves the text as it was and says why.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, Sizable as _, h_flex};
use gpui_kit::*;
use kanso_ai::tidy::TidyKind;
use kanso_engine::voice_capture::{TidyState, is_meeting, new_capture_id};
use kanso_engine::voice_tidy::{RenderMeta, date_label, render_tidy};
use kanso_voice::permission::{self, Permission};
use kanso_voice::{Dictation, VoiceError};

use crate::ai_feature;
use crate::capture::CaptureView;
use crate::config::{TidyMode, VoiceEngine};
use crate::theme::Colors;
use crate::voice::{self, ModelState, Voice};
use crate::voice_state::{Effect, Event, Phase};
use crate::workspace::AppState;
use crate::zoom::px;

/// How long ⌥ is held before it counts as "talk" (so ⌥ typing a symbol
/// doesn't start the microphone).
const HOLD: Duration = Duration::from_millis(300);
/// How long **Undo tidy** stays.
pub const UNDO_FOR: Duration = Duration::from_secs(10);
/// How often the listening bar is redrawn.
const TICK: Duration = Duration::from_millis(100);

/// Where the Tidy button stands.
#[derive(Default)]
pub enum TidyUi {
    #[default]
    Idle,
    /// Waiting for the model; `cancel` stops it.
    Working(Arc<AtomicBool>),
    /// Done; the words as they were, until the task runs out.
    Undo {
        previous: String,
        _expires: Task<()>,
    },
    /// Done and the undo has expired.
    Done,
    /// The model didn't tidy it; the text is as it was.
    Failed(String),
}

/// The window's voice state.
#[derive(Default)]
pub struct CaptureVoice {
    pub phase: Phase,
    sessions: u64,
    dictation: Option<Dictation>,
    ticker: Option<Task<()>>,
    transcribing: Option<Task<()>>,
    hold: Option<Task<()>>,
    alt_down: bool,
    /// ⌥ started this listening, so releasing it stops.
    held: bool,
    /// The words heard so far, while listening.
    pub live: String,
    pub elapsed: Duration,
    /// The text came from voice.
    pub from_voice: bool,
    /// How long was spoken, in all.
    pub heard_ms: u64,
    /// The capture's ID, made when the first words arrive.
    pub id: Option<String>,
    pub tidy: TidyUi,
    /// The text in the window is a tidy version.
    pub tidied: bool,
    /// The model is on its way to being ready (a download started here).
    pub error: Option<String>,
}

impl CaptureVoice {
    pub fn listening(&self) -> bool {
        matches!(self.phase, Phase::Listening { .. })
    }

    pub fn busy(&self) -> bool {
        self.phase.is_active()
    }

    /// The tidy state the note records.
    pub fn tidy_state(&self) -> TidyState {
        if self.tidied {
            TidyState::Done
        } else {
            TidyState::None
        }
    }
}

/// "0:07".
fn clock(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    format!("{}:{:02}", secs / 60, secs % 60)
}

impl CaptureView {
    /// The mic button, or the voice shortcut: starts listening, or stops and
    /// writes what was said.
    // no-prop-test: glue over `Phase::next`, which has the properties.
    pub fn toggle_mic(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.voice.error = None;
        self.error = None;
        let config = cx.global::<AppState>().config.voice;
        if config.engine == VoiceEngine::System && crate::dictation::AVAILABLE {
            // The system's own dictation types where the keyboard is.
            self.body.update(cx, |body, cx| body.focus(window, cx));
            crate::dictation::start();
            self.voice.from_voice = true;
            cx.notify();
            return;
        }
        match cx.global::<Voice>().model.clone() {
            ModelState::Ready => {}
            ModelState::Downloading { .. } => {
                self.say("Voice input is ready once the model has downloaded.", cx);
                return;
            }
            _ => {
                voice::start_download(cx);
                self.say(
                    "Downloading the speech model. Press the mic again once it's done.",
                    cx,
                );
                return;
            }
        }
        self.step_voice(Event::Press, window, cx);
    }

    fn say(&mut self, words: &str, cx: &mut Context<Self>) {
        self.voice.error = Some(words.to_owned());
        cx.notify();
    }

    /// Escape while listening drops what was said.
    pub fn cancel_mic(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.step_voice(Event::Cancel, window, cx);
    }

    // no-prop-test: glue over `Phase::next`, which has the properties.
    fn step_voice(&mut self, event: Event, window: &mut Window, cx: &mut Context<Self>) {
        let (phase, effect) = self.voice.phase.next(event, self.voice.sessions + 1);
        self.voice.phase = phase;
        match effect {
            Effect::None | Effect::Insert | Effect::ShowError => {}
            Effect::Listen(session) => {
                self.voice.sessions = session;
                self.voice.live.clear();
                self.listen(window, cx);
            }
            Effect::Transcribe => self.transcribe(window, cx),
            Effect::Drop => {
                if let Some(dictation) = self.voice.dictation.take() {
                    dictation.cancel();
                }
                self.voice.transcribing = None;
                self.voice.ticker = None;
                self.voice.held = false;
                self.voice.live.clear();
            }
        }
        cx.notify();
    }

    // no-prop-test: microphone glue; no oracle without a device.
    fn listen(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match permission::status() {
            Permission::Granted => self.open_microphone(window, cx),
            Permission::Denied => self.mic_failed(VoiceError::MicrophoneDenied, window, cx),
            Permission::Undetermined => {
                let (tx, rx) = async_channel::bounded(1);
                permission::request(move |granted| {
                    tx.try_send(granted).ok();
                });
                let view = cx.entity().downgrade();
                let window_handle = window.window_handle();
                cx.spawn(async move |_, cx| {
                    let granted = rx.recv().await.unwrap_or(false);
                    cx.update(|cx| {
                        let _ = window_handle.update(cx, |_, window, cx| {
                            view.update(cx, |this, cx| {
                                if !this.voice.listening() {
                                    return;
                                }
                                if granted {
                                    this.open_microphone(window, cx);
                                } else {
                                    this.mic_failed(VoiceError::MicrophoneDenied, window, cx);
                                }
                            })
                            .ok();
                        });
                    });
                })
                .detach();
            }
        }
    }

    // no-prop-test: microphone glue.
    fn open_microphone(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let dir = voice::model_dir(cx);
        match Dictation::start(&dir) {
            Ok(dictation) => {
                self.voice.dictation = Some(dictation);
                let window_handle = window.window_handle();
                let ticker = cx.spawn(async move |this, cx| {
                    loop {
                        cx.background_executor().timer(TICK).await;
                        let stop = cx.update(|cx| {
                            window_handle
                                .update(cx, |_, window, cx| {
                                    this.update(cx, |this, cx| this.tick(window, cx))
                                        .unwrap_or(true)
                                })
                                .unwrap_or(true)
                        });
                        if stop {
                            break;
                        }
                    }
                });
                self.voice.ticker = Some(ticker);
            }
            Err(err) => self.mic_failed(err, window, cx),
        }
    }

    /// Redraws the listening bar. True once listening is over.
    fn tick(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(dictation) = &self.voice.dictation else {
            return true;
        };
        self.voice.elapsed = dictation.elapsed();
        self.voice.live = dictation.live().1;
        let ended = !dictation.is_recording();
        cx.notify();
        if ended && self.voice.listening() {
            // The device went away, or the half hour ran out: what was heard
            // is still written.
            self.step_voice(Event::Press, window, cx);
            return true;
        }
        !self.voice.listening()
    }

    fn mic_failed(&mut self, err: VoiceError, window: &mut Window, cx: &mut Context<Self>) {
        self.step_voice(Event::MicFailed, window, cx);
        if let Some(words) = voice::error_words(&err) {
            self.say(&words, cx);
        }
    }

    // no-prop-test: glue over `Dictation::stop`.
    fn transcribe(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.voice.ticker = None;
        self.voice.held = false;
        let Phase::Transcribing { session } = self.voice.phase else {
            return;
        };
        let Some(dictation) = self.voice.dictation.take() else {
            return;
        };
        let spoken = dictation.elapsed();
        let done = dictation.stop();
        let window_handle = window.window_handle();
        let task = cx.spawn(async move |this, cx| {
            let result = done.recv().await.unwrap_or(Err(VoiceError::Cancelled));
            cx.update(|cx| {
                let _ = window_handle.update(cx, |_, window, cx| {
                    this.update(cx, |this, cx| {
                        let ok = result.is_ok();
                        this.voice.phase = this
                            .voice
                            .phase
                            .next(Event::Transcribed { session, ok }, 0)
                            .0;
                        this.voice.live.clear();
                        match result {
                            Ok(text) => this.heard(&text, spoken, window, cx),
                            Err(err) => {
                                if let Some(words) = voice::error_words(&err) {
                                    this.say(&words, cx);
                                }
                            }
                        }
                        cx.notify();
                    })
                    .ok();
                });
            });
        });
        self.voice.transcribing = Some(task);
    }

    /// Puts what was said at the end of the text.
    fn heard(
        &mut self,
        transcript: &str,
        spoken: Duration,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let commands = cx.global::<AppState>().config.voice.spoken_commands;
        let before = self.text(cx);
        let Some((text, _)) =
            kanso_engine::voice_commands::dictated(&before, None, transcript, commands)
        else {
            return;
        };
        let new = format!("{before}{text}");
        self.voice.from_voice = true;
        self.voice.heard_ms += u64::try_from(spoken.as_millis()).unwrap_or(u64::MAX);
        self.voice.id.get_or_insert_with(new_capture_id);
        // A tidy version is no longer what the words say.
        self.voice.tidied = false;
        self.voice.tidy = TidyUi::Idle;
        self.body
            .update(cx, |body, cx| body.set_value(new, window, cx));
        cx.notify();
        let config = cx.global::<AppState>().config.clone();
        if config.voice.tidy == TidyMode::Always && config.ai.enabled {
            self.tidy_now(window, cx);
        }
    }

    /// ⌥ pressed or released over the window: held for a moment, it talks.
    pub fn modifiers_changed(
        &mut self,
        modifiers: Modifiers,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let alt_only =
            modifiers.alt && !modifiers.control && !modifiers.platform && !modifiers.shift;
        if alt_only && !self.voice.alt_down {
            self.voice.alt_down = true;
            let window_handle = window.window_handle();
            self.voice.hold = Some(cx.spawn(async move |this, cx| {
                cx.background_executor().timer(HOLD).await;
                cx.update(|cx| {
                    let _ = window_handle.update(cx, |_, window, cx| {
                        this.update(cx, |this, cx| {
                            if this.voice.alt_down && this.voice.phase == Phase::Idle {
                                this.toggle_mic(window, cx);
                                this.voice.held = this.voice.listening();
                            }
                        })
                        .ok();
                    });
                });
            }));
        } else if !modifiers.alt && self.voice.alt_down {
            self.voice.alt_down = false;
            self.voice.hold = None;
            if self.voice.held && self.voice.listening() {
                self.toggle_mic(window, cx);
            }
            self.voice.held = false;
        }
    }

    /// Marks the text as spoken, for tests (no microphone here).
    #[cfg(test)]
    pub fn spoken_for_test(&mut self, heard_ms: u64) {
        self.voice.from_voice = true;
        self.voice.heard_ms = heard_ms;
    }

    /// The Tidy state a test looks at.
    #[cfg(test)]
    pub fn tidy_for_test(&self) -> (&'static str, bool) {
        (
            match self.voice.tidy {
                TidyUi::Idle => "idle",
                TidyUi::Working(_) => "working",
                TidyUi::Undo { .. } => "undo",
                TidyUi::Done => "done",
                TidyUi::Failed(_) => "failed",
            },
            self.voice.tidied,
        )
    }

    /// Whether the Tidy button is offered: the words came from voice, AI is
    /// set up, and Settings doesn't have tidying off.
    pub fn can_tidy(&self, cx: &App) -> bool {
        let config = &cx.global::<AppState>().config;
        self.voice.from_voice
            && !self.voice.tidied
            && config.ai.enabled
            && config.voice.tidy != TidyMode::Off
            && !self.voice.busy()
            && !self.text(cx).trim().is_empty()
    }

    /// Sends the words to the person's AI and puts the tidy version in the
    /// text area.
    // no-prop-test: glue over `ai_feature::tidy` and `render_tidy`, which have them.
    pub fn tidy_now(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.voice.tidy, TidyUi::Working(_)) {
            return;
        }
        let text = self.text(cx);
        if text.trim().is_empty() {
            return;
        }
        let config = cx.global::<AppState>().config.voice;
        let meeting = is_meeting(self.voice.heard_ms, config.meetings);
        let kind = if meeting {
            TidyKind::Meeting
        } else {
            TidyKind::Note
        };
        let maker = ai_feature::maker(cx);
        crate::metrics::record(cx, crate::metrics::Feature::Ai);
        let cancel = Arc::new(AtomicBool::new(false));
        self.voice.tidy = TidyUi::Working(cancel.clone());
        let (tx, rx) = async_channel::bounded(1);
        let words = text.clone();
        let spawned = std::thread::Builder::new()
            .name("kanso-tidy".into())
            .spawn(move || {
                tx.try_send(ai_feature::tidy(&maker, kind, "", &words, &cancel))
                    .ok();
            });
        if spawned.is_err() {
            self.voice.tidy = TidyUi::Failed("Couldn't start.".into());
            cx.notify();
            return;
        }
        let duration = self.voice.heard_ms;
        let window_handle = window.window_handle();
        cx.spawn(async move |this, cx| {
            let result = rx.recv().await;
            cx.update(|cx| {
                let _ = window_handle.update(cx, |_, window, cx| {
                    this.update(cx, |this, cx| match result {
                        // Stopped with Escape meanwhile.
                        _ if !matches!(this.voice.tidy, TidyUi::Working(_)) => {}
                        Ok(Ok(tidy)) => this.tidied_text(&text, &tidy, kind, duration, window, cx),
                        Ok(Err(reason)) => {
                            this.voice.tidy = TidyUi::Failed(reason);
                            cx.notify();
                        }
                        Err(_) => {}
                    })
                    .ok();
                });
            });
        })
        .detach();
        cx.notify();
    }

    fn tidied_text(
        &mut self,
        words: &str,
        tidy: &kanso_ai::tidy::Tidy,
        kind: TidyKind,
        duration_ms: u64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Typed over while the model worked: leave it alone.
        if self.text(cx) != words {
            self.voice.tidy = TidyUi::Idle;
            cx.notify();
            return;
        }
        let meta = RenderMeta {
            kind,
            duration_ms,
            on_device: true,
            date: date_label(now_ms()),
            fallback_title: kanso_engine::voice_capture::voice_title(words),
        };
        let rendered = render_tidy(tidy, words, &meta);
        let previous = words.to_owned();
        self.body
            .update(cx, |body, cx| body.set_value(rendered.body(), window, cx));
        self.voice.tidied = true;
        let expires = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(UNDO_FOR).await;
            let _ = this.update(cx, |this, cx| {
                if matches!(this.voice.tidy, TidyUi::Undo { .. }) {
                    this.voice.tidy = TidyUi::Done;
                    cx.notify();
                }
            });
        });
        self.voice.tidy = TidyUi::Undo {
            previous,
            _expires: expires,
        };
        cx.notify();
    }

    /// **Undo tidy**: the words as they were said.
    pub fn undo_tidy(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let TidyUi::Undo { previous, .. } = std::mem::take(&mut self.voice.tidy) {
            self.body
                .update(cx, |body, cx| body.set_value(previous, window, cx));
            self.voice.tidied = false;
            cx.notify();
        }
    }

    /// The listening bar under the header.
    pub fn render_voice_bar(&self, cx: &App) -> Option<AnyElement> {
        let colors = Colors::get(cx);
        let bar = h_flex()
            .flex_none()
            .h(px(30.))
            .px(px(14.))
            .gap(px(8.))
            .items_center()
            .text_size(px(12.5));
        if self.voice.listening() {
            let live = self.voice.live.clone();
            return Some(
                bar.child(div().size(px(8.)).rounded_full().bg(colors.danger))
                    .child(
                        div()
                            .text_color(colors.text)
                            .child(format!("Listening · {}", clock(self.voice.elapsed))),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_color(colors.text_muted)
                            .child(live),
                    )
                    .child(div().text_color(colors.text_muted).child("Esc drops it"))
                    .into_any_element(),
            );
        }
        if matches!(self.voice.phase, Phase::Transcribing { .. }) {
            return Some(
                bar.text_color(colors.text_muted)
                    .child("Writing what you said…")
                    .into_any_element(),
            );
        }
        if let TidyUi::Working(_) = self.voice.tidy {
            return Some(
                bar.text_color(colors.text_muted)
                    .child("Tidying…")
                    .into_any_element(),
            );
        }
        if let TidyUi::Failed(reason) = &self.voice.tidy {
            return Some(
                bar.text_color(colors.danger)
                    .child(format!("Couldn't tidy this note: {reason}"))
                    .into_any_element(),
            );
        }
        self.voice.error.clone().map(|error| {
            bar.text_color(colors.danger)
                .child(error)
                .into_any_element()
        })
    }

    /// The mic, and Tidy or Undo tidy, for the header.
    pub fn voice_buttons(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let colors = Colors::get(cx);
        let mut out: Vec<AnyElement> = Vec::new();
        let listening = self.voice.listening();
        out.push(
            Button::new("capture-mic")
                .ghost()
                .small()
                .debug_selector(|| "capture-mic".into())
                .icon(
                    Icon::new(if listening {
                        Lucide::Square
                    } else {
                        Lucide::Mic
                    })
                    .text_color(if listening {
                        colors.danger
                    } else {
                        colors.text_secondary
                    }),
                )
                .tooltip(if listening {
                    "Stop listening"
                } else {
                    "Speak (or hold ⌥)"
                })
                .on_click(cx.listener(|this, _, window, cx| this.toggle_mic(window, cx)))
                .into_any_element(),
        );
        if let TidyUi::Undo { .. } = self.voice.tidy {
            out.push(
                Button::new("capture-undo-tidy")
                    .ghost()
                    .small()
                    .debug_selector(|| "capture-undo-tidy".into())
                    .label("Undo tidy")
                    .on_click(cx.listener(|this, _, window, cx| this.undo_tidy(window, cx)))
                    .into_any_element(),
            );
        } else if self.can_tidy(cx) && !matches!(self.voice.tidy, TidyUi::Working(_)) {
            out.push(
                Button::new("capture-tidy")
                    .ghost()
                    .small()
                    .debug_selector(|| "capture-tidy".into())
                    .icon(Icon::new(Lucide::Sparkles).text_color(colors.text_secondary))
                    .tooltip("Tidy with AI: a title, paragraphs and tasks")
                    .on_click(cx.listener(|this, _, window, cx| this.tidy_now(window, cx)))
                    .into_any_element(),
            );
        }
        out
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::clock;
    use std::time::Duration;

    // no-prop-test: three fixed durations.
    #[test]
    fn the_clock_reads_minutes_and_seconds() {
        assert_eq!(clock(Duration::from_secs(7)), "0:07");
        assert_eq!(clock(Duration::from_secs(61)), "1:01");
        assert_eq!(clock(Duration::from_millis(59_999)), "0:59");
    }
}
