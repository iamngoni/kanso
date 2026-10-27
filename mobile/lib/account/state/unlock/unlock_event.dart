// Kanso — unlock_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'unlock_bloc.dart';

@freezed
sealed class UnlockEvent with _$UnlockEvent {
  /// The screen opened. With [resumeApproval] (this phone asked another
  /// device earlier and the request still stands) the wait picks up where it
  /// was.
  const factory UnlockEvent.started({
    required bool resumeApproval,
    required String deviceName,
  }) = UnlockStartedEvent;

  /// Unlock was pressed with the account's password.
  const factory UnlockEvent.passwordSubmitted({
    required String username,
    required String password,
  }) = UnlockPasswordSubmittedEvent;

  /// Approve from another device was pressed, or Try again after a request
  /// ended.
  const factory UnlockEvent.approvalRequested({required String deviceName}) =
      UnlockApprovalRequestedEvent;

  /// Cancel was pressed while waiting for another device.
  const factory UnlockEvent.approvalCancelled() = UnlockApprovalCancelledEvent;

  /// Sign out was pressed.
  const factory UnlockEvent.signedOut() = UnlockSignedOutEvent;

  /// The question about the notes on this phone was answered; null cancels.
  const factory UnlockEvent.bindingAnswered(BindingAnswer? answer) =
      UnlockBindingAnsweredEvent;
}
