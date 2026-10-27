// Kanso — account_task.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What the Account page is doing, while a change goes through.
enum AccountTask {
  /// Signing another device out.
  revokeDevice,

  /// Signing this phone out.
  signOut,

  /// Changing the password.
  changePassword,

  /// Making a new recovery key.
  newRecoveryKey,

  /// Deleting the account.
  deleteAccount,

  /// Adding a passkey.
  addPasskey,

  /// Removing a passkey.
  removePasskey,

  /// Linking Apple, Google or GitHub.
  linkProvider,

  /// Unlinking Apple, Google or GitHub.
  unlinkProvider,
}
