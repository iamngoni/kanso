// Kanso — sync_settings_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'sync_settings_bloc.dart';

@freezed
abstract class SyncSettingsState with _$SyncSettingsState {
  const factory SyncSettingsState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// The settings as shown; null until they are read.
    SyncSettingsRow? settings,

    /// How often sync may run, in the desktop's words.
    @Default(<SyncIntervalOption>[]) List<SyncIntervalOption> options,

    /// Why the settings could not be read or the last change saved.
    ApplicationError? error,
  }) = _SyncSettingsState;

  const SyncSettingsState._();

  /// The option the settings name, if they are read.
  SyncIntervalOption? get interval {
    final secs = settings?.intervalSecs;
    if (secs == null) {
      return null;
    }
    for (final option in options) {
      if (option.minutes * 60 == secs) {
        return option;
      }
    }
    return null;
  }
}
