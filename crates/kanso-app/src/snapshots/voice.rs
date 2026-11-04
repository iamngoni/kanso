//! Snapshot states for voice input: the pill in the note's header while
//! listening and writing, a recording being transcribed, and Settings'
//! Voice input in each model state.

use std::time::Duration;

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_store::Store;

use crate::config::VoiceEngine;
use crate::settings::SettingsPage;
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
                label: "Microphone",
                dot_lit: true,
            }),
            cx,
        );
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
                label: "Microphone",
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
