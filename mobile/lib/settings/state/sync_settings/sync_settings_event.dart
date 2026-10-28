// Kanso — sync_settings_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'sync_settings_bloc.dart';

@freezed
sealed class SyncSettingsEvent with _$SyncSettingsEvent {
  /// The settings are shown: read them.
  const factory SyncSettingsEvent.started() = SyncSettingsStartedEvent;

  /// Sync automatically was turned on or off.
  const factory SyncSettingsEvent.enabledChanged({required bool enabled}) =
      SyncSettingsEnabledChangedEvent;

  /// How often to sync was chosen, in minutes: one of the options.
  const factory SyncSettingsEvent.intervalChosen(int minutes) =
      SyncSettingsIntervalChosenEvent;
}
