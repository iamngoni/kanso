// Kanso — form_problem.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What is wrong with what was typed in an account form, found on the phone
/// before anything is sent. The words are `FormProblemText`'s.
enum FormProblem {
  /// A username that breaks the rules.
  usernameInvalid,

  /// A valid username nobody may sign up with.
  usernameReserved,

  /// A password under 12 characters.
  passwordTooShort,

  /// A password over 256 characters.
  passwordTooLong,

  /// A password that is the username.
  passwordSameAsUsername,

  /// A new password and its repeat that differ.
  passwordsDiffer,

  /// No password was typed, or one no account can have: answered like a
  /// wrong one.
  credentialsDontMatch,

  /// No recovery key was typed.
  recoveryKeyMissing,
}
