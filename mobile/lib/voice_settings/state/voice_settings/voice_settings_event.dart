// Kanso — voice_settings_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'voice_settings_bloc.dart';

@freezed
sealed class VoiceSettingsEvent with _$VoiceSettingsEvent {
  /// The page opened, or the app came back to it: read the settings and ask
  /// what the recognizer can do.
  const factory VoiceSettingsEvent.started() = VoiceSettingsStartedEvent;

  /// The person switched spoken punctuation and commands to [on].
  const factory VoiceSettingsEvent.spokenCommandsChanged({required bool on}) =
      VoiceSettingsSpokenCommandsChangedEvent;

  /// The person asked for the phone's Settings, to allow the microphone.
  const factory VoiceSettingsEvent.systemSettingsRequested() =
      VoiceSettingsSystemSettingsRequestedEvent;
}
