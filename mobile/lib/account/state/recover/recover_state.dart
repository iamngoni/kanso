// Kanso — recover_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'recover_bloc.dart';

@freezed
abstract class RecoverState with _$RecoverState {
  const factory RecoverState({
    /// The server of the account.
    @Default('') String server,

    /// Whether a recovery is being made.
    @Default(false) bool busy,

    /// What is wrong with what was typed.
    FormProblem? problem,

    /// Why the last call failed.
    ApplicationError? error,

    /// The question about the notes on this phone, while it waits.
    BindingQuestion? question,

    /// The new recovery key, to write down.
    RecoveryKeyText? recoveryKey,

    /// How the recovery ended; null while it has not.
    AccountOutcome? outcome,
  }) = _RecoverState;
}
