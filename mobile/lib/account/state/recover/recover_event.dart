// Kanso — recover_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'recover_bloc.dart';

@freezed
sealed class RecoverEvent with _$RecoverEvent {
  /// The screen opened, for the account on [server] (the default server when
  /// none is given).
  const factory RecoverEvent.started({String? server}) = RecoverStartedEvent;

  /// Reset password was pressed.
  const factory RecoverEvent.submitted({
    required String username,
    required String recoveryKey,
    required String password,
    required String repeat,
    required String deviceName,
  }) = RecoverSubmittedEvent;

  /// The question about the notes on this phone was answered; null cancels.
  const factory RecoverEvent.bindingAnswered(BindingAnswer? answer) =
      RecoverBindingAnsweredEvent;
}
