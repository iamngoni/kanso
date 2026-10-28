// Kanso — unlock_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'unlock_bloc.dart';

@freezed
abstract class UnlockState with _$UnlockState {
  const factory UnlockState({
    @Default(UnlockStatus.idle) UnlockStatus status,

    /// Whether to offer turning on a biometric now; only with
    /// [UnlockStatus.opened].
    @Default(false) bool offerBiometrics,

    /// Why it failed, with [UnlockStatus.failed].
    ApplicationError? error,
  }) = _UnlockState;
}
