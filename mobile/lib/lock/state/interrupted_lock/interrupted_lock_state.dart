// Kanso — interrupted_lock_state.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'interrupted_lock_bloc.dart';

/// What the bar above a note says about a lock that stopped.
enum InterruptedStage {
  /// Looking whether a lock stopped on the note.
  checking,

  /// Nothing stopped: no bar.
  none,

  /// A lock stopped: the bar offers Finish locking and Undo locking.
  stopped,

  /// Finishing or undoing it.
  working,

  /// The session holds no key for it: the lock password is asked.
  needsPassword,

  /// The typed lock password was wrong.
  wrongPassword,

  /// It failed; see the state's error.
  failed,
}

/// Which of the two the person chose.
enum InterruptedAction { finish, undo }

@freezed
abstract class InterruptedLockState with _$InterruptedLockState {
  const factory InterruptedLockState({
    @Default('') String noteId,
    @Default(InterruptedStage.checking) InterruptedStage stage,

    /// How many pictures and files the stopped lock covers, and how many are
    /// sealed.
    @Default(0) int files,
    @Default(0) int sealed,

    /// What the person chose, while it is asked for the password.
    InterruptedAction? action,

    /// How far finishing has come.
    SealProgress? progress,

    /// Whether to say, once, that notes locked before this update seal their
    /// pictures and files.
    @Default(false) bool filesNotice,
    ApplicationError? error,
  }) = _InterruptedLockState;
}
