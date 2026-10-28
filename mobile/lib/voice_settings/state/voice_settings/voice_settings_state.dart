// Kanso — voice_settings_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'voice_settings_bloc.dart';

@freezed
abstract class VoiceSettingsState with _$VoiceSettingsState {
  const factory VoiceSettingsState({
    @Default(LoadStatus.loading) LoadStatus status,
    @Default(VoiceSettings()) VoiceSettings settings,

    /// What the recognizer can do; null until asked.
    DictationCapability? capability,
  }) = _VoiceSettingsState;
}
