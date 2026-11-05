// Kanso — interrupted_lock_event.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'interrupted_lock_bloc.dart';

@freezed
sealed class InterruptedLockEvent with _$InterruptedLockEvent {
  /// The note [noteId] is shown (or changed): look whether a lock stopped on
  /// it.
  const factory InterruptedLockEvent.checked(String noteId) =
      InterruptedLockCheckedEvent;

  /// The person chose Finish locking, with [password] when it was asked.
  const factory InterruptedLockEvent.finished({String? password}) =
      InterruptedLockFinishedEvent;

  /// The person chose Undo locking, with [password] when it was asked.
  const factory InterruptedLockEvent.undone({String? password}) =
      InterruptedLockUndoneEvent;

  /// The lock password sheet was closed without a password.
  const factory InterruptedLockEvent.passwordDeclined() =
      InterruptedLockPasswordDeclinedEvent;

  /// Finishing got as far as [progress].
  const factory InterruptedLockEvent.progressed(SealProgress progress) =
      InterruptedLockProgressedEvent;

  /// The one-time notice was shown.
  const factory InterruptedLockEvent.noticeShown() =
      InterruptedLockNoticeShownEvent;
}
