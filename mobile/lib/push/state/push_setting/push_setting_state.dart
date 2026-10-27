// Kanso — push_setting_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'push_setting_bloc.dart';

@freezed
abstract class PushSettingState with _$PushSettingState {
  const factory PushSettingState({
    @Default(PushSettingStatus.loading) PushSettingStatus status,

    /// Whether this phone is to be woken. On until turned off.
    @Default(true) bool enabled,

    /// Whether a change is being made.
    @Default(false) bool saving,

    /// Why the last change could not be made.
    ApplicationError? error,
  }) = _PushSettingState;
}
