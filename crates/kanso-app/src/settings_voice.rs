//! Settings › Editor › Voice input: how the mic listens, the speech model
//! it downloads to listen on this device, and the microphone's permission.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::Button;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Sizable as _, WindowExt as _, h_flex};
use gpui_kit::*;
use kanso_voice::permission::{self, Permission};

use crate::config::{TidyMode, VoiceEngine};
use crate::settings::{Row, SettingsView, section as settings_section, segmented};
use crate::settings_discovery::{megabytes, meter, this_device};
use crate::theme::Colors;
use crate::voice::{self, MODEL, ModelState, Voice};
use crate::workspace::AppState;
use crate::zoom::px;

/// Where System Settings lets apps use the microphone.
const MICROPHONE_SETTINGS: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone";

/// What the spoken commands row says.
const SPOKEN_COMMANDS: &str = "Say \"comma\", \"full stop\", \"new line\", \"new paragraph\", \"bullet point\" or \"new to-do\" on their own, with a short pause either side, and they're written. \"Delete that\" takes out the last thing you said, and \"undo that\" the last step.";

/// The model row's title and line for `state`.
pub fn model_words(state: &ModelState) -> (String, String) {
    let size = megabytes(MODEL.total_size());
    match state {
        ModelState::Unknown | ModelState::Missing => (
            "Voice model".to_owned(),
            format!(
                "About {size}, downloaded once. It turns what you say into text on {}.",
                this_device()
            ),
        ),
        ModelState::Downloading { done, total } => (
            format!(
                "Downloading the voice model… {}%",
                crate::model_fetch::percent(*done, *total)
            ),
            format!("{} of {}", megabytes(*done), megabytes(*total)),
        ),
        ModelState::Ready => (
            "Voice model downloaded".to_owned(),
            format!(
                "{size} on {}. {}",
                this_device(),
                kanso_voice::model::ATTRIBUTION
            ),
        ),
        ModelState::Failed(why) => (
            "The download stopped".to_owned(),
            format!("{why} What arrived is kept, so trying again carries on from there."),
        ),
    }
}

/// The Voice input section.
// no-prop-test: layout and buttons; `model_words` says what it shows, and
// the settings snapshots show each state.
pub fn section(cx: &mut Context<SettingsView>) -> AnyElement {
    let colors = Colors::get(cx);
    let engine = cx.global::<AppState>().config.voice.engine;
    let model = cx
        .try_global::<Voice>()
        .map(|v| v.model.clone())
        .unwrap_or_default();
    let mut how = Row::new(Lucide::Mic, "Voice input").detail(
        "Press the mic beside a note's title, speak, and press it again: what you said goes in at the cursor. Esc drops it.",
    );
    if crate::dictation::AVAILABLE {
        how = how.trailing(segmented(
            "voice-engine",
            [
                ("On This Device", VoiceEngine::OnDevice),
                ("System Dictation", VoiceEngine::System),
            ],
            engine,
            |engine, _, cx| AppState::update_config(cx, |config| config.voice.engine = engine),
            colors,
        ));
    }
    let mut rows = vec![how];
    let on_device = engine != VoiceEngine::System || !crate::dictation::AVAILABLE;
    if on_device {
        rows.push(model_row(&model, colors));
        let spoken = cx.global::<AppState>().config.voice.spoken_commands;
        rows.push(
            Row::new(Lucide::Pilcrow, "Spoken punctuation and commands")
                .detail(SPOKEN_COMMANDS)
                .trailing(
                    div().debug_selector(|| "voice-commands".into()).child(
                        Switch::new("voice-commands")
                            .checked(spoken)
                            .on_click(|on, _, cx| {
                                let on = *on;
                                AppState::update_config(cx, |config| {
                                    config.voice.spoken_commands = on
                                })
                            }),
                    ),
                ),
        );
        if engine == VoiceEngine::OnDevice && permission::status() == Permission::Denied {
            let mut denied = Row::new(Lucide::TriangleAlert, "Kanso can't use the microphone")
                .detail(
                    voice::error_words(&kanso_voice::VoiceError::MicrophoneDenied)
                        .unwrap_or_default(),
                );
            if cfg!(target_os = "macos") {
                denied = denied.trailing(
                    Button::new("voice-permission")
                        .outline()
                        .small()
                        .label("Open System Settings")
                        .on_click(|_, _, cx| cx.open_url(MICROPHONE_SETTINGS)),
                );
            }
            rows.push(denied);
        }
    } else {
        rows.push(Row::new(Lucide::Keyboard, voice::system_dictation_name()).detail(
            "Your system's own dictation types into the note. Set it up, and pick its language, in your system's keyboard settings.",
        ));
    }
    rows.extend(capture_rows(cx, colors));
    settings_section("Voice input", rows, colors)
}

/// What the Tidy row says under its title. Pure.
pub fn tidy_detail(ai_on: bool) -> &'static str {
    if ai_on {
        "Your own AI gives a voice note a title, short paragraphs and tasks for what was said you would do. Only the words are sent. The transcript stays in the note, folded under it."
    } else {
        "Set up AI in Settings › AI to tidy notes."
    }
}

/// Voice capture's rows (issue #193): tidying, and what counts as a meeting.
// no-prop-test: layout; `tidy_detail` says what it shows.
fn capture_rows(cx: &mut Context<SettingsView>, colors: &Colors) -> Vec<Row> {
    let config = cx.global::<AppState>().config.clone();
    let ai_on = config.ai.enabled;
    let mut tidy = Row::new(Lucide::Sparkles, "Tidy voice notes").detail(tidy_detail(ai_on));
    if ai_on {
        tidy = tidy.trailing(segmented(
            "voice-tidy",
            [
                ("Off", TidyMode::Off),
                ("Ask Each Time", TidyMode::Ask),
                ("Always", TidyMode::Always),
            ],
            config.voice.tidy,
            |mode, _, cx| AppState::update_config(cx, |config| config.voice.tidy = mode),
            colors,
        ));
    }
    let meetings = Row::new(Lucide::Users, "Meetings")
        .detail("Treat recordings longer than 3 minutes as meetings: tidy gives them a Summary, Decisions, Action items and Notes.")
        .trailing(
            div().debug_selector(|| "voice-meetings".into()).child(
                Switch::new("voice-meetings")
                    .checked(config.voice.meetings)
                    .on_click(|on, _, cx| {
                        let on = *on;
                        AppState::update_config(cx, |config| config.voice.meetings = on)
                    }),
            ),
        );
    vec![tidy, meetings]
}

// no-prop-test: layout; `model_words` has the tests.
fn model_row(model: &ModelState, colors: &Colors) -> Row {
    let (title, detail) = model_words(model);
    let icon = match model {
        ModelState::Ready => Lucide::Check,
        ModelState::Failed(_) => Lucide::TriangleAlert,
        _ => Lucide::Download,
    };
    let row = Row::new(icon, title).detail(detail);
    match model {
        ModelState::Downloading { done, total } => row.trailing(
            h_flex()
                .gap(px(10.))
                .items_center()
                .child(meter(*done as f32 / (*total).max(1) as f32, colors))
                .child(
                    Button::new("voice-download-cancel")
                        .outline()
                        .small()
                        .label("Stop")
                        .on_click(|_, _, cx| voice::cancel_download(cx)),
                ),
        ),
        ModelState::Ready => row.trailing(
            Button::new("voice-remove")
                .outline()
                .small()
                .label("Remove")
                .on_click(|_, window, cx| {
                    if let Err(err) = voice::remove_model(cx) {
                        window.push_notification(
                            gpui_kit::component::notification::Notification::error(format!(
                                "Couldn't remove the voice model: {err}"
                            )),
                            cx,
                        );
                    }
                }),
        ),
        ModelState::Failed(_) => row.trailing(
            Button::new("voice-retry")
                .outline()
                .small()
                .label("Try Again")
                .on_click(|_, _, cx| voice::start_download(cx)),
        ),
        ModelState::Unknown | ModelState::Missing => row.trailing(
            Button::new("voice-download")
                .outline()
                .small()
                .label("Download")
                .on_click(|_, _, cx| voice::start_download(cx)),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{ModelState, model_words, tidy_detail};
    use proptest::prelude::*;

    #[test]
    fn model_words_examples() {
        let (title, detail) = model_words(&ModelState::Missing);
        assert_eq!(title, "Voice model");
        assert!(detail.starts_with("About 670.6\u{a0}MB"), "{detail}");
        assert_eq!(
            model_words(&ModelState::Downloading {
                done: 335_000_000,
                total: 670_000_000
            })
            .0,
            "Downloading the voice model… 50%"
        );
        assert!(model_words(&ModelState::Ready).1.contains("NVIDIA"));
    }

    // no-prop-test: two fixed sentences.
    #[test]
    fn tidy_row_points_to_ai_settings_without_ai() {
        assert!(tidy_detail(false).contains("Settings › AI"));
        assert!(tidy_detail(true).contains("transcript stays"));
    }

    fn reads(state: &ModelState, why: &str) -> Result<(), TestCaseError> {
        let (title, detail) = model_words(state);
        prop_assert!(!title.is_empty() && !detail.is_empty());
        prop_assert!(!title.contains("101%"));
        if let ModelState::Failed(_) = state {
            prop_assert!(detail.starts_with(why));
        }
        Ok(())
    }

    proptest! {
        /// Every state reads, the download's progress never past 100%, and a
        /// stopped download says why.
        #[test]
        fn prop_every_state_reads(done in 0u64..800_000_000, total in 1u64..800_000_000, why in "[A-Za-z ]{1,30}") {
            let downloading = ModelState::Downloading { done: done.min(total), total };
            let failed = ModelState::Failed(why.clone());
            for state in [&ModelState::Unknown, &ModelState::Missing, &downloading, &ModelState::Ready, &failed] {
                reads(state, &why)?;
            }
        }
    }
}
