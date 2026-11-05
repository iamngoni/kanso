//! Snapshot states for voice input: the pill in the note's header while
//! listening and writing, a recording being transcribed, and Settings'
//! Voice input in each model state.

use std::time::Duration;

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_store::Store;

use crate::config::VoiceEngine;
use crate::settings::SettingsPage;
use crate::speakers::Speakers;
use crate::voice::{ModelState, Voice};
use crate::voice_state::Phase;
use crate::workspace::{AppState, Workspace};

fn set_voice(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    phase: Phase,
    model: ModelState,
) {
    cx.update(|cx| {
        cx.global_mut::<AppState>().config.voice.engine = VoiceEngine::OnDevice;
        let editor = workspace.read(cx).editor.clone();
        let voice = Voice::for_snapshot(
            phase,
            &editor,
            window,
            (0.45, Duration::from_secs(14)),
            model,
        );
        cx.set_global(voice);
    });
}

pub(super) fn editor_listening(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    set_voice(
        cx,
        window,
        workspace,
        Phase::Listening { session: 1 },
        ModelState::Ready,
    );
}

pub(super) fn editor_writing(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    set_voice(
        cx,
        window,
        workspace,
        Phase::Transcribing { session: 1 },
        ModelState::Ready,
    );
}

fn settings_voice(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    model: ModelState,
) {
    set_voice(cx, window, workspace, Phase::Idle, model);
    super::show_settings(cx, window, workspace, SettingsPage::Editor);
    // Voice input is the Editor page's last section.
    cx.update(|cx| {
        workspace.update(cx, |workspace, cx| {
            workspace
                .settings
                .update(cx, |settings, _| settings.scroll_to_end_for_test());
        });
    });
}

pub(super) fn settings_voice_missing(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_voice(cx, window, workspace, ModelState::Missing);
}

pub(super) fn settings_voice_downloading(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_voice(
        cx,
        window,
        workspace,
        ModelState::Downloading {
            done: 231_400_000,
            total: 670_619_803,
        },
    );
}

pub(super) fn settings_voice_ready(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_voice(cx, window, workspace, ModelState::Ready);
}

pub(super) fn settings_voice_failed(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_voice(
        cx,
        window,
        workspace,
        ModelState::Failed("Couldn't reach the download. Check the connection.".into()),
    );
}

/// A recording dropped into the note, a third transcribed.
pub(super) fn editor_transcribing(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        let editor = workspace.read(cx).editor.clone();
        crate::transcribe::show_for_snapshot(&editor, window, "Standup 21 Oct.m4a", 0.34, cx);
    });
}

/// The recording bar over the main window, 42 minutes in.
pub(super) fn recording_bar(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        crate::recording::show_for_snapshot(
            crate::recording_model::Notice::Recording(crate::recording_model::BarState {
                seconds: 42 * 60 + 17,
                level: 0.6,
                paused: false,
                label: "Microphone".into(),
                dot_lit: true,
            }),
            cx,
        );
    });
}

/// A saved recording open in the preview, with the real player and scrubber.
pub(super) fn recording_player(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let directory = tempfile::tempdir().unwrap();
    let mut recorder = kanso_audio::Recorder::create(
        &directory.path().join("in-progress"),
        kanso_audio::RecorderOptions::new(
            kanso_audio::Channels::Mono,
            vec![kanso_audio::RecordSource::Microphone],
        ),
    )
    .unwrap();
    let pcm: Vec<f32> = (0..48_000 * 5)
        .map(|i| 0.35 * (std::f32::consts::TAU * 440.0 * i as f32 / 48_000.0).sin())
        .collect();
    recorder.push(&pcm).unwrap();
    let finished = recorder.finish().unwrap();
    let saved = kanso_engine::recording_flow::save_recording(
        store,
        &kanso_engine::recording_flow::Recorded {
            session_id: "snapshot-player".into(),
            parts: finished.parts.into_iter().map(|(path, _)| path).collect(),
            duration_secs: 5,
            mic_only: true,
            when: "28 May 2026, 09:45".into(),
        },
        &kanso_engine::recording_flow::Destination::NewNote { book_id: None },
        kanso_store::Origin::Ui,
    )
    .unwrap();
    let body = store.note(&saved.note_id).unwrap().unwrap().body;
    super::open_new_note(cx, workspace, store, "Recording player", &body);
    cx.update(|cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.set_view_mode_for_test(crate::config::ViewMode::Preview, cx);
        });
    });
}

/// The same bar while paused.
pub(super) fn recording_paused(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        crate::recording::show_for_snapshot(
            crate::recording_model::Notice::Recording(crate::recording_model::BarState {
                seconds: 3_723,
                level: 0.0,
                paused: true,
                label: "Microphone".into(),
                dot_lit: false,
            }),
            cx,
        );
    });
}

/// The banner after a crash.
pub(super) fn recording_recover(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        crate::recording::show_for_snapshot(
            crate::recording_model::Notice::Crash(crate::recording_model::CrashBanner {
                id: "x".into(),
                text: kanso_engine::recording_flow::recovery_text(Some("14:05"), 52 * 60),
            }),
            cx,
        );
    });
}

/// Settings › Editor › Voice input with Tell speakers apart on and its two
/// models in `model`.
fn settings_speakers(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    model: ModelState,
) {
    cx.update(|cx| {
        cx.global_mut::<AppState>().config.voice.speakers = true;
        cx.set_global(Speakers::for_snapshot(model));
    });
    settings_voice(cx, window, workspace, ModelState::Ready);
}

pub(super) fn settings_speakers_missing(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_speakers(cx, window, workspace, ModelState::Missing);
}

pub(super) fn settings_speakers_downloading(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_speakers(
        cx,
        window,
        workspace,
        ModelState::Downloading {
            done: 11_200_000,
            total: 28_071_056,
        },
    );
}

pub(super) fn settings_speakers_ready(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_speakers(cx, window, workspace, ModelState::Ready);
}
