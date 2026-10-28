// Kanso — change_password_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'change_password_bloc.dart';

@freezed
abstract class ChangePasswordState with _$ChangePasswordState {
  const factory ChangePasswordState({
    @Default(LockFormStatus.ready) LockFormStatus status,

    /// How many locked notes or notebooks the current password didn't open.
    @Default(0) int kept,
    ApplicationError? error,
  }) = _ChangePasswordState;
}
