// Kanso — lock_remove_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'lock_remove_bloc.dart';

@freezed
abstract class LockRemoveState with _$LockRemoveState {
  const factory LockRemoveState({
    @Default(RemoveStatus.idle) RemoveStatus status,

    /// Why it failed, with [RemoveStatus.failed]. An invalid request carries
    /// the library's own sentence, which the sheet shows as it is.
    ApplicationError? error,
  }) = _LockRemoveState;
}
