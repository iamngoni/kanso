// Kanso — unlock_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'unlock_bloc.dart';

@freezed
abstract class UnlockState with _$UnlockState {
  const factory UnlockState({
    /// Whether a password is being tried, or the phone is signing out.
    @Default(false) bool busy,

    /// What is wrong with the password typed.
    FormProblem? problem,

    /// Why the last call failed.
    ApplicationError? error,

    /// The wait for another device; non-null while the screen shows it.
    ApprovalWait? approval,

    /// The question about the notes on this phone, while it waits.
    BindingQuestion? question,

    /// How the unlock ended; null while it has not.
    AccountOutcome? outcome,
  }) = _UnlockState;
}
