// Kanso — push_setting_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'push_setting_bloc.dart';

@freezed
sealed class PushSettingEvent with _$PushSettingEvent {
  /// Settings opened: read the setting.
  const factory PushSettingEvent.started() = PushSettingStartedEvent;

  /// The person switched the setting on or off.
  const factory PushSettingEvent.toggled({required bool enabled}) =
      PushSettingToggledEvent;

  /// The person read the notice of a change that failed.
  const factory PushSettingEvent.errorDismissed() =
      PushSettingErrorDismissedEvent;
}
