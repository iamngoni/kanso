// Kanso — ai_settings_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'ai_settings_bloc.dart';

@freezed
abstract class AiSettingsState with _$AiSettingsState {
  const factory AiSettingsState({
    @Default(LoadStatus.loading) LoadStatus status,
    @Default(AiSettings()) AiSettings settings,

    /// The places answers can come from.
    @Default(<AiProviderInfo>[]) List<AiProviderInfo> providers,
    @Default(AiTestStatus.idle) AiTestStatus test,

    /// What the model said, or why the test failed, once it has run.
    @Default('') String testMessage,

    /// Whether a key is being saved or removed.
    @Default(false) bool keyBusy,

    /// Why the last change could not be saved.
    ApplicationError? error,
  }) = _AiSettingsState;

  const AiSettingsState._();

  /// The provider that is chosen, if the core listed it.
  AiProviderInfo? get provider {
    for (final info in providers) {
      if (info.key == settings.provider) {
        return info;
      }
    }
    return null;
  }
}
