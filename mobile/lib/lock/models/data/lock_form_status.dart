// Kanso — lock_form_status.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where a form that locks, or changes the lock password, is.
enum LockFormStatus {
  /// Reading what the form needs.
  loading,

  /// Waiting for the person.
  ready,

  /// Working on it.
  working,

  /// Done.
  done,

  /// The two typed passwords differ.
  mismatch,

  /// The new password is too short.
  tooShort,

  /// The lock password typed is not the lock password.
  wrongPassword,

  /// It failed; see the state's error.
  failed,
}
