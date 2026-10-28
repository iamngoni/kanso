// Kanso — reminders_settings_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'reminders_settings_bloc.dart';

@freezed
sealed class RemindersSettingsEvent with _$RemindersSettingsEvent {
  /// Settings opened: read the switch and the permission.
  const factory RemindersSettingsEvent.started() =
      RemindersSettingsStartedEvent;

  /// The app came back to the foreground: the permission may have changed.
  const factory RemindersSettingsEvent.resumed() =
      RemindersSettingsResumedEvent;

  /// The person flipped the switch.
  const factory RemindersSettingsEvent.toggled({required bool enabled}) =
      RemindersSettingsToggledEvent;
}
