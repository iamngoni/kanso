// Kanso — lock_setup_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'lock_setup_bloc.dart';

@freezed
abstract class LockSetupState with _$LockSetupState {
  const factory LockSetupState({
    LockTarget? target,
    @Default(LockMode.setPassword) LockMode mode,
    @Default(LockFormStatus.loading) LockFormStatus status,

    /// How many notes were sealed, once [status] is done.
    @Default(0) int count,

    /// Why it failed, with [LockFormStatus.failed].
    ApplicationError? error,
  }) = _LockSetupState;
}
