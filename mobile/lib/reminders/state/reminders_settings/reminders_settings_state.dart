// Kanso — reminders_settings_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'reminders_settings_bloc.dart';

@freezed
abstract class RemindersSettingsState with _$RemindersSettingsState {
  const factory RemindersSettingsState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// The switch.
    @Default(true) bool enabled,
    @Default(ReminderPermission.notDetermined) ReminderPermission permission,
    ApplicationError? error,
  }) = _RemindersSettingsState;

  const RemindersSettingsState._();

  /// Whether the switch is on but the system will not show notifications:
  /// the row says how to allow them.
  bool get blockedBySystem =>
      enabled && permission == ReminderPermission.denied;
}
